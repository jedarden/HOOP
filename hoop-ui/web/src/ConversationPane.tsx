import { useAtom, useAtomValue } from 'jotai';
import { useState, useMemo } from 'react';
import {
  conversationsAtom,
  streamingContentFamily,
  selectedConversationIdAtom,
  workersAtom,
  beadsAtom,
  Conversation,
  DictatedNote,
  getSessionKindBadge,
  getAdapterAndModel,
  SessionMessage,
  formatContent,
} from './atoms';
import AudioPlayer from './components/AudioPlayer';

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return date.toLocaleDateString();
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

function DictatedNotePlayer({ note }: { note: DictatedNote }) {
  const transcript = note.transcript_words.length > 0
    ? { text: note.transcript, words: note.transcript_words }
    : undefined;

  return (
    <div className="dictated-note-player">
      <div className="dictated-note-meta">
        <span className="dictated-note-label">Voice Note</span>
        {note.language && (
          <span className="dictated-note-lang">{note.language}</span>
        )}
        <span className="dictated-note-date">
          {new Date(note.recorded_at).toLocaleString()}
        </span>
      </div>
      <AudioPlayer audioUrl={note.audio_url} transcript={transcript} />
    </div>
  );
}

function MessageBubble({ message }: { message: SessionMessage }) {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  return (
    <div className={`message-bubble ${isUser ? 'message-user' : 'message-assistant'} ${isSystem ? 'message-system' : ''}`}>
      <div className="message-header">
        <span className="message-role">
          {isSystem ? 'System' : isUser ? 'User' : 'Assistant'}
        </span>
        {message.timestamp && (
          <span className="message-time">{formatTimestamp(message.timestamp)}</span>
        )}
      </div>
      <div className="message-content">
        <pre className="message-text">{formatContent(message.content)}</pre>
      </div>
      {message.usage && (
        <div className="message-usage">
          <span className="usage-label">tokens:</span>
          <span className="usage-value">{formatTokens(message.usage.input_tokens + message.usage.output_tokens)}</span>
        </div>
      )}
    </div>
  );
}

function ConversationView({ conversation }: { conversation: Conversation }) {
  const workers = useAtomValue(workersAtom);
  const beads = useAtomValue(beadsAtom);
  // Per-conversation atom: only re-renders this view when THIS conversation streams.
  // Token deltas on other conversations don't cause re-renders here.
  // eslint-disable-next-line deprecation/deprecation
  const streamingText = useAtomValue(streamingContentFamily(conversation.id));

  const badge = getSessionKindBadge(conversation.kind, conversation.worker_metadata);
  const adapterModel = conversation.worker_metadata
    ? getAdapterAndModel(workers, conversation.worker_metadata.worker)
    : { adapter: conversation.provider, model: null };

  const bead = beads.find(b => b.id === conversation.worker_metadata?.bead);

  return (
    <div className="conversation-view">
      <div className="conversation-header">
        <div className="conversation-title-row">
          <h3 className="conversation-title">{conversation.title}</h3>
          <span className={`badge ${badge.className}`}>{badge.label}</span>
        </div>
        <div className="conversation-meta">
          <span className="meta-item">
            <span className="meta-label">project:</span>
            <span className="meta-value">{conversation.cwd.split('/').pop() || conversation.cwd}</span>
          </span>
          {bead && (
            <span className="meta-item expert-view">
              <span className="meta-label">bead:</span>
              <span className="meta-value">{bead.id}</span>
            </span>
          )}
          <span className="meta-item">
            <span className="meta-label">adapter:</span>
            <span className="meta-value">{adapterModel.adapter}</span>
            {adapterModel.model && <span className="meta-value"> · {adapterModel.model}</span>}
          </span>
          <span className="meta-item">
            <span className="meta-label">tokens:</span>
            <span className="meta-value">{formatTokens(conversation.total_tokens)}</span>
          </span>
          <span className="meta-item">
            <span className="meta-label">updated:</span>
            <span className="meta-value">{formatTimestamp(conversation.updated_at)}</span>
          </span>
          {!conversation.complete && (
            <span className="meta-item status-indicator">
              <span className="status-dot live" />
              <span className="status-label">live</span>
            </span>
          )}
        </div>
      </div>

      <div className="conversation-messages">
        {conversation.kind === 'dictated' && conversation.dictated_note && (
          <DictatedNotePlayer note={conversation.dictated_note} />
        )}
        {conversation.messages.map((message, index) => (
          <MessageBubble
            key={index}
            message={message}
          />
        ))}
        {streamingText.length > 0 && (
          <div className="message-bubble message-assistant streaming">
            <div className="message-header">
              <span className="message-role">Assistant</span>
              <span className="message-time">now</span>
            </div>
            <div className="message-content">
              <pre className="message-text">{streamingText}</pre>
              <span className="streaming-indicator">▋</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ConversationListItem({
  conversation,
  isSelected,
  onClick,
}: {
  conversation: Conversation;
  isSelected: boolean;
  onClick: () => void;
}) {
  const badge = getSessionKindBadge(conversation.kind, conversation.worker_metadata);

  return (
    <div
      className={`conversation-list-item ${isSelected ? 'selected' : ''}`}
      onClick={onClick}
    >
      <div className="conversation-item-header">
        <span className={`badge ${badge.className} badge-sm`}>{badge.label}</span>
        <span className="conversation-time">{formatTimestamp(conversation.updated_at)}</span>
      </div>
      <h4 className="conversation-item-title">{conversation.title}</h4>
      <div className="conversation-item-meta">
        <span className="meta-project">{conversation.cwd.split('/').pop() || conversation.cwd}</span>
        <span className="meta-tokens">{formatTokens(conversation.total_tokens)} tokens</span>
      </div>
      {!conversation.complete && (
        <div className="conversation-item-status">
          <span className="status-dot live" />
          <span>live</span>
        </div>
      )}
    </div>
  );
}

interface ConversationPaneProps {
  conversations?: Conversation[];
}

export default function ConversationPane({ conversations: conversationsProp }: ConversationPaneProps) {
  const [globalConversations] = useAtom(conversationsAtom);
  const conversations = conversationsProp ?? globalConversations;
  const [selectedConversationId, setSelectedConversationId] = useAtom(selectedConversationIdAtom);
  const [filter, setFilter] = useState<'all' | 'fleet' | 'operator' | 'ad-hoc' | 'dictated'>('all');

  const filteredConversations = useMemo(() => {
    if (filter === 'all') return conversations;
    return conversations.filter(c => {
      if (filter === 'fleet') return c.kind === 'worker';
      if (filter === 'operator') return c.kind === 'operator';
      if (filter === 'ad-hoc') return c.kind === 'ad-hoc';
      if (filter === 'dictated') return c.kind === 'dictated';
      return true;
    });
  }, [conversations, filter]);

  const selectedConversation = conversations.find(c => c.id === selectedConversationId);

  return (
    <section className="conversation-section">
      <div className="conversation-pane">
        <div className="conversation-list">
          <div className="conversation-list-header">
            <h2>Conversations</h2>
            <div className="filter-tabs">
              <button
                className={`filter-tab ${filter === 'all' ? 'active' : ''}`}
                onClick={() => setFilter('all')}
              >
                All ({conversations.length})
              </button>
              <button
                className={`filter-tab ${filter === 'fleet' ? 'active' : ''}`}
                onClick={() => setFilter('fleet')}
              >
                Fleet
              </button>
              <button
                className={`filter-tab ${filter === 'operator' ? 'active' : ''}`}
                onClick={() => setFilter('operator')}
              >
                Operator
              </button>
              <button
                className={`filter-tab ${filter === 'ad-hoc' ? 'active' : ''}`}
                onClick={() => setFilter('ad-hoc')}
              >
                Ad-hoc
              </button>
              <button
                className={`filter-tab ${filter === 'dictated' ? 'active' : ''}`}
                onClick={() => setFilter('dictated')}
              >
                Dictated
              </button>
            </div>
          </div>
          <div className="conversation-list-items">
            {filteredConversations.map((conversation) => (
              <ConversationListItem
                key={conversation.id}
                conversation={conversation}
                isSelected={conversation.id === selectedConversationId}
                onClick={() => setSelectedConversationId(conversation.id)}
              />
            ))}
            {filteredConversations.length === 0 && (
              <div className="conversation-list-empty">
                <p>No conversations found.</p>
              </div>
            )}
          </div>
        </div>

        <div className="conversation-detail">
          {selectedConversation ? (
            <ConversationView conversation={selectedConversation} />
          ) : (
            <div className="conversation-detail-empty">
              <p>Select a conversation to view its transcript.</p>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
