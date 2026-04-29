import { useCallback, useEffect, useRef, useState } from 'react';

export interface HexViewerProps {
  projectName: string;
  path: string;
}

interface HexRow {
  offset: number;
  hex: string;
  ascii: string;
}

interface HexResponse {
  offset: number;
  length: number;
  total_size: number;
  rows: HexRow[];
}

const ROWS_PER_PAGE = 256; // 4KB per page

export function HexViewer({ projectName, path }: HexViewerProps) {
  const [rows, setRows] = useState<HexRow[]>([]);
  const [totalSize, setTotalSize] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [jumpOffset, setJumpOffset] = useState('');
  const [copied, setCopied] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  // Load hex data at the given offset
  const loadHexData = useCallback(async (startOffset: number) => {
    setLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams({
        path,
        hex: 'true',
        offset: startOffset.toString(),
        length: (ROWS_PER_PAGE * 16).toString(),
      });

      const response = await fetch(
        `/api/projects/${encodeURIComponent(projectName)}/files/content?${params}`
      );

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const data: HexResponse = await response.json();

      if (startOffset === 0) {
        setTotalSize(data.total_size);
      }

      setRows(prev => {
        if (startOffset === 0) {
          return data.rows;
        }
        // Filter out duplicates and append new rows
        const existingOffsets = new Set(prev.map(r => r.offset));
        const newRows = data.rows.filter(r => !existingOffsets.has(r.offset));
        return [...prev, ...newRows];
      });

      return data.rows.length;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return 0;
    } finally {
      setLoading(false);
    }
  }, [projectName, path]);

  // Initial load
  useEffect(() => {
    setRows([]);
    setTotalSize(0);
    setError(null);
    loadHexData(0);
  }, [path, projectName, loadHexData]);

  // Infinite scroll using Intersection Observer
  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel) return;

    const observer = new IntersectionObserver(
      entries => {
        if (entries[0].isIntersecting && !loading && rows.length > 0) {
          const lastOffset = rows[rows.length - 1].offset;
          if (lastOffset + 16 * rows.length < totalSize) {
            loadHexData(lastOffset + 16 * rows.length);
          }
        }
      },
      { rootMargin: '200px' }
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loading, rows, totalSize, loadHexData]);

  // Jump to offset
  const handleJump = useCallback(() => {
    const offsetValue = parseInt(jumpOffset, jumpOffset.startsWith('0x') ? 16 : 10);
    if (isNaN(offsetValue) || offsetValue < 0) {
      setError('Invalid offset');
      return;
    }

    if (offsetValue >= totalSize) {
      setError(`Offset exceeds file size (${totalSize} bytes)`);
      return;
    }

    // Clear current rows and load from new offset
    setRows([]);
    setError(null);
    loadHexData(offsetValue);
  }, [jumpOffset, totalSize, loadHexData]);

  // Copy hex or ASCII
  const handleCopy = useCallback((text: string, type: string) => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(type);
      setTimeout(() => setCopied(null), 1500);
    });
  }, []);

  // Format offset for display
  const formatOffset = (o: number): string => {
    return `0x${o.toString(16).padStart(8, '0')}`;
  };

  // Check if ASCII character is printable
  const isPrintable = (char: string): boolean => {
    const code = char.charCodeAt(0);
    return code >= 32 && code <= 126;
  };

  const fileName = path.split('/').pop() ?? path;

  return (
    <div className="hex-viewer">
      <div className="hex-viewer-toolbar">
        <span className="hex-viewer-title">{fileName}</span>
        <span className="hex-viewer-size">{totalSize.toLocaleString()} bytes</span>
        <div className="hex-viewer-jump">
          <input
            type="text"
            className="hex-viewer-jump-input"
            placeholder="Offset (0x1000 or 4096)"
            value={jumpOffset}
            onChange={e => setJumpOffset(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter') handleJump();
            }}
          />
          <button
            className="hex-viewer-btn"
            onClick={handleJump}
            disabled={loading}
            title="Jump to offset"
          >
            Go
          </button>
        </div>
        <button
          className="hex-viewer-btn"
          onClick={() => {
            setRows([]);
            loadHexData(0);
          }}
          disabled={loading}
          title="Return to start"
        >
          Reset
        </button>
      </div>

      <div className="hex-viewer-header">
        <span className="hex-viewer-header-offset">Offset</span>
        <span className="hex-viewer-header-hex">Hex (16 bytes per row)</span>
        <span className="hex-viewer-header-ascii">ASCII</span>
      </div>

      <div ref={containerRef} className="hex-viewer-body">
        {error && (
          <div className="hex-viewer-error">{error}</div>
        )}

        {rows.length === 0 && !loading && !error && (
          <div className="hex-viewer-empty">No data</div>
        )}

        {rows.map(row => (
          <div key={row.offset} className="hex-viewer-row">
            <span className="hex-viewer-offset">{formatOffset(row.offset)}</span>
            <span
              className="hex-viewer-hex"
              onClick={() => handleCopy(row.hex, 'hex')}
              title="Click to copy hex"
            >
              {row.hex}
            </span>
            <span
              className="hex-viewer-ascii"
              onClick={() => handleCopy(row.ascii, 'ascii')}
              title="Click to copy ASCII"
            >
              {row.ascii.split('').map((char, i) => (
                <span
                  key={i}
                  className={isPrintable(char) ? 'hex-viewer-ascii-printable' : 'hex-viewer-ascii-nonprintable'}
                >
                  {char}
                </span>
              ))}
            </span>
          </div>
        ))}

        {loading && (
          <div className="hex-viewer-loading">Loading…</div>
        )}

        <div ref={sentinelRef} className="hex-viewer-sentinel" />
      </div>

      {copied && (
        <div className="hex-viewer-copied">
          Copied {copied}
        </div>
      )}
    </div>
  );
}
