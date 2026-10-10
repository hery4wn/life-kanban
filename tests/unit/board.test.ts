import { describe, expect, it } from "vitest";
import { emptyBoard, moveCard, validateBoard } from "../../src/utils/board";
const card = (id: string) => ({
  id,
  title: id,
  description: "",
  tag: "",
  createdAt: "2026-10-04T03:00:00Z",
});
describe("card ordering and persistence validation", () => {
  it("reorders within a column using the final index", () => {
    const b = emptyBoard();
    b.columns[0].cards = [card("a"), card("b"), card("c")];
    moveCard(b, "a", "column-0", 2);
    expect(b.columns[0].cards.map((c) => c.id)).toEqual(["b", "c", "a"]);
    moveCard(b, "a", "column-0", 0);
    expect(b.columns[0].cards.map((c) => c.id)).toEqual(["a", "b", "c"]);
  });
  it("moves between columns without duplicating or dropping a card", () => {
    const b = emptyBoard();
    b.columns[0].cards = [card("a"), card("b")];
    b.columns[1].cards = [card("c")];
    moveCard(b, "b", "column-1", 0);
    expect(b.columns[0].cards.map((c) => c.id)).toEqual(["a"]);
    expect(b.columns[1].cards.map((c) => c.id)).toEqual(["b", "c"]);
    moveCard(b, "b", "column-2", 99);
    expect(b.columns[2].cards.map((c) => c.id)).toEqual(["b"]);
    expect(validateBoard(JSON.parse(JSON.stringify(b)))).toEqual(b);
  });
  it("keeps all data for an invalid destination or missing card", () => {
    const b = emptyBoard();
    b.columns[0].cards = [card("a")];
    moveCard(b, "a", "missing", 0);
    moveCard(b, "missing", "column-0", 0);
    expect(b.columns[0].cards.map((c) => c.id)).toEqual(["a"]);
  });
  it("rejects damaged files and duplicate IDs instead of replacing them with defaults", () => {
    expect(() => validateBoard({})).toThrow();
    const b = emptyBoard();
    b.columns[0].cards = [card("a"), card("a")];
    expect(() => validateBoard(b)).toThrow("duplicate");
    b.columns[0].cards = [card("a")];
    b.settings.opacity = 0;
    expect(() => validateBoard(b)).toThrow("settings");
  });
});

it("migrates original boards and preserves hidden legacy card details", () => {
  const b = emptyBoard();
  b.columns[0].cards = [
    { ...card("a"), description: "Existing private note", tag: "Work" },
  ];
  const old = JSON.parse(JSON.stringify(b));
  delete old.quickTasks;
  delete old.timer;
  const loaded = validateBoard(old);
  expect(loaded.quickTasks).toEqual([]);
  expect(loaded.timer.durationSeconds).toBe(1500);
  expect(loaded.columns[0].cards[0].description).toBe("Existing private note");
});
it("rejects duplicate quick task ids and invalid timer state", () => {
  const b = emptyBoard();
  b.quickTasks = [
    { id: "a", title: "Task", done: false },
    { id: "a", title: "Task 2", done: true },
  ];
  expect(() => validateBoard(b)).toThrow();
  b.quickTasks = [];
  b.timer.endsAt = NaN;
  expect(() => validateBoard(b)).toThrow("timer");
});
