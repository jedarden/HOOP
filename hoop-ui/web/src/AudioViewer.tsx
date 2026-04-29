import { useCallback, useEffect, useRef, useState } from 'react';

export interface AudioViewerProps {
  projectName: string;
  path: string;
}

export function AudioViewer({ projectName, path }: AudioViewerProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [duration, setDuration] = useState<number | null>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [volume, setVolume] = useState(1);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [waveformData, setWaveformData] = useState<number[]>([]);

  const audioRef = useRef<HTMLAudioElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animationRef = useRef<number | null>(null);

  const audioUrl = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(path)}&raw=true`;
  const fileName = path.split('/').pop() ?? path;

  // Load audio and generate waveform
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleLoadStart = () => setLoading(true);
    const handleCanPlay = () => setLoading(false);
    const handleError = () => {
      setLoading(false);
      setError(true);
    };
    const handleLoadedMetadata = () => {
      setDuration(audio.duration);
      generateWaveform(audio);
    };

    audio.addEventListener('loadstart', handleLoadStart);
    audio.addEventListener('canplay', handleCanPlay);
    audio.addEventListener('error', handleError);
    audio.addEventListener('loadedmetadata', handleLoadedMetadata);

    return () => {
      audio.removeEventListener('loadstart', handleLoadStart);
      audio.removeEventListener('canplay', handleCanPlay);
      audio.removeEventListener('error', handleError);
      audio.removeEventListener('loadedmetadata', handleLoadedMetadata);
    };
  }, [audioUrl]);

  // Update current time during playback
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);
    const handleEnded = () => {
      setIsPlaying(false);
      setCurrentTime(0);
    };

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('play', handlePlay);
    audio.addEventListener('pause', handlePause);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('play', handlePlay);
      audio.removeEventListener('pause', handlePause);
      audio.removeEventListener('ended', handleEnded);
    };
  }, []);

  // Generate waveform visualization
  const generateWaveform = useCallback(async (audio: HTMLAudioElement) => {
    try {
      if (!audioContextRef.current) {
        audioContextRef.current = new AudioContext();
      }
      const ctx = audioContextRef.current;

      // Fetch audio data
      const response = await fetch(audioUrl);
      const arrayBuffer = await response.arrayBuffer();
      const audioBuffer = await ctx.decodeAudioData(arrayBuffer);

      // Extract PCM data
      const channelData = audioBuffer.getChannelData(0);
      const samples = 1000;
      const blockSize = Math.floor(channelData.length / samples);
      const waveform: number[] = [];

      for (let i = 0; i < samples; i++) {
        const start = i * blockSize;
        let sum = 0;
        for (let j = 0; j < blockSize; j++) {
          sum += Math.abs(channelData[start + j]);
        }
        waveform.push(sum / blockSize);
      }

      // Normalize to 0-1 range
      const max = Math.max(...waveform);
      setWaveformData(waveform.map(v => v / max));
    } catch (err) {
      console.warn('Failed to generate waveform:', err);
      setWaveformData([]);
    }
  }, [audioUrl]);

  // Draw waveform with progress
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || waveformData.length === 0) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const draw = () => {
      const width = rect.width;
      const height = rect.height;
      const centerY = height / 2;

      ctx.clearRect(0, 0, width, height);

      // Draw waveform
      const barWidth = width / waveformData.length;
      const progressIndex = duration ? Math.floor((currentTime / duration) * waveformData.length) : 0;

      waveformData.forEach((amplitude, i) => {
        const barHeight = amplitude * height * 0.8;
        const x = i * barWidth;
        const isPlayed = i <= progressIndex;

        ctx.fillStyle = isPlayed ? '#4285f4' : '#e0e0e0';
        ctx.fillRect(x, centerY - barHeight / 2, barWidth - 1, barHeight);
      });

      if (isPlaying) {
        animationRef.current = requestAnimationFrame(draw);
      }
    };

    draw();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [waveformData, currentTime, duration, isPlaying]);

  // Playback controls
  const togglePlayback = () => {
    const audio = audioRef.current;
    if (!audio) return;

    if (isPlaying) {
      audio.pause();
    } else {
      audio.play();
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const audio = audioRef.current;
    if (!audio) return;

    const time = parseFloat(e.target.value);
    audio.currentTime = time;
    setCurrentTime(time);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const audio = audioRef.current;
    if (!audio) return;

    const vol = parseFloat(e.target.value);
    audio.volume = vol;
    setVolume(vol);
  };

  const changePlaybackRate = (rate: number) => {
    const audio = audioRef.current;
    if (!audio) return;

    audio.playbackRate = rate;
    setPlaybackRate(rate);
  };

  const skip = (seconds: number) => {
    const audio = audioRef.current;
    if (!audio) return;

    audio.currentTime = Math.max(0, Math.min(duration || 0, audio.currentTime + seconds));
  };

  const formatTime = (seconds: number) => {
    if (!isFinite(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="audio-viewer">
      <audio ref={audioRef} src={audioUrl} preload="metadata" />

      <div className="audio-viewer-info">
        <span className="audio-viewer-filename" title={fileName}>{fileName}</span>
        {duration && <span className="audio-viewer-duration">{formatTime(duration)}</span>}
      </div>

      <div className="audio-viewer-waveform">
        <canvas ref={canvasRef} className="audio-viewer-canvas" />
      </div>

      <div className="audio-viewer-timeline">
        <span className="audio-viewer-time audio-viewer-time-current">{formatTime(currentTime)}</span>
        <input
          type="range"
          min={0}
          max={duration || 0}
          step={0.1}
          value={currentTime}
          onChange={handleSeek}
          className="audio-viewer-seek"
          disabled={!duration || loading}
        />
        <span className="audio-viewer-time audio-viewer-time-total">
          {duration ? formatTime(duration) : '-:--'}
        </span>
      </div>

      <div className="audio-viewer-controls">
        <button
          className="audio-viewer-btn"
          onClick={() => skip(-10)}
          title="Skip back 10 seconds"
          disabled={loading}
        >
          −10
        </button>
        <button
          className="audio-viewer-btn audio-viewer-btn-play"
          onClick={togglePlayback}
          title={isPlaying ? 'Pause' : 'Play'}
          disabled={loading}
        >
          {isPlaying ? '⏸' : '▶'}
        </button>
        <button
          className="audio-viewer-btn"
          onClick={() => skip(10)}
          title="Skip forward 10 seconds"
          disabled={loading}
        >
          +10
        </button>

        <div className="audio-viewer-separator" />

        <button
          className={`audio-viewer-btn${playbackRate === 0.5 ? ' audio-viewer-btn--active' : ''}`}
          onClick={() => changePlaybackRate(0.5)}
          title="0.5x speed"
          disabled={loading}
        >
          0.5×
        </button>
        <button
          className={`audio-viewer-btn${playbackRate === 1 ? ' audio-viewer-btn--active' : ''}`}
          onClick={() => changePlaybackRate(1)}
          title="Normal speed"
          disabled={loading}
        >
          1×
        </button>
        <button
          className={`audio-viewer-btn${playbackRate === 1.5 ? ' audio-viewer-btn--active' : ''}`}
          onClick={() => changePlaybackRate(1.5)}
          title="1.5x speed"
          disabled={loading}
        >
          1.5×
        </button>
        <button
          className={`audio-viewer-btn${playbackRate === 2 ? ' audio-viewer-btn--active' : ''}`}
          onClick={() => changePlaybackRate(2)}
          title="2x speed"
          disabled={loading}
        >
          2×
        </button>

        <div className="audio-viewer-separator" />

        <button
          className="audio-viewer-btn audio-viewer-btn-volume"
          onClick={() => {
            const audio = audioRef.current;
            if (!audio) return;
            audio.muted = !audio.muted;
          }}
          title={`Volume: ${Math.round(volume * 100)}%`}
          disabled={loading}
        >
          {volume === 0 ? '🔇' : volume < 0.5 ? '🔉' : '🔊'}
        </button>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={volume}
          onChange={handleVolumeChange}
          className="audio-viewer-volume-slider"
          disabled={loading}
          title="Volume"
        />

        <div className="audio-viewer-separator" />

        <a
          href={audioUrl}
          download={fileName}
          className="audio-viewer-btn audio-viewer-btn-download"
          title="Download audio"
        >
          ⬇
        </a>
      </div>

      {loading && (
        <div className="audio-viewer-status">Loading audio…</div>
      )}
      {error && (
        <div className="audio-viewer-status audio-viewer-status--error">
          Failed to load audio
          <a href={audioUrl} download={fileName} className="audio-viewer-download-fallback">
            Download {fileName}
          </a>
        </div>
      )}
    </div>
  );
}
