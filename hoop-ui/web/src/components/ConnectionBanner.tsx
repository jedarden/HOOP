import { useEffect, useState } from 'react';
import { useAtomValue } from 'jotai';
import { connectionStatusAtom, reconnectAttemptAtom, reconnectDelayAtom } from '../atoms';

export function ConnectionBanner() {
  const connectionStatus = useAtomValue(connectionStatusAtom);
  const reconnectAttempt = useAtomValue(reconnectAttemptAtom);
  const reconnectDelay = useAtomValue(reconnectDelayAtom);

  const [countdown, setCountdown] = useState(0);

  // Countdown timer for next reconnection attempt
  useEffect(() => {
    if (connectionStatus !== 'disconnected') {
      setCountdown(0);
      return;
    }

    // Reset countdown when delay changes
    setCountdown(Math.ceil(reconnectDelay / 1000));

    const interval = setInterval(() => {
      setCountdown(prev => {
        if (prev <= 1) {
          // Will reconnect, reset to show transition
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [connectionStatus, reconnectDelay]);

  if (connectionStatus === 'connected') {
    return null;
  }

  return (
    <div className="connection-banner" role="alert">
      <div className="banner-content">
        <span className="banner-icon">
          {connectionStatus === 'connecting' ? '⟳' : '⚠'}
        </span>
        <span className="banner-text">
          {connectionStatus === 'connecting' ? 'Connecting...' : (
            <>Disconnected — <span style={{ fontWeight: 'normal' }}>mutations disabled</span></>
          )}
        </span>
        {connectionStatus === 'disconnected' && reconnectDelay > 0 && (
          <span className="banner-retry">
            Retrying in {countdown}s (attempt {reconnectAttempt + 1})
          </span>
        )}
      </div>
    </div>
  );
}
