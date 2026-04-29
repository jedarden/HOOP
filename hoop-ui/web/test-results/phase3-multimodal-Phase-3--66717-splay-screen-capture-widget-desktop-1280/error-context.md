# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: phase3-multimodal.spec.ts >> Phase 3 - Screen Capture >> should display screen capture widget
- Location: e2e/phase3-multimodal.spec.ts:332:3

# Error details

```
Error: expect(locator).toBeAttached() failed

Locator: locator('.screen-capture-widget').first()
Expected: attached
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeAttached" with timeout 5000ms
  - waiting for locator('.screen-capture-widget').first()

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
            - link "Search →" [ref=e37] [cursor=pointer]:
              - /url: "#/search"
            - link "Dashboard →" [ref=e38] [cursor=pointer]:
              - /url: "#/dashboard"
            - link "Live map →" [ref=e39] [cursor=pointer]:
              - /url: "#/fleet"
            - link "Timeline →" [ref=e40] [cursor=pointer]:
              - /url: "#/timeline"
            - link "Diagnostics →" [ref=e41] [cursor=pointer]:
              - /url: "#/diagnostics"
        - generic [ref=e44]: Loading projects…
  - generic [ref=e45]:
    - generic [ref=e46]: 🎤
    - generic [ref=e47]: ⌘+⇧+D
    - button "Dictation settings" [ref=e48] [cursor=pointer]: ⚙
  - button "Open settings" [ref=e50] [cursor=pointer]:
    - img [ref=e51]
  - dialog "Welcome tour" [ref=e54]:
    - generic [ref=e55]:
      - heading "Welcome to HOOP" [level=2] [ref=e56]
      - button "Close tour" [ref=e57] [cursor=pointer]: ×
    - generic [ref=e58]:
      - paragraph [ref=e59]: HOOP is your AI-powered development companion. Let's take a quick tour.
      - generic [ref=e60]:
        - generic [ref=e61]:
          - heading "Stitches" [level=4] [ref=e62]
          - paragraph [ref=e63]: Conversations and work items — where you collaborate with AI agents to get things done.
        - generic [ref=e64]:
          - heading "Patterns" [level=4] [ref=e65]
          - paragraph [ref=e66]: Reusable workflows that span multiple projects and automate repetitive tasks.
    - button "Next" [ref=e74] [cursor=pointer]
```

# Test source

