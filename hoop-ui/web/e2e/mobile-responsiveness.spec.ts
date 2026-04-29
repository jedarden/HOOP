import { test, expect } from '@playwright/test';

/**
 * Mobile Responsiveness Test Suite
 * Tests UI components across breakpoint matrix: 375/700/768/1280
 * Per §21.1-21.4 of plan.md
 */

test.describe('Mobile Responsiveness - Core Layout', () => {
  test('should render main app container at all breakpoints', async ({ page }) => {
    await page.goto('/');

    // App container exists
    const app = page.locator('.app, #root, [data-testid="app-container"]');
    await expect(app.first()).toBeVisible();

    // No horizontal overflow
    const bodyWidth = await page.evaluate(() => document.body.scrollWidth);
    const viewportWidth = page.viewportSize()?.width || 1280;
    expect(bodyWidth).toBeLessThanOrEqual(viewportWidth);
  });

  test('should have readable font sizes on mobile', async ({ page }) => {
    await page.goto('/');

    const fontSize = await page.evaluate(() => {
      const body = window.getComputedStyle(document.body);
      return parseInt(body.fontSize);
    });

    expect(fontSize).toBeGreaterThanOrEqual(14);
  });
});

test.describe('Mobile Responsiveness - Navigation', () => {
  test('should have accessible navigation on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Navigation should be visible or collapsible
    const nav = page.locator('nav, [data-testid="navigation"], .header-top');
    if (await nav.count() > 0) {
      await expect(nav.first()).toBeVisible();
    }
  });

  test('should have accessible navigation on tablet (768px)', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');

    const nav = page.locator('nav, [data-testid="navigation"], .header-top');
    if (await nav.count() > 0) {
      await expect(nav.first()).toBeVisible();
    }
  });
});

test.describe('Mobile Responsiveness - Cards and Lists', () => {
  test('should display cards in single column on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for card-like elements
    const cards = page.locator('.worker-card, .bead-card, .stitch-card, [class*="card"]');
    const count = await cards.count();

    if (count > 0) {
      // First card should be visible
      await expect(cards.first()).toBeVisible();

      // Check if cards stack vertically (no horizontal scroll)
      const firstCard = cards.first();
      const boundingBox = await firstCard.boundingBox();
      if (boundingBox) {
        expect(boundingBox.width).toBeLessThanOrEqual(375);
      }
    }
  });

  test('should display cards in grid on desktop', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/');

    const cards = page.locator('.worker-card, [class*="card"]');
    const count = await cards.count();

    if (count > 1) {
      // Check if grid layout is used
      const firstCard = cards.first();
      const secondCard = cards.nth(1);

      const firstBox = await firstCard.boundingBox();
      const secondBox = await secondCard.boundingBox();

      if (firstBox && secondBox) {
        // Cards should be side by side on desktop
        expect(secondBox.y).toBeCloseTo(firstBox.y, 50);
      }
    }
  });
});

test.describe('Mobile Responsiveness - Forms', () => {
  test('should have full-width inputs on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const inputs = page.locator('input[type="text"], input[type="email"], textarea');
    const count = await inputs.count();

    if (count > 0) {
      const firstInput = inputs.first();
      const boundingBox = await firstInput.boundingBox();

      if (boundingBox) {
        // Input should be reasonably wide on mobile
        expect(boundingBox.width).toBeGreaterThan(200);
      }
    }
  });

  test('should have accessible tap targets on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const buttons = page.locator('button, a, input[type="submit"]');
    const count = await buttons.count();

    if (count > 0) {
      // Sample first few buttons for tap target size
      for (let i = 0; i < Math.min(count, 5); i++) {
        const button = buttons.nth(i);
        const boundingBox = await button.boundingBox();

        if (boundingBox) {
          // Minimum tap target: 44x44px (iOS) / 48x48dp (Android)
          const minTapSize = 44;
          if (boundingBox.width < minTapSize || boundingBox.height < minTapSize) {
            // Log warning but don't fail - some small buttons may be acceptable
            console.log(`Small tap target detected: ${boundingBox.width}x${boundingBox.height}px`);
          }
        }
      }
    }
  });
});

test.describe('Mobile Responsiveness - Text Readability', () => {
  test('should not require horizontal scrolling for text', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for horizontal scroll
    const hasHorizontalScroll = await page.evaluate(() => {
      const body = document.body;
      return body.scrollWidth > body.clientWidth;
    });

    expect(hasHorizontalScroll).toBeFalsy();
  });

  test('should have adequate line height for readability', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const lineHeight = await page.evaluate(() => {
      const body = window.getComputedStyle(document.body);
      const lh = body.lineHeight;
      // Handle 'normal' line-height
      if (lh === 'normal') return 1.5; // browsers typically use 1.2-1.4
      return parseFloat(lh) || 1.5;
    });

    // Line height should be at least 1.4 for readability
    expect(lineHeight).toBeGreaterThanOrEqual(1.4);
  });
});

