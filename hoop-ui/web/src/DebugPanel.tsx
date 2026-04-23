import { useAtomValue, useSetAtom } from 'jotai';
import { useState, useMemo, useRef, useEffect } from 'react';
import { conversationsAtom, beadsAtom, beadEventsAtom, SessionMessage, formatContent, BeadEventFromEvents } from './atoms';

interface Attachment {
  path: string;
  type: 'file' | 'image' | 'output' | 'stderr';
  size?: number;
  content?: string;
  mimeType?: string;
}

interface TimelineStep {
  id: string;
  index: number;
  timestamp: string;
  type: 'system' | 'user' | 'assistant' | 'tool_call' | 'tool_result' | 'content_block' | 'error' | 'bead_claim' | 'bead_close' | 'bead_release' | 'bead_update' | 'stderr' | 'state_transition' | 'gap' | 'attachment';
  role: string;
  content?: string | { [key: string]: any } | null;
  usage?: { input_tokens: number; output_tokens: number; cache_read_tokens: number; cache_write_tokens: number };
  toolName?: string;
  toolInput?: { [key: string]: any };
  toolResult?: { [key: string]: any };
  toolError?: string;
  contentBlockType?: 'text' | 'thinking' | 'image';
  stopReason?: string;
  attachments?: Attachment[];
  model?: string;
  // Bead event metadata
  beadId?: string;
  worker?: string;
  eventType?: string;
  lineNumber?: number;
  rawEvent?: string;
  // State transition metadata
  fromState?: string;
  toState?: string;
  // Gap before this step (temporal gap detection)
  gapBefore?: { index: number; gapMs: number; message: string };
  // Reconstruction metadata
  source: 'session' | 'events' | 'inferred';
}

interface ToolUseBlock {
  name: string;
  id?: string;
  input?: { [key: string]: any };
}

interface ToolResultBlock {
  tool_use_id?: string;
  content?: { [key: string]: any } | string;
  error?: string;
  is_error?: boolean;
}