```ts
  238 |   test.use({ viewport: { width: 1280, height: 720 } });
  239 | 
  240 |   test('should display dictation widget', async ({ page }) => {
  241 |     await page.goto('/');
  242 |     await page.waitForLoadState('networkidle');
  243 | 
  244 |     // Dictation widget should be present
  245 |     const dictationWidget = page.locator('.dictation-widget');
  246 |     await expect(dictationWidget.first()).toBeAttached();
  247 | 
  248 |     // Should have mic icon
  249 |     const micIcon = page.locator('.dictation-mic-icon');
  250 |     await expect(micIcon.first()).toBeVisible();
  251 |   });
  252 | 
  253 |   test('should respond to dictation hotkey', async ({ page }) => {
  254 |     await page.goto('/');
  255 |     await page.waitForLoadState('networkidle');
  256 | 
  257 |     // Trigger dictation hotkey
  258 |     const hotkey = process.platform === 'darwin' ? 'Meta+Shift+d' : 'Control+Shift+d';
  259 | 
  260 |     // Dispatch keyboard event
  261 |     await page.keyboard.press(hotkey);
  262 |     await page.waitForTimeout(500);
  263 | 
  264 |     // Dictation widget state should change (may show recording UI or settings)
  265 |     const dictationWidget = page.locator('.dictation-widget');
  266 |     await expect(dictationWidget.first()).toBeAttached();
  267 |   });
  268 | 
  269 |   test('should allow rebinding dictation hotkey', async ({ page }) => {
  270 |     await page.goto('/');
  271 |     await page.waitForLoadState('networkidle');
  272 | 
  273 |     // Click settings gear
  274 |     const gearBtn = page.locator('.dictation-gear-btn');
  275 |     const gearCount = await gearBtn.count();
  276 | 
  277 |     if (gearCount > 0) {
  278 |       await gearBtn.first().click();
  279 |       await page.waitForTimeout(200);
  280 | 
  281 |       // Settings panel should appear
  282 |       const settingsPanel = page.locator('.dictation-settings-panel');
  283 |       await expect(settingsPanel.first()).toBeVisible();
  284 | 
  285 |       // Click rebind button
  286 |       const rebindBtn = page.locator('button:has-text("Rebind")');
  287 |       const rebindCount = await rebindBtn.count();
  288 | 
  289 |       if (rebindCount > 0) {
  290 |         await rebindBtn.first().click();
  291 |         await page.waitForTimeout(200);
  292 | 
  293 |         // Hotkey binder should appear
  294 |         const binder = page.locator('.dictation-hotkey-binder');
  295 |         await expect(binder.first()).toBeVisible();
  296 | 
  297 |         // Cancel to avoid actually rebinding
  298 |         const cancelBtn = binder.locator('button:has-text("Cancel")');
  299 |         await cancelBtn.click();
  300 |       }
  301 |     }
  302 |   });
  303 | 
  304 |   test('should display notes timeline when notes exist', async ({ page }) => {
  305 |     // This test requires actual dictated notes to exist
  306 |     test.skip(true, 'Requires pre-existing dictated notes');
  307 | 
  308 |     await page.goto('/');
  309 |     await page.waitForSelector('.fleet-cards-grid', { timeout: 10000 });
  310 | 
  311 |     const firstCard = page.locator('.project-card-fleet').first();
  312 |     await firstCard.click();
  313 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  314 | 
  315 |     // Look for notes timeline
  316 |     const notesTimeline = page.locator('.notes-timeline-section');
  317 |     const timelineCount = await notesTimeline.count();
  318 | 
  319 |     if (timelineCount > 0) {
  320 |       await expect(notesTimeline.first()).toBeVisible();
  321 | 
  322 |       // Should have time window picker
  323 |       const windowPicker = page.locator('.timeline-window-picker');
  324 |       await expect(windowPicker.first()).toBeAttached();
  325 |     }
  326 |   });
  327 | });
  328 | 
  329 | test.describe('Phase 3 - Screen Capture', () => {
  330 |   test.use({ viewport: { width: 1280, height: 720 } });
  331 | 
  332 |   test('should display screen capture widget', async ({ page }) => {
  333 |     await page.goto('/');
  334 |     await page.waitForLoadState('networkidle');
  335 | 
  336 |     // Screen capture widget should be present
  337 |     const screenCaptureWidget = page.locator('.screen-capture-widget');
> 338 |     await expect(screenCaptureWidget.first()).toBeAttached();
      |                                               ^ Error: expect(locator).toBeAttached() failed
  339 |   });
  340 | 
  341 |   test('should start screen capture on button click', async ({ page }) => {
  342 |     await page.goto('/');
  343 |     await page.waitForLoadState('networkidle');
  344 | 
  345 |     const recordBtn = page.locator('.screen-capture-btn-record');
  346 |     const recordCount = await recordBtn.count();
  347 | 
  348 |     if (recordCount > 0 && await recordBtn.first().isEnabled()) {
  349 |       // Note: This will trigger the browser's screen picker
  350 |       // We'll cancel immediately to avoid actually recording
  351 |       const promise = recordBtn.first().click();
  352 | 
  353 |       // Wait a moment for the picker to appear
  354 |       await page.waitForTimeout(500);
  355 | 
  356 |       // Press Escape to cancel the picker
  357 |       await page.keyboard.press('Escape');
  358 | 
  359 |       try {
  360 |         await promise;
  361 |       } catch {
  362 |         // Click might have been cancelled, that's ok
  363 |       }
  364 |     }
  365 |   });
  366 | 
  367 |   test('should show recording state when capturing', async ({ page }) => {
  368 |     // This test requires actual screen capture to be in progress
  369 |     test.skip(true, 'Requires active screen capture session');
  370 | 
  371 |     await page.goto('/');
  372 |     await page.waitForLoadState('networkidle');
  373 | 
  374 |     // Widget should be in recording state
  375 |     const recordingWidget = page.locator('.screen-capture-widget--recording');
  376 |     await expect(recordingWidget.first()).toBeVisible();
  377 | 
  378 |     // Should have timer display
  379 |     const timer = page.locator('.screen-capture-timer');
  380 |     await expect(timer.first()).toBeVisible();
  381 | 
  382 |     // Should have stop button
  383 |     const stopBtn = page.locator('.screen-capture-btn--stop');
  384 |     await expect(stopBtn.first()).toBeVisible();
  385 |   });
  386 | });
  387 | 
  388 | test.describe('Phase 3 - Multimodal Agent Chat', () => {
  389 |   test('should support file attachments in agent chat', async ({ page }) => {
  390 |     await page.goto('/#/fleet');
  391 |     await page.waitForLoadState('networkidle');
  392 | 
  393 |     // Find agent chat pane
  394 |     const agentChat = page.locator('.agent-chat-pane');
  395 |     const chatCount = await agentChat.count();
  396 | 
  397 |     if (chatCount > 0) {
  398 |       await expect(agentChat.first()).toBeVisible();
  399 | 
  400 |       // Look for attachment button
  401 |       const attachBtn = page.locator('.acp-attach-btn');
  402 |       await expect(attachBtn.first()).toBeAttached();
  403 |     }
  404 |   });
  405 | 
  406 |   test('should handle image paste in chat input', async ({ page }) => {
  407 |     await page.goto('/#/fleet');
  408 |     await page.waitForLoadState('networkidle');
  409 | 
  410 |     const agentChat = page.locator('.agent-chat-pane');
  411 |     const chatCount = await agentChat.count();
  412 | 
  413 |     if (chatCount > 0) {
  414 |       // Find textarea
  415 |       const textarea = page.locator('.acp-textarea');
  416 |       const textareaCount = await textarea.count();
  417 | 
  418 |       if (textareaCount > 0 && await textarea.first().isVisible()) {
  419 |         // Create a small test image
  420 |         const testImage = Buffer.from(
  421 |           'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==',
  422 |           'base64'
  423 |         );
  424 | 
  425 |         // Simulate paste event with image
  426 |         await textarea.first().evaluate((el) => {
  427 |           const dataTransfer = new DataTransfer();
  428 |           const file = new File(['test'], 'test.png', { type: 'image/png' });
  429 |           dataTransfer.items.add(file);
  430 | 
  431 |           const pasteEvent = new ClipboardEvent('paste', {
  432 |             bubbles: true,
  433 |             cancelable: true,
  434 |             clipboardData: dataTransfer,
  435 |           });
  436 | 
  437 |           el.dispatchEvent(pasteEvent);
  438 |         });
```