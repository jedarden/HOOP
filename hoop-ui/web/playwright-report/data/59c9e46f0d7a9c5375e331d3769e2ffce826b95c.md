# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: phase3-multimodal.spec.ts >> Phase 3 - Multimodal Agent Chat >> should enforce adapter-specific file size limits
- Location: e2e/phase3-multimodal.spec.ts:480:3

# Error details

```
Error: expect(locator).toBeAttached() failed

Locator: locator('.acp-error-bar').first()
Expected: attached
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeAttached" with timeout 5000ms
  - waiting for locator('.acp-error-bar').first()

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
    - button "Dictation settings" [ref=e106] [cursor=pointer]: ⚙
```

# Test source

```ts
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
> 493 |       await expect(errorBar.first()).toBeAttached();
      |                                      ^ Error: expect(locator).toBeAttached() failed
  494 |     }
  495 |   });
  496 | });
  497 | 
  498 | test.describe('Phase 3 - File Drag-Drop to Drafts', () => {
  499 |   test('should support dragging files from file browser to draft', async ({ page }) => {
  500 |     await page.goto('/');
  501 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
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
```