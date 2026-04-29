# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: smoke-tests.spec.ts >> Smoke Tests - Drafts View >> should display draft list or empty state
- Location: e2e/smoke-tests.spec.ts:362:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('.drafts-tab, .drafts-container')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('.drafts-tab, .drafts-container')

```

# Test source

```ts
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
  311 |     await expect(page.locator('.patterns-view, .patterns-container')).toBeVisible();
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
> 369 |     await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
      |                                                                  ^ Error: expect(locator).toBeVisible() failed
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
  455 |     await expect(page.locator('.cross-project-dashboard, .dashboard')).toBeVisible();
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
```