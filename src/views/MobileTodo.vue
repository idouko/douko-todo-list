<template>
  <div
    ref="layoutRef"
    class="mobile-layout"
    @touchstart.passive="onTouchStart"
    @touchend="onTouchEnd"
  >
    <div
      class="pull-indicator"
      :class="{ active: pullDistance > 0 || refreshing }"
      :style="pullIndicatorStyle"
    >
      <span v-if="refreshing" class="pull-indicator-text">{{ $t('mobile.refreshing') }}</span>
      <span v-else class="pull-indicator-text">{{ pullDistance >= 60 ? $t('mobile.releaseToRefresh') : $t('mobile.pullRefresh') }}</span>
    </div>
    <header class="mobile-header">
      <h1 class="mobile-title">{{ $t('mobile.title') }}</h1>
      <p class="mobile-subtitle">{{ $t('mobile.subtitle') }}</p>
    </header>
    <main class="mobile-main">
      <MobileTodoForm @submit="onSubmit" />
      <div v-if="loading && !refreshing" class="loading-wrap">{{ $t('mobile.loading') }}</div>
      <ul v-else class="mobile-list">
        <MobileTodoItem
          v-for="item in todoList"
          :key="item.id"
          :todo="item"
          :groups="groups"
          @toggle="onToggle"
          @delete="onDelete"
          @edit="onEdit"
        />
      </ul>
    </main>
    <footer class="mobile-footer">
      {{ $t('mobile.footer') }}
    </footer>
    <!-- 编辑任务：右侧抽屉 -->
    <el-drawer
      v-model="editDrawerVisible"
      :title="$t('mobile.editTask')"
      direction="rtl"
      size="85%"
      destroy-on-close
      class="mobile-edit-drawer"
      @closed="() => { editForm = null; editingTodoId = null; }"
    >
      <el-form
        v-if="editForm"
        :model="editForm"
        label-position="top"
        class="mobile-edit-form"
      >
        <el-form-item :label="$t('common.content')">
          <el-input
            v-model="editForm.content"
            type="textarea"
            :rows="3"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
        <el-form-item :label="$t('common.status')">
          <el-radio-group v-model="editForm.status">
            <el-radio value="pending">{{ $t('list.pending') }}</el-radio>
            <el-radio value="completed">{{ $t('list.completed') }}</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item :label="$t('list.importance')">
          <el-select v-model="editForm.importance" style="width: 100%">
            <el-option :label="$t('common.normal')" value="normal" />
            <el-option :label="$t('common.important')" value="important" />
            <el-option :label="$t('common.urgent')" value="urgent" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('common.group')">
          <el-select
            v-model="editForm.group_id"
            :placeholder="$t('form.groupPlaceholder')"
            clearable
            filterable
            style="width: 100%"
          >
            <el-option
              v-for="g in groups"
              :key="g.id"
              :label="g.name"
              :value="g.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('desktop.start')">
          <el-date-picker
            v-model="editForm.start_time"
            type="datetime"
            :placeholder="$t('form.startPlaceholder')"
            value-format="YYYY-MM-DDTHH:mm:ss"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item :label="$t('desktop.end')">
          <el-date-picker
            v-model="editForm.end_time"
            type="datetime"
            :placeholder="$t('form.endPlaceholder')"
            value-format="YYYY-MM-DDTHH:mm:ss"
            style="width: 100%"
          />
        </el-form-item>
        <div class="mobile-edit-actions">
          <el-button @click="editDrawerVisible = false">{{ $t('common.cancel') }}</el-button>
          <el-button type="primary" :loading="editSubmitting" @click="onEditSubmit">
            {{ $t('common.save') }}
          </el-button>
        </div>
      </el-form>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import MobileTodoForm from "@/components/MobileTodoForm.vue";
import MobileTodoItem from "@/components/MobileTodoItem.vue";
import {
  getTodoList,
  createTodo,
  updateTodo,
  updateTodoStatus,
  deleteTodo,
  getGroups,
  type TodoItem,
  type GroupItem,
} from "@/utils/request";

const layoutRef = ref<HTMLElement | null>(null);
const todoList = ref<TodoItem[]>([]);
const groups = ref<GroupItem[]>([]);
const loading = ref(false);
const refreshing = ref(false);

const editDrawerVisible = ref(false);
const editForm = ref<{
  content: string;
  status: "pending" | "completed";
  importance: "normal" | "important" | "urgent";
  group_id: string | undefined;
  start_time: string | undefined;
  end_time: string | undefined;
} | null>(null);
const editSubmitting = ref(false);
let editingTodoId: string | null = null;

const PULL_THRESHOLD = 60;
const PULL_MAX = 80;
let pullStartY: number | null = null;
const pullDistance = ref(0);

const pullIndicatorStyle = computed(() => {
  const h = 48;
  const pulling = pullDistance.value > 0 || refreshing.value;
  const y = pulling
    ? (refreshing.value ? PULL_THRESHOLD : Math.min(pullDistance.value, PULL_MAX))
    : 0;
  return {
    transform: `translateY(${-h + y}px)`,
    minHeight: `${h}px`,
  };
});

