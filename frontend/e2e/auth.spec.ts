import { test, expect } from '@playwright/test';

// Test credentials - should match seeded data in backend
const TEST_USER = {
  email: 'demo@zeltra.io',
  password: 'password123',
};

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any existing auth state
    await page.context().clearCookies();
    await page.evaluate(() => localStorage.clear());
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Fill in credentials
    await page.getByLabel(/email/i).fill(TEST_USER.email);
    await page.getByLabel(/password/i).fill(TEST_USER.password);

    // Submit form
    await page.getByRole('button', { name: /sign in/i }).click();

    // Wait for API response and redirect
    await page.waitForURL(/\/dashboard/, { timeout: 10000 });

    // Verify dashboard content is visible
    await expect(page.getByText('Financial Overview')).toBeVisible({ timeout: 5000 });
  });

  test('should show error with invalid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Fill in invalid credentials
    await page.getByLabel(/email/i).fill('wrong@example.com');
    await page.getByLabel(/password/i).fill('wrongpass');

    // Submit form
    await page.getByRole('button', { name: /sign in/i }).click();

    // Wait for error response
    await page.waitForTimeout(1000);

    // Verify error message (toast notification)
    await expect(page.getByText(/invalid|failed|error/i)).toBeVisible({ timeout: 5000 });
    
    // Verify still on login page
    await expect(page).toHaveURL(/\/login/);
  });

  test('should logout successfully', async ({ page }) => {
    // 1. Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill(TEST_USER.email);
    await page.getByLabel(/password/i).fill(TEST_USER.password);
    await page.getByRole('button', { name: /sign in/i }).click();
    await page.waitForURL(/\/dashboard/, { timeout: 10000 });

    // 2. Perform logout via user menu
    const userMenu = page.getByRole('button', { name: /user menu|profile|account/i });
    if (await userMenu.isVisible({ timeout: 3000 }).catch(() => false)) {
      await userMenu.click();
      await page.getByRole('menuitem', { name: /logout|sign out/i }).click();
    } else {
      // Try sidebar logout button
      const logoutBtn = page.getByRole('button', { name: /logout|sign out/i });
      if (await logoutBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
        await logoutBtn.click();
      }
    }

    // 3. Verify redirection to login
    await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
  });

  test('should persist auth state on page reload', async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill(TEST_USER.email);
    await page.getByLabel(/password/i).fill(TEST_USER.password);
    await page.getByRole('button', { name: /sign in/i }).click();
    await page.waitForURL(/\/dashboard/, { timeout: 10000 });

    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Should still be on dashboard (not redirected to login)
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('should redirect unauthenticated users to login', async ({ page }) => {
    // Try to access protected route without auth
    await page.goto('/dashboard');
    
    // Should redirect to login
    await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
  });
});
