import { useState, useRef, useEffect } from 'react';

/// Central glossary of UI elements with one-sentence explanations
///
/// Each entry provides:
/// - id: unique identifier matching the data-explain-this attribute
/// - label: short title for the element
/// - explanation: one-sentence "what this is / when to use it"
/// - category: for grouping related items
export const UI_GLOSSARY: Record<
  string,
  { label: string; explanation: string; category: string }
> = {
  // Navigation
  'project-switcher': {
    label: 'Project Switcher',
    explanation: 'Navigate between registered projects or search to find one quickly.',
    category: 'navigation',
  },
  'stitch-list': {
    label: 'Stitch List',
    explanation: 'All conversations and work items in this project, sorted by recent activity.',
    category: 'navigation',
  },
  'pattern-list': {
    label: 'Pattern List',
    explanation: 'Groupings of Stitches organized around goals or initiatives.',
    category: 'navigation',
  },
  'search-palette': {
    label: 'Search Palette',
    explanation: 'Quick search across projects, Stitches, beads, and conversations. Press Cmd/Ctrl+K.',
    category: 'navigation',
  },

  // Stitch operations
  'stitch-create': {
    label: 'Create Stitch',
    explanation: 'Start a new conversation or work item in this project.',
    category: 'stitches',
  },
  'stitch-filter': {
    label: 'Filter Stitches',
    explanation: 'Show only Stitches matching the selected status or kind.',
    category: 'stitches',
  },
  'stitch-draft': {
    label: 'Stitch Draft',
    explanation: 'Preview work before committing. Shows cost estimate and risk assessment.',
    category: 'stitches',
  },
  'stitch-link': {
    label: 'Link Bead',
    explanation: 'Connect a bead to this Stitch for full traceability between tasks and conversations.',
    category: 'stitches',
  },

  // Agent
  'agent-chat': {
    label: 'Agent Chat',
    explanation: 'Conversational AI that can answer questions, draft work, and summarize your projects.',
    category: 'agent',
  },
  'agent-morning-brief': {
    label: 'Morning Brief',
    explanation: 'Daily summary of what closed, what failed, what\'s stuck, and recommended follow-ups.',
    category: 'agent',
  },
  'agent-capacity': {
    label: 'Capacity Widget',
    explanation: 'Track API usage and remaining quota for your Claude account tier.',
    category: 'agent',
  },

  // Dictation
  'dictation-widget': {
    label: 'Dictation Widget',
    explanation: 'Record voice notes with automatic transcription. Click the mic or press the hotkey.',
    category: 'dictation',
  },
  'dictation-hotkey': {
    label: 'Dictation Hotkey',
    explanation: 'Push-to-talk shortcut for quick voice notes. Configure in settings.',
    category: 'dictation',
  },

  // File browser
  'file-browser': {
    label: 'File Browser',
    explanation: 'Explore project source code with syntax highlighting and change tracking.',
    category: 'files',
  },
  'file-search': {
    label: 'File Search',
    explanation: 'Find files by name or search content across the entire project.',
    category: 'files',
  },

  // Reflection Ledger
  'reflection-ledger': {
    label: 'Reflection Ledger',
    explanation: 'Learned rules from repeated patterns. Approved rules inject into every agent session.',
    category: 'learning',
  },
  'reflection-proposal': {
    label: 'Reflection Proposal',
    explanation: 'Agent-detected pattern from your work. Approve to make it a reusable rule.',
    category: 'learning',
  },

  // Settings
  'settings-menu': {
    label: 'Settings Menu',
    explanation: 'Configure preferences, manage projects, and view system status.',
    category: 'settings',
  },
  'theme-toggle': {
    label: 'Theme Toggle',
    explanation: 'Switch between light and dark mode.',
    category: 'settings',
  },

  // Fleet monitoring
  'fleet-status': {
    label: 'Fleet Status',
    explanation: 'Overview of all NEEDLE workers and their current state.',
    category: 'fleet',
  },
  'worker-timeline': {
    label: 'Worker Timeline',
    explanation: 'Chronological view of worker sessions and their bead executions.',
    category: 'fleet',
  },

  // Cost tracking
  'cost-today': {
    label: 'Cost Today',
    explanation: 'Total API spending for this project since midnight.',
    category: 'cost',
  },
  'cost-anomaly': {
    label: 'Cost Anomaly',
    explanation: 'Alert when spending exceeds expected patterns. Click to investigate.',
    category: 'cost',
  },
};

