# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: phase3-multimodal.spec.ts >> Phase 3 - File Browser Media Preview >> should display image files in file browser
- Location: e2e/phase3-multimodal.spec.ts:23:3

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
  1   | import { test, expect } from '@playwright/test';
  2   | 
  3   | /**
  4   |  * Phase 3: Multimodal Features E2E Tests
  5   |  *
  6   |  * Tests for Phase 3 features:
  7   |  * - File browser with media preview
  8   |  * - Dictated notes workflow
  9   |  * - Screen capture workflow
  10  |  * - Multimodal agent chat input
  11  |  *
  12  |  * Per plan-phase3.md closing criteria:
  13  |  * 1. File browser <1s on 20k files
  14  |  * 2. Syntax highlighting for 10+ languages
  15  |  * 3. Image/audio/video preview
  16  |  * 4. 10MB attachment in Stitch
  17  |  * 5. Voice capture → Note <60s
  18  |  *
  19  |  * Run: npm run test:e2e phase3-multimodal.spec.ts
  20  |  */
  21  | 
  22  | test.describe('Phase 3 - File Browser Media Preview', () => {
  23  |   test('should display image files in file browser', async ({ page }) => {
  24  |     await page.goto('/');
> 25  |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
      |                ^ TimeoutError: page.waitForSelector: Timeout 10000ms exceeded.
  26  | 
  27  |     const firstCard = page.locator('.project-card-fleet').first();
  28  |     const cardCount = await firstCard.count();
  29  | 
  30  |     if (cardCount > 0) {
  31  |       await firstCard.click();
  32  |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  33  | 
  34  |       // Navigate to Files tab
  35  |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  36  |       const filesTabCount = await filesTab.count();
  37  | 
  38  |       if (filesTabCount > 0) {
  39  |         await filesTab.first().click();
  40  |         await page.waitForTimeout(500);
  41  | 
  42  |         // Look for image files in the file tree
  43  |         const imageFiles = page.locator('.file-tree-node').filter({ hasText: /\.(png|jpg|jpeg|gif|webp|svg)$/i });
  44  |         const imageCount = await imageFiles.count();
  45  | 
  46  |         if (imageCount > 0) {
  47  |           // Click first image file
  48  |           await imageFiles.first().click();
  49  |           await page.waitForTimeout(500);
  50  | 
  51  |           // Image viewer should be visible
  52  |           const imageViewer = page.locator('.image-viewer, .file-preview-body--image');
  53  |           await expect(imageViewer.first()).toBeVisible();
  54  |         }
  55  |       }
  56  |     }
  57  |   });
  58  | 
  59  |   test('should display audio files with audio player', async ({ page }) => {
  60  |     await page.goto('/');
  61  |     await page.waitForSelector('.fleet-cards-grid, .fleet-empty', { timeout: 10000 });
  62  | 
  63  |     const firstCard = page.locator('.project-card-fleet').first();
  64  |     const cardCount = await firstCard.count();
  65  | 
  66  |     if (cardCount > 0) {
  67  |       await firstCard.click();
  68  |       await page.waitForSelector('.app-project-detail', { timeout: 5000 });
  69  | 
  70  |       // Navigate to Files tab
  71  |       const filesTab = page.locator('button[role="tab"]', { hasText: 'Files' });
  72  |       const filesTabCount = await filesTab.count();
  73  | 
  74  |       if (filesTabCount > 0) {
  75  |         await filesTab.first().click();
  76  |         await page.waitForTimeout(500);
  77  | 
  78  |         // Look for audio files in the file tree
  79  |         const audioFiles = page.locator('.file-tree-node').filter({ hasText: /\.(mp3|m4a|wav|ogg|flac|opus|webm)$/i });
  80  |         const audioCount = await audioFiles.count();
  81  | 
  82  |         if (audioCount > 0) {
  83  |           // Click first audio file
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
```