import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Controls } from "./components/Controls";
import { MetricsBar } from "./components/MetricsBar";
import { SettingsPanel } from "./components/SettingsPanel";
import { Toast } from "./components/Toast";
import { TranscriptVaultPanel } from "./components/TranscriptVault";
import { TranscriptView } from "./components/TranscriptView";
import {
  archiveCurrentDraft,
  composeLiveTranscript,
  deleteVaultEntry,
  loadTranscriptVaultState,
  saveTranscriptVaultState,
  syncVaultDraft,
  toggleVaultEntryPin,
} from "./lib/transcriptVault";
import {
  getStatus,
  loadModel,
  registerHotkey,
  resetTranscript,
  setContinuousMode,
  setInsertionMode,
  setVadEnabled,
  isTauriRuntime,
  startRecording,
  stopRecording,
} from "./tauri";
import type {
  DictationStatus,
  HotkeyUpdate,
  InsertionMode,
  TranscriptionUpdate,
} from "./types";

const emptyStatus: DictationStatus = {
  recording: false,
  continuousMode: false,
  vadEnabled: true,
  modelLoaded: false,
  language: "auto",
  modelPath: null,
  hotkey: null,
  insertionMode: "finalPaste",
  finalizedText: "",
  partialText: "",
  latencyMs: 0,
  chunkProcessingMs: 0,
  lastError: null,
};

type ToastKind = "success" | "error";

const DEFAULT_HOTKEY = "control+shift+Space";

function formatModelLabel(modelPath: string | null) {
  if (!modelPath) return "Bundled base model";
  const segments = modelPath.split(/[\\/]/).filter(Boolean);
  const filename = segments[segments.length - 1] ?? modelPath;
  return filename
    .replace(/^ggml-/i, "")
    .replace(/\.bin$/i, "")
    .replace(/[-_]+/g, " ")
    .replace(/\bq(\d)/gi, "Q$1")
    .trim();
}

function formatHotkeyLabel(hotkey: string) {
  return hotkey
    .replace(/control/gi, "⌃")
    .replace(/shift/gi, "⇧")
    .replace(/alt/gi, "⌥")
    .replace(/cmd/gi, "⌘")
    .replace(/\+/g, " ")
    .trim();
}

