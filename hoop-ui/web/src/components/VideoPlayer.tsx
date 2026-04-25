import { useRef, useState, useEffect, useCallback } from 'react';
import { FrameSample, TranscriptWord } from '../atoms';
import TranscriptView from './TranscriptView';

interface VideoPlayerProps {
  videoUrl: string;
  chapters?: FrameSample[];
  transcript?: { text: string; words: TranscriptWord[] };
}

function formatTime(time: number): string {
  if (isNaN(time) || !isFinite(time)) return '0:00';
  const h = Math.floor(time / 3600);
  const m = Math.floor((time % 3600) / 60);
  const s = Math.floor(time % 60);
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export default function VideoPlayer({ videoUrl, chapters = [], transcript }: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [activeChapterIdx, setActiveChapterIdx] = useState(-1);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const handleTimeUpdate = () => {
      setCurrentTime(video.currentTime);
      // Find active chapter: last chapter whose timestamp <= currentTime
      let idx = -1;
      for (let i = 0; i < chapters.length; i++) {
        if (chapters[i].timestamp_secs <= video.currentTime) idx = i;
      }
      setActiveChapterIdx(idx);
    };
    const handleLoadedMetadata = () => setDuration(video.duration);
    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);
    const handleEnded = () => setIsPlaying(false);

    video.addEventListener('timeupdate', handleTimeUpdate);
    video.addEventListener('loadedmetadata', handleLoadedMetadata);
    video.addEventListener('play', handlePlay);
    video.addEventListener('pause', handlePause);
    video.addEventListener('ended', handleEnded);

    return () => {
      video.removeEventListener('timeupdate', handleTimeUpdate);
      video.removeEventListener('loadedmetadata', handleLoadedMetadata);
      video.removeEventListener('play', handlePlay);
      video.removeEventListener('pause', handlePause);
      video.removeEventListener('ended', handleEnded);
    };
  }, [chapters]);

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const time = parseFloat(e.target.value);
    if (videoRef.current) {
      videoRef.current.currentTime = time;
      setCurrentTime(time);
    }
  };

  const togglePlayPause = () => {
    if (!videoRef.current) return;
    if (isPlaying) videoRef.current.pause();
    else videoRef.current.play();
  };

  const skipBackward = () => {
    if (videoRef.current) videoRef.current.currentTime = Math.max(0, videoRef.current.currentTime - 5);
  };

  const skipForward = () => {
    if (videoRef.current) videoRef.current.currentTime = Math.min(duration, videoRef.current.currentTime + 10);
  };

  const cyclePlaybackRate = () => {
    const rates = [0.5, 0.75, 1, 1.25, 1.5, 2];
    const next = rates[(rates.indexOf(playbackRate) + 1) % rates.length];
    setPlaybackRate(next);
    if (videoRef.current) videoRef.current.playbackRate = next;
  };

  const seekToChapter = useCallback((ts: number) => {
    if (!videoRef.current) return;
    videoRef.current.currentTime = ts;
    setCurrentTime(ts);
    if (!isPlaying) videoRef.current.play();
  }, [isPlaying]);

  const handleWordClick = useCallback((time: number) => {
    if (!videoRef.current) return;
    videoRef.current.currentTime = time;
    setCurrentTime(time);
    if (!isPlaying) videoRef.current.play();
  }, [isPlaying]);

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  return (
    <div className="video-player">
      <video
        ref={videoRef}
        src={videoUrl}
        preload="metadata"
        className="video-player-element"
        onClick={togglePlayPause}
      />

      {/* Chapter markers bar */}
      {chapters.length > 0 && (
        <div className="video-chapters-bar">
          <span className="video-chapters-label">Chapters</span>
          <div className="video-chapters-list">
            {chapters.map((ch, i) => (
              <button
                key={i}
                className={`video-chapter-btn${i === activeChapterIdx ? ' video-chapter-btn--active' : ''}`}
                onClick={() => seekToChapter(ch.timestamp_secs)}
                title={`${formatTime(ch.timestamp_secs)} — ${ch.label}`}
              >
                <span className="video-chapter-time">{formatTime(ch.timestamp_secs)}</span>
                <span className="video-chapter-label">{ch.label}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Controls */}
      <div className="audio-controls">
        <div className="audio-main-controls">
          <button className="audio-btn" onClick={skipBackward} aria-label="Back 5s">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 5V1l-5 5 5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z"/>
              <text x="9" y="15" fontSize="7" fill="currentColor">5</text>
            </svg>
          </button>

          <button className="audio-btn audio-btn-play" onClick={togglePlayPause} aria-label={isPlaying ? 'Pause' : 'Play'}>
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

          <button className="audio-btn" onClick={skipForward} aria-label="Forward 10s">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 13c0 4.97 4.03 9 9 9s9-4.03 9-9-4.03-9-9-9v3L7 2 12 7V4c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6z"/>
              <text x="8" y="15" fontSize="6" fill="currentColor">10</text>
            </svg>
          </button>
        </div>

        <div className="audio-time-display">
          <span className="audio-time">{formatTime(currentTime)}</span>
          <span className="audio-time-separator">/</span>
          <span className="audio-time audio-time-total">{formatTime(duration)}</span>
        </div>

        <button className="audio-btn audio-btn-rate" onClick={cyclePlaybackRate} aria-label="Playback rate">
          {playbackRate}×
        </button>
      </div>

      {/* Seek bar with chapter tick marks */}
      <div className="audio-progress video-progress">
        <input
          type="range"
          min="0"
          max={duration || 0}
          step="0.1"
          value={currentTime}
          onChange={handleSeek}
          className="audio-progress-bar"
          aria-label="Seek"
        />
        <div className="audio-progress-fill" style={{ width: `${progress}%` }} />
        {/* Chapter ticks overlaid on the progress bar */}
        {duration > 0 && chapters.map((ch, i) => (
          <div
            key={i}
            className={`video-chapter-tick${i === activeChapterIdx ? ' video-chapter-tick--active' : ''}`}
            style={{ left: `${(ch.timestamp_secs / duration) * 100}%` }}
            title={`${formatTime(ch.timestamp_secs)}: ${ch.label}`}
            onClick={() => seekToChapter(ch.timestamp_secs)}
          />
        ))}
      </div>

      {/* Transcript overlay */}
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