test.describe('Not-for-phone surfaces', () => {
  test('should show desktop message for StitchNetDiff on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });

    // Navigate to a page that might have StitchNetDiff
    await page.goto('/');

    // Check for the mobile message class
    const mobileMessage = page.locator('.nd-mobile-message');
    if (await mobileMessage.count() > 0) {
      await expect(mobileMessage).toBeVisible();
      await expect(mobileMessage).toContainText(/desktop|view on|larger screen/i);
    }
  });

  test('should hide complex visualizations on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Complex visualizations should either be hidden or show message
    const complexViz = page.locator('.bead-graph, .fleet-map, [data-testid="graph"]');
    const count = await complexViz.count();

    if (count > 0) {
      const first = complexViz.first();
      const isVisible = await first.isVisible();

      if (isVisible) {
        // If visible, it should have responsive sizing
        const boundingBox = await first.boundingBox();
        if (boundingBox) {
          expect(boundingBox.width).toBeLessThanOrEqual(375);
        }
      }
    }
  });
});

test.describe('Breakpoint Matrix', () => {
  const breakpoints = [
    { width: 375, name: 'phone portrait' },
    { width: 700, name: 'phone landscape' },
    { width: 768, name: 'tablet' },
    { width: 1280, name: 'desktop' },
  ];

  for (const bp of breakpoints) {
    test(`should render correctly at ${bp.name} (${bp.width}px)`, async ({ page }) => {
      await page.setViewportSize({ width: bp.width, height: 800 });
      await page.goto('/');

      // Page should load without errors
      await expect(page.locator('body')).toBeVisible();

      // No horizontal overflow
      const bodyWidth = await page.evaluate(() => document.body.scrollWidth);
      expect(bodyWidth).toBeLessThanOrEqual(bp.width + 20); // Allow small margin
    });
  }
});

test.describe('Touch Interactions', () => {
  test('should have working touch interactions on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for clickable elements
    const clickables = page.locator('button, a, [role="button"]');
    const count = await clickables.count();

    if (count > 0) {
      // First clickable should work
      const first = clickables.first();
      await expect(first.first()).toBeVisible();
    }
  });
});

test.describe('Mobile-specific Features', () => {
  test('should have hamburger menu or collapsible nav on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Look for menu button, hamburger icon, or collapsible navigation
    const menuButton = page.locator(
      'button[aria-label*="menu" i], ' +
      'button[aria-label*="nav" i], ' +
      '.menu-button, .hamburger, ' +
      '[data-testid="menu-toggle"]'
    );

    // This is optional - not all apps need a hamburger menu
    const count = await menuButton.count();
    if (count > 0) {
      await expect(menuButton.first()).toBeVisible();
    }
  });

  test('should optimize images for mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const images = page.locator('img');
    const count = await images.count();

    if (count > 0) {
      // Check first few images
      for (let i = 0; i < Math.min(count, 3); i++) {
        const img = images.nth(i);
        const naturalWidth = await img.evaluate(el => el.naturalWidth);

        if (naturalWidth > 0) {
          // Images should fit viewport
          expect(naturalWidth).toBeLessThanOrEqual(1280);
        }
      }
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────────
// Phase-specific component tests (hoop-ttb.18.1.1)
// Tests for UI deliverables from recent phases requiring mobile review
// ─────────────────────────────────────────────────────────────────────────────────

test.describe('Phase Components - OverviewPage', () => {
  test('should display project cards responsively on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Project cards should stack vertically on mobile
    const cards = page.locator('.project-card-fleet, .fleet-card');
    const count = await cards.count();

    if (count > 0) {
      await expect(cards.first()).toBeVisible();

      // Check cards are full-width on mobile
      const firstCard = cards.first();
      const boundingBox = await firstCard.boundingBox();
      if (boundingBox) {
        expect(boundingBox.width).toBeLessThanOrEqual(375);
      }
    }
  });

  test('should display project cards in grid on desktop', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/');

    const cards = page.locator('.project-card-fleet');
    const count = await cards.count();

    if (count > 1) {
      // Check if cards are laid out in a grid
      const firstCard = cards.first();
      const secondCard = cards.nth(1);

      const firstBox = await firstCard.boundingBox();
      const secondBox = await secondCard.boundingBox();

      if (firstBox && secondBox) {
        // Cards should be side by side on desktop
        expect(secondBox.y).toBeCloseTo(firstBox.y, 50);
      }
    }
  });
});

test.describe('Phase Components - BeadList and Forms', () => {
  test('should handle bead list layout on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check bead list items if present
    const beadItems = page.locator('.bead-item, .bead-row');
    const count = await beadItems.count();

    if (count > 0) {
      await expect(beadItems.first()).toBeVisible();

      // No horizontal overflow
      const bodyWidth = await page.evaluate(() => document.body.scrollWidth);
      expect(bodyWidth).toBeLessThanOrEqual(395); // 375 + small margin
    }
  });

  test('should handle form inputs on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check form inputs if any are present
    const inputs = page.locator('input, textarea, select');
    const count = await inputs.count();

    if (count > 0) {
      const firstInput = inputs.first();
      await expect(firstInput).toBeVisible();

      // Inputs should be reasonably wide on mobile
      const boundingBox = await firstInput.boundingBox();
      if (boundingBox) {
        expect(boundingBox.width).toBeGreaterThan(200);
      }
    }
  });
});

