use crate::error::{AppError, AppResult};
use image::{imageops::FilterType, GrayImage, Luma, RgbaImage};
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Value;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Lazy global session — IS-Net inference is stateless, so a single shared
/// session is fine and saves the ~300 ms load cost per call.
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

/// IS-Net "general use" expects a 1024² square input. That's the whole point of
/// switching off u2netp: 10× more pixels through the network, dramatically
/// sharper mask edges, far less halo around hair and detail.
const INPUT_SIZE: u32 = 1024;

/// IS-Net normalization: divide by image max, then subtract 0.5. No ImageNet
/// per-channel mean/std (that's u2net's recipe — different model, different
/// preprocessing).
const BIAS: f32 = 0.5;

fn session(model_path: &Path) -> AppResult<&'static Mutex<Session>> {
    if let Some(s) = SESSION.get() {
        return Ok(s);
    }
    let bytes = std::fs::read(model_path)?;
    let session = Session::builder()
        .map_err(ort_err)?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(ort_err)?
        .commit_from_memory(&bytes)
        .map_err(ort_err)?;
    let _ = SESSION.set(Mutex::new(session));
    Ok(SESSION.get().expect("set just now"))
}

/// Auto background removal via the bundled IS-Net matte model.
/// Returns a single-channel alpha matte at the original image's dimensions.
///
/// Pipeline mirrors rembg's IS-Net session:
///   1. Resize source to 1024² with Lanczos3 (preserves edge detail)
///   2. Divide by per-image max, subtract 0.5 → roughly [-0.5, +0.5]
///   3. NCHW float32 input
///   4. Min-max normalize the network output to [0, 1]
///   5. Apply a sigmoid alpha-sharpening curve — pushes confident foreground
///      to 1 and confident background to 0 while keeping the soft transition
///      around hair and detail (this is what kills the halo)
///   6. Lanczos3 resize the 1024² mask back to source dims
pub fn run_auto(source: &RgbaImage, model_path: &Path) -> AppResult<GrayImage> {
    if !model_path.exists() {
        return Err(AppError::Msg(format!(
            "model file missing at {}. Run scripts/fetch-models.ps1 (or .sh) to download it.",
            model_path.display()
        )));
    }

    let (orig_w, orig_h) = (source.width(), source.height());

    let resized = image::imageops::resize(source, INPUT_SIZE, INPUT_SIZE, FilterType::Lanczos3);
    let size = INPUT_SIZE as usize;
    let mut input = Array4::<f32>::zeros((1, 3, size, size));
    let mut max_v: f32 = 1e-5;
    for px in resized.pixels() {
        for c in 0..3 {
            let v = px[c] as f32;
            if v > max_v { max_v = v; }
        }
    }
    for (y, row) in resized.rows().enumerate() {
        for (x, px) in row.enumerate() {
            for c in 0..3 {
                let v = (px[c] as f32) / max_v;
                input[[0, c, y, x]] = v - BIAS;
            }
        }
    }

    let input_value = Value::from_array(input).map_err(ort_err)?;

    let mtx = session(model_path)?;
    let mut sess = mtx
        .lock()
        .map_err(|e| AppError::Msg(format!("session lock poisoned: {e}")))?;

    // Don't hardcode the input name — read it from the model metadata so we
    // don't break the moment we swap a model whose export used a different
    // convention.
    let input_name = sess
        .inputs()
        .first()
        .map(|i| i.name().to_string())
        .ok_or_else(|| AppError::Msg("model exposes no inputs".into()))?;

    let outputs = sess
        .run(ort::inputs![input_name.as_str() => input_value])
        .map_err(ort_err)?;

    let (_shape, out_slice) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(ort_err)?;

    // Min-max normalize to [0, 1].
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &v in out_slice.iter() {
        if v < lo { lo = v; }
        if v > hi { hi = v; }
    }
    let range = (hi - lo).max(1e-5);

    let mut mask_1024 = GrayImage::new(INPUT_SIZE, INPUT_SIZE);
    for (idx, &v) in out_slice.iter().enumerate() {
        let y = (idx / size) as u32;
        let x = (idx % size) as u32;
        let n = ((v - lo) / range).clamp(0.0, 1.0);
        let sharp = sharpen_alpha(n);
        mask_1024.put_pixel(x, y, Luma([(sharp * 255.0).round() as u8]));
    }

    Ok(image::imageops::resize(
        &mask_1024,
        orig_w,
        orig_h,
        FilterType::Lanczos3,
    ))
}

/// Sigmoid centered at 0.5. RMBG-1.4 already produces clean alpha, so we use
/// a gentler slope than we would for u2net family models — too aggressive a
/// curve flattens hair into a hard cutout, which is exactly the artifact we
/// want to avoid. Slope 6 trims faint background bleed (α < 0.1) without
/// touching the 0.3..0.7 band where real translucent detail lives.
fn sharpen_alpha(v: f32) -> f32 {
    const SLOPE: f32 = 6.0;
    let z = (v - 0.5) * SLOPE;
    1.0 / (1.0 + (-z).exp())
}

fn ort_err<C>(e: ort::Error<C>) -> AppError {
    AppError::Msg(format!("onnx runtime: {e}"))
}
