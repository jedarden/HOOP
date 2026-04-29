import { useEffect, useRef, useState } from 'react';

export interface VideoViewerProps {
  projectName: string;
  path: string;
}

export function VideoViewer({ projectName, path }: VideoViewerProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [duration, setDuration] = useState<number | null>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [volume, setVolume] = useState(1);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isPiP, setIsPiP] = useState(false);

  const videoRef = useRef<HTMLVideoElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const videoUrl = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(path)}&raw=true`;
  const fileName = path.split('/').pop() ?? path;

  // Handle video events
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const handleLoadStart = () => setLoading(true);
    const handleCanPlay = () => setLoading(false);
    const handleError = () => {
      setLoading(false);
      setError(true);
    };
    const handleLoadedMetadata = () => {
      setDuration(video.duration);
    };
    const handleTimeUpdate = () => setCurrentTime(video.currentTime);
    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);
    const handleEnded = () => {
      setIsPlaying(false);
      setCurrentTime(0);
    };
    const handleRateChange = () => setPlaybackRate(video.playbackRate);
    const handleEnterFullscreen = () => setIsFullscreen(true);
    const handleExitFullscreen = () => setIsFullscreen(false);
    const handleEnterPiP = () => setIsPiP(true);
    const handleLeavePiP = () => setIsPiP(false);

    video.addEventListener('loadstart', handleLoadStart);
    video.addEventListener('canplay', handleCanPlay);
    video.addEventListener('error', handleError);
    video.addEventListener('loadedmetadata', handleLoadedMetadata);
    video.addEventListener('timeupdate', handleTimeUpdate);
    video.addEventListener('play', handlePlay);
    video.addEventListener('pause', handlePause);
    video.addEventListener('ended', handleEnded);
    video.addEventListener('ratechange', handleRateChange);

    document.addEventListener('fullscreenchange', () => {
      setIsFullscreen(!!document.fullscreenElement);
    });
    video.addEventListener('enterpictureinpicture', handleEnterPiP);
    video.addEventListener('leavepictureinpicture', handleLeavePiP);

    return () => {
      video.removeEventListener('loadstart', handleLoadStart);
      video.removeEventListener('canplay', handleCanPlay);
      video.removeEventListener('error', handleError);
      video.removeEventListener('loadedmetadata', handleLoadedMetadata);
      video.removeEventListener('timeupdate', handleTimeUpdate);
      video.removeEventListener('play', handlePlay);
      video.removeEventListener('pause', handlePause);
      video.removeEventListener('ended', handleEnded);
      video.removeEventListener('ratechange', handleRateChange);
      video.removeEventListener('enterpictureinpicture', handleEnterPiP);
      video.removeEventListener('leavepictureinpicture', handleLeavePiP);
    };
  }, [videoUrl]);

  // Playback controls
  const togglePlayback = () => {
    const video = videoRef.current;
    if (!video) return;

    if (isPlaying) {
      video.pause();
    } else {
      video.play();
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;

    const time = parseFloat(e.target.value);
    video.currentTime = time;
    setCurrentTime(time);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const video = videoRef.current;
    if (!video) return;

    const vol = parseFloat(e.target.value);
    video.volume = vol;
    setVolume(vol);
    if (video.muted && vol > 0) {
      video.muted = false;
    }
  };

  const toggleMute = () => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !video.muted;
  };

  const changePlaybackRate = (rate: number) => {
    const video = videoRef.current;
    if (!video) return;

    video.playbackRate = rate;
    setPlaybackRate(rate);
  };

  const skip = (seconds: number) => {
    const video = videoRef.current;
    if (!video) return;

    video.currentTime = Math.max(0, Math.min(duration || 0, video.currentTime + seconds));
  };

  const toggleFullscreen = async () => {
    const container = containerRef.current;
    if (!container) return;

    if (!document.fullscreenElement) {
      await container.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  };

  const togglePiP = async () => {
    const video = videoRef.current;
    if (!video) return;

    if (document.pictureInPictureElement) {
      await document.exitPictureInPicture();
    } else if (document.pictureInPictureEnabled) {
      await video.requestPictureInPicture();
    }
  };

  const formatTime = (seconds: number) => {
    if (!isFinite(seconds)) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div ref={containerRef} className={`video-viewer${isFullscreen ? ' video-viewer--fullscreen' : ''}`}>
      <div className="video-viewer-container">
        <video
          ref={videoRef}
          src={videoUrl}
          preload="metadata"
          controls={false}
          className="video-viewer-element"
        />

        {loading && (
          <div className="video-viewer-status video-viewer-status--loading">
            <div className="video-viewer-spinner" />
            Loading video…
          </div>
        )}
        {error && (
          <div className="video-viewer-status video-viewer-status--error">
            Failed to load video
            <a href={videoUrl} download={fileName} className="video-viewer-download-fallback">
              Download {fileName}
            </a>
          </div>
        )}
      </div>

      <div className="video-viewer-toolbar">
        <div className="video-viewer-info">
          <span className="video-viewer-filename" title={fileName}>{fileName}</span>
        </div>

        <div className="video-viewer-timeline">
          <span className="video-viewer-time video-viewer-time-current">{formatTime(currentTime)}</span>
          <input
            type="range"
            min={0}
            max={duration || 0}
            step={0.1}
            value={currentTime}
            onChange={handleSeek}
            className="video-viewer-seek"
            disabled={!duration || loading}
          />
          <span className="video-viewer-time video-viewer-time-total">
            {duration ? formatTime(duration) : '-:--'}
          </span>
        </div>

        <div className="video-viewer-controls">
          <button
            className="video-viewer-btn"
            onClick={() => skip(-10)}
            title="Skip back 10 seconds"
            disabled={loading}
          >
            −10
          </button>
          <button
            className="video-viewer-btn"
            onClick={() => skip(-5)}
            title="Skip back 5 seconds"
            disabled={loading}
          >
            −5
          </button>
          <button
            className="video-viewer-btn video-viewer-btn-play"
            onClick={togglePlayback}
            title={isPlaying ? 'Pause (Space)' : 'Play (Space)'}
            disabled={loading}
          >
            {isPlaying ? '⏸' : '▶'}
          </button>
          <button
            className="video-viewer-btn"
            onClick={() => skip(5)}
            title="Skip forward 5 seconds"
            disabled={loading}
          >
            +5
          </button>
          <button
            className="video-viewer-btn"
            onClick={() => skip(10)}
            title="Skip forward 10 seconds"
            disabled={loading}
          >
            +10
          </button>

          <div className="video-viewer-separator" />

          <button
            className="video-viewer-btn video-viewer-btn-volume"
            onClick={toggleMute}
            title={volume === 0 ? 'Unmute' : 'Mute'}
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
            className="video-viewer-volume-slider"
            disabled={loading}
            title="Volume"
          />

          <div className="video-viewer-separator" />

          <button
            className={`video-viewer-btn${playbackRate === 0.5 ? ' video-viewer-btn--active' : ''}`}
            onClick={() => changePlaybackRate(0.5)}
            title="0.5x speed"
            disabled={loading}
          >
            0.5×
          </button>
          <button
            className={`video-viewer-btn${playbackRate === 1 ? ' video-viewer-btn--active' : ''}`}
            onClick={() => changePlaybackRate(1)}
            title="Normal speed"
            disabled={loading}
          >
            1×
          </button>
          <button
            className={`video-viewer-btn${playbackRate === 1.5 ? ' video-viewer-btn--active' : ''}`}
            onClick={() => changePlaybackRate(1.5)}
            title="1.5x speed"
            disabled={loading}
          >
            1.5×
          </button>
          <button
            className={`video-viewer-btn${playbackRate === 2 ? ' video-viewer-btn--active' : ''}`}
            onClick={() => changePlaybackRate(2)}
            title="2x speed"
            disabled={loading}
          >
            2×
          </button>

          <div className="video-viewer-separator" />

          <button
            className="video-viewer-btn"
            onClick={togglePiP}
            title="Picture-in-picture"
            disabled={loading || !document.pictureInPictureEnabled}
          >
            📺
          </button>
          <button
            className="video-viewer-btn"
            onClick={toggleFullscreen}
            title="Fullscreen"
            disabled={loading}
          >
            {isFullscreen ? '⛶' : '⛶'}
          </button>

          <div className="video-viewer-separator" />

          <a
            href={videoUrl}
            download={fileName}
            className="video-viewer-btn video-viewer-btn-download"
            title="Download video"
          >
            ⬇
          </a>
        </div>
      </div>

      {/* Keyboard shortcuts */}
      {isPlaying && (
        <div style={{ display: 'none' }}>
          {/* Space: play/pause */}
          {/* Arrow left/right: skip 5s */}
          {/* Shift + arrow: skip 10s */}
          {/* F: fullscreen */}
          {/* M: mute */}
        </div>
      )}
    </div>
  );
}
