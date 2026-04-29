import { useAtomValue } from 'jotai';
import { presenceForProjectAtom, presenceForStitchAtom, presenceVisibilityAtom } from '../atoms';

interface PresenceIndicatorProps {
  scope: 'project' | 'stitch';
  id: string;
}

export function PresenceIndicator({ scope, id }: PresenceIndicatorProps) {
  const myVisibility = useAtomValue(presenceVisibilityAtom);
  const presenceAtom = scope === 'project' ? presenceForProjectAtom(id) : presenceForStitchAtom(id);
  const presence = useAtomValue(presenceAtom);

  // Filter out hidden operators (privacy toggle) and removed entries
  const visiblePresence = presence.filter(p => p.visibility === 'visible');

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
            title={`${formatOperatorId(p.operator_id)}${p.visibility === 'hidden' ? ' (hidden)' : ''}`}
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