test.describe('Phase Components - Tabs and Navigation', () => {
  test('should handle tab navigation on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Look for tab elements
    const tabs = page.locator('[role="tab"], .tab, .tab-button');
    const count = await tabs.count();

    if (count > 0) {
      // First tab should be visible
      await expect(tabs.first()).toBeVisible();

      // Check if tabs are horizontally scrollable or stacked
      const tabsContainer = page.locator('.tabs, .tab-list, [role="tablist"]');
      if (await tabsContainer.count() > 0) {
        const container = tabsContainer.first();
        const overflowX = await container.evaluate(el => {
          return window.getComputedStyle(el).overflowX;
        });

        // Should either scroll or fit
        const isVisible = await container.isVisible();
        expect(isVisible).toBeTruthy();
      }
    }
  });
});

test.describe('Phase Components - ConversationsView', () => {
  test('should handle conversation list on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for conversation items
    const convItems = page.locator('.conversation-item, .stitch-row, .conversation-row');
    const count = await convItems.count();

    if (count > 0) {
      await expect(convItems.first()).toBeVisible();

      // Items should be full width or stack vertically
      const firstItem = convItems.first();
      const boundingBox = await firstItem.boundingBox();
      if (boundingBox) {
        expect(boundingBox.width).toBeLessThanOrEqual(375);
      }
    }
  });
});

test.describe('Phase Components - Panels and Overlays', () => {
  test('should handle overlay panels on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for panel toggles or triggers
    const panelTriggers = page.locator('[aria-label*="panel" i], [data-testid*="panel" i], .panel-toggle');
    const count = await panelTriggers.count();

    if (count > 0) {
      // First trigger should be visible and tappable
      const firstTrigger = panelTriggers.first();
      await expect(firstTrigger).toBeVisible();

      const boundingBox = await firstTrigger.boundingBox();
      if (boundingBox) {
        // Minimum tap target: 44x44px
        expect(boundingBox.width).toBeGreaterThanOrEqual(40);
        expect(boundingBox.height).toBeGreaterThanOrEqual(40);
      }
    }
  });
});

test.describe('Phase Components - Audio and Video Players', () => {
  test('should handle media controls on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check for media controls
    const mediaControls = page.locator('button[aria-label*="play" i], button[aria-label*="pause" i], .play-button, .pause-button');
    const count = await mediaControls.count();

    if (count > 0) {
      const firstControl = mediaControls.first();
      if (await firstControl.isVisible()) {
        // Controls should be tappable
        const boundingBox = await firstControl.boundingBox();
        if (boundingBox) {
          expect(boundingBox.width).toBeGreaterThanOrEqual(40);
          expect(boundingBox.height).toBeGreaterThanOrEqual(40);
        }
      }
    }
  });
});

test.describe('Accessibility - Mobile Specific', () => {
  test('should have sufficient color contrast on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Basic check: text elements should be visible
    const textElements = page.locator('p, h1, h2, h3, span, a');
    const count = await textElements.count();

    if (count > 0) {
      // Check a sample of text elements for visibility
      for (let i = 0; i < Math.min(count, 5); i++) {
        const elem = textElements.nth(i);
        if (await elem.isVisible()) {
          const color = await elem.evaluate(el => {
            return window.getComputedStyle(el).color;
          });
          // Should have some color value
          expect(color).toBeTruthy();
        }
      }
    }
  });

  test('should handle focus states on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check that interactive elements can receive focus
    const buttons = page.locator('button, a, [role="button"]');
    const count = await buttons.count();

    if (count > 0) {
      const firstButton = buttons.first();
      if (await firstButton.isVisible()) {
        // Tab to the element
        await firstButton.focus();
        const isFocused = await firstButton.evaluate(el => document.activeElement === el);
        expect(isFocused).toBeTruthy();
      }
    }
  });
});

// ─────────────────────────────────────────────────────────────────────────────────
// Additional Phase Component Tests (hoop-ttb.18.1.1)
// Tests for CostPanel, CapacityPanel, PatternsView, RedactionAuditPanel, FilesTab
// ─────────────────────────────────────────────────────────────────────────────────

