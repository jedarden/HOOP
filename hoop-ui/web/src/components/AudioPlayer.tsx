import { useRef, useState, useEffect, useCallback } from 'react';
import { TranscriptData } from '../atoms';
import TranscriptView from './TranscriptView';

interface AudioPlayerProps {
  audioUrl: string;
  transcript?: TranscriptData;
}

export default function AudioPlayer({ audioUrl, transcript }: AudioPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackRate, setPlaybackRate] = useState(1);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handleLoadedMetadata = () => setDuration(audio.duration);
    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);
    const handleEnded = () => setIsPlaying(false);

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('loadedmetadata', handleLoadedMetadata);
    audio.addEventListener('play', handlePlay);
    audio.addEventListener('pause', handlePause);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('loadedmetadata', handleLoadedMetadata);
      audio.removeEventListener('play', handlePlay);
      audio.removeEventListener('pause', handlePause);
      audio.removeEventListener('ended', handleEnded);
    };
  }, []);

  const formatTime = (time: number): string => {
    if (isNaN(time)) return '0:00';
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const time = parseFloat(e.target.value);
    if (audioRef.current) {
      audioRef.current.currentTime = time;
      setCurrentTime(time);
    }
  };

  const togglePlayPause = () => {
    if (audioRef.current) {
      if (isPlaying) {
        audioRef.current.pause();
      } else {
        audioRef.current.play();
      }
    }
  };

  const skipBackward = () => {
    if (audioRef.current) {
      audioRef.current.currentTime = Math.max(0, audioRef.current.currentTime - 5);
    }
  };

  const skipForward = () => {
    if (audioRef.current) {
      audioRef.current.currentTime = Math.min(duration, audioRef.current.currentTime + 5);
    }
  };

  const cyclePlaybackRate = () => {
    const rates = [0.5, 0.75, 1, 1.25, 1.5, 2];
    const currentIndex = rates.indexOf(playbackRate);
    const nextRate = rates[(currentIndex + 1) % rates.length];
    setPlaybackRate(nextRate);
    if (audioRef.current) {
      audioRef.current.playbackRate = nextRate;
    }
  };

  const handleWordClick = useCallback((time: number) => {
    if (audioRef.current) {
      audioRef.current.currentTime = time;
      setCurrentTime(time);
      if (!isPlaying) {
        audioRef.current.play();
      }
    }
  }, [isPlaying]);

  return (
    <div className="audio-player">
      <audio ref={audioRef} src={audioUrl} preload="metadata" />

      <div className="audio-controls">
        <div className="audio-main-controls">
          <button
            className="audio-btn"
            onClick={skipBackward}
            aria-label="Skip backward 5 seconds"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M11.99 5V1l-5 5 5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6h-2c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8zm-1.1 11h-.2c-.63 0-1.11-.48-1.11-1.11v-1.02c0-.63.48-1.11 1.11-1.11h.2c.63 0 1.11.48 1.11 1.11v1.02c0 .63-.48 1.11-1.11 1.11zm0-4h-.2c-.63 0-1.11-.48-1.11-1.11v-1.02c0-.63.48-1.11 1.11-1.11h.2c.63 0 1.11.48 1.11 1.11v1.02c0 .63-.48 1.11-1.11 1.11z"/>
            </svg>
          </button>

          <button
            className="audio-btn audio-btn-play"
            onClick={togglePlayPause}
            aria-label={isPlaying ? 'Pause' : 'Play'}
          >
            {isPlaying ? (
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
              </svg>
            ) : (
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
            )}
          </button>

          <button
            className="audio-btn"
            onClick={skipForward}
            aria-label="Skip forward 5 seconds"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 13c0 4.97 4.03 9 9 9s9-4.03 9-9-4.03-9-9-9-9 4.03-9 9zm3.5 0c0-3.04 2.46-5.5 5.5-5.5v3l5.5-5.5-5.5-5.5v3c-4.97 0-9 4.03-9 9zm5.5 6h-.2c-.63 0-1.11-.48-1.11-1.11v-1.02c0-.63.48-1.11 1.11-1.11h.2c.63 0 1.11.48 1.11 1.11v1.02c0 .63-.48 1.11-1.11 1.11zm0-4h-.2c-.63 0-1.11-.48-1.11-1.11v-1.02c0-.63.48-1.11 1.11-1.11h.2c.63 0 1.11.48 1.11 1.11v1.02c0 .63-.48 1.11-1.11 1.11z"/>
            </svg>
          </button>
        </div>

        <div className="audio-time-display">
          <span className="audio-time">{formatTime(currentTime)}</span>
          <span className="audio-time-separator">/</span>
          <span className="audio-time audio-time-total">{formatTime(duration)}</span>
        </div>

        <button
          className="audio-btn audio-btn-rate"
          onClick={cyclePlaybackRate}
          aria-label="Playback rate"
        >
          {playbackRate}×
        </button>
      </div>

      <div className="audio-progress">
        <input
          type="range"
          min="0"
          max={duration || 0}
          step="0.01"
          value={currentTime}
          onChange={handleSeek}
          className="audio-progress-bar"
          aria-label="Seek"
        />
        <div
          className="audio-progress-fill"
          style={{ width: `${duration ? (currentTime / duration) * 100 : 0}%` }}
        />
      </div>

      {transcript && transcript.words.length > 0 && (
        <TranscriptView
          transcript={transcript}
          currentTime={currentTime}
          onWordClick={handleWordClick}
        />
      )}
    </div>
  );
}
