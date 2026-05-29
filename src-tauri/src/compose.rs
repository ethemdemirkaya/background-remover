use crate::commands::{Background, ExportFormat};
use crate::error::{AppError, AppResult};
use image::{
    codecs::{png::PngEncoder, webp::WebPEncoder},
    GrayImage, ImageBuffer, ImageEncoder, Rgba, RgbaImage,
};
use std::io::Cursor;

/// PNG-encode a single-channel mask so it can be marshalled to the webview cheaply
/// and drawn as an overlay. (CLAUDE.md §3.3 — never giant JSON arrays.)
pub fn encode_mask_png(mask: &GrayImage) -> AppResult<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    PngEncoder::new(&mut buf).write_image(
        mask.as_raw(),
        mask.width(),
        mask.height(),
        image::ExtendedColorType::L8,
    )?;
    Ok(buf.into_inner())
}

pub fn encode_rgba_png(img: &RgbaImage) -> AppResult<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    PngEncoder::new(&mut buf).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(buf.into_inner())
}

/// Build the displayed cutout: source RGB with the mask used as alpha, followed
/// by a foreground color decontamination pass to strip background-color bleed
/// from soft edges. This is what makes hair edges look clean instead of carrying
/// a halo of the original background's color.
pub fn build_cutout(source: &RgbaImage, mask: &GrayImage) -> AppResult<RgbaImage> {
    if mask.dimensions() != source.dimensions() {
        return Err(AppError::Msg(format!(
            "mask dims {:?} != source dims {:?}",
            mask.dimensions(),
            source.dimensions()
        )));
    }
    let (w, h) = source.dimensions();
    let mut out: RgbaImage = ImageBuffer::new(w, h);
    for (out_px, (src_px, m_px)) in out
        .pixels_mut()
        .zip(source.pixels().zip(mask.pixels()))
    {
        *out_px = Rgba([src_px[0], src_px[1], src_px[2], m_px[0]]);
    }
    decontaminate_foreground(&mut out);
    Ok(out)
}

/// Iteratively bleed colors from confidently-opaque pixels into adjacent
/// semi-transparent ones. After a handful of passes, the 0 < α < 0.95 ring
/// around the subject — the band where matting models leave background-color
/// contamination — gets repainted with clean foreground color while keeping
/// its original alpha. End result: no more cyan halo around hair, no more
/// rust-colored fringe around shoulders.
///
/// Tuned defaults: 6 iterations × 8-neighbor pull is enough to reach ~6 px
/// into the soft edge band, which covers every soft transition real-world
/// matting models produce.
fn decontaminate_foreground(img: &mut RgbaImage) {
    const SOLID_THRESHOLD: u8 = 235;
    const BLEED_FLOOR: u8 = 8;
    const ITERATIONS: u32 = 6;

    let (w, h) = img.dimensions();
    let n = (w * h) as usize;

    // Channel buffers — flat arrays let us index by (y * w + x) and skip the
    // image crate's bounds-check overhead in the hot loop.
    let mut r = vec![0u8; n];
    let mut g = vec![0u8; n];
    let mut b = vec![0u8; n];
    let mut a = vec![0u8; n];
    let mut solid = vec![false; n];

    for (i, px) in img.pixels().enumerate() {
        r[i] = px[0]; g[i] = px[1]; b[i] = px[2]; a[i] = px[3];
        solid[i] = px[3] >= SOLID_THRESHOLD;
    }

    let w = w as usize;
    let h = h as usize;

    for _ in 0..ITERATIONS {
        // Snapshot the current state so all neighbor reads in this pass see
        // the same values; writes go into the *next* buffer.
        let r_snap = r.clone();
        let g_snap = g.clone();
        let b_snap = b.clone();
        let solid_snap = solid.clone();

        for y in 0..h {
            let row = y * w;
            for x in 0..w {
                let idx = row + x;
                if solid_snap[idx] { continue; }
                if a[idx] < BLEED_FLOOR { continue; }

                let mut sr: u32 = 0;
                let mut sg: u32 = 0;
                let mut sb: u32 = 0;
                let mut count: u32 = 0;

                let y0 = y.saturating_sub(1);
                let y1 = (y + 1).min(h - 1);
                let x0 = x.saturating_sub(1);
                let x1 = (x + 1).min(w - 1);
                for ny in y0..=y1 {
                    let nrow = ny * w;
                    for nx in x0..=x1 {
                        if nx == x && ny == y { continue; }
                        let ni = nrow + nx;
                        if solid_snap[ni] {
                            sr += r_snap[ni] as u32;
                            sg += g_snap[ni] as u32;
                            sb += b_snap[ni] as u32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    r[idx] = (sr / count) as u8;
                    g[idx] = (sg / count) as u8;
                    b[idx] = (sb / count) as u8;
                    solid[idx] = true;
                }
            }
        }
    }

    // Write cleaned RGB back; alpha is untouched.
    for (i, px) in img.pixels_mut().enumerate() {
        px[0] = r[i];
        px[1] = g[i];
        px[2] = b[i];
        // alpha is already correct (we didn't modify a[])
        px[3] = a[i];
    }
}

/// Composite the cutout (premultiplied foreground RGBA after decontamination)
/// over the requested background and encode the result.
pub fn apply_and_encode(
    source: &RgbaImage,
    mask_png: &[u8],
    background: &Background,
    format: ExportFormat,
) -> AppResult<Vec<u8>> {
    let mask = image::load_from_memory(mask_png)?.to_luma8();
    let cutout = build_cutout(source, &mask)?;

    let (w, h) = source.dimensions();
    let mut out: RgbaImage = ImageBuffer::new(w, h);

    match background {
        Background::Transparent => {
            out.clone_from(&cutout);
        }
        Background::Color { hex } => {
            let [r, g, b] = parse_hex(hex)?;
            for (out_px, fg_px) in out.pixels_mut().zip(cutout.pixels()) {
                let a = fg_px[3] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((fg_px[0] as u32 * a + r as u32 * inv) / 255) as u8,
                    ((fg_px[1] as u32 * a + g as u32 * inv) / 255) as u8,
                    ((fg_px[2] as u32 * a + b as u32 * inv) / 255) as u8,
                    255,
                ]);
            }
        }
        Background::Blur { radius } => {
            let blurred = image::imageops::blur(source, (*radius as f32).max(1.0));
            for (out_px, (fg_px, bg_px)) in
                out.pixels_mut().zip(cutout.pixels().zip(blurred.pixels()))
            {
                let a = fg_px[3] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((fg_px[0] as u32 * a + bg_px[0] as u32 * inv) / 255) as u8,
                    ((fg_px[1] as u32 * a + bg_px[1] as u32 * inv) / 255) as u8,
                    ((fg_px[2] as u32 * a + bg_px[2] as u32 * inv) / 255) as u8,
                    255,
                ]);
            }
        }
        Background::Image { path } => {
            let bg_full = image::open(path)?.to_rgba8();
            let bg = if bg_full.dimensions() == source.dimensions() {
                bg_full
            } else {
                image::imageops::resize(
                    &bg_full,
                    source.width(),
                    source.height(),
                    image::imageops::FilterType::Lanczos3,
                )
            };
            for (out_px, (fg_px, bg_px)) in
                out.pixels_mut().zip(cutout.pixels().zip(bg.pixels()))
            {
                let a = fg_px[3] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((fg_px[0] as u32 * a + bg_px[0] as u32 * inv) / 255) as u8,
                    ((fg_px[1] as u32 * a + bg_px[1] as u32 * inv) / 255) as u8,
                    ((fg_px[2] as u32 * a + bg_px[2] as u32 * inv) / 255) as u8,
                    255,
                ]);
            }
        }
    }

    encode(&out, format)
}

fn encode(img: &RgbaImage, format: ExportFormat) -> AppResult<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    match format {
        ExportFormat::Png => {
            PngEncoder::new(&mut buf).write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
        ExportFormat::Webp => {
            WebPEncoder::new_lossless(&mut buf).write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )?;
        }
    }
    Ok(buf.into_inner())
}

fn parse_hex(hex: &str) -> AppResult<[u8; 3]> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return Err(AppError::Msg(format!("bad hex: {hex}")));
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| AppError::Msg(e.to_string()))?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| AppError::Msg(e.to_string()))?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| AppError::Msg(e.to_string()))?;
    Ok([r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma, Rgba, RgbaImage};

    #[test]
    fn parse_hex_accepts_uppercase_lowercase_and_optional_hash() {
        assert_eq!(parse_hex("#ffffff").unwrap(), [255, 255, 255]);
        assert_eq!(parse_hex("FFFFFF").unwrap(), [255, 255, 255]);
        assert_eq!(parse_hex("  #ff8800  ").unwrap(), [255, 136, 0]);
    }

    #[test]
    fn parse_hex_rejects_bad_length() {
        assert!(parse_hex("#abc").is_err());
        assert!(parse_hex("ffffff00").is_err());
    }

    #[test]
    fn parse_hex_rejects_non_hex_chars() {
        assert!(parse_hex("#zzzzzz").is_err());
    }

    /// build_cutout copies source RGB into the cutout for fully-opaque mask
    /// pixels and zeroes alpha for fully-transparent ones.
    #[test]
    fn build_cutout_maps_mask_luminance_to_alpha() {
        let mut source = RgbaImage::new(2, 1);
        source.put_pixel(0, 0, Rgba([200, 100, 50, 255]));
        source.put_pixel(1, 0, Rgba([10, 20, 30, 255]));

        let mut mask = GrayImage::new(2, 1);
        mask.put_pixel(0, 0, Luma([255])); // fully foreground
        mask.put_pixel(1, 0, Luma([0]));   // fully background

        let out = build_cutout(&source, &mask).unwrap();
        let fg = out.get_pixel(0, 0);
        let bg = out.get_pixel(1, 0);

        assert_eq!(fg[3], 255, "fully-opaque mask should keep full alpha");
        // RGB is preserved for solid pixels; decontamination doesn't touch them.
        assert_eq!([fg[0], fg[1], fg[2]], [200, 100, 50]);
        assert_eq!(bg[3], 0, "fully-transparent mask should zero alpha");
    }

    /// Decontamination doesn't touch the alpha channel — only RGB inside the
    /// soft band. This is the invariant that keeps hair softness intact.
    #[test]
    fn decontamination_preserves_alpha() {
        let mut img = RgbaImage::new(8, 8);
        // 3×3 opaque foreground square in the middle, soft alpha around it.
        for y in 0..8 {
            for x in 0..8 {
                let in_solid = (3..=5).contains(&x) && (3..=5).contains(&y);
                let in_band = !in_solid && (2..=6).contains(&x) && (2..=6).contains(&y);
                let alpha = if in_solid { 255 } else if in_band { 80 } else { 0 };
                img.put_pixel(x, y, Rgba([10, 10, 200, alpha]));
            }
        }
        let before: Vec<u8> = img.pixels().map(|p| p[3]).collect();
        decontaminate_foreground(&mut img);
        let after: Vec<u8> = img.pixels().map(|p| p[3]).collect();
        assert_eq!(before, after, "alpha channel must be untouched by decontamination");
    }
}
