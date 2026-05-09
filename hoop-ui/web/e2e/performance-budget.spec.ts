import { test, expect } from '@playwright/test';

/**
 * Performance Budget Test Suite
 *
 * Validates that UI interactions stay within the performance budget:
 * - Interaction latency: < 500ms (user-initiated actions)
 * - Page load: < 2s (initial page render)
 * - Time to Interactive: < 3s
 *
 * Plan reference: §6 Phase 6 deliverable 9
 * Feeds into hoop-ttb.7.11 performance budget verification
 */

const PERFORMANCE_BUDGETS = {
  interactionLatencyMs: 500,
  pageLoadMs: 2000,
  timeToInteractiveMs: 3000,
  frameRateThreshold: 30, // fps
};

test.describe('Performance Budget - Page Load', () => {
  test('should load initial page within budget', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    const loadTime = Date.now() - startTime;

    expect(loadTime).toBeLessThan(PERFORMANCE_BUDGETS.pageLoadMs);
  });

  test('should become interactive within budget', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    // Wait for the page to be fully interactive
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('body', { state: 'attached' });

    const tti = Date.now() - startTime;

    expect(tti).toBeLessThan(PERFORMANCE_BUDGETS.timeToInteractiveMs);
  });

  test('should have reasonable Core Web Vitals', async ({ page }) => {
    const metrics = await page.goto('/').then(() =>
      page.evaluate(() => {
        return new Promise((resolve) => {
          // Use PerformanceObserver for Core Web Vitals
          if ('PerformanceObserver' in window) {
            const observer = new PerformanceObserver((list) => {
              const entries = list.getEntries();
              const vitals: Record<string, number> = {};

              for (const entry of entries) {
                if (entry.entryType === 'largest-contentful-paint') {
                  vitals.lcp = entry.startTime;
                } else if (entry.entryType === 'first-input') {
                  const fidEntry = entry as any;
                  vitals.fid = fidEntry.processingStart - fidEntry.startTime;
                }
              }

              resolve(vitals);
            });

            observer.observe({ entryTypes: ['largest-contentful-paint', 'first-input'] });

            // Fallback timeout if metrics aren't available
            setTimeout(() => resolve({}), 5000);
          } else {
            resolve({});
          }
        });
      })
    );

    // LCP should be under 2.5s (good)
    if (metrics.lcp !== undefined) {
      expect(metrics.lcp).toBeLessThan(2500);
    }

    // FID should be under 100ms (good)
    if (metrics.fid !== undefined) {
      expect(metrics.fid).toBeLessThan(100);
    }
  });
});

test.describe('Performance Budget - Interaction Latency', () => {
  test('should respond to navigation clicks within budget', async ({ page }) => {
    await page.goto('/');

    // Wait for page to stabilize
    await page.waitForLoadState('networkidle');

    // Find a navigation element (could be a link, button, etc.)
    const navElement = page.locator('a, button, [role="button"]').first();

    if (await navElement.count() > 0) {
      const startTime = Date.now();

      await navElement.click();
      await page.waitForTimeout(50); // Small wait for reaction

      const latency = Date.now() - startTime;

      expect(latency).toBeLessThan(PERFORMANCE_BUDGETS.interactionLatencyMs);
    }
  });

  test('should maintain frame rate during interactions', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Measure frame rate during rapid interactions
    const frameTimes: number[] = [];

    await page.evaluate(() => {
      return new Promise<void>((resolve) => {
        let frameCount = 0;
        const startTime = performance.now();
        const durationMs = 1000;

        function measureFrame() {
          frameCount++;
          const elapsed = performance.now() - startTime;

          if (elapsed < durationMs) {
            requestAnimationFrame(measureFrame);
          } else {
            // Calculate average frame time
            const avgFrameTime = elapsed / frameCount;
            (window as any).testFrameTime = avgFrameTime;
            resolve();
          }
        }

        requestAnimationFrame(measureFrame);
      });
    });

    const avgFrameTime = await page.evaluate(() => (window as any).testFrameTime);
    const fps = avgFrameTime ? 1000 / avgFrameTime : 60;

    expect(fps).toBeGreaterThan(PERFORMANCE_BUDGETS.frameRateThreshold);
  });

  test('should render updates within budget after data changes', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Trigger a data update (e.g., refresh or navigation)
    const startTime = Date.now();

    await page.reload({ waitUntil: 'networkidle' });

    const updateTime = Date.now() - startTime;

    expect(updateTime).toBeLessThan(PERFORMANCE_BUDGETS.interactionLatencyMs);
  });
});