interface ExplainThisProps {
  /**
   * Identifier for the UI element, must match a key in UI_GLOSSARY
   */
  id: string;

  /**
   * Placement of the tooltip relative to the wrapped element
   * @default 'top'
   */
  placement?: 'top' | 'bottom' | 'left' | 'right';

  /**
   * Whether to show the explanation inline instead of as a tooltip
   * @default false
   */
  inline?: boolean;

  /**
   * CSS class name for the wrapped element
   */
  className?: string;

  /**
   * Additional styles to apply
   */
  style?: React.CSSProperties;

  /**
   * Children to wrap with the explain-this behavior
   */
  children: React.ReactNode;
}

/**
 * ExplainThis hover component
 *
 * Wraps any UI element to add a "?" icon that shows an explanation on hover.
 * Explanations are sourced from the central UI_GLOSSARY.
 *
 * @example
 * ```tsx
 * <ExplainThis id="agent-chat">
 *   <button>Chat</button>
 * </ExplainThis>
 * ```
 */
export function ExplainThis({
  id,
  placement = 'top',
  inline = false,
  className,
  style,
  children,
}: ExplainThisProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const triggerRef = useRef<HTMLSpanElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  const entry = UI_GLOSSARY[id];

  // If no glossary entry, just render children (fail silently)
  if (!entry) {
    return <>{children}</>;
  }

  const handleMouseEnter = (e: React.MouseEvent) => {
    if (!triggerRef.current) return;

    const rect = triggerRef.current.getBoundingClientRect();
    const tooltipWidth = 280; // Approximate max width
    const tooltipHeight = 80; // Approximate height

    let top = rect.top;
    let left = rect.left;

    switch (placement) {
      case 'top':
        top = rect.top - tooltipHeight - 8;
        left = rect.left + rect.width / 2 - tooltipWidth / 2;
        break;
      case 'bottom':
        top = rect.bottom + 8;
        left = rect.left + rect.width / 2 - tooltipWidth / 2;
        break;
      case 'left':
        top = rect.top + rect.height / 2 - tooltipHeight / 2;
        left = rect.left - tooltipWidth - 8;
        break;
      case 'right':
        top = rect.top + rect.height / 2 - tooltipHeight / 2;
        left = rect.right + 8;
        break;
    }

    // Keep tooltip within viewport
    const padding = 16;
    top = Math.max(padding, Math.min(top, window.innerHeight - tooltipHeight - padding));
    left = Math.max(padding, Math.min(left, window.innerWidth - tooltipWidth - padding));

    setPosition({ top, left });
    setIsVisible(true);
  };

  const handleMouseLeave = () => {
    setIsVisible(false);
  };

  // Inline mode: show explanation below the element
  if (inline) {
    return (
      <span className={`explain-this-inline ${className || ''}`} style={style}>
        {children}
        <span className="explain-this-inline-text">
          <strong>{entry.label}:</strong> {entry.explanation}
        </span>
      </span>
    );
  }

  // Tooltip mode: show on hover
  return (
    <span
      ref={triggerRef}
      className={`explain-this-trigger ${className || ''}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      style={style}
    >
      {children}
      <span className="explain-this-icon" aria-label={`Explain: ${entry.label}`}>
        ?
      </span>
      {isVisible && (
        <div
          ref={tooltipRef}
          className="explain-this-tooltip"
          style={{
            top: `${position.top}px`,
            left: `${position.left}px`,
          }}
          role="tooltip"
          aria-label={entry.explanation}
        >
          <div className="explain-this-category">{entry.category}</div>
          <div className="explain-this-label">{entry.label}</div>
          <div className="explain-this-explanation">{entry.explanation}</div>
        </div>
      )}
    </span>
  );
}

/**
 * Higher-order component to add explain-this to any component
 */
export function withExplainThis<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  id: string,
  placement?: ExplainThisProps['placement']
) {
  return function ExplainThisWrapped(props: P) {
    return (
      <ExplainThis id={id} placement={placement}>
        <WrappedComponent {...props} />
      </ExplainThis>
    );
  };
}

/**
 * Hook to get explanation text for a given ID
 * Useful for custom implementations or help text
 */
export function useExplanation(id: string): { label: string; explanation: string; category: string } | null {
  return UI_GLOSSARY[id] || null;
}
