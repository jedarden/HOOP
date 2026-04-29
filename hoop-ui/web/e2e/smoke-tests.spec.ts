import { test, expect } from '@playwright/test';

/**
 * Smoke Tests - Major UI Panels
 *
 * Tests that each major panel loads without errors and displays expected content.
 * Per §14.2 UI tests of plan.md.
 *
 * Run: npm run test:e2e smoke-tests.spec.ts
 */

test.describe('Smoke Tests - Overview Page', () => {
  test('should load overview page with header and navigation', async ({ page }) => {
    await page.goto('/');

    // Main header
    await expect(page.locator('h1').filter({ hasText: 'HOOP' })).toBeVisible();

    // Connection indicator
    await expect(page.locator('.connection-indicator')).toBeVisible();

    // Settings menu button
    await expect(page.locator('button[aria-label="Settings"]')).toBeVisible();
  });

  test('should display fleet summary strip with metrics', async ({ page }) => {
    await page.goto('/');

    // Fleet summary strip
    await expect(page.locator('.fleet-summary-strip')).toBeVisible();

    // At least projects counter should be visible
    await expect(page.locator('.fleet-summary-strip').locator('.fss-item').first()).toBeVisible();
  });

  test('should display project cards or empty/loading state', async ({ page }) => {
    await page.goto('/');

    // Wait for page to stabilize
    await page.waitForLoadState('networkidle');

    // Either loading, empty, or cards grid
    const loading = page.locator('.fleet-loading');
    const empty = page.locator('.fleet-empty');
    const cards = page.locator('.fleet-cards-grid');

    const isLoading = await loading.count() > 0;
    const isEmpty = await empty.count() > 0;
    const hasCards = await cards.count() > 0;

    expect(isLoading || isEmpty || hasCards).toBeTruthy();
  });

  test('should have navigation links to cross-project views', async ({ page }) => {
    await page.goto('/');

    // Dashboard link
    await expect(page.locator('a[href="#/dashboard"]')).toBeVisible();

    // Fleet link
    await expect(page.locator('a[href="#/fleet"]')).toBeVisible();

    // Timeline link
    await expect(page.locator('a[href="#/timeline"]')).toBeVisible();

    // Diagnostics link
    await expect(page.locator('a[href="#/diagnostics"]')).toBeVisible();
  });
});

test.describe('Smoke Tests - Project Detail', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for project cards to load
    await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });
  });

  test('should navigate to project detail when clicking a project card', async ({ page }) => {
    // Find first project card
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    // Get project name from card
    const projectName = await firstCard.locator('.pcf-label').textContent();

    // Click card
    await firstCard.click();

    // Should navigate to project detail
    await expect(page.locator('.app-project-detail')).toBeVisible();

    // Should have back link
    await expect(page.locator('a.back-link')).toBeVisible();
  });

  test('should display all tab buttons in project detail', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();

    // Wait for project detail to load
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Tab list should be visible
    const tabList = page.locator('.tab-list');
    if (await tabList.count() > 0) {
      await expect(tabList).toBeVisible();

      // Check for standard tabs
      const expectedTabs = ['Stitches', 'Fleet Map', 'Bead Graph', 'Conversations'];
      for (const tabName of expectedTabs) {
        const tab = page.locator('button[role="tab"]', { hasText: tabName });
        await expect(tab.first()).toBeVisible();
      }
    }
  });

  test('should display Stitches tab content', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Click Stitches tab
    const stitchesTab = page.locator('button[role="tab"]', { hasText: 'Stitches' });
    const tabCount = await stitchesTab.count();
    if (tabCount > 0) {
      await stitchesTab.first().click();

      // Stitches tab should be active
      await expect(stitchesTab.first()).toHaveAttribute('aria-selected', 'true');

      // Content area should be visible
      await expect(page.locator('.tab-content, main')).toBeVisible();
    }
  });

  test('should display Fleet Map tab content', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Click Fleet Map tab
    const fleetTab = page.locator('button[role="tab"]', { hasText: 'Fleet Map' });
    const tabCount = await fleetTab.count();
    if (tabCount > 0) {
      await fleetTab.first().click();

      // Fleet Map should be visible
      await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();
    }
  });

  test('should display Cost tab content', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Click Cost tab
    const costTab = page.locator('button[role="tab"]', { hasText: 'Cost' });
    const tabCount = await costTab.count();
    if (tabCount > 0) {
      await costTab.first().click();

      // Cost panel should be visible
      const costPanel = page.locator('.cost-panel, .cost-empty, .cost-loading');
      await expect(costPanel.first()).toBeVisible();
    }
  });

  test('should display Files tab content', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Click Files tab
    const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
    const tabCount = await filesTab.count();
    if (tabCount > 0) {
      await filesTab.first().click();

      // Files tab should be visible
      await expect(page.locator('.files-tab, .file-browser, .file-tree')).toBeVisible();
    }
  });

  test('should display Debug tab content', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();

    const cardCount = await firstCard.count();
    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    // Click Debug tab
    const debugTab = page.locator('button[role="tab"]', { hasText: 'Debug' });
    const tabCount = await debugTab.count();
    if (tabCount > 0) {
      await debugTab.first().click();

      // Debug panel should be visible
      await expect(page.locator('.debug-panel, .debug-empty')).toBeVisible();
    }
  });
});

