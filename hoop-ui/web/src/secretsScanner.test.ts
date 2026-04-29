/**
 * Secrets scanner parity tests (§18)
 *
 * Verifies that the client-side secrets scanner produces the same results
 * as the backend scanner for a set of fixture secrets.
 *
 * Test scenarios:
 * 1. Client detects Anthropic API keys
 * 2. Client detects AWS access keys
 * 3. Client detects GitHub tokens
 * 4. Client detects Slack tokens
 * 5. Client detects JWTs
 * 6. Client detects Bearer tokens
 * 7. Client detects environment variable secrets
 * 8. Client detects JSON secret fields
 * 9. Multiple secrets in one text are all detected
 * 10. Overlapping matches are deduplicated correctly
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  scanForSecretsSync,
  scanForSecrets,
  prefetchSecretPatterns,
  getSecretSeverity,
  truncateSecret,
  clearPatternCache,
} from './components/secretsScanner';

// Mock the global fetch for patterns API
// These patterns MUST match backend default_secret_patterns() in config_resolver.rs exactly
// for true parity between client and backend scanning.
// Source of truth: hoop-daemon/src/config_resolver.rs::default_secret_patterns()
const mockPatterns = [
  // Stripe API keys
  {
    id: 'stripe_api_key',
    name: 'Stripe API Key',
    severity: 'high',
    patterns: [
      '\\bsk_live_[0-9a-zA-Z]{24,}\\b',
      '\\bsk_test_[0-9a-zA-Z]{24,}\\b',
      '\\bir_live_[0-9a-zA-Z]{32,}\\b',
      '\\bir_test_[0-9a-zA-Z]{32,}\\b',
    ],
  },
  // OpenAI API keys
  {
    id: 'openai_api_key',
    name: 'OpenAI API Key',
    severity: 'high',
    patterns: [
      '\\bsk-[a-zA-Z0-9]{48}\\b',
      '\\bsk-proj-[a-zA-Z0-9_-]{48,}\\b',
    ],
  },
  // Anthropic API keys
  {
    id: 'anthropic_api_key',
    name: 'Anthropic API Key',
    severity: 'high',
    patterns: [
      'sk-ant-[a-zA-Z0-9_-]{20,}',
    ],
  },
  // Generic API keys (sk- prefix) - matches Anthropic keys too, so order matters
  {
    id: 'generic_sk_key',
    name: 'Generic API Key',
    severity: 'high',
    patterns: [
      '\\bsk-[a-zA-Z0-9]{20,}\\b',
    ],
  },
  // AWS access keys
  {
    id: 'aws_access_key',
    name: 'AWS Access Key',
    severity: 'high',
    patterns: [
      '\\bAKIA[A-Z0-9]{16}\\b',
      '\\bASIA[A-Z0-9]{16}\\b',
    ],
  },
  // AWS secret access key
  {
    id: 'aws_secret_key',
    name: 'AWS Secret Access Key',
    severity: 'high',
    patterns: ['(?i)aws_secret_access_key\\s*[:=]\\s*[A-Za-z0-9/+=]{40}\\b'],
  },
  // GitHub tokens
  {
    id: 'github_token',
    name: 'GitHub Token',
    severity: 'high',
    patterns: [
      '\\bghp_[a-zA-Z0-9]{36}\\b',
      '\\bghs_[a-zA-Z0-9]{36}\\b',
      '\\bghu_[a-zA-Z0-9]{36}\\b',
      '\\bgithub_pat_[a-zA-Z0-9_]{82}\\b',
    ],
  },
  // JWT tokens
  {
    id: 'jwt',
    name: 'JWT',
    severity: 'high',
    patterns: ['\\bey[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}\\b'],
  },
  // Slack tokens
  {
    id: 'slack_token',
    name: 'Slack Token',
    severity: 'high',
    patterns: [
      '\\bxoxb-[0-9A-Za-z-]{24,}\\b',
      '\\bxoxp-[0-9A-Za-z-]{24,}\\b',
    ],
  },
  // Bearer tokens
  {
    id: 'bearer_token',
    name: 'Bearer Token',
    severity: 'high',
    patterns: ['(?i)bearer\\s+[A-Za-z0-9._\\-+/]{20,}'],
  },
  // Environment variable secrets
  {
    id: 'env_var_secret',
    name: 'Environment Variable Secret',
    severity: 'high',
    patterns: ['(?i)(?:api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|private[_-]?key|client[_-]?secret|anthropic[_-]?api[_-]?key|openai[_-]?api[_-]?key|github[_-]?token)\\s*[:=]\\s*["\']?([A-Za-z0-9+/_.~\\-]{16,})["\']?'],
  },
  // JSON secret fields
  {
    id: 'json_secret_field',
    name: 'JSON Secret Field',
    severity: 'high',
    patterns: ['(?i)"(?:password|passwd|secret|token|api_key|apikey|access_token|auth_token|private_key|client_secret)"\\s*:\\s*"([^"]{8,})"'],
  },
];

// Fixture secrets that should be detected by both client and backend
// NOTE: Lengths must exactly match backend pattern requirements
// NOTE: Avoid variable names that trigger env_var_secret pattern (e.g., "OPENAI_API_KEY=")
const FIXTURE_SECRETS = {
  // Stripe keys (min 24 chars after prefix)
  stripeLiveKey: 'My Stripe key is sk_live_51AbCdEf1234567890AbCdEf1234567890AbC',
  stripeTestKey: 'Stripe test: sk_test_51AbCdEf1234567890AbCdEf1234567890AbC',
  stripeIrLive: 'IR key: ir_live_51AbCdEf1234567890AbCdEf1234567890AbCdEf123456',
  // OpenAI keys (exactly 48 chars for sk-, min 48 for sk-proj-)
  // Use non-standard variable names to avoid env_var_secret pattern
  openaiKey: 'My OpenAI key is sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijkl',
  // Pattern: \bsk-proj-[a-zA-Z0-9_-]{48,}\b means at least 48 chars AFTER 'sk-proj-'
  openaiProjKey: 'OpenAI project key: sk-proj-AbCdEf1234567890AbCdEf1234567890AbCdEf1234567890abc',
  // Anthropic key without the variable name to avoid matching env_var_secret pattern
  anthropicKey: 'Here is my key sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666 please keep it safe',
  anthropicShortKey: 'Key: sk-ant-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456',
  // AWS keys (AKIA/ASIA + 16 chars, secret = 40 chars)
  awsAccessKey: 'aws_access_key_id = AKIAIOSFODNN7EXAMPLE',
  awsTempKey: 'ASIA1234567890ABCDEF',
  awsSecretKey: 'aws_secret_access_key = ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890+/AB',
  // GitHub tokens (36 chars for ghp/ghs/ghu/gho/ghr, 82 for github_pat)
  githubToken: 'token=ghp_16C7e42F292c6912E7710c838347Ae178B4a',
  // Use non-standard variable name to avoid env_var_secret pattern
  githubPat: 'My GitHub token is github_pat_1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ12345678901234567890',
  // Slack tokens (min 24 chars after prefix)
  slackBot: 'SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456',
  slackUser: 'xoxp-1234567890-1234567890123-12345678901234567890123456',
  // JWT and Bearer
  jwt: 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U',
  // Generic sk- key (without common variable names)
  genericKey: 'Key: sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmn',
  // Environment variable secrets (intentionally uses standard variable names)
  envVarSecret: 'export openai_api_key=sk-proj-AbCdEf1234567890',
  jsonSecret: '{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}',
  multipleSecrets: `
    My API keys:
    Stripe: sk_live_51AbCdEf1234567890AbCdEf
    OpenAI: sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijkl
    Anthropic: sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD
    GitHub: ghp_1234567890abcdef1234567890abcd123456
    AWS: AKIA1234567890ABCDEF
  `,
  overlappingSecrets: 'Key=sk-ant-test1234567890ABCDEFGHIJ1234567890ABCD and Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N',
};

describe('secrets scanner parity (§18)', () => {
  beforeEach(async () => {
    // Clear the pattern cache before each test
    clearPatternCache();

    // Mock fetch to return backend patterns
    globalThis.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({
          schema_version: '1.0.0',
          patterns: mockPatterns,
        }),
      } as Response)
    );

    // Prefetch patterns to ensure cache is populated
    await prefetchSecretPatterns();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('detection with backend patterns (async)', () => {
    it('detects Stripe live key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.stripeLiveKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Stripe API Key')).toBe(true);
    });

    it('detects Stripe test key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.stripeTestKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Stripe API Key')).toBe(true);
    });

    it('detects Stripe IR key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.stripeIrLive);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Stripe API Key')).toBe(true);
    });

    it('detects OpenAI API key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.openaiKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'OpenAI API Key')).toBe(true);
    });

    it('detects OpenAI project key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.openaiProjKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'OpenAI API Key')).toBe(true);
    });

    it('detects Anthropic API key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.anthropicKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Anthropic API Key')).toBe(true);
      expect(result.matches.some((m) => m.value.includes('sk-ant-'))).toBe(true);
    });

    it('detects Anthropic short key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.anthropicShortKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Anthropic API Key')).toBe(true);
    });

    it('detects AWS access key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.awsAccessKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Access Key')).toBe(true);
    });

    it('detects AWS temporary key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.awsTempKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Access Key')).toBe(true);
    });

    it('detects AWS secret key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.awsSecretKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Secret Access Key')).toBe(true);
    });

    it('detects GitHub token', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.githubToken);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'GitHub Token')).toBe(true);
    });

    it('detects GitHub PAT', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.githubPat);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'GitHub Token')).toBe(true);
    });

    it('detects Slack bot token', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.slackBot);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Slack Token')).toBe(true);
    });

    it('detects Slack user token', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.slackUser);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Slack Token')).toBe(true);
    });

    it('detects JWT', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.jwt);
      expect(result.count).toBeGreaterThan(0);
      // JWT is detected either by JWT pattern or Bearer Token pattern
      const hasJWTOrBearer = result.matches.some(
        (m) => m.type === 'JWT' || m.type === 'Bearer Token'
      );
      expect(hasJWTOrBearer).toBe(true);
    });

    it('detects generic API key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.genericKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Generic API Key')).toBe(true);
    });

    it('detects environment variable secret', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.envVarSecret);
      expect(result.count).toBeGreaterThan(0);
      expect(
        result.matches.some((m) => m.type === 'Environment Variable Secret')
      ).toBe(true);
    });

    it('detects JSON secret field', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.jsonSecret);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'JSON Secret Field')).toBe(true);
    });

    it('detects all secrets in multi-secret text', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.multipleSecrets);
      // Should detect at least 5 secrets (Stripe, OpenAI, Anthropic, GitHub, AWS)
      expect(result.count).toBeGreaterThanOrEqual(5);
      const types = result.matches.map((m) => m.type);
      expect(types).toContain('Stripe API Key');
      expect(types).toContain('OpenAI API Key');
      expect(types).toContain('Anthropic API Key');
      expect(types).toContain('GitHub Token');
      expect(types).toContain('AWS Access Key');
    });
  });

  describe('detection with cached patterns (sync)', () => {
    it('detects Stripe key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.stripeLiveKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Stripe API Key')).toBe(true);
    });

    it('detects OpenAI key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.openaiKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'OpenAI API Key')).toBe(true);
    });

    it('detects Anthropic API key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.anthropicKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Anthropic API Key')).toBe(true);
    });

    it('detects AWS access key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.awsAccessKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Access Key')).toBe(true);
    });

    it('detects GitHub token synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.githubToken);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'GitHub Token')).toBe(true);
    });

    it('detects Slack token synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.slackBot);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Slack Token')).toBe(true);
    });
  });

  describe('match deduplication', () => {
    it('removes overlapping matches keeping the most specific', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.overlappingSecrets);
      // Should detect both secrets without overlapping duplicates
      expect(result.count).toBeGreaterThan(0);
      // Verify no overlapping matches (end index <= next start index)
      const sorted = [...result.matches].sort((a, b) => a.startIndex - b.startIndex);
      for (let i = 0; i < sorted.length - 1; i++) {
        expect(sorted[i].endIndex).toBeLessThanOrEqual(sorted[i + 1].startIndex);
      }
    });

    it('sorts matches by position', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.multipleSecrets);
      // Verify matches are sorted by start index
      for (let i = 0; i < result.matches.length - 1; i++) {
        expect(result.matches[i].startIndex).toBeLessThanOrEqual(
          result.matches[i + 1].startIndex
        );
      }
    });
  });

  describe('severity classification', () => {
    it('classifies Anthropic API key as high severity', () => {
      const severity = getSecretSeverity('Anthropic API Key', 'high');
      expect(severity).toBe('high');
    });

    it('classifies AWS access key as high severity', () => {
      const severity = getSecretSeverity('AWS Access Key', 'high');
      expect(severity).toBe('high');
    });

    it('falls back to heuristic when backend severity is missing', () => {
      const severity = getSecretSeverity('Anthropic API Key', undefined);
      expect(severity).toBe('high');
    });
  });

  describe('secret truncation for display', () => {
    it('truncates long secrets showing first and last chars', () => {
      const secret = 'sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555';
      const truncated = truncateSecret(secret, 4);
      expect(truncated).toHaveLength(secret.length);
      expect(truncated.startsWith('sk-a')).toBe(true);
      expect(truncated.endsWith('5555')).toBe(true);
      expect(truncated).toContain('*');
    });

    it('handles short secrets correctly', () => {
      const secret = 'short';
      const truncated = truncateSecret(secret, 4);
      // Short secrets are fully redacted
      expect(truncated).toHaveLength(secret.length);
      expect(truncated).toMatch(/^\*+$/);
    });
  });

  describe('clean text has no secrets', () => {
    it('returns empty result for clean text', () => {
      const cleanText = 'This is a normal message with no secrets. Just plain text.';
      const result = scanForSecretsSync(cleanText);
      expect(result.count).toBe(0);
      expect(result.matches).toHaveLength(0);
    });
  });

  describe('backend API requirement', () => {
    it('throws error when backend is unavailable', async () => {
      // Clear the pattern cache
      clearPatternCache();

      // Mock fetch to fail
      globalThis.fetch = vi.fn(() =>
        Promise.resolve({
          ok: false,
          status: 503,
          statusText: 'Service Unavailable',
        } as Response)
      );

      await expect(prefetchSecretPatterns()).rejects.toThrow();
    });
  });

  describe('match positions are correct', () => {
    it('returns correct byte offsets for detected secrets', () => {
      const text = 'My key is sk-ant-test1234567890ABCDEFGH end';
      const result = scanForSecretsSync(text);
      expect(result.count).toBeGreaterThan(0);
      const match = result.matches[0];
      expect(match.startIndex).toBeGreaterThanOrEqual(0);
      expect(match.endIndex).toBeGreaterThan(match.startIndex);
      expect(match.endIndex).toBeLessThanOrEqual(text.length);
      // Verify the matched value is correct
      expect(text.substring(match.startIndex, match.endIndex)).toContain(match.value);
    });
  });
});
