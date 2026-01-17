import { test as setup, expect } from '@playwright/test';
import path from 'path';

const authFile = path.join(__dirname, '../playwright/.auth/user.json');

setup('authenticate', async ({ page }) => {
  // Navigate to login page
  await page.goto('http://10.0.0.5:3000/login');

  // Fill in login credentials
  await page.getByRole('textbox', { name: 'Email' }).fill('corp@zeltra.io');
  await page.getByRole('textbox', { name: 'Password' }).fill('qwertyui');

  // Click sign in button
  await page.getByRole('button', { name: 'Sign in' }).click();

  // Wait for redirect to dashboard
  await page.waitForURL('**/dashboard');

  // Verify we're logged in by checking for user info
  await expect(page.getByText('Reza Febryan')).toBeVisible();

  // Save signed-in state to 'playwright/.auth/user.json'
  // This includes localStorage with JWT tokens
  await page.context().storageState({ path: authFile });

  console.log('✅ Authentication setup complete - storage state saved');
});
