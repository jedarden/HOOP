/**
 * File context utility tests (§6 Phase 3)
 *
 * Verifies path-sensitive routing functionality:
 * - File tree drag-drop detection
 * - File context fetching with git blame
 * - Markdown formatting for file snippets
 * - Language detection for syntax highlighting
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  fetchFileContext,
  formatFileContextAsMarkdown,
  formatFileContextAsText,
  getFileTreeDropData,
  DEFAULT_FILE_CONTEXT_OPTIONS,
  type FileContext,
} from './fileContext';

// Mock fetch for API calls
const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

describe('fileContext utility (§6 Phase 3)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('getFileTreeDropData', () => {
    it('extracts file path and project name from valid drop event', () => {
      const mockEvent = {
        dataTransfer: {
          getData: vi.fn((type: string) => {
            if (type === 'application/hoop-file-path') return 'src/main.rs';
            if (type === 'application/hoop-project-name') return 'HOOP';
            return '';
          }),
        },
      } as unknown as React.DragEvent;

      const result = getFileTreeDropData(mockEvent);

      expect(result).not.toBeNull();
      expect(result?.path).toBe('src/main.rs');
      expect(result?.projectName).toBe('HOOP');
    });

    it('returns null when file path is missing', () => {
      const mockEvent = {
        dataTransfer: {
          getData: vi.fn((type: string) => {
            if (type === 'application/hoop-file-path') return '';
            if (type === 'application/hoop-project-name') return 'HOOP';
            return '';
          }),
        },
      } as unknown as React.DragEvent;

      const result = getFileTreeDropData(mockEvent);

      expect(result).toBeNull();
    });

    it('returns null when project name is missing', () => {
      const mockEvent = {
        dataTransfer: {
          getData: vi.fn((type: string) => {
            if (type === 'application/hoop-file-path') return 'src/main.rs';
            if (type === 'application/hoop-project-name') return '';
            return '';
          }),
        },
      } as unknown as React.DragEvent;

      const result = getFileTreeDropData(mockEvent);

      expect(result).toBeNull();
    });

    it('returns null when both are missing', () => {
      const mockEvent = {
        dataTransfer: {
          getData: vi.fn(() => ''),
        },
      } as unknown as React.DragEvent;

      const result = getFileTreeDropData(mockEvent);

      expect(result).toBeNull();
    });
  });

  describe('fetchFileContext', () => {
    const mockFileContent = 'line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12\nline13\nline14\nline15\nline16\nline17\nline18\nline19\nline20\nline21\nline22\nline23\nline24\nline25';
    const mockBlameData = [
      { sha: 'abc123def456789', author: 'Test Author', summary: 'Initial commit' },
    ];

    it('fetches file content with git blame', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve(mockFileContent),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockBlameData),
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs');

      expect(result.projectName).toBe('HOOP');
      expect(result.path).toBe('src/main.rs');
      expect(result.sha).toBe('abc123def456789');
      expect(result.lineCount).toBe(25);
      expect(result.content).toContain('line1');
      expect(result.content).toContain('... (5 more lines)');
    });

    it('respects maxLines option', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve(mockFileContent),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockBlameData),
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs', { maxLines: 10 });

      expect(result.content).toContain('line10');
      expect(result.content).toContain('... (15 more lines)');
      expect(result.content).not.toContain('line11');
    });

    it('includes line numbers when includeLineNumbers is true', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve('line1\nline2\nline3'),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockBlameData),
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs', { includeLineNumbers: true });

      expect(result.content).toContain('   1 | line1');
      expect(result.content).toContain('   2 | line2');
      expect(result.content).toContain('   3 | line3');
    });

    it('excludes line numbers when includeLineNumbers is false', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve('line1\nline2\nline3'),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockBlameData),
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs', { includeLineNumbers: false });

      expect(result.content).toBe('line1\nline2\nline3');
    });

    it('handles missing git blame gracefully', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve('line1\nline2\nline3'),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: false,
            status: 404,
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs');

      expect(result.sha).toBeNull();
      expect(result.content).toContain('line1');
    });

    it('throws error when file content fetch fails', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      } as Response);

      await expect(fetchFileContext('HOOP', 'src/main.rs')).rejects.toThrow('Failed to fetch file content: 500');
    });

    it('uses default options when none provided', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/files/content')) {
          return Promise.resolve({
            ok: true,
            text: () => Promise.resolve(mockFileContent),
          } as Response);
        }
        if (url.includes('/files/blame')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockBlameData),
          } as Response);
        }
        return Promise.resolve({
          ok: false,
          status: 404,
        } as Response);
      });

      const result = await fetchFileContext('HOOP', 'src/main.rs');

      expect(result.content).toContain('   1 |'); // Line numbers included by default
      expect(result.content).toContain('... (5 more lines)'); // Truncated to 20 lines by default
    });
  });

  describe('formatFileContextAsMarkdown', () => {
    it('formats Rust file with correct syntax hint', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'src/main.rs',
        sha: 'abc1234',
        content: '   1 | line1\n   2 | line2',
        lineCount: 2,
      };

      const result = formatFileContextAsMarkdown(context);

      expect(result).toContain('`HOOP/src/main.rs` (abc1234)');
      expect(result).toContain('```rust');
      expect(result).toContain('line1');
      expect(result).toContain('line2');
      expect(result).toContain('```');
    });

    it('formats TypeScript file with correct syntax hint', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'web/src/App.tsx',
        sha: null,
        content: 'line1',
        lineCount: 1,
      };

      const result = formatFileContextAsMarkdown(context);

      expect(result).toContain('`HOOP/web/src/App.tsx`');
      expect(result).toContain('```typescript');
      expect(result).not.toContain('(null)');
    });

    it('formats Python file with correct syntax hint', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'script.py',
        sha: 'def456',
        content: 'print("hello")',
        lineCount: 1,
      };

      const result = formatFileContextAsMarkdown(context);

      expect(result).toContain('```python');
    });

    it('handles unknown file extensions', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'data.xyz',
        sha: null,
        content: 'content',
        lineCount: 1,
      };

      const result = formatFileContextAsMarkdown(context);

      expect(result).toContain('```xyz');
    });

    it('includes total line count when truncated', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'src/main.rs',
        sha: null,
        content: 'line1\n... (100 more lines)',
        lineCount: 105,
      };

      const result = formatFileContextAsMarkdown(context);

      expect(result).toContain('(105 total lines)');
    });

    it('detects common file extensions correctly', () => {
      const extensions = [
        { ext: 'ts', expected: 'typescript' },
        { ext: 'tsx', expected: 'typescript' },
        { ext: 'js', expected: 'javascript' },
        { ext: 'jsx', expected: 'javascript' },
        { ext: 'rs', expected: 'rust' },
        { ext: 'py', expected: 'python' },
        { ext: 'go', expected: 'go' },
        { ext: 'java', expected: 'java' },
        { ext: 'cpp', expected: 'cpp' },
        { ext: 'c', expected: 'c' },
        { ext: 'yaml', expected: 'yaml' },
        { ext: 'json', expected: 'json' },
        { ext: 'md', expected: 'markdown' },
        { ext: 'sh', expected: 'bash' },
        { ext: 'sql', expected: 'sql' },
      ];

      extensions.forEach(({ ext, expected }) => {
        const context: FileContext = {
          projectName: 'HOOP',
          path: `file.${ext}`,
          sha: null,
          content: 'content',
          lineCount: 1,
        };

        const result = formatFileContextAsMarkdown(context);
        expect(result).toContain(`\`\`\`${expected}`);
      });
    });
  });

  describe('formatFileContextAsText', () => {
    it('formats as plain text with file header', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'src/main.rs',
        sha: 'abc1234',
        content: 'line1\nline2',
        lineCount: 2,
      };

      const result = formatFileContextAsText(context);

      expect(result).toContain('File: HOOP/src/main.rs (abc1234)');
      expect(result).toContain('line1');
      expect(result).toContain('line2');
    });

    it('omits SHA when null', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'src/main.rs',
        sha: null,
        content: 'line1',
        lineCount: 1,
      };

      const result = formatFileContextAsText(context);

      expect(result).toContain('File: HOOP/src/main.rs');
      expect(result).not.toContain('(null)');
    });

    it('omits total line count when not truncated', () => {
      const context: FileContext = {
        projectName: 'HOOP',
        path: 'src/main.rs',
        sha: null,
        content: 'line1\nline2',
        lineCount: 2,
      };

      const result = formatFileContextAsText(context);

      expect(result).not.toContain('more lines)');
    });
  });

  describe('DEFAULT_FILE_CONTEXT_OPTIONS', () => {
    it('has correct default values', () => {
      expect(DEFAULT_FILE_CONTEXT_OPTIONS.maxLines).toBe(20);
      expect(DEFAULT_FILE_CONTEXT_OPTIONS.includeLineNumbers).toBe(true);
    });
  });
});
