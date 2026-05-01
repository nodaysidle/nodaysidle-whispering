import type { InsertionMode } from "../types";

interface SettingsPanelProps {
  modelPath: string;
  language: string;
  hotkey: string;
  vadEnabled: boolean;
  captureActive: boolean;
  busy: boolean;
  onModelPathChange: (path: string) => void;
  onLanguageChange: (lang: string) => void;
  onHotkeyChange: (key: string) => void;
  onVadChange: (enabled: boolean) => void;
  onLoadModel: () => void;
  onRegisterHotkey: () => void;
  // Insertion mode is no longer part of the UI to simplify it.
  // We keep the props for potential future use or to avoid breaking App.tsx immediately.
  insertionMode: InsertionMode;
  onInsertionModeChange: (mode: InsertionMode) => void;
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

export function SettingsPanel({
  modelPath,
  language,
  hotkey,
  vadEnabled,
  captureActive,
  busy,
  onModelPathChange,
  onLanguageChange,
  onHotkeyChange,
  onVadChange,
  onLoadModel,
  onRegisterHotkey,
}: SettingsPanelProps) {
  const isDisabled = busy || captureActive;

  const handleVadToggle = () => {
    if (isDisabled) return;
    onVadChange(!vadEnabled);
  };

  return (
    <fieldset disabled={isDisabled} className="settings-panel">
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
        <p className="field-hint">Use the format expected by global-hotkey, for example <code>control+shift+Space</code>.</p>
      </div>
      <button
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
            handleVadToggle();
          }
        }}
      >
        <label>Voice Activity Detection</label>
        <div className="slider-container">
          <input
            type="checkbox"
            checked={vadEnabled}
            readOnly
            disabled={isDisabled}
          />
          <span className="slider"></span>
        </div>
      </div>
    </fieldset>
  );
}
