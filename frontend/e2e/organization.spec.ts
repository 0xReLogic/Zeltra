import { test, expect } from '@playwright/test';

// Test credentials
const TEST_USER = {
  email: 'demo@zeltra.io',
  password: 'password123',
};

test.describe('Organization Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill(TEST_USER.email);
    await page.getByLabel(/password/i).fill(TEST_USER.password);
    await page.getByRole('button', { name: /sign in/i }).click();
    await page.waitForURL(/\/dashboard/, { timeout: 10000 });
  });

  test('should navigate to organization settings', async ({ page }) => {
    // Navigate to settings
    await page.goto('/dashboard/settings/organization');
    await page.waitForLoadState('networkidle');

    // Verify page loaded
    await expect(page.getByText('Organization Settings')).toBeVisible({ timeout: 5000 });
  });

  test('should display create organization button', async ({ page }) => {
    await page.goto('/dashboard/settings/organization');
    await page.waitForLoadState('networkidle');

    // Check for create button
    const createBtn = page.getByRole('button', { name: /create organization/i });
    await expect(createBtn).toBeVisible({ timeout: 5000 });
  });

  test('should open create organization dialog', async ({ page }) => {
    await page.goto('/dashboard/settings/organization');
    await page.waitForLoadState('networkidle');

    // Click create button
    await page.getByRole('button', { name: /create organization/i }).click();

    // Verify dialog opened
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 3000 });
    await expect(page.getByText('Create Organization')).toBeVisible();

    // Verify form fields
    await expect(page.getByLabel(/organization name/i)).toBeVisible();
    await expect(page.getByLabel(/url slug/i)).toBeVisible();
    await expect(page.getByLabel(/base currency/i)).toBeVisible();
  });

  test('should validate slug format', async ({ page }) => {
    await page.goto('/dashboard/settings/organization');
    await page.waitForLoadState('networkidle');

    // Open dialog
    await page.getByRole('button', { name: /create organization/i }).click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Fill invalid slug
    await page.getByLabel(/organization name/i).fill('Test Org');
    await page.getByLabel(/url slug/i).clear();
    await page.getByLabel(/url slug/i).fill('Invalid Slug!@#');

    // Try to submit
    await page.getByRole('button', { name: /create organization/i }).last().click();

    // Should show validation error
    await expect(page.getByText(/lowercase|letters|numbers|hyphens/i)).toBeVisible({ timeout: 3000 });
  });

  test('should auto-generate slug from name', async ({ page }) => {
    await page.goto('/dashboard/settings/organization');
    await page.waitForLoadState('networkidle');

    // Open dialog
    await page.getByRole('button', { name: /create organization/i }).click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Fill name
    await page.getByLabel(/organization name/i).fill('My Test Company');

    // Check slug was auto-generated
    const slugInput = page.getByLabel(/url slug/i);
    await expect(slugInput).toHaveValue('my-test-company');
  });
});
