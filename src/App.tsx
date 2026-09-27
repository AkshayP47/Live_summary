import { useEffect, useRef, useState } from "react";
import "./App.css";
import {
    listenForAudioError,
    listenForAudioLevel,
    listenForTranscript,
    pauseAudioCapture,
    resumeAudioCapture,
    startAudioCapture,
    stopAudioCapture,
} from "./services/tauri";
import type { TranscriptSegment } from "./services/tauri";

function App() {
    const [status, setStatus] = useState<"ready" | "listening" | "paused">(
        "ready",
    );
    const [elapsedSeconds, setElapsedSeconds] = useState(0);
    const [audioLevel, setAudioLevel] = useState(0);
    const [audioDetected, setAudioDetected] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [transcript, setTranscript] = useState<TranscriptSegment[]>([]);
    const statusRef = useRef(status);

    useEffect(() => {
        statusRef.current = status;
    }, [status]);

    useEffect(() => {
        let removeLevelListener: (() => void) | undefined;
        let removeErrorListener: (() => void) | undefined;
        let removeTranscriptListener: (() => void) | undefined;

        void listenForAudioLevel((level) => {
            // Freeze the meter while paused; the capture thread keeps running.
            if (statusRef.current !== "listening") return;
            setAudioLevel(level.percent);
            setAudioDetected(level.detected);
        })
            .then((remove) => {
                removeLevelListener = remove;
            })
            .catch(() => undefined);
        void listenForAudioError((message) => {
            setError(message);
            setStatus("ready");
        })
            .then((remove) => {
                removeErrorListener = remove;
            })
            .catch(() => undefined);
        void listenForTranscript((segment) => {
            setTranscript((current) => {
                const duplicate = current.some(
                    (item) =>
                        item.start_ms === segment.start_ms &&
                        item.text === segment.text,
                );
                return duplicate ? current : [...current, segment];
            });
        })
            .then((remove) => {
                removeTranscriptListener = remove;
            })
            .catch(() => undefined);

        return () => {
            removeLevelListener?.();
            removeErrorListener?.();
            removeTranscriptListener?.();
        };
    }, []);

    useEffect(() => {
        if (status !== "listening") return;

        const timer = window.setInterval(() => {
            setElapsedSeconds((seconds) => seconds + 1);
        }, 1000);

        return () => window.clearInterval(timer);
    }, [status]);

    const elapsed = `${String(Math.floor(elapsedSeconds / 60)).padStart(2, "0")}:${String(
        elapsedSeconds % 60,
    ).padStart(2, "0")}`;

    const statusLabel = {
        ready: "Ready to capture",
        listening: "Listening",
        paused: "Paused",
    }[status];

    return (
        <main className="app-shell">
            <header className="app-header">
                <div className="brand-mark" aria-hidden="true">
                    PC
                </div>
                <div>
                    <p className="eyebrow">Local transcription workspace</p>
                    <h1>PC Caption AI</h1>
                </div>
                <button
                    className="settings-button"
                    type="button"
                    aria-label="Open settings"
                    title="Settings"
                >
                    <span aria-hidden="true">*</span>
                </button>
            </header>

            <section className="capture-panel" aria-label="Capture status">
                <div className={`status-dot ${status}`} aria-hidden="true" />
                <div className="status-copy">
                    <span className="status-label">{statusLabel}</span>
                    <span className="status-detail">
                        Windows playback audio
                    </span>
                </div>
                <time className="elapsed" dateTime={`PT${elapsedSeconds}S`}>
                    {elapsed}
                </time>
            </section>

            {error !== null && (
                <p className="error-banner" role="alert">
                    {error}
                </p>
            )}

            <section className="transcript-panel" aria-label="Live transcript">
                <div className="panel-heading">
                    <span>Transcript</span>
                    <span className="local-badge">LOCAL</span>
                </div>
                <div
                    className={
                        transcript.length > 0
                            ? "transcript-list"
                            : "transcript-empty"
                    }
                >
                    {transcript.length > 0 ? (
                        transcript.map((segment) => (
                            <article
                                className="transcript-segment"
                                key={`${segment.start_ms}-${segment.text}`}
                            >
                                <time>{formatTimestamp(segment.start_ms)}</time>
                                <p>{segment.text}</p>
                            </article>
                        ))
                    ) : (
                        <>
                            <div className="waveform" aria-hidden="true">
                                <i />
                                <i />
                                <i />
                                <i />
                                <i />
                                <i />
                                <i />
                            </div>
                            <h2>Your captions will appear here</h2>
                            <p>
                                Start a session to detect the audio playing on
                                this PC.
                            </p>
                        </>
                    )}
                </div>
            </section>

            <section className="signal-row" aria-label="Audio signal">
                <div className="signal-heading">
                    <span>Audio signal</span>
                    <span className="signal-value">
                        {audioDetected
                            ? `${audioLevel}% detected`
                            : "No audio detected"}
                    </span>
                </div>
                <div className="meter" aria-hidden="true">
                    <span
                        className={
                            status === "listening"
                                ? "meter-fill active"
                                : "meter-fill"
                        }
                        style={{ width: `${audioLevel}%` }}
                    />
                </div>
            </section>

            <footer className="controls">
                <button
                    className="button primary"
                    type="button"
                    onClick={() => {
                        setError(null);
                        setTranscript([]);
                        setElapsedSeconds(0);
                        setAudioLevel(0);
                        setAudioDetected(false);
                        void startAudioCapture()
                            .then(() => {
                                setStatus("listening");
                            })
                            .catch((message: unknown) =>
                                setError(
                                    message instanceof Error
                                        ? message.message
                                        : String(message),
                                ),
                            );
                    }}
                    disabled={status !== "ready"}
                >
                    <span
                        className="button-icon start-icon"
                        aria-hidden="true"
                    />
                    Start
                </button>
                {status === "paused" ? (
                    <button
                        className="button secondary"
                        type="button"
                        onClick={() => {
                            void resumeAudioCapture()
                                .then(() => setStatus("listening"))
                                .catch((message: unknown) =>
                                    setError(
                                        message instanceof Error
                                            ? message.message
                                            : String(message),
                                    ),
                                );
                        }}
                    >
                        <span
                            className="button-icon start-icon"
                            aria-hidden="true"
                        />
                        Resume
                    </button>
                ) : (
                    <button
                        className="button secondary"
                        type="button"
                        onClick={() => {
                            void pauseAudioCapture()
                                .then(() => {
                                    setStatus("paused");
                                    setAudioLevel(0);
                                    setAudioDetected(false);
                                })
                                .catch((message: unknown) =>
                                    setError(
                                        message instanceof Error
                                            ? message.message
                                            : String(message),
                                    ),
                                );
                        }}
                        disabled={status !== "listening"}
                    >
                        <span
                            className="button-icon pause-icon"
                            aria-hidden="true"
                        />
                        Pause
                    </button>
                )}
                <button
                    className="button stop"
                    type="button"
                    onClick={() => {
                        void stopAudioCapture()
                            .then(() => {
                                setStatus("ready");
                                setElapsedSeconds(0);
                                setAudioLevel(0);
                                setAudioDetected(false);
                            })
                            .catch((message: unknown) =>
                                setError(
                                    message instanceof Error
                                        ? message.message
                                        : String(message),
                                ),
                            );
                    }}
                    disabled={status === "ready"}
                >
                    <span
                        className="button-icon stop-icon"
                        aria-hidden="true"
                    />
                    Stop &amp; summarize
                </button>
            </footer>
            <p className="privacy-note">
                Audio stays on this computer. AI summary comes after you stop.
            </p>
        </main>
    );
}

export default App;

function formatTimestamp(milliseconds: number) {
    const totalSeconds = Math.floor(milliseconds / 1000);
    return `${String(Math.floor(totalSeconds / 60)).padStart(2, "0")}:${String(totalSeconds % 60).padStart(2, "0")}`;
}