interface ContentBlock {
  type: 'text' | 'tool_use' | 'tool_result' | 'thinking' | 'image';
  text?: string;
  tool_use?: ToolUseBlock;
  tool_result?: ToolResultBlock;
  thinking?: string;
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

// Parse structured content from Claude Code sessions
function parseContentBlocks(content: any): ContentBlock[] {
  if (!content) return [];

  // String content - wrap as text block
  if (typeof content === 'string') {
    return [{ type: 'text', text: content }];
  }

  // Array of content blocks (Claude's structured format)
  if (Array.isArray(content)) {
    return content.map(block => {
      if (typeof block === 'string') {
        return { type: 'text' as const, text: block };
      }
      if (typeof block === 'object' && block !== null) {
        return block as ContentBlock;
      }
      return { type: 'text' as const, text: String(block) };
    });
  }

  // Object content - might be a single block or wrapped
  if (typeof content === 'object') {
    // Check if it's already a content block
    if (content.type === 'text' || content.type === 'tool_use' || content.type === 'tool_result' || content.type === 'thinking') {
      return [content as ContentBlock];
    }
    // Check if it has tool_use directly
    if (content.tool_use) {
      return [{ type: 'tool_use', tool_use: content.tool_use }];
    }
    // Check for tool_result
    if (content.tool_result) {
      return [{ type: 'tool_result', tool_result: content.tool_result }];
    }
    // Generic object - stringify
    return [{ type: 'text', text: JSON.stringify(content, null, 2) }];
  }

  return [{ type: 'text', text: String(content) }];
}

// Check if content contains stderr output
function extractStderr(content: any): string | null {
  if (!content) return null;

  // Look for stderr patterns in tool results
  if (typeof content === 'object' && content.tool_result) {
    const result = content.tool_result.content || content.tool_result;
    if (typeof result === 'string' && result.includes('stderr')) {
      // Try to extract stderr from common patterns
      const stderrMatch = result.match(/stderr["':\s]*([^\n]*)/i);
      if (stderrMatch) return stderrMatch[1];
      // Or return the whole result if it looks like stderr
      if (result.includes('stderr:') || result.includes('STDERR:')) {
        return result;
      }
    }
  }

  if (typeof content === 'string') {
    // Check for stderr markers
    if (content.includes('stderr:') || content.includes('STDERR:') || content.includes('[stderr]')) {
      return content;
    }
  }

  return null;
}

// Check for state transitions in content
function extractStateTransition(content: any): { from: string; to: string } | null {
  if (!content) return null;

  const contentStr = typeof content === 'string' ? content : JSON.stringify(content);

  // Look for state transition patterns
  const transitionPatterns = [
    /state:\s*(\w+)\s*->\s*(\w+)/i,
    /transition[ed]?\s+from\s+(\w+)\s+to\s+(\w+)/i,
    /(\w+)\s+->\s+(\w+)\s+\(state\)/i,
  ];

  for (const pattern of transitionPatterns) {
    const match = contentStr.match(pattern);
    if (match) {
      return { from: match[1], to: match[2] };
    }
  }

  return null;
}

// Correlate bead events with session messages based on timestamp proximity
// Returns a map of message index to bead events that occurred around that message
function correlateEventsWithMessages(
  messages: SessionMessage[],
  beadEvents: BeadEventFromEvents[]
): Map<number, BeadEventFromEvents[]> {
  const correlation = new Map<number, BeadEventFromEvents[]>();

  for (const event of beadEvents) {
    const eventTime = new Date(event.timestamp).getTime();
    let closestMessageIndex = -1;
    let minDiff = Infinity;

    // Find the closest message by timestamp
    for (let i = 0; i < messages.length; i++) {
      const msg = messages[i];
      const msgTime = msg.timestamp ? new Date(msg.timestamp).getTime() : 0;
      const diff = Math.abs(eventTime - msgTime);

      if (diff < minDiff) {
        minDiff = diff;
        closestMessageIndex = i;
      }
    }

    // If we found a reasonably close match (within 5 seconds), correlate it
    if (closestMessageIndex >= 0 && minDiff < 5000) {
      if (!correlation.has(closestMessageIndex)) {
        correlation.set(closestMessageIndex, []);
      }
      correlation.get(closestMessageIndex)!.push(event);
    }
  }

  return correlation;
}

// Reconstruct timeline from session messages with full tool call parsing and bead event correlation
// Combines messages and bead events, maintaining original order with event correlation
function reconstructTimeline(
  messages: SessionMessage[],
  beadEvents: BeadEventFromEvents[] = []
): TimelineStep[] {
  const steps: TimelineStep[] = [];
  let stepIndex = 0;

  // Track added bead events to avoid duplicates
  const addedBeadEvents = new Set<string>();

  // Pre-correlate events with messages for proper ordering
  const eventCorrelation = correlateEventsWithMessages(messages, beadEvents);

  // Add session messages with correlated events interspersed
  for (let msgIndex = 0; msgIndex < messages.length; msgIndex++) {
    const msg = messages[msgIndex];
    const msgTime = msg.timestamp ? new Date(msg.timestamp).getTime() : Date.now();

    // Check if there are correlated events for this message
    const correlatedEvents = eventCorrelation.get(msgIndex) || [];

    // Sort correlated events by timestamp to insert them in correct order
    correlatedEvents.sort((a, b) => {
      const aTime = new Date(a.timestamp).getTime();
      const bTime = new Date(b.timestamp).getTime();
      return aTime - bTime;
    });

    // Insert events that occurred before this message
    for (const event of correlatedEvents) {
      const eventTime = new Date(event.timestamp).getTime();
      if (eventTime < msgTime) {
        const eventKey = `${event.line_number}-${event.event_type}`;
        if (addedBeadEvents.has(eventKey)) {
          continue;
        }

        let stepType: TimelineStep['type'] = 'bead_claim';
        let description = '';
        let fromState: string | undefined;
        let toState: string | undefined;

        switch (event.event_type) {
          case 'claim':
            stepType = 'bead_claim';
            description = `Bead ${event.bead_id} claimed by ${event.worker}`;
            fromState = 'unclaimed';
            toState = 'executing';
            break;
          case 'close':
            stepType = 'bead_close';
            description = `Bead ${event.bead_id} closed by ${event.worker}`;
            fromState = 'executing';
            toState = 'closed';
            break;
          case 'release':
            stepType = 'bead_release';
            description = `Bead ${event.bead_id} released by ${event.worker}`;
            fromState = 'executing';
            toState = 'idle';
            break;
          case 'update':
            stepType = 'bead_update';
            description = `Bead ${event.bead_id} updated by ${event.worker}`;
            break;
        }

        steps.push({
          id: `bead-event-${event.timestamp}-${event.line_number || stepIndex}`,
          index: stepIndex++,
          timestamp: event.timestamp,
          type: stepType,
          role: 'bead',
          beadId: event.bead_id,
          worker: event.worker,
          eventType: event.event_type,
          content: description,
          lineNumber: event.line_number,
          rawEvent: event.raw,
          fromState,
          toState,
          source: 'events',
        });

        addedBeadEvents.add(eventKey);
      }
    }

    const baseTimestamp = msg.timestamp || new Date().toISOString();

    // Check for stderr in message content
    const stderr = extractStderr(msg.content);
    if (stderr) {
      steps.push({
        id: `stderr-${stepIndex++}-${baseTimestamp}`,
        index: stepIndex,
        timestamp: baseTimestamp,
        type: 'stderr',
        role: 'system',
        content: stderr,
        source: 'session',
      });
    }

    // Check for state transitions
    const stateTransition = extractStateTransition(msg.content);
    if (stateTransition) {
      steps.push({
        id: `state-${stepIndex++}-${baseTimestamp}`,
        index: stepIndex,
        timestamp: baseTimestamp,
        type: 'state_transition',
        role: 'system',
        content: `State transition: ${stateTransition.from} → ${stateTransition.to}`,
        fromState: stateTransition.from,
        toState: stateTransition.to,
        source: 'inferred',
      });
    }

    // Parse content blocks
    const contentBlocks = parseContentBlocks(msg.content);

    // System message - single step
    if (msg.role === 'system') {
      steps.push({
        id: `step-${stepIndex++}`,
        index: stepIndex,
        timestamp: baseTimestamp,
        type: 'system',
        role: msg.role,
        content: msg.content,
        usage: msg.usage,
        source: 'session',
      });
      continue;
    }

    // User message - single step
    if (msg.role === 'user') {
      steps.push({
        id: `step-${stepIndex++}`,
        index: stepIndex,
        timestamp: baseTimestamp,
        type: 'user',
        role: msg.role,
        content: msg.content,
        attachments: extractAttachments(msg.content),
        source: 'session',
      });
      continue;
    }

    // Assistant message - may contain multiple content blocks
    if (msg.role === 'assistant') {
      if (contentBlocks.length === 0) {
        // Empty assistant message
        steps.push({
          id: `step-${stepIndex++}`,
          index: stepIndex,
          timestamp: baseTimestamp,
          type: 'assistant',
          role: msg.role,
          content: null,
          usage: msg.usage,
          source: 'session',
        });
      } else if (contentBlocks.length === 1 && contentBlocks[0].type === 'text') {
        // Simple text response
        steps.push({
          id: `step-${stepIndex++}`,
          index: stepIndex,
          timestamp: baseTimestamp,
          type: 'assistant',
          role: msg.role,
          content: contentBlocks[0].text,
          usage: msg.usage,
          source: 'session',
        });
      } else {
        // Multiple content blocks - expand into separate steps
        for (const block of contentBlocks) {
          if (block.type === 'text') {
            steps.push({
              id: `step-${stepIndex++}`,
              index: stepIndex,
              timestamp: baseTimestamp,
              type: 'assistant',
              role: msg.role,
              content: block.text,
              usage: msg.usage,
              contentBlockType: 'text',
              source: 'session',
            });
          } else if (block.type === 'thinking') {
            steps.push({
              id: `step-${stepIndex++}`,
              index: stepIndex,
              timestamp: baseTimestamp,
              type: 'assistant',
              role: msg.role,
              content: block.thinking,
              contentBlockType: 'thinking',
              source: 'session',
            });
          } else if (block.type === 'tool_use' && block.tool_use) {
            steps.push({
              id: `step-${stepIndex++}`,
              index: stepIndex,
              timestamp: baseTimestamp,
              type: 'tool_call',
              role: msg.role,
              toolName: block.tool_use.name,
              toolInput: block.tool_use.input,
              usage: msg.usage,
              source: 'session',
            });
          } else if (block.type === 'tool_result' && block.tool_result) {
            const attachments = extractAttachments(block.tool_result.content, block.tool_use?.name);
            steps.push({
              id: `step-${stepIndex++}`,
              index: stepIndex,
              timestamp: baseTimestamp,
              type: 'tool_result',
              role: msg.role,
              toolResult: typeof block.tool_result.content === 'object'
                ? block.tool_result.content
                : { result: block.tool_result.content },
              toolError: block.tool_result.error || (block.tool_result.is_error ? 'Tool execution failed' : undefined),
              attachments,
              source: 'session',
            });
          } else if (block.type === 'image') {
            steps.push({
              id: `step-${stepIndex++}`,
              index: stepIndex,
              timestamp: baseTimestamp,
              type: 'content_block',
              role: msg.role,
              contentBlockType: 'image',
              content: block,
              source: 'session',
            });
          }
        }
      }
      continue;
    }

    // Tool role (tool result message)
    if (msg.role === 'tool') {
      const attachments = extractAttachments(msg.content, msg.tool_name);
      steps.push({
        id: `step-${stepIndex++}`,
        index: stepIndex,
        timestamp: baseTimestamp,
        type: 'tool_result',
        role: msg.role,
        toolResult: msg.content as { [key: string]: any },
        attachments,
        source: 'session',
      });
      continue;
    }

    // Unknown role - treat as assistant
    steps.push({
      id: `step-${stepIndex++}`,
      index: stepIndex,
      timestamp: baseTimestamp,
      type: 'assistant',
      role: msg.role,
      content: msg.content,
      usage: msg.usage,
      source: 'session',
    });
  }

  // Add orphaned events (events that didn't correlate with any message)
  // This ensures full reconstruction with no gaps
  const orphanedEvents = beadEvents.filter(event => {
    const eventKey = `${event.line_number}-${event.event_type}`;
    return !addedBeadEvents.has(eventKey);
  });

  // Sort orphaned events by timestamp
  orphanedEvents.sort((a, b) => {
    const aTime = new Date(a.timestamp).getTime();
    const bTime = new Date(b.timestamp).getTime();
    return aTime - bTime;
  });

  // Insert orphaned events at appropriate positions based on timestamp
  for (const event of orphanedEvents) {
    const eventTime = new Date(event.timestamp).getTime();
    let insertIndex = steps.length; // Default to end

    // Find the right position based on timestamp
    for (let i = 0; i < steps.length; i++) {
      const stepTime = new Date(steps[i].timestamp).getTime();
      if (eventTime < stepTime) {
        insertIndex = i;
        break;
      }
    }

    let stepType: TimelineStep['type'] = 'bead_claim';
    let description = '';
    let fromState: string | undefined;
    let toState: string | undefined;

    switch (event.event_type) {
      case 'claim':
        stepType = 'bead_claim';
        description = `Bead ${event.bead_id} claimed by ${event.worker} (orphaned)`;
        fromState = 'unclaimed';
        toState = 'executing';
        break;
      case 'close':
        stepType = 'bead_close';
        description = `Bead ${event.bead_id} closed by ${event.worker} (orphaned)`;
        fromState = 'executing';
        toState = 'closed';
        break;
      case 'release':
        stepType = 'bead_release';
        description = `Bead ${event.bead_id} released by ${event.worker} (orphaned)`;
        fromState = 'executing';
        toState = 'idle';
        break;
      case 'update':
        stepType = 'bead_update';
        description = `Bead ${event.bead_id} updated by ${event.worker} (orphaned)`;
        break;
    }

    steps.splice(insertIndex, 0, {
      id: `bead-event-${event.timestamp}-${event.line_number || stepIndex}`,
      index: insertIndex,
      timestamp: event.timestamp,
      type: stepType,
      role: 'bead',
      beadId: event.bead_id,
      worker: event.worker,
      eventType: event.event_type,
      content: description,
      lineNumber: event.lineNumber,
      rawEvent: event.raw,
      fromState,
      toState,
      source: 'events',
    });

    stepIndex++;
  }

  // Detect gaps in the timeline and create explicit gap steps
  // A gap is defined as a time difference > 10 seconds between consecutive steps
  const gapSteps: TimelineStep[] = [];
  const stepsWithGaps: TimelineStep[] = [];

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i];

    // Check if there's a gap before this step
    if (i > 0) {
      const prevTime = new Date(steps[i - 1].timestamp).getTime();
      const currTime = new Date(step.timestamp).getTime();
      const gapMs = currTime - prevTime;

      if (gapMs > 10000) { // More than 10 seconds
        const gapSeconds = Math.floor(gapMs / 1000);
        const gapMinutes = Math.floor(gapSeconds / 60);
        const gapMinutesText = gapMinutes > 0 ? `${gapMinutes}m ` : '';
        const gapDescription = gapSeconds > 60
          ? `${gapMinutesText}${gapSeconds % 60}s gap detected`
          : `${gapSeconds}s gap detected`;

        // Create a gap step
        gapSteps.push({
          id: `gap-${i}-${steps[i - 1].timestamp}`,
          index: -1, // Will be re-indexed
          timestamp: new Date(prevTime + gapMs / 2).toISOString(), // Place in middle of gap
          type: 'gap',
          role: 'system',
          content: gapDescription,
          gapBefore: { index: i, gapMs, message: gapDescription },
          source: 'inferred',
        });
      }
    }

    stepsWithGaps.push(step);
  }

  // Merge steps with gap steps, maintaining chronological order
  const allSteps: TimelineStep[] = [];
  let stepIdx = 0;
  let gapIdx = 0;

  while (stepIdx < stepsWithGaps.length || gapIdx < gapSteps.length) {
    if (stepIdx >= stepsWithGaps.length) {
      allSteps.push(gapSteps[gapIdx++]);
    } else if (gapIdx >= gapSteps.length) {
      allSteps.push(stepsWithGaps[stepIdx++]);
    } else {
      const stepTime = new Date(stepsWithGaps[stepIdx].timestamp).getTime();
      const gapTime = new Date(gapSteps[gapIdx].timestamp).getTime();

      if (gapTime < stepTime) {
        allSteps.push(gapSteps[gapIdx++]);
      } else {
        allSteps.push(stepsWithGaps[stepIdx++]);
      }
    }
  }

  // Re-index for final display
  allSteps.forEach((step, i) => {
    step.index = i;
  });

  return allSteps;
}

// Extract attachments from tool results with enhanced detection
function extractAttachments(toolResult: any, toolName?: string): Attachment[] | undefined {
  if (!toolResult) return undefined;

  const attachments: Attachment[] = [];

  // Helper to determine file type from path
  const getFileType = (path: string): Attachment['type'] => {
    const ext = path.split('.').pop()?.toLowerCase();
    if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp'].includes(ext || '')) return 'image';
    if (path.includes('stderr') || path.includes('.log')) return 'stderr';
    if (path.includes('stdout') || path.includes('output')) return 'output';
    return 'file';
  };

  // Helper to get mime type from extension
  const getMimeType = (path: string): string | undefined => {
    const ext = path.split('.').pop()?.toLowerCase();
    const mimeMap: Record<string, string> = {
      'png': 'image/png',
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'gif': 'image/gif',
      'svg': 'image/svg+xml',
      'webp': 'image/webp',
      'pdf': 'application/pdf',
      'json': 'application/json',
      'txt': 'text/plain',
      'log': 'text/plain',
      'md': 'text/markdown',
      'html': 'text/html',
      'css': 'text/css',
      'js': 'text/javascript',
      'ts': 'text/typescript',
      'rs': 'text/rust',
      'py': 'text/python',
    };
    return mimeMap[ext || ''];
  };

  // Check for Write tool results which create files
  if (toolName === 'Write' && toolResult.file_path) {
    attachments.push({
      path: toolResult.file_path,
      type: getFileType(toolResult.file_path),
      mimeType: getMimeType(toolResult.file_path),
    });
  }

  // Check for Edit tool results which modify files
  if (toolName === 'Edit' && toolResult.file_path) {
    attachments.push({
      path: toolResult.file_path,
      type: getFileType(toolResult.file_path),
      mimeType: getMimeType(toolResult.file_path),
    });
  }

  // Check for Bash tool results with file outputs
  if (toolName === 'Bash' && typeof toolResult === 'object') {
    // Look for file paths in stdout/stderr
    for (const key of ['stdout', 'stderr', 'output']) {
      const output = toolResult[key];
      if (typeof output === 'string') {
        // Match file paths in common formats
        const pathPatterns = [
          /Written to [^\s"']+?/g,
          /Created [^\s"']+?/g,
          /Saved to [^\s"']+?/g,
          /File: ([^\s"']+?)/g,
          /-> ([^\s"']+?)/g,
        ];
        for (const pattern of pathPatterns) {
          const matches = output.match(pattern);
          if (matches) {
            for (const match of matches) {
              const path = match.replace(/^(Written to |Created |Saved to |File: |-> )/, '');
              if (path.startsWith('/') || path.includes('.')) {
                attachments.push({
                  path,
                  type: getFileType(path),
                  mimeType: getMimeType(path),
                });
              }
            }
          }
        }
        // Also look for bare file paths
        const filePaths = output.match(/[\/~][^\s"']+?\.[a-zA-Z0-9]+/g);
        if (filePaths) {
          for (const path of filePaths) {
            if (!attachments.find(a => a.path === path)) {
              attachments.push({
                path,
                type: getFileType(path),
                mimeType: getMimeType(path),
              });
            }
          }
        }
      }
    }
  }

  // Check for explicit attachments array
  if (Array.isArray(toolResult.attachments)) {
    for (const attachment of toolResult.attachments) {
      if (typeof attachment === 'string') {
        attachments.push({
          path: attachment,
          type: getFileType(attachment),
          mimeType: getMimeType(attachment),
        });
      } else if (typeof attachment === 'object' && attachment.path) {
        attachments.push({
          path: attachment.path,
          type: attachment.type || getFileType(attachment.path),
          size: attachment.size,
          content: attachment.content,
          mimeType: attachment.mimeType || getMimeType(attachment.path),
        });
      }
    }
  }

  // Check in nested tool_result content
  if (toolResult.tool_result && toolResult.tool_result.content) {
    const nested = extractAttachments(toolResult.tool_result.content, toolName);
    if (nested) attachments.push(...nested);
  }

  // Check if content is a string with file paths
  if (typeof toolResult === 'string') {
    const filePaths = toolResult.match(/[\/~][^\s"']+?\.[a-zA-Z0-9]+/g);
    if (filePaths) {
      for (const path of filePaths) {
        if (!attachments.find(a => a.path === path)) {
          attachments.push({
            path,
            type: getFileType(path),
            mimeType: getMimeType(path),
          });
        }
      }
    }
  }

  return attachments.length > 0 ? attachments : undefined;
}

interface DebugPanelProps {
  projectName: string;
  projectPath: string;
}

export default function DebugPanel({ projectPath }: DebugPanelProps) {
  const conversations = useAtomValue(conversationsAtom);
  const beads = useAtomValue(beadsAtom);
  const setBeadEvents = useSetAtom(beadEventsAtom);

  const [selectedBeadId, setSelectedBeadId] = useState<string | null>(null);
  const [selectedConversationId, setSelectedConversationId] = useState<string | null>(null);
  const [selectedStepIndex, setSelectedStepIndex] = useState<number | null>(null);
  const [autoScroll, setAutoScroll] = useState(false);
  const [loadingBeadEvents, setLoadingBeadEvents] = useState<Set<string>>(new Set());
  const timelineRef = useRef<HTMLDivElement>(null);

  // Fetch bead events for a specific bead from the API
  const fetchBeadEvents = async (beadId: string) => {
    if (loadingBeadEvents.has(beadId)) return;

    setLoadingBeadEvents(prev => new Set(prev).add(beadId));

    try {
      const response = await fetch(`/api/beads/${beadId}/events`);
      if (response.ok) {
        const events = await response.json();
        setBeadEvents(prev => {
          const next = new Map(prev);
          next.set(beadId, events);
          return next;
        });
      }
    } catch (e) {
      console.error(`Failed to fetch events for bead ${beadId}:`, e);
    } finally {
      setLoadingBeadEvents(prev => {
        const next = new Set(prev);
        next.delete(beadId);
        return next;
      });
    }
  };

  // Filter beads with worker sessions for this project
  const beadsWithSessions = useMemo(() => {
    const workerConversations = conversations.filter(c =>
      c.kind === 'worker' &&
      c.worker_metadata &&
      c.cwd.startsWith(projectPath)
    );

    // Group conversations by bead
    const beadGroups = new Map<string, typeof workerConversations>();
    for (const conv of workerConversations) {
      const beadId = conv.worker_metadata?.bead;
      if (beadId) {
        if (!beadGroups.has(beadId)) {
          beadGroups.set(beadId, []);
        }
        beadGroups.get(beadId)!.push(conv);
      }
    }

    // Create bead items with their sessions
    return Array.from(beadGroups.entries()).map(([beadId, sessions]) => {
      const bead = beads.find(b => b.id === beadId);
      const latestSession = sessions.sort((a, b) =>
        new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
      )[0];

      return {
        beadId,
        bead,
        sessions,
        latestSession,
        worker: latestSession.worker_metadata?.worker,
        totalMessages: sessions.reduce((sum, s) => sum + s.messages.length, 0),
        totalTokens: sessions.reduce((sum, s) => sum + s.total_tokens, 0),
        isActive: sessions.some(s => !s.complete),
      };
    }).sort((a, b) => {
      // Sort by active status first, then by latest session time
      if (a.isActive && !b.isActive) return -1;
      if (!a.isActive && b.isActive) return 1;
      return new Date(b.latestSession.updated_at).getTime() - new Date(a.latestSession.updated_at).getTime();
    });
  }, [conversations, beads, projectPath]);

  // Selected bead with its sessions
  const selectedBeadData = useMemo(() => {
    return beadsWithSessions.find(b => b.beadId === selectedBeadId);
  }, [beadsWithSessions, selectedBeadId]);

  // Default to the latest conversation for the selected bead
  const selectedConversation = useMemo(() => {
    if (selectedConversationId) {
      return selectedBeadData?.sessions.find(s => s.id === selectedConversationId);
    }
    return selectedBeadData?.latestSession;
  }, [selectedBeadData, selectedConversationId]);

  // Get bead events for selected bead
  const beadEventsForSelected = useAtomValue(beadEventsAtom);
  const currentBeadEvents = useMemo(() => {
    return selectedBeadId ? (beadEventsForSelected.get(selectedBeadId) || []) : [];
  }, [beadEventsForSelected, selectedBeadId]);

  // Fetch events when bead is selected
  useEffect(() => {
    if (selectedBeadId && !beadEventsForSelected.has(selectedBeadId)) {
      fetchBeadEvents(selectedBeadId);
    }
  }, [selectedBeadId, beadEventsForSelected]);

  // Reconstruct timeline for selected conversation with real bead events
  const timeline = useMemo(() => {
    if (!selectedConversation) return [];

    return reconstructTimeline(selectedConversation.messages, currentBeadEvents);
  }, [selectedConversation, currentBeadEvents]);

  // Calculate total cost for this session
  const totalCost = useMemo(() => {
    if (!selectedConversation) return 0;
    const totalTokens = selectedConversation.messages.reduce((sum, msg) => {
      if (!msg.usage) return sum;
      return sum + (msg.usage.input_tokens || 0) + (msg.usage.output_tokens || 0);
    }, 0);
    // Rough estimate: $3 per million tokens for input/output
    return (totalTokens / 1_000_000) * 3;
  }, [selectedConversation]);

  // Select bead handler
  const handleBeadSelect = (beadId: string) => {
    setSelectedBeadId(beadId);
    setSelectedConversationId(null);
    setSelectedStepIndex(null);
  };

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (selectedStepIndex === null || timeline.length === 0) return;
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedStepIndex(Math.max(0, selectedStepIndex - 1));
        setAutoScroll(true);
      } else if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedStepIndex(Math.min(timeline.length - 1, selectedStepIndex + 1));
        setAutoScroll(true);
      } else if (e.key === 'Home') {
        e.preventDefault();
        setSelectedStepIndex(0);
        setAutoScroll(true);
      } else if (e.key === 'End') {
        e.preventDefault();
        setSelectedStepIndex(timeline.length - 1);
        setAutoScroll(true);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedStepIndex, timeline.length]);

  // Scroll selected step into view when navigating
  useEffect(() => {
    if (!autoScroll || selectedStepIndex === null || !timelineRef.current) return;
    const stepEl = timelineRef.current.querySelector(`[data-step-index="${selectedStepIndex}"]`);
    if (stepEl) stepEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
    setAutoScroll(false);
  }, [autoScroll, selectedStepIndex]);

  // Get step type icon
  const getStepIcon = (type: TimelineStep['type']): string => {
    switch (type) {
      case 'system': return '⚙️';
      case 'user': return '👤';
      case 'assistant': return '🤖';
      case 'tool_call': return '🔧';
      case 'tool_result': return '📤';
      case 'content_block': return '📄';
      case 'error': return '❌';
      case 'bead_claim': return '🎯';
      case 'bead_close': return '✅';
      case 'bead_release': return '⏎';
      case 'bead_update': return '🔄';
      case 'stderr': return '⚠️';
      case 'state_transition': return '🔄';
      case 'gap': return '⏱';
      default: return '•';
    }
  };

  // Get step type color
  const getStepColor = (type: TimelineStep['type']): string => {
    switch (type) {
      case 'system': return '#666';
      case 'user': return '#1976d2';
      case 'assistant': return '#333';
      case 'tool_call': return '#f9ab00';
      case 'tool_result': return '#34a853';
      case 'content_block': return '#8430ce';
      case 'error': return '#ea4335';
      case 'bead_claim': return '#4285f4';
      case 'bead_close': return '#34a853';
      case 'bead_release': return '#f9ab00';
      case 'bead_update': return '#8430ce';
      case 'stderr': return '#ea4335';
      case 'state_transition': return '#8430ce';
      case 'gap': return '#f59e0b';
      default: return '#999';
    }
  };

  return (
    <div className="debug-panel">
      <div className="debug-sidebar">
        <div className="debug-sidebar-header">
          <h3>Beads</h3>
          <span className="session-count">{beadsWithSessions.length}</span>
        </div>
        <div className="debug-session-list">
          {beadsWithSessions.length === 0 ? (
            <div className="debug-empty">
              <p>No beads with worker sessions found</p>
              <p className="debug-empty-hint">Beads executed by NEEDLE workers will appear here</p>
            </div>
          ) : (
            beadsWithSessions.map(beadData => (
              <button
                key={beadData.beadId}
                className={`debug-bead-item ${selectedBeadId === beadData.beadId ? 'selected' : ''} ${beadData.isActive ? 'active' : ''}`}
                onClick={() => handleBeadSelect(beadData.beadId)}
              >
                <div className="debug-bead-header">
                  <span className="debug-bead-id">{beadData.beadId}</span>
                  {beadData.isActive && (
                    <span className="debug-bead-status-indicator" title="Active">●</span>
                  )}
                  <span className="debug-bead-time">{formatTimestamp(beadData.latestSession.updated_at)}</span>
                </div>
                {beadData.bead && (
                  <div className="debug-bead-title">{beadData.bead.title}</div>
                )}
                <div className="debug-bead-worker">Worker: {beadData.worker}</div>
                <div className="debug-bead-meta">
                  <span>{beadData.sessions.length} session{beadData.sessions.length > 1 ? 's' : ''}</span>
                  <span>{beadData.totalMessages} messages</span>
                  <span>{formatTokens(beadData.totalTokens)} tokens</span>
                </div>
                {beadData.bead && (
                  <div className="debug-bead-status">
                    <span className={`status-badge ${beadData.bead.status}`}>{beadData.bead.status}</span>
                    <span className={`type-badge ${beadData.bead.issue_type}`}>{beadData.bead.issue_type}</span>
                  </div>
                )}
              </button>
            ))
          )}
        </div>
      </div>

      <div className="debug-main">
        {selectedBeadData ? (
          <>
            <div className="debug-header">
              <div className="debug-header-info">
                <h2>Per-Bead Debug View</h2>
                <div className="debug-header-badges">
                  <span className="badge badge-bead">
                    Bead: {selectedBeadData.beadId}
                  </span>
                  <span className="badge badge-worker">
                    Worker: {selectedBeadData.worker}
                  </span>
                  {selectedBeadData.bead && (
                    <>
                      <span className={`status-badge ${selectedBeadData.bead.status}`}>{selectedBeadData.bead.status}</span>
                      <span className={`type-badge ${selectedBeadData.bead.issue_type}`}>{selectedBeadData.bead.issue_type}</span>
                    </>
                  )}
                </div>
                {selectedBeadData.bead && (
                  <div className="debug-header-title">{selectedBeadData.bead.title}</div>
                )}
              </div>
              <div className="debug-header-stats">
                <div className="debug-stat">
                  <span className="debug-stat-value">{selectedBeadData.sessions.length}</span>
                  <span className="debug-stat-label">sessions</span>
                </div>
                <div className="debug-stat">
                  <span className="debug-stat-value">{formatTokens(selectedBeadData.totalTokens)}</span>
                  <span className="debug-stat-label">tokens</span>
                </div>
                <div className="debug-stat">
                  <span className="debug-stat-value">${((selectedBeadData.totalTokens / 1_000_000) * 3).toFixed(2)}</span>
                  <span className="debug-stat-label">est. cost</span>
                </div>
                {selectedConversation && (
                  <div className="debug-stat">
                    <span className="debug-stat-value">{timeline.length}</span>
                    <span className="debug-stat-label">steps</span>
                  </div>
                )}
              </div>
            </div>

            {/* Session selector for beads with multiple sessions */}
            {selectedBeadData.sessions.length > 1 && (
              <div className="debug-session-selector">
                <span className="session-selector-label">Session:</span>
                <div className="session-selector-buttons">
                  {selectedBeadData.sessions.map((session, idx) => (
                    <button
                      key={session.id}
                      className={`session-selector-button ${selectedConversationId === session.id || (!selectedConversationId && idx === 0) ? 'selected' : ''}`}
                      onClick={() => {
                        setSelectedConversationId(session.id);
                        setSelectedStepIndex(null);
                      }}
                      title={`Session ${idx + 1}: ${formatTimestamp(session.created_at)}`}
                    >
                      {formatTimestamp(session.created_at)}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {selectedConversation && (
            <>
            <div className="debug-header">
              <div className="debug-header-info">
                <h2>Execution Timeline</h2>
                <div className="debug-header-badges">
                  <span className="badge badge-worker">
                    Worker: {selectedConversation.worker_metadata?.worker}
                  </span>
                  <span className="badge badge-session">
                    {selectedConversation.messages.length} messages
                  </span>
                  <span className="badge badge-events">
                    {currentBeadEvents.length} events from events.jsonl
                  </span>
                </div>
              </div>
              <div className="debug-header-stats">
                <div className="debug-stat">
                  <span className="debug-stat-value">{formatTokens(selectedConversation.total_tokens)}</span>
                  <span className="debug-stat-label">tokens</span>
                </div>
                <div className="debug-stat">
                  <span className="debug-stat-value">${totalCost.toFixed(2)}</span>
                  <span className="debug-stat-label">est. cost</span>
                </div>
                <div className="debug-stat">
                  <span className="debug-stat-value">{timeline.length}</span>
                  <span className="debug-stat-label">steps</span>
                </div>
              </div>
            </div>

            {/* Timeline scrubber */}
            <div className="debug-timeline-container">
              <div className="debug-timeline-header">
                <span className="debug-timeline-title">Timeline</span>
                {selectedStepIndex !== null && (
                  <span className="debug-timeline-position">
                    Step {selectedStepIndex + 1} of {timeline.length}
                  </span>
                )}
              </div>
              <div className="debug-timeline-track" ref={timelineRef}>
                {timeline.map((step, index) => (
                  <button
                    key={step.id}
                    data-step-index={index}
                    className={`timeline-step ${step.type} ${selectedStepIndex === index ? 'selected' : ''}`}
                    onClick={() => setSelectedStepIndex(index)}
                    title={`${step.type} - ${formatTimestamp(step.timestamp)}: ${step.toolName || step.contentBlockType || 'message'}`}
                    style={{ '--step-color': getStepColor(step.type) } as React.CSSProperties}
                  >
                    <span className="timeline-step-icon">{getStepIcon(step.type)}</span>
                    <span className="timeline-step-dot" />
                  </button>
                ))}
              </div>
              <div className="debug-timeline-controls">
                <button
                  className="timeline-control-btn"
                  onClick={() => { setSelectedStepIndex(0); setAutoScroll(true); }}
                  disabled={selectedStepIndex === 0}
                  title="First step (Home)"
                >
                  ⏮
                </button>
                <button
                  className="timeline-control-btn"
                  onClick={() => { setSelectedStepIndex(Math.max(0, (selectedStepIndex ?? 0) - 1)); setAutoScroll(true); }}
                  disabled={selectedStepIndex === 0}
                  title="Previous step (←)"
                >
                  ◀
                </button>
                <button
                  className="timeline-control-btn"
                  onClick={() => { setSelectedStepIndex(Math.min(timeline.length - 1, (selectedStepIndex ?? 0) + 1)); setAutoScroll(true); }}
                  disabled={selectedStepIndex === timeline.length - 1}
                  title="Next step (→)"
                >
                  ▶
                </button>
                <button
                  className="timeline-control-btn"
                  onClick={() => { setSelectedStepIndex(timeline.length - 1); setAutoScroll(true); }}
                  disabled={selectedStepIndex === timeline.length - 1}
                  title="Last step (End)"
                >
                  ⏭
                </button>
                <button
                  className="timeline-control-btn"
                  onClick={() => setSelectedStepIndex(null)}
                  title="Clear selection"
                >
                  ✕
                </button>
              </div>
            </div>

            {/* Step detail view */}
            <div className="debug-step-view">
              {selectedStepIndex !== null ? (
                <StepDetail step={timeline[selectedStepIndex]} stepNumber={selectedStepIndex + 1} totalSteps={timeline.length} />
              ) : (
                <div className="debug-step-empty">
                  <p>Select a step from the timeline to view details</p>
                  <div className="debug-step-hint">
                    <span>Timeline shows {timeline.length} steps from this session</span>
                  </div>
                  <div className="debug-step-controls-hint">
                    <span>Use arrow keys or click on a step to navigate</span>
                  </div>
                </div>
              )}
            </div>

            {/* All steps list when nothing selected */}
            {selectedStepIndex === null && (
              <div className="debug-all-steps">
                <h3>All Steps</h3>
                <div className="debug-steps-list">
                  {timeline.map((step, index) => (
                    <div
                      key={step.id}
                      className={`debug-step-card ${step.type}`}
                      onClick={() => setSelectedStepIndex(index)}
                    >
                      <div className="debug-step-card-header">
                        <span className="step-number">{index + 1}</span>
                        <span className="step-icon">{getStepIcon(step.type)}</span>
                        <span className={`step-type-badge ${step.type}`}>{step.type}</span>
                        <span className="step-time">{formatTimestamp(step.timestamp)}</span>
                      </div>
                      {step.role && <div className="step-role">Role: {step.role}</div>}
                      {step.toolName && <div className="step-tool">Tool: {step.toolName}</div>}
                      {step.contentBlockType && <div className="step-content-type">Type: {step.contentBlockType}</div>}
                      {step.usage && (
                        <div className="step-usage">
                          {formatTokens(step.usage.input_tokens + step.usage.output_tokens)} tokens
                        </div>
                      )}
                      {step.attachments && step.attachments.length > 0 && (
                        <div className="step-attachments">
                          📎 {step.attachments.length} attachment{step.attachments.length > 1 ? 's' : ''}
                        </div>
                      )}
                      {step.toolError && (
                        <div className="step-error">❌ {step.toolError}</div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
            </>
            )}

            {!selectedConversation && (
              <div className="debug-empty-state">
                <p>Select a bead to view its execution timeline</p>
              </div>
            )}
          </>
        ) : (
          <div className="debug-empty-state">
            <p>Select a bead to view debug information</p>
            <p className="empty-hint">Beads with worker sessions will appear in the sidebar</p>
          </div>
        )}
      </div>
    </div>
  );
}

function StepDetail({ step, stepNumber, totalSteps }: { step: TimelineStep; stepNumber: number; totalSteps: number }) {
  const contentStr = step.content ? formatContent(step.content) : '';

  // Render gap step
  if (step.type === 'gap') {
    const gapSeconds = Math.floor((step.gapBefore?.gapMs || 0) / 1000);
    const gapMinutes = Math.floor(gapSeconds / 60);
    return (
      <div className="debug-step-detail debug-gap-step">
        <div className="debug-step-detail-header">
          <div className="debug-step-detail-header-top">
            <div className="step-header-left">
              <span className="step-number-badge">{stepNumber} / {totalSteps}</span>
              <span className={`step-type-badge gap large`}>⏱ Gap Detected</span>
              <span className="step-source-badge">inferred</span>
            </div>
            <div className="step-header-right">
              <span className="step-time">{formatTimestamp(step.timestamp)}</span>
            </div>
          </div>
        </div>
        <div className="debug-step-content gap-content">
          <div className="gap-indicator">
            <span className="gap-icon">⚠️</span>
            <div className="gap-details">
              <h4>Temporal Gap Detected</h4>
              <p className="gap-description">
                {gapMinutes > 0 ? `${gapMinutes}m ${gapSeconds % 60}s` : `${gapSeconds}s`} of inactivity
              </p>
              <p className="gap-hint">No events or messages were recorded during this period. This could indicate:</p>
              <ul className="gap-reasons">
                <li>Long-running tool execution (e.g., large build, file download)</li>
                <li>Worker processing without intermediate updates</li>
                <li>Missing data due to session interruption</li>
                <li>External wait (user input, network request, etc.)</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Render bead event step from events.jsonl
  if (step.type.startsWith('bead_')) {
    return (
      <div className="debug-step-detail debug-bead-event-step">
        <div className="debug-step-detail-header">
          <div className="debug-step-detail-header-top">
            <div className="step-header-left">
              <span className="step-number-badge">{stepNumber} / {totalSteps}</span>
              <span className={`step-type-badge ${step.type} large`}>{step.eventType || step.type}</span>
              <span className="step-source-badge">events.jsonl</span>
            </div>
            <div className="step-header-right">
              <span className="step-time">{formatTimestamp(step.timestamp)}</span>
            </div>
          </div>
          {step.beadId && <div className="step-bead-id">Bead: {step.beadId}</div>}
          {step.worker && <div className="step-worker">Worker: {step.worker}</div>}
          {step.lineNumber && <div className="step-line-number">Line: {step.lineNumber}</div>}
        </div>
        <div className="debug-step-content">
          <h4>Event Description</h4>
          <p className="bead-event-description">{contentStr}</p>
          {step.rawEvent && (
            <details className="raw-event-details">
              <summary>Raw Event</summary>
              <pre className="debug-pre">{step.rawEvent}</pre>
            </details>
          )}
        </div>
      </div>
    );
  }

  // Render stderr step
  if (step.type === 'stderr') {
    return (
      <div className="debug-step-detail debug-stderr-step">
        <div className="debug-step-detail-header">
          <div className="debug-step-detail-header-top">
            <div className="step-header-left">
              <span className="step-number-badge">{stepNumber} / {totalSteps}</span>
              <span className={`step-type-badge ${step.type} large`}>stderr</span>
            </div>
            <div className="step-header-right">
              <span className="step-time">{formatTimestamp(step.timestamp)}</span>
            </div>
          </div>
        </div>
        <div className="debug-step-content">
          <h4>Standard Error Output</h4>
          <pre className="debug-pre stderr-content">{contentStr}</pre>
        </div>
      </div>
    );
  }

  // Render state transition step
  if (step.type === 'state_transition') {
    return (
      <div className="debug-step-detail debug-state-transition-step">
        <div className="debug-step-detail-header">
          <div className="debug-step-detail-header-top">
            <div className="step-header-left">
              <span className="step-number-badge">{stepNumber} / {totalSteps}</span>
              <span className={`step-type-badge ${step.type} large`}>State Transition</span>
            </div>
            <div className="step-header-right">
              <span className="step-time">{formatTimestamp(step.timestamp)}</span>
            </div>
          </div>
        </div>
        <div className="debug-step-content">
          <div className="state-transition">
            <span className="state-from">{step.fromState}</span>
            <span className="state-arrow">→</span>
            <span className="state-to">{step.toState}</span>
          </div>
          {contentStr && <pre className="debug-pre">{contentStr}</pre>}
        </div>
      </div>
    );
  }

  return (
    <div className="debug-step-detail">
      <div className="debug-step-detail-header">
        <div className="debug-step-detail-header-top">
          <div className="step-header-left">
            <span className="step-number-badge">{stepNumber} / {totalSteps}</span>
            <span className={`step-type-badge ${step.type} large`}>{step.type}</span>
          </div>
          <div className="step-header-right">
            <span className="step-time">{formatTimestamp(step.timestamp)}</span>
            {step.role && <span className="step-role-detail">Role: {step.role}</span>}
          </div>
        </div>
        {step.toolName && <div className="step-tool">Tool Called: <code>{step.toolName}</code></div>}
        {step.contentBlockType && <div className="step-content-type">Content Block: <code>{step.contentBlockType}</code></div>}
        {step.stopReason && <div className="step-stop-reason">Stop: {step.stopReason}</div>}
      </div>

      {/* Token usage */}
      {step.usage && (
        <div className="debug-step-usage">
          <span className="usage-item">
            <span className="usage-label">Input:</span>
            <span className="usage-value">{formatTokens(step.usage.input_tokens)}</span>
          </span>
          <span className="usage-item">
            <span className="usage-label">Output:</span>
            <span className="usage-value">{formatTokens(step.usage.output_tokens)}</span>
          </span>
          {step.usage.cache_read_tokens > 0 && (
            <span className="usage-item">
              <span className="usage-label">Cache Read:</span>
              <span className="usage-value">{formatTokens(step.usage.cache_read_tokens)}</span>
            </span>
          )}
          {step.usage.cache_write_tokens > 0 && (
            <span className="usage-item">
              <span className="usage-label">Cache Write:</span>
              <span className="usage-value">{formatTokens(step.usage.cache_write_tokens)}</span>
            </span>
          )}
        </div>
      )}

      {/* Attachments */}
      {step.attachments && step.attachments.length > 0 && (
        <div className="debug-step-attachments">
          <h4>Attachments ({step.attachments.length})</h4>
          <div className="attachments-list">
            {step.attachments.map((path, i) => (
              <div key={i} className="attachment-item">
                <span className="attachment-icon">📎</span>
                <code className="attachment-path">{path}</code>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Tool input */}
      {step.toolInput && (
        <div className="debug-step-tool-input">
          <h4>Tool Input</h4>
          <pre className="debug-pre">{JSON.stringify(step.toolInput, null, 2)}</pre>
        </div>
      )}

      {/* Tool result */}
      {step.toolResult && (
        <div className="debug-step-tool-result">
          <h4>Tool Result</h4>
          {step.toolError ? (
            <div className="tool-error">
              <span className="error-icon">❌</span>
              <span className="error-message">{step.toolError}</span>
            </div>
          ) : (
            <pre className="debug-pre">{formatContent(step.toolResult)}</pre>
          )}
        </div>
      )}

      {/* Content */}
      {contentStr && (
        <div className="debug-step-content">
          <h4>Content</h4>
          {step.contentBlockType === 'thinking' ? (
            <div className="thinking-block">
              <details>
                <summary>Thinking process</summary>
                <pre className="debug-pre">{contentStr}</pre>
              </details>
            </div>
          ) : (
            <pre className="debug-pre">{contentStr}</pre>
          )}
        </div>
      )}

      {/* Model info if available */}
      {step.model && (
        <div className="debug-step-model">
          <span className="model-label">Model:</span>
          <span className="model-value">{step.model}</span>
        </div>
      )}
    </div>
  );
}
