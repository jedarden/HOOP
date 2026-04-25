import { useAtomValue, useSetAtom, useAtom } from 'jotai';
import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import { conversationsAtom, streamingActiveIdsAtom, selectedConversationIdAtom, Conversation, dictatedNotesAtom, NoteSummary, DictatedNote, conversationFilterAtom, ConversationFilter, screenCapturesAtom, ScreenCaptureSummary, ScreenCaptureData, fileNavigationAtom } from './atoms';
import AudioPlayer from './components/AudioPlayer';
import VideoPlayer from './components/VideoPlayer';
import { scanForSecrets, getSecretSeverity, truncateSecret } from './components/secretsScanner';
import BeadDraftForm from './BeadDraftForm';
import StitchDraftForm from './StitchDraftForm';
import { TabId } from './ProjectDetail';

const PAGE_SIZE = 50;

type StitchStatus = 'active' | 'awaiting_review' | 'done' | 'all';
type TranscriptionStatus = 'Pending' | 'Completed' | 'Failed';

interface FilterConfig {
  status: StitchStatus;
  search: string;
}

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const then = new Date(timestamp);
  const seconds = Math.floor((now.getTime() - then.getTime()) / 1000);

  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}

function getKindBadge(kind: string): { label: string; className: string } {
  switch (kind) {
    case 'worker':
      return { label: 'Worker', className: 'badge-worker' };
    case 'operator':
      return { label: 'Operator', className: 'badge-operator' };
    case 'dictated':
      return { label: 'Dictated', className: 'badge-dictated' };
    case 'ad-hoc':
      return { label: 'Ad-hoc', className: 'badge-ad-hoc' };
    case 'screen-capture':
      return { label: 'Screen', className: 'badge-screen-capture' };
    default:
      return { label: kind, className: 'badge-unknown' };
  }
}

function getStatusBadge(status: string): { label: string; className: string } {
  switch (status) {
    case 'active':
      return { label: 'Active', className: 'status-stitch-active' };
    case 'awaiting_review':
      return { label: 'Awaiting Review', className: 'status-stitch-review' };
    case 'done':
      return { label: 'Done', className: 'status-stitch-done' };
    default:
      return { label: status, className: 'status-unknown' };
  }
}

function TranscriptionStatusBadge({ status }: { status?: TranscriptionStatus }) {
  if (!status || status === 'Completed') return null;
  const config = status === 'Pending'
    ? { label: 'Transcribing', className: 'badge-transcribing' }
    : { label: 'Transcription Failed', className: 'badge-transcription-failed' };
  return <span className={`badge ${config.className}`}>{config.label}</span>;
}

// Unified stitch item — conversation, dictated note, or screen capture
interface StitchItem {
  id: string;
  title: string;
  kind: string;
  status: string;
  createdAt: string;
  lastActivityAt: string;
  project: string;
  messageCount: number;
  totalTokens: number;
  linkedBeads: string[];
  workerMetadata?: Conversation['worker_metadata'];
  isStreaming: boolean;
  // Dictated note fields
  dictatedNote?: NoteSummary;
  // Screen capture fields
  screenCapture?: ScreenCaptureSummary;
}

