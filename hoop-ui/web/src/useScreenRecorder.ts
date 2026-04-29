import { useRef, useEffect, useState, useCallback } from 'react';
import { useSetAtom } from 'jotai';
import { screenCapturesAtom, ScreenCaptureSummary } from './atoms';

export type ScreenRecordPhase = 'idle' | 'selecting' | 'recording' | 'uploading' | 'error';

export interface ScreenRecorderState {
  phase: ScreenRecordPhase;
  duration: number;
  error: string | null;
  previewStream: MediaStream | null;
  startRecording: () => void;
  stopRecording: () => void;
  clearError: () => void;
  uploadProgress: number; // bytes uploaded
}

export interface FrameSample {
  timestamp_secs: number;
  label: string;
  thumbnail?: string; // base64 data URL
}

function extractFrameFromVideo(video: HTMLVideoElement): Promise<string> {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      reject(new Error('Failed to get canvas context'));
      return;
    }

    // Use actual video dimensions for better quality
    const scale = Math.min(320 / video.videoWidth, 180 / video.videoHeight, 1);
    canvas.width = video.videoWidth * scale;
    canvas.height = video.videoHeight * scale;

    // Draw current frame directly from live stream (no seeking)
    try {
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      const dataUrl = canvas.toDataURL('image/jpeg', 0.7);
      resolve(dataUrl);
    } catch (err) {
      reject(new Error(`Failed to draw frame: ${err}`));
    }
  });
}

export function useScreenRecorder(projectName: string): ScreenRecorderState {
  const setScreenCaptures = useSetAtom(screenCapturesAtom);

  const [phase, setPhase] = useState<ScreenRecordPhase>('idle');
  const [duration, setDuration] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [previewStream, setPreviewStream] = useState<MediaStream | null>(null);
  const [uploadProgress, setUploadProgress] = useState(0);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const startTimeRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const frameSampleIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const frameSamplesRef = useRef<FrameSample[]>([]);
  const videoElementRef = useRef<HTMLVideoElement | null>(null);
  const projectRef = useRef(projectName);
  const phaseRef = useRef(phase);
  const streamIdRef = useRef<string | null>(null);
  const uploadInProgressRef = useRef(false);
  const pendingChunksRef = useRef<Blob[]>([]);

  useEffect(() => { projectRef.current = projectName; }, [projectName]);
  useEffect(() => { phaseRef.current = phase; }, [phase]);

  // Store uploadChunk as a ref to avoid circular dependency
  const uploadChunkRef = useRef<(chunk: Blob) => Promise<boolean>>(async () => false);
  uploadChunkRef.current = async (chunk: Blob): Promise<boolean> => {
    const streamId = streamIdRef.current;
    if (!streamId) return false;

    const project = projectRef.current;
    if (!project) return false;

    try {
      const arrayBuffer = await chunk.arrayBuffer();
      const resp = await fetch(
        `/api/p/${encodeURIComponent(project)}/screen-captures/stream/${streamId}`,
        {
          method: 'PATCH',
          headers: { 'Content-Type': 'application/octet-stream' },
          body: arrayBuffer,
        }
      );

      if (!resp.ok) {
        const text = await resp.text();
        console.error('Failed to upload chunk:', text);
        return false;
      }

      const result = await resp.json();
      setUploadProgress(result.received_bytes || 0);
      return true;
    } catch (err) {
      console.error('Error uploading chunk:', err);
      return false;
    }
  };

  const stopRecording = useCallback(async () => {
    if (mediaRecorderRef.current?.state === 'recording') {
      mediaRecorderRef.current.stop();
    }
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    if (frameSampleIntervalRef.current) {
      clearInterval(frameSampleIntervalRef.current);
      frameSampleIntervalRef.current = null;
    }
    if (previewStream) {
      previewStream.getTracks().forEach(t => t.stop());
      setPreviewStream(null);
    }

    // Complete the streaming upload
    const streamId = streamIdRef.current;
    if (streamId && !uploadInProgressRef.current) {
      uploadInProgressRef.current = true;
      const elapsed = (Date.now() - startTimeRef.current) / 1000;

      if (elapsed >= 2) {
        setPhase('uploading');

        try {
          const project = projectRef.current;
          const resp = await fetch(
            `/api/p/${encodeURIComponent(project)}/screen-captures/stream/${streamId}/complete`,
            {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                duration_secs: elapsed,
                frame_samples: frameSamplesRef.current,
              }),
            }
          );

          if (!resp.ok) {
            const text = await resp.text();
            throw new Error(text || resp.statusText);
          }

          const created = await resp.json();

          const newCapture: ScreenCaptureSummary = {
            stitch_id: created.stitch_id,
            project,
            title: created.title,
            recorded_at: created.recorded_at,
            duration_secs: elapsed,
            chapter_count: frameSamplesRef.current.length,
            has_transcript: false,
          };

          setScreenCaptures(prev => {
            const next = new Map(prev);
            const existing = next.get(project) ?? [];
            next.set(project, [newCapture, ...existing]);
            return next;
          });
        } catch (err) {
          setError(err instanceof Error ? err.message : 'Upload failed');
          setPhase('error');
        }
      }

      streamIdRef.current = null;
      uploadInProgressRef.current = false;
    }

    setPhase('idle');
    setDuration(0);
    setUploadProgress(0);
    frameSamplesRef.current = [];
    pendingChunksRef.current = [];
  }, [previewStream, setScreenCaptures]);

  const captureFrameSample = useCallback(async () => {
    const video = videoElementRef.current;
    if (!video || video.readyState < 2) return;

    const timestampSecs = (Date.now() - startTimeRef.current) / 1000;

    try {
      const thumbnail = await extractFrameFromVideo(video);

      const sample: FrameSample = {
        timestamp_secs: timestampSecs,
        label: `Chapter ${(frameSamplesRef.current.length + 1).toString()}`,
        thumbnail,
      };

      frameSamplesRef.current.push(sample);
    } catch (err) {
      console.warn('Failed to extract frame sample:', err);
    }
  }, []);

  const startRecording = useCallback(async () => {
    if (phaseRef.current !== 'idle') return;
    const project = projectRef.current;
    if (!project) return;

    // Browser compatibility check
    if (!navigator.mediaDevices || !navigator.mediaDevices.getDisplayMedia) {
      setError('Your browser does not support screen capture. Please use Chrome 72+, Firefox 66+, or Safari 13+.');
      setPhase('error');
      return;
    }

    // Check MediaRecorder support
    if (typeof MediaRecorder === 'undefined') {
      setError('MediaRecorder is not supported in your browser. Please update to a modern browser.');
      setPhase('error');
      return;
    }

    setPhase('selecting');

    let stream: MediaStream;
    try {
      // Request screen capture with audio
      stream = await navigator.mediaDevices.getDisplayMedia({
        video: {
          displaySurface: 'monitor',
          frameRate: { ideal: 30 },
          height: { ideal: 1080 },
          width: { ideal: 1920 },
        },
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          sampleRate: 44100,
          channelCount: 2,
        },
      });
    } catch (err) {
      if (err instanceof Error && err.name === 'NotAllowedError') {
        setError('Screen capture was cancelled');
      } else if (err instanceof Error && err.name === 'NotSupportedError') {
        setError('Screen capture with audio is not supported. Try selecting a browser tab or window instead of the entire screen.');
      } else {
        setError(err instanceof Error ? err.message : 'Failed to capture screen');
      }
      setPhase('error');
      return;
    }

    setPreviewStream(stream);

    const videoTrack = stream.getVideoTracks()[0];
    const audioTrack = stream.getAudioTracks()?.[0];

    if (!videoTrack) {
      stream.getTracks().forEach(t => t.stop());
      setError('No video track in captured stream');
      setPhase('error');
      return;
    }

    // Log audio capture status (browser support varies)
    if (!audioTrack) {
      console.warn('No audio track captured. The selected source may not support audio, or the browser may not support audio capture with screen share. Try sharing a browser tab instead of the entire screen.');
    }

    // Detect Safari for better format selection
    const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);

    // Prefer MP4/H.264 for Safari (better compatibility), WebM/VP9 for others
    let mimeType: string;
    if (isSafari) {
      mimeType = MediaRecorder.isTypeSupported('video/mp4;codecs=h264,aac')
        ? 'video/mp4;codecs=h264,aac'
        : MediaRecorder.isTypeSupported('video/mp4')
        ? 'video/mp4'
        : MediaRecorder.isTypeSupported('video/webm;codecs=vp8,opus')
        ? 'video/webm;codecs=vp8,opus'
        : 'video/webm';
    } else {
      mimeType = MediaRecorder.isTypeSupported('video/webm;codecs=vp9,opus')
        ? 'video/webm;codecs=vp9,opus'
        : MediaRecorder.isTypeSupported('video/webm;codecs=vp8,opus')
        ? 'video/webm;codecs=vp8,opus'
        : MediaRecorder.isTypeSupported('video/webm')
        ? 'video/webm'
        : MediaRecorder.isTypeSupported('video/mp4;codecs=h264,aac')
        ? 'video/mp4;codecs=h264,aac'
        : MediaRecorder.isTypeSupported('video/mp4')
        ? 'video/mp4'
        : 'video/webm';
    }

    // Start streaming upload session
    try {
      const startResp = await fetch(`/api/p/${encodeURIComponent(project)}/screen-captures/stream`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          project,
          video_content_type: mimeType,
        }),
      });

      if (!startResp.ok) {
        const text = await startResp.text();
        throw new Error(text || startResp.statusText);
      }

      const startData = await startResp.json();
      streamIdRef.current = startData.stream_id;
    } catch (err) {
      stream.getTracks().forEach(t => t.stop());
      setError(err instanceof Error ? err.message : 'Failed to start upload session');
      setPhase('error');
      return;
    }

    // Configure MediaRecorder with bitrate settings
    const recorderOptions: MediaRecorderOptions = {
      mimeType,
      videoBitsPerSecond: 2500000, // 2.5 Mbps video
    };

    // Add audio bitrate if audio track is present and format supports it
    if (audioTrack) {
      recorderOptions.audioBitsPerSecond = 128000; // 128 kbps audio
    }

    const recorder = new MediaRecorder(stream, recorderOptions);
    mediaRecorderRef.current = recorder;
    frameSamplesRef.current = [];
    pendingChunksRef.current = [];

    // Stream chunks as they become available
    recorder.ondataavailable = async (e) => {
      if (e.data.size > 0) {
        pendingChunksRef.current.push(e.data);
        // Upload chunk immediately (streaming)
        const success = await uploadChunkRef.current(e.data);
        if (!success) {
          console.warn('Failed to upload chunk, will retry on next chunk');
        }
      }
    };

    recorder.onstop = async () => {
      // Final upload happens in stopRecording callback
      stream.getTracks().forEach(t => t.stop());
      setPreviewStream(null);

      // Clean up video element
      if (videoElementRef.current) {
        videoElementRef.current.pause();
        videoElementRef.current.srcObject = null;
        videoElementRef.current = null;
      }
    };

    const video = document.createElement('video');
    video.srcObject = stream;
    video.muted = true;
    video.play();
    videoElementRef.current = video;

    await new Promise(resolve => {
      video.onloadedmetadata = resolve;
    });

    // Emit chunks every 1 second for streaming
    recorder.start(1000);
    startTimeRef.current = Date.now();
    setPhase('recording');
    setDuration(0);
    setUploadProgress(0);

    timerRef.current = setInterval(() => {
      setDuration((Date.now() - startTimeRef.current) / 1000);
    }, 100);

    frameSampleIntervalRef.current = setInterval(() => {
      captureFrameSample();
    }, 5000);

    videoTrack.onended = () => {
      stopRecording();
    };
  }, [captureFrameSample, stopRecording]);

  return {
    phase,
    duration,
    error,
    previewStream,
    startRecording,
    stopRecording,
    clearError: () => {
      setError(null);
      setPhase('idle');
    },
    uploadProgress,
  };
}
