import { useRef, useEffect, useState, useCallback } from 'react';
import { useAtom, useAtomValue } from 'jotai';
import { dictationHotkeyAtom, DictationHotkey, DICTATION_HOTKEY_STORAGE_KEY, activeProjectNameAtom } from '../atoms';
import { useDictationRecorder } from '../useDictationRecorder';

function formatHotkey(hk: DictationHotkey): string {
  const parts: string[] = [];
  if (hk.ctrl) parts.push('Ctrl');
  if (hk.meta) parts.push('⌘');
  if (hk.alt) parts.push('⌥');
  if (hk.shift) parts.push('⇧');
  parts.push(hk.key.toUpperCase());
  return parts.join('+');
}

function formatDuration(secs: number): string {
  const s = Math.floor(secs);
  const m = Math.floor(s / 60);
  return `${m}:${(s % 60).toString().padStart(2, '0')}`;
}

function Oscilloscope({ analyser }: { analyser: AnalyserNode }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const bufferLen = analyser.frequencyBinCount;
    const data = new Uint8Array(bufferLen);

    const draw = () => {
      rafRef.current = requestAnimationFrame(draw);
      analyser.getByteTimeDomainData(data);

      const W = canvas.width;
      const H = canvas.height;
      ctx.clearRect(0, 0, W, H);

      ctx.lineWidth = 2;
      ctx.strokeStyle = '#ef4444';
      ctx.beginPath();

      const sliceW = W / bufferLen;
      let x = 0;
      for (let i = 0; i < bufferLen; i++) {
        const v = data[i] / 128.0;
        const y = (v * H) / 2;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
        x += sliceW;
      }
      ctx.lineTo(W, H / 2);
      ctx.stroke();
    };

    draw();
    return () => cancelAnimationFrame(rafRef.current);
  }, [analyser]);

  return (
    <canvas
      ref={canvasRef}
      className="dictation-oscilloscope"
      width={200}
      height={40}
      aria-hidden="true"
    />
  );
}

function HotkeyBinder({
  current,
  onChange,
  onCancel,
}: {
  current: DictationHotkey;
  onChange: (hk: DictationHotkey) => void;
  onCancel: () => void;
}) {
  const [captured, setCaptured] = useState<DictationHotkey | null>(null);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) return;
      setCaptured({
        key: e.key.toLowerCase(),
        meta: e.metaKey,
        ctrl: e.ctrlKey,
        shift: e.shiftKey,
        alt: e.altKey,
      });
    };
    window.addEventListener('keydown', onKeyDown, { capture: true });
    return () => window.removeEventListener('keydown', onKeyDown, { capture: true });
  }, []);

  return (
    <div className="dictation-hotkey-binder">
      <p className="dictation-binder-prompt">Press a key combination…</p>
      {captured ? (
        <>
          <kbd className="dictation-kbd">{formatHotkey(captured)}</kbd>
          <div className="dictation-binder-actions">
            <button className="dictation-btn dictation-btn--save" onClick={() => onChange(captured)}>
              Save
            </button>
            <button className="dictation-btn" onClick={onCancel}>
              Cancel
            </button>
          </div>
        </>
      ) : (
        <>
          <p className="dictation-binder-current">
            Current: <kbd className="dictation-kbd">{formatHotkey(current)}</kbd>
          </p>
          <button className="dictation-btn" onClick={onCancel}>
            Cancel
          </button>
        </>
      )}
    </div>
  );
}

export function DictationWidget() {
  const projectName = useAtomValue(activeProjectNameAtom);
  const [hotkey, setHotkeyAtom] = useAtom(dictationHotkeyAtom);
  const [showSettings, setShowSettings] = useState(false);
  const [binding, setBinding] = useState(false);

  const { phase, duration, error, analyser, clearError } = useDictationRecorder(projectName);

  const handleHotkeyChange = useCallback(
    (hk: DictationHotkey) => {
      setHotkeyAtom(hk);
      try {
        localStorage.setItem(DICTATION_HOTKEY_STORAGE_KEY, JSON.stringify(hk));
      } catch {}
      setBinding(false);
      setShowSettings(false);
    },
    [setHotkeyAtom],
  );

  if (phase === 'error' && error) {
    return (
      <div className="dictation-widget dictation-widget--error" role="alert">
        <span className="dictation-error-icon" aria-hidden="true">⚠</span>
        <span className="dictation-error-text">{error}</span>
        <button className="dictation-btn dictation-btn--dismiss" onClick={clearError}>
          ✕
        </button>
      </div>
    );
  }

  if (phase === 'uploading') {
    return (
      <div className="dictation-widget dictation-widget--uploading" aria-live="polite">
        <span className="dictation-spinner" aria-hidden="true" />
        <span>Uploading note…</span>
      </div>
    );
  }

  if (phase === 'recording') {
    return (
      <div className="dictation-widget dictation-widget--recording" role="status" aria-live="assertive">
        <span className="dictation-rec-dot" aria-label="Recording" />
        <span className="dictation-timer">{formatDuration(duration)}</span>
        {analyser && <Oscilloscope analyser={analyser} />}
        <span className="dictation-stop-hint">Release {formatHotkey(hotkey)} to stop</span>
      </div>
    );
  }

  // Idle
  return (
    <div className={`dictation-widget dictation-widget--idle${!projectName ? ' dictation-widget--no-project' : ''}`}>
      {showSettings ? (
        binding ? (
          <HotkeyBinder
            current={hotkey}
            onChange={handleHotkeyChange}
            onCancel={() => {
              setBinding(false);
              setShowSettings(false);
            }}
          />
        ) : (
          <div className="dictation-settings-panel">
            <div className="dictation-settings-row">
              <span className="dictation-settings-label">Hotkey</span>
              <kbd className="dictation-kbd">{formatHotkey(hotkey)}</kbd>
              <button className="dictation-btn" onClick={() => setBinding(true)}>
                Rebind
              </button>
            </div>
            <button
              className="dictation-btn dictation-settings-close"
              onClick={() => setShowSettings(false)}
            >
              Close
            </button>
          </div>
        )
      ) : (
        <>
          <span className="dictation-mic-icon" aria-hidden="true">🎤</span>
          <span className="dictation-hotkey-label">{formatHotkey(hotkey)}</span>
          {projectName && (
            <span className="dictation-project-name" title={projectName}>
              {projectName}
            </span>
          )}
          <button
            className="dictation-gear-btn"
            onClick={() => setShowSettings(true)}
            title="Dictation settings"
            aria-label="Dictation settings"
          >
            ⚙
          </button>
        </>
      )}
    </div>
  );
}
