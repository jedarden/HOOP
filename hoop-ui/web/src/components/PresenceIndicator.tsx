import { useAtomValue } from 'jotai';
import { presenceForProjectAtom, presenceForStitchAtom } from '../atoms';
import type { Presence } from '../types.gen';

interface PresenceIndicatorProps {
  scope: 'project' | 'stitch';
  id: string;
}

export function PresenceIndicator({ scope, id }: PresenceIndicatorProps) {
  const projectPresence = useAtomValue(presenceForProjectAtom);
  const stitchPresence = useAtomValue(presenceForStitchAtom);

  // Get the operator IDs for this scope
  const operatorIds = scope === 'project'
    ? projectPresence.get(id)
    : stitchPresence.get(id);

  if (!operatorIds || operatorIds.size === 0) return null;

  // Convert Set of operator IDs to Presence-like objects for display
  const visiblePresence = Array.from(operatorIds).map(operatorId => ({
    operator_id: operatorId,
    visibility: 'visible' as const,
  }));

  if (visiblePresence.length === 0) return null;

  // Generate color for each operator based on their ID
  const getOperatorColor = (operatorId: string): string => {
    let hash = 0;
    for (let i = 0; i < operatorId.length; i++) {
      hash = operatorId.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash % 360);
    return `hsl(${hue}, 70%, 50%)`;
  };

  // Format operator ID for display
  const formatOperatorId = (operatorId: string): string => {
    // Extract username from tailscale:user@example.com or os:username
    if (operatorId.startsWith('tailscale:')) {
      const email = operatorId.slice(11);
      const username = email.split('@')[0];
      return username || email;
    }
    if (operatorId.startsWith('os:')) {
      return operatorId.slice(3);
    }
    return operatorId;
  };

  // Limit display to at most 5 dots
  const displayPresence = visiblePresence.slice(0, 5);
  const extraCount = visiblePresence.length - 5;

  return (
    <div className="presence-indicator" title={`${visiblePresence.length} operator${visiblePresence.length > 1 ? 's' : ''} viewing`}>
      <div className="presence-dots">
        {displayPresence.map((p) => (
          <div
            key={p.operator_id}
            className="presence-dot"
            style={{ backgroundColor: getOperatorColor(p.operator_id) }}
            title={formatOperatorId(p.operator_id)}
          />
        ))}
        {extraCount > 0 && (
          <div className="presence-dot presence-dot-extra" title={`+${extraCount} more`}>
            +{extraCount}
          </div>
        )}
      </div>
    </div>
  );
}
