interface ControlsProps {
  recording: boolean;
  continuousMode: boolean;
  modelLoaded: boolean;
  busy: boolean;
  hotkeyLabel: string;
  onStart: () => void;
  onStop: () => void;
  onContinuousChange: (enabled: boolean) => void;
}

export function Controls({
  recording,
  continuousMode,
  modelLoaded,
  busy,
  hotkeyLabel,
  onStart,
  onStop,
  onContinuousChange,
}: ControlsProps) {
  const handleToggleClick = () => {
    if (!modelLoaded || busy) return;
    onContinuousChange(!continuousMode);
  };

  return (
    <div className="controls-grid controls-primary">
      <button
        type="button"
        className={`record-button-primary ${recording ? "is-recording" : ""}`}
        onClick={recording ? onStop : onStart}
        disabled={!modelLoaded || busy}
        aria-label={recording ? "Stop dictation" : "Start dictation"}
      >
        <span className="record-button-primary__label">
          {recording ? "■ Stop" : modelLoaded ? "● Record" : "Load model first"}
        </span>
        <span className="record-button-primary__meta">
          {modelLoaded ? `${hotkeyLabel} push-to-talk` : "Open setup → Load Model"}
        </span>
      </button>

      <div
        className="toggle-switch continuous-toggle"
        onClick={handleToggleClick}
        role="switch"
        aria-checked={continuousMode}
        aria-disabled={!modelLoaded || busy}
        tabIndex={!modelLoaded || busy ? -1 : 0}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            handleToggleClick();
          }
        }}
      >
        <div className="toggle-copy">
          <span id="continuous-mode-label">Continuous Mode</span>
          <span>VAD splits long sessions automatically.</span>
        </div>
        <div className="slider-container">
          <input
            type="checkbox"
            id="continuous-mode-toggle"
            checked={continuousMode}
            readOnly
            disabled={!modelLoaded || busy}
            aria-labelledby="continuous-mode-label"
          />
          <span className="slider"></span>
        </div>
      </div>
    </div>
  );
}
