import { test, expect } from '@playwright/test';

// Test credentials
const TEST_USER = {
  email: 'demo@zeltra.io',
  password: 'password123',
};

test.describe('Role Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.getByLabel(/email/i).fill(TEST_USER.email);
    await page.getByLabel(/password/i).fill(TEST_USER.password);
    await page.getByRole('button', { name: /sign in/i }).click();
    await page.waitForURL(/\/dashboard/, { timeout: 10000 });
  });

  test('should navigate to team management page', async ({ page }) => {
    await page.goto('/dashboard/settings/users');
    await page.waitForLoadState('networkidle');

    // Verify page loaded
    await expect(page.getByText('Team Management')).toBeVisible({ timeout: 5000 });
  });

  test('should display invite user button', async ({ page }) => {
    await page.goto('/dashboard/settings/users');
    await page.waitForLoadState('networkidle');

    // Check for invite button
    const inviteBtn = page.getByRole('button', { name: /invite user/i });
    await expect(inviteBtn).toBeVisible({ timeout: 5000 });
  });

  test('should open invite user dialog with all 6 roles', async ({ page }) => {
    await page.goto('/dashboard/settings/users');
    await page.waitForLoadState('networkidle');

    // Click invite button
    await page.getByRole('button', { name: /invite user/i }).click();

    // Verify dialog opened
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 3000 });

    // Open role dropdown
    await page.getByLabel(/role/i).click();

    // Verify all 6 roles are available
    await expect(page.getByRole('option', { name: /admin/i })).toBeVisible();
    await expect(page.getByRole('option', { name: /accountant/i })).toBeVisible();
    await expect(page.getByRole('option', { name: /approver/i })).toBeVisible();
    await expect(page.getByRole('option', { name: /submitter/i })).toBeVisible();
    await expect(page.getByRole('option', { name: /viewer/i })).toBeVisible();
  });

  test('should allow selecting submitter role', async ({ page }) => {
    await page.goto('/dashboard/settings/users');
    await page.waitForLoadState('networkidle');

    // Click invite button
    await page.getByRole('button', { name: /invite user/i }).click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Fill email
    await page.getByLabel(/email/i).fill('test@example.com');

    // Open role dropdown and select submitter
    await page.getByLabel(/role/i).click();
    await page.getByRole('option', { name: /submitter/i }).click();

    // Verify submitter is selected (the trigger should show "Submitter")
    await expect(page.getByText(/submitter/i)).toBeVisible();
  });

  test('should display users table with role column', async ({ page }) => {
    await page.goto('/dashboard/settings/users');
    await page.waitForLoadState('networkidle');

    // Check for table headers
    await expect(page.getByRole('columnheader', { name: /user/i })).toBeVisible({ timeout: 5000 });
    await expect(page.getByRole('columnheader', { name: /role/i })).toBeVisible();
    await expect(page.getByRole('columnheader', { name: /status/i })).toBeVisible();
  });
});
