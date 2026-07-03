# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: smoke-tests.spec.ts >> Smoke Tests - Patterns View >> should load patterns view
- Location: e2e/smoke-tests.spec.ts:307:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('.patterns-view, .patterns-container')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('.patterns-view, .patterns-container')

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
          - link "Dashboard" [ref=e12] [cursor=pointer]:
            - /url: "#/dashboard"
          - link "Drafts" [ref=e13] [cursor=pointer]:
            - /url: "#/drafts"
          - link "Fleet" [ref=e14] [cursor=pointer]:
            - /url: "#/fleet"
          - link "Audit" [ref=e15] [cursor=pointer]:
            - /url: "#/audit"
          - link "Redaction Audit" [ref=e16] [cursor=pointer]:
            - /url: "#/redaction-audit"
        - generic [ref=e17]: Connecting...
    - main [ref=e19]:
      - generic [ref=e20]:
        - generic [ref=e21]:
          - heading "Patterns" [level=2] [ref=e23]
          - paragraph [ref=e24]: Patterns group related stitches toward a shared goal, tracking aggregate progress and cost.
        - generic [ref=e25]: "SyntaxError: Unexpected token '<', \"<!DOCTYPE \"... is not valid JSON"
  - generic [ref=e26]:
    - generic [ref=e27]: 🎤
    - button "Dictation settings" [ref=e28] [cursor=pointer]: ⚙
  - button "Open settings" [ref=e30] [cursor=pointer]:
    - img [ref=e31]
  - dialog "Welcome tour" [ref=e34]:
    - generic [ref=e35]:
      - heading "Welcome to HOOP" [level=2] [ref=e36]
      - button "Close tour" [ref=e37] [cursor=pointer]: ×
    - generic [ref=e38]:
      - paragraph [ref=e39]: HOOP is your AI-powered development companion. Let's take a quick tour.
      - generic [ref=e40]:
        - generic [ref=e41]:
          - heading "Stitches" [level=4] [ref=e42]
          - paragraph [ref=e43]: Conversations and work items — where you collaborate with AI agents to get things done.
        - generic [ref=e44]:
          - heading "Patterns" [level=4] [ref=e45]
          - paragraph [ref=e46]: Reusable workflows that span multiple projects and automate repetitive tasks.
    - button "Next" [ref=e54] [cursor=pointer]
