import { useRef, useEffect, useState, useCallback } from 'react';
import { useAtomValue, useSetAtom } from 'jotai';
import { dictationHotkeyAtom, dictatedNotesAtom, DictationHotkey, NoteSummary } from './atoms';

export type DictationPhase = 'idle' | 'recording' | 'uploading' | 'error';

export interface DictationRecorderState {
  phase: DictationPhase;
  duration: number;
  error: string | null;
  analyser: AnalyserNode | null;
  clearError: () => void;
}

function matchesHotkey(e: KeyboardEvent, hk: DictationHotkey): boolean {
  return (
    e.key.toLowerCase() === hk.key.toLowerCase() &&
    e.metaKey === hk.meta &&
    e.ctrlKey === hk.ctrl &&
    e.shiftKey === hk.shift &&
    e.altKey === hk.alt
  );
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  const chunk = 8192;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + chunk)));
  }
  return btoa(binary);
}

export function useDictationRecorder(projectName: string): DictationRecorderState {
  const hotkey = useAtomValue(dictationHotkeyAtom);
  const setDictatedNotes = useSetAtom(dictatedNotesAtom);

  const [phase, setPhase] = useState<DictationPhase>('idle');
  const [duration, setDuration] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [analyser, setAnalyser] = useState<AnalyserNode | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const startTimeRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const keyHeldRef = useRef(false);
  const projectRef = useRef(projectName);
  const phaseRef = useRef(phase);

  useEffect(() => { projectRef.current = projectName; }, [projectName]);
  useEffect(() => { phaseRef.current = phase; }, [phase]);

  const stopRecording = useCallback(() => {
    if (mediaRecorderRef.current?.state === 'recording') {
      mediaRecorderRef.current.stop();
    }
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const startRecording = useCallback(async () => {
    if (phaseRef.current !== 'idle') return;
    const project = projectRef.current;
    if (!project) return;

    let stream: MediaStream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Microphone access denied');
      setPhase('error');
      return;
    }

    const ctx = new AudioContext();
    const source = ctx.createMediaStreamSource(stream);
    const analyserNode = ctx.createAnalyser();
    analyserNode.fftSize = 512;
    source.connect(analyserNode);
    setAnalyser(analyserNode);

    const mimeType = MediaRecorder.isTypeSupported('audio/webm;codecs=opus')
      ? 'audio/webm;codecs=opus'
      : MediaRecorder.isTypeSupported('audio/webm')
      ? 'audio/webm'
      : 'audio/mp4';

    const recorder = new MediaRecorder(stream, { mimeType });
    mediaRecorderRef.current = recorder;
    chunksRef.current = [];

    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunksRef.current.push(e.data);
    };

    recorder.onstop = async () => {
      stream.getTracks().forEach(t => t.stop());
      ctx.close();
      setAnalyser(null);

      const elapsed = (Date.now() - startTimeRef.current) / 1000;

      if (elapsed < 2) {
        // Too short — discard silently
        setPhase('idle');
        setDuration(0);
        return;
      }

      setPhase('uploading');

      const blob = new Blob(chunksRef.current, { type: mimeType });
      const ext = mimeType.includes('webm') ? 'webm' : 'mp4';
      const ts = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      const filename = `note-${ts}.${ext}`;

      try {
        const arrayBuffer = await blob.arrayBuffer();
        const base64 = arrayBufferToBase64(arrayBuffer);
        const proj = projectRef.current;

        const resp = await fetch(`/api/p/${encodeURIComponent(proj)}/dictated-notes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            project: proj,
            audio_data: base64,
            audio_filename: filename,
            audio_content_type: mimeType,
            duration_secs: elapsed,
          }),
        });

        if (!resp.ok) {
          const text = await resp.text();
          throw new Error(text || resp.statusText);
        }

        const created = await resp.json();

        const newNote: NoteSummary = {
          stitch_id: created.stitch_id,
          project: proj,
          title: created.title,
          kind: 'dictated',
          recorded_at: created.recorded_at,
          transcribed_at: created.transcribed_at ?? created.recorded_at,
          duration_secs: elapsed,
          language: null,
          tags: [],
          transcript_preview: '',
          transcript: '',
          last_activity_at: created.recorded_at,
          created_at: created.recorded_at,
          audio_filename: filename,
          transcription_status: 'Pending',
        };

        setDictatedNotes(prev => {
          const next = new Map(prev);
          const existing = next.get(proj) ?? [];
          next.set(proj, [newNote, ...existing]);
          return next;
        });

        setPhase('idle');
        setDuration(0);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Upload failed');
        setPhase('error');
      }
    };

    recorder.start(100);
    startTimeRef.current = Date.now();
    setPhase('recording');
    setDuration(0);

    timerRef.current = setInterval(() => {
      setDuration((Date.now() - startTimeRef.current) / 1000);
    }, 100);
  }, [setDictatedNotes]);

  // Global hotkey binding
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!matchesHotkey(e, hotkey)) return;
      if (keyHeldRef.current) return;
      const target = e.target as HTMLElement;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target.isContentEditable
      ) return;

      e.preventDefault();
      keyHeldRef.current = true;
      startRecording();
    };

    const onKeyUp = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() !== hotkey.key.toLowerCase()) return;
      if (!keyHeldRef.current) return;
      keyHeldRef.current = false;
      stopRecording();
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [hotkey, startRecording, stopRecording]);

  return {
    phase,
    duration,
    error,
    analyser,
    clearError: () => {
      setError(null);
      setPhase('idle');
    },
  };
}
