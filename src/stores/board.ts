import { reactive, ref } from "vue";
import type { Board, Card } from "../types/board";
import { emptyBoard, moveCard, validateBoard } from "../utils/board";
import { loadBoard, saveBoard } from "../utils/platform";

export function useBoard() {
  const board = reactive<Board>(emptyBoard());
  const loaded = ref(false),
    saving = ref(false),
    error = ref(""),
    warning = ref("");
  let queue: Promise<void> = Promise.resolve();
  let revision = 0;
  async function load() {
    try {
      const result = await loadBoard();
      Object.assign(board, validateBoard(result.board));
      warning.value = result.warning ?? "";
      error.value = "";
      loaded.value = true;
    } catch (e) {
      error.value = `Could not open your board: ${String(e)}`;
    }
  }
  function persist() {
    if (!loaded.value) return Promise.reject(new Error("Board is not loaded"));
    const snapshot = JSON.parse(JSON.stringify(board)) as Board;
    const thisRevision = ++revision;
    saving.value = true;
    const task = queue.catch(() => {}).then(() => saveBoard(snapshot));
    queue = task;
    task
      .then(
        () => {
          if (thisRevision === revision) error.value = "";
        },
        (e) => {
          error.value = `Changes not saved: ${String(e)}. Keep Life Kanban open and retry.`;
        },
      )
      .finally(() => {
        if (thisRevision === revision) saving.value = false;
      });
    return task;
  }
  const save = () => {
    void persist().catch(() => {});
  };
  function add(columnId: string, title: string) {
    const column = board.columns.find((c) => c.id === columnId);
    if (!column || !title.trim()) return;
    column.cards.push({
      id: crypto.randomUUID(),
      title: title.trim().slice(0, 200),
      description: "",
      tag: "",
      createdAt: new Date().toISOString(),
    });
    save();
  }
  function update(
    id: string,
    fields: Pick<Card, "title" | "description" | "tag">,
  ) {
    const card = board.columns.flatMap((c) => c.cards).find((c) => c.id === id);
    if (!card || !fields.title.trim()) return;
    Object.assign(card, fields, {
      title: fields.title.trim(),
      description: fields.description.trim(),
      tag: fields.tag.trim(),
    });
    save();
  }
  function remove(id: string) {
    const col = board.columns.find((c) =>
      c.cards.some((card) => card.id === id),
    );
    if (col) {
      col.cards.splice(
        col.cards.findIndex((c) => c.id === id),
        1,
      );
      save();
    }
  }
  function move(id: string, columnId: string, index: number) {
    moveCard(board, id, columnId, index);
    save();
  }
  function rename(id: string, name: string) {
    const col = board.columns.find((c) => c.id === id);
    if (col && name.trim()) {
      col.name = name.trim().slice(0, 40);
      save();
    }
  }
  return {
    board,
    loaded,
    saving,
    error,
    warning,
    load,
    persist,
    save,
    add,
    update,
    remove,
    move,
    rename,
    flush: () => queue,
  };
}
