import { useState, useRef, useEffect, useCallback } from 'react';
import { useAtom, useAtomValue } from 'jotai';
import {
  agentSessionStatusAtom,
  agentInflightAtom,
  agentChatMessagesAtom,
  agentChatScopeAtom,
  projectCardsAtom,
  AgentChatMessage,
  AgentToolCallInProgress,
} from './atoms';

interface PendingAttachment {
  id: string;
  name: string;
  size: number;
  file?: File;
  previewUrl?: string;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Per-adapter upload size caps enforced at the UI layer.
const ADAPTER_CAPS: Record<string, { maxFileSizeBytes: number; maxTotalBytes: number }> = {
  anthropic: { maxFileSizeBytes: 5 * 1024 * 1024,  maxTotalBytes: 20 * 1024 * 1024 },
  zai:       { maxFileSizeBytes: 20 * 1024 * 1024, maxTotalBytes: 20 * 1024 * 1024 },
  gemini:    { maxFileSizeBytes: 20 * 1024 * 1024, maxTotalBytes: 20 * 1024 * 1024 },
  claude:    { maxFileSizeBytes: 50 * 1024 * 1024, maxTotalBytes: 100 * 1024 * 1024 },
  codex:     { maxFileSizeBytes: 50 * 1024 * 1024, maxTotalBytes: 100 * 1024 * 1024 },
  opencode:  { maxFileSizeBytes: 50 * 1024 * 1024, maxTotalBytes: 100 * 1024 * 1024 },
};

function getAdapterCap(adapter: string | null) {
  const key = (adapter ?? 'claude').toLowerCase();
  return ADAPTER_CAPS[key] ?? ADAPTER_CAPS.claude;
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve((reader.result as string).split(',')[1]);
    reader.onerror = () => reject(new Error(`Failed to read ${file.name}`));
    reader.readAsDataURL(file);
  });
}

function ToolCallBubble({ toolCall }: { toolCall: AgentToolCallInProgress }) {
  const [expanded, setExpanded] = useState(false);
  const isPending = toolCall.status === 'pending';

  return (
    <div className={`acp-tool-call ${isPending ? 'acp-tool-pending' : ''}`}>
      <button
        className="acp-tool-call-header"
        onClick={() => setExpanded((v) => !v)}
        aria-expanded={expanded}
      >
        <span className="acp-tool-icon" aria-hidden="true">⚙</span>
        <span className="acp-tool-name">{toolCall.name}</span>
        {isPending && <span className="acp-tool-spinner" aria-label="running" />}
        {!isPending && toolCall.is_error && (
          <span className="acp-tool-error-badge" aria-label="error">✕</span>
        )}
        {!isPending && !toolCall.is_error && (
          <span className="acp-tool-ok-badge" aria-label="ok">✓</span>
        )}
        <span className="acp-tool-toggle" aria-hidden="true">{expanded ? '▲' : '▼'}</span>
      </button>
      {expanded && (
        <div className="acp-tool-body">
          <div className="acp-tool-section">
            <span className="acp-tool-label">Input</span>
            <pre className="acp-tool-json">{JSON.stringify(toolCall.input, null, 2)}</pre>
          </div>
          {toolCall.output !== undefined && (
            <div className="acp-tool-section">
              <span className={`acp-tool-label ${toolCall.is_error ? 'acp-tool-label-error' : ''}`}>
                {toolCall.is_error ? 'Error' : 'Output'}
              </span>
              <pre className={`acp-tool-json ${toolCall.is_error ? 'acp-tool-json-error' : ''}`}>
                {JSON.stringify(toolCall.output, null, 2)}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ChatMessage({ message }: { message: AgentChatMessage }) {
  const isUser = message.role === 'user';
  return (
    <div
      className={`acp-message ${isUser ? 'acp-message-user' : 'acp-message-assistant'}`}
      role="article"
      aria-label={`${isUser ? 'You' : 'Agent'}: ${message.content.slice(0, 80)}`}
    >
      <span className="acp-role">{isUser ? 'You' : 'Agent'}</span>
      <div className="acp-content">
        <pre className="acp-message-text">{message.content}</pre>
        {message.attachments && message.attachments.length > 0 && (
          <div className="acp-msg-attachments">
            {message.attachments.map((name) => (
              <span key={name} className="acp-msg-attachment-chip">📎 {name}</span>
            ))}
          </div>
        )}
        {message.tool_calls && message.tool_calls.length > 0 && (
          <div className="acp-tool-calls">
            {message.tool_calls.map((tc) => (
              <ToolCallBubble key={tc.id} toolCall={tc} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default function AgentChatPane() {
  const [agentStatus, setAgentStatus] = useAtom(agentSessionStatusAtom);
  const [messages, setMessages] = useAtom(agentChatMessagesAtom);
  const inflight = useAtomValue(agentInflightAtom);
  const [scope, setScope] = useAtom(agentChatScopeAtom);
  const projectCards = useAtomValue(projectCardsAtom);

  const [input, setInput] = useState('');
  const [isSending, setIsSending] = useState(false);
  const [sendError, setSendError] = useState<string | null>(null);
  const [attachments, setAttachments] = useState<PendingAttachment[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Cleanup object URLs on unmount
  const attachmentsRef = useRef<PendingAttachment[]>([]);
  useEffect(() => { attachmentsRef.current = attachments; }, [attachments]);
  useEffect(() => {
    return () => {
      attachmentsRef.current.forEach(a => { if (a.previewUrl) URL.revokeObjectURL(a.previewUrl); });
    };
  }, []);

  // Fetch initial agent status on mount
  useEffect(() => {
    fetch('/api/agent/status')
      .then((r) => r.ok ? r.json() : null)
      .then((status) => {
        if (status) setAgentStatus(status);
      })
      .catch(() => { /* daemon may not be running */ });
  }, [setAgentStatus]);

  // Auto-scroll to bottom when messages or inflight change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, inflight]);

  // Auto-resize textarea as user types
  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleSpawn = useCallback(async () => {
    try {
      const r = await fetch('/api/agent/spawn', { method: 'POST' });
      if (!r.ok) setSendError('Failed to start session');
    } catch {
      setSendError('Failed to start session');
    }
  }, []);

  const handleDisable = useCallback(async () => {
    try {
      await fetch('/api/agent/disable', { method: 'POST' });
    } catch {
      /* best-effort */
    }
  }, []);

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || isSending || !agentStatus?.active) return;

    setSendError(null);

    // Encode pending attachments as base64 before clearing state.
    let encodedAttachments: { name: string; content: string; mime: string }[] = [];
    if (attachments.length > 0) {
      try {
        encodedAttachments = await Promise.all(
          attachments.map(async (a) => ({
            name: a.name,
            content: await readFileAsBase64(a.file!),
            mime: a.file?.type || 'application/octet-stream',
          }))
        );
      } catch (err) {
        setSendError(`Failed to read attachment: ${err instanceof Error ? err.message : String(err)}`);
        return;
      }
    }

    const userMsg: AgentChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: Date.now(),
      session_id: agentStatus.session_id ?? '',
      attachments: attachments.length > 0 ? attachments.map((a) => a.name) : undefined,
    };
    setMessages((prev) => [...prev, userMsg]);
    setInput('');
    setAttachments([]);
    setIsSending(true);

    try {
      const r = await fetch('/api/agent/turn', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          prompt: text,
          ...(encodedAttachments.length > 0 && { attachments: encodedAttachments }),
        }),
      });
      if (!r.ok) {
        const body = await r.text().catch(() => '');
        setSendError(`Turn failed (${r.status})${body ? ': ' + body : ''}`);
      }
    } catch {
      setSendError('Network error sending turn');
    } finally {
      setIsSending(false);
    }
  }, [input, isSending, agentStatus, attachments, setMessages]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }, [handleSend]);

  const handleAttach = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files ?? []);
    const cap = getAdapterCap(agentStatus?.adapter ?? null);
    const oversized = files.filter(f => f.size > cap.maxFileSizeBytes);
    if (oversized.length > 0) {
      setSendError(`File too large: ${oversized.map(f => f.name).join(', ')} (max ${formatBytes(cap.maxFileSizeBytes)} per file for ${agentStatus?.adapter ?? 'claude'} adapter)`);
      if (fileInputRef.current) fileInputRef.current.value = '';
      return;
    }
    setAttachments((prev) => {
      const next = [
        ...prev,
        ...files.map((f) => ({
          id: crypto.randomUUID(),
          name: f.name,
          size: f.size,
          file: f,
          previewUrl: f.type.startsWith('image/') ? URL.createObjectURL(f) : undefined,
        })),
      ];
      const totalSize = next.reduce((s, a) => s + a.size, 0);
      if (totalSize > cap.maxTotalBytes) {
        setSendError(`Total attachments exceed ${formatBytes(cap.maxTotalBytes)} limit for ${agentStatus?.adapter ?? 'claude'} adapter`);
        return prev;
      }
      return next;
    });
    if (fileInputRef.current) fileInputRef.current.value = '';
  }, [agentStatus?.adapter]);

  const removeAttachment = useCallback((id: string) => {
    setAttachments((prev) => {
      const item = prev.find(a => a.id === id);
      if (item?.previewUrl) URL.revokeObjectURL(item.previewUrl);
      return prev.filter((a) => a.id !== id);
    });
  }, []);

  const handleTextareaPaste = useCallback((e: React.ClipboardEvent<HTMLTextAreaElement>) => {
    const items = e.clipboardData?.items;
    if (!items) return;
    const imageFiles: File[] = [];
    for (const item of Array.from(items)) {
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile();
        if (file) imageFiles.push(new File([file], file.name || `image-${Date.now()}.png`, { type: file.type }));
      }
    }
    if (imageFiles.length > 0) {
      e.preventDefault();
      setAttachments((prev) => [
        ...prev,
        ...imageFiles.map((f) => ({
          id: crypto.randomUUID(),
          name: f.name,
          size: f.size,
          file: f,
          previewUrl: URL.createObjectURL(f),
        })),
      ]);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length === 0) return;
    const cap = getAdapterCap(agentStatus?.adapter ?? null);
    const oversized = files.filter(f => f.size > cap.maxFileSizeBytes);
    if (oversized.length > 0) {
      setSendError(`File too large: ${oversized.map(f => f.name).join(', ')} (max ${formatBytes(cap.maxFileSizeBytes)} per file)`);
      return;
    }
    setAttachments((prev) => {
      const next = [
        ...prev,
        ...files.map((f) => ({
          id: crypto.randomUUID(),
          name: f.name,
          size: f.size,
          file: f,
          previewUrl: f.type.startsWith('image/') ? URL.createObjectURL(f) : undefined,
        })),
      ];
      const totalSize = next.reduce((s, a) => s + a.size, 0);
      if (totalSize > cap.maxTotalBytes) {
        setSendError(`Total attachments exceed ${formatBytes(cap.maxTotalBytes)} limit`);
        return prev;
      }
      return next;
    });
  }, [agentStatus?.adapter]);

  const toggleProjectScope = useCallback((name: string) => {
    setScope((prev) => ({
      projects: prev.projects.includes(name)
        ? prev.projects.filter((p) => p !== name)
        : [...prev.projects, name],
    }));
  }, [setScope]);

  const clearScope = useCallback(() => setScope({ projects: [] }), [setScope]);

  const isActive = agentStatus?.active ?? false;
  const isConfigured = agentStatus !== null;

  return (
    <section className="agent-chat-pane" aria-label="Agent Chat">
      {/* Header */}
      <div className="acp-header">
        <div className="acp-title-row">
          <h2>Agent Chat</h2>
          <div className="acp-session-controls">
            {isActive ? (
              <>
                <div className="acp-status-badge active" aria-label="Session active">
                  <span className="acp-status-dot" aria-hidden="true" />
                  <span>{agentStatus?.adapter ?? 'claude'}</span>
                  {agentStatus?.model && (
                    <span className="acp-model-tag">· {agentStatus.model.replace('claude-', '').replace(/-\d{8}$/, '')}</span>
                  )}
                  {agentStatus && agentStatus.turn_count > 0 && (
                    <span className="acp-turn-count">{agentStatus.turn_count} turns</span>
                  )}
                </div>
                <button className="acp-btn-sm acp-btn-disable" onClick={handleDisable}>
                  Disable
                </button>
              </>
            ) : (
              <>
                <div className="acp-status-badge inactive" aria-label="No active session">
                  <span className="acp-status-dot" aria-hidden="true" />
                  <span>{isConfigured ? 'Inactive' : 'Not configured'}</span>
                </div>
                {isConfigured && (
                  <button className="acp-btn-sm acp-btn-spawn" onClick={handleSpawn}>
                    Start Session
                  </button>
                )}
              </>
            )}
          </div>
        </div>

        {/* Project context scope bar */}
        <div className="acp-scope-bar" role="group" aria-label="Project scope">
          <span className="acp-scope-label">Scope</span>
          <div className="acp-scope-pills">
            <button
              className={`acp-scope-pill ${scope.projects.length === 0 ? 'acp-scope-active' : ''}`}
              onClick={clearScope}
              aria-pressed={scope.projects.length === 0}
            >
              All Projects
            </button>
            {projectCards.map((p) => (
              <button
                key={p.name}
                className={`acp-scope-pill ${scope.projects.includes(p.name) ? 'acp-scope-active' : ''}`}
                onClick={() => toggleProjectScope(p.name)}
                aria-pressed={scope.projects.includes(p.name)}
                style={p.color ? { '--pill-accent': p.color } as React.CSSProperties : undefined}
              >
                {p.color && (
                  <span className="acp-pill-dot" style={{ background: p.color }} aria-hidden="true" />
                )}
                {p.label || p.name}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Message list */}
      <div className="acp-messages" role="log" aria-label="Conversation" aria-live="polite">
        {messages.length === 0 && !inflight && (
          <div className="acp-empty" aria-label="No messages yet">
            {isActive
              ? 'Send a message to start the conversation.'
              : isConfigured
                ? 'Start an agent session to begin.'
                : 'Agent not configured in hoop config.'}
          </div>
        )}

        {messages.map((msg) => (
          <ChatMessage key={msg.id} message={msg} />
        ))}

        {/* In-flight response — separate reactive atom for real-time isolation */}
        {inflight && (
          <div className="acp-message acp-message-assistant acp-message-inflight" role="article">
            <span className="acp-role">Agent</span>
            <div className="acp-content">
              {inflight.tool_calls.length > 0 && (
                <div className="acp-tool-calls">
                  {inflight.tool_calls.map((tc) => (
                    <ToolCallBubble key={tc.id} toolCall={tc} />
                  ))}
                </div>
              )}
              {inflight.text && (
                <pre className="acp-message-text acp-inflight-text">
                  {inflight.text}
                  <span className="acp-cursor" aria-hidden="true">▌</span>
                </pre>
              )}
              {!inflight.text && inflight.tool_calls.length === 0 && (
                <span className="acp-cursor acp-cursor-standalone" aria-hidden="true">▌</span>
              )}
            </div>
          </div>
        )}

        {/* Thinking indicator: sent but no streaming yet */}
        {isSending && !inflight && (
          <div className="acp-message acp-message-assistant acp-thinking" role="article" aria-label="Agent thinking">
            <span className="acp-role">Agent</span>
            <div className="acp-content">
              <span className="acp-thinking-dots" aria-hidden="true">
                <span className="dot" />
                <span className="dot" />
                <span className="dot" />
              </span>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Error banner */}
      {sendError && (
        <div className="acp-error-bar" role="alert">
          <span>{sendError}</span>
          <button className="acp-error-dismiss" onClick={() => setSendError(null)} aria-label="Dismiss error">
            ×
          </button>
        </div>
      )}

      {/* Input area */}
      <div
        className={`acp-input-area${isDragOver ? ' acp-drag-over' : ''}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {attachments.length > 0 && (
          <div className="acp-attachments" role="list" aria-label="Pending attachments">
            {attachments.map((a) => (
              <div key={a.id} className="acp-attachment-chip" role="listitem">
                {a.previewUrl ? (
                  <img src={a.previewUrl} alt={a.name} className="acp-attachment-preview" />
                ) : (
                  <span className="acp-attachment-icon" aria-hidden="true">📎</span>
                )}
                <span className="acp-attachment-name">{a.name}</span>
                <span className="acp-attachment-size">{formatBytes(a.size)}</span>
                <button
                  className="acp-attachment-remove"
                  onClick={() => removeAttachment(a.id)}
                  aria-label={`Remove ${a.name}`}
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        )}

        <div className="acp-input-row">
          {/* Attachment picker — phase 3 integration point */}
          <button
            className="acp-attach-btn"
            onClick={handleAttach}
            title="Attach file"
            aria-label="Attach file"
            disabled={!isActive}
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
            </svg>
          </button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            style={{ display: 'none' }}
            onChange={handleFileChange}
            accept="image/*,text/*,.pdf,.json,.csv,.md,.txt,.py,.rs,.ts,.tsx,.js"
            aria-hidden="true"
            tabIndex={-1}
          />

          <textarea
            ref={textareaRef}
            className="acp-textarea"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            onPaste={handleTextareaPaste}
            placeholder={
              isActive
                ? 'Message the agent… (Enter to send, Shift+Enter for new line)'
                : isConfigured
                  ? 'Start a session to chat'
                  : 'Agent not configured'
            }
            disabled={!isActive || isSending}
            rows={1}
            aria-label="Message input"
            aria-multiline="true"
          />

          <button
            className="acp-send-btn"
            onClick={handleSend}
            disabled={!isActive || !input.trim() || isSending}
            aria-label="Send message"
            title="Send (Enter)"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          </button>
        </div>

        {isActive && agentStatus && (
          <div className="acp-cost-hint">
            ${agentStatus.cost_usd.toFixed(4)} · {(agentStatus.input_tokens + agentStatus.output_tokens).toLocaleString()} tokens
          </div>
        )}
      </div>
    </section>
  );
}
