<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import type { Board } from "../types/board";
const props = defineProps<{ board: Board; editing: boolean }>();
const emit = defineEmits<{ save: [] }>();
const now = ref(Date.now()),
  selected = ref(new Date()),
  month = ref(new Date().getMonth()),
  year = ref(new Date().getFullYear());
const monthLabel = computed(() =>
  new Date(year.value, month.value, 1).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  }),
);
const dates = computed(() => {
  const offset = (new Date(year.value, month.value, 1).getDay() + 6) % 7;
  const count =
    Math.ceil(
      (offset + new Date(year.value, month.value + 1, 0).getDate()) / 7,
    ) * 7;
  return Array.from(
    { length: count },
    (_, i) => new Date(year.value, month.value, i - offset + 1),
  );
});
function navigate(delta: number) {
  const date = new Date(year.value, month.value + delta, 1);
  year.value = date.getFullYear();
  month.value = date.getMonth();
}
function select(date: Date) {
  selected.value = date;
  year.value = date.getFullYear();
  month.value = date.getMonth();
}
const quickTitle = ref("");
const complete = computed(
  () => props.board.quickTasks.filter((t) => t.done).length,
);
const percent = computed(() =>
  props.board.quickTasks.length
    ? Math.round((complete.value / props.board.quickTasks.length) * 100)
    : 0,
);
function add() {
  const title = quickTitle.value.trim();
  if (!props.editing || !title) return;
  props.board.quickTasks.push({ id: crypto.randomUUID(), title, done: false });
  quickTitle.value = "";
  emit("save");
}
function remove(id: string) {
  props.board.quickTasks.splice(
    props.board.quickTasks.findIndex((t) => t.id === id),
    1,
  );
  emit("save");
}
const minutes = ref(Math.floor(props.board.timer.durationSeconds / 60)),
  seconds = ref(props.board.timer.durationSeconds % 60),
  status = ref("");