function getScrollTop() {
  return window.scrollY ?? document.documentElement.scrollTop ?? 0;
}

function onTouchStart(e: TouchEvent) {
  if (e.touches.length === 0) return;
  if (getScrollTop() <= 0) {
    pullStartY = e.touches[0].clientY;
  }
}

function onTouchMove(e: TouchEvent) {
  if (e.touches.length === 0 || pullStartY === null) return;
  if (getScrollTop() > 0) {
    pullStartY = null;
    pullDistance.value = 0;
    return;
  }
  const dy = e.touches[0].clientY - pullStartY;
  if (dy > 0) {
    e.preventDefault();
    pullDistance.value = Math.min(dy, PULL_MAX);
  }
}

function bindTouchMove() {
  const el = layoutRef.value;
  if (!el) return;
  el.addEventListener("touchmove", onTouchMove, { passive: false });
  return () => el.removeEventListener("touchmove", onTouchMove);
}

function onTouchEnd() {
  if (pullDistance.value >= PULL_THRESHOLD && !refreshing.value) {
    refreshing.value = true;
    doRefresh().finally(() => {
      refreshing.value = false;
    });
  }
  pullStartY = null;
  pullDistance.value = 0;
}

function doRefresh() {
  return Promise.all([
    getTodoList("comprehensive").then(
      (list) => { todoList.value = list; },
      () => { todoList.value = []; }
    ),
    getGroups().then(
      (list) => { groups.value = list; },
      () => {}
    ),
  ]);
}

function loadList() {
  loading.value = true;
  getTodoList("comprehensive")
    .then((list) => {
      todoList.value = list;
    })
    .catch(() => {
      todoList.value = [];
    })
    .finally(() => {
      loading.value = false;
    });
}

function loadGroups() {
  getGroups().then((list) => {
    groups.value = list;
  }).catch(() => {});
}

function onSubmit(content: string) {
  loading.value = true;
  createTodo({ content })
    .then(() => loadList())
    .finally(() => {
      loading.value = false;
    });
}

function onToggle(id: string, status: "pending" | "completed") {
  const next = status === "completed" ? "pending" : "completed";
  updateTodoStatus(id, next).then(() => loadList());
}

function onDelete(id: string) {
  deleteTodo(id).then(() => loadList());
}

function onEdit(row: TodoItem) {
  editingTodoId = row.id;
  editForm.value = {
    content: row.content,
    status: row.status,
    importance: row.importance,
    group_id: row.group_id,
    start_time: row.start_time,
    end_time: row.end_time,
  };
  editDrawerVisible.value = true;
}

function onEditSubmit() {
  if (!editForm.value || !editingTodoId) return;
  const payload = {
    content: editForm.value.content.trim(),
    status: editForm.value.status,
    importance: editForm.value.importance,
    group_id: editForm.value.group_id ?? undefined,
    start_time: editForm.value.start_time ?? undefined,
    end_time: editForm.value.end_time ?? undefined,
  };
  editSubmitting.value = true;
  updateTodo(editingTodoId, payload)
    .then(() => {
      const idx = todoList.value.findIndex((t) => t.id === editingTodoId);
      if (idx !== -1) todoList.value[idx] = { ...todoList.value[idx], ...payload };
      editDrawerVisible.value = false;
      editForm.value = null;
      editingTodoId = null;
    })
    .finally(() => {
      editSubmitting.value = false;
    });
}

let unbindTouchMove: (() => void) | undefined;
onMounted(() => {
  loadList();
  loadGroups();
  nextTick(() => {
    unbindTouchMove = bindTouchMove();
  });
});
onUnmounted(() => {
  unbindTouchMove?.();
});
</script>

<style scoped>
.mobile-layout {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(180deg, #fdf8f3 0%, #f5ebe0 100%);
  padding: 0 16px;
  touch-action: pan-y;
}
.mobile-header {
  padding: 28px 0 20px;
  text-align: center;
}
.mobile-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: #5c4a3a;
  margin-bottom: 6px;
  letter-spacing: 0.02em;
}
.mobile-subtitle {
  font-size: 13px;
  color: #8b7355;
}
.mobile-main {
  flex: 1;
  padding-bottom: 24px;
}
.loading-wrap {
  text-align: center;
  padding: 28px;
  color: #9a8576;
  font-size: 14px;
}
.mobile-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.mobile-footer {
  padding: 20px 0 28px;
  font-size: 12px;
  color: #9a8576;
  text-align: center;
}

.pull-indicator {
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  min-height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #9a8576;
  font-size: 13px;
  background: linear-gradient(180deg, #fdf8f3 0%, #f5ebe0 100%);
  z-index: 10;
  transition: transform 0.15s ease-out;
  pointer-events: none;
}
.pull-indicator.active {
  transition: transform 0.2s ease-out;
}
.pull-indicator-text {
  white-space: nowrap;
}

.mobile-edit-form {
  padding: 0 4px;
}
.mobile-edit-actions {
  margin-top: 24px;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}
</style>
