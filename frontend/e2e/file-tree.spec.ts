import { test, expect } from "@playwright/test";

test.describe("File Tree", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("renders the Explorer sidebar", async ({ page }) => {
    await expect(page.getByText("Explorer")).toBeVisible();
  });

  test("displays project files section", async ({ page }) => {
    await expect(page.getByText("Project Files")).toBeVisible();
  });

  test("shows files from the test fixture", async ({ page }) => {
    await expect(page.getByText("hello.txt")).toBeVisible({ timeout: 5000 });
  });

  test("shows directories from the test fixture", async ({ page }) => {
    await expect(page.getByText("src")).toBeVisible({ timeout: 5000 });
  });

  test("can expand a folder by clicking it", async ({ page }) => {
    // The src folder button is in the sidebar — click on it to toggle
    const srcButton = page.locator("button").filter({ hasText: /^src$/ }).first();
    await expect(srcButton).toBeVisible({ timeout: 5000 });

    // src starts collapsed (depth >= 1). Click to expand.
    await srcButton.click();

    // After expanding, children should appear
    await expect(page.getByText("main.rs")).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("lib.rs")).toBeVisible({ timeout: 5000 });
  });

  test("can collapse a folder", async ({ page }) => {
    const srcButton = page.locator("button").filter({ hasText: /^src$/ }).first();
    await expect(srcButton).toBeVisible({ timeout: 5000 });

    // Expand
    await srcButton.click();
    await expect(page.getByText("main.rs")).toBeVisible({ timeout: 5000 });

    // Collapse
    await srcButton.click();
    await expect(page.getByText("main.rs")).not.toBeVisible({ timeout: 3000 });
  });

  test("search filters files", async ({ page }) => {
    const searchInput = page.getByPlaceholder("Search files...");
    await searchInput.fill("hello");
    await expect(page.getByText("hello.txt")).toBeVisible();
    await expect(page.getByText("script.py")).not.toBeVisible();
  });

  test("clearing search shows all files again", async ({ page }) => {
    const searchInput = page.getByPlaceholder("Search files...");
    await searchInput.fill("hello");
    await expect(page.getByText("script.py")).not.toBeVisible();
    await searchInput.clear();
    await expect(page.getByText("script.py")).toBeVisible({ timeout: 3000 });
  });
});
