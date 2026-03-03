import { test, expect } from "@playwright/test";

test.describe("Editor", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("shows placeholder when no file is open", async ({ page }) => {
    await expect(page.getByText("Select a file to edit")).toBeVisible();
  });

  test("opens a file when clicked in tree", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".border-b-primary").getByText("hello.txt")).toBeVisible({
      timeout: 5000,
    });
  });

  test("shows file path in breadcrumb", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".font-mono").getByText("hello.txt")).toBeVisible({
      timeout: 5000,
    });
  });

  test("can switch between tabs", async ({ page }) => {
    // Open first file
    await page.getByText("hello.txt").first().click();
    await expect(page.locator(".border-b-primary").getByText("hello.txt")).toBeVisible({
      timeout: 5000,
    });

    // Open second file
    await page.getByText("readme.md").first().click();
    await expect(page.locator(".border-b-primary").getByText("readme.md")).toBeVisible({
      timeout: 5000,
    });

    // Switch back to first tab by clicking the tab (not the sidebar)
    const tabs = page.locator("[class*='cursor-pointer'][class*='select-none']");
    const helloTab = tabs.filter({ hasText: "hello.txt" });
    await helloTab.click();
    await expect(page.locator(".border-b-primary").getByText("hello.txt")).toBeVisible({
      timeout: 3000,
    });
  });

  test("can close a tab", async ({ page }) => {
    await page.getByText("hello.txt").first().click();
    await expect(page.locator(".border-b-primary").getByText("hello.txt")).toBeVisible({
      timeout: 5000,
    });

    // Click the close button on the tab
    const activeTab = page.locator(".border-b-primary");
    await activeTab.hover();
    const closeBtn = activeTab.locator("button");
    await closeBtn.click();

    await expect(page.getByText("Select a file to edit")).toBeVisible({ timeout: 3000 });
  });

  test("loads Monaco editor for code files", async ({ page }) => {
    // Click script.py directly (it's at root level, no folder expansion needed)
    await page.getByText("script.py").click();

    // Wait for Monaco to lazy-load — can take a few seconds
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });
  });
});
