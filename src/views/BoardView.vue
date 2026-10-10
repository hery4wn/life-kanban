<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import DailyWidgets from "../components/DailyWidgets.vue";
import FrogFriend from "../components/FrogFriend.vue";
import BoardColumn from "../components/BoardColumn.vue";
import CardEditor from "../components/CardEditor.vue";
import SettingsPanel from "../components/SettingsPanel.vue";
import ModalShell from "../components/ModalShell.vue";
import { useBoard } from "../stores/board";
import { useDesktop } from "../composables/useDesktop";
import { desktop } from "../utils/platform";
import type { Card, Settings } from "../types/board";
const store = useBoard(),
  native = useDesktop();
const { board, loaded, saving, error, warning } = store;
const { editing, autostart } = native;
const selected = ref<string | null>(null),
  activeCard = ref<Card | null>(null),
  deleting = ref<Card | null>(null),
  settingsOpen = ref(false);
const total = computed(() =>
  board.columns.reduce((n, col) => n + col.cards.length, 0),
);
const shortcutLabel = computed(() =>
  native.shortcut.value.replaceAll("Control", "Ctrl").replaceAll("+", " + "),
);
const systemDark = ref(matchMedia("(prefers-color-scheme: dark)").matches);
const dark = computed(
  () =>
    board.settings.theme === "dark" ||
    (board.settings.theme === "system" && systemDark.value),
);
const allErrors = computed(() =>
  [error.value, native.error.value].filter(Boolean).join(" "),
);
const colFor = (id: string) =>
  board.columns.find((c) => c.cards.some((card) => card.id === id))!;
