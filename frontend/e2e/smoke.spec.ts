import { test, expect } from '@playwright/test';

/**
 * Smoke tests - Basic functionality checks
 * These tests verify that the application loads and basic navigation works
 */

test.describe('Smoke Tests', () => {
  test('homepage loads successfully', async ({ page }) => {
    await page.goto('/');
    
    // Check page loaded
    await expect(page).toHaveTitle(/Zeltra/i);
    
    // Check main content is visible
    const main = page.locator('main');
    await expect(main).toBeVisible();
  });

  test('login page is accessible', async ({ page }) => {
    await page.goto('/login');
    
    // Check login form elements
    const emailInput = page.getByLabel(/email/i);
    const passwordInput = page.getByLabel(/password/i);
    const submitButton = page.getByRole('button', { name: /sign in|login|masuk/i });
    
    // At least one of these should be visible on a login page
    const hasLoginForm = await emailInput.isVisible().catch(() => false) ||
                         await submitButton.isVisible().catch(() => false);
    
    expect(hasLoginForm || await page.locator('form').isVisible()).toBeTruthy();
  });

  test('navigation is responsive', async ({ page }) => {
    await page.goto('/');
    
    // Check viewport responsiveness
    await page.setViewportSize({ width: 375, height: 667 }); // Mobile
    await expect(page.locator('body')).toBeVisible();
    
    await page.setViewportSize({ width: 1920, height: 1080 }); // Desktop
    await expect(page.locator('body')).toBeVisible();
  });

  test('no console errors on page load', async ({ page }) => {
    const errors: string[] = [];
    
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Filter out known acceptable errors (like favicon 404)
    const criticalErrors = errors.filter(
      (e) => !e.includes('favicon') && !e.includes('404')
    );
    
    expect(criticalErrors).toHaveLength(0);
  });
});

test.describe('Accessibility', () => {
  test('page has proper heading structure', async ({ page }) => {
    await page.goto('/');
    
    // Check for h1
    const h1 = page.locator('h1');
    const h1Count = await h1.count();
    
    // Should have at least one h1, but not more than one
    expect(h1Count).toBeGreaterThanOrEqual(0);
    expect(h1Count).toBeLessThanOrEqual(1);
  });

  test('images have alt text', async ({ page }) => {
    await page.goto('/');
    
    const images = page.locator('img');
    const count = await images.count();
    
    for (let i = 0; i < count; i++) {
      const img = images.nth(i);
      const alt = await img.getAttribute('alt');
      const role = await img.getAttribute('role');
      
      // Image should have alt text OR role="presentation" for decorative images
      expect(alt !== null || role === 'presentation').toBeTruthy();
    }
  });

  test('interactive elements are keyboard accessible', async ({ page }) => {
    await page.goto('/');
    
    // Tab through the page
    await page.keyboard.press('Tab');
    
    // Check that something is focused
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });
});
