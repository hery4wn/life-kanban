<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Sortable from "sortablejs";
import type { Card, Column } from "../types/board";
import CardNote from "./CardNote.vue";
const props = defineProps<{
  column: Column;
  editing: boolean;
  selected: string | null;
  index: number;
}>();
const emit = defineEmits<{
  add: [title: string];
  edit: [card: Card];
  select: [id: string];
  remove: [card: Card];
  drop: [id: string, columnId: string, index: number];
  rename: [name: string];
  keyboardMove: [id: string, direction: string];
}>();
const list = ref<HTMLElement>(),
  input = ref<HTMLInputElement>(),
  nameInput = ref<HTMLInputElement>();
const adding = ref(false),
  title = ref(""),
  renaming = ref(false),
  name = ref("");
let sortable: Sortable | undefined;
async function startAdd() {
  adding.value = true;
  await nextTick();
  input.value?.focus();
}
function add() {
  if (!title.value.trim()) return;
  emit("add", title.value);
  title.value = "";
  input.value?.focus();
}
async function startRename() {
  name.value = props.column.name;
  renaming.value = true;
  await nextTick();
  nameInput.value?.select();
}
function rename() {
  if (renaming.value) {
    emit("rename", name.value);
    renaming.value = false;
  }
}
onMounted(() => {
  sortable = Sortable.create(list.value!, {
    group: "life-kanban",
    animation: matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 160,
    disabled: !props.editing,
    draggable: ".note",
    filter: "button",
    preventOnFilter: false,
    ghostClass: "note-ghost",
    chosenClass: "note-chosen",
    forceFallback: true,
    fallbackOnBody: false,
    onEnd(event) {
      // Restore Sortable's DOM mutation before changing Vue's keyed state.
      event.item.remove();
      event.from.insertBefore(
        event.item,
        event.from.children[event.oldIndex ?? 0] ?? null,
      );
      const id = event.item.dataset.cardId,
        target = event.to.dataset.columnId;
      if (id && target && event.newIndex !== undefined)
        emit("drop", id, target, event.newIndex);
    },
  });
});
watch(
  () => props.editing,
  (value) => {
    sortable?.option("disabled", !value);
    if (!value) {
      adding.value = false;
      renaming.value = false;
    }
  },
);
onBeforeUnmount(() => sortable?.destroy());
</script>
<template>
  <section
    class="column"
    :aria-label="column.name"
    :data-column="column.id"
    :data-col="index"
  >
    <header class="column-header">
      <span class="column-symbol" aria-hidden="true"
        ><svg viewBox="0 0 24 24">
          <template v-if="index === 0">
            <path d="M5 8h14l2 10H3L5 8Z" />
            <path d="M4 14h5l1 3h4l1-3h5M8 5h8" />
          </template>
          <template v-else-if="index === 1">
            <rect x="5" y="5" width="14" height="16" rx="3" />
            <path d="M9 5V3h6v2M9 10h6M9 14h6" />
          </template>
          <path v-else-if="index === 2" d="m13 3-8 11h6l-1 7 9-12h-6l1-6Z" />
          <template v-else>
            <circle cx="12" cy="12" r="9" />
            <path d="m8 12 3 3 5-6" />
          </template></svg
      ></span>
      <form v-if="renaming && editing" @submit.prevent="rename">
        <input
          ref="nameInput"
          v-model="name"
          maxlength="40"
          aria-label="Column name"
          @blur="rename"
          @keydown.esc.stop="renaming = false"
        />
      </form>
      <button
        v-else-if="editing"
        class="column-name"
        :aria-label="`Rename ${column.name}`"
        @click="startRename"
      >
        {{ column.name }}
      </button>
      <h2 v-else class="column-name">{{ column.name }}</h2>
      <span class="count">{{ column.cards.length }}</span>
      <button
        v-if="editing"
        class="icon-btn column-add"
        :aria-label="`Add card to ${column.name}`"
        @click="startAdd"
      >
        +
      </button>
    </header>
    <div ref="list" class="notes" :data-column-id="column.id">
      <CardNote
        v-for="card in column.cards"
        :key="card.id"
        :card="card"
        :editing="editing"
        :selected="selected === card.id"
        :done="index === 3"
        @select="emit('select', card.id)"
        @edit="emit('edit', card)"
        @remove="emit('remove', card)"
        @move="emit('keyboardMove', card.id, $event)"
      />
      <p v-if="!column.cards.length && !adding" class="empty-column">
        {{
          [
            "Room for a new idea.",
            "A little intention.",
            "One thing at a time.",
            "Small wins live here.",
          ][index]
        }}
      </p>
    </div>
    <form v-if="adding && editing" class="quick-add" @submit.prevent="add">
      <input
        ref="input"
        v-model="title"
        placeholder="One small thing…"
        aria-label="New card title"
        maxlength="200"
        @keydown.esc.stop="adding = false"
      />
      <div>
        <span>Enter to add</span
        ><button type="button" class="text-button" @click="adding = false">
          Close
        </button>
      </div>
    </form>
    <button v-else-if="editing" class="add-project" @click="startAdd">
      <span>+</span> Add a project
    </button>
  </section>
</template>
