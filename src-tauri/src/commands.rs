use tauri::{AppHandle, Manager, State};

use crate::audio::AudioCapture;

#[tauri::command]
pub fn start_audio_capture(app: AppHandle, capture: State<'_, AudioCapture>) -> Result<(), String> {
    let model_path = default_model_path(&app)?;
    capture.start(app, model_path)
}

#[tauri::command]
pub fn stop_audio_capture(capture: State<'_, AudioCapture>) -> Result<(), String> {
    capture.stop()
}

#[tauri::command]
pub fn pause_audio_capture(capture: State<'_, AudioCapture>) -> Result<(), String> {
    capture.pause()
}

#[tauri::command]
pub fn resume_audio_capture(capture: State<'_, AudioCapture>) -> Result<(), String> {
    capture.resume()
}

fn default_model_path(app: &AppHandle) -> Result<String, String> {
    const RELATIVE: &str = "models/ggml-base.en.bin";
    let mut checked: Vec<String> = Vec::new();
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    // Tauri resource dir (bundled app) takes priority when available.
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("ggml-base.en.bin"));
        candidates.push(resource_dir.join("models").join("ggml-base.en.bin"));
    }

    // Current working directory. Note: under `tauri dev` this is often
    // src-tauri, so also check its parent (the project root).
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current.join(RELATIVE));
        if let Some(parent) = current.parent() {
            candidates.push(parent.join(RELATIVE));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(RELATIVE));
            candidates.push(dir.join("resources").join(RELATIVE));
            // Debug layout: src-tauri/target/debug/app.exe -> project root is
            // two levels up from target/debug.
            if let Some(root) = dir.parent().and_then(|p| p.parent()) {
                candidates.push(root.join(RELATIVE));
            }
        }
    }

    for candidate in candidates {
        checked.push(candidate.to_string_lossy().into_owned());
        if candidate.is_file() {
            return Ok(candidate.to_string_lossy().into_owned());
        }
    }

    Err(format!(
        "Whisper model not found. Download ggml-base.en.bin from https://huggingface.co/ggerganov/whisper.cpp and place it at {} (or run `npm run download-model`). Checked: {}.",
        std::env::current_dir()
            .map(|d| d.join(RELATIVE).to_string_lossy().into_owned())
            .unwrap_or_else(|_| RELATIVE.to_string()),
        checked.join("; ")
    ))
}