test.describe('Smoke Tests - Fleet View', () => {
  test('should load fleet view with worker map', async ({ page }) => {
    await page.goto('/#/fleet');

    // Fleet map should be visible
    await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();

    // Connection indicator
    await expect(page.locator('.connection-indicator')).toBeVisible();

    // Back link
    await expect(page.locator('a.back-link')).toBeVisible();
  });

  test('should display agent chat pane', async ({ page }) => {
    await page.goto('/#/fleet');

    // Agent chat pane should be visible
    await expect(page.locator('.agent-chat-pane, .chat-pane')).toBeVisible();

    // Chat input should be present
    await expect(page.locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')).toBeAttached();
  });

  test('should display capacity panel', async ({ page }) => {
    await page.goto('/#/fleet');

    // Capacity panel should be visible
    await expect(page.locator('.capacity-panel')).toBeVisible();
  });
});

test.describe('Smoke Tests - Cross-Project Dashboard', () => {
  test('should load dashboard view', async ({ page }) => {
    await page.goto('/#/dashboard');

    // Dashboard should be visible
    await expect(page.locator('.cross-project-dashboard, .dashboard')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
    await expect(page.locator('.connection-indicator')).toBeVisible();
  });

  test('should display dashboard metrics', async ({ page }) => {
    await page.goto('/#/dashboard');

    // Wait for content to load
    await page.waitForLoadState('networkidle');

    // Dashboard should have some content
    const dashboard = page.locator('.cross-project-dashboard, .dashboard');
    await expect(dashboard.first()).toBeVisible();
  });
});

test.describe('Smoke Tests - Patterns View', () => {
  test('should load patterns view', async ({ page }) => {
    await page.goto('/#/patterns');

    // Patterns view should be visible
    await expect(page.locator('.patterns-view, .patterns-container')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });

  test('should display pattern list or empty state', async ({ page }) => {
    await page.goto('/#/patterns');

    // Wait for content
    await page.waitForLoadState('networkidle');

    // Either pattern list or empty state
    const patternList = page.locator('.pattern-list, .patterns-container');
    await expect(patternList.first()).toBeVisible();
  });
});

test.describe('Smoke Tests - Conversations View', () => {
  test('should load conversations view', async ({ page }) => {
    await page.goto('/#/conversations');

    // Conversations view should be visible
    await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });

  test('should display conversation list or empty state', async ({ page }) => {
    await page.goto('/#/conversations');

    // Wait for content
    await page.waitForLoadState('networkidle');

    // Conversations container should be visible
    await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();
  });
});

test.describe('Smoke Tests - Drafts View', () => {
  test('should load drafts view', async ({ page }) => {
    await page.goto('/#/drafts');

    // Drafts view should be visible
    await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });

  test('should display draft list or empty state', async ({ page }) => {
    await page.goto('/#/drafts');

    // Wait for content
    await page.waitForLoadState('networkidle');

    // Drafts container should be visible
    await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  });
});

test.describe('Smoke Tests - Timeline View', () => {
  test('should load timeline view', async ({ page }) => {
    await page.goto('/#/timeline');

    // Timeline should be visible
    await expect(page.locator('.worker-timeline, .timeline-view')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });
});

test.describe('Smoke Tests - Audit View', () => {
  test('should load audit view', async ({ page }) => {
    await page.goto('/#/audit');

    // Audit panel should be visible
    await expect(page.locator('.audit-panel, .audit-log')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });
});

test.describe('Smoke Tests - Redaction Audit View', () => {
  test('should load redaction audit view', async ({ page }) => {
    await page.goto('/#/redaction-audit');

    // Redaction audit panel should be visible
    await expect(page.locator('.redaction-audit-panel')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });
});

test.describe('Smoke Tests - Unassigned Sessions View', () => {
  test('should load unassigned sessions view', async ({ page }) => {
    await page.goto('/#/unassigned');

    // Unassigned sessions should be visible
    await expect(page.locator('.unassigned-sessions, .unassigned-container')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });
});

test.describe('Smoke Tests - Diagnostics View', () => {
  test('should load diagnostics view', async ({ page }) => {
    await page.goto('/#/diagnostics');

    // Diagnostics should be visible
    await expect(page.locator('.unknown-events-diagnostics, .diagnostics-container')).toBeVisible();

    // Navigation
    await expect(page.locator('a.back-link')).toBeVisible();
  });
});

test.describe('Smoke Tests - Navigation Between Views', () => {
  test('should navigate from overview to project and back', async ({ page }) => {
    await page.goto('/');

    // Wait for projects to load
    await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });

    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount > 0) {
      await firstCard.click();
      await expect(page.locator('.app-project-detail')).toBeVisible();

      // Navigate back
      await page.locator('a.back-link').first().click();
      await expect(page.locator('.app:not(.app-project-detail)')).toBeVisible();
    }
  });

  test('should navigate between cross-project views', async ({ page }) => {
    await page.goto('/#/dashboard');
    await expect(page.locator('.cross-project-dashboard, .dashboard')).toBeVisible();

    await page.goto('/#/patterns');
    await expect(page.locator('.patterns-view, .patterns-container')).toBeVisible();

    await page.goto('/#/conversations');
    await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();

    await page.goto('/#/drafts');
    await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  });

  test('should navigate from overview to fleet and back', async ({ page }) => {
    await page.goto('/');

    // Click fleet link
    await page.locator('a[href="#/fleet"]').first().click();
    await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();

    // Navigate back
    await page.locator('a.back-link').first().click();
    await expect(page.locator('.fleet-cards-grid, .fleet-empty, .fleet-loading')).toBeAttached();
  });
});

test.describe('Smoke Tests - Search Palette', () => {
  test('should open search palette with Cmd/Ctrl+K', async ({ page }) => {
    await page.goto('/');

    // Press Cmd+K (or Ctrl+K on non-Mac)
    await page.keyboard.press((process.platform === 'darwin' ? 'Meta' : 'Control') + '+k');

    // Search palette should be visible
    await expect(page.locator('.search-palette, [role="dialog"]')).toBeVisible();
  });

  test('should close search palette on escape', async ({ page }) => {
    await page.goto('/');

    // Open search palette
    await page.keyboard.press((process.platform === 'darwin' ? 'Meta' : 'Control') + '+k');
    await expect(page.locator('.search-palette, [role="dialog"]')).toBeVisible();

    // Close with Escape
    await page.keyboard.press('Escape');
    await expect(page.locator('.search-palette, [role="dialog"]')).not.toBeVisible();
  });
});

test.describe('Smoke Tests - Settings Menu', () => {
  test('should open settings menu', async ({ page }) => {
    await page.goto('/');

    // Click settings button
    await page.locator('button[aria-label="Settings"]').click();

    // Settings menu should be visible
    await expect(page.locator('.settings-menu, [role="menu"]')).toBeVisible();
  });

  test('should close settings menu on click outside', async ({ page }) => {
    await page.goto('/');

    // Open settings
    await page.locator('button[aria-label="Settings"]').click();
    await expect(page.locator('.settings-menu, [role="menu"]')).toBeVisible();

    // Click outside
    await page.locator('main').click();
    await expect(page.locator('.settings-menu, [role="menu"]')).not.toBeVisible();
  });
});

test.describe('Smoke Tests - Error Handling', () => {
  test('should show 404 for non-existent project', async ({ page }) => {
    await page.goto('/#/non-existent-project-xyz123');

    // Should show not found message
    await expect(page.locator('text=/not found/i')).toBeVisible();
  });

  test('should handle connection errors gracefully', async ({ page }) => {
    // This test verifies the UI handles connection state
    await page.goto('/');

    // Connection indicator should be present (either connected or connecting)
    await expect(page.locator('.connection-indicator')).toBeVisible();
  });
});
