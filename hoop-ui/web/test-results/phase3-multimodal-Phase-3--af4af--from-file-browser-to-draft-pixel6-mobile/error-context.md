# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: phase3-multimodal.spec.ts >> Phase 3 - File Drag-Drop to Drafts >> should support dragging files from file browser to draft
- Location: e2e/phase3-multimodal.spec.ts:499:3

# Error details

```
TimeoutError: page.waitForSelector: Timeout 10000ms exceeded.
Call log:
  - waiting for locator('.fleet-cards-grid, .fleet-empty') to be visible

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
  439 | 
  440 |         await page.waitForTimeout(500);
  441 | 
  442 |         // Attachment preview should appear
  443 |         const attachmentChip = page.locator('.acp-attachment-chip');
  444 |         // Note: Paste might not work without real image data, so we just check the handler exists
  445 |       }
  446 |     }
  447 |   });
  448 | 
  449 |   test('should support drag-drop file attachments', async ({ page }) => {
  450 |     await page.goto('/#/fleet');
  451 |     await page.waitForLoadState('networkidle');
  452 | 
  453 |     const agentChat = page.locator('.agent-chat-pane');
  454 |     const chatCount = await agentChat.count();
  455 | 
  456 |     if (chatCount > 0) {
  457 |       const inputArea = agentChat.locator('.acp-input-area');
  458 |       const inputCount = await inputArea.count();
  459 | 
  460 |       if (inputCount > 0) {
  461 |         // Check if input area has drag-drop handlers
  462 |         const hasDragDrop = await inputArea.first().evaluate(el => {
  463 |           const events = [
  464 |             'dragover',
  465 |             'dragleave',
  466 |             'drop',
  467 |           ];
  468 |           return events.some(event => {
  469 |             const listeners = (el as any).getEventListeners?.(event);
  470 |             return listeners && listeners.length > 0;
  471 |           });
  472 |         });
  473 | 
  474 |         // Drag-drop support should be available
  475 |         expect(true).toBeTruthy();
  476 |       }
  477 |     }
  478 |   });
  479 | 
  480 |   test('should enforce adapter-specific file size limits', async ({ page }) => {
  481 |     await page.goto('/#/fleet');
  482 |     await page.waitForLoadState('networkidle');
  483 | 
  484 |     const agentChat = page.locator('.agent-chat-pane');
  485 |     const chatCount = await agentChat.count();
  486 | 
  487 |     if (chatCount > 0) {
  488 |       // The adapter caps should be defined in the component
  489 |       // This test verifies the UI handles size limits
  490 | 
  491 |       // Check for error display capability
  492 |       const errorBar = page.locator('.acp-error-bar');
  493 |       await expect(errorBar.first()).toBeAttached();
  494 |     }
  495 |   });
  496 | });
  497 | 
  498 | test.describe('Phase 3 - File Drag-Drop to Drafts', () => {
  499 |   test('should support dragging files from file browser to draft', async ({ page }) => {
  500 |     await page.goto('/');
> 501 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
      |                ^ TimeoutError: page.waitForSelector: Timeout 10000ms exceeded.
  502 | 
  503 |     const firstCard = page.locator('.project-card-fleet').first();
  504 |     const cardCount = await firstCard.count();
  505 | 
  506 |     if (cardCount > 0) {
  507 |       await firstCard.click();
  508 |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  509 | 
  510 |       // Navigate to Files tab
  511 |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  512 |       const filesTabCount = await filesTab.count();
  513 | 
  514 |       if (filesTabCount > 0) {
  515 |         await filesTab.first().click();
  516 |         await page.waitForTimeout(500);
  517 | 
  518 |         // Check if files are draggable
  519 |         const draggableFile = page.locator('.file-tree-node--draggable');
  520 |         const draggableCount = await draggableFile.count();
  521 | 
  522 |         if (draggableCount > 0) {
  523 |           // File should have draggable attribute
  524 |           const isDraggable = await draggableFile.first().getAttribute('draggable');
  525 |           expect(isDraggable).toBe('true');
  526 |         }
  527 |       }
  528 |     }
  529 |   });
  530 | });
  531 | 
  532 | test.describe('Phase 3 - Performance', () => {
  533 |   test('should load file tree quickly for large repos', async ({ page }) => {
  534 |     test.skip(true, 'Requires a large repo (20k+ files) for accurate testing');
  535 | 
  536 |     await page.goto('/');
  537 |     await page.waitForSelector('.fleet-cards-grid', { timeout: 10000 });
  538 | 
  539 |     const firstCard = page.locator('.project-card-fleet').first();
  540 |     await firstCard.click();
  541 |     await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  542 | 
  543 |     // Navigate to Files tab
  544 |     const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  545 |     await filesTab.first().click();
  546 | 
  547 |     // Measure time to load file tree
  548 |     const startTime = Date.now();
  549 | 
  550 |     // Wait for file tree to be populated
  551 |     await page.waitForSelector('.file-tree-node', { timeout: 5000 });
  552 | 
  553 |     const loadTime = Date.now() - startTime;
  554 | 
  555 |     // Should load in under 1 second per closing criteria
  556 |     expect(loadTime).toBeLessThan(1000);
  557 |   });
  558 | });
  559 | 
  560 | test.describe('Phase 3 - Mobile UX', () => {
  561 |   test('should optimize dictation widget for mobile', async ({ page }) => {
  562 |     await page.setViewportSize({ width: 412, height: 915 }); // Pixel 6
  563 |     await page.goto('/');
  564 |     await page.waitForLoadState('networkidle');
  565 | 
  566 |     // Dictation widget should be present on mobile
  567 |     const dictationWidget = page.locator('.dictation-widget');
  568 |     await expect(dictationWidget.first()).toBeAttached();
  569 | 
  570 |     // Should show mobile-optimized mic button
  571 |     const micIcon = page.locator('.dictation-mic-icon--mobile');
  572 |     const micCount = await micIcon.count();
  573 | 
  574 |     // May or may not have mobile-specific class depending on implementation
  575 |     if (micCount > 0) {
  576 |       await expect(micIcon.first()).toBeVisible();
  577 |     }
  578 |   });
  579 | 
  580 |   test('should provide haptic feedback for dictation on mobile', async ({ page }) => {
  581 |     // Vibration API test - requires actual mobile device or emulator
  582 |     test.skip(true, 'Requires mobile device with vibration support');
  583 | 
  584 |     await page.setViewportSize({ width: 412, height: 915 });
  585 |     await page.goto('/');
  586 | 
  587 |     // Trigger dictation
  588 |     const hotkey = process.platform === 'darwin' ? 'Meta+Shift+d' : 'Control+Shift+d';
  589 |     await page.keyboard.press(hotkey);
  590 |     await page.waitForTimeout(500);
  591 | 
  592 |     // Check if vibration API was called (not directly testable in Playwright)
  593 |     // This is documented in the implementation
  594 |   });
  595 | });
  596 | 
```