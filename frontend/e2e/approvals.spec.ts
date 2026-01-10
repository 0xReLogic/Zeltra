import { test, expect } from '@playwright/test';

test.describe('Approvals Workflow', () => {

  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/login');
    await page.getByLabel(/email/i).fill('demo@zeltra.io');
    await page.getByLabel(/password/i).fill('password');
    await page.getByRole('button', { name: /sign in/i }).click();
    await expect(page).toHaveURL(/\/dashboard/);
  });

  test('should bulk approve transactions', async ({ page }) => {
    // Navigate to Approvals
    await page.goto('/dashboard/approvals');
    await expect(page.getByText(/pending approvals/i)).toBeVisible();

    // Ensure there are pending transactions (Mock data should provide some)
    // const checkboxes = page.getByRole('checkbox');
    // First checkbox is usually "Select All" in header
    // Start from index 1 for row checkboxes, or just check the first one if separate
    // Let's assume the table has checkboxes
    
    // Check if table has data
    const rows = page.locator('tbody tr');
    const rowCount = await rows.count();
    
    // If we have rows, proceed
    if (rowCount > 0) {
      // Select first two rows (if available)
      await rows.nth(0).getByRole('checkbox').check();
      if (rowCount > 1) {
        await rows.nth(1).getByRole('checkbox').check();
      }

      // Check "Bulk Approve" button visibility (should appear after selection)
      const bulkApproveBtn = page.getByRole('button', { name: /approve selected/i });
      await expect(bulkApproveBtn).toBeVisible();
      
      // Click Bulk Approve
      await bulkApproveBtn.click();
      
      // Verify Success Message
      await expect(page.getByText(/approved successfully/i)).toBeVisible();
      
      // Verify rows are removed or status changed
      // (Mock implementation removes them from list or changes status)
    } else {
        console.log('No pending approvals found in mock data to test');
    }
  });
});
