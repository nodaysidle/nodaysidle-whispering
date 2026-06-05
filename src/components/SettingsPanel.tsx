interface SettingsPanelProps {
  modelPath: string;
  language: string;
  hotkey: string;
  vadEnabled: boolean;
  captureActive: boolean;
  busy: boolean;
  modelLoaded: boolean;
  isOpen: boolean;
  onToggleOpen: () => void;
  onModelPathChange: (path: string) => void;
  onLanguageChange: (lang: string) => void;
  onHotkeyChange: (key: string) => void;
  onVadChange: (enabled: boolean) => void;
  onLoadModel: () => void;
  onRegisterHotkey: () => void;
  onResetDefaults: () => void;
}

const COMMON_LANGUAGES = [
  { code: "auto", name: "Automatic" },
  { code: "en", name: "English" },
  { code: "es", name: "Spanish" },
  { code: "fr", name: "French" },
  { code: "de", name: "German" },
  { code: "it", name: "Italian" },
  { code: "pt", name: "Portuguese" },
  { code: "ru", name: "Russian" },
  { code: "ja", name: "Japanese" },
  { code: "ko", name: "Korean" },
  { code: "zh", name: "Chinese" },
];

function formatSummaryModel(modelLoaded: boolean, modelPath: string) {
  if (!modelLoaded) return "model not loaded";
  if (!modelPath.trim()) return "bundled model";
  return modelPath.split(/[\\/]/).filter(Boolean).pop() ?? "custom model";
}

export function SettingsPanel({
  modelPath,
  language,
  hotkey,
  vadEnabled,
  captureActive,
  busy,
  modelLoaded,
  isOpen,
  onToggleOpen,
  onModelPathChange,
  onLanguageChange,
  onHotkeyChange,
  onVadChange,
  onLoadModel,
  onRegisterHotkey,
  onResetDefaults,
}: SettingsPanelProps) {
  const isDisabled = busy || captureActive;

  const handleVadToggle = () => {
    if (isDisabled) return;
    onVadChange(!vadEnabled);
  };

  return (
    <section className="settings-panel">
      <button
        type="button"
        className="settings-summary"
        onClick={onToggleOpen}
        aria-expanded={isOpen}
      >
        <span>Capture setup</span>
        <span className="summary-detail">
          {formatSummaryModel(modelLoaded, modelPath)} · {language} · {hotkey}
        </span>
        <span className="settings-chevron" aria-hidden="true">{isOpen ? "−" : "+"}</span>
      </button>

      {isOpen ? (
        <fieldset disabled={isDisabled} className="settings-body">
          <div className="settings-heading">
            <div>
              <span className="eyebrow">SETUP</span>
              <h2>Capture setup</h2>
            </div>
            <button type="button" className="ghost-button settings-reset" onClick={onResetDefaults} disabled={busy}>
              Reset defaults
            </button>
          </div>
          <div className="field-group">
            <label htmlFor="model-path">Model Path</label>
            <input
              id="model-path"
              type="text"
              className="neumorphic-pressed"
              value={modelPath}
              onChange={(e) => onModelPathChange(e.target.value)}
              placeholder="models/ggml-base.en-q5_1.bin"
            />
            <p className="field-hint">Leave blank to use the bundled model inside the app bundle.</p>
          </div>
          <div className="field-group">
            <label htmlFor="language">Language</label>
            <select
              id="language"
              className="neumorphic-pressed"
              value={language}
              onChange={(e) => onLanguageChange(e.target.value)}
            >
              {COMMON_LANGUAGES.map(({ code, name }) => (
                <option key={code} value={code}>
                  {name}
                </option>
              ))}
            </select>
          </div>
          <button
            type="button"
            onClick={onLoadModel}
            className="settings-button neumorphic-raised"
          >
            {busy ? "Loading..." : "Load Model"}
          </button>

          <div className="field-group">
            <label htmlFor="hotkey">Push-to-Talk Hotkey</label>
            <input
              id="hotkey"
              type="text"
              className="neumorphic-pressed"
              value={hotkey}
              onChange={(e) => onHotkeyChange(e.target.value)}
              placeholder="control+shift+Space"
            />
            <p className="field-hint">Default: <code>control+shift+Space</code></p>
            <p className="field-hint muted">macOS may require Accessibility permission for global hotkeys and text insertion.</p>
          </div>
          <button
            type="button"
            onClick={onRegisterHotkey}
            className="settings-button neumorphic-raised"
          >
            Register Hotkey
          </button>

          <div
            className="toggle-switch"
            onClick={handleVadToggle}
            role="switch"
            aria-checked={vadEnabled}
            aria-disabled={isDisabled}
            tabIndex={isDisabled ? -1 : 0}
            onKeyDown={(e) => {
              if (e.key === " " || e.key === "Enter") {
                e.preventDefault();
                handleVadToggle();
              }
            }}
          >
            <span id="vad-toggle-label">Voice Activity Detection</span>
            <div className="slider-container">
              <input
                type="checkbox"
                id="vad-toggle"
                checked={vadEnabled}
                readOnly
                disabled={isDisabled}
                aria-labelledby="vad-toggle-label"
              />
              <span className="slider"></span>
            </div>
          </div>
        </fieldset>
      ) : null}
    </section>
  );
}
