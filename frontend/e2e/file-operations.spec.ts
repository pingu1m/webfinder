import { test, expect } from "@playwright/test";

test.describe("File Operations", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("can create a new file via header button", async ({ page }) => {
    // Click the New File button in sidebar header
    await page.locator("button[title='New File']").click();

    // Dialog should appear
    await expect(page.getByText("New File")).toBeVisible({ timeout: 3000 });

    // Type filename
    const input = page.locator("div.fixed input");
    await input.fill("test-new-file.txt");
    await page.locator("div.fixed button", { hasText: "Confirm" }).click();

    // File should appear in tree
    await expect(page.getByText("test-new-file.txt")).toBeVisible({
      timeout: 5000,
    });
  });

  test("can create a new folder via header button", async ({ page }) => {
    await page.locator("button[title='New Folder']").click();

    await expect(page.getByText("New Folder")).toBeVisible({ timeout: 3000 });

    const input = page.locator("div.fixed input");
    await input.fill("test-new-folder");
    await page.locator("div.fixed button", { hasText: "Confirm" }).click();

    await expect(page.getByText("test-new-folder")).toBeVisible({
      timeout: 5000,
    });
  });

  test("can cancel new file dialog", async ({ page }) => {
    await page.locator("button[title='New File']").click();
    await expect(page.getByText("New File")).toBeVisible({ timeout: 3000 });

    await page.locator("div.fixed button", { hasText: "Cancel" }).click();
    await expect(page.locator("div.fixed")).not.toBeVisible({ timeout: 2000 });
  });

  test("shows open files section after opening a file", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.getByText("Open Files")).toBeVisible({ timeout: 3000 });
  });

  test("can close all open files", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.getByText("Open Files")).toBeVisible({ timeout: 3000 });

    // Click close all button (X next to "Open Files")
    const closeAllBtn = page
      .locator("button[title='Close all']");
    await closeAllBtn.click();

    await expect(page.getByText("Select a file to edit")).toBeVisible({
      timeout: 3000,
    });
  });
});
