import { test, expect } from "@playwright/test";

test.describe("Runner", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("text=Explorer");
  });

  test("shows Run button for Python files", async ({ page }) => {
    await page.getByText("script.py").click();
    await expect(page.getByRole("button", { name: /Run/ })).toBeVisible({ timeout: 10000 });
  });

  test("clicking Run shows the output panel", async ({ page }) => {
    await page.getByText("script.py").click();

    const runBtn = page.getByRole("button", { name: /Run/ }).first();
    await expect(runBtn).toBeVisible({ timeout: 10000 });
    await runBtn.click();

    // Output panel header button should appear
    await expect(page.getByRole("button", { name: "Output" })).toBeVisible({ timeout: 5000 });
  });

  test("shows output from runner execution", async ({ page }) => {
    await page.getByText("script.py").click();

    const runBtn = page.getByRole("button", { name: /Run/ }).first();
    await expect(runBtn).toBeVisible({ timeout: 10000 });
    await runBtn.click();

    // The output appears in the dark-background output panel
    const outputArea = page.locator("[class*='font-mono'][class*='bg-']");
    await expect(outputArea.getByText("Hello from Python!")).toBeVisible({
      timeout: 15000,
    });
  });

  test("output panel shows exit code or completion", async ({ page }) => {
    await page.getByText("script.py").click();

    const runBtn = page.getByRole("button", { name: /Run/ }).first();
    await expect(runBtn).toBeVisible({ timeout: 10000 });
    await runBtn.click();

    // Wait for the process to produce output (proves execution happened)
    const outputArea = page.locator("[class*='font-mono'][class*='bg-']");
    await expect(outputArea.getByText("Hello from Python!")).toBeVisible({
      timeout: 15000,
    });

    // The running indicator (green pulse dot) should eventually disappear
    // OR an exit code badge should appear
    // Wait for a reasonable time for the process to complete
    await page.waitForTimeout(3000);

    // At this point, the process should have finished.
    // Check that the run button is no longer showing "Running..."
    await expect(page.getByText("Running...")).not.toBeVisible({ timeout: 5000 });
  });

  test("can clear output lines", async ({ page }) => {
    await page.getByText("script.py").click();

    const runBtn = page.getByRole("button", { name: /Run/ }).first();
    await expect(runBtn).toBeVisible({ timeout: 10000 });
    await runBtn.click();

    // Wait for output area to have content
    const outputArea = page.locator("[class*='font-mono'][class*='bg-']");
    await expect(outputArea.getByText("Hello from Python!")).toBeVisible({
      timeout: 15000,
    });

    // Clear
    await page.locator("button[title='Clear']").click();

    // The output area should no longer contain the Python output
    await expect(outputArea.getByText("Hello from Python!")).not.toBeVisible({
      timeout: 3000,
    });
  });
});