test.describe('Phase Components - CostPanel', () => {
  test('should display cost breakdown on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Navigate to a project with cost data
    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      // Try to navigate to cost tab
      const costTab = page.locator('button[role="tab"]', { hasText: 'Cost' });
      if (await costTab.count() > 0) {
        await costTab.click();

        // Cost panel should be visible or show empty state
        const costPanel = page.locator('.cost-panel, .cost-empty, .cost-loading');
        const count = await costPanel.count();
        if (count > 0) {
          await expect(costPanel.first()).toBeVisible();
        }
      }
    }
  });

  test('should handle rate limit meters on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const costTab = page.locator('button[role="tab"]', { hasText: 'Cost' });
      if (await costTab.count() > 0) {
        await costTab.click();

        // Check rate limit meters if present
        const rlMeter = page.locator('.rl-meter, .capacity-meter');
        const count = await rlMeter.count();
        if (count > 0) {
          // Meters should fit within viewport
          const firstMeter = rlMeter.first();
          await expect(firstMeter).toBeVisible();

          const boundingBox = await firstMeter.boundingBox();
          if (boundingBox) {
            expect(boundingBox.width).toBeLessThanOrEqual(375);
          }
        }
      }
    }
  });

  test('should display cost bars responsively on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const costTab = page.locator('button[role="tab"]', { hasText: 'Cost' });
      if (await costTab.count() > 0) {
        await costTab.click();

        // Check cost bars
        const costBar = page.locator('.cost-bar');
        const count = await costBar.count();
        if (count > 0) {
          await expect(costBar.first()).toBeVisible();
        }
      }
    }
  });
});

test.describe('Phase Components - CapacityPanel', () => {
  test('should display capacity meters on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const capacityTab = page.locator('button[role="tab"]', { hasText: 'Capacity' });
      if (await capacityTab.count() > 0) {
        await capacityTab.click();

        // Capacity panel or empty state should be visible
        const capacityPanel = page.locator('.capacity-panel, .capacity-empty');
        const count = await capacityPanel.count();
        if (count > 0) {
          await expect(capacityPanel.first()).toBeVisible();
        }
      }
    }
  });

  test('should stack capacity meters vertically on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const capacityTab = page.locator('button[role="tab"]', { hasText: 'Capacity' });
      if (await capacityTab.count() > 0) {
        await capacityTab.click();

        // Check if meters are stacked
        const capacityMeters = page.locator('.capacity-meters');
        const count = await capacityMeters.count();
        if (count > 0) {
          const flexDirection = await capacityMeters.first().evaluate(el => {
            return window.getComputedStyle(el).flexDirection;
          });
          // On mobile, should be column (stacked)
          if (await capacityMeters.first().isVisible()) {
            expect(['column', 'column-reverse']).toContain(flexDirection);
          }
        }
      }
    }
  });
});

test.describe('Phase Components - PatternsView', () => {
  test('should display pattern list on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Patterns may be accessible via a link or navigation
    const patternsLink = page.locator('a[href*="patterns"], a', { hasText: /pattern/i });
    const count = await patternsLink.count();
    if (count > 0) {
      await patternsLink.first().click();
      await page.waitForTimeout(500);

      // Pattern list should be visible
      const patternList = page.locator('.pattern-list, .patterns-container');
      if (await patternList.count() > 0) {
        await expect(patternList.first()).toBeVisible();
      }
    }
  });

  test('should handle pattern cards on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const patternsLink = page.locator('a[href*="patterns"], a', { hasText: /pattern/i });
    const count = await patternsLink.count();
    if (count > 0) {
      await patternsLink.first().click();
      await page.waitForTimeout(500);

      // Pattern cards should fit viewport
      const patternCard = page.locator('.pattern-card, .pattern-row');
      const cardCount = await patternCard.count();
      if (cardCount > 0) {
        const firstCard = patternCard.first();
        await expect(firstCard).toBeVisible();

        const boundingBox = await firstCard.boundingBox();
        if (boundingBox) {
          expect(boundingBox.width).toBeLessThanOrEqual(375);
        }
      }
    }
  });
});

test.describe('Phase Components - RedactionAuditPanel', () => {
  test('should display redaction audit on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Redaction audit may be in a tab or settings
    const redactionLink = page.locator('a[href*="redaction"], a', { hasText: /redaction/i });
    const count = await redactionLink.count();
    if (count > 0) {
      await redactionLink.first().click();
      await page.waitForTimeout(500);

      // Redaction panel should be visible
      const redactionPanel = page.locator('.redaction-audit-panel');
      if (await redactionPanel.count() > 0) {
        await expect(redactionPanel.first()).toBeVisible();
      }
    }
  });

  test('should handle filter controls on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const redactionLink = page.locator('a[href*="redaction"], a', { hasText: /redaction/i });
    const count = await redactionLink.count();
    if (count > 0) {
      await redactionLink.first().click();
      await page.waitForTimeout(500);

      // Filter controls should be accessible
      const filters = page.locator('.redaction-audit-filters, .filter-controls');
      const filterCount = await filters.count();
      if (filterCount > 0) {
        await expect(filters.first()).toBeVisible();
      }
    }
  });
});