export default function App() {
  const [status, setStatus] = useState<DictationStatus>(emptyStatus);
  const [modelPath, setModelPath] = useState("");
  const [language, setLanguage] = useState("auto");
  const [hotkey, setHotkey] = useState(DEFAULT_HOTKEY);
  const [hotkeyPressed, setHotkeyPressed] = useState(false);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [toast, setToast] = useState<{ kind: ToastKind; message: string } | null>(null);
  const [vault, setVault] = useState(() => loadTranscriptVaultState());
  const dismissToast = useCallback(() => setToast(null), []);

  useEffect(() => {
    saveTranscriptVaultState(vault);
  }, [vault]);

  useEffect(() => {
    getStatus()
      .then((next) => {
        setStatus(next);
        setLanguage(next.language);
        if (next.modelPath) setModelPath(next.modelPath);
        if (next.hotkey) setHotkey(next.hotkey);
      })
      .catch((error) =>
        setStatus((current) => ({ ...current, lastError: String(error) })),
      );

    if (!isTauriRuntime()) return;

    const unlistenUpdate = listen<TranscriptionUpdate>(
      "dictation:update",
      (event) => {
        setStatus((current) => ({
          ...current,
          finalizedText: event.payload.finalizedText,
          partialText: event.payload.partialText,
          latencyMs: event.payload.latencyMs,
          chunkProcessingMs: event.payload.chunkProcessingMs,
          lastError: null,
        }));
      },
    );

    const unlistenHotkey = listen<HotkeyUpdate>("dictation:hotkey", (event) => {
      setHotkeyPressed(event.payload.pressed);
      getStatus()
        .then(setStatus)
        .catch((error) =>
          setStatus((current) => ({ ...current, lastError: String(error) })),
        );
    });

    const unlistenError = listen<string>("dictation:error", (event) => {
      setStatus((current) => ({ ...current, lastError: event.payload }));
    });

    return () => {
      unlistenUpdate.then((dispose) => dispose());
      unlistenHotkey.then((dispose) => dispose());
      unlistenError.then((dispose) => dispose());
    };
  }, []);

  const liveTranscript = useMemo(
    () => composeLiveTranscript(status),
    [status.finalizedText, status.partialText],
  );

  useEffect(() => {
    setVault((current) =>
      syncVaultDraft(
        current,
        liveTranscript,
        {
          language: status.language,
          modelPath: status.modelPath,
        },
      ),
    );
  }, [liveTranscript, status.language, status.modelPath]);

  const activity = useMemo(() => {
    if (hotkeyPressed) return "Push-to-talk";
    if (status.recording) return "Recording";
    if (status.continuousMode) return "Listening";
    return "Standby";
  }, [hotkeyPressed, status.recording, status.continuousMode]);

  const transcriptStats = useMemo(() => {
    const words = liveTranscript
      .trim()
      .split(/\s+/)
      .filter(Boolean);
    return {
      words: words.length,
      characters: liveTranscript.length,
    };
  }, [liveTranscript]);

  async function runAction(
    action: () => Promise<DictationStatus>,
    success?: string,
    busyLabel = "Working",
  ) {
    setBusyAction(busyLabel);
    setStatus((current) => ({ ...current, lastError: null }));
    setToast(null);
    try {
      const next = await action();
      setStatus(next);
      if (next.modelPath) setModelPath(next.modelPath);
      if (next.hotkey) setHotkey(next.hotkey);
      setLanguage(next.language);
      if (success) {
        setToast({ kind: "success", message: success });
      }
    } catch (error) {
      const message = String(error);
      setStatus((current) => ({ ...current, lastError: message }));
      setToast({ kind: "error", message });
    } finally {
      setBusyAction(null);
    }
  }

  const archiveDraftSilently = useCallback(() => {
    setVault((current) =>
      archiveCurrentDraft(current, {
        language: status.language,
        modelPath: status.modelPath,
      }),
    );
  }, [status.language, status.modelPath]);

  const handleArchiveDraft = useCallback(() => {
    archiveDraftSilently();
    setToast({ kind: "success", message: "Transcript snapshotted into the vault." });
  }, [archiveDraftSilently]);

  const handleCopyText = useCallback(async (text: string) => {
    const content = text.trim();
    if (!content) return;

    try {
      await navigator.clipboard.writeText(content);
      setToast({ kind: "success", message: "Copied to clipboard." });
    } catch (error) {
      setToast({ kind: "error", message: String(error) });
    }
  }, []);

  const handleTogglePin = useCallback((id: string) => {
    setVault((current) => toggleVaultEntryPin(current, id));
  }, []);

  const handleDeleteEntry = useCallback((id: string) => {
    setVault((current) => deleteVaultEntry(current, id));
  }, []);

  const handleClearArchive = useCallback(() => {
    setVault((current) => ({ ...current, entries: [] }));
    setToast({ kind: "success", message: "Vault history cleared." });
  }, []);

  return (
    <main className="app-shell">
      <div className="workbench">
        <div className="header neumorphic-raised">
          <div>
            <h1>NoDaysIdle Whispering</h1>
            <p className="header-subtitle">Minimal, local-first dictation with a private transcript vault.</p>
            <div className="header-chips">
              <span className={`header-chip ${status.modelLoaded ? "is-ready" : "is-warning"}`}>
                {status.modelLoaded ? "Model loaded" : "No model loaded"}
              </span>
              <span className="header-chip">{formatModelLabel(status.modelPath)}</span>
              <span className="header-chip">{formatHotkeyLabel(hotkey)}</span>
            </div>
          </div>
          <div
            className={`status-indicator ${
              status.recording || status.continuousMode ? "is-live" : ""
            }`}
          >
            <span className="dot"></span>
            <span>{activity}</span>
          </div>
        </div>

        <div className="metrics-bar neumorphic-raised">
          <MetricsBar
            latencyMs={status.latencyMs}
            chunkProcessingMs={status.chunkProcessingMs}
            wordCount={transcriptStats.words}
            characterCount={transcriptStats.characters}
          />
        </div>

        <div className="neumorphic-raised">
          <Controls
            recording={status.recording}
            continuousMode={status.continuousMode}
            modelLoaded={status.modelLoaded}
            busy={busyAction !== null}
            onStart={() =>
              runAction(startRecording, undefined, "Starting capture")
            }
            onStop={() =>
              runAction(stopRecording, "Transcript finalized and saved to the vault.", "Finalizing audio")
            }
            onContinuousChange={(enabled) =>
              runAction(
                () => setContinuousMode(enabled),
                undefined,
                "Updating listen mode",
              )
            }
          />
        </div>

        <div className="content-grid">
          <div className="transcript-container neumorphic-raised">
            <TranscriptView
              finalText={status.finalizedText}
              partialText={status.partialText}
              onCopyText={() => handleCopyText(liveTranscript)}
              onArchiveDraft={handleArchiveDraft}
              onReset={() => {
                archiveDraftSilently();
                setToast({ kind: "success", message: "Transcript archived in the vault before clearing." });
                runAction(resetTranscript, undefined, "Clearing transcript");
              }}
            />
          </div>

          <div className="sidebar-stack">
            <TranscriptVaultPanel
              vault={vault}
              onArchiveDraft={handleArchiveDraft}
              onCopyText={handleCopyText}
              onTogglePin={handleTogglePin}
              onDeleteEntry={handleDeleteEntry}
              onClearArchive={handleClearArchive}
            />

            <div className="settings-shell neumorphic-raised">
              <SettingsPanel
                modelPath={modelPath}
                language={language}
                vadEnabled={status.vadEnabled}
                insertionMode={status.insertionMode}
                hotkey={hotkey}
                captureActive={status.recording || status.continuousMode}
                busy={busyAction !== null}
                onModelPathChange={setModelPath}
                onLanguageChange={setLanguage}
                onHotkeyChange={setHotkey}
                onLoadModel={() =>
                  runAction(
                    () => loadModel(modelPath, language),
                    "Model loaded",
                    "Loading model",
                  )
                }
                onRegisterHotkey={() =>
                  runAction(
                    () => registerHotkey(hotkey),
                    `Registered ${hotkey}`,
                    "Registering hotkey",
                  )
                }
                onResetDefaults={() => {
                  setModelPath("");
                  setLanguage("auto");
                  setHotkey(DEFAULT_HOTKEY);
                  setToast({ kind: "success", message: "Settings reset to app defaults." });
                }}
                onVadChange={(enabled) =>
                  runAction(
                    () => setVadEnabled(enabled),
                    undefined,
                    "Updating VAD",
                  )
                }
                onInsertionModeChange={(mode: InsertionMode) =>
                  runAction(
                    () => setInsertionMode(mode),
                    undefined,
                    "Updating insertion",
                  )
                }
              />
            </div>
          </div>
        </div>

        <footer className="system-message">
          <span>{busyAction ?? ""}</span>
        </footer>
      </div>

      <Toast
        message={toast?.message ?? ""}
        kind={toast?.kind ?? "success"}
        onClose={dismissToast}
      />
    </main>
  );
}