const running = computed(() => props.board.timer.endsAt !== null);
const paused = computed(
  () =>
    !running.value &&
    props.board.timer.remainingSeconds > 0 &&
    props.board.timer.remainingSeconds < props.board.timer.durationSeconds,
);
const remaining = computed(() =>
  Math.ceil(
    running.value
      ? Math.max(0, (props.board.timer.endsAt! - now.value) / 1000)
      : props.board.timer.remainingSeconds,
  ),
);
const display = computed(
  () =>
    `${String(Math.floor(remaining.value / 60)).padStart(2, "0")}:${String(remaining.value % 60).padStart(2, "0")}`,
);
const localTime = computed(() =>
  new Date(now.value).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  }),
);
const hourAngle = computed(() => {
  const date = new Date(now.value);
  return (date.getHours() % 12) * 30 + date.getMinutes() / 2;
});
const minuteAngle = computed(() => new Date(now.value).getMinutes() * 6);
let wake: ReturnType<typeof setTimeout> | undefined;
function tick() {
  now.value = Date.now();
  if (running.value && props.board.timer.endsAt! <= now.value) {
    props.board.timer.endsAt = null;
    props.board.timer.remainingSeconds = 0;
    status.value = "Time’s up!";
    emit("save");
  }
  if (wake) clearTimeout(wake);
  wake = setTimeout(tick, running.value ? 1000 : 60000 - (Date.now() % 60000));
}
function setDuration() {
  if (running.value || paused.value) return;
  const min = Number(minutes.value),
    sec = Number(seconds.value);
  if (
    !Number.isInteger(min) ||
    min < 0 ||
    min > 180 ||
    !Number.isInteger(sec) ||
    sec < 0 ||
    sec > 59 ||
    min * 60 + sec < 1
  ) {
    status.value = "Set a valid duration";
    return;
  }
  props.board.timer.durationSeconds = min * 60 + sec;
  props.board.timer.remainingSeconds = min * 60 + sec;
  status.value = "";
  emit("save");
}
function toggle() {
  if (!props.editing) return;
  now.value = Date.now();
  if (running.value) {
    props.board.timer.remainingSeconds = Math.max(
      0,
      (props.board.timer.endsAt! - now.value) / 1000,
    );
    props.board.timer.endsAt = null;
    status.value = props.board.timer.remainingSeconds ? "Paused" : "Time’s up!";
  } else {
    if (!paused.value) {
      setDuration();
      if (status.value) return;
      props.board.timer.remainingSeconds = props.board.timer.durationSeconds;
    }
    props.board.timer.endsAt =
      now.value + props.board.timer.remainingSeconds * 1000;
    status.value = "";
  }
  emit("save");
  tick();
}
function reset() {
  props.board.timer.endsAt = null;
  props.board.timer.remainingSeconds = props.board.timer.durationSeconds;
  status.value = "";
  emit("save");
  tick();
}
onMounted(() => {
  tick();
  document.addEventListener("visibilitychange", tick);
});
onBeforeUnmount(() => {
  clearTimeout(wake);
  document.removeEventListener("visibilitychange", tick);
});
</script>
<template>
  <aside class="sidebar" aria-label="Desktop widgets">
    <section class="calendar" aria-label="Calendar">
      <div class="calendar-head">
        <span>{{ monthLabel }}</span>
        <div class="calendar-nav">
          <button
            class="icon-btn"
            :disabled="!editing"
            aria-label="Previous month"
            @click="navigate(-1)"
          >
            ‹</button
          ><button
            class="icon-btn"
            :disabled="!editing"
            aria-label="Next month"
            @click="navigate(1)"
          >
            ›
          </button>
        </div>
      </div>
      <div class="calendar-grid">
        <span
          v-for="(day, i) in ['M', 'T', 'W', 'T', 'F', 'S', 'S']"
          :key="i"
          class="weekday"
          >{{ day }}</span
        ><button
          v-for="date in dates"
          :key="date.toISOString()"
          class="day"
          :class="{
            other: date.getMonth() !== month,
            today: date.toDateString() === new Date(now).toDateString(),
            selected: date.toDateString() === selected.toDateString(),
          }"
          :disabled="!editing"
          :aria-label="
            date.toLocaleDateString(undefined, {
              month: 'long',
              day: 'numeric',
              year: 'numeric',
            })
          "
          :aria-pressed="date.toDateString() === selected.toDateString()"
          @click="select(date)"
        >
          {{ date.getDate() }}
        </button>
      </div>
      <div class="calendar-bottom">
        <span>{{
          selected.toLocaleDateString(undefined, {
            weekday: "short",
            month: "short",
            day: "numeric",
          })
        }}</span
        ><button
          class="text-button"
          :disabled="!editing"
          @click="select(new Date())"
        >
          Today ↗
        </button>
      </div>
    </section>
    <section class="quick-tasks" aria-label="Quick tasks">
      <div class="todo-heading">
        <h2>
          <span class="tiny-flower" aria-hidden="true">✿</span> Little to-dos
        </h2>
        <span>{{ complete }} / {{ board.quickTasks.length }}</span>
      </div>
      <p class="todo-intro">Quick things. No project needed.</p>
      <div class="todo-list">
        <div
          v-for="task in board.quickTasks"
          :key="task.id"
          class="todo"
          :class="{ done: task.done }"
        >
          <label
            ><input
              v-model="task.done"
              type="checkbox"
              :disabled="!editing"
              @change="emit('save')"
            /><span>{{ task.title }}</span></label
          ><button
            v-if="editing"
            class="icon-btn remove"
            :aria-label="`Remove ${task.title}`"
            @click="remove(task.id)"
          >
            ×
          </button>
        </div>
        <p v-if="!board.quickTasks.length" class="todo-empty">
          Room for a little thing.
        </p>
      </div>
      <form v-if="editing" class="quick-form" @submit.prevent="add">
        <input
          v-model="quickTitle"
          aria-label="New quick task"
          placeholder="Add a quick thing…"
          maxlength="120"
          required
        /><button aria-label="Add quick task">+</button>
      </form>
      <div class="progress-caption">
        <span>{{
          percent === 100 && board.quickTasks.length
            ? "All the little things, done!"
            : "Small wins add up."
        }}</span
        ><span>{{ percent }}%</span>
      </div>
      <div
        class="progress-track"
        role="progressbar"
        aria-label="Quick task completion"
        :aria-valuenow="percent"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <div class="progress-fill" :style="{ width: percent + '%' }"></div>
      </div>
    </section>
    <section
      class="time-widget"
      :class="{ finished: status === 'Time’s up!' }"
      aria-label="Clock and timer"
    >
      <div class="round-clock">
        <svg viewBox="0 0 100 100" aria-hidden="true">
          <circle cx="50" cy="50" r="46" class="clock-face" />
          <g class="clock-ticks">
            <line
              v-for="i in 12"
              :key="i"
              x1="50"
              y1="10"
              x2="50"
              :y2="i % 3 === 0 ? 17 : 14"
              :transform="`rotate(${i * 30} 50 50)`"
            />
          </g>
          <line
            id="hour-hand"
            x1="50"
            y1="50"
            x2="50"
            y2="29"
            :transform="`rotate(${hourAngle} 50 50)`"
          />
          <line
            id="minute-hand"
            x1="50"
            y1="50"
            x2="50"
            y2="20"
            :transform="`rotate(${minuteAngle} 50 50)`"
          />
          <circle cx="50" cy="50" r="3" fill="#65794f" /></svg
        ><time
          :datetime="new Date(now).toISOString()"
          aria-label="Current local time"
          >{{ localTime }}</time
        >
      </div>
      <div class="timer-controls">
        <div class="timer-top">
          <span>Little focus timer</span
          ><output id="timer-display" aria-label="Time remaining">{{
            display
          }}</output>
        </div>
        <div v-if="editing" class="duration-controls">
          <label
            ><input
              v-model.number="minutes"
              type="number"
              min="0"
              max="180"
              step="1"
              :disabled="running || paused"
              aria-label="Timer minutes"
              @input="setDuration"
            />
            min</label
          ><label
            ><input
              v-model.number="seconds"
              type="number"
              min="0"
              max="59"
              step="1"
              :disabled="running || paused"
              aria-label="Timer seconds"
              @input="setDuration"
            />
            sec</label
          >
        </div>
        <div class="timer-actions">
          <template v-if="editing"
            ><button @click="toggle">
              {{ running ? "Pause" : paused ? "Resume" : "Start" }}</button
            ><button id="timer-reset" aria-label="Reset timer" @click="reset">
              ↺
            </button></template
          ><span role="status" aria-live="polite">{{ status }}</span>
        </div>
      </div>
    </section>
  </aside>
</template>
