interface ControlsProps {
  recording: boolean;
  continuousMode: boolean;
  modelLoaded: boolean;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onContinuousChange: (enabled: boolean) => void;
}

export function Controls({
  recording,
  continuousMode,
  modelLoaded,
  busy,
  onStart,
  onStop,
  onContinuousChange,
}: ControlsProps) {
  const handleToggleClick = () => {
    if (!modelLoaded || busy) return;
    onContinuousChange(!continuousMode);
  };

  return (
    <div className="controls-grid">
      <button
        className={`control-button record-button neumorphic-raised ${
          recording ? "is-recording" : ""
        }`}
        onClick={recording ? onStop : onStart}
        disabled={!modelLoaded || busy}
        aria-label={recording ? "Stop dictation" : "Start dictation"}
      >
        {recording ? "Stop Dictation" : "Start Dictation"}
      </button>

      <div
        className="toggle-switch"
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
        <label id="continuous-mode-label">Continuous Mode</label>
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
