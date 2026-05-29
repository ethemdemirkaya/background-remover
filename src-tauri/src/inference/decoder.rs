use crate::commands::Prompt;
use crate::error::{AppError, AppResult};
use image::{GrayImage, RgbaImage};

/// SAM decoder. Takes the cached embedding (from `encoder::encode`) plus the user's
/// point/box prompts and produces a single-channel mask the same size as `source`.
/// Designed to run in milliseconds — that's what makes Smart Select feel live.
pub fn run(_source: &RgbaImage, _embedding: &[f32], _prompts: &[Prompt]) -> AppResult<GrayImage> {
    Err(AppError::Msg("Smart Select coming in Phase 2".into()))
}
