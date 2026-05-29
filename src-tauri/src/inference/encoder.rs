use crate::error::{AppError, AppResult};
use image::RgbaImage;

/// Run the SAM image encoder once per loaded image and return the embedding.
/// Cached in `Document::embedding`, then reused by `decoder::run` for every click
/// — that's the encoder-once / decoder-per-click split (CLAUDE.md §3.1).
///
/// Phase 2: wire to MobileSAM encoder ONNX. For now we return None so load_image
/// stays fast and Auto mode keeps working.
#[allow(dead_code)]
pub fn encode(_img: &RgbaImage) -> AppResult<Vec<f32>> {
    Err(AppError::Msg("Smart Select coming in Phase 2".into()))
}