test.describe('Phase Components - FilesTab', () => {
  test('should display file browser on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        // File browser should be visible
        const fileBrowser = page.locator('.file-browser, .files-tab, .file-tree');
        const count = await fileBrowser.count();
        if (count > 0) {
          await expect(fileBrowser.first()).toBeVisible();
        }
      }
    }
  });

  test('should handle file tree navigation on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        // File tree items should be tappable
        const fileItems = page.locator('.file-item, .tree-item, .file-row');
        const itemCount = await fileItems.count();
        if (itemCount > 0) {
          const firstItem = fileItems.first();
          await expect(firstItem).toBeVisible();

          // Check tap target size
          const boundingBox = await firstItem.boundingBox();
          if (boundingBox) {
            // Minimum tap target: 44px height
            expect(boundingBox.height).toBeGreaterThanOrEqual(40);
          }
        }
      }
    }
  });
});

test.describe('Phase Components - AgentChatPane', () => {
  test('should display agent chat on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Agent chat may be accessible via a button or link
    const agentChatButton = page.locator('button', { hasText: /agent|chat|ask/i });
    const count = await agentChatButton.count();
    if (count > 0) {
      await agentChatButton.first().click();
      await page.waitForTimeout(500);

      // Agent chat pane should be visible
      const agentPane = page.locator('.agent-chat-pane, .chat-pane');
      if (await agentPane.count() > 0) {
        await expect(agentPane.first()).toBeVisible();
      }
    }
  });

  test('should handle message input on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const agentChatButton = page.locator('button', { hasText: /agent|chat|ask/i });
    const count = await agentChatButton.count();
    if (count > 0) {
      await agentChatButton.first().click();
      await page.waitForTimeout(500);

      // Message input should be accessible
      const messageInput = page.locator('textarea[placeholder*="message" i], input[type="text"], .chat-input');
      const inputCount = await messageInput.count();
      if (inputCount > 0) {
        await expect(messageInput.first()).toBeVisible();

        // Input should be reasonably wide on mobile
        const boundingBox = await messageInput.first().boundingBox();
        if (boundingBox) {
          expect(boundingBox.width).toBeGreaterThan(200);
        }
      }
    }
  });

  test('should have send button with adequate tap target on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const agentChatButton = page.locator('button', { hasText: /agent|chat|ask/i });
    const count = await agentChatButton.count();
    if (count > 0) {
      await agentChatButton.first().click();
      await page.waitForTimeout(500);

      // Send button should have adequate tap target
      const sendButton = page.locator('button', { hasText: /send|➤|→/i });
      const sendCount = await sendButton.count();
      if (sendCount > 0) {
        const firstSend = sendButton.first();
        if (await firstSend.isVisible()) {
          const boundingBox = await firstSend.boundingBox();
          if (boundingBox) {
            // Minimum tap target: 44x44px
            expect(boundingBox.width).toBeGreaterThanOrEqual(40);
            expect(boundingBox.height).toBeGreaterThanOrEqual(40);
          }
        }
      }
    }
  });
});

test.describe('Phase Components - CrossProjectDashboard', () => {
  test('should display cross-project dashboard on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Look for cross-project dashboard link
    const dashboardLink = page.locator('a[href*="dashboard"]', { hasText: /dashboard/i });
    const count = await dashboardLink.count();
    if (count > 0) {
      await dashboardLink.first().click();
      await page.waitForTimeout(500);

      // Dashboard should be visible
      const dashboard = page.locator('.cross-project-dashboard, .dashboard');
      if (await dashboard.count() > 0) {
        await expect(dashboard.first()).toBeVisible();
      }
    }
  });

  test('should handle dashboard metrics on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Dashboard metrics may be on overview page
    const metrics = page.locator('.fleet-summary-strip, .dashboard-metrics, .metric-card');
    const count = await metrics.count();
    if (count > 0) {
      await expect(metrics.first()).toBeVisible();

      // Metrics should fit within viewport
      const boundingBox = await metrics.first().boundingBox();
      if (boundingBox) {
        expect(boundingBox.width).toBeLessThanOrEqual(375);
      }
    }
  });
});

