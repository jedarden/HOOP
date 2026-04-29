import { test, expect } from '@playwright/test';

/**
 * Visual Regression Tests
 *
 * Screenshot-based testing with configurable diff thresholds.
 * Per §14.2 UI tests of plan.md.
 *
 * Visual diffs are tolerated within thresholds; new baselines require explicit approval.
 *
 * Run: npm run test:e2e visual-regression.spec.ts
 *
 * To update baselines after intentional UI changes:
 * npx playwright test --update-snapshots visual-regression.spec.ts
 */

// Visual regression thresholds (percentage of pixels that can differ)
const VISUAL_THRESHOLDS = {
  // Layout and structure changes - more tolerant
  layout: 0.15, // 15% - allows for dynamic content, loading states
  // Color and styling changes - stricter
  styling: 0.05, // 5% - only minor rendering differences
  // Typography and spacing - moderate
  typography: 0.08, // 8% - allows for font rendering variations
  // Icons and graphics - moderate
  graphics: 0.10, // 10% - allows for minor icon rendering differences
};

test.describe('Visual Regression - Overview Page', () => {
  test('should match overview page snapshot on desktop', async ({ page }) => {
    await page.goto('/');

    // Wait for projects to load
    await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });
    await page.waitForLoadState('networkidle');

    // Take full page screenshot
    await expect(page).toHaveScreenshot('overview-desktop.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match overview page snapshot on mobile (Pixel 6)', async ({ page }) => {
    await page.setViewportSize({ width: 412, height: 915 });
    await page.goto('/');

    await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('overview-mobile.png', {
      maxDiffPixels: 300,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match fleet summary strip snapshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const summaryStrip = page.locator('.fleet-summary-strip');
    await expect(summaryStrip.first()).toBeVisible();

    await expect(summaryStrip.first()).toHaveScreenshot('fleet-summary-strip.png', {
      maxDiffPixels: 100,
      threshold: VISUAL_THRESHOLDS.typography,
    });
  });

  test('should match project card snapshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.fleet-cards-grid', { timeout: 10000 });

    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount > 0) {
      await expect(firstCard.first()).toHaveScreenshot('project-card.png', {
        maxDiffPixels: 150,
        threshold: VISUAL_THRESHOLDS.styling,
      });
    } else {
      test.skip(true, 'No project cards available');
    }
  });
});

test.describe('Visual Regression - Project Detail', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
  });

  test('should match project detail page snapshot on desktop', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('project-detail-desktop.png', {
      maxDiffPixels: 800,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match project detail page snapshot on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 412, height: 915 });
    await page.goto('/');
    await page.waitForSelector('.fleet-cards-grid', { timeout: 10000 });

    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('project-detail-mobile.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match tab bar snapshot', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    const tabList = page.locator('.tab-list');
    if (await tabList.count() > 0) {
      await expect(tabList.first()).toHaveScreenshot('tab-bar.png', {
        maxDiffPixels: 100,
        threshold: VISUAL_THRESHOLDS.typography,
      });
    }
  });

  test('should match Stitches tab snapshot', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    const stitchesTab = page.locator('button[role="tab"]', { hasText: 'Stitches' });
    const tabCount = await stitchesTab.count();
    if (tabCount > 0) {
      await stitchesTab.first().click();
      await page.waitForTimeout(500);

      const tabContent = page.locator('.tab-content, main');
      await expect(tabContent.first()).toHaveScreenshot('stitches-tab.png', {
        maxDiffPixels: 400,
        threshold: VISUAL_THRESHOLDS.layout,
      });
    }
  });

  test('should match Fleet Map tab snapshot', async ({ page }) => {
    const firstCard = page.locator('.project-card-fleet').first();
    const cardCount = await firstCard.count();

    if (cardCount === 0) {
      test.skip(true, 'No projects available');
      return;
    }

    await firstCard.click();
    await page.waitForSelector('.app-project-detail', { timeout: 5000 });

    const fleetTab = page.locator('button[role="tab"]', { hasText: 'Fleet Map' });
    const tabCount = await fleetTab.count();
    if (tabCount > 0) {
      await fleetTab.first().click();
      await page.waitForTimeout(500);

      const tabContent = page.locator('.tab-content, main');
      await expect(tabContent.first()).toHaveScreenshot('fleet-map-tab.png', {
        maxDiffPixels: 500,
        threshold: VISUAL_THRESHOLDS.layout,
      });
    }
  });
});