function DictatedNoteDetail({ note, onUpdate }: { note: NoteSummary; onUpdate: (updated: NoteSummary) => void }) {
  const audioUrl = `/api/dictated-notes/${note.stitch_id}/audio`;
  const [fullNote, setFullNote] = useState<DictatedNote | null>(null);
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [editedTitle, setEditedTitle] = useState(note.title);
  const [isEditingTags, setIsEditingTags] = useState(false);
  const [editedTags, setEditedTags] = useState(note.tags.join(', '));
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    fetch(`/api/dictated-notes/${note.stitch_id}`)
      .then(r => r.ok ? r.json() : null)
      .then(data => {
        if (data) {
          setFullNote({
            stitch_id: data.stitch_id,
            audio_url: audioUrl,
            transcript: data.transcript,
            transcript_words: data.transcript_words || [],
            duration_secs: data.duration_secs,
            language: data.language,
            recorded_at: data.recorded_at,
            transcription_status: data.transcription_status || 'Pending',
          });
        }
      })
      .catch(() => {});
  }, [note.stitch_id, audioUrl]);

  const secretsWarning = useMemo(() => {
    if (!fullNote) return null;
    return scanForSecrets(fullNote.transcript);
  }, [fullNote]);

  const transcript = fullNote && fullNote.transcript_words.length > 0
    ? { text: fullNote.transcript, words: fullNote.transcript_words }
    : undefined;

  const handleSaveTitle = async () => {
    setIsSaving(true);
    try {
      const response = await fetch(`/api/dictated-notes/${note.stitch_id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: editedTitle }),
      });
      if (response.ok) {
        onUpdate({ ...note, title: editedTitle });
      }
    } catch (e) {
      console.error('Failed to update title:', e);
    } finally {
      setIsSaving(false);
      setIsEditingTitle(false);
    }
  };

  const handleSaveTags = async () => {
    setIsSaving(true);
    try {
      const tags = editedTags.split(',').map(t => t.trim()).filter(t => t);
      const response = await fetch(`/api/dictated-notes/${note.stitch_id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tags }),
      });
      if (response.ok) {
        onUpdate({ ...note, tags });
      }
    } catch (e) {
      console.error('Failed to update tags:', e);
    } finally {
      setIsSaving(false);
      setIsEditingTags(false);
    }
  };

  const status = fullNote?.transcription_status || note.transcription_status || 'Pending';

  return (
    <div className="dictated-note-detail">
      {/* Transcription Status */}
      {status === 'Pending' && (
        <div className="dictated-note-status-bar status-transcribing">
          <span className="status-spinner" />
          <span>Transcribing audio...</span>
        </div>
      )}
      {status === 'Failed' && (
        <div className="dictated-note-status-bar status-failed">
          <span>Transcription failed — showing raw preview</span>
        </div>
      )}

      {/* Secrets Warning Banner */}
      {secretsWarning && secretsWarning.count > 0 && (
        <div className={`secrets-warning-banner secrets-${getSecretSeverity(secretsWarning.matches[0].type)}`}>
          <div className="secrets-warning-header">
            <span className="secrets-warning-icon">⚠️</span>
            <span className="secrets-warning-title">
              Potential {secretsWarning.count} secret{secretsWarning.count > 1 ? 's' : ''} detected
            </span>
          </div>
          <div className="secrets-warning-list">
            {secretsWarning.matches.slice(0, 5).map((match, i) => (
              <div key={i} className="secrets-warning-item">
                <span className="secrets-warning-type">{match.type}:</span>
                <span className="secrets-warning-value">{truncateSecret(match.value)}</span>
              </div>
            ))}
            {secretsWarning.matches.length > 5 && (
              <span className="secrets-warning-more">
                +{secretsWarning.matches.length - 5} more...
              </span>
            )}
          </div>
        </div>
      )}

      {/* Editable Title */}
      <div className="dictated-note-title-row">
        {isEditingTitle ? (
          <div className="dictated-note-edit-title">
            <input
              type="text"
              value={editedTitle}
              onChange={(e) => setEditedTitle(e.target.value)}
              className="dictated-note-title-input"
              disabled={isSaving}
              onKeyDown={(e) => e.key === 'Enter' && handleSaveTitle()}
            />
            <button
              onClick={handleSaveTitle}
              disabled={isSaving || !editedTitle.trim()}
              className="dictated-note-save-btn"
            >
              {isSaving ? 'Saving...' : 'Save'}
            </button>
            <button
              onClick={() => { setIsEditingTitle(false); setEditedTitle(note.title); }}
              disabled={isSaving}
              className="dictated-note-cancel-btn"
            >
              Cancel
            </button>
          </div>
        ) : (
          <div className="dictated-note-title-display">
            <h4 className="dictated-note-title">{note.title}</h4>
            <button
              onClick={() => setIsEditingTitle(true)}
              className="dictated-note-edit-btn"
              aria-label="Edit title"
            >
              ✏️
            </button>
          </div>
        )}
      </div>

      {/* Editable Tags */}
      <div className="dictated-note-tags-row">
        {isEditingTags ? (
          <div className="dictated-note-edit-tags">
            <input
              type="text"
              value={editedTags}
              onChange={(e) => setEditedTags(e.target.value)}
              placeholder="Enter comma-separated tags"
              className="dictated-note-tags-input"
              disabled={isSaving}
              onKeyDown={(e) => e.key === 'Enter' && handleSaveTags()}
            />
            <button
              onClick={handleSaveTags}
              disabled={isSaving}
              className="dictated-note-save-btn"
            >
              {isSaving ? 'Saving...' : 'Save'}
            </button>
            <button
              onClick={() => { setIsEditingTags(false); setEditedTags(note.tags.join(', ')); }}
              disabled={isSaving}
              className="dictated-note-cancel-btn"
            >
              Cancel
            </button>
          </div>
        ) : (
          <div className="dictated-note-tags-display">
            <span className="meta-label">Tags:</span>
            {note.tags.length > 0 ? (
              note.tags.map((tag, i) => (
                <span key={i} className="dictated-tag-chip">{tag}</span>
              ))
            ) : (
              <span className="meta-value meta-value-empty">No tags</span>
            )}
            <button
              onClick={() => setIsEditingTags(true)}
              className="dictated-note-edit-btn"
              aria-label="Edit tags"
            >
              ✏️
            </button>
          </div>
        )}
      </div>

      {/* Metadata */}
      <div className="dictated-note-detail-meta">
        {note.duration_secs != null && (
          <span className="meta-item">
            <span className="meta-label">Duration:</span>
            <span className="meta-value">{Math.round(note.duration_secs)}s</span>
          </span>
        )}
        {note.language && (
          <span className="meta-item">
            <span className="meta-label">Language:</span>
            <span className="meta-value">{note.language}</span>
          </span>
        )}
        <span className="meta-item">
          <span className="meta-label">Recorded:</span>
          <span className="meta-value">{new Date(note.recorded_at).toLocaleString()}</span>
        </span>
        <span className="meta-item">
          <span className="meta-label">Transcribed:</span>
          <span className="meta-value">{new Date(note.transcribed_at).toLocaleString()}</span>
        </span>
      </div>

      {/* Audio Player or Transcript Preview */}
      {status === 'Completed' && fullNote ? (
        <AudioPlayer audioUrl={audioUrl} transcript={transcript} />
      ) : (
        <div className="dictated-note-transcript-preview">
          <p>{note.transcript_preview}</p>
        </div>
      )}
    </div>
  );
}

