import { useMemo, useState } from "react";
import type { TranscriptVaultState } from "../lib/transcriptVault";

interface TranscriptVaultPanelProps {
  vault: TranscriptVaultState;
  onArchiveDraft: () => void;
  onCopyText: (text: string) => void;
  onTogglePin: (id: string) => void;
  onDeleteEntry: (id: string) => void;
  onClearArchive: () => void;
}

function formatTimestamp(value: string | null) {
  if (!value) return "—";

  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    day: "2-digit",
    month: "short",
  }).format(new Date(value));
}

export function TranscriptVaultPanel({
  vault,
  onArchiveDraft,
  onCopyText,
  onTogglePin,
  onDeleteEntry,
  onClearArchive,
}: TranscriptVaultPanelProps) {
  const [search, setSearch] = useState("");
  const [expandedEntryIds, setExpandedEntryIds] = useState<Set<string>>(() => new Set());

  const requestClearArchive = () => {
    if (!vault.entries.length) return;
    if (window.confirm("Clear every saved transcript from this local vault?")) {
      onClearArchive();
    }
  };

  const requestDeleteEntry = (id: string, title: string) => {
    if (window.confirm(`Delete "${title}"?`)) {
      onDeleteEntry(id);
    }
  };

  const toggleExpanded = (id: string) => {
    setExpandedEntryIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const filteredEntries = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return vault.entries;

    return vault.entries.filter((entry) => {
      return (
        entry.title.toLowerCase().includes(term) ||
        entry.text.toLowerCase().includes(term) ||
        entry.language.toLowerCase().includes(term)
      );
    });
  }, [search, vault.entries]);

  const liveText = vault.draftText.trim();

  return (
    <section className="vault-panel neumorphic-raised">
      <div className="panel-header">
        <div>
          <span className="eyebrow">VAULT</span>
          <h2>Transcript Vault</h2>
          <p>Automatically saved transcripts</p>
        </div>
        <span className="vault-pill">{vault.entries.length} saved</span>
      </div>

      <div className={`vault-live-card neumorphic-pressed ${liveText ? "has-live-text" : "is-empty"}`}>
        <div className="vault-live-card__topline">
          <span className="vault-label">Current cache</span>
          <span className="vault-timestamp">{formatTimestamp(vault.draftUpdatedAt)}</span>
        </div>
        {liveText ? (
          <>
            <div className="vault-live-card__body">{liveText}</div>
            <div className="vault-live-card__footer">
              <button type="button" className="ghost-button" onClick={() => onCopyText(liveText)}>
                Copy live
              </button>
              <button type="button" className="ghost-button" onClick={onArchiveDraft}>
                Save snapshot
              </button>
            </div>
          </>
        ) : (
          <div className="vault-live-card__empty">No active transcript</div>
        )}
      </div>

      <div className="vault-toolbar">
        <div className="search-shell">
          <span aria-hidden="true">⌕</span>
          <input
            type="search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search saved transcripts"
            aria-label="Search saved transcripts"
            className="neumorphic-pressed"
          />
        </div>
        <button type="button" className="ghost-button danger" onClick={requestClearArchive} disabled={!vault.entries.length}>
          Clear history
        </button>
      </div>

      <div className="vault-list">
        {vault.entries.length === 0 ? (
          <div className="vault-empty">
            <strong>No saved transcripts yet.</strong>
            <span>Finish a dictation session and it will land here automatically.</span>
          </div>
        ) : filteredEntries.length === 0 ? (
          <div className="vault-empty">
            <strong>No results for “{search}”.</strong>
            <span>Try a different search term.</span>
          </div>
        ) : (
          filteredEntries.map((entry) => (
            <article key={entry.id} className={`vault-entry neumorphic-pressed ${entry.pinned ? "is-pinned" : ""}`}>
              <div className="vault-entry__meta">
                <strong>{entry.title}</strong>
                <span>
                  {entry.wordCount} words · {entry.language} · {formatTimestamp(entry.updatedAt)}
                </span>
              </div>
              <p>{expandedEntryIds.has(entry.id) || entry.text.length <= 220 ? entry.text : `${entry.text.slice(0, 220).trim()}…`}</p>
              <div className="vault-entry__actions">
                {entry.text.length > 220 ? (
                  <button type="button" className="ghost-button" onClick={() => toggleExpanded(entry.id)}>
                    {expandedEntryIds.has(entry.id) ? "Show less" : "Show more"}
                  </button>
                ) : null}
                <button type="button" className="ghost-button" onClick={() => onCopyText(entry.text)}>
                  Copy
                </button>
                <button type="button" className="ghost-button" onClick={() => onTogglePin(entry.id)}>
                  {entry.pinned ? "Unpin" : "Pin"}
                </button>
                <button type="button" className="ghost-button danger" onClick={() => requestDeleteEntry(entry.id, entry.title)}>
                  Delete
                </button>
              </div>
            </article>
          ))
        )}
      </div>
    </section>
  );
}
