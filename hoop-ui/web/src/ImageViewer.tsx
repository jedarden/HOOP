import { useCallback, useEffect, useRef, useState } from 'react';

export interface ImageViewerProps {
  projectName: string;
  path: string;
}

export function ImageViewer({ projectName, path }: ImageViewerProps) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState(false);

  const containerRef = useRef<HTMLDivElement>(null);
  const dragOriginRef = useRef<{ mouseX: number; mouseY: number; panX: number; panY: number } | null>(null);
  const didDragRef = useRef(false);

  const imageUrl = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(path)}&image=true`;
  const fileName = path.split('/').pop() ?? path;

  // Reset state when path changes
  useEffect(() => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
    setLoaded(false);
    setError(false);
  }, [path]);

  // Scroll-wheel zoom toward cursor position
  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    setZoom(currentZoom => {
      const newZoom = Math.max(0.1, Math.min(20, currentZoom * factor));
      const rect = containerRef.current?.getBoundingClientRect();
      if (rect) {
        const mx = e.clientX - rect.left - rect.width / 2;
        const my = e.clientY - rect.top - rect.height / 2;
        setPan(currentPan => ({
          x: mx - (mx - currentPan.x) * (newZoom / currentZoom),
          y: my - (my - currentPan.y) * (newZoom / currentZoom),
        }));
      }
      return newZoom;
    });
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    el.addEventListener('wheel', handleWheel, { passive: false });
    return () => el.removeEventListener('wheel', handleWheel);
  }, [handleWheel]);

  // Mouse-drag pan; click without drag (movement < 5px) opens fullscreen
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    setDragging(true);
    didDragRef.current = false;
    dragOriginRef.current = { mouseX: e.clientX, mouseY: e.clientY, panX: pan.x, panY: pan.y };
  };

  useEffect(() => {
    if (!dragging) return;
    const onMove = (e: MouseEvent) => {
      const origin = dragOriginRef.current;
      if (!origin) return;
      const dx = e.clientX - origin.mouseX;
      const dy = e.clientY - origin.mouseY;
      if (Math.hypot(dx, dy) > 4) didDragRef.current = true;
      setPan({ x: origin.panX + dx, y: origin.panY + dy });
    };
    const onUp = () => {
      setDragging(false);
      if (!didDragRef.current) setFullscreen(true);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
  }, [dragging]);

  // Pinch-to-zoom (touch)
  const lastPinchDist = useRef<number | null>(null);
  const handleTouchStart = (e: React.TouchEvent) => {
    if (e.touches.length === 2) {
      lastPinchDist.current = Math.hypot(
        e.touches[1].clientX - e.touches[0].clientX,
        e.touches[1].clientY - e.touches[0].clientY,
      );
    }
  };
  const handleTouchMove = (e: React.TouchEvent) => {
    if (e.touches.length !== 2 || lastPinchDist.current === null) return;
    e.preventDefault();
    const dist = Math.hypot(
      e.touches[1].clientX - e.touches[0].clientX,
      e.touches[1].clientY - e.touches[0].clientY,
    );
    const factor = dist / lastPinchDist.current;
    setZoom(z => Math.max(0.1, Math.min(20, z * factor)));
    lastPinchDist.current = dist;
  };
  const handleTouchEnd = () => { lastPinchDist.current = null; };

  const resetView = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const zoomIn = () => setZoom(z => Math.min(20, z * 1.25));
  const zoomOut = () => setZoom(z => Math.max(0.1, z / 1.25));

  const imgTransform = {
    transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
    transformOrigin: 'center center',
    // Disable EXIF auto-rotation so pan/zoom math stays consistent with
    // the raw pixel dimensions the server returns.
    imageOrientation: 'none' as const,
    cursor: dragging ? 'grabbing' : zoom > 1 ? 'grab' : 'pointer',
    transition: dragging ? 'none' : 'transform 0.05s ease-out',
    userSelect: 'none' as const,
  };

  return (
    <div className="image-viewer">
      <div className="image-viewer-toolbar">
        <button className="image-viewer-btn" onClick={zoomOut} title="Zoom out">−</button>
        <span className="image-viewer-zoom-label">{Math.round(zoom * 100)}%</span>
        <button className="image-viewer-btn" onClick={zoomIn} title="Zoom in">+</button>
        <button className="image-viewer-btn" onClick={resetView} title="Reset view (1:1)">1:1</button>
        <span className="image-viewer-sep" />
        <button className="image-viewer-btn image-viewer-btn--fullscreen" onClick={() => setFullscreen(true)} title="Full viewport">⤢</button>
      </div>

      <div
        ref={containerRef}
        className="image-viewer-canvas"
        onMouseDown={handleMouseDown}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        title="Click to open full-viewport · Scroll to zoom · Drag to pan"
      >
        {!loaded && !error && (
          <div className="image-viewer-status">Loading…</div>
        )}
        {error && (
          <div className="image-viewer-status image-viewer-status--error">Failed to load image</div>
        )}
        <img
          src={imageUrl}
          alt={fileName}
          style={{ ...imgTransform, display: loaded ? 'block' : 'none' }}
          draggable={false}
          onLoad={() => setLoaded(true)}
          onError={() => setError(true)}
        />
      </div>

      {fullscreen && (
        <div
          className="image-viewer-fullscreen"
          onClick={() => setFullscreen(false)}
          title="Click to close"
        >
          <img
            src={imageUrl}
            alt={fileName}
            className="image-viewer-fullscreen-img"
            draggable={false}
          />
        </div>
      )}
    </div>
  );
}
