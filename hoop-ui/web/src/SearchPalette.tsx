import { useAtom, useAtomValue } from 'jotai';
import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import {
  searchPaletteOpenAtom,
  projectCardsAtom,
  beadsAtom,
  conversationsAtom,
  formatContent,
} from './atoms';

interface SearchResult {
  kind: 'project' | 'bead' | 'conversation';
  id: string;
  title: string;
  snippet: string;
  project?: string;
  href?: string;
}

const MAX_RESULTS = 50;
const SNIPPET_HALF = 30;

function getSnippet(text: string, query: string): string {
  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const idx = lowerText.indexOf(lowerQuery);
  if (idx === -1) return text.slice(0, 60);
  const start = Math.max(0, idx - SNIPPET_HALF);
  const end = Math.min(text.length, idx + query.length + SNIPPET_HALF);
  let snippet = text.slice(start, end);
  if (start > 0) snippet = '…' + snippet;
  if (end < text.length) snippet = snippet + '…';
  return snippet;
}

export function SearchPalette() {
  const [open, setOpen] = useAtom(searchPaletteOpenAtom);
  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const projectCards = useAtomValue(projectCardsAtom);
  const beads = useAtomValue(beadsAtom);
  const conversations = useAtomValue(conversationsAtom);

  // 150ms debounce
  useEffect(() => {
    const t = setTimeout(() => setDebouncedQuery(query), 150);
    return () => clearTimeout(t);
  }, [query]);

  // Focus input when opened; reset state
  useEffect(() => {
    if (open) {
      setQuery('');
      setDebouncedQuery('');
      setSelectedIdx(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [open]);

  // Reset selection index when results change
  useEffect(() => {
    setSelectedIdx(0);
  }, [debouncedQuery]);

  const results = useMemo((): SearchResult[] => {
    const q = debouncedQuery.trim().toLowerCase();
    if (!q) return [];
    const out: SearchResult[] = [];

    // Project matches — name + label
    for (const p of projectCards) {
      if (out.length >= MAX_RESULTS) break;
      if (`${p.name} ${p.label}`.toLowerCase().includes(q)) {
        out.push({
          kind: 'project',
          id: p.name,
          title: p.label || p.name,
          snippet: p.name,
          href: `#/${p.name}`,
        });
      }
    }

    // Bead title matches
    for (const b of beads) {
      if (out.length >= MAX_RESULTS) break;
      if (b.title.toLowerCase().includes(q)) {
        out.push({
          kind: 'bead',
          id: b.id,
          title: b.title,
          snippet: getSnippet(b.title, debouncedQuery.trim()),
        });
      }
    }

    // Conversation matches — title first, then message content
    for (const conv of conversations) {
      if (out.length >= MAX_RESULTS) break;

      if (conv.title.toLowerCase().includes(q)) {
        out.push({
          kind: 'conversation',
          id: conv.id,
          title: conv.title,
          snippet: getSnippet(conv.title, debouncedQuery.trim()),
          project: conv.cwd,
        });
        continue;
      }

      // Search message bodies
      for (const msg of conv.messages) {
        if (out.length >= MAX_RESULTS) break;
        const text = formatContent(msg.content);
        if (text.toLowerCase().includes(q)) {
          out.push({
            kind: 'conversation',
            id: conv.id,
            title: conv.title,
            snippet: getSnippet(text, debouncedQuery.trim()),
            project: conv.cwd,
          });
          break;
        }
      }
    }

    return out;
  }, [debouncedQuery, projectCards, beads, conversations]);

  const navigate = useCallback((r: SearchResult) => {
    if (r.href) window.location.hash = r.href.slice(1);
    setOpen(false);
  }, [setOpen]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setOpen(false);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIdx(i => Math.min(i + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIdx(i => Math.max(i - 1, 0));
    } else if (e.key === 'Enter') {
      if (results[selectedIdx]) navigate(results[selectedIdx]);
    }
  }, [results, selectedIdx, setOpen, navigate]);

  // Scroll selected item into view
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const item = list.children[selectedIdx] as HTMLElement | undefined;
    item?.scrollIntoView({ block: 'nearest' });
  }, [selectedIdx]);

  if (!open) return null;

  return (
    <div
      className="sp-overlay"
      onClick={() => setOpen(false)}
      role="presentation"
    >
      <div
        className="sp-panel"
        role="dialog"
        aria-label="Search"
        aria-modal="true"
        onClick={e => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className="sp-input-row">
          <svg className="sp-search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" strokeWidth="1.5"/>
            <path d="M10 10L14 14" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
          </svg>
          <input
            ref={inputRef}
            className="sp-input"
            type="text"
            placeholder="Search projects, beads, conversations…"
            value={query}
            onChange={e => setQuery(e.target.value)}
            aria-autocomplete="list"
            aria-controls="sp-results"
            aria-activedescendant={results[selectedIdx] ? `sp-item-${selectedIdx}` : undefined}
            autoComplete="off"
            spellCheck={false}
          />
          <kbd className="sp-esc-hint">esc</kbd>
        </div>

        {results.length > 0 && (
          <ul
            id="sp-results"
            ref={listRef}
            className="sp-results"
            role="listbox"
            aria-label="Search results"
          >
            {results.map((r, i) => (
              <li
                key={`${r.kind}-${r.id}-${i}`}
                id={`sp-item-${i}`}
                role="option"
                aria-selected={i === selectedIdx}
                className={`sp-item sp-item-${r.kind}${i === selectedIdx ? ' sp-item-selected' : ''}`}
                onMouseEnter={() => setSelectedIdx(i)}
                onClick={() => navigate(r)}
              >
                <span className={`sp-badge sp-badge-${r.kind}`}>{r.kind}</span>
                <span className="sp-item-body">
                  <span className="sp-item-title">{r.title}</span>
                  {r.snippet !== r.title && (
                    <span className="sp-item-snippet">{r.snippet}</span>
                  )}
                </span>
                {r.project && (
                  <span className="sp-item-project" title={r.project}>
                    {r.project.split('/').pop()}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}

        {debouncedQuery.trim() && results.length === 0 && (
          <div className="sp-empty">No results for &ldquo;{debouncedQuery.trim()}&rdquo;</div>
        )}

        {!debouncedQuery.trim() && (
          <div className="sp-hint-row">
            <span className="sp-hint">Search across projects, beads, and conversations</span>
            <span className="sp-hint-keys">
              <kbd>&uarr;</kbd><kbd>&darr;</kbd> navigate &nbsp; <kbd>&#x23CE;</kbd> open
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