function ScreenCaptureDetail({ summary }: { summary: ScreenCaptureSummary }) {
  const [data, setData] = useState<ScreenCaptureData | null>(null);

  useEffect(() => {
    let mounted = true;
    fetch(`/api/screen-capture/${summary.stitch_id}`)
      .then(r => r.ok ? r.json() : null)
      .then((d: ScreenCaptureData | null) => {
        if (mounted && d) setData(d);
      })
      .catch(() => {});
    return () => { mounted = false; };
  }, [summary.stitch_id]);

  return (
    <div className="dictated-note-detail">
      <div className="dictated-note-detail-meta">
        {summary.duration_secs != null && (
          <span className="meta-item">
            <span className="meta-label">Duration:</span>
            <span className="meta-value">{Math.round(summary.duration_secs)}s</span>
          </span>
        )}
        {summary.chapter_count > 0 && (
          <span className="meta-item">
            <span className="meta-label">Chapters:</span>
            <span className="meta-value">{summary.chapter_count}</span>
          </span>
        )}
        <span className="meta-item">
          <span className="meta-label">Recorded:</span>
          <span className="meta-value">{new Date(summary.recorded_at).toLocaleString()}</span>
        </span>
      </div>

      {data ? (
        <VideoPlayer
          videoUrl={data.video_url}
          chapters={data.chapters}
          transcript={data.transcript ?? undefined}
        />
      ) : (
        <div className="dictated-note-transcript-preview">
          <p>Loading video...</p>
        </div>
      )}
    </div>
  );
}

// ─── Net-diff types (local) ───────────────────────────────────────────────────

interface NetDiffFile {
  old_path: string;
  new_path: string;
  is_new: boolean;
  is_deleted: boolean;
  added: number;
  removed: number;
}

interface NetDiffWorkspace {
  workspace: string;
  commit_shas: string[];
  ref_range: string;
  files: NetDiffFile[];
  total_added: number;
  total_removed: number;
}

