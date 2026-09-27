import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type AudioLevel = {
    percent: number;
    detected: boolean;
};

export type TranscriptSegment = {
    start_ms: number;
    end_ms: number;
    text: string;
};

const browserModeMessage =
    "PC Caption AI is running in a browser preview. Run `npm run tauri dev` to capture Windows audio.";

const isTauriRuntime = () =>
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const startAudioCapture = () => {
    if (!isTauriRuntime()) return Promise.reject(new Error(browserModeMessage));
    return invoke<void>("start_audio_capture");
};

export const stopAudioCapture = () => {
    if (!isTauriRuntime()) return Promise.reject(new Error(browserModeMessage));
    return invoke<void>("stop_audio_capture");
};

export const pauseAudioCapture = () => {
    if (!isTauriRuntime()) return Promise.reject(new Error(browserModeMessage));
    return invoke<void>("pause_audio_capture");
};

export const resumeAudioCapture = () => {
    if (!isTauriRuntime()) return Promise.reject(new Error(browserModeMessage));
    return invoke<void>("resume_audio_capture");
};

export const listenForAudioLevel = (
    callback: (level: AudioLevel) => void,
): Promise<UnlistenFn> => {
    if (!isTauriRuntime()) return Promise.resolve(() => undefined);
    return listen<AudioLevel>("audio-level", (event) =>
        callback(event.payload),
    );
};

export const listenForAudioError = (
    callback: (message: string) => void,
): Promise<UnlistenFn> => {
    if (!isTauriRuntime()) return Promise.resolve(() => undefined);
    return listen<string>("audio-error", (event) => callback(event.payload));
};

export const listenForTranscript = (
    callback: (segment: TranscriptSegment) => void,
): Promise<UnlistenFn> => {
    if (!isTauriRuntime()) return Promise.resolve(() => undefined);
    return listen<TranscriptSegment>("transcript-segment", (event) =>
        callback(event.payload),
    );
};
