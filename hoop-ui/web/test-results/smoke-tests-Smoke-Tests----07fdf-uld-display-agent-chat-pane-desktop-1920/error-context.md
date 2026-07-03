# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: smoke-tests.spec.ts >> Smoke Tests - Fleet View >> should display agent chat pane
- Location: e2e/smoke-tests.spec.ts:264:3

# Error details

```
Error: expect(locator).toBeAttached() failed

Locator: locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')
Expected: attached
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeAttached" with timeout 5000ms
  - waiting for locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')

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
          - link "Drafts" [ref=e12] [cursor=pointer]:
            - /url: "#/drafts"
          - link "Worker Timeline →" [ref=e13] [cursor=pointer]:
            - /url: "#/timeline"
          - link "Audit Log →" [ref=e14] [cursor=pointer]:
            - /url: "#/audit"
          - link "Redaction Audit →" [ref=e15] [cursor=pointer]:
            - /url: "#/redaction-audit"
        - generic [ref=e16]: Connecting...
    - main [ref=e18]:
      - generic [ref=e19]:
        - heading "Fleet Map" [level=2] [ref=e20]
        - paragraph [ref=e22]: No workers detected. Waiting for heartbeats...
      - generic [ref=e23]:
        - generic [ref=e24]:
          - heading "Beads (0)" [level=2] [ref=e25]
          - generic [ref=e26] [cursor=pointer]:
            - checkbox "Expert Mode" [ref=e27]
            - text: Expert Mode
        - paragraph [ref=e29]: No beads found. Beads will appear here as they are created.
      - generic [ref=e31]:
        - generic [ref=e32]:
          - generic [ref=e33]:
            - heading "Conversations" [level=2] [ref=e34]
            - generic [ref=e35]:
              - button "All (0)" [ref=e36] [cursor=pointer]
              - button "Fleet" [ref=e37] [cursor=pointer]
              - button "Operator" [ref=e38] [cursor=pointer]
              - button "Ad-hoc" [ref=e39] [cursor=pointer]
              - button "Dictated" [ref=e40] [cursor=pointer]
          - paragraph [ref=e43]: No conversations found.
        - paragraph [ref=e46]: Select a conversation to view its transcript.
      - region "Agent Chat" [ref=e47]:
        - generic [ref=e48]:
          - generic [ref=e49]:
            - heading "Agent Chat" [level=2] [ref=e50]
            - generic "No active session" [ref=e52]:
              - generic [ref=e54]: Not configured
          - group "Project scope" [ref=e55]:
            - generic [ref=e56]: Scope
            - button "All Projects" [pressed] [ref=e58] [cursor=pointer]
        - log "Conversation" [ref=e59]:
          - generic "No messages yet" [ref=e60]: Agent not configured in hoop config.
        - generic [ref=e62]:
          - button "Attach file" [disabled] [ref=e63]:
            - img [ref=e64]
          - textbox "Message input" [disabled] [ref=e66]:
            - /placeholder: Agent not configured
          - button "Send message" [disabled] [ref=e67]:
            - img [ref=e68]
      - generic [ref=e71]:
        - generic [ref=e72]:
          - heading "Capacity" [level=3] [ref=e73]
          - generic [ref=e74]:
            - generic [ref=e75]:
              - generic [ref=e76]: Active Workers
              - generic [ref=e77]: "0"
            - generic [ref=e78]:
              - generic [ref=e79]: Accounts
              - generic [ref=e80]: "0"
        - generic [ref=e81]:
          - paragraph [ref=e82]: No capacity data available yet
          - generic [ref=e83]:
            - generic [ref=e86]: <40%
            - generic [ref=e89]: 40-70%
            - generic [ref=e92]: 70-90%
            - generic [ref=e95]: ">90%"
            - generic [ref=e96]:
              - generic [ref=e97]: ▼
              - generic [ref=e98]: Forecast
          - generic [ref=e99]:
            - paragraph [ref=e100]:
              - strong [ref=e101]: "Observation only:"
              - text: HOOP displays capacity but does not enforce limits.
            - paragraph [ref=e102]:
              - strong [ref=e103]: Forecast arrows
              - text: show when the limit will be reached at current burn rate.
  - generic [ref=e104]:
    - generic [ref=e105]: 🎤
    - generic [ref=e106]: ⌘+⇧+D
    - button "Dictation settings" [ref=e107] [cursor=pointer]: ⚙
```

# Test source

```ts
  171 |     if (tabCount > 0) {
  172 |       await fleetTab.first().click();
  173 | 
  174 |       // Fleet Map should be visible
  175 |       await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();
  176 |     }
  177 |   });
  178 | 
  179 |   test('should display Cost tab content', async ({ page }) => {
  180 |     const firstCard = page.locator('.project-card-fleet').first();
  181 | 
  182 |     const cardCount = await firstCard.count();
  183 |     if (cardCount === 0) {
  184 |       test.skip(true, 'No projects available');
  185 |       return;
  186 |     }
  187 | 
  188 |     await firstCard.click();
  189 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  190 | 
  191 |     // Click Cost tab
  192 |     const costTab = page.locator('button[role="tab"]', { hasText: 'Cost' });
  193 |     const tabCount = await costTab.count();
  194 |     if (tabCount > 0) {
  195 |       await costTab.first().click();
  196 | 
  197 |       // Cost panel should be visible
  198 |       const costPanel = page.locator('.cost-panel, .cost-empty, .cost-loading');
  199 |       await expect(costPanel.first()).toBeVisible();
  200 |     }
  201 |   });
  202 | 
  203 |   test('should display Files tab content', async ({ page }) => {
  204 |     const firstCard = page.locator('.project-card-fleet').first();
  205 | 
  206 |     const cardCount = await firstCard.count();
  207 |     if (cardCount === 0) {
  208 |       test.skip(true, 'No projects available');
  209 |       return;
  210 |     }
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
> 271 |     await expect(page.locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')).toBeAttached();
      |                                                                                                       ^ Error: expect(locator).toBeAttached() failed
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
  369 |     await expect(page.locator('.drafts-tab, .drafts-container')).toBeVisible();
  370 |   });
  371 | });
```