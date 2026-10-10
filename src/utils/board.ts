import type { Board } from "../types/board";
export const defaultShortcut = "Control+Shift+K";
export function emptyBoard(): Board {
  return {
    version: 1,
    quickTasks: [],
    timer: { durationSeconds: 1500, remainingSeconds: 1500, endsAt: null },
    columns: ["Backlog", "To do", "In progress", "Done"].map((name, i) => ({
      id: `column-${i}`,
      name,
      cards: [],
    })),
    settings: {
      opacity: 85,
      blur: 12,
      theme: "system",
      shortcut: defaultShortcut,
    },
  };
}
export function validateBoard(value: unknown): Board {
  const b = value as Board;
  if (
    !b ||
    b.version !== 1 ||
    !Array.isArray(b.columns) ||
    b.columns.length !== 4
  )
    throw new Error(
      "Unrecognized board format. Your existing file has been preserved.",
    );
  const ids = new Set<string>();
  const id = (x: unknown) => {
    if (typeof x !== "string" || !x || ids.has(x))
      throw new Error("Invalid or duplicate ID");
    ids.add(x);
  };
  for (const col of b.columns) {
    id(col.id);
    if (
      typeof col.name !== "string" ||
      !col.name.trim() ||
      col.name.length > 40 ||
      !Array.isArray(col.cards)
    )
      throw new Error("Invalid column");
    for (const card of col.cards) {
      id(card.id);
      if (
        typeof card.title !== "string" ||
        !card.title.trim() ||
        card.title.length > 200 ||
        typeof card.description !== "string" ||
        card.description.length > 2000 ||
        typeof card.tag !== "string" ||
        card.tag.length > 40 ||
        typeof card.createdAt !== "string" ||
        !Number.isFinite(Date.parse(card.createdAt))
      )
        throw new Error("Invalid card");
    }
  }
  // Older boards acquire the new widgets without changing existing cards.
  b.quickTasks ??= [];
  b.timer ??= { durationSeconds: 1500, remainingSeconds: 1500, endsAt: null };
  if (!Array.isArray(b.quickTasks)) throw new Error("Invalid quick tasks");
  for (const task of b.quickTasks) {
    id(task.id);
    if (
      typeof task.title !== "string" ||
      !task.title.trim() ||
      task.title.length > 120 ||
      typeof task.done !== "boolean"
    )
      throw new Error("Invalid quick task");
  }
  const t = b.timer;
  if (
    !Number.isInteger(t.durationSeconds) ||
    t.durationSeconds < 1 ||
    t.durationSeconds > 10859 ||
    !Number.isFinite(t.remainingSeconds) ||
    t.remainingSeconds < 0 ||
    t.remainingSeconds > t.durationSeconds ||
    (t.endsAt !== null && (!Number.isSafeInteger(t.endsAt) || t.endsAt < 0))
  )
    throw new Error("Invalid timer");
  const s = b.settings;
  if (
    !s ||
    !Number.isFinite(s.opacity) ||
    s.opacity < 35 ||
    s.opacity > 100 ||
    !Number.isFinite(s.blur) ||
    s.blur < 0 ||
    s.blur > 24 ||
    !["system", "light", "dark"].includes(s.theme) ||
    typeof s.shortcut !== "string" ||
    !s.shortcut
  )
    throw new Error("Invalid settings");
  return b;
}
// targetIndex is measured AFTER removing the card, matching Sortable's newIndex.
export function moveCard(
  board: Board,
  cardId: string,
  targetId: string,
  targetIndex: number,
): void {
  const source = board.columns.find((col) =>
    col.cards.some((c) => c.id === cardId),
  );
  const target = board.columns.find((col) => col.id === targetId);
  if (!source || !target) return;
  const [card] = source.cards.splice(
    source.cards.findIndex((c) => c.id === cardId),
    1,
  );
  target.cards.splice(
    Math.max(0, Math.min(targetIndex, target.cards.length)),
    0,
    card,
  );
}