interface NetDiffResponse {
  workspaces: NetDiffWorkspace[];
  total_added: number;
  total_removed: number;
  truncated: boolean;
  bead_count: number;
  commit_count: number;
}

// ─── TouchedFilesPanel ────────────────────────────────────────────────────────

interface TouchedFilesPanelProps {
  stitchId: string;
  onFileClick: (filePath: string, refRange: string) => void;
}

function TouchedFilesPanel({ stitchId, onFileClick }: TouchedFilesPanelProps) {
  const [netDiff, setNetDiff] = useState<NetDiffResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    setLoading(true);
    fetch(`/api/stitches/${encodeURIComponent(stitchId)}/net-diff`)
      .then(r => r.ok ? r.json() : Promise.reject(`HTTP ${r.status}`))
      .then((data: NetDiffResponse) => {
        if (mounted) { setNetDiff(data); setLoading(false); }
      })
      .catch(() => { if (mounted) setLoading(false); });
    return () => { mounted = false; };
  }, [stitchId]);

  if (loading) return <div className="touched-files-loading">Loading touched files…</div>;
  if (!netDiff) return null;

  const allFiles: Array<{ filePath: string; added: number; removed: number; refRange: string }> = [];
  for (const ws of netDiff.workspaces) {
    for (const f of ws.files) {
      const filePath = f.new_path || f.old_path;
      if (filePath) allFiles.push({ filePath, added: f.added, removed: f.removed, refRange: ws.ref_range });
    }
  }

  if (allFiles.length === 0) return null;

  return (
    <div className="touched-files-panel">
      <div className="touched-files-header">
        <span className="touched-files-title">Files touched</span>
        <div className="touched-files-summary">
          <span>{allFiles.length} file{allFiles.length !== 1 ? 's' : ''}</span>
          {netDiff.total_added > 0 && <span className="diff-stat-add">+{netDiff.total_added}</span>}
          {netDiff.total_removed > 0 && <span className="diff-stat-rem">-{netDiff.total_removed}</span>}
          {netDiff.truncated && <span className="touched-files-truncated">(truncated)</span>}
        </div>
      </div>
      <div className="touched-files-list">
        {allFiles.map(({ filePath, added, removed, refRange }) => (
          <div
            key={filePath}
            className="touched-file-row"
            onClick={(e) => { e.stopPropagation(); onFileClick(filePath, refRange); }}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === 'Enter' && onFileClick(filePath, refRange)}
          >
            <span className="touched-file-icon">📄</span>
            <span className="touched-file-path">{filePath}</span>
            <div className="touched-file-stats">
              {added > 0 && <span className="diff-stat-add">+{added}</span>}
              {removed > 0 && <span className="diff-stat-rem">-{removed}</span>}
            </div>
            <span className="touched-file-arrow">→</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── StitchesTab ──────────────────────────────────────────────────────────────

interface StitchesTabProps {
  projectName: string;
  projectPath: string;
  conversations?: Conversation[];
  onSwitchTab?: (tab: TabId) => void;
}

export default function StitchesTab({ projectName, projectPath: _projectPath, conversations: conversationsProp, onSwitchTab }: StitchesTabProps) {
  const globalConversations = useAtomValue(conversationsAtom);
  const conversations = conversationsProp ?? globalConversations;
  const streamingActiveIds = useAtomValue(streamingActiveIdsAtom);
  const setSelectedConversationId = useSetAtom(selectedConversationIdAtom);
  const setFileNavigation = useSetAtom(fileNavigationAtom);
  const dictatedNotesMap = useAtomValue(dictatedNotesAtom);
  const screenCapturesMap = useAtomValue(screenCapturesAtom);

  // Fetch dictated notes for this project on mount
  const setDictatedNotes = useSetAtom(dictatedNotesAtom);
  useEffect(() => {
    let mounted = true;
    fetch(`/api/p/${encodeURIComponent(projectName)}/dictated-notes`)
      .then(r => r.ok ? r.json() : [])
      .then((notes: NoteSummary[]) => {
        if (mounted) {
          setDictatedNotes(prev => {
            const next = new Map(prev);
            next.set(projectName, notes);
            return next;
          });
        }
      })
      .catch(() => {});
    return () => { mounted = false; };
  }, [projectName, setDictatedNotes]);

  // Fetch screen captures for this project on mount
  const setScreenCaptures = useSetAtom(screenCapturesAtom);
  useEffect(() => {
    let mounted = true;
    fetch(`/api/p/${encodeURIComponent(projectName)}/screen-captures`)
      .then(r => r.ok ? r.json() : [])
      .then((captures: ScreenCaptureSummary[]) => {
        if (mounted) {
          setScreenCaptures(prev => {
            const next = new Map(prev);
            next.set(projectName, captures);
            return next;
          });
        }
      })
      .catch(() => {});
    return () => { mounted = false; };
  }, [projectName, setScreenCaptures]);

  // Callback to update a single note in the atom (no page reload)
  const handleNoteUpdate = useCallback((updated: NoteSummary) => {
    setDictatedNotes(prev => {
      const next = new Map(prev);
      const existing = next.get(projectName) ?? [];
      next.set(projectName, existing.map(n => n.stitch_id === updated.stitch_id ? updated : n));
      return next;
    });
  }, [projectName, setDictatedNotes]);

  const dictatedNotes = dictatedNotesMap.get(projectName) ?? [];
  const screenCaptures = screenCapturesMap.get(projectName) ?? [];

  const [classificationFilter, setClassificationFilter] = useAtom(conversationFilterAtom);
  const [filter, setFilter] = useState<FilterConfig>({
    status: 'all',
    search: '',
  });
  const [page, setPage] = useState(1);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showDraftForm, setShowDraftForm] = useState(false);
  const [showStitchForm, setShowStitchForm] = useState(false);
  const [lastCreatedId, setLastCreatedId] = useState<string | null>(null);
  const [lastStitchIds, setLastStitchIds] = useState<string[] | null>(null);
  const [lastStitchId, setLastStitchId] = useState<string | null>(null);

  // Merge conversations + dictated notes into unified stitch items
  const stitchItems = useMemo(() => {
    const items: StitchItem[] = [];

    // Conversations from WebSocket
    for (const conv of conversations) {
      items.push({
        id: conv.id,
        title: conv.title,
        kind: conv.kind,
        status: conv.complete ? 'done' : 'active',
        createdAt: conv.created_at,
        lastActivityAt: conv.updated_at,
        project: projectName,
        messageCount: conv.messages.length,
        totalTokens: conv.total_tokens,
        linkedBeads: conv.worker_metadata?.bead ? [conv.worker_metadata.bead] : [],
        workerMetadata: conv.worker_metadata,
        isStreaming: streamingActiveIds.has(conv.id),
      });
    }

    // Dictated notes from REST API
    for (const note of dictatedNotes) {
      items.push({
        id: note.stitch_id,
        title: note.title,
        kind: 'dictated',
        status: 'active',
        createdAt: note.created_at,
        lastActivityAt: note.last_activity_at,
        project: note.project,
        messageCount: 0,
        totalTokens: 0,
        linkedBeads: [],
        isStreaming: false,
        dictatedNote: note,
      });
    }

    // Screen captures from REST API
    for (const cap of screenCaptures) {
      items.push({
        id: cap.stitch_id,
        title: cap.title,
        kind: 'screen-capture',
        status: 'done',
        createdAt: cap.recorded_at,
        lastActivityAt: cap.recorded_at,
        project: cap.project,
        messageCount: 0,
        totalTokens: 0,
        linkedBeads: [],
        isStreaming: false,
        screenCapture: cap,
      });
    }

    // Most recent activity at top
    return items.sort((a, b) =>
      new Date(b.lastActivityAt).getTime() - new Date(a.lastActivityAt).getTime()
    );
  }, [conversations, streamingActiveIds, projectName, dictatedNotes, screenCaptures]);

  // Reset to first page when filter changes
  useEffect(() => {
    setPage(1);
  }, [filter, classificationFilter]);

  const filteredItems = useMemo(() => {
    return stitchItems.filter(item => {
      if (classificationFilter !== 'all') {
        const matchesKind = classificationFilter === 'fleet'
          ? item.kind === 'worker'
          : item.kind === (classificationFilter as string);
        if (!matchesKind) return false;
      }
      if (filter.status !== 'all' && item.status !== filter.status) return false;
      if (filter.search) {
        const q = filter.search.toLowerCase();
        // Search title and ID
        if (item.title.toLowerCase().includes(q) || item.id.toLowerCase().includes(q)) {
          return true;
        }
        // Search transcript text for dictated notes
        if (item.kind === 'dictated' && item.dictatedNote) {
          if (item.dictatedNote.transcript?.toLowerCase().includes(q)) {
            return true;
          }
          if (item.dictatedNote.transcript_preview.toLowerCase().includes(q)) {
            return true;
          }
          // Search tags
          if (item.dictatedNote.tags.some(t => t.toLowerCase().includes(q))) {
            return true;
          }
        }
        return false;
      }
      return true;
    });
  }, [stitchItems, filter, classificationFilter]);

  const visibleItems = useMemo(() => filteredItems.slice(0, page * PAGE_SIZE), [filteredItems, page]);
  const hasMore = visibleItems.length < filteredItems.length;

  const activeCount = stitchItems.filter(i => i.status === 'active').length;
  const streamingCount = stitchItems.filter(i => i.isStreaming).length;
  const doneCount = stitchItems.filter(i => i.status === 'done').length;

  const handleCardClick = useCallback((id: string) => {
    setSelectedId(prev => {
      const next = prev === id ? null : id;
      if (next) setSelectedConversationId(next);
      return next;
    });
  }, [setSelectedConversationId]);

  // Infinite scroll sentinel
  const sentinelRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!sentinelRef.current || !hasMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) setPage(prev => prev + 1);
      },
      { threshold: 0.1 }
    );
    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [hasMore]);

  const handleBeadCreated = useCallback((beadId: string) => {
    setLastCreatedId(beadId);
    setShowDraftForm(false);
    setSelectedId(beadId);
  }, []);

  const handleStitchCreated = useCallback((beadIds: string[], stitchId?: string) => {
    setLastStitchIds(beadIds);
    setLastStitchId(stitchId ?? null);
    setShowStitchForm(false);
    if (beadIds.length > 0) setSelectedId(beadIds[0]);
  }, []);

  return (
    <div className="stitches-tab">
      {showDraftForm && (
        <BeadDraftForm
          projectName={projectName}
          onClose={() => setShowDraftForm(false)}
          onCreated={handleBeadCreated}
        />
      )}

      {showStitchForm && (
        <StitchDraftForm
          projectName={projectName}
          onClose={() => setShowStitchForm(false)}
          onCreated={handleStitchCreated}
        />
      )}

      {lastCreatedId && (
        <div className="bead-created-banner" role="status">
          Bead <strong>{lastCreatedId}</strong> created.{' '}
          <button className="bead-created-dismiss" onClick={() => setLastCreatedId(null)}>Dismiss</button>
        </div>
      )}

      {lastStitchIds && lastStitchIds.length > 0 && (
        <div className="bead-created-banner" role="status">
          {lastStitchId && (
            <>Stitch <strong>{lastStitchId}</strong>: </>
          )}
          {!lastStitchId && (
            <>Stitch created </>
          )}
          {lastStitchIds.length} bead{lastStitchIds.length !== 1 ? 's' : ''}: <strong>{lastStitchIds.join(', ')}</strong>{' '}
          {onSwitchTab && (
            <button className="bead-created-redirect" onClick={() => onSwitchTab('graph')}>
              View in Bead Graph
            </button>
          )}
          <button className="bead-created-dismiss" onClick={() => { setLastStitchIds(null); setLastStitchId(null); }}>Dismiss</button>
        </div>
      )}

      <div className="stitches-header">
        <div className="stitches-stats">
          <div className="stitch-stat">
            <span className="stat-value">{activeCount}</span>
            <span className="stat-label">Active</span>
          </div>
          {streamingCount > 0 && (
            <div className="stitch-stat stitch-stat--streaming">
              <span className="stat-value">{streamingCount}</span>
              <span className="stat-label">Live</span>
            </div>
          )}
          <div className="stitch-stat">
            <span className="stat-value">0</span>
            <span className="stat-label">Awaiting Review</span>
          </div>
          <div className="stitch-stat">
            <span className="stat-value">{doneCount}</span>
            <span className="stat-label">Done</span>
          </div>
        </div>

        <div className="stitches-filters">
          <button
            className="new-bead-btn"
            onClick={() => setShowDraftForm(true)}
            title="Draft a new bead in this project"
          >
            + New Bead
          </button>

          <button
            className="new-bead-btn new-stitch-btn"
            onClick={() => setShowStitchForm(true)}
            title="Create a new stitch with decomposition"
          >
            + New Stitch
          </button>

          <input
            type="text"
            placeholder="Search stitches..."
            value={filter.search}
            onChange={(e) => setFilter(prev => ({ ...prev, search: e.target.value }))}
            className="stitch-search"
          />

          <select
            value={classificationFilter}
            onChange={(e) => setClassificationFilter(e.target.value as ConversationFilter)}
            className="stitch-filter-select"
          >
            <option value="all">All Classifications</option>
            <option value="fleet">Fleet (Workers)</option>
            <option value="operator">Operator</option>
            <option value="ad-hoc">Ad-hoc</option>
            <option value="dictated">Dictated</option>
            <option value="screen-capture">Screen</option>
          </select>

          <select
            value={filter.status}
            onChange={(e) => setFilter(prev => ({ ...prev, status: e.target.value as StitchStatus }))}
            className="stitch-filter-select"
          >
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="awaiting_review">Awaiting Review</option>
            <option value="done">Done</option>
          </select>
        </div>
      </div>

      {filteredItems.length === 0 ? (
        <div className="stitches-empty">
          <p>No stitches found</p>
          {filter.search || filter.status !== 'all' ? (
            <button
              className="clear-filters-button"
              onClick={() => setFilter({ status: 'all', search: '' })}
            >
              Clear filters
            </button>
          ) : (
            <p className="empty-hint">Stitches will appear here as conversations happen</p>
          )}
        </div>
      ) : (
        <div className="stitches-list">
          {visibleItems.map(item => {
            const kindBadge = getKindBadge(item.kind);
            const statusBadge = getStatusBadge(item.status);
            const isSelected = selectedId === item.id;
            const selectedConv = isSelected ? conversations.find(c => c.id === item.id) : undefined;

            return (
              <div
                key={item.id}
                className={[
                  'stitch-card',
                  item.isStreaming ? 'stitch-card--streaming' : '',
                  isSelected ? 'stitch-card--selected' : '',
                ].filter(Boolean).join(' ')}
                onClick={() => handleCardClick(item.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && handleCardClick(item.id)}
                aria-expanded={isSelected}
              >
                <div className="stitch-card-header">
                  <div className="stitch-title-row">
                    {item.isStreaming && (
                      <span className="stitch-activity-dot" aria-label="Streaming" />
                    )}
                    <h3 className="stitch-title">{item.title}</h3>
                  </div>
                  <div className="stitch-badges">
                    <span className={`badge ${kindBadge.className}`}>{kindBadge.label}</span>
                    <span className={`badge ${statusBadge.className}`}>{statusBadge.label}</span>
                  </div>
                </div>

                <div className="stitch-card-meta">
                  <span className="meta-item stitch-last-activity">
                    <span className="meta-label">Last activity:</span>
                    <span className="meta-value">{formatTimeAgo(item.lastActivityAt)} ago</span>
                  </span>
                  {item.kind !== 'dictated' && item.kind !== 'screen-capture' && (
                    <span className="meta-item">
                      <span className="meta-label">Messages:</span>
                      <span className="meta-value">{item.messageCount}</span>
                    </span>
                  )}
                  {item.kind === 'dictated' && item.dictatedNote?.duration_secs != null && (
                    <span className="meta-item">
                      <span className="meta-label">Duration:</span>
                      <span className="meta-value">{Math.round(item.dictatedNote.duration_secs)}s</span>
                    </span>
                  )}
                  {item.kind === 'dictated' && item.dictatedNote && (
                    <TranscriptionStatusBadge status={item.dictatedNote.transcription_status} />
                  )}
                  {item.kind === 'screen-capture' && item.screenCapture?.duration_secs != null && (
                    <span className="meta-item">
                      <span className="meta-label">Duration:</span>
                      <span className="meta-value">{Math.round(item.screenCapture.duration_secs)}s</span>
                    </span>
                  )}
                  {item.kind === 'screen-capture' && item.screenCapture && item.screenCapture.chapter_count > 0 && (
                    <span className="meta-item">
                      <span className="meta-label">Chapters:</span>
                      <span className="meta-value">{item.screenCapture.chapter_count}</span>
                    </span>
                  )}
                  {item.totalTokens > 0 && (
                    <span className="meta-item">
                      <span className="meta-label">Tokens:</span>
                      <span className="meta-value">{item.totalTokens.toLocaleString()}</span>
                    </span>
                  )}
                  {item.linkedBeads.length > 0 && (
                    <span className="meta-item">
                      <span className="meta-label">Beads:</span>
                      <span className="meta-value">{item.linkedBeads.length}</span>
                    </span>
                  )}
                </div>

                {item.workerMetadata && (
                  <div className="stitch-worker-info">
                    <span className="worker-badge">
                      Worker: {item.workerMetadata.worker}
                    </span>
                    {item.workerMetadata.bead && (
                      <span className="bead-badge">{item.workerMetadata.bead}</span>
                    )}
                  </div>
                )}

                {isSelected && item.kind === 'dictated' && item.dictatedNote && (
                  <div className="stitch-detail" onClick={(e) => e.stopPropagation()}>
                    <div className="stitch-detail-header">
                      <span className="stitch-detail-id">{item.id}</span>
                      <span className="stitch-detail-created">
                        Created {formatTimeAgo(item.createdAt)} ago
                      </span>
                    </div>
                    <DictatedNoteDetail note={item.dictatedNote} onUpdate={handleNoteUpdate} />
                  </div>
                )}

                {isSelected && item.kind === 'screen-capture' && item.screenCapture && (
                  <div className="stitch-detail" onClick={(e) => e.stopPropagation()}>
                    <div className="stitch-detail-header">
                      <span className="stitch-detail-id">{item.id}</span>
                      <span className="stitch-detail-created">
                        Created {formatTimeAgo(item.createdAt)} ago
                      </span>
                    </div>
                    <ScreenCaptureDetail summary={item.screenCapture} />
                  </div>
                )}

                {isSelected && selectedConv && item.kind !== 'dictated' && item.kind !== 'screen-capture' && (
                  <div className="stitch-detail" onClick={(e) => e.stopPropagation()}>
                    <div className="stitch-detail-header">
                      <span className="stitch-detail-id">{item.id}</span>
                      <span className="stitch-detail-created">
                        Created {formatTimeAgo(item.createdAt)} ago
                      </span>
                    </div>
                    <div className="stitch-detail-messages">
                      {selectedConv.messages.slice(-5).map((msg, i) => (
                        <div
                          key={i}
                          className={`stitch-detail-message stitch-detail-message--${msg.role}`}
                        >
                          <span className="stitch-detail-role">{msg.role}</span>
                          <span className="stitch-detail-content">
                            {typeof msg.content === 'string'
                              ? msg.content.slice(0, 200) + (msg.content.length > 200 ? '…' : '')
                              : JSON.stringify(msg.content).slice(0, 200)}
                          </span>
                        </div>
                      ))}
                      {selectedConv.messages.length > 5 && (
                        <p className="stitch-detail-truncated">
                          +{selectedConv.messages.length - 5} earlier messages
                        </p>
                      )}
                      {selectedConv.messages.length === 0 && (
                        <p className="stitch-detail-truncated">No messages yet</p>
                      )}
                    </div>
                    <TouchedFilesPanel
                      stitchId={item.id}
                      onFileClick={(filePath, refRange) => {
                        setFileNavigation({ projectName, filePath, refRange });
                        onSwitchTab?.('files');
                      }}
                    />
                  </div>
                )}
              </div>
            );
          })}

          {hasMore && (
            <div ref={sentinelRef} className="stitches-load-sentinel">
              <button
                className="stitch-load-more"
                onClick={() => setPage(prev => prev + 1)}
              >
                Load more ({filteredItems.length - visibleItems.length} remaining)
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
