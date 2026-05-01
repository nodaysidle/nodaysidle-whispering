export type InsertionMode = "off" | "incrementalTyping" | "finalPaste";

export type DictationStatus = {
  recording: boolean;
  continuousMode: boolean;
  vadEnabled: boolean;
  modelLoaded: boolean;
  language: string;
  modelPath: string | null;
  hotkey: string | null;
  insertionMode: InsertionMode;
  finalizedText: string;
  partialText: string;
  latencyMs: number;
  chunkProcessingMs: number;
  lastError: string | null;
};

export type TranscriptionUpdate = {
  text: string;
  finalizedText: string;
  partialText: string;
  isFinal: boolean;
  speechActive: boolean;
  windowStartMs: number;
  windowEndMs: number;
  latencyMs: number;
  chunkProcessingMs: number;
};

export type HotkeyUpdate = {
  hotkey: string;
  pressed: boolean;
};