```

# Test source

```ts
  211 | 
  212 |     await firstCard.click();
  213 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  214 | 
  215 |     // Click Files tab
  216 |     const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  217 |     const tabCount = await filesTab.count();
  218 |     if (tabCount > 0) {
  219 |       await filesTab.first().click();
  220 | 
  221 |       // Files tab should be visible
  222 |       await expect(page.locator('.files-tab, .file-browser, .file-tree')).toBeVisible();
  223 |     }
  224 |   });
  225 | 
  226 |   test('should display Debug tab content', async ({ page }) => {
  227 |     const firstCard = page.locator('.project-card-fleet').first();
  228 | 
  229 |     const cardCount = await firstCard.count();
  230 |     if (cardCount === 0) {
  231 |       test.skip(true, 'No projects available');
  232 |       return;
  233 |     }
  234 | 
  235 |     await firstCard.click();
  236 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  237 | 
  238 |     // Click Debug tab
  239 |     const debugTab = page.locator('button[role="tab"]', { hasText: 'Debug' });
  240 |     const tabCount = await debugTab.count();
  241 |     if (tabCount > 0) {
  242 |       await debugTab.first().click();
  243 | 
  244 |       // Debug panel should be visible
  245 |       await expect(page.locator('.debug-panel, .debug-empty')).toBeVisible();
  246 |     }
  247 |   });
  248 | });
  249 | 
  250 | test.describe('Smoke Tests - Fleet View', () => {
  251 |   test('should load fleet view with worker map', async ({ page }) => {
  252 |     await page.goto('/#/fleet');
  253 | 
  254 |     // Fleet map should be visible
  255 |     await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();
  256 | 
  257 |     // Connection indicator
  258 |     await expect(page.locator('.connection-indicator')).toBeVisible();
  259 | 
  260 |     // Back link
  261 |     await expect(page.locator('a.back-link')).toBeVisible();
  262 |   });
  263 | 
  264 |   test('should display agent chat pane', async ({ page }) => {
  265 |     await page.goto('/#/fleet');
  266 | 
  267 |     // Agent chat pane should be visible
  268 |     await expect(page.locator('.agent-chat-pane, .chat-pane')).toBeVisible();
  269 | 
  270 |     // Chat input should be present
  271 |     await expect(page.locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')).toBeAttached();
  272 |   });
  273 | 
  274 |   test('should display capacity panel', async ({ page }) => {
  275 |     await page.goto('/#/fleet');
  276 | 
  277 |     // Capacity panel should be visible
  278 |     await expect(page.locator('.capacity-panel')).toBeVisible();
  279 |   });
  280 | });
  281 | 
  282 | test.describe('Smoke Tests - Cross-Project Dashboard', () => {
  283 |   test('should load dashboard view', async ({ page }) => {
  284 |     await page.goto('/#/dashboard');
  285 | 
  286 |     // Dashboard should be visible
  287 |     await expect(page.locator('.cross-project-dashboard, .dashboard')).toBeVisible();
  288 | 
  289 |     // Navigation
  290 |     await expect(page.locator('a.back-link')).toBeVisible();
  291 |     await expect(page.locator('.connection-indicator')).toBeVisible();
  292 |   });
  293 | 
  294 |   test('should display dashboard metrics', async ({ page }) => {
  295 |     await page.goto('/#/dashboard');
  296 | 
  297 |     // Wait for content to load
  298 |     await page.waitForLoadState('networkidle');
  299 | 
  300 |     // Dashboard should have some content
  301 |     const dashboard = page.locator('.cross-project-dashboard, .dashboard');
  302 |     await expect(dashboard.first()).toBeVisible();
  303 |   });
  304 | });
  305 | 
  306 | test.describe('Smoke Tests - Patterns View', () => {
  307 |   test('should load patterns view', async ({ page }) => {
  308 |     await page.goto('/#/patterns');
  309 | 
  310 |     // Patterns view should be visible
> 311 |     await expect(page.locator('.patterns-view, .patterns-container')).toBeVisible();
      |                                                                       ^ Error: expect(locator).toBeVisible() failed
  312 | 
  313 |     // Navigation
  314 |     await expect(page.locator('a.back-link')).toBeVisible();
  315 |   });
  316 | 
  317 |   test('should display pattern list or empty state', async ({ page }) => {
  318 |     await page.goto('/#/patterns');
  319 | 
  320 |     // Wait for content
  321 |     await page.waitForLoadState('networkidle');
  322 | 
  323 |     // Either pattern list or empty state
  324 |     const patternList = page.locator('.pattern-list, .patterns-container');
  325 |     await expect(patternList.first()).toBeVisible();
  326 |   });
  327 | });
  328 | 
  329 | test.describe('Smoke Tests - Conversations View', () => {
  330 |   test('should load conversations view', async ({ page }) => {
  331 |     await page.goto('/#/conversations');
  332 | 
  333 |     // Conversations view should be visible
  334 |     await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();
  335 | 
  336 |     // Navigation
  337 |     await expect(page.locator('a.back-link')).toBeVisible();
  338 |   });
  339 | 
  340 |   test('should display conversation list or empty state', async ({ page }) => {
  341 |     await page.goto('/#/conversations');
  342 | 
  343 |     // Wait for content
  344 |     await page.waitForLoadState('networkidle');
  345 | 
  346 |     // Conversations container should be visible
  347 |     await expect(page.locator('.conversations-view, .conversations-container')).toBeVisible();
  348 |   });
  349 | });
  350 | 
  351 | test.describe('Smoke Tests - Drafts View', () => {
  352 |   test('should load drafts view', async ({ page }) => {
  353 |     await page.goto('/#/drafts');
  354 | 
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
```