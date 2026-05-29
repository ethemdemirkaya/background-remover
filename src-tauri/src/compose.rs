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

/// Apply the mask as alpha to the source and composite the chosen background.
/// Returns the final encoded bytes in the requested format.
pub fn apply_and_encode(
    source: &RgbaImage,
    mask_png: &[u8],
    background: &Background,
    format: ExportFormat,
) -> AppResult<Vec<u8>> {
    let mask = image::load_from_memory(mask_png)?.to_luma8();
    if mask.dimensions() != source.dimensions() {
        return Err(AppError::Msg(format!(
            "mask dims {:?} != source dims {:?}",
            mask.dimensions(),
            source.dimensions()
        )));
    }

    let mut out: RgbaImage = ImageBuffer::new(source.width(), source.height());

    match background {
        Background::Transparent => {
            for (out_px, (src_px, m_px)) in out
                .pixels_mut()
                .zip(source.pixels().zip(mask.pixels()))
            {
                *out_px = Rgba([src_px[0], src_px[1], src_px[2], m_px[0]]);
            }
        }
        Background::Color { hex } => {
            let [r, g, b] = parse_hex(hex)?;
            for (out_px, (src_px, m_px)) in out
                .pixels_mut()
                .zip(source.pixels().zip(mask.pixels()))
            {
                let a = m_px[0] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((src_px[0] as u32 * a + r as u32 * inv) / 255) as u8,
                    ((src_px[1] as u32 * a + g as u32 * inv) / 255) as u8,
                    ((src_px[2] as u32 * a + b as u32 * inv) / 255) as u8,
                    255,
                ]);
            }
        }
        Background::Blur { radius } => {
            // Cheap box blur via the `image` crate's gaussian; quality > speed here is fine.
            let blurred = image::imageops::blur(source, (*radius as f32).max(1.0));
            for (out_px, ((src_px, m_px), bg_px)) in out
                .pixels_mut()
                .zip(source.pixels().zip(mask.pixels()).zip(blurred.pixels()))
            {
                let a = m_px[0] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((src_px[0] as u32 * a + bg_px[0] as u32 * inv) / 255) as u8,
                    ((src_px[1] as u32 * a + bg_px[1] as u32 * inv) / 255) as u8,
                    ((src_px[2] as u32 * a + bg_px[2] as u32 * inv) / 255) as u8,
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
            for (out_px, ((src_px, m_px), bg_px)) in out
                .pixels_mut()
                .zip(source.pixels().zip(mask.pixels()).zip(bg.pixels()))
            {
                let a = m_px[0] as u32;
                let inv = 255 - a;
                *out_px = Rgba([
                    ((src_px[0] as u32 * a + bg_px[0] as u32 * inv) / 255) as u8,
                    ((src_px[1] as u32 * a + bg_px[1] as u32 * inv) / 255) as u8,
                    ((src_px[2] as u32 * a + bg_px[2] as u32 * inv) / 255) as u8,
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
