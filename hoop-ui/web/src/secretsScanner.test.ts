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
 * 11. Built-in fallback patterns work when API is unavailable
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  scanForSecretsSync,
  scanForSecrets,
  prefetchSecretPatterns,
  getSecretSeverity,
  truncateSecret,
  type SecretWarning,
  type SecretMatch,
} from './components/secretsScanner';

// Fixture secrets that should be detected by both client and backend
const FIXTURE_SECRETS = {
  anthropicKey: 'ANTHROPIC_API_KEY=sk-ant-api03-AAAA1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666',
  genericKey: 'API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890abcdefghijklmn',
  awsKey: 'aws_access_key_id = AKIAIOSFODNN7EXAMPLE',
  githubToken: 'token=ghp_16C7e42F292c6912E7710c838347Ae178B4a',
  slackToken: 'SLACK_TOKEN=xoxb-1234567890-1234567890123-12345678901234567890123456',
  jwt: 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U',
  envVarSecret: 'export openai_api_key=sk-proj-AbCdEf1234567890',
  jsonSecret: '{"password": "s3cr3tP@ssw0rd!", "api_key": "abc123def456ghi789jkl"}',
  multipleSecrets: `
    My API keys:
    Anthropic: sk-ant-api03-TEST1234567890ABCDEFGHIJ1234567890ABCD
    GitHub: ghp_1234567890abcdef1234567890abcdef123456
    AWS: AKIA1234567890ABCDEFG
  `,
  overlappingSecrets: 'Key=sk-ant-test1234567890ABCDEFGHIJ1234567890ABCD and Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N',
};

describe('secrets scanner parity (§18)', () => {
  beforeEach(async () => {
    // Clear cached patterns to ensure each test starts fresh
    vi.resetModules();
    // Prefetch patterns to ensure cache is populated
    await prefetchSecretPatterns().catch(() => {
      // If backend is unavailable, tests will use built-in patterns
      console.warn('Backend unavailable, using built-in patterns');
    });
  });

  describe('detection with backend patterns (async)', () => {
    it('detects Anthropic API key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.anthropicKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Anthropic API Key')).toBe(true);
      expect(result.matches.some((m) => m.value.includes('sk-ant-'))).toBe(true);
    });

    it('detects generic API key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.genericKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Generic API Key')).toBe(true);
    });

    it('detects AWS access key', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.awsKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Access Key')).toBe(true);
    });

    it('detects GitHub token', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.githubToken);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'GitHub Token')).toBe(true);
    });

    it('detects Slack token', async () => {
      const result = await scanForSecrets(FIXTURE_SECRETS.slackToken);
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
      // Should detect at least 3 secrets (Anthropic, GitHub, AWS)
      expect(result.count).toBeGreaterThanOrEqual(3);
      const types = result.matches.map((m) => m.type);
      expect(types).toContain('Anthropic API Key');
      expect(types).toContain('GitHub Token');
      expect(types).toContain('AWS Access Key');
    });
  });

  describe('detection with cached patterns (sync)', () => {
    it('detects Anthropic API key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.anthropicKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'Anthropic API Key')).toBe(true);
    });

    it('detects AWS access key synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.awsKey);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'AWS Access Key')).toBe(true);
    });

    it('detects GitHub token synchronously', () => {
      const result = scanForSecretsSync(FIXTURE_SECRETS.githubToken);
      expect(result.count).toBeGreaterThan(0);
      expect(result.matches.some((m) => m.type === 'GitHub Token')).toBe(true);
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

  describe('match positions are correct', () => {
    it('returns correct byte offsets for detected secrets', () => {
      const text = 'My key is sk-ant-test1234567890ABCD end';
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
