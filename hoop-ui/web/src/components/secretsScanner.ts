/**
 * Secrets Scanner for dictated notes (§18)
 *
 * Scans transcript text for potential secrets and sensitive information.
 * Uses regex patterns fetched from the backend to ensure parity with
 * the authoritative server-side scanner.
 */

export interface SecretMatch {
  type: string; // Pattern id (e.g., "anthropic_api_key")
  value: string;
  startIndex: number;
  endIndex: number;
}

export interface SecretWarning {
  matches: SecretMatch[];
  count: number;
}

// Secret pattern from backend API
export interface SecretPattern {
  id: string;
  name: string;
  severity: string;
  patterns: string[];
}

// Response from /api/config/secrets-patterns
interface SecretsPatternsResponse {
  schema_version: string;
  patterns: SecretPattern[];
}

// ─── Pattern cache ─────────────────────────────────────────────────────────────

let cachedPatterns: SecretPattern[] | null = null;
let fetchPromise: Promise<SecretPattern[]> | null = null;

/**
 * Convert Rust regex pattern to JavaScript-compatible pattern.
 *
 * Rust regex uses inline flags like (?i) for case-insensitive matching,
 * which JavaScript doesn't support. This function strips those flags
 * since the client always adds the 'i' flag to the RegExp constructor.
 *
 * @param rustPattern - Pattern string from backend (may contain (?i), (?-i), etc.)
 * @returns JavaScript-compatible pattern string
 */