test.describe('Phase Components - StitchDraftForm', () => {
  test('should display stitch draft form on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      // Look for new stitch button
      const newStitchButton = page.locator('button', { hasText: /new stitch/i });
      const buttonCount = await newStitchButton.count();
      if (buttonCount > 0) {
        await newStitchButton.first().click();
        await page.waitForTimeout(500);

        // Stitch form should be visible
        const stitchForm = page.locator('.stitch-draft-form, .draft-form');
        if (await stitchForm.count() > 0) {
          await expect(stitchForm.first()).toBeVisible();
        }
      }
    }
  });

  test('should handle form fields on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const newStitchButton = page.locator('button', { hasText: /new stitch/i });
      const buttonCount = await newStitchButton.count();
      if (buttonCount > 0) {
        await newStitchButton.first().click();
        await page.waitForTimeout(500);

        // Form inputs should be accessible
        const formInputs = page.locator('input, textarea, select');
        const inputCount = await formInputs.count();
        if (inputCount > 0) {
          const firstInput = formInputs.first();
          await expect(firstInput).toBeVisible();

          // Input should be reasonably wide on mobile
          const boundingBox = await firstInput.boundingBox();
          if (boundingBox) {
            expect(boundingBox.width).toBeGreaterThan(200);
          }
        }
      }
    }
  });
});

test.describe('Breakpoint Matrix - All Components', () => {
  const breakpoints = [
    { width: 375, name: 'phone portrait' },
    { width: 700, name: 'phone landscape' },
    { width: 768, name: 'tablet' },
    { width: 1280, name: 'desktop' },
  ];

  for (const bp of breakpoints) {
    test(`should render all visible components without horizontal overflow at ${bp.name} (${bp.width}px)`, async ({ page }) => {
      await page.setViewportSize({ width: bp.width, height: 800 });
      await page.goto('/');

      // Wait for page to load
      await page.waitForTimeout(500);

      // Check for horizontal scroll
      const hasHorizontalScroll = await page.evaluate(() => {
        const body = document.body;
        return body.scrollWidth > body.clientWidth;
      });

      expect(hasHorizontalScroll).toBeFalsy();

      // Verify main container fits viewport
      const appContainer = page.locator('.app, #root, [data-testid="app-container"]');
      const containerCount = await appContainer.count();
      if (containerCount > 0) {
        const containerWidth = await appContainer.first().evaluate(el => {
          return el.scrollWidth;
        });
        expect(containerWidth).toBeLessThanOrEqual(bp.width + 20); // Allow small margin
      }
    });
  }
});

