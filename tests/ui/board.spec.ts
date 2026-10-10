import { test, expect, type Page } from "@playwright/test";
async function edit(page: Page) {
  await page.getByRole("button", { name: "Edit board" }).click();
}
async function add(page: Page, title: string, col = "Backlog") {
  const column = page.getByRole("region", { name: col, exact: true });
  if (!(await column.getByRole("textbox", { name: "New card title" }).count()))
    await column.getByRole("button", { name: `Add card to ${col}` }).click();
  await column.getByRole("textbox", { name: "New card title" }).fill(title);
  await column.getByRole("textbox", { name: "New card title" }).press("Enter");
}
test.beforeEach(async ({ page }) => {
  await page.goto("/");
});
test("CRUD, safe deletion, column names and persistence survive reload", async ({
  page,
}) => {
  await edit(page);
  await add(page, "Read a chapter");
  await page.getByRole("button", { name: "Edit Read a chapter" }).click();
  await page.getByLabel("Title", { exact: true }).fill("Read two chapters");
  await expect(page.getByLabel("Description")).toHaveCount(0);
  await expect(page.getByLabel("Tag", { exact: false })).toHaveCount(0);
  await page.getByLabel("Column", { exact: true }).selectOption("column-1");
  await page.getByRole("button", { name: "Save card" }).click();
  await page.getByRole("button", { name: "Rename Backlog" }).click();
  await page.getByRole("textbox", { name: "Column name" }).fill("Someday");
  await page.getByRole("textbox", { name: "Column name" }).press("Enter");
  await page.reload();
  await expect(page.getByRole("heading", { name: "Someday" })).toBeVisible();
  const todo = page.getByRole("region", { name: "To do", exact: true });
  await expect(todo.getByRole("article")).toContainText("Read two chapters");
  await expect(todo.getByRole("article")).not.toContainText("Personal");
  await expect(page.getByRole("button", { name: /Delete Read/ })).toHaveCount(
    0,
  );
  await edit(page);
  await todo.getByRole("article").focus();
  await page.keyboard.press("Delete");
  await expect(page.getByRole("dialog")).toContainText("Let this one go?");
  await page.keyboard.press("Escape");
  await expect(todo.getByRole("article")).toHaveCount(1);
  await page.getByRole("button", { name: "Delete Read two chapters" }).click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Delete card", exact: true })
    .click();
  await page.reload();
  await expect(page.getByRole("article")).toHaveCount(0);
});
test("pointer drag moves cards across columns and reorders within a column", async ({
  page,
}) => {
  await edit(page);
  await add(page, "First");
  await add(page, "Second");
  await add(page, "Third");
  await page.getByRole("button", { name: "Close", exact: true }).click();
  const source = page.getByRole("article", { name: "First", exact: true });
  const target = page.locator('[data-column="column-1"] .notes');
  const start = (await source.boundingBox())!,
    end = (await target.boundingBox())!;
  await page.mouse.move(start.x + 60, start.y + 50);
  await page.mouse.down();
  await page.mouse.move(start.x + 70, start.y + 55, { steps: 5 });
  await page.mouse.move(end.x + 50, end.y + 8, { steps: 25 });
  await page.waitForTimeout(250);
  await page.mouse.up();
  await expect(page.locator('[data-column="column-1"] article')).toHaveText(
    /First/,
  );
  const second = (await page
    .getByRole("article", { name: "Second", exact: true })
    .boundingBox())!;
  const third = (await page
    .getByRole("article", { name: "Third", exact: true })
    .boundingBox())!;
  await page.mouse.move(third.x + 60, third.y + 50);
  await page.mouse.down();
  await page.mouse.move(third.x + 70, third.y + 45, { steps: 5 });
  await page.mouse.move(second.x + 65, second.y + 8, { steps: 25 });
  await page.waitForTimeout(250);
  await page.mouse.up();
  await expect(page.locator('[data-column="column-0"] article h3')).toHaveText([
    "Third",
    "Second",
  ]);
  await page.reload();
  await expect(page.locator('[data-column="column-0"] article h3')).toHaveText([
    "Third",
    "Second",
  ]);
  await expect(page.locator('[data-column="column-1"] article h3')).toHaveText([
    "First",
  ]);
});
test("keyboard move, mode transitions, theme and opacity", async ({ page }) => {
  await edit(page);
  await add(page, "Keyboard card");
  await page.getByRole("article").focus();
  await page.keyboard.press("Alt+ArrowRight");
  await expect(page.locator('[data-column="column-1"] article')).toHaveCount(1);
  await expect(page.locator('[data-column="column-1"] article')).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(
    page.getByRole("button", { name: "Finish editing" }),
  ).toHaveCount(0);
  await page.keyboard.press("Control+Shift+K");
  await expect(
    page.getByRole("button", { name: "Finish editing" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await page.getByLabel("Appearance").selectOption("dark");
  await page.getByLabel("Board opacity").fill("60");
  await page.getByRole("button", { name: "Save settings" }).click();
  await expect(page.locator(".workspace")).toHaveClass(/is-dark/);
  await page.reload();
  await expect(page.locator(".workspace")).toHaveClass(/is-dark/);
  await expect(page.locator(".workspace")).toHaveAttribute("style", /0.6/);
});
test("damaged browser data is preserved, with recovery feedback", async ({
  page,
}) => {
  await page.evaluate(() =>
    localStorage.setItem("life-kanban-browser-preview-v1", "{bad"),
  );
  await page.reload();
  await expect(page.getByRole("alert")).toContainText(
    "Could not open your board",
  );
  expect(
    await page.evaluate(() =>
      localStorage.getItem("life-kanban-browser-preview-v1"),
    ),
  ).toBe("{bad");
});
for (const [width, height] of [
  [1366, 768],
  [1920, 1080],
  [2560, 1440],
  [911, 512],
]) {
  test(`fits ${width}×${height} without horizontal clipping`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height });
    await edit(page);
    await add(
      page,
      "A long intention that still wraps comfortably within its note",
    );
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await expect(
      page.getByRole("button", { name: "Finish editing" }),
    ).toBeInViewport();
  });
}