watch(editing, (value) => {
  if (!value) {
    activeCard.value = null;
    deleting.value = null;
    settingsOpen.value = false;
    selected.value = null;
  }
});
async function finish() {
  try {
    await store.flush();
    await native.setEditing(false);
  } catch {
    /* Keep recovery controls available when saving failed. */
  }
}
function keydown(event: KeyboardEvent) {
  if (!desktop && event.ctrlKey && event.shiftKey && event.code === "KeyK") {
    event.preventDefault();
    void native.setEditing(!editing.value);
    return;
  }
  if (
    event.key === "Escape" &&
    editing.value &&
    !document.querySelector("dialog[open]")
  ) {
    event.preventDefault();
    void finish();
  }
}
async function keyboardMove(id: string, direction: string) {
  if (!editing.value) return;
  const source = colFor(id),
    colIndex = board.columns.indexOf(source),
    cardIndex = source.cards.findIndex((c) => c.id === id);
  if (direction === "up" || direction === "down")
    store.move(id, source.id, cardIndex + (direction === "up" ? -1 : 1));
  else {
    const target = board.columns[colIndex + (direction === "left" ? -1 : 1)];
    if (target) store.move(id, target.id, target.cards.length);
  }
  await nextTick();
  document
    .querySelector<HTMLElement>(`[data-card-id="${CSS.escape(id)}"]`)
    ?.focus();
}
function saveCard(
  fields: Pick<Card, "title" | "description" | "tag">,
  columnId: string,
) {
  const id = activeCard.value!.id;
  store.update(id, fields);
  if (colFor(id).id !== columnId)
    store.move(
      id,
      columnId,
      board.columns.find((c) => c.id === columnId)!.cards.length,
    );
  activeCard.value = null;
}
async function applySettings(settings: Settings, startup: boolean) {
  const previous = { ...board.settings },
    oldAutostart = autostart.value;
  try {
    if (settings.shortcut !== native.shortcut.value)
      await native.setShortcut(settings.shortcut);
    if (startup !== autostart.value) await native.setAutostart(startup);
    Object.assign(board.settings, settings);
    await store.persist();
  } catch (e) {
    Object.assign(board.settings, previous);
    try {
      if (native.shortcut.value !== previous.shortcut)
        await native.setShortcut(previous.shortcut);
      if (autostart.value !== oldAutostart)
        await native.setAutostart(oldAutostart);
    } catch (rollback) {
      native.error.value = `Could not restore Windows settings: ${String(rollback)}`;
    }
    throw e;
  }
}
const media = matchMedia("(prefers-color-scheme: dark)");
const themeChange = (event: MediaQueryListEvent) => {
  systemDark.value = event.matches;
};
onMounted(async () => {
  document.addEventListener("keydown", keydown);
  media.addEventListener("change", themeChange);
  await store.load();
  try {
    await native.initialize(
      () => {
        activeCard.value = null;
        deleting.value = null;
        settingsOpen.value = true;
      },
      async () => {
        try {
          await store.flush();
          await invoke("quit_app");
        } catch {
          await native.setEditing(true);
        }
      },
    );
  } catch (e) {
    native.error.value = String(e);
  }
});
onBeforeUnmount(() => {
  native.dispose();
  document.removeEventListener("keydown", keydown);
  media.removeEventListener("change", themeChange);
});
</script>
<template>
  <main
    class="workspace"
    :class="{
      'is-editing': editing,
      'is-dark': dark,
      'browser-preview': !desktop,
    }"
    :style="{
      '--surface-alpha': String(board.settings.opacity / 100),
      '--surface-blur': `${board.settings.blur}px`,
    }"
  >
    <div class="app">
      <header class="topbar">
        <div class="brand">
          <span class="logo" aria-hidden="true"><i></i><i></i><i></i></span>Life
          Kanban
        </div>
        <div class="top-actions">
          <span v-if="!desktop" class="preview-badge"
            >Browser preview · separate data</span
          >
          <template v-if="editing"
            ><span class="save-status" role="status">{{
              saving ? "Saving…" : error ? "Not saved" : "Saved on this device"
            }}</span
            ><button
              class="text-button"
              :disabled="!loaded"
              @click="settingsOpen = true"
            >
              Settings</button
            ><button class="mode" @click="finish">
              <span class="mode-dot"></span>Finish editing ↗
            </button></template
          >
          <button
            v-else-if="!desktop"
            class="mode"
            @click="native.setEditing(true)"
          >
            Edit board ↗</button
          ><span v-else class="quiet-label">{{ shortcutLabel }} to edit</span>
        </div>
      </header>
      <div class="layout">
        <div class="main">
          <div class="section-head">
            <div class="section-title">
              <h2>My projects</h2>
              <span class="project-count">{{ total }}</span>
            </div>
            <FrogFriend />
          </div>
          <div v-if="allErrors || warning" class="notice" role="alert">
            <span>{{ allErrors || warning }}</span
            ><button
              v-if="error && loaded && editing"
              class="text-button"
              @click="store.save"
            >
              Retry save</button
            ><button
              v-if="!loaded && editing"
              class="text-button"
              @click="store.load"
            >
              Retry opening
            </button>
          </div>
          <div
            v-if="loaded"
            class="board"
            :aria-label="
              editing ? 'Editable Kanban board' : 'Kanban board in View Mode'
            "
          >
            <BoardColumn
              v-for="(column, index) in board.columns"
              :key="column.id"
              :column="column"
              :index="index"
              :editing="editing"
              :selected="selected"
              @add="store.add(column.id, $event)"
              @edit="activeCard = $event"
              @select="selected = $event"
              @remove="deleting = $event"
              @drop="store.move"
              @rename="store.rename(column.id, $event)"
              @keyboard-move="keyboardMove"
            />
          </div>
          <footer class="board-footer">
            <span v-if="editing"
              >Drag to move · Double-click to edit · Alt + arrows to move · Esc
              to finish</span
            ><span v-else>One little hop at a time.</span>
          </footer>
        </div>
        <DailyWidgets
          v-if="loaded"
          :board="board"
          :editing="editing"
          @save="store.save"
        />
      </div>
    </div>
    <CardEditor
      v-if="activeCard && editing && !deleting"
      :card="activeCard"
      :columns="board.columns"
      :column-id="colFor(activeCard.id).id"
      @close="activeCard = null"
      @save="saveCard"
      @remove="deleting = activeCard"
    />
    <ModalShell
      v-if="deleting && editing"
      title="Let this one go?"
      @close="deleting = null"
      ><p class="delete-message">
        Delete “{{ deleting.title }}”? This cannot be undone.
      </p>
      <footer class="modal-actions">
        <button autofocus class="secondary" @click="deleting = null">
          Keep card</button
        ><button
          class="danger-button"
          @click="
            store.remove(deleting!.id);
            deleting = null;
            activeCard = null;
          "
        >
          Delete card
        </button>
      </footer></ModalShell
    >
    <SettingsPanel
      v-if="settingsOpen && editing"
      :settings="{ ...board.settings, shortcut: native.shortcut.value }"
      :autostart="autostart"
      :apply="applySettings"
      @close="settingsOpen = false"
    />
  </main>
</template>