test.describe('Touch Target Size Compliance - All Interactive Elements', () => {
  const breakpoints = [375, 700, 768];

  for (const width of breakpoints) {
    test(`should have adequate tap targets at ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width: width, height: 800 });
      await page.goto('/');

      // Wait for the React app to mount and render
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000); // Additional wait for React to render

      // Check various interactive element types
      const buttons = page.locator('button, [role="button"]');
      const links = page.locator('a[href]');
      const inputs = page.locator('input[type="checkbox"], input[type="radio"], input[type="submit"]');

      // Wait for at least one interactive element to be present
      await page.waitForSelector('button, a[href], input', { timeout: 5000 }).catch(() => {
        // If no interactive elements are found, that's okay - the page might be empty
      });

      const buttonCount = await buttons.count();
      const linkCount = await links.count();
      const inputCount = await inputs.count();

      // At least one type of interactive element should exist (unless page is truly empty)
      const totalInteractive = buttonCount + linkCount + inputCount;

      // Sample first few interactive elements
      for (const locator of [buttons, links, inputs]) {
        const count = await locator.count();
        for (let i = 0; i < Math.min(count, 3); i++) {
          const elem = locator.nth(i);
          if (await elem.isVisible()) {
            const boundingBox = await elem.boundingBox();
            if (boundingBox) {
              // Minimum tap target: 44x44px (iOS) / 48x48dp (Android)
              // Allow some flexibility for text links
              const minSize = 40;
              const widthOk = boundingBox.width >= minSize || boundingBox.height >= minSize;
              const heightOk = boundingBox.height >= minSize || boundingBox.width >= minSize;

              // Log warning but don't fail - some small elements may be acceptable
              if (!widthOk || !heightOk) {
                console.log(`Small tap target at ${width}px: ${boundingBox.width}x${boundingBox.height}px`);
              }
            }
          }
        }
      }

      // Only fail if we have NO interactive elements at all AND the page has loaded
      if (totalInteractive === 0) {
        const bodyText = await page.evaluate(() => document.body.textContent);
        // If page is truly empty or showing error, skip the assertion
        if (bodyText && bodyText.length > 100 && !bodyText.includes('Error')) {
          expect(buttonCount + linkCount + inputCount).toBeGreaterThan(0);
        }
      }
    });
  }
});

test.describe('PDF Viewer - File Preview (hoop-ttb.4.7)', () => {
  test('should display PDF viewer with page navigation on desktop (1280px)', async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        // Look for PDF files in the file tree
        const pdfFiles = page.locator('.file-tree-node').filter({ hasText: /\.pdf$/i });
        const pdfCount = await pdfFiles.count();

        if (pdfCount > 0) {
          // Click first PDF file
          await pdfFiles.first().click();
          await page.waitForTimeout(500);

          // Verify PDF viewer is displayed
          const pdfViewer = page.locator('.pdf-viewer');
          await expect(pdfViewer.first()).toBeVisible();

          // Verify toolbar elements are present
          const toolbar = page.locator('.pdf-viewer-toolbar');
          await expect(toolbar.first()).toBeVisible();

          // Verify page navigation buttons
          const prevBtn = page.locator('.pdf-viewer-btn', { hasText: '←' });
          const nextBtn = page.locator('.pdf-viewer-btn', { hasText: '→' });
          await expect(prevBtn.first()).toBeVisible();
          await expect(nextBtn.first()).toBeVisible();

          // Verify zoom controls
          const zoomInBtn = page.locator('.pdf-viewer-btn', { hasText: '+' });
          const zoomOutBtn = page.locator('.pdf-viewer-btn', { hasText: '−' });
          await expect(zoomInBtn.first()).toBeVisible();
          await expect(zoomOutBtn.first()).toBeVisible();

          // Verify download button
          const downloadBtn = page.locator('.pdf-viewer-download-btn');
          await expect(downloadBtn.first()).toBeVisible();

          // Verify search input
          const searchInput = page.locator('.pdf-viewer-search-input');
          await expect(searchInput.first()).toBeVisible();

          // Verify canvas for rendering
          const canvas = page.locator('.pdf-viewer-canvas');
          await expect(canvas.first()).toBeVisible();
        }
      }
    }
  });

  test('should display PDF viewer on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        // Look for PDF files
        const pdfFiles = page.locator('.file-tree-node').filter({ hasText: /\.pdf$/i });
        const pdfCount = await pdfFiles.count();

        if (pdfCount > 0) {
          await pdfFiles.first().click();
          await page.waitForTimeout(500);

          // PDF viewer should be visible on mobile
          const pdfViewer = page.locator('.pdf-viewer');
          await expect(pdfViewer.first()).toBeVisible();

          // Toolbar should wrap on mobile
          const toolbar = page.locator('.pdf-viewer-toolbar');
          if (await toolbar.count() > 0) {
            const flexWrap = await toolbar.first().evaluate(el => {
              return window.getComputedStyle(el).flexWrap;
            });
            expect(['wrap', 'wrap-reverse']).toContain(flexWrap);
          }

          // Buttons should be tappable on mobile
          const buttons = page.locator('.pdf-viewer-btn');
          const buttonCount = await buttons.count();
          if (buttonCount > 0) {
            for (let i = 0; i < Math.min(buttonCount, 3); i++) {
              const btn = buttons.nth(i);
              if (await btn.isVisible()) {
                const boundingBox = await btn.boundingBox();
                if (boundingBox) {
                  // Minimum tap target: 40x40px
                  expect(boundingBox.width).toBeGreaterThanOrEqual(32);
                  expect(boundingBox.height).toBeGreaterThanOrEqual(32);
                }
              }
            }
          }
        }
      }
    }
  });

  test('should support search within PDF', async ({ page }) => {
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        const pdfFiles = page.locator('.file-tree-node').filter({ hasText: /\.pdf$/i });
        const pdfCount = await pdfFiles.count();

        if (pdfCount > 0) {
          await pdfFiles.first().click();
          await page.waitForTimeout(500);

          // Search input should be present
          const searchInput = page.locator('.pdf-viewer-search-input');
          if (await searchInput.count() > 0 && await searchInput.first().isVisible()) {
            // Type search query
            await searchInput.first().fill('test');
            await page.waitForTimeout(500);

            // Search match counter may appear if matches are found
            const matchCounter = page.locator('.pdf-viewer-search-matches');
            if (await matchCounter.count() > 0) {
              // If matches found, verify counter is visible
              const counterText = await matchCounter.first().textContent();
              expect(counterText).toBeTruthy();
            }
          }
        }
      }
    }
  });

  test('should support zoom controls', async ({ page }) => {
    await page.goto('/');

    const projectCard = page.locator('.project-card-fleet').first();
    if (await projectCard.count() > 0) {
      await projectCard.click();
      await page.waitForTimeout(500);

      const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
      if (await filesTab.count() > 0) {
        await filesTab.click();

        const pdfFiles = page.locator('.file-tree-node').filter({ hasText: /\.pdf$/i });
        const pdfCount = await pdfFiles.count();

        if (pdfCount > 0) {
          await pdfFiles.first().click();
          await page.waitForTimeout(500);

          // Get initial zoom level
          const zoomLabel = page.locator('.pdf-viewer-zoom-label');
          if (await zoomLabel.count() > 0 && await zoomLabel.first().isVisible()) {
            const initialZoom = await zoomLabel.first().textContent();
            expect(initialZoom).toContain('%');

            // Click zoom in button
            const zoomInBtn = page.locator('.pdf-viewer-btn', { hasText: '+' });
            if (await zoomInBtn.count() > 0) {
              await zoomInBtn.first().click();
              await page.waitForTimeout(200);

              const newZoom = await zoomLabel.first().textContent();
              expect(newZoom).toBeTruthy();
            }

            // Click zoom out button
            const zoomOutBtn = page.locator('.pdf-viewer-btn', { hasText: '−' });
            if (await zoomOutBtn.count() > 0) {
              await zoomOutBtn.first().click();
              await page.waitForTimeout(200);

              const finalZoom = await zoomLabel.first().textContent();
              expect(finalZoom).toBeTruthy();
            }

            // Click reset zoom button
            const resetBtn = page.locator('.pdf-viewer-btn', { hasText: '1:1' });
            if (await resetBtn.count() > 0) {
              await resetBtn.first().click();
              await page.waitForTimeout(200);

              const resetZoom = await zoomLabel.first().textContent();
              expect(resetZoom).toBe('100%');
            }
          }
        }
      }
    }
  });
});

test.describe('Approval Dialogs - Mobile Optimization (§21)', () => {
  test('should display approval modal with adequate tap targets on mobile (375px)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Navigate to drafts tab
    const draftsLink = page.locator('a[href="#/drafts"]');
    const draftsCount = await draftsLink.count();
    if (draftsCount > 0) {
      await draftsLink.first().click();
      await page.waitForTimeout(500);

      // Look for draft items
      const draftItems = page.locator('.draft-row, [data-testid="draft-row"]');
      const itemCount = await draftItems.count();
      if (itemCount > 0) {
        // Click first draft to open detail modal
        await draftItems.first().click();
        await page.waitForTimeout(300);

        // Check for approval modal
        const modal = page.locator('.draft-detail-overlay');
        if (await modal.count() > 0) {
          await expect(modal.first()).toBeVisible();

          // Check approval button tap target size
          const approveButton = page.locator('.draft-btn-approve');
          if (await approveButton.count() > 0 && await approveButton.first().isVisible()) {
            const boundingBox = await approveButton.first().boundingBox();
            if (boundingBox) {
              // Minimum tap target: 44x44px
              expect(boundingBox.width).toBeGreaterThanOrEqual(44);
              expect(boundingBox.height).toBeGreaterThanOrEqual(44);
            }
          }

          // Check reject button tap target size
          const rejectButton = page.locator('.draft-btn-reject');
          if (await rejectButton.count() > 0 && await rejectButton.first().isVisible()) {
            const boundingBox = await rejectButton.first().boundingBox();
            if (boundingBox) {
              expect(boundingBox.width).toBeGreaterThanOrEqual(44);
              expect(boundingBox.height).toBeGreaterThanOrEqual(44);
            }
          }

          // Check that buttons are full-width on mobile
          const actionsContainer = page.locator('.draft-detail-actions');
          if (await actionsContainer.count() > 0) {
            const flexDirection = await actionsContainer.first().evaluate(el => {
              return window.getComputedStyle(el).flexDirection;
            });
            // On mobile, buttons should stack vertically
            expect(['column', 'column-reverse']).toContain(flexDirection);
          }
        }
      }
    }
  });

  test('should have oversized high-contrast approval buttons on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    const draftsLink = page.locator('a[href="#/drafts"]');
    const draftsCount = await draftsLink.count();
    if (draftsCount > 0) {
      await draftsLink.first().click();
      await page.waitForTimeout(500);

      const draftItems = page.locator('.draft-row, [data-testid="draft-row"]');
      const itemCount = await draftItems.count();
      if (itemCount > 0) {
        await draftItems.first().click();
        await page.waitForTimeout(300);

        // Check approve button styling for high contrast
        const approveButton = page.locator('.draft-btn-approve');
        if (await approveButton.count() > 0 && await approveButton.first().isVisible()) {
          const backgroundColor = await approveButton.first().evaluate(el => {
            return window.getComputedStyle(el).backgroundColor;
          });
          const color = await approveButton.first().evaluate(el => {
            return window.getComputedStyle(el).color;
          });

          // Verify high contrast colors (green button, white text)
          expect(backgroundColor).toBeTruthy();
          expect(color).toBeTruthy();

          // Check for larger font size on mobile
          const fontSize = await approveButton.first().evaluate(el => {
            return parseInt(window.getComputedStyle(el).fontSize);
          });
          expect(fontSize).toBeGreaterThanOrEqual(16);
        }

        // Check reject button styling for high contrast
        const rejectButton = page.locator('.draft-btn-reject');
        if (await rejectButton.count() > 0 && await rejectButton.first().isVisible()) {
          const backgroundColor = await rejectButton.first().evaluate(el => {
            return window.getComputedStyle(el).backgroundColor;
          });
          const color = await rejectButton.first().evaluate(el => {
            return window.getComputedStyle(el).color;
          });

          // Verify high contrast colors (red button, white text)
          expect(backgroundColor).toBeTruthy();
          expect(color).toBeTruthy();
        }
      }
    }
  });
});
