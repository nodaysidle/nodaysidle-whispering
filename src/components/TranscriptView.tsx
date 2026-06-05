import { useEffect, useRef } from "react";

interface TranscriptViewProps {
  finalText: string;
  partialText: string;
  modelLoaded: boolean;
  hotkeyLabel: string;
  wordCount: number;
  characterCount: number;
  onReset: () => void;
  onCopyText: () => void;
  onArchiveDraft: () => void;
}

export function TranscriptView({
  finalText,
  partialText,
  modelLoaded,
  hotkeyLabel,
  wordCount,
  characterCount,
  onReset,
  onCopyText,
  onArchiveDraft,
}: TranscriptViewProps) {
  const transcriptRef = useRef<HTMLDivElement>(null);
  const isTranscriptEmpty = finalText.length === 0 && partialText.length === 0;

  useEffect(() => {
    const transcript = transcriptRef.current;
    if (!transcript) return;

    const distanceFromBottom =
      transcript.scrollHeight - transcript.scrollTop - transcript.clientHeight;
    if (distanceFromBottom < 120) {
      transcript.scrollTo({ top: transcript.scrollHeight, behavior: "smooth" });
    }
  }, [finalText, partialText]);

  return (
    <div ref={transcriptRef} className="transcript-view neumorphic-pressed" aria-live="polite">
      <div className="panel-header transcript-header">
        <div>
          <span className="eyebrow">LIVE OUTPUT</span>
          <h2>Transcript</h2>
          <p>Real-time local transcription</p>
        </div>
        <div className="transcript-meta">
          <span className={`vault-pill ${isTranscriptEmpty ? "" : "is-live"}`}>
            {isTranscriptEmpty ? "Idle" : "Live"}
          </span>
          {!isTranscriptEmpty ? <span>{wordCount}w · {characterCount}c</span> : null}
        </div>
      </div>

      {!isTranscriptEmpty ? (
        <div className="transcript-actions">
          <button type="button" className="ghost-button" onClick={onCopyText}>
            Copy live
          </button>
          <button type="button" className="ghost-button" onClick={onArchiveDraft}>
            Save snapshot
          </button>
          <button type="button" onClick={onReset} className="ghost-button transcript-clear">
            Clear transcript
          </button>
        </div>
      ) : null}

      {isTranscriptEmpty ? (
        <div className="transcript-empty premium-empty">
          <div className="waveform-mark" aria-hidden="true">
            <span></span><span></span><span></span><span></span><span></span>
          </div>
          <strong>{modelLoaded ? "Ready." : "Model not loaded."}</strong>
          <span>{modelLoaded ? `Press ${hotkeyLabel} and speak.` : "Open setup, load the bundled model, then register the hotkey."}</span>
        </div>
      ) : (
        <div className="transcript-text">
          {finalText ? <span>{finalText}</span> : null}
          {partialText ? <span className="partial-text">{partialText}</span> : null}
        </div>
      )}
    </div>
  );
}
