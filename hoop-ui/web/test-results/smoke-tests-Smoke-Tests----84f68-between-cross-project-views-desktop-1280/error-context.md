# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: smoke-tests.spec.ts >> Smoke Tests - Navigation Between Views >> should navigate between cross-project views
- Location: e2e/smoke-tests.spec.ts:453:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('.cross-project-dashboard, .dashboard')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('.cross-project-dashboard, .dashboard')

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - alert [ref=e3]:
    - generic [ref=e4]:
      - generic [ref=e5]: ⟳
      - generic [ref=e6]: Connecting...
  - generic [ref=e7]:
    - banner [ref=e8]:
      - generic [ref=e9]:
        - generic [ref=e10]:
          - link "← All Projects" [ref=e11] [cursor=pointer]:
            - /url: "#/"
          - link "Patterns" [ref=e12] [cursor=pointer]:
            - /url: "#/patterns"
          - link "Conversations" [ref=e13] [cursor=pointer]:
            - /url: "#/conversations"
          - link "Drafts" [ref=e14] [cursor=pointer]:
            - /url: "#/drafts"
          - link "Fleet" [ref=e15] [cursor=pointer]:
            - /url: "#/fleet"
          - link "Timeline" [ref=e16] [cursor=pointer]:
            - /url: "#/timeline"
          - link "Audit" [ref=e17] [cursor=pointer]:
            - /url: "#/audit"
          - link "Redaction Audit" [ref=e18] [cursor=pointer]:
            - /url: "#/redaction-audit"
        - generic [ref=e19]: Connecting...
    - main [ref=e21]:
      - generic [ref=e23]:
        - generic [ref=e24]:
          - heading "Cross-Project Dashboard" [level=2] [ref=e25]
          - tablist "Time range" [ref=e26]:
            - tab "Today" [selected] [ref=e27] [cursor=pointer]
            - tab "Last 7 days" [ref=e28] [cursor=pointer]
            - tab "This month" [ref=e29] [cursor=pointer]
        - alert [ref=e30]: "Error loading dashboard: SyntaxError: Unexpected token '<', \"<!DOCTYPE \"... is not valid JSON"
  - generic [ref=e31]:
    - generic [ref=e32]: 🎤
    - generic [ref=e33]: ⌘+⇧+D
    - button "Dictation settings" [ref=e34] [cursor=pointer]: ⚙
