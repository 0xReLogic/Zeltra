import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test('should login successfully with valid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Fill in credentials
    await page.getByLabel(/email/i).fill('demo@zeltra.io');
    await page.getByLabel(/password/i).fill('password');

    // Submit form
    await page.getByRole('button', { name: /sign in/i }).click();

    // Verify redirection to dashboard
    await expect(page).toHaveURL(/\/dashboard/);
    
    // Verify dashboard content is visible
    await expect(page.getByText('Financial Overview')).toBeVisible();
    await expect(page.getByText('Recent Activity')).toBeVisible();
  });

  test('should show error with invalid credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill in invalid credentials
    await page.getByLabel(/email/i).fill('wrong@example.com');
    await page.getByLabel(/password/i).fill('wrongpass');

    // Submit form
    await page.getByRole('button', { name: /sign in/i }).click();

    // Verify error message (assuming toast or alert)
    await expect(page.getByText(/invalid/i)).toBeVisible();
    
    // Verify still on login page
    await expect(page).toHaveURL(/\/login/);
  });

  test('should logout successfully', async ({ page }) => {
    // 1. Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('demo@zeltra.io');
    await page.getByLabel(/password/i).fill('password');
    await page.getByRole('button', { name: /sign in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);

    // 2. Perform logout
    // Assuming there is a user menu or logout button in the layout
    // We might need to adjust the selector based on the actual UI implementation
    const userMenu = page.getByRole('button', { name: /user menu|profile/i });
    if (await userMenu.isVisible()) {
        await userMenu.click();
        await page.getByRole('menuitem', { name: /logout|sign out/i }).click();
    } else {
        // Fallback for simple layouts
        await page.getByRole('button', { name: /logout|sign out/i }).click();
    }

    // 3. Verify redirection to login
    await expect(page).toHaveURL(/\/login/);
  });
});
