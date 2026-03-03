import { test, expect } from "@playwright/test";

test.describe("Context Menu", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("shows context menu on right-click of a file", async ({ page }) => {
    await page.getByText("hello.txt").click({ button: "right" });
    await expect(page.getByText("Rename")).toBeVisible({ timeout: 3000 });
    await expect(page.getByText("Delete")).toBeVisible();
    await expect(page.getByText("Copy")).toBeVisible();
  });

  test("shows extra options for folders", async ({ page }) => {
    const srcFolder = page.locator("button", { hasText: "src" }).first();
    await srcFolder.click({ button: "right" });
    await expect(page.getByText("New File")).toBeVisible({ timeout: 3000 });
    await expect(page.getByText("New Folder")).toBeVisible();
    await expect(page.getByText("Rename")).toBeVisible();
    await expect(page.getByText("Delete")).toBeVisible();
  });

  test("closes context menu on click elsewhere", async ({ page }) => {
    await page.getByText("hello.txt").click({ button: "right" });
    await expect(page.getByText("Rename")).toBeVisible({ timeout: 3000 });

    // Click elsewhere
    await page.locator("body").click();
    await expect(page.getByText("Rename")).not.toBeVisible({ timeout: 2000 });
  });

  test("rename opens dialog with current name", async ({ page }) => {
    await page.getByText("hello.txt").click({ button: "right" });
    await page.getByText("Rename").click();

    const dialog = page.locator("div.fixed");
    await expect(dialog).toBeVisible({ timeout: 3000 });
    const input = dialog.locator("input");
    await expect(input).toHaveValue("hello.txt");
  });

  test("delete shows confirmation dialog", async ({ page }) => {
    // Listen for the native confirm dialog
    page.on("dialog", async (dialog) => {
      expect(dialog.type()).toBe("confirm");
      expect(dialog.message()).toContain("hello.txt");
      await dialog.dismiss();
    });

    await page.getByText("hello.txt").click({ button: "right" });
    await page.getByText("Delete").click();
  });
});
