interface TranscriptViewProps {
  finalText: string;
  partialText: string;
  onReset: () => void;
  onCopyText: () => void;
  onArchiveDraft: () => void;
}

export function TranscriptView({
  finalText,
  partialText,
  onReset,
  onCopyText,
  onArchiveDraft,
}: TranscriptViewProps) {
  const isTranscriptEmpty = finalText.length === 0 && partialText.length === 0;
  const canCopy = !isTranscriptEmpty;

  return (
    <>
      <div className="transcript-view neumorphic-pressed" aria-live="polite">
        <div className="panel-header transcript-header">
          <div>
            <h2>Live Transcript</h2>
            <p>What you say appears here first. The vault keeps a redundant copy in the sidebar.</p>
          </div>
          <span className="vault-pill">{isTranscriptEmpty ? "Idle" : "Live"}</span>
        </div>

        <div className="transcript-actions">
          <button type="button" className="ghost-button" onClick={onCopyText} disabled={!canCopy}>
            Copy live
          </button>
          <button type="button" className="ghost-button" onClick={onArchiveDraft} disabled={!canCopy}>
            Save snapshot
          </button>
          <button type="button" onClick={onReset} disabled={isTranscriptEmpty} className="ghost-button transcript-clear">
            Clear transcript
          </button>
        </div>

        {isTranscriptEmpty ? (
          <div className="transcript-empty">
            <strong>Ready for dictation.</strong>
            <span>Load a model, press the hotkey, and your transcript will land here.</span>
          </div>
        ) : (
          <>
            <span>{finalText}</span>
            {partialText ? <span className="partial-text">{partialText}</span> : null}
          </>
        )}
      </div>
    </>
  );
}
