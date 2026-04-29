import { useAtomValue } from 'jotai';
import { activeProjectNameAtom } from '../atoms';
import { useScreenRecorder } from '../useScreenRecorder';

function formatDuration(secs: number): string {
  const s = Math.floor(secs);
  const m = Math.floor(s / 60);
  return `${m}:${(s % 60).toString().padStart(2, '0')}`;
}

interface ScreenCaptureWidgetProps {
  onStart?: () => void;
  onStop?: () => void;
}

export function ScreenCaptureWidget({ onStart, onStop }: ScreenCaptureWidgetProps) {
  const projectName = useAtomValue(activeProjectNameAtom);
  const { phase, duration, error, startRecording, stopRecording, clearError } =
    useScreenRecorder(projectName ?? '');

  const handleStart = () => {
    onStart?.();
    startRecording();
  };

  const handleStop = () => {
    onStop?.();
    stopRecording();
  };

  if (phase === 'error' && error) {
    return (
      <div className="screen-capture-widget screen-capture-widget--error" role="alert">
        <span className="screen-capture-error-icon" aria-hidden="true">
          ⚠
        </span>
        <span className="screen-capture-error-text">{error}</span>
        <button
          className="screen-capture-btn screen-capture-btn--dismiss"
          onClick={clearError}
        >
          ✕
        </button>
      </div>
    );
  }

  if (phase === 'selecting') {
    return (
      <div className="screen-capture-widget screen-capture-widget--selecting" aria-live="polite">
        <span className="screen-capture-spinner" aria-hidden="true" />
        <span>Select screen or window to record…</span>
      </div>
    );
  }

  if (phase === 'uploading') {
    return (
      <div className="screen-capture-widget screen-capture-widget--uploading" aria-live="polite">
        <span className="screen-capture-spinner" aria-hidden="true" />
        <span>Uploading screen capture…</span>
      </div>
    );
  }

  if (phase === 'recording') {
    return (
      <div
        className="screen-capture-widget screen-capture-widget--recording"
        role="status"
        aria-live="assertive"
      >
        <span className="screen-capture-rec-dot" aria-label="Recording" />
        <span className="screen-capture-timer">{formatDuration(duration)}</span>
        <span className="screen-capture-stop-hint">Recording…</span>
        <button
          className="screen-capture-btn screen-capture-btn--stop"
          onClick={handleStop}
          aria-label="Stop recording"
        >
          ■ Stop
        </button>
      </div>
    );
  }

  // Idle
  return (
    <div
      className={`screen-capture-widget screen-capture-widget--idle${
        !projectName ? ' screen-capture-widget--no-project' : ''
      }`}
    >
      <span className="screen-capture-icon" aria-hidden="true">
        📹
      </span>
      <span className="screen-capture-label">Screen Capture</span>
      {projectName && (
        <span className="screen-capture-project-name" title={projectName}>
          {projectName}
        </span>
      )}
      <button
        className="screen-capture-btn screen-capture-btn--record"
        onClick={handleStart}
        disabled={!projectName}
        title={projectName ? 'Start screen recording' : 'Select a project first'}
        aria-label="Start screen recording"
      >
        ● Record
      </button>
    </div>
  );
}