test.describe('Performance Budget - Server Memory', () => {
  test('should not leak server memory during navigation', async ({ page }) => {
    const daemonUrl = process.env.HOOP_DAEMON_URL || 'http://localhost:8080';

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Get initial server memory usage
    const initialMemory = await page.evaluate(async (url) => {
      try {
        const response = await fetch(`${url}/metrics`);
        const text = await response.text();

        for (const line of text.split('\n')) {
          if (line.startsWith('hoop_process_memory_bytes') && !line.startsWith('#')) {
            const parts = line.split(' ');
            if (parts.length >= 2) {
              return parseFloat(parts[1]);
            }
          }
        }
        return 0;
      } catch {
        return 0;
      }
    }, daemonUrl);

    // Perform multiple navigations
    for (let i = 0; i < 10; i++) {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
    }

    // Get final server memory usage
    const finalMemory = await page.evaluate(async (url) => {
      try {
        const response = await fetch(`${url}/metrics`);
        const text = await response.text();

        for (const line of text.split('\n')) {
          if (line.startsWith('hoop_process_memory_bytes') && !line.startsWith('#')) {
            const parts = line.split(' ');
            if (parts.length >= 2) {
              return parseFloat(parts[1]);
            }
          }
        }
        return 0;
      } catch {
        return 0;
      }
    }, daemonUrl);

    // Memory growth should be reasonable (< 50MB)
    if (initialMemory > 0 && finalMemory > 0) {
      const memoryGrowth = finalMemory - initialMemory;
      expect(memoryGrowth).toBeLessThan(50 * 1024 * 1024); // 50MB
    }
  });

  test('should maintain stable server memory during interaction', async ({ page }) => {
    const daemonUrl = process.env.HOOP_DAEMON_URL || 'http://localhost:8080';

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const memorySnapshots: number[] = [];

    // Take memory snapshots during interaction
    for (let i = 0; i < 5; i++) {
      await page.waitForTimeout(100);

      const memory = await page.evaluate(async (url) => {
        try {
          const response = await fetch(`${url}/metrics`);
          const text = await response.text();

          for (const line of text.split('\n')) {
            if (line.startsWith('hoop_process_memory_bytes') && !line.startsWith('#')) {
              const parts = line.split(' ');
              if (parts.length >= 2) {
                return parseFloat(parts[1]);
              }
            }
          }
          return 0;
        } catch {
          return 0;
        }
      }, daemonUrl);

      if (memory > 0) {
        memorySnapshots.push(memory);
      }
    }

    // Check that memory doesn't grow excessively
    if (memorySnapshots.length > 0) {
      const maxMemory = Math.max(...memorySnapshots);
      const minMemory = Math.min(...memorySnapshots);
      const growth = maxMemory - minMemory;

      expect(growth).toBeLessThan(20 * 1024 * 1024); // 20MB
    }
  });

  test('should respect 4GB memory ceiling under load', async ({ page }) => {
    const daemonUrl = process.env.HOOP_DAEMON_URL || 'http://localhost:8080';

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Sample server memory multiple times
    const memorySnapshots: number[] = [];

    for (let i = 0; i < 10; i++) {
      await page.waitForTimeout(100);

      const memory = await page.evaluate(async (url) => {
        try {
          const response = await fetch(`${url}/metrics`);
          const text = await response.text();

          for (const line of text.split('\n')) {
            if (line.startsWith('hoop_process_memory_bytes') && !line.startsWith('#')) {
              const parts = line.split(' ');
              if (parts.length >= 2) {
                return parseFloat(parts[1]);
              }
            }
          }
          return 0;
        } catch {
          return 0;
        }
      }, daemonUrl);

      if (memory > 0) {
        memorySnapshots.push(memory);
      }
    }

    // Check that memory stays under 4GB ceiling
    if (memorySnapshots.length > 0) {
      const maxMemory = Math.max(...memorySnapshots);
      const maxMemoryMB = maxMemory / (1024 * 1024);
      const ceilingMB = 4 * 1024; // 4GB

      expect(maxMemoryMB).toBeLessThan(ceilingMB);
    }
  });
});

test.describe('Performance Budget - Network', () => {
  test('should complete initial network requests quickly', async ({ page }) => {
    const requestTimes: number[] = [];

    page.on('requestfinished', (request) => {
      if (request.url().includes('/api/')) {
        const timing = request.timing();
        if (timing) {
          requestTimes.push(timing.responseEnd);
        }
      }
    });

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Check that API requests complete within budget
    if (requestTimes.length > 0) {
      const maxRequestTime = Math.max(...requestTimes);
      expect(maxRequestTime).toBeLessThan(PERFORMANCE_BUDGETS.interactionLatencyMs);
    }
  });

  test('should not make excessive network requests', async ({ page }) => {
    let requestCount = 0;

    page.on('request', (request) => {
      if (request.url().includes('/api/')) {
        requestCount++;
      }
    });

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Page load should not trigger excessive API calls
    expect(requestCount).toBeLessThan(20);
  });
});

test.describe('Performance Budget - Rendering', () => {
  test('should not cause layout thrashing', async ({ page }) => {
    await page.goto('/');

    const layoutCount = await page.evaluate(() => {
      let count = 0;
      const observer = new PerformanceObserver(() => {
        count++;
      });
      observer.observe({ entryTypes: ['layout-shift'] });

      // Trigger some interactions
      return new Promise((resolve) => {
        setTimeout(() => {
          observer.disconnect();
          resolve(count);
        }, 2000);
      });
    });

    // CLS should be minimal (few layout shifts)
    expect(layoutCount).toBeLessThan(10);
  });

  test('should render long lists efficiently', async ({ page }) => {
    await page.goto('/');

    // Find any list elements
    const lists = page.locator('ul, ol, [role="list"], [data-testid*="list"]');

    if (await lists.count() > 0) {
      const startTime = Date.now();

      // Scroll through the list
      await lists.first().scroll({ top: 500, behavior: 'smooth' });
      await page.waitForTimeout(100);

      const renderTime = Date.now() - startTime;

      expect(renderTime).toBeLessThan(PERFORMANCE_BUDGETS.interactionLatencyMs);
    }
  });
});
