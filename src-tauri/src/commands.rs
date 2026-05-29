use crate::compose;
use crate::error::{AppError, AppResult};
use crate::inference;
use crate::state::{AppState, Document};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

const MATTE_MODEL: &str = "u2netp.onnx";

/// Resolve a bundled resource path. In a release build this comes from the
/// bundle; in `tauri dev` it falls back to `src-tauri/resources/<name>` relative
/// to the running executable.
fn resource_path(app: &AppHandle, rel: &str) -> AppResult<PathBuf> {
    if let Ok(p) = app
        .path()
        .resolve(format!("resources/{rel}"), BaseDirectory::Resource)
    {
        if p.exists() { return Ok(p); }
    }
    if let Ok(exe) = std::env::current_exe() {
        // target/<profile>/exe → src-tauri/resources/<rel>
        if let Some(src_tauri) = exe.ancestors().nth(3) {
            let candidate = src_tauri.join("resources").join(rel);
            if candidate.exists() { return Ok(candidate); }
        }
    }
    Err(AppError::Msg(format!("resource not found: {rel}")))
}

#[derive(Serialize)]
pub struct ImageMeta {
    pub image_id: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[allow(dead_code)] // fields consumed once the SAM decoder is wired in Phase 2
pub enum Prompt {
    Point { x: f32, y: f32, label: PromptLabel },
    Box { x0: f32, y0: f32, x1: f32, y1: f32, label: PromptLabel },
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PromptLabel { Add, Remove }

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Background {
    Transparent,
    Color { hex: String },
    Image { path: String },
    Blur { radius: u32 },
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat { Png, Webp }

#[tauri::command]
pub async fn load_image(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<ImageMeta> {
    let docs = state.documents.clone();
    let path = PathBuf::from(path);

    // Heavy work off the async runtime. CLAUDE.md §10: never block the runtime with CPU.
    let (id, meta) = tauri::async_runtime::spawn_blocking(move || -> AppResult<(String, ImageMeta)> {
        let img = image::open(&path)?;
        let (w, h) = img.dimensions();
        let rgba = img.to_rgba8();
        let id = Uuid::new_v4().to_string();

        // SAM encoder lands in Phase 2 — for now we don't precompute an embedding.
        // Auto mode runs RMBG on demand, which is fast enough that lazy is fine.
        docs.insert(id.clone(), Document { source: rgba, embedding: None });
        Ok((id.clone(), ImageMeta { image_id: id, width: w, height: h }))
    })
    .await
    .map_err(|e| AppError::Msg(e.to_string()))??;

    let _ = id;
    Ok(meta)
}

#[tauri::command]
pub async fn auto_remove(
    app: AppHandle,
    state: State<'_, AppState>,
    image_id: String,
) -> AppResult<Vec<u8>> {
    let docs = state.documents.clone();
    let model = resource_path(&app, &format!("models/{MATTE_MODEL}"))?;
    tauri::async_runtime::spawn_blocking(move || -> AppResult<Vec<u8>> {
        let doc = docs.get(&image_id).ok_or_else(|| AppError::UnknownImage(image_id.clone()))?;
        let mask = inference::matte::run_auto(&doc.source, model.as_path() as &Path)?;
        compose::encode_mask_png(&mask)
    })
    .await
    .map_err(|e| AppError::Msg(e.to_string()))?
}

#[tauri::command]
pub async fn smart_select(
    state: State<'_, AppState>,
    image_id: String,
    prompts: Vec<Prompt>,
) -> AppResult<Vec<u8>> {
    let docs = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || -> AppResult<Vec<u8>> {
        let doc = docs.get(&image_id).ok_or_else(|| AppError::UnknownImage(image_id.clone()))?;
        let empty = Vec::new();
        let embedding = doc.embedding.as_deref().unwrap_or(&empty);
        let mask = inference::decoder::run(&doc.source, embedding, &prompts)?;
        compose::encode_mask_png(&mask)
    })
    .await
    .map_err(|e| AppError::Msg(e.to_string()))?
}

#[tauri::command]
pub async fn export_image(
    state: State<'_, AppState>,
    image_id: String,
    mask: Vec<u8>,
    background: Background,
    format: ExportFormat,
    save_path: Option<String>,
) -> AppResult<Option<Vec<u8>>> {
    let docs = state.documents.clone();
    tauri::async_runtime::spawn_blocking(move || -> AppResult<Option<Vec<u8>>> {
        let doc = docs.get(&image_id).ok_or_else(|| AppError::UnknownImage(image_id.clone()))?;
        let bytes = compose::apply_and_encode(&doc.source, &mask, &background, format)?;
        if let Some(p) = save_path {
            std::fs::write(&p, &bytes)?;
            Ok(None)
        } else {
            Ok(Some(bytes))
        }
    })
    .await
    .map_err(|e| AppError::Msg(e.to_string()))?
}

#[tauri::command]
pub async fn clear_image(state: State<'_, AppState>, image_id: String) -> AppResult<()> {
    state.documents.remove(&image_id);
    Ok(())
}
