<template>
  <li
    class="mobile-item"
    :class="{ completed: todo.status === 'completed', [importanceClass]: true, 'menu-open': slideOffset < 0 }"
  >
    <div
      class="mobile-item-slide"
      :style="{ transform: `translateX(${slideOffset}px)` }"
      @touchstart="onTouchStart"
      @touchmove="onTouchMove"
      @touchend="onTouchEnd"
    >
      <div class="mobile-item-main">
        <button
          type="button"
          class="mobile-item-radio"
          :class="{ checked: todo.status === 'completed' }"
          :aria-label="todo.status === 'completed' ? $t('list.toggleToPending') : $t('list.toggleToCompleted')"
          @click.stop="$emit('toggle', todo.id, todo.status)"
        >
          <span class="radio-inner" />
        </button>
        <div class="mobile-item-body">
          <span class="mobile-item-content">{{ todo.content }}</span>
          <div v-if="hasMeta" class="mobile-item-meta">
            <span v-if="groupName" class="meta-tag meta-group">{{ groupName }}</span>
            <span v-if="importanceLabel" class="meta-tag meta-importance">{{ importanceLabel }}</span>
            <span v-if="timeText" class="meta-time">{{ timeText }}</span>
          </div>
        </div>
      </div>
    </div>
    <div class="mobile-item-actions-swipe">
      <button
        type="button"
        class="action-swipe-btn edit"
        @click="onEditClick"
      >
        {{ $t('mobile.editAction') }}
      </button>
      <button
        type="button"
        class="action-swipe-btn delete"
        @click="$emit('delete', todo.id)"
      >
        {{ $t('mobile.deleteAction') }}
      </button>
    </div>
  </li>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import type { TodoItem, GroupItem } from "@/utils/request";

const props = defineProps<{
  todo: TodoItem;
  groups: GroupItem[];
}>();

const emit = defineEmits<{
  (e: "toggle", id: string, status: "pending" | "completed"): void;
  (e: "delete", id: string): void;
  (e: "edit", todo: TodoItem): void;
}>();

const { locale, t } = useI18n();

const MENU_WIDTH = 120;
const slideOffset = ref(0);
let touchStartX = 0;
let touchStartOffset = 0;

function onEditClick(e: Event) {
  e.stopPropagation();
  emit("edit", props.todo);
  slideOffset.value = 0;
}

function onTouchStart(e: TouchEvent) {
  if (e.touches.length === 0) return;
  touchStartX = e.touches[0].clientX;
  touchStartOffset = slideOffset.value;
}

function onTouchMove(e: TouchEvent) {
  if (e.touches.length === 0) return;
  const dx = e.touches[0].clientX - touchStartX;
  const next = touchStartOffset + dx;
  if (next > 0) {
    slideOffset.value = Math.min(next, 0);
  } else {
    slideOffset.value = Math.max(next, -MENU_WIDTH);
  }
}

function onTouchEnd() {
  const current = slideOffset.value;
  if (current > -MENU_WIDTH / 2) {
    slideOffset.value = 0;
  } else {
    slideOffset.value = -MENU_WIDTH;
  }
  touchStartX = 0;
  touchStartOffset = slideOffset.value;
}

const groupName = computed(() => {
  if (!props.todo.group_id) return t("desktop.ungrouped");
  const g = props.groups.find((x) => x.id === props.todo.group_id);
  return g?.name ?? t("desktop.ungrouped");
});

const importanceLabel = computed(() => {
  const imp = props.todo.importance;
  if (!imp || imp === "normal") return "";
  return t("common." + imp);
});

const importanceClass = computed(() => {
  const imp = props.todo.importance;
  return imp === "urgent" ? "importance-urgent" : imp === "important" ? "importance-important" : "";
});

function formatTime(s: string) {
  if (!s) return "";
  try {
    const d = new Date(s);
    const loc = locale.value === "ja" ? "ja-JP" : locale.value === "en" ? "en" : "zh-CN";
    return d.toLocaleString(loc, {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return s;
  }
}

const timeText = computed(() => {
  const s = props.todo.start_time;
  const e = props.todo.end_time;
  if (s && e) return formatTime(s) + " ~ " + formatTime(e);
  if (s) return formatTime(s);
  if (e) return "~ " + formatTime(e);
  return "";
});

const hasMeta = computed(() => groupName.value || importanceLabel.value || timeText.value);
</script>

<style scoped>
.mobile-item {
  position: relative;
  margin-bottom: 12px;
  list-style: none;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(92, 74, 58, 0.06);
}
.mobile-item.menu-open {
  overflow: visible;
}
.mobile-item-slide {
  position: relative;
  z-index: 1;
  background: #fffefc;
  border-radius: 12px;
  border: 1px solid rgba(139, 115, 85, 0.12);
  transition: transform 0.15s ease-out, background 0.2s, border-color 0.2s, opacity 0.2s;
}
/* 已完成：整卡置灰 */
.mobile-item.completed .mobile-item-slide {
  background: #f5f2ee;
  border-color: rgba(139, 115, 85, 0.08);
  opacity: 0.88;
}
.mobile-item.importance-important .mobile-item-slide {
  border-left: 4px solid #d4a574;
}
.mobile-item.importance-urgent .mobile-item-slide {
  border-left: 4px solid #c17f6c;
}
.mobile-item-main {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 18px;
  min-height: 52px;
}
.mobile-item-radio {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.radio-inner {
  display: block;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid rgba(139, 115, 85, 0.4);
  background: #fff;
  transition: border-color 0.2s, background 0.2s;
}
.mobile-item-radio.checked .radio-inner {
  border-color: #d4a574;
  background: #d4a574;
  box-shadow: inset 0 0 0 4px #fff;
}
.mobile-item-body {
  flex: 1;
  min-width: 0;
}
.mobile-item.completed .mobile-item-content {
  text-decoration: line-through;
  color: #8b7d6f;
}
.mobile-item.completed .mobile-item-meta,
.mobile-item.completed .meta-tag,
.mobile-item.completed .meta-time {
  color: #a89b8d;
}
.mobile-item.completed .meta-tag {
  background: rgba(139, 125, 111, 0.12);
  color: #8b7d6f;
}
.mobile-item-content {
  display: block;
  margin-bottom: 6px;
  font-size: 15px;
  line-height: 1.5;
  word-break: break-word;
  color: #5c4a3a;
}
.mobile-item-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  font-size: 12px;
  color: #8b7355;
}
.meta-tag {
  padding: 2px 8px;
  border-radius: 6px;
  background: rgba(212, 165, 116, 0.15);
  color: #7a6349;
}
.meta-importance {
  background: rgba(193, 127, 108, 0.12);
  color: #8b5a4a;
}
.meta-time {
  color: #8b7355;
}
.mobile-item-actions-swipe {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 120px;
  display: flex;
  border-radius: 0 12px 12px 0;
  overflow: hidden;
}
.action-swipe-btn {
  flex: 1;
  border: none;
  font-size: 14px;
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 8px;
}
.action-swipe-btn.edit {
  background: #b8956b;
}
.action-swipe-btn.delete {
  background: #c17f6c;
}
</style>
