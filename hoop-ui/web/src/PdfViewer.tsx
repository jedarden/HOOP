import { useCallback, useEffect, useRef, useState } from 'react';
import * as pdfjsLib from 'pdfjs-dist';

// Set worker source
pdfjsLib.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.mjs`;

export interface PdfViewerProps {
  projectName: string;
  path: string;
}

export function PdfViewer({ projectName, path }: PdfViewerProps) {
  const [pdf, setPdf] = useState<pdfjsLib.PDFDocumentProxy | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(0);
  const [scale, setScale] = useState(1.0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchMatches, setSearchMatches] = useState<number[]>([]);
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const renderTaskRef = useRef<pdfjsLib.RenderTask | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const pdfUrl = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(path)}&raw=true`;
  const fileName = path.split('/').pop() ?? path;

  // Load PDF document
  useEffect(() => {
    let mounted = true;

    const loadPdf = async () => {
      setLoading(true);
      setError(null);
      try {
        const loadingTask = pdfjsLib.getDocument(pdfUrl);
        const pdfDoc = await loadingTask.promise;
        if (!mounted) return;
        setPdf(pdfDoc);
        setTotalPages(pdfDoc.numPages);
        setCurrentPage(1);
      } catch (err) {
        if (!mounted) return;
        setError(err instanceof Error ? err.message : 'Failed to load PDF');
      } finally {
        if (mounted) setLoading(false);
      }
    };

    loadPdf();

    return () => {
      mounted = false;
      renderTaskRef.current?.cancel();
    };
  }, [pdfUrl]);

  // Render current page
  useEffect(() => {
    if (!pdf || currentPage < 1 || currentPage > totalPages) return;

    let mounted = true;

    const renderPage = async () => {
      try {
        const page = await pdf.getPage(currentPage);
        const viewport = page.getViewport({ scale });

        const canvas = canvasRef.current;
        if (!canvas || !mounted) return;

        const context = canvas.getContext('2d');
        if (!context) return;

        canvas.height = viewport.height;
        canvas.width = viewport.width;

        // Cancel any ongoing render
        renderTaskRef.current?.cancel();

        const renderTask = page.render({
          canvasContext: context,
          viewport: viewport,
        });

        renderTaskRef.current = renderTask;
        await renderTask.promise;

        if (!mounted) return;
      } catch (err) {
        if (err instanceof Error && err.name === 'RenderingCancelledException') {
          return;
        }
        if (mounted) {
          setError(err instanceof Error ? err.message : 'Failed to render page');
        }
      }
    };

    renderPage();

    return () => {
      mounted = false;
      renderTaskRef.current?.cancel();
    };
  }, [pdf, currentPage, scale, totalPages]);

  // Search functionality
  const performSearch = useCallback(async () => {
    if (!pdf || !searchQuery.trim()) {
      setSearchMatches([]);
      setCurrentMatchIndex(0);
      return;
    }

    const matches: number[] = [];
    for (let i = 1; i <= totalPages; i++) {
      try {
        const page = await pdf.getPage(i);
        const textContent = await page.getTextContent();
        const pageText = textContent.items.map(item => ('str' in item ? item.str : '')).join(' ');

        if (pageText.toLowerCase().includes(searchQuery.toLowerCase())) {
          matches.push(i);
        }
      } catch {
        // Skip pages that fail to load
      }
    }

    setSearchMatches(matches);
    setCurrentMatchIndex(matches.length > 0 ? 0 : -1);
  }, [pdf, searchQuery, totalPages]);

  useEffect(() => {
    const debounceTimer = setTimeout(performSearch, 300);
    return () => clearTimeout(debounceTimer);
  }, [performSearch]);

  const goToMatch = (direction: 'next' | 'prev') => {
    if (searchMatches.length === 0) return;

    if (direction === 'next') {
      const nextIndex = (currentMatchIndex + 1) % searchMatches.length;
      setCurrentMatchIndex(nextIndex);
      setCurrentPage(searchMatches[nextIndex]);
    } else {
      const prevIndex = (currentMatchIndex - 1 + searchMatches.length) % searchMatches.length;
      setCurrentMatchIndex(prevIndex);
      setCurrentPage(searchMatches[prevIndex]);
    }
  };

  const zoomIn = () => setScale(s => Math.min(3.0, s + 0.25));
  const zoomOut = () => setScale(s => Math.max(0.5, s - 0.25));
  const resetZoom = () => setScale(1.0);

  const goToPrevPage = () => setCurrentPage(p => Math.max(1, p - 1));
  const goToNextPage = () => setCurrentPage(p => Math.min(totalPages, p + 1));

  return (
    <div className="pdf-viewer">
      <div className="pdf-viewer-toolbar">
        <div className="pdf-viewer-toolbar-group">
          <button className="pdf-viewer-btn" onClick={goToPrevPage} disabled={currentPage <= 1} title="Previous page">←</button>
          <span className="pdf-viewer-page-label">
            Page <input
              type="number"
              min={1}
              max={totalPages}
              value={currentPage}
              onChange={e => setCurrentPage(Math.min(totalPages, Math.max(1, parseInt(e.target.value) || 1)))}
              className="pdf-viewer-page-input"
            /> / {totalPages}
          </span>
          <button className="pdf-viewer-btn" onClick={goToNextPage} disabled={currentPage >= totalPages} title="Next page">→</button>
        </div>

        <div className="pdf-viewer-toolbar-sep" />

        <div className="pdf-viewer-toolbar-group">
          <button className="pdf-viewer-btn" onClick={zoomOut} disabled={scale <= 0.5} title="Zoom out">−</button>
          <span className="pdf-viewer-zoom-label">{Math.round(scale * 100)}%</span>
          <button className="pdf-viewer-btn" onClick={zoomIn} disabled={scale >= 3.0} title="Zoom in">+</button>
          <button className="pdf-viewer-btn" onClick={resetZoom} title="Reset zoom">1:1</button>
        </div>

        <div className="pdf-viewer-toolbar-sep" />

        <div className="pdf-viewer-toolbar-group pdf-viewer-search">
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search in PDF…"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            className="pdf-viewer-search-input"
          />
          {searchMatches.length > 0 && (
            <span className="pdf-viewer-search-matches">
              {currentMatchIndex + 1} / {searchMatches.length}
            </span>
          )}
          {searchMatches.length > 1 && (
            <>
              <button className="pdf-viewer-btn pdf-viewer-search-btn" onClick={() => goToMatch('prev')} title="Previous match">↑</button>
              <button className="pdf-viewer-btn pdf-viewer-search-btn" onClick={() => goToMatch('next')} title="Next match">↓</button>
            </>
          )}
        </div>

        <div className="pdf-viewer-toolbar-sep" />

        <a href={pdfUrl} download={fileName} className="pdf-viewer-btn pdf-viewer-download-btn" title="Download PDF">
          ⬇
        </a>
      </div>

      <div className="pdf-viewer-canvas-container">
        {loading && (
          <div className="pdf-viewer-status">Loading PDF…</div>
        )}
        {error && (
          <div className="pdf-viewer-status pdf-viewer-status--error">
            Failed to load PDF: {error}
            <div className="pdf-viewer-download-fallback">
              <a href={pdfUrl} download={fileName}>Download {fileName}</a>
            </div>
          </div>
        )}
        {!loading && !error && (
          <div className="pdf-viewer-canvas-wrapper" style={{ overflow: 'auto' }}>
            <canvas ref={canvasRef} className="pdf-viewer-canvas" />
          </div>
        )}
      </div>
    </div>
  );
}
