/**
 * File context utility for drag-and-drop from file tree.
 *
 * Fetches file content with git revision info and creates a formatted
 * snippet that can be inserted into agent chat or stitch drafts.
 */

export interface FileContext {
  projectName: string;
  path: string;
  sha: string | null;
  content: string;
  lineCount: number;
}

export interface FileContextOptions {
  /** Maximum number of lines to include from the file (default: 20) */
  maxLines?: number;
  /** Include line numbers (default: true) */
  includeLineNumbers?: boolean;
}

/**
 * Fetch file content with git blame info for the most recent SHA.
 *
 * @param projectName - Project name
 * @param filePath - Relative file path within the project
 * @param options - Configuration options
 * @returns File context with path, SHA, and content snippet
 */
export async function fetchFileContext(
  projectName: string,
  filePath: string,
  options: FileContextOptions = {},
): Promise<FileContext> {
  const { maxLines = 20, includeLineNumbers = true } = options;

  // Fetch raw file content
  const contentUrl = `/api/projects/${encodeURIComponent(projectName)}/files/content?path=${encodeURIComponent(filePath)}&raw=true`;
  const contentResponse = await fetch(contentUrl);

  if (!contentResponse.ok) {
    throw new Error(`Failed to fetch file content: ${contentResponse.status}`);
  }

  const content = await contentResponse.text();

  // Fetch git blame to get the most recent SHA
  let sha: string | null = null;
  try {
    const blameUrl = `/api/projects/${encodeURIComponent(projectName)}/files/blame?path=${encodeURIComponent(filePath)}`;
    const blameResponse = await fetch(blameUrl);

    if (blameResponse.ok) {
      const blameLines = await blameResponse.json();
      if (Array.isArray(blameLines) && blameLines.length > 0) {
        // Get the SHA from the first line (most recent change at the top)
        sha = blameLines[0]?.sha || null;
      }
    }
  } catch (e) {
    // Blame fetch failure is not critical - proceed without SHA
    console.warn('Failed to fetch git blame:', e);
  }

  // Split into lines and truncate to maxLines
  const lines = content.split('\n');
  const truncatedLines = lines.slice(0, maxLines);
  const isTruncated = lines.length > maxLines;

  // Build the snippet content
  let snippetContent: string;
  if (includeLineNumbers) {
    snippetContent = truncatedLines
      .map((line, i) => `${String(i + 1).padStart(4, ' ')} | ${line}`)
      .join('\n');
    if (isTruncated) {
      snippetContent += `\n... (${lines.length - maxLines} more lines)`;
    }
  } else {
    snippetContent = truncatedLines.join('\n');
    if (isTruncated) {
      snippetContent += `\n\n... (${lines.length - maxLines} more lines)`;
    }
  }

  return {
    projectName,
    path: filePath,
    sha,
    content: snippetContent,
    lineCount: lines.length,
  };
}

/**
 * Format file context as a markdown code block for insertion into text areas.
 *
 * @param context - File context from fetchFileContext
 * @returns Formatted markdown string
 */
export function formatFileContextAsMarkdown(context: FileContext): string {
  const { projectName, path, sha, content, lineCount } = context;

  // Detect file extension for syntax hint
  const ext = path.split('.').pop()?.toLowerCase() || 'txt';
  const langMap: Record<string, string> = {
    ts: 'typescript',
    tsx: 'typescript',
    js: 'javascript',
    jsx: 'javascript',
    rs: 'rust',
    py: 'python',
    go: 'go',
    java: 'java',
    cpp: 'cpp',
    c: 'c',
    h: 'c',
    cs: 'csharp',
    php: 'php',
    rb: 'ruby',
    sh: 'bash',
    yaml: 'yaml',
    yml: 'yaml',
    json: 'json',
    toml: 'toml',
    md: 'markdown',
    css: 'css',
    scss: 'scss',
    html: 'html',
    xml: 'xml',
    sql: 'sql',
  };
  const lang = langMap[ext] || ext;

  const shaSuffix = sha ? ` (${sha.slice(0, 7)})` : '';
  const totalLinesNote = content !== content.split('\n').slice(0, 20).join('\n')
    ? `\n... (${lineCount} total lines)`
    : '';

  return `📎 \`${projectName}/${path}\`${shaSuffix}

\`\`\`${lang}
${content}
${totalLinesNote ? `\n${totalLinesNote}` : ''}
\`\`\`
`;
}

/**
 * Format file context as plain text for insertion into text areas.
 *
 * @param context - File context from fetchFileContext
 * @returns Formatted plain text string
 */
export function formatFileContextAsText(context: FileContext): string {
  const { projectName, path, sha, content, lineCount } = context;

  const shaSuffix = sha ? ` (${sha.slice(0, 7)})` : '';
  const totalLinesNote = content.includes('more lines')
    ? ''
    : `\n... (${lineCount} total lines)`;

  return `File: ${projectName}/${path}${shaSuffix}
${content}
${totalLinesNote}
`;
}

/**
 * Check if a drag event contains file tree data from FilesTab.
 *
 * @param e - Drag event
 * @returns Object with projectName and path if valid, null otherwise
 */
export function getFileTreeDropData(e: React.DragEvent): { projectName: string; path: string } | null {
  const filePath = e.dataTransfer.getData('application/hoop-file-path');
  const projectName = e.dataTransfer.getData('application/hoop-project-name');

  if (filePath && projectName) {
    return { projectName, path: filePath };
  }

  return null;
}

/**
 * Default options for file context fetch.
 */
export const DEFAULT_FILE_CONTEXT_OPTIONS: FileContextOptions = {
  maxLines: 20,
  includeLineNumbers: true,
};
