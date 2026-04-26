import { useState, useCallback, useRef, useEffect } from 'react';
import { useAtomValue } from 'jotai';
import { mutationsDisabledAtom } from './atoms';
import StitchDraftForm, { StitchKind } from './StitchDraftForm';

interface ChatMessage {
  id: string;
  role: 'user' | 'system';
  content: string;
  timestamp: number;
}

interface ParsedDraft {
  kind: StitchKind;
  title: string;
  description: string;
  priority: string;
  labels: string[];
  hasAcceptanceCriteria: boolean;
  confidence: number;
  explanation?: string;
}

interface ChatToStitchPaneProps {
  projectName: string;
  onStitchCreated: (beadIds: string[], stitchId?: string) => void;
}

// Rule-based NL parser for Phase 4 (lightweight, pre-agent)
function parseIntentToDraft(input: string): ParsedDraft {
  const text = input.trim();
  const lines = text.split('\n').filter(l => l.trim());
  const firstLine = lines[0] || '';
  const rest = lines.slice(1).join('\n').trim();

  // Default values
  let kind: StitchKind = 'task';
  let title = firstLine;
  let description = rest;
  const labels: string[] = [];
  let priority = '';
  let hasAcceptanceCriteria = false;
  let confidence = 0.5;
  const explanation: string[] = [];

  // Detect kind from keywords
  // Fix/bug detection
  if (/\b(fix|bug|broken|crash|error|issue|fail|broken|doesn't work|not working)\b/i.test(firstLine)) {
    kind = 'fix';
    explanation.push('Detected "fix" kind from bug/fix keywords');
    confidence += 0.2;
  }

  // Investigation detection
  if (/\b(investigate|explore|look into|debug|figure out|why|what's causing|understand)\b/i.test(firstLine)) {
    kind = 'investigation';
    explanation.push('Detected "investigation" kind from exploration keywords');
    confidence += 0.2;
  }

  // Review detection
  if (/\b(review|audit|check|verify|validate|inspect)\b/i.test(firstLine)) {
    kind = 'review';
    explanation.push('Detected "review" kind from review keywords');
    confidence += 0.2;
  }

  // Genesis detection (project/feature implementation)
  if (/\b(genesis|implement|build from scratch|new project|create.*from start)\b/i.test(firstLine)) {
    kind = 'genesis';
    explanation.push('Detected "genesis" kind from implementation keywords');
    confidence += 0.2;
  }

  // Extract title more intelligently
  const titleMatch = firstLine.match(/^(?:create|build|fix|implement|add|investigate|review|genesis|task|feature)\s+(.+)/i);
  if (titleMatch) {
    title = titleMatch[1].trim();
    explanation.push('Extracted title from action verb');
    confidence += 0.1;
  }

  // Extract labels from hashtags
  const hashtagMatches = text.match(/#(\w[\w-]*)/g);
  if (hashtagMatches) {
    labels.push(...hashtagMatches.map(h => h.slice(1)));
    explanation.push(`Extracted ${labels.length} label(s) from hashtags`);
    confidence += 0.1;
  }

  // Extract assignee from @mentions (store as label for now)
  const mentionMatches = text.match(/@(\w[\w.-]*)/g);
  if (mentionMatches) {
    labels.push(...mentionMatches.map(m => `assignee:${m.slice(1)}`));
    explanation.push('Detected assignee from @mention');
  }

  // Detect acceptance criteria
  if (/\b(should|must|require|verify that|ensure that|validate)\b/i.test(text)) {
    hasAcceptanceCriteria = true;
    explanation.push('Detected acceptance criteria from modal verbs');
    confidence += 0.1;
  }

  // Detect priority
  if (/\b(urgent|asap|critical|immediate)\b/i.test(text)) {
    priority = '0';
    explanation.push('Detected high priority from urgency keywords');
  } else if (/\b(low priority|backlog|eventually|when possible)\b/i.test(text)) {
    priority = '3';
    explanation.push('Detected low priority from de-prioritization keywords');
  }

  // Build description if not explicitly provided
  if (!description) {
    // Look for description after common patterns
    const descMatch = text.match(/(?:to|:|-)\s+(.+)/is);
    if (descMatch) {
      description = descMatch[1].trim();
    }
  }

  // Remove hashtags and mentions from description
  description = description
    .replace(/#\w[\w-]*/g, '')
    .replace(/@\w[\w.-]*/g, '')
    .trim();

  // Cap confidence
  confidence = Math.min(confidence, 1.0);

  return {
    kind,
    title: title || 'Untitled',
    description: description || '',
    priority,
    labels,
    hasAcceptanceCriteria,
    confidence,
    explanation: explanation.length > 0 ? explanation.join('. ') : undefined,
  };
}

function ChatMessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === 'user';

  return (
    <div className={`csp-message ${isUser ? 'csp-message-user' : 'csp-message-system'}`}>
      <span className="csp-role">{isUser ? 'You' : 'System'}</span>
      <div className="csp-content">
        <pre className="csp-message-text">{message.content}</pre>
      </div>
    </div>
  );
}

function DraftPreviewCard({
  draft,
  onEdit,
  onDiscard
}: {
  draft: ParsedDraft;
  onEdit: () => void;
  onDiscard: () => void;
}) {
  return (
    <div className="csp-draft-preview">
      <div className="csp-draft-header">
        <span className="csp-draft-title">Draft Stitch</span>
        <div className="csp-draft-badges">
          <span className={`csp-kind-badge csp-kind-${draft.kind}`}>
            {draft.kind}
          </span>
          {draft.confidence < 0.7 && (
            <span className="csp-confidence-warning">
              Low confidence ({Math.round(draft.confidence * 100)}%)
            </span>
          )}
        </div>
      </div>

      <div className="csp-draft-body">
        <div className="csp-draft-field">
          <span className="csp-field-label">Title</span>
          <span className="csp-field-value">{draft.title}</span>
        </div>

        {draft.description && (
          <div className="csp-draft-field">
            <span className="csp-field-label">Description</span>
            <span className="csp-field-value csp-field-desc">{draft.description.slice(0, 200)}{draft.description.length > 200 ? '…' : ''}</span>
          </div>
        )}

        <div className="csp-draft-meta">
          {draft.priority !== '' && (
            <span className="csp-meta-item">Priority: {draft.priority}</span>
          )}
          {draft.labels.length > 0 && (
            <span className="csp-meta-item">
              Labels: {draft.labels.map(l => `#${l}`).join(' ')}
            </span>
          )}
          {draft.hasAcceptanceCriteria && (
            <span className="csp-meta-item">Has acceptance criteria</span>
          )}
        </div>

        {draft.explanation && (
          <div className="csp-draft-explanation">
            <span className="csp-explanation-label">Parsed:</span>
            <span className="csp-explanation-text">{draft.explanation}</span>
          </div>
        )}
      </div>

      <div className="csp-draft-actions">
        <button
          className="csp-btn csp-btn-discard"
          onClick={onDiscard}
        >
          Discard
        </button>
        <button
          className="csp-btn csp-btn-edit"
          onClick={onEdit}
        >
          Edit & Submit
        </button>
      </div>
    </div>
  );
}

export default function ChatToStitchPane({ projectName, onStitchCreated }: ChatToStitchPaneProps) {
  const mutationsDisabled = useAtomValue(mutationsDisabledAtom);

  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: 'welcome',
      role: 'system',
      content: 'Describe what you want to do, and I\'ll create a stitch draft for you to review. You can use hashtags for labels (#urgent) and @mentions for assignees.',
      timestamp: Date.now(),
    },
  ]);
  const [input, setInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [currentDraft, setCurrentDraft] = useState<ParsedDraft | null>(null);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, currentDraft]);

  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 150)}px`;
    }
  }, [input]);

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || isProcessing || mutationsDisabled) return;

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: Date.now(),
    };
    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setIsProcessing(true);

    // Simulate brief processing delay for UX
    await new Promise(resolve => setTimeout(resolve, 400));

    const draft = parseIntentToDraft(text);
    setCurrentDraft(draft);

    const systemMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'system',
      content: `I've parsed this as a ${draft.kind} stitch. Please review the draft below - you can edit any details before submitting.`,
      timestamp: Date.now(),
    };
    setMessages(prev => [...prev, systemMsg]);
    setIsProcessing(false);
  }, [input, isProcessing, mutationsDisabled]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }, [handleSend]);

  const handleEditDraft = useCallback(() => {
    // Will open the StitchDraftForm with pre-filled values
    setCurrentDraft(prev => {
      if (prev) {
        // The parent component will handle opening the form
        // We signal this by setting a special state
        return { ...prev, title: prev.title }; // Trigger re-render with draft data
      }
      return prev;
    });
  }, []);

  const handleDiscardDraft = useCallback(() => {
    setCurrentDraft(null);
    setMessages(prev => [...prev, {
      id: crypto.randomUUID(),
      role: 'system',
      content: 'Draft discarded. Try again with a different description?',
      timestamp: Date.now(),
    }]);
  }, []);

  const handleFormCreated = useCallback((beadIds: string[], stitchId?: string) => {
    setCurrentDraft(null);
    onStitchCreated(beadIds, stitchId);
  }, [onStitchCreated]);

  return (
    <div className="chat-to-stitch-pane">
      {/* Header */}
      <div className="csp-header">
        <h2>Chat to Stitch</h2>
        <p className="csp-subtitle">Describe your intent → Get a draft → Edit & submit</p>
      </div>

      {/* Messages */}
      <div className="csp-messages">
        {messages.map(msg => (
          <ChatMessageBubble key={msg.id} message={msg} />
        ))}

        {isProcessing && (
          <div className="csp-message csp-message-system csp-thinking">
            <span className="csp-role">System</span>
            <div className="csp-content">
              <span className="csp-thinking-dots">
                <span className="dot" />
                <span className="dot" />
                <span className="dot" />
              </span>
            </div>
          </div>
        )}

        {/* Draft preview card */}
        {currentDraft && (
          <DraftPreviewCard
            draft={currentDraft}
            onEdit={handleEditDraft}
            onDiscard={handleDiscardDraft}
          />
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="csp-input-area">
        <textarea
          ref={textareaRef}
          className="csp-textarea"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            mutationsDisabled
              ? 'Offline — reconnecting...'
              : 'Describe what you want to do… (Enter to send, Shift+Enter for newline)'
          }
          disabled={isProcessing || mutationsDisabled}
          rows={1}
        />
        <button
          className="csp-send-btn"
          onClick={handleSend}
          disabled={!input.trim() || isProcessing || mutationsDisabled}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <line x1="22" y1="2" x2="11" y2="13" />
            <polygon points="22 2 15 22 11 13 2 9 22 2" />
          </svg>
        </button>
      </div>

      {/* StitchDraftForm overlay for editing */}
      {currentDraft && (
        <StitchDraftForm
          projectName={projectName}
          onClose={() => setCurrentDraft(null)}
          onCreated={handleFormCreated}
          initialDraft={currentDraft}
        />
      )}
    </div>
  );
}