test("quick tasks are independent and persist after reload", async ({
  page,
}) => {
  await edit(page);
  await page.getByLabel("New quick task").fill("Water the plants");
  await page
    .getByRole("button", { name: "Add quick task", exact: true })
    .click();
  await page.getByLabel("Water the plants", { exact: true }).check();
  await page.reload();
  await expect(
    page.getByLabel("Water the plants", { exact: true }),
  ).toBeChecked();
  await expect(page.getByRole("article")).toHaveCount(0);
  await edit(page);
  await page
    .getByRole("button", { name: "Remove Water the plants", exact: true })
    .click();
  await page.reload();
  await expect(
    page.getByLabel("Water the plants", { exact: true }),
  ).toHaveCount(0);
});
test("calendar handles month boundaries and returns to today", async ({
  page,
}) => {
  await edit(page);
  const original = await page.locator(".calendar-head>span").textContent();
  await page.getByRole("button", { name: "Next month" }).click();
  await expect(page.locator(".calendar-head>span")).not.toHaveText(original!);
  await page.getByRole("button", { name: "Previous month" }).click();
  await expect(page.locator(".calendar-head>span")).toHaveText(original!);
  await page.getByRole("button", { name: "Today", exact: false }).click();
  await expect(page.locator(".calendar .day.selected")).toHaveCount(1);
});
test("timer pauses, resumes across reload, finishes and resets", async ({
  page,
}) => {
  await page.clock.install({ time: new Date("2026-10-10T10:25:00+08:00") });
  await page.reload();
  await edit(page);
  await page.getByLabel("Timer minutes").fill("0");
  await page.getByLabel("Timer seconds").fill("8");
  await page.getByRole("button", { name: "Start", exact: true }).click();
  await page.clock.fastForward(2000);
  await page.getByRole("button", { name: "Pause", exact: true }).click();
  const paused = await page.locator("#timer-display").textContent();
  await page.clock.fastForward(10000);
  await expect(page.locator("#timer-display")).toHaveText(paused!);
  await page.getByRole("button", { name: "Resume", exact: true }).click();
  await page.reload();
  await expect(page.locator("#timer-display")).not.toHaveText("25:00");
  await page.clock.fastForward(10000);
  await expect(page.locator(".time-widget")).toHaveClass(/finished/);
  await edit(page);
  await page.getByRole("button", { name: "Reset timer", exact: true }).click();
  await expect(page.locator("#timer-display")).toHaveText("00:08");
});
