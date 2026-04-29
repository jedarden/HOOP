# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: smoke-tests.spec.ts >> Smoke Tests - Overview Page >> should load overview page with header and navigation
- Location: e2e/smoke-tests.spec.ts:13:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('button[aria-label="Open settings"]')
Expected: visible
Error: strict mode violation: locator('button[aria-label="Open settings"]') resolved to 2 elements:
    1) <button title="Settings" class="settings-trigger" aria-label="Open settings">…</button> aka getByTestId('overview-header').getByRole('button', { name: 'Open settings' })
    2) <button title="Settings" class="settings-trigger" aria-label="Open settings">…</button> aka getByRole('button', { name: 'Open settings' }).nth(1)

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('button[aria-label="Open settings"]')

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
        - heading "HOOP" [level=1] [ref=e10]
        - generic [ref=e11]:
          - button "Open settings" [ref=e13] [cursor=pointer]:
            - img [ref=e14]
          - generic [ref=e16]: Connecting...
      - paragraph [ref=e18]: The operator's pane of glass and conversational handle.
    - main [ref=e19]:
      - generic [ref=e20]:
        - generic [ref=e21]:
          - generic [ref=e22]: "0"
          - generic [ref=e23]: projects
        - generic [ref=e24]:
          - generic [ref=e25]: "0"
          - generic [ref=e26]: workers
        - generic [ref=e27]:
          - generic [ref=e28]: "0"
          - generic [ref=e29]: active stitches
        - generic [ref=e30]:
          - generic [ref=e31]: $0
          - generic [ref=e32]: spend today
      - generic [ref=e33]:
        - generic [ref=e34]:
          - heading "Fleet" [level=2] [ref=e35]
          - generic [ref=e36]:
            - link "Cross-project dashboard →" [ref=e37] [cursor=pointer]:
              - /url: "#/dashboard"
            - link "Live worker map →" [ref=e38] [cursor=pointer]:
              - /url: "#/fleet"
            - link "Worker timeline →" [ref=e39] [cursor=pointer]:
              - /url: "#/timeline"
            - link "Diagnostics →" [ref=e40] [cursor=pointer]:
              - /url: "#/diagnostics"
        - generic [ref=e43]: Loading projects…
  - generic [ref=e44]:
    - generic [ref=e45]: 🎤
    - generic [ref=e46]: ⌘+⇧+D
    - button "Dictation settings" [ref=e47] [cursor=pointer]: ⚙
  - button "Open settings" [ref=e49] [cursor=pointer]:
    - img [ref=e50]
  - dialog "Welcome tour" [ref=e53]:
    - generic [ref=e54]:
      - heading "Welcome to HOOP" [level=2] [ref=e55]
      - button "Close tour" [ref=e56] [cursor=pointer]: ×
    - generic [ref=e57]:
      - paragraph [ref=e58]: HOOP is your AI-powered development companion. Let's take a quick tour.
      - generic [ref=e59]:
        - generic [ref=e60]:
          - heading "Stitches" [level=4] [ref=e61]
          - paragraph [ref=e62]: Conversations and work items — where you collaborate with AI agents to get things done.
        - generic [ref=e63]:
          - heading "Patterns" [level=4] [ref=e64]
          - paragraph [ref=e65]: Reusable workflows that span multiple projects and automate repetitive tasks.
    - button "Next" [ref=e73] [cursor=pointer]
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
  2   | 
  3   | /**
  4   |  * Smoke Tests - Major UI Panels
  5   |  *
  6   |  * Tests that each major panel loads without errors and displays expected content.
  7   |  * Per §14.2 UI tests of plan.md.
  8   |  *
  9   |  * Run: npm run test:e2e smoke-tests.spec.ts
  10  |  */
  11  | 
  12  | test.describe('Smoke Tests - Overview Page', () => {
  13  |   test('should load overview page with header and navigation', async ({ page }) => {
  14  |     await page.goto('/');
  15  | 
  16  |     // Main header
  17  |     await expect(page.locator('h1').filter({ hasText: 'HOOP' })).toBeVisible();
  18  | 
  19  |     // Connection indicator
  20  |     await expect(page.locator('.connection-indicator')).toBeVisible();
  21  | 
  22  |     // Settings menu button
> 23  |     await expect(page.locator('button[aria-label="Open settings"]')).toBeVisible();
      |                                                                      ^ Error: expect(locator).toBeVisible() failed
  24  |   });
  25  | 
  26  |   test('should display fleet summary strip with metrics', async ({ page }) => {
  27  |     await page.goto('/');
  28  | 
  29  |     // Fleet summary strip
  30  |     await expect(page.locator('.fleet-summary-strip')).toBeVisible();
  31  | 
  32  |     // At least projects counter should be visible
  33  |     await expect(page.locator('.fleet-summary-strip').locator('.fss-item').first()).toBeVisible();
  34  |   });
  35  | 
  36  |   test('should display project cards or empty/loading state', async ({ page }) => {
  37  |     await page.goto('/');
  38  | 
  39  |     // Wait for page to stabilize
  40  |     await page.waitForLoadState('networkidle');
  41  | 
  42  |     // Either loading, empty, or cards grid
  43  |     const loading = page.locator('.fleet-loading');
  44  |     const empty = page.locator('.fleet-empty');
  45  |     const cards = page.locator('.fleet-cards-grid');
  46  | 
  47  |     const isLoading = await loading.count() > 0;
  48  |     const isEmpty = await empty.count() > 0;
  49  |     const hasCards = await cards.count() > 0;
  50  | 
  51  |     expect(isLoading || isEmpty || hasCards).toBeTruthy();
  52  |   });
  53  | 
  54  |   test('should have navigation links to cross-project views', async ({ page }) => {
  55  |     await page.goto('/');
  56  | 
  57  |     // Dashboard link
  58  |     await expect(page.locator('a[href="#/dashboard"]')).toBeVisible();
  59  | 
  60  |     // Fleet link
  61  |     await expect(page.locator('a[href="#/fleet"]')).toBeVisible();
  62  | 
  63  |     // Timeline link
  64  |     await expect(page.locator('a[href="#/timeline"]')).toBeVisible();
  65  | 
  66  |     // Diagnostics link
  67  |     await expect(page.locator('a[href="#/diagnostics"]')).toBeVisible();
  68  |   });
  69  | });
  70  | 
  71  | test.describe('Smoke Tests - Project Detail', () => {
  72  |   test.beforeEach(async ({ page }) => {
  73  |     await page.goto('/');
  74  | 
  75  |     // Wait for project cards to load
  76  |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty, .fleet-loading', { timeout: 10000 });
  77  |   });
  78  | 
  79  |   test('should navigate to project detail when clicking a project card', async ({ page }) => {
  80  |     // Find first project card
  81  |     const firstCard = page.locator('.project-card-fleet').first();
  82  | 
  83  |     const cardCount = await firstCard.count();
  84  |     if (cardCount === 0) {
  85  |       test.skip(true, 'No projects available');
  86  |       return;
  87  |     }
  88  | 
  89  |     // Get project name from card
  90  |     const projectName = await firstCard.locator('.pcf-label').textContent();
  91  | 
  92  |     // Click card
  93  |     await firstCard.click();
  94  | 
  95  |     // Should navigate to project detail
  96  |     await expect(page.locator('.app-project-detail')).toBeVisible();
  97  | 
  98  |     // Should have back link
  99  |     await expect(page.locator('a.back-link')).toBeVisible();
  100 |   });
  101 | 
  102 |   test('should display all tab buttons in project detail', async ({ page }) => {
  103 |     const firstCard = page.locator('.project-card-fleet').first();
  104 | 
  105 |     const cardCount = await firstCard.count();
  106 |     if (cardCount === 0) {
  107 |       test.skip(true, 'No projects available');
  108 |       return;
  109 |     }
  110 | 
  111 |     await firstCard.click();
  112 | 
  113 |     // Wait for project detail to load
  114 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  115 | 
  116 |     // Tab list should be visible
  117 |     const tabList = page.locator('.tab-list');
  118 |     if (await tabList.count() > 0) {
  119 |       await expect(tabList).toBeVisible();
  120 | 
  121 |       // Check for standard tabs
  122 |       const expectedTabs = ['Stitches', 'Fleet Map', 'Bead Graph', 'Conversations'];
  123 |       for (const tabName of expectedTabs) {
```