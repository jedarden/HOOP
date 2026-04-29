# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: phase3-multimodal.spec.ts >> Phase 3 - File Browser Media Preview >> should support file search and filtering
- Location: e2e/phase3-multimodal.spec.ts:182:3

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
  84  |           await audioFiles.first().click();
  85  |           await page.waitForTimeout(500);
  86  | 
  87  |           // Audio viewer should be visible
  88  |           const audioViewer = page.locator('.audio-viewer, .file-preview-body--audio');
  89  |           await expect(audioViewer.first()).toBeVisible();
  90  | 
  91  |           // Check for audio controls
  92  |           const playButton = page.locator('.audio-viewer-btn-play, button[aria-label*="Play" i], button[aria-label*="Pause" i]');
  93  |           await expect(playButton.first()).toBeAttached();
  94  |         }
  95  |       }
  96  |     }
  97  |   });
  98  | 
  99  |   test('should display video files with video player', async ({ page }) => {
  100 |     await page.goto('/');
  101 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
  102 | 
  103 |     const firstCard = page.locator('.project-card-fleet').first();
  104 |     const cardCount = await firstCard.count();
  105 | 
  106 |     if (cardCount > 0) {
  107 |       await firstCard.click();
  108 |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  109 | 
  110 |       // Navigate to Files tab
  111 |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  112 |       const filesTabCount = await filesTab.count();
  113 | 
  114 |       if (filesTabCount > 0) {
  115 |         await filesTab.first().click();
  116 |         await page.waitForTimeout(500);
  117 | 
  118 |         // Look for video files in the file tree
  119 |         const videoFiles = page.locator('.file-tree-node').filter({ hasText: /\.(mp4|webm|mov|avi|mkv|m4v)$/i });
  120 |         const videoCount = await videoFiles.count();
  121 | 
  122 |         if (videoCount > 0) {
  123 |           // Click first video file
  124 |           await videoFiles.first().click();
  125 |           await page.waitForTimeout(500);
  126 | 
  127 |           // Video viewer should be visible
  128 |           const videoViewer = page.locator('.video-viewer, .file-preview-body--video');
  129 |           await expect(videoViewer.first()).toBeVisible();
  130 | 
  131 |           // Check for video controls
  132 |           const playButton = page.locator('.video-viewer-btn-play, button[aria-label*="Play" i], button[aria-label*="Pause" i]');
  133 |           await expect(playButton.first()).toBeAttached();
  134 |         }
  135 |       }
  136 |     }
  137 |   });
  138 | 
  139 |   test('should support syntax highlighting for code files', async ({ page }) => {
  140 |     await page.goto('/');
  141 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
  142 | 
  143 |     const firstCard = page.locator('.project-card-fleet').first();
  144 |     const cardCount = await firstCard.count();
  145 | 
  146 |     if (cardCount > 0) {
  147 |       await firstCard.click();
  148 |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  149 | 
  150 |       // Navigate to Files tab
  151 |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  152 |       const filesTabCount = await filesTab.count();
  153 | 
  154 |       if (filesTabCount > 0) {
  155 |         await filesTab.first().click();
  156 |         await page.waitForTimeout(500);
  157 | 
  158 |         // Look for code files in the file tree
  159 |         const codeFiles = page.locator('.file-tree-node').filter({ hasText: /\.(rs|ts|tsx|js|py|go|clj|yaml|toml|md|sh|sql|dockerfile)$/i });
  160 |         const codeCount = await codeFiles.count();
  161 | 
  162 |         if (codeCount > 0) {
  163 |           // Click first code file
  164 |           await codeFiles.first().click();
  165 |           await page.waitForTimeout(500);
  166 | 
  167 |           // Code viewer should be visible
  168 |           const codeViewer = page.locator('.code-viewer, .shiki, .hl-wrapper');
  169 |           await expect(codeViewer.first()).toBeAttached();
  170 | 
  171 |           // Check for syntax highlighting (colored spans)
  172 |           const highlightedCode = page.locator('.code-viewer span[style*="color"], .shiki span[style*="color"], .hl-code span[style*="color"]');
  173 |           const highlightCount = await highlightedCode.count();
  174 | 
  175 |           // At least some syntax highlighting should be present
  176 |           expect(highlightCount).toBeGreaterThan(0);
  177 |         }
  178 |       }
  179 |     }
  180 |   });
  181 | 
  182 |   test('should support file search and filtering', async ({ page }) => {
  183 |     await page.goto('/');
> 184 |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
      |                ^ TimeoutError: page.waitForSelector: Timeout 10000ms exceeded.
  185 | 
  186 |     const firstCard = page.locator('.project-card-fleet').first();
  187 |     const cardCount = await firstCard.count();
  188 | 
  189 |     if (cardCount > 0) {
  190 |       await firstCard.click();
  191 |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  192 | 
  193 |       // Navigate to Files tab
  194 |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  195 |       const filesTabCount = await filesTab.count();
  196 | 
  197 |       if (filesTabCount > 0) {
  198 |         await filesTab.first().click();
  199 |         await page.waitForTimeout(500);
  200 | 
  201 |         // Look for filter inputs
  202 |         const filterBar = page.locator('.files-filter-bar');
  203 |         const filterBarCount = await filterBar.count();
  204 | 
  205 |         if (filterBarCount > 0) {
  206 |           // Extension filter input
  207 |           const extInput = page.locator('#ff-ext');
  208 |           await expect(extInput.first()).toBeAttached();
  209 | 
  210 |           // Grep filter input
  211 |           const grepInput = page.locator('#ff-grep');
  212 |           await expect(grepInput.first()).toBeAttached();
  213 | 
  214 |           // Test extension filter
  215 |           await extInput.first().fill('.rs');
  216 |           await page.waitForTimeout(300);
  217 | 
  218 |           // Should show filtered results
  219 |           const searchResults = page.locator('.file-search-row');
  220 |           const resultCount = await searchResults.count();
  221 | 
  222 |           if (resultCount > 0) {
  223 |             // Results should be .rs files
  224 |             const firstResultText = await searchResults.first().textContent();
  225 |             expect(firstResultText).toMatch(/\.rs$/);
  226 |           }
  227 | 
  228 |           // Clear filter
  229 |           await extInput.first().fill('');
  230 |           await page.waitForTimeout(300);
  231 |         }
  232 |       }
  233 |     }
  234 |   });
  235 | });
  236 | 
  237 | test.describe('Phase 3 - Dictated Notes', () => {
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
```