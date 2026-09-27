# PC Caption AI

PC Caption AI is a small Windows desktop application for capturing system playback audio, transcribing it locally, and summarizing the finished transcript.

## Current phase

Phase 3 is complete: the app captures the default Windows playback mix through WASAPI loopback, resamples it to 16 kHz mono, runs local whisper.cpp transcription, and displays timestamped transcript segments. The transcript is not saved yet.

Planned implementation order:

1. Pause, resume, stop, and periodic transcript saves
2. OpenAI transcript summarization
3. TXT, Markdown, and SRT export

## Development

Install Rust with `rustup`, Visual Studio Build Tools with the MSVC and Windows 10/11 SDK components, LLVM, and CMake. The launcher configures the standard installation paths automatically. Then run:

```powershell
npm install
npm run tauri dev
```

Frontend-only checks:

```powershell
npm run build
npm run lint
```

The application will never upload raw audio. The planned OpenAI integration will receive only the final transcript after the user requests a summary.

## Whisper model

Download the English base model `ggml-base.en.bin` from the whisper.cpp model releases and place it at `models/ggml-base.en.bin`. The application reports a readable error if the model is missing; it never substitutes fake captions.

## Phase 2 manual test

With audio playing in YouTube, VLC, or another Windows application:

1. Run `npm run tauri dev`.
2. Click `Start`.
3. Confirm the audio meter moves and the signal label reports detected audio.
4. Click `Stop & summarize` and confirm the meter returns to zero.

## Phase 3 manual test

1. Place `ggml-base.en.bin` at `models/ggml-base.en.bin`.
2. Play clear English speech in YouTube or VLC.
3. Click `Start` and wait for a few seconds of speech.
4. Confirm timestamped segments appear in the transcript panel.
