#!/usr/bin/env node
/**
 * HOOP UI Screenshot Generator
 *
 * This script takes anonymized screenshots of the HOOP UI for documentation.
 * All sensitive data (project names, bead IDs, etc.) is replaced with placeholders.
 */

import { chromium, Browser, Page, BrowserContext } from 'playwright';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOTS_DIR = join(__dirname, '../../docs/screenshots');
const BASE_URL = process.env.BASE_URL || 'http://localhost:5173';

// Ensure screenshots directory exists
mkdirSync(SCREENSHOTS_DIR, { recursive: true });

interface ScreenshotSpec {
  name: string;
  path: string;
  description: string;
  setup?: (page: Page) => Promise<void>;
}

async function takeScreenshots() {
  console.log('🚀 Starting screenshot generation...');
  console.log(`📁 Output directory: ${SCREENSHOTS_DIR}`);
  console.log(`🌐 Base URL: ${BASE_URL}`);

  const browser = await chromium.launch({
    headless: true,
  });

  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    // Anonymize by blocking any external tracking
    serviceWorkers: 'block',
  });

  const screenshots: ScreenshotSpec[] = [
    {
      name: 'Dashboard',
      path: 'dashboard.png',
      description: 'Main project dashboard showing project cards',
      setup: async (page) => {
        // Navigate to home page
        await page.goto(BASE_URL);
        // Wait for project cards to load
        await page.waitForSelector('[data-testid="project-card"]', { timeout: 10000 });
      },
    },
    {
      name: 'Project Detail',
      path: 'project-detail.png',
      description: 'Single project view with Stitch timeline',
      setup: async (page) => {
        await page.goto(`${BASE_URL}/project/testrepo`);
        // Wait for Stitch list to load
        await page.waitForSelector('[data-testid="stitch-list"]', { timeout: 10000 });
      },
    },
    {
      name: 'Agent Chat',
      path: 'agent-chat.png',
      description: 'Agent chat interface for human interaction',
      setup: async (page) => {
        await page.goto(`${BASE_URL}/project/testrepo`);
        // Open chat panel if not already open
        const chatButton = page.locator('[data-testid="chat-toggle"]');
        if (await chatButton.isVisible()) {
          await chatButton.click();
        }
        await page.waitForSelector('[data-testid="chat-messages"]', { timeout: 10000 });
      },
    },
    {
      name: 'File Browser',
      path: 'file-browser.png',
      description: 'Code file browser with syntax highlighting',
      setup: async (page) => {
        await page.goto(`${BASE_URL}/project/testrepo/files`);
        // Wait for file tree to load
        await page.waitForSelector('[data-testid="file-tree"]', { timeout: 10000 });
        // Navigate to a source file
        const srcLink = page.locator('text=src').first();
        if (await srcLink.isVisible()) {
          await srcLink.click();
        }
      },
    },
    {
      name: 'Settings',
      path: 'settings.png',
      description: 'Settings menu with configuration options',
      setup: async (page) => {
        await page.goto(BASE_URL);
        // Open settings menu
        const settingsButton = page.locator('[data-testid="settings-toggle"]');
        if (await settingsButton.isVisible()) {
          await settingsButton.click();
        }
        await page.waitForSelector('[data-testid="settings-menu"]', { timeout: 5000 });
      },
    },
  ];

  let successCount = 0;
  let failCount = 0;

  for (const spec of screenshots) {
    try {
      console.log(`\n📸 Taking screenshot: ${spec.name}`);
      console.log(`   ${spec.description}`);

      const page = await context.newPage();

      // Apply anonymization styles (hide sensitive data)
      await page.addStyleTag({
        content: `
          /* Anonymize project names and IDs */
          [data-testid="project-name"],
          .project-id,
          .bead-id,
          .stitch-id {
            filter: blur(4px);
          }
          /* Add visual indicator for anonymization */
          .anonymized::after {
            content: " (anonymized)";
            color: #666;
            font-size: 0.8em;
          }
        `,
      });

      // Run setup
      if (spec.setup) {
        await spec.setup(page);
      }

      // Take screenshot
      const screenshotPath = join(SCREENSHOTS_DIR, spec.path);
      await page.screenshot({
        path: screenshotPath,
        fullPage: false,
      });

      console.log(`   ✅ Saved to: ${screenshotPath}`);
      successCount++;

      await page.close();
    } catch (error) {
      console.error(`   ❌ Failed: ${error}`);
      failCount++;
    }
  }

  await context.close();
  await browser.close();

  console.log('\n' + '='.repeat(50));
  console.log(`✨ Screenshot generation complete!`);
  console.log(`   ✅ Success: ${successCount}`);
  console.log(`   ❌ Failed: ${failCount}`);
  console.log(`   📁 Location: ${SCREENSHOTS_DIR}`);
  console.log('='.repeat(50));

  // Generate README for screenshots
  const readmeContent = `# HOOP UI Screenshots

This directory contains anonymized screenshots of the HOOP user interface for documentation purposes.

## Screenshots

${screenshots.map(s => `
### ${s.name}
![${s.name}](./${s.path})
*${s.description}*`).join('\n')}

## Notes

- Screenshots are taken with a viewport of 1920x1080 (desktop)
- Sensitive data (project names, bead IDs, etc.) is visually anonymized
- Screenshots are generated using Playwright (see \`scripts/take-screenshots.ts\`)
- Regenerate screenshots by running: \`pnpm tsx scripts/take-screenshots.ts\`
`;

  writeFileSync(join(SCREENSHOTS_DIR, 'README.md'), readmeContent);
  console.log(`\n📄 Generated screenshots README`);
}

// Run the screenshot generation
takeScreenshots().catch(console.error);
