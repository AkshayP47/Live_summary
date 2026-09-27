use tauri::{AppHandle, State};

use crate::audio::AudioCapture;

#[tauri::command]
pub fn start_audio_capture(app: AppHandle, capture: State<'_, AudioCapture>) -> Result<(), String> {
    let model_path = default_model_path()?;
    capture.start(app, model_path)
}

#[tauri::command]
pub fn stop_audio_capture(capture: State<'_, AudioCapture>) -> Result<(), String> {
    capture.stop()
}

fn default_model_path() -> Result<String, String> {
    let path = std::env::current_dir()
        .map_err(|_| "Unable to locate the application folder.".to_string())?
        .join("models")
        .join("ggml-base.en.bin");
    Ok(path.to_string_lossy().into_owned())
}
