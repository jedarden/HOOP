/**
 * Secrets Scanner for dictated notes
 *
 * Scans transcript text for potential secrets and sensitive information.
 * Uses regex patterns to detect common secret formats.
 */

export interface SecretMatch {
  type: string;
  value: string;
  startIndex: number;
  endIndex: number;
}

export interface SecretWarning {
  matches: SecretMatch[];
  count: number;
}

// Common secret patterns
const SECRET_PATTERNS = [
  {
    type: 'API Key',
    // Matches: sk-..., api_key..., API_KEY=..., etc.
    patterns: [
      /(?:sk_|api[_-]?key|apikey|secret[_-]?key|secretkey)\s*[:=]\s*['"']?([a-zA-Z0-9_\-]{20,})['"']?/gi,
      /(sk_[a-zA-Z0-9]{20,})/g,
    ],
  },
  {
    type: 'Bearer Token',
    patterns: [
      /(?:bearer|authorization)\s*:\s*['"']?([a-zA-Z0-9_\-\.]{20,})['"']?/gi,
    ],
  },
  {
    type: 'JWT Token',
    patterns: [
      /eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+/g,
    ],
  },
  {
    type: 'AWS Access Key',
    patterns: [
      /(AKIA[0-9A-Z]{16})/g,
    ],
  },
  {
    type: 'AWS Secret Key',
    patterns: [
      /(?:aws[_-]?secret[_-]?access[_-]?key|secret[_-]?key)\s*[:=]\s*['"']?([a-zA-Z0-9/+=]{40})['"']?/gi,
    ],
  },
  {
    type: 'GitHub Token',
    patterns: [
      /(ghp_[a-zA-Z0-9]{36})/g,
      /(gho_[a-zA-Z0-9]{36})/g,
      /(ghu_[a-zA-Z0-9]{36})/g,
      /(ghs_[a-zA-Z0-9]{36})/g,
      /(ghr_[a-zA-Z0-9]{36})/g,
    ],
  },
  {
    type: 'Password',
    patterns: [
      /(?:password|passwd|pwd)\s*[:=]\s*['"']?([^\s'"']{8,})['"']?/gi,
    ],
  },
  {
    type: 'Private Key',
    patterns: [
      /-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----/g,
      /-----BEGIN\s+EC\s+PRIVATE\s+KEY-----/g,
      /-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----/g,
    ],
  },
  {
    type: 'Database URL',
    patterns: [
      /(?:postgres|mysql|mongodb|redis|sqlite)?:\/\/[^\s'"']+/gi,
    ],
  },
  {
    type: 'Email Address',
    patterns: [
      /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g,
    ],
  },
  {
    type: 'IP Address',
    patterns: [
      /\b(?:\d{1,3}\.){3}\d{1,3}\b/g,
    ],
  },
  {
    type: 'Phone Number',
    patterns: [
      /\b\+?[\d\s\-()]{10,}\b/g,
    ],
  },
  {
    type: 'Credit Card',
    patterns: [
      /\b(?:\d{4}[-\s]?){3}\d{4}\b/g,
    ],
  },
  {
    type: 'SSN',
    patterns: [
      /\b\d{3}-\d{2}-\d{4}\b/g,
    ],
  },
];

/**
 * Scan text for potential secrets
 */
export function scanForSecrets(text: string): SecretWarning {
  const matches: SecretMatch[] = [];

  for (const secretType of SECRET_PATTERNS) {
    for (const pattern of secretType.patterns) {
      let match;
      // Reset regex state for each pattern
      pattern.lastIndex = 0;

      while ((match = pattern.exec(text)) !== null) {
        const value = match[1] || match[0];
        matches.push({
          type: secretType.type,
          value,
          startIndex: match.index,
          endIndex: match.index + value.length,
        });
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
 * Remove overlapping matches, keeping the most specific one
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
 * Get severity level based on secret type
 */
export function getSecretSeverity(type: string): 'high' | 'medium' | 'low' {
  const highSeverity = ['API Key', 'AWS Access Key', 'AWS Secret Key', 'GitHub Token', 'Private Key', 'JWT Token', 'Bearer Token'];
  const mediumSeverity = ['Password', 'Database URL'];

  if (highSeverity.includes(type)) return 'high';
  if (mediumSeverity.includes(type)) return 'medium';
  return 'low';
}

/**
 * Truncate secret value for display (show first and last few characters)
 */
export function truncateSecret(value: string, visibleChars: number = 4): string {
  if (value.length <= visibleChars * 2) {
    return '*'.repeat(value.length);
  }
  return `${value.slice(0, visibleChars)}${'*'.repeat(Math.min(value.length - visibleChars * 2, 8))}${value.slice(-visibleChars)}`;
}
