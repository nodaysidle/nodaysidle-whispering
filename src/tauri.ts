import { invoke } from "@tauri-apps/api/core";
import type { DictationStatus, InsertionMode } from "./types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

let browserPreviewStatus: DictationStatus = {
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
  lastError: null
};

export function isTauriRuntime() {
  return typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;
}

function previewStatus(update?: Partial<DictationStatus>) {
  browserPreviewStatus = { ...browserPreviewStatus, ...update };
  return Promise.resolve(browserPreviewStatus);
}

export function getStatus() {
  if (!isTauriRuntime()) return previewStatus();
  return invoke<DictationStatus>("get_status");
}

export function loadModel(modelPath: string, language: string) {
  if (!isTauriRuntime()) {
    return previewStatus({
      modelLoaded: true,
      modelPath: modelPath.trim() || "models/ggml-base.en-q5_1.bin",
      language: language.trim() || "auto",
      lastError: null
    });
  }
  return invoke<DictationStatus>("load_model", { modelPath, language });
}

export function startRecording() {
  if (!isTauriRuntime()) return previewStatus({ recording: true, lastError: null });
  return invoke<DictationStatus>("start_recording");
}

export function stopRecording() {
  if (!isTauriRuntime()) return previewStatus({ recording: false });
  return invoke<DictationStatus>("stop_recording");
}

export function setContinuousMode(enabled: boolean) {
  if (!isTauriRuntime()) return previewStatus({ continuousMode: enabled });
  return invoke<DictationStatus>("set_continuous_mode", { enabled });
}

export function setVadEnabled(enabled: boolean) {
  if (!isTauriRuntime()) return previewStatus({ vadEnabled: enabled });
  return invoke<DictationStatus>("set_vad_enabled", { enabled });
}

export function setInsertionMode(mode: InsertionMode) {
  if (!isTauriRuntime()) return previewStatus({ insertionMode: mode });
  return invoke<DictationStatus>("set_insertion_mode", { mode });
}

export function registerHotkey(hotkey: string) {
  if (!isTauriRuntime()) return previewStatus({ hotkey });
  return invoke<DictationStatus>("register_push_to_talk_hotkey", { hotkey });
}

export function resetTranscript() {
  if (!isTauriRuntime()) {
    return previewStatus({
      finalizedText: "",
      partialText: "",
      latencyMs: 0,
      chunkProcessingMs: 0,
      lastError: null
    });
  }
  return invoke<DictationStatus>("reset_transcript");
}