function rustPatternToJs(rustPattern: string): string {
  // Remove inline flag groups: (?i), (?-i), (?s), (?-s), (?m), (?-m), etc.
  // Also remove non-capturing groups with just flags: (?i:something)
  return rustPattern.replace(/\(\?[-a-z]+\)/g, '').replace(/\(\?[-a-z]+:/g, '(?:');
}

/**
 * Clear the cached patterns. For testing only.
 */
export function clearPatternCache(): void {
  cachedPatterns = null;
  fetchPromise = null;
}

/**
 * Fetch secret patterns from the backend API (§18).
 *
 * This function caches the result and ensures only one fetch request
 * is in flight at a time. Patterns are served from config.yml or
 * backend defaults (single source of truth).
 *
 * @throws Error if the backend is unavailable (no local fallback)
 */
export async function prefetchSecretPatterns(): Promise<SecretPattern[]> {
  // Return cached patterns if available
  if (cachedPatterns) {
    return cachedPatterns;
  }

  // Return existing fetch promise if fetch is in progress
  if (fetchPromise) {
    return fetchPromise;
  }

  // Start fetching patterns
  fetchPromise = (async () => {
    try {
      const response = await fetch('/api/config/secrets-patterns');
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const data: SecretsPatternsResponse = await response.json();
      cachedPatterns = data.patterns;
      console.log(`[secretsScanner] Loaded ${data.patterns.length} patterns from backend`);
      return data.patterns;
    } catch (e) {
      console.error('[secretsScanner] Failed to fetch patterns from backend:', e);
      throw new Error('Secrets scanner requires backend connection. Please ensure the daemon is running.');
    } finally {
      fetchPromise = null;
    }
  })();

  return fetchPromise;
}

/**
 * Get cached patterns synchronously.
 *
 * Returns cached patterns if available, otherwise returns an empty array.
 * Use prefetchSecretPatterns() to ensure patterns are loaded from the
 * backend before calling this.
 */
function getPatternsSync(): SecretPattern[] {
  if (cachedPatterns) {
    return cachedPatterns;
  }

  // Patterns not loaded yet - return empty array for fail-safe behavior
  return [];
}

/**
 * Scan text for secrets using cached patterns (synchronous).
 *
 * This function is optimized for use in React render cycles where
 * async operations are not allowed. It uses patterns that were
 * pre-fetched via prefetchSecretPatterns(). If patterns haven't
 * been loaded yet, returns an empty result (fail-safe).
 *
 * IMPORTANT: Always call prefetchSecretPatterns() before using this
 * function, otherwise it will return an empty result.
 *
 * @param text - The text to scan for secrets
 * @returns SecretWarning with matches and count
 */
export function scanForSecretsSync(text: string): SecretWarning {
  const patterns = getPatternsSync();
  if (patterns.length === 0) {
    // Patterns not loaded yet - fail-safe, return empty result
    console.warn('[secretsScanner] scanForSecretsSync: No patterns loaded');
    return { matches: [], count: 0 };
  }

  const matches: SecretMatch[] = [];

  for (const secretType of patterns) {
    for (const patternStr of secretType.patterns) {
      try {
        const jsPattern = rustPatternToJs(patternStr);
        const pattern = new RegExp(jsPattern, 'gi');
        let match;

        // Reset regex state
        pattern.lastIndex = 0;

        while ((match = pattern.exec(text)) !== null) {
          // For patterns with capture groups, use the captured value;
          // otherwise use the full match
          const value = (match as any)[1] || match[0];
          matches.push({
            type: secretType.name,
            value,
            startIndex: match.index,
            endIndex: match.index + value.length,
          });
        }
      } catch (e) {
        console.warn(`[secretsScanner] Invalid pattern "${patternStr}" for type "${secretType.name}":`, e);
      }
    }
  }

  // Remove duplicates and sort by position
  const uniqueMatches = removeOverlappingMatches(matches);

  return {
    matches: uniqueMatches,
    count: uniqueMatches.length,
  };
}

/**
 * Scan text for secrets (async version).
 *
 * This function ensures patterns are loaded from the backend before
 * scanning. Use this for explicit user actions where a brief delay
 * is acceptable (e.g., before form submission).
 *
 * @param text - The text to scan for secrets
 * @returns SecretWarning with matches and count
 */
export async function scanForSecrets(text: string): Promise<SecretWarning> {
  // Ensure patterns are loaded from backend
  const patterns = await prefetchSecretPatterns();
  const matches: SecretMatch[] = [];

  for (const secretType of patterns) {
    for (const patternStr of secretType.patterns) {
      try {
        const jsPattern = rustPatternToJs(patternStr);
        const pattern = new RegExp(jsPattern, 'gi');
        let match;

        // Reset regex state
        pattern.lastIndex = 0;

        while ((match = pattern.exec(text)) !== null) {
          // For patterns with capture groups, use the captured value;
          // otherwise use the full match
          const value = (match as any)[1] || match[0];
          matches.push({
            type: secretType.name,
            value,
            startIndex: match.index,
            endIndex: match.index + value.length,
          });
        }
      } catch (e) {
        console.warn(`[secretsScanner] Invalid pattern "${patternStr}" for type "${secretType.name}":`, e);
      }
    }
  }

  // Remove duplicates and sort by position
  const uniqueMatches = removeOverlappingMatches(matches);

  return {
    matches: uniqueMatches,
    count: uniqueMatches.length,
  };
}

/**
 * Remove overlapping matches, keeping the most specific one.
 */
function removeOverlappingMatches(matches: SecretMatch[]): SecretMatch[] {
  if (matches.length === 0) return [];

  // Sort by start index, then by length (longer matches first)
  const sorted = [...matches].sort((a, b) => {
    if (a.startIndex !== b.startIndex) {
      return a.startIndex - b.startIndex;
    }
    return (b.endIndex - b.startIndex) - (a.endIndex - a.startIndex);
  });

  const filtered: SecretMatch[] = [];
  let lastEnd = -1;

  for (const match of sorted) {
    if (match.startIndex >= lastEnd) {
      filtered.push(match);
      lastEnd = match.endIndex;
    }
  }

  return filtered;
}

/**
 * Get severity level based on secret type.
 *
 * Uses the severity from the backend pattern, or falls back to
 * heuristic classification for unknown types.
 *
 * @param type - The secret type name
 * @param backendSeverity - The severity from the backend pattern (optional)
 * @returns 'high' | 'medium' | 'low'
 */
export function getSecretSeverity(type: string, backendSeverity?: string): 'high' | 'medium' | 'low' {
  // Use backend severity if provided
  if (backendSeverity === 'high' || backendSeverity === 'medium' || backendSeverity === 'low') {
    return backendSeverity;
  }

  // Fallback heuristic classification
  const highSeverity = [
    'Anthropic API Key',
    'Generic API Key',
    'AWS Access Key',
    'AWS Secret Key',
    'GitHub Token',
    'Private Key',
    'JWT',
    'Bearer Token',
  ];
  const mediumSeverity = [
    'Password',
    'Database URL',
    'Environment Variable Secret',
    'JSON Secret Field',
  ];

  if (highSeverity.includes(type)) return 'high';
  if (mediumSeverity.includes(type)) return 'medium';
  return 'low';
}

/**
 * Truncate secret value for display (show first and last few characters).
 *
 * @param value - The secret value to truncate
 * @param visibleChars - Number of characters to show at each end (default: 4)
 * @returns Truncated string with asterisks in the middle
 */
export function truncateSecret(value: string, visibleChars: number = 4): string {
  if (value.length <= visibleChars * 2) {
    return '*'.repeat(value.length);
  }
  return `${value.slice(0, visibleChars)}${'*'.repeat(value.length - visibleChars * 2)}${value.slice(-visibleChars)}`;
}
