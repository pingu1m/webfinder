import { test, expect } from "@playwright/test";

test.describe("Settings Panel", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("opens settings panel via sidebar button", async ({ page }) => {
    await page.locator("button[title='Settings']").click();
    await expect(page.getByText("Settings").first()).toBeVisible({ timeout: 3000 });
    await expect(page.getByText("Auto-save")).toBeVisible({ timeout: 3000 });
  });

  test("settings panel shows all controls", async ({ page }) => {
    await page.locator("button[title='Settings']").click();
    await expect(page.getByText("Auto-save")).toBeVisible({ timeout: 3000 });
    await expect(page.getByText("Font size", { exact: true })).toBeVisible();
    await expect(page.getByText("Tab size", { exact: true })).toBeVisible();
    await expect(page.getByText("Word wrap", { exact: true })).toBeVisible();
    await expect(page.getByText("Theme", { exact: true })).toBeVisible();
  });

  test("close button returns to editor", async ({ page }) => {
    await page.getByText("hello.txt").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 15000 });

    await page.locator("button[title='Settings']").click();
    await expect(page.getByText("Auto-save")).toBeVisible({ timeout: 3000 });
    await expect(page.locator(".monaco-editor")).not.toBeVisible();

    await page.locator("button[title='Back to editor']").click();
    await expect(page.locator(".monaco-editor")).toBeVisible({ timeout: 10000 });
  });

  test("toggling auto-save persists to backend", async ({ page, request }) => {
    await page.locator("button[title='Settings']").click();
    await expect(page.getByText("Auto-save")).toBeVisible({ timeout: 3000 });

    const toggle = page.locator("button[role='switch']").first();
    await toggle.click();
    // Wait for debounced API call
    await page.waitForTimeout(500);

    const res = await request.get("/api/info");
    const info = await res.json();
    expect(info.config.auto_save).toBe(true);
  });
});
