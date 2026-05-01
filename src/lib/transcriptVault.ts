import type { DictationStatus } from "../types";

const STORAGE_KEY = "nodaysidle-whispering.transcript-vault.v1";
const MAX_ENTRIES = 24;

export type TranscriptVaultEntry = {
  id: string;
  title: string;
  text: string;
  createdAt: string;
  updatedAt: string;
  source: "session" | "manual";
  pinned: boolean;
  wordCount: number;
  charCount: number;
  language: string;
  modelPath: string | null;
};

export type TranscriptVaultState = {
  draftText: string;
  draftUpdatedAt: string | null;
  lastLiveText: string;
  entries: TranscriptVaultEntry[];
};

const emptyVaultState: TranscriptVaultState = {
  draftText: "",
  draftUpdatedAt: null,
  lastLiveText: "",
  entries: [],
};

function nowIso() {
  return new Date().toISOString();
}

function createId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }

  return `vault-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function normalize(text: string) {
  return text.replace(/\s+/g, " ").trim();
}

function countWords(text: string) {
  return normalize(text)
    ? normalize(text).split(/\s+/).filter(Boolean).length
    : 0;
}

export function composeLiveTranscript(status: DictationStatus) {
  return [status.finalizedText, status.partialText].filter(Boolean).join(" ").trim();
}

export function formatVaultTitle(text: string) {
  const words = normalize(text).split(/\s+/).filter(Boolean);
  if (!words.length) return "Untitled transcript";
  return words.slice(0, 8).join(" ") + (words.length > 8 ? "…" : "");
}

function makeEntry(
  text: string,
  status: Pick<DictationStatus, "language" | "modelPath">,
  source: TranscriptVaultEntry["source"],
): TranscriptVaultEntry {
  const createdAt = nowIso();
  const normalizedText = text.trim();

  return {
    id: createId(),
    title: formatVaultTitle(normalizedText),
    text: normalizedText,
    createdAt,
    updatedAt: createdAt,
    source,
    pinned: false,
    wordCount: countWords(normalizedText),
    charCount: normalizedText.length,
    language: status.language,
    modelPath: status.modelPath,
  };
}

function capEntries(entries: TranscriptVaultEntry[]) {
  const pinned = entries.filter((entry) => entry.pinned);
  const unpinned = entries.filter((entry) => !entry.pinned);
  const cappedUnpinned = unpinned.slice(0, Math.max(0, MAX_ENTRIES - pinned.length));
  return [...pinned, ...cappedUnpinned].sort((a, b) => {
    if (a.pinned !== b.pinned) return Number(b.pinned) - Number(a.pinned);
    return b.updatedAt.localeCompare(a.updatedAt);
  });
}

function archiveText(
  state: TranscriptVaultState,
  text: string,
  status: Pick<DictationStatus, "language" | "modelPath">,
  source: TranscriptVaultEntry["source"],
) {
  const normalizedText = text.trim();
  if (!normalizedText) return state;

  const latest = state.entries[0];
  if (latest && normalize(latest.text) === normalize(normalizedText)) {
    return {
      ...state,
      entries: [
        {
          ...latest,
          updatedAt: nowIso(),
          language: status.language,
          modelPath: status.modelPath,
        },
        ...state.entries.slice(1),
      ],
    };
  }

  return {
    ...state,
    entries: capEntries([makeEntry(normalizedText, status, source), ...state.entries]),
  };
}

export function syncVaultDraft(
  state: TranscriptVaultState,
  liveText: string,
  status: Pick<DictationStatus, "language" | "modelPath">,
) {
  const normalizedLive = liveText.trim();
  const normalizedLastLive = state.lastLiveText.trim();

  if (normalizedLive) {
    if (normalizedLive === normalize(state.draftText) && normalizedLive === normalizedLastLive) {
      return state;
    }

    return {
      ...state,
      draftText: normalizedLive,
      draftUpdatedAt: nowIso(),
      lastLiveText: normalizedLive,
    };
  }

  if (!normalizedLastLive) {
    return {
      ...state,
      draftText: "",
      draftUpdatedAt: state.draftUpdatedAt,
      lastLiveText: "",
    };
  }

  const archived = archiveText(state, normalizedLastLive, status, "session");
  return {
    ...archived,
    draftText: "",
    draftUpdatedAt: nowIso(),
    lastLiveText: "",
  };
}

export function archiveCurrentDraft(
  state: TranscriptVaultState,
  status: Pick<DictationStatus, "language" | "modelPath">,
  source: TranscriptVaultEntry["source"] = "manual",
) {
  const text = state.draftText.trim() || state.lastLiveText.trim();
  if (!text) return state;

  return archiveText(state, text, status, source);
}

export function toggleVaultEntryPin(state: TranscriptVaultState, id: string) {
  return {
    ...state,
    entries: state.entries
      .map((entry) =>
        entry.id === id
          ? { ...entry, pinned: !entry.pinned, updatedAt: nowIso() }
          : entry,
      )
      .sort((a, b) => {
        if (a.pinned !== b.pinned) return Number(b.pinned) - Number(a.pinned);
        return b.updatedAt.localeCompare(a.updatedAt);
      }),
  };
}

export function deleteVaultEntry(state: TranscriptVaultState, id: string) {
  return {
    ...state,
    entries: state.entries.filter((entry) => entry.id !== id),
  };
}

export function loadTranscriptVaultState() {
  if (typeof window === "undefined") return emptyVaultState;

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return emptyVaultState;

    const parsed = JSON.parse(raw) as Partial<TranscriptVaultState>;
    return {
      draftText: typeof parsed.draftText === "string" ? parsed.draftText : "",
      draftUpdatedAt:
        typeof parsed.draftUpdatedAt === "string" ? parsed.draftUpdatedAt : null,
      lastLiveText: typeof parsed.lastLiveText === "string" ? parsed.lastLiveText : "",
      entries: Array.isArray(parsed.entries)
        ? parsed.entries
            .filter((entry): entry is TranscriptVaultEntry => {
              return (
                entry &&
                typeof entry.id === "string" &&
                typeof entry.title === "string" &&
                typeof entry.text === "string"
              );
            })
            .sort((a, b) => {
              if (a.pinned !== b.pinned) return Number(b.pinned) - Number(a.pinned);
              return b.updatedAt.localeCompare(a.updatedAt);
            })
            .slice(0, MAX_ENTRIES)
        : [],
    };
  } catch {
    return emptyVaultState;
  }
}

export function saveTranscriptVaultState(state: TranscriptVaultState) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

export function getVaultPreview(state: TranscriptVaultState) {
  return normalize(state.draftText || state.lastLiveText);
}
