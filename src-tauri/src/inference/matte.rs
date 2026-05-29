use crate::error::{AppError, AppResult};
use image::{imageops::FilterType, GrayImage, Luma, RgbaImage};
use ndarray::Array4;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Value;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Lazy global session — U-2-Net family models are stateless under inference,
/// so a single shared session is fine and saves the ~150 ms load cost per call.
/// Wrapped in a Mutex because ort sessions aren't Sync.
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();

const INPUT_SIZE: u32 = 320;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

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

/// Auto background removal via the bundled u2netp ONNX matte model.
/// Returns a single-channel alpha matte at the original image's dimensions.
///
/// Pipeline mirrors rembg's reference implementation:
///   1. Resize source to 320² (Triangle)
///   2. Per-channel ImageNet normalization
///   3. NCHW float32 input
///   4. Min-max normalize the network output
///   5. Resize the 320² mask back to source dims
pub fn run_auto(source: &RgbaImage, model_path: &Path) -> AppResult<GrayImage> {
    if !model_path.exists() {
        return Err(AppError::Msg(format!(
            "model file missing at {}. Run scripts/fetch-models.ps1 (or .sh) to download it.",
            model_path.display()
        )));
    }

    let (orig_w, orig_h) = (source.width(), source.height());

    let resized = image::imageops::resize(source, INPUT_SIZE, INPUT_SIZE, FilterType::Triangle);
    let size = INPUT_SIZE as usize;
    let mut input = Array4::<f32>::zeros((1, 3, size, size));
    let mut max_v: f32 = 1e-5;
    // First pass: collect max so we can match rembg's "divide by max" pre-normalization.
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
                input[[0, c, y, x]] = (v - MEAN[c]) / STD[c];
            }
        }
    }

    // ort rc.12: feed via Value::from_array on an owned ndarray.
    let input_value = Value::from_array(input).map_err(ort_err)?;

    let mtx = session(model_path)?;
    let mut sess = mtx
        .lock()
        .map_err(|e| AppError::Msg(format!("session lock poisoned: {e}")))?;

    // Don't hardcode the input name — u2net family models typically use "input.1"
    // but some exports use "input" or "img". Read it from the model metadata.
    let input_name = sess
        .inputs()
        .first()
        .map(|i| i.name().to_string())
        .ok_or_else(|| AppError::Msg("model exposes no inputs".into()))?;

    let outputs = sess
        .run(ort::inputs![input_name.as_str() => input_value])
        .map_err(ort_err)?;

    // Output is (1, 1, 320, 320) f32. rc.12's try_extract_tensor returns (&Shape, &[f32]).
    let (_shape, out_slice) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(ort_err)?;

    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &v in out_slice.iter() {
        if v < lo { lo = v; }
        if v > hi { hi = v; }
    }
    let range = (hi - lo).max(1e-5);

    let mut mask_320 = GrayImage::new(INPUT_SIZE, INPUT_SIZE);
    for (idx, &v) in out_slice.iter().enumerate() {
        let y = (idx / size) as u32;
        let x = (idx % size) as u32;
        let n = ((v - lo) / range).clamp(0.0, 1.0);
        mask_320.put_pixel(x, y, Luma([(n * 255.0).round() as u8]));
    }

    Ok(image::imageops::resize(
        &mask_320,
        orig_w,
        orig_h,
        FilterType::Triangle,
    ))
}

fn ort_err<C>(e: ort::Error<C>) -> AppError {
    AppError::Msg(format!("onnx runtime: {e}"))
}