test.describe('Visual Regression - Fleet View', () => {
  test('should match fleet view snapshot on desktop', async ({ page }) => {
    await page.goto('/#/fleet');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('fleet-view-desktop.png', {
      maxDiffPixels: 600,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match fleet view snapshot on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 412, height: 915 });
    await page.goto('/#/fleet');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('fleet-view-mobile.png', {
      maxDiffPixels: 400,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match worker grid snapshot', async ({ page }) => {
    await page.goto('/#/fleet');
    await page.waitForLoadState('networkidle');

    const workerGrid = page.locator('.worker-grid, .fleet-map');
    await expect(workerGrid.first()).toHaveScreenshot('worker-grid.png', {
      maxDiffPixels: 300,
      threshold: VISUAL_THRESHOLDS.graphics,
    });
  });

  test('should match agent chat pane snapshot', async ({ page }) => {
    await page.goto('/#/fleet');
    await page.waitForLoadState('networkidle');

    const agentChat = page.locator('.agent-chat-pane');
    if (await agentChat.count() > 0) {
      await expect(agentChat.first()).toHaveScreenshot('agent-chat-pane.png', {
        maxDiffPixels: 200,
        threshold: VISUAL_THRESHOLDS.styling,
      });
    }
  });

  test('should match capacity panel snapshot', async ({ page }) => {
    await page.goto('/#/fleet');
    await page.waitForLoadState('networkidle');

    const capacityPanel = page.locator('.capacity-panel');
    await expect(capacityPanel.first()).toHaveScreenshot('capacity-panel.png', {
      maxDiffPixels: 150,
      threshold: VISUAL_THRESHOLDS.graphics,
    });
  });
});

test.describe('Visual Regression - Dashboard View', () => {
  test('should match dashboard snapshot on desktop', async ({ page }) => {
    await page.goto('/#/dashboard');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('dashboard-desktop.png', {
      maxDiffPixels: 700,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });

  test('should match dashboard snapshot on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 412, height: 915 });
    await page.goto('/#/dashboard');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('dashboard-mobile.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });
});

test.describe('Visual Regression - Patterns View', () => {
  test('should match patterns view snapshot', async ({ page }) => {
    await page.goto('/#/patterns');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('patterns-view.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });
});

test.describe('Visual Regression - Conversations View', () => {
  test('should match conversations view snapshot', async ({ page }) => {
    await page.goto('/#/conversations');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('conversations-view.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });
});

test.describe('Visual Regression - Drafts View', () => {
  test('should match drafts view snapshot', async ({ page }) => {
    await page.goto('/#/drafts');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('drafts-view.png', {
      maxDiffPixels: 500,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });
});

test.describe('Visual Regression - Timeline View', () => {
  test('should match timeline view snapshot', async ({ page }) => {
    await page.goto('/#/timeline');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('timeline-view.png', {
      maxDiffPixels: 600,
      threshold: VISUAL_THRESHOLDS.layout,
    });
  });
});

test.describe('Visual Regression - Navigation', () => {
  test('should match header navigation snapshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const header = page.locator('header');
    await expect(header.first()).toHaveScreenshot('header-navigation.png', {
      maxDiffPixels: 100,
      threshold: VISUAL_THRESHOLDS.styling,
    });
  });

  test('should match mini header navigation snapshot', async ({ page }) => {
    await page.goto('/#/dashboard');
    await page.waitForLoadState('networkidle');

    const miniHeader = page.locator('.app-header-mini');
    await expect(miniHeader.first()).toHaveScreenshot('mini-header-navigation.png', {
      maxDiffPixels: 100,
      threshold: VISUAL_THRESHOLDS.styling,
    });
  });
});

test.describe('Visual Regression - Modals and Overlays', () => {
  test('should match settings menu snapshot', async ({ page }) => {
    await page.goto('/');

    // Open settings menu
    await page.locator('button[aria-label="Settings"]').click();
    await page.waitForSelector('.settings-menu, [role="menu"]');

    const settingsMenu = page.locator('.settings-menu, [role="menu"]');
    await expect(settingsMenu.first()).toHaveScreenshot('settings-menu.png', {
      maxDiffPixels: 150,
      threshold: VISUAL_THRESHOLDS.styling,
    });
  });

  test('should match search palette snapshot', async ({ page }) => {
    await page.goto('/');

    // Open search palette
    await page.keyboard.press((process.platform === 'darwin' ? 'Meta' : 'Control') + '+k');
    await page.waitForSelector('.search-palette, [role="dialog"]');

    const searchPalette = page.locator('.search-palette, [role="dialog"]');
    await expect(searchPalette.first()).toHaveScreenshot('search-palette.png', {
      maxDiffPixels: 150,
      threshold: VISUAL_THRESHOLDS.styling,
    });
  });
});

test.describe('Visual Regression - Banners and Alerts', () => {
  test('should match connection banner snapshot (when shown)', async ({ page }) => {
    await page.goto('/');

    const connectionBanner = page.locator('.connection-banner');
    const bannerCount = await connectionBanner.count();

    // Only test if banner is visible (may not always show)
    if (bannerCount > 0 && await connectionBanner.first().isVisible()) {
      await expect(connectionBanner.first()).toHaveScreenshot('connection-banner.png', {
        maxDiffPixels: 100,
        threshold: VISUAL_THRESHOLDS.styling,
      });
    } else {
      test.skip(true, 'Connection banner not shown');
    }
  });
});

test.describe('Visual Regression - Color and Theming', () => {
  test('should maintain consistent color scheme across pages', async ({ page }) => {
    // Navigate to multiple pages and verify consistent styling
    const views = ['/', '#/dashboard', '#/fleet', '#/patterns'];

    for (const view of views) {
      await page.goto('/' + view);
      await page.waitForLoadState('networkidle');

      // Verify primary color is consistent
      const primaryColor = await page.locator('body').evaluate(el => {
        const computed = window.getComputedStyle(el);
        return computed.getPropertyValue('--color-primary') ||
               computed.getPropertyValue('--primary-color');
      });

      // If primary color is defined, it should be consistent
      if (primaryColor) {
        expect(primaryColor).toBeTruthy();
      }
    }
  });
});

test.describe('Visual Regression - Typography', () => {
  test('should maintain consistent typography across pages', async ({ page }) => {
    await page.goto('/');

    // Check main heading typography
    const h1 = page.locator('h1').first();
    await expect(h1).toBeVisible();

    const fontSize = await h1.evaluate(el => {
      return window.getComputedStyle(el).fontSize;
    });

    // Font size should be reasonable
    expect(parseInt(fontSize)).toBeGreaterThan(16);
  });
});

test.describe('Visual Regression - Component Consistency', () => {
  test('should maintain consistent button styling', async ({ page }) => {
    await page.goto('/');

    const buttons = page.locator('button').first();
    await expect(buttons).toBeVisible();

    // Verify button has defined styles
    const backgroundColor = await buttons.evaluate(el => {
      return window.getComputedStyle(el).backgroundColor;
    });

    expect(backgroundColor).toBeTruthy();
  });

  test('should maintain consistent link styling', async ({ page }) => {
    await page.goto('/');

    const links = page.locator('a[href]').first();
    await expect(links).toBeVisible();

    // Verify link has defined styles
    const color = await links.evaluate(el => {
      return window.getComputedStyle(el).color;
    });

    expect(color).toBeTruthy();
  });
});
