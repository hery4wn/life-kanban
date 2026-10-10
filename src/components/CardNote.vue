<script setup lang="ts">
import type { Card } from "../types/board";
defineProps<{
  card: Card;
  editing: boolean;
  selected: boolean;
  done: boolean;
}>();
defineEmits<{ edit: []; select: []; remove: []; move: [direction: string] }>();
</script>
<template>
  <article
    class="note"
    :class="{ selected, done }"
    :data-card-id="card.id"
    :tabindex="editing ? 0 : -1"
    :aria-label="card.title"
    @focus="$emit('select')"
    @click="editing && $emit('select')"
    @dblclick="
      editing &&
      !($event.target as HTMLElement).closest('button') &&
      $emit('edit')
    "
    @keydown.enter.self.prevent="editing && $emit('edit')"
    @keydown.delete.self.prevent="editing && $emit('remove')"
    @keydown.alt.up.self.prevent="$emit('move', 'up')"
    @keydown.alt.down.self.prevent="$emit('move', 'down')"
    @keydown.alt.left.self.prevent="$emit('move', 'left')"
    @keydown.alt.right.self.prevent="$emit('move', 'right')"
  >
    <h3>{{ card.title }}</h3>
    <div v-if="editing" class="note-actions">
      <button
        class="icon-btn"
        :aria-label="`Edit ${card.title}`"
        @click.stop="$emit('edit')"
      >
        ↗</button
      ><button
        class="icon-btn"
        :aria-label="`Delete ${card.title}`"
        @click.stop="$emit('remove')"
      >
        ×
      </button>
    </div>
  </article>
</template>
