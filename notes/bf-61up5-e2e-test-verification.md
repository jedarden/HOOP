# E2E Test Verification - Phase 2 Exit Gate

**Task**: Verify Playwright e2e tests pass on desktop + mobile viewport
**Date**: 2026-05-31
**Result**: ❌ **TESTS DO NOT PASS**

## Summary

The Playwright e2e tests in `hoop-ui/web/e2e/` were executed against the running daemon (port 3000) and Vite dev server (port 5173). **All three required test files failed** due to CSS selectors and DOM structure assumptions that don't match the actual UI implementation.

## Test Execution

```bash
cd hoop-ui/web
npx playwright test smoke-tests.spec.ts
# Exit code: 1 - 252 tests failed
```

## Root Cause: Selector Mismatches

The tests were written with assumptions about the UI structure that don't match the actual React components. Key mismatches:

### 1. Settings Button (smoke-tests.spec.ts:23)

**Test expects:**
```typescript
await expect(page.locator('button[aria-label="Settings"]')).toBeVisible();
```

**Actual implementation (SettingsMenu.tsx:56-60):**
```tsx
<button
  className="settings-trigger"
  onClick={handleOpen}
  aria-label="Open settings"  // ← Different label
  title="Settings"
>
```

**Fix needed:** Use `button[aria-label="Open settings"]` or `.settings-trigger`

---

### 2. Fleet Map (smoke-tests.spec.ts:255)

**Test expects:**
```typescript
await expect(page.locator('.fleet-map, .worker-grid')).toBeVisible();
```

**Actual implementation (FleetMap.tsx:122):**
```tsx
<div className="worker-grid">
```

**Fix needed:** Use `.worker-grid` only (no `.fleet-map` class exists)

---

### 3. Agent Chat Pane Input (smoke-tests.spec.ts:271)

**Test expects:**
```typescript
await expect(page.locator('textarea[placeholder*="message" i], input[type="text"], .chat-input')).toBeAttached();
```

**Actual implementation (AgentChatPane.tsx:621-637):**
```tsx
<textarea
  ref={textareaRef}
  className="acp-textarea"
  value={input}
  // ...
  placeholder={
    isDragOver
      ? "Drop files here to attach..."
      : (!isActive && agentStatus.tool_name)
          ? "Agent is running..."
          : "Ask anything about this project..."  // ← Doesn't contain "message"
  }
  aria-label="Message input"
/>
```

**Fix needed:** Use `textarea.acp-textarea` or `textarea[aria-label="Message input"]`

---

### 4. Patterns View Container (smoke-tests.spec.ts:325)

**Test expects:**
```typescript
const patternList = page.locator('.pattern-list, .patterns-container');
await expect(patternList.first()).toBeVisible();
```

**Actual implementation:** Need to verify PatternsView component structure, but neither class appears to be used.

---

### 5. Timeline View (smoke-tests.spec.ts:378)

**Test expects:**
```typescript
await expect(page.locator('.worker-timeline, .timeline-view')).toBeVisible();
```

**Actual implementation:** Need to verify WorkerTimeline component classes.

---

### 6. Navigation Timeout with Overlay (smoke-tests.spec.ts:471)

**Test error:**
```
<div class="wt-overlay">…</div> intercepts pointer events
```

**Issue:** The WelcomeTour overlay (`wt-overlay`) blocks navigation clicks during tests.

---

## Mobile Responsiveness Test Failures

Similar selector issues exist in `mobile-responsiveness.spec.ts`:

1. Settings button selector mismatch
2. Fleet map selector mismatch  
3. Agent chat pane selector mismatch
4. Touch target size tests fail due to incorrect element selection
5. PDF viewer tests fail when PDF files don't exist in test data

---

## Performance Budget Test Failures

The `performance-budget.spec.ts` tests also fail for similar reasons - they attempt to interact with UI elements that don't exist or have different selectors.

---

## Recommended Next Steps

To make the e2e tests pass, one of two approaches is needed:

### Option A: Fix the Test Selectors

Update all test files to use correct selectors matching the actual UI implementation:

1. `smoke-tests.spec.ts` - Update ~50 selectors
2. `mobile-responsiveness.spec.ts` - Update ~30 selectors
3. `performance-budget.spec.ts` - Update ~20 selectors

**Estimated effort:** 2-4 hours of selector updates and re-testing.

### Option B: Add Test-Specific Data Attributes

Add `data-testid` attributes to the UI components for stable testing:

```tsx
<button data-testid="settings-button" className="settings-trigger" ...>
<div data-testid="fleet-map" className="worker-grid" ...>
<textarea data-testid="agent-chat-input" className="acp-textarea" ...>
```

**Estimated effort:** 1-2 hours to add attributes + update test selectors.

---

## Verification Status

**Phase 2 Exit Gate Requirement:** "Confirm smoke-tests.spec.ts, mobile-responsiveness.spec.ts, and performance-budget.spec.ts all pass."

**Status:** ❌ **NOT VERIFIED** - Tests fail due to selector mismatches.

**Blocker:** The e2e test suite needs selector fixes or `data-testid` attributes added to the UI before the Phase 2 exit gate can be considered verified.

---

## Test Output Sample

```
Running 252 tests using 6 workers

  1) [chromium] › e2e/smoke-tests.spec.ts:13:3 › Smoke Tests - Overview Page › should load overview page with header and navigation

    Error: expect(locator).toBeVisible() failed
    Locator: locator('button[aria-label="Settings"]')
    Expected: visible
    Timeout: 5000ms
    Error: element(s) not found
```

---

## Files Requiring Updates

| File | Lines to Fix | Severity |
|------|--------------|----------|
| `e2e/smoke-tests.spec.ts` | ~50 selectors | High |
| `e2e/mobile-responsiveness.spec.ts` | ~30 selectors | High |
| `e2e/performance-budget.spec.ts` | ~20 selectors | High |

---

## Environment Notes

- Daemon running on: `http://localhost:3000`
- UI dev server on: `http://localhost:5173` (Vite)
- Playwright version: 1.59.1
- Projects in test data: 0 (empty fleet)

The empty fleet state also contributes to test skips for project-dependent tests.
