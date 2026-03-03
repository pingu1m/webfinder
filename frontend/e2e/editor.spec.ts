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
    await page.getByText("script.py").click();

    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });
  });

  test("shows dirty indicator after editing", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });

    // Click into Monaco to focus, then type
    await page.locator(".monaco-editor").click();
    await page.keyboard.type("X");
    await page.waitForTimeout(500);

    // Dirty indicator (yellow dot) should appear in the tab
    await expect(page.locator(".border-b-primary .text-yellow-500")).toBeVisible({
      timeout: 3000,
    });
  });

  test("shows Save button when file is dirty", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });

    await page.locator(".monaco-editor").click();
    await page.keyboard.type("Z");
    await page.waitForTimeout(500);

    await expect(page.getByRole("button", { name: /Save/ })).toBeVisible({
      timeout: 3000,
    });
  });

  test("Cmd+S saves and persists content across reopen", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });

    // Select all existing content and replace with unique text
    const uniqueText = `saved-${Date.now()}`;
    await page.locator(".monaco-editor").click();
    await page.keyboard.press("Meta+a");
    await page.keyboard.type(uniqueText);
    await page.waitForTimeout(300);

    // Save via keyboard shortcut
    await page.keyboard.press("Meta+s");
    await page.waitForTimeout(1000);

    // Close the file
    const activeTab = page.locator(".border-b-primary");
    await activeTab.hover();
    await activeTab.locator("button").click();
    await expect(page.getByText("Select a file to edit")).toBeVisible({ timeout: 3000 });

    // Reopen — content should be persisted
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });
    await page.waitForTimeout(1000);

    // Verify the content was saved
    const editorContent = await page.locator(".monaco-editor .view-lines").textContent();
    expect(editorContent).toContain(uniqueText);
  });

  test("does not show Run button for non-runnable files", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });

    // hello.txt is a .txt file — no runner configured
    await expect(page.getByRole("button", { name: /Run/ })).not.toBeVisible({
      timeout: 2000,
    });
  });
});
