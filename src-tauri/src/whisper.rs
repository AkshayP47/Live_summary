use serde::Serialize;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const SAMPLE_RATE: usize = 16_000;

#[derive(Debug, Serialize, Clone)]
pub struct TranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct WhisperTranscriber {
    context: WhisperContext,
}

impl WhisperTranscriber {
    pub fn new(model_path: &str) -> Result<Self, String> {
        if !std::path::Path::new(model_path).is_file() {
            return Err(format!(
                "Whisper model not found. Download ggml-base.en.bin into {}.",
                model_path
            ));
        }

        let context = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .map_err(|_| "Unable to load the Whisper model.".to_string())?;

        Ok(Self { context })
    }

    pub fn transcribe(
        &mut self,
        samples: &[f32],
        offset_ms: u64,
    ) -> Result<Vec<TranscriptSegment>, String> {
        if samples.len() < SAMPLE_RATE / 2 {
            return Ok(Vec::new());
        }

        let mut state = self
            .context
            .create_state()
            .map_err(|_| "Unable to create a Whisper transcription state.".to_string())?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
        params.set_n_threads(2);
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, samples)
            .map_err(|_| "Whisper could not process the audio chunk.".to_string())?;

        let mut segments = Vec::new();
        for segment in state.as_iter() {
            let text = segment.to_string().trim().to_string();
            if text.is_empty() {
                continue;
            }
            segments.push(TranscriptSegment {
                start_ms: offset_ms + (segment.start_timestamp().max(0) as u64 * 10),
                end_ms: offset_ms + (segment.end_timestamp().max(0) as u64 * 10),
                text,
            });
        }

        Ok(segments)
    }
}