```

# Test source

```ts
  355 |     // Drafts view should be visible
  356 |     await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  357 | 
  358 |     // Navigation
  359 |     await expect(page.locator('a.back-link')).toBeVisible();
  360 |   });
  361 | 
  362 |   test('should display draft list or empty state', async ({ page }) => {
  363 |     await page.goto('/#/drafts');
  364 | 
  365 |     // Wait for content
  366 |     await page.waitForLoadState('networkidle');
  367 | 
  368 |     // Drafts container should be visible
  369 |     await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  370 |   });
  371 | });
  372 | 
  373 | test.describe('Smoke Tests - Timeline View', () => {
  374 |   test('should load timeline view', async ({ page }) => {
  375 |     await page.goto('/#/timeline');
  376 | 
  377 |     // Timeline should be visible
  378 |     await expect(page.locator('.worker-timeline, .timeline-view')).toBeVisible();
  379 | 
  380 |     // Navigation
  381 |     await expect(page.locator('a.back-link')).toBeVisible();
  382 |   });
  383 | });
  384 | 
  385 | test.describe('Smoke Tests - Audit View', () => {
  386 |   test('should load audit view', async ({ page }) => {
  387 |     await page.goto('/#/audit');
  388 | 
  389 |     // Audit panel should be visible
  390 |     await expect(page.locator('.audit-panel, .audit-log')).toBeVisible();
  391 | 
  392 |     // Navigation
  393 |     await expect(page.locator('a.back-link')).toBeVisible();
  394 |   });
  395 | });
  396 | 
  397 | test.describe('Smoke Tests - Redaction Audit View', () => {
  398 |   test('should load redaction audit view', async ({ page }) => {
  399 |     await page.goto('/#/redaction-audit');
  400 | 
  401 |     // Redaction audit panel should be visible
  402 |     await expect(page.locator('.redaction-audit-panel')).toBeVisible();
  403 | 
  404 |     // Navigation
  405 |     await expect(page.locator('a.back-link')).toBeVisible();
  406 |   });
  407 | });
  408 | 
  409 | test.describe('Smoke Tests - Unassigned Sessions View', () => {
  410 |   test('should load unassigned sessions view', async ({ page }) => {
  411 |     await page.goto('/#/unassigned');
  412 | 
  413 |     // Unassigned sessions should be visible
  414 |     await expect(page.locator('.unassigned-sessions, .unassigned-container')).toBeVisible();
  415 | 
  416 |     // Navigation
  417 |     await expect(page.locator('a.back-link')).toBeVisible();
  418 |   });
  419 | });
  420 | 
  421 | test.describe('Smoke Tests - Diagnostics View', () => {
  422 |   test('should load diagnostics view', async ({ page }) => {
  423 |     await page.goto('/#/diagnostics');
  424 | 
  425 |     // Diagnostics should be visible
  426 |     await expect(page.locator('.unknown-events-diagnostics, .diagnostics-container')).toBeVisible();
  427 | 
  428 |     // Navigation
  429 |     await expect(page.locator('a.back-link')).toBeVisible();
  430 |   });
  431 | });
  432 | 
  433 | test.describe('Smoke Tests - Navigation Between Views', () => {
  434 |   test('should navigate from overview to project and back', async ({ page }) => {
  435 |     await page.goto('/');
  436 | 
  437 |     // Wait for projects to load
  438 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });
  439 | 
  440 |     const firstCard = page.locator('.project-card-fleet').first();
  441 |     const cardCount = await firstCard.count();
  442 | 
  443 |     if (cardCount > 0) {
  444 |       await firstCard.click();
  445 |       await expect(page.locator('.app-project-detail')).toBeVisible();
  446 | 
  447 |       // Navigate back
  448 |       await page.locator('a.back-link').first().click();
  449 |       await expect(page.locator('.app:not(.app-project-detail)')).toBeVisible();
  450 |     }
  451 |   });
  452 | 
  453 |   test('should navigate between cross-project views', async ({ page }) => {
  454 |     await page.goto('/#/dashboard');
> 455 |     await expect(page.locator('.cross-project-dashboard, .dashboard')).toBeVisible();
      |                                                                        ^ Error: expect(locator).toBeVisible() failed
  456 | 
  457 |     await page.goto('/#/patterns');
  458 |     await expect(page.locator('.patterns-view, .patterns-container')).toBeVisible();
  459 | 
  460 |     await page.goto('/#/conversations');
  461 |     await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();
  462 | 
  463 |     await page.goto('/#/drafts');
  464 |     await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  465 |   });
  466 | 
  467 |   test('should navigate from overview to fleet and back', async ({ page }) => {
  468 |     await page.goto('/');
  469 | 
  470 |     // Click fleet link
  471 |     await page.locator('a[href="#/fleet"]').first().click();
  472 |     await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();
  473 | 
  474 |     // Navigate back
  475 |     await page.locator('a.back-link').first().click();
  476 |     await expect(page.locator('.fleet-cards-grid, .fleet-empty, .fleet-loading')).toBeAttached();
  477 |   });
  478 | });
  479 | 
  480 | test.describe('Smoke Tests - Search Palette', () => {
  481 |   test('should open search palette with Cmd/Ctrl+K', async ({ page }) => {
  482 |     await page.goto('/');
  483 | 
  484 |     // Press Cmd+K (or Ctrl+K on non-Mac)
  485 |     await page.keyboard.press((process.platform === 'darwin' ? 'Meta' : 'Control') + '+k');
  486 | 
  487 |     // Search palette should be visible
  488 |     await expect(page.locator('.search-palette, [role="dialog"]')).toBeVisible();
  489 |   });
  490 | 
  491 |   test('should close search palette on escape', async ({ page }) => {
  492 |     await page.goto('/');
  493 | 
  494 |     // Open search palette
  495 |     await page.keyboard.press((process.platform === 'darwin' ? 'Meta' : 'Control') + '+k');
  496 |     await expect(page.locator('.search-palette, [role="dialog"]')).toBeVisible();
  497 | 
  498 |     // Close with Escape
  499 |     await page.keyboard.press('Escape');
  500 |     await expect(page.locator('.search-palette, [role="dialog"]')).not.toBeVisible();
  501 |   });
  502 | });
  503 | 
  504 | test.describe('Smoke Tests - Settings Menu', () => {
  505 |   test('should open settings menu', async ({ page }) => {
  506 |     await page.goto('/');
  507 | 
  508 |     // Click settings button
  509 |     await page.locator('button[aria-label="Settings"]').click();
  510 | 
  511 |     // Settings menu should be visible
  512 |     await expect(page.locator('.settings-menu, [role="menu"]')).toBeVisible();
  513 |   });
  514 | 
  515 |   test('should close settings menu on click outside', async ({ page }) => {
  516 |     await page.goto('/');
  517 | 
  518 |     // Open settings
  519 |     await page.locator('button[aria-label="Settings"]').click();
  520 |     await expect(page.locator('.settings-menu, [role="menu"]')).toBeVisible();
  521 | 
  522 |     // Click outside
  523 |     await page.locator('main').click();
  524 |     await expect(page.locator('.settings-menu, [role="menu"]')).not.toBeVisible();
  525 |   });
  526 | });
  527 | 
  528 | test.describe('Smoke Tests - Error Handling', () => {
  529 |   test('should show 404 for non-existent project', async ({ page }) => {
  530 |     await page.goto('/#/non-existent-project-xyz123');
  531 | 
  532 |     // Should show not found message
  533 |     await expect(page.locator('text=/not found/i')).toBeVisible();
  534 |   });
  535 | 
  536 |   test('should handle connection errors gracefully', async ({ page }) => {
  537 |     // This test verifies the UI handles connection state
  538 |     await page.goto('/');
  539 | 
  540 |     // Connection indicator should be present (either connected or connecting)
  541 |     await expect(page.locator('.connection-indicator')).toBeVisible();
  542 |   });
  543 | });
  544 | 
```