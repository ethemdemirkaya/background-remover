use dashmap::DashMap;
use image::RgbaImage;
use std::sync::Arc;

/// One open document: the decoded source, plus an optional SAM image embedding
/// that the encoder produced on load (CLAUDE.md §3.1 — encode once, decode per click).
pub struct Document {
    pub source: RgbaImage,
    /// SAM encoder output, populated by `inference::encoder::encode` once Phase 2 lands.
    #[allow(dead_code)]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Default)]
pub struct AppState {
    pub documents: Arc<DashMap<String, Document>>,
}
