<template>
  <div class="desktop-layout" :class="{ 'is-sidebar-only': isSidebarOnly }">
    <aside v-show="isSidebarOnly" class="sidebar">
      <div class="sidebar-groups">
        <button
          type="button"
          class="sidebar-item"
          :class="{ active: currentGroupId === 'all' }"
          @click="currentGroupId = 'all'"
        >
          <span class="sidebar-item-name">{{ $t('desktop.all') }}</span>
          <span class="sidebar-item-count">{{ todoList.length }}/{{ todoList.length }}</span>
        </button>
        <button
          v-for="g in groups"
          :key="g.id"
          type="button"
          class="sidebar-item"
          :class="{ active: currentGroupId === g.id }"
          @click="currentGroupId = g.id"
        >
          <span class="sidebar-item-name">{{ g.name }}</span>
          <span class="sidebar-item-count">{{ countByGroupId(g.id) }}/{{ countByGroupId(g.id) }}</span>
        </button>
        <button
          type="button"
          class="sidebar-item"
          :class="{ active: currentGroupId === 'ungrouped' }"
          @click="currentGroupId = 'ungrouped'"
        >
          <span class="sidebar-item-name">{{ $t('desktop.ungrouped') }}</span>
          <span class="sidebar-item-count">{{ ungroupedCount }}/{{ ungroupedCount }}</span>
        </button>
        <button
          type="button"
          class="sidebar-item sidebar-item-add"
          :title="$t('desktop.addGroup')"
          @click="onAddGroup"
        >
          <span class="sidebar-item-add-text">{{ $t('desktop.addGroup') }}</span>
          <span class="sidebar-item-add-icon">+</span>
        </button>
      </div>
    </aside>
    <div v-show="!isSidebarOnly" class="main-wrap">
      <header class="desktop-title-bar" data-tauri-drag-region @dblclick.prevent>
        <h1 class="desktop-title">{{ currentGroupTitle }}</h1>
        <div class="desktop-title-actions">
          <button
            v-if="isTauri"
            type="button"
            class="title-bar-btn"
            :class="{ 'title-bar-btn-active': alwaysOnTop }"
            :title="alwaysOnTop ? $t('desktop.unpin') : $t('desktop.pinToTop')"
            @click="toggleAlwaysOnTop"
          >
            <svg class="pin-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
            </svg>
          </button>
          <template v-if="isTauri">
            <button type="button" class="title-bar-btn" :title="$t('desktop.minimize')" @click="windowMinimize">
              <el-icon><Minus /></el-icon>
            </button>
            <button type="button" class="title-bar-btn" :title="$t('desktop.maximize')" @click="windowToggleMaximize">
              <el-icon><FullScreen /></el-icon>
            </button>
            <button type="button" class="title-bar-btn title-bar-btn-close" :title="$t('desktop.close')" @click="windowClose">
              <el-icon><Close /></el-icon>
            </button>
          </template>
        </div>
      </header>
    <main class="main-content">
      <div class="form-card form-card-simple">
        <el-input
          v-model="addInputContent"
          :placeholder="$t('form.placeholder')"
          clearable
          maxlength="500"
          show-word-limit
          class="add-input"
          @keyup.enter="openAddDialog"
        />
        <el-button type="primary" class="add-btn" :disabled="!addInputContent.trim()" @click="openAddDialog">
          {{ $t('form.add') }}
        </el-button>
      </div>
      <div class="form-table-divider" />
      <TodoList
        :todo-list="filteredTodoList"
        :loading="loading"
        :groups="groups"
        :sort-rule="sortRule"
        :hide-group-column="currentGroupId !== 'all'"
        :pagination-enabled="paginationEnabled"
        :page-size="paginationPageSize"
        @toggle="onToggle"
        @refresh="loadData"
        @sort-change="sortRule = $event"
        @reorder="onReorder"
        @edit="onEdit"
      />
    </main>
    <!-- 新增/编辑任务弹窗（复用） -->
    <el-dialog
      v-model="taskDialogVisible"
      :title="taskDialogMode === 'edit' ? $t('desktop.editTask') : $t('desktop.addTask')"
      destroy-on-close
      :close-on-click-modal="false"
      class="task-dialog"
      width="calc(100vw - 40px)"
      @closed="onTaskDialogClosed"
    >
      <el-form :model="taskForm" label-position="top">
        <el-form-item :label="$t('common.content')">
          <el-input
            v-model="taskForm.content"
            type="textarea"
            :rows="3"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
        <el-form-item v-if="taskDialogMode === 'edit'" :label="$t('common.status')">
          <el-radio-group v-model="taskForm.status">
            <el-radio value="pending">{{ $t('list.pending') }}</el-radio>
            <el-radio value="completed">{{ $t('list.completed') }}</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item :label="$t('list.importance')">
          <el-select v-model="taskForm.importance" style="width: 100%" :placeholder="$t('form.importance')">
            <el-option :label="$t('common.normal')" value="normal">
              <span class="importance-option">
                <span class="importance-option-dot importance-option-dot-normal" />
                <span>{{ $t('common.normal') }}</span>
              </span>
            </el-option>
            <el-option :label="$t('common.important')" value="important">
              <span class="importance-option">
                <span class="importance-option-dot importance-option-dot-important" />
                <span>{{ $t('common.important') }}</span>
              </span>
            </el-option>
            <el-option :label="$t('common.urgent')" value="urgent">
              <span class="importance-option">
                <span class="importance-option-dot importance-option-dot-urgent" />
                <span>{{ $t('common.urgent') }}</span>
              </span>
            </el-option>
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('common.group')">
          <el-select
            v-model="taskForm.group_id"
            :placeholder="$t('form.groupPlaceholder')"
            clearable
            filterable
            :allow-create="taskDialogMode === 'add'"
            default-first-option
            style="width: 100%"
            @change="onTaskGroupChange"
          >
            <el-option
              v-for="g in groups"
              :key="g.id"
              :label="g.name"
              :value="g.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('desktop.timeRange')">
          <el-date-picker
            v-model="taskTimeRange"
            type="datetimerange"
            :start-placeholder="$t('form.startPlaceholder')"
            :end-placeholder="$t('form.endPlaceholder')"
            value-format="YYYY-MM-DDTHH:mm:ss"
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button v-if="taskDialogMode === 'edit'" type="danger" plain :loading="taskDeleting" @click="onTaskDelete">
            {{ $t('common.delete') }}
          </el-button>
          <span class="dialog-footer-right">
            <el-button @click="taskDialogVisible = false">{{ $t('common.cancel') }}</el-button>
            <el-button type="primary" :disabled="!taskForm.content.trim()" :loading="taskSubmitting" @click="onTaskSubmit">
              {{ taskDialogMode === 'edit' ? $t('common.save') : $t('form.add') }}
            </el-button>
          </span>
        </span>
      </template>
    </el-dialog>
    <!-- 分组管理弹窗（主窗体） -->
    <el-dialog
      v-model="manageGroupsDialogVisible"
      :title="$t('desktop.dialog.groups')"
      class="manage-groups-dialog"
      destroy-on-close
      :width="manageGroupsDialogWidth"
      @opened="initManageGroupsSortable"
      @closed="onManageGroupsDialogClosed"
    >
      <div class="manage-groups-add">
        <el-input
          v-model="manageGroupsNewName"
          :placeholder="$t('desktop.message.pleaseEnterGroupName')"
          clearable
          maxlength="50"
          show-word-limit
          @keyup.enter="manageGroupsAdd"
        />
        <el-button type="primary" :loading="manageGroupsSubmitting" @click="manageGroupsAdd">
          {{ $t('desktop.add') }}
        </el-button>
      </div>
      <div ref="manageGroupsListRef" class="manage-groups-list">
        <div
          v-for="g in manageGroupsList"
          :key="g.id"
          class="manage-groups-item"
        >
          <span class="manage-groups-drag-handle" :title="$t('desktop.dragToReorder')">
            <el-icon><Rank /></el-icon>
          </span>
          <el-input
            v-model="manageGroupsEditNames[g.id]"
            size="small"
            maxlength="50"
            @blur="(e: Event) => onManageGroupNameBlur(g, (e.target as HTMLInputElement).value)"
            @keyup.enter="(e: Event) => (e.target as HTMLInputElement).blur()"
          />
          <el-button
            type="danger"
            link
            size="small"
            :title="$t('desktop.del')"
            @click="manageGroupsDelete(g)"
          >
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
      </div>
      <template #footer>
        <el-button @click="manageGroupsDialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="manageGroupsSubmitting" @click="manageGroupsConfirm">
          {{ $t('common.ok') }}
        </el-button>
      </template>
    </el-dialog>
    <footer class="desktop-footer">
      <button type="button" class="footer-icon-btn" :title="$t('desktop.qrcode')" @click="openQrcode">
        <el-icon><Grid /></el-icon>
      </button>
      <span class="desktop-footer-text">同步服务端: {{ syncServerUrl }}</span>
      <span class="desktop-footer-spacer" />
      <button type="button" class="footer-icon-btn" :title="$t('desktop.settings')" @click="openSettings">
        <el-icon><Setting /></el-icon>
      </button>
    </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage, ElMessageBox } from "element-plus";
import { Grid, Setting, Minus, FullScreen, Close, Rank, Delete } from "@element-plus/icons-vue";
import TodoList from "@/components/TodoList.vue";
import { getContrastTextColor } from "@/utils/theme";
import {
  getTodoList,
  getGroups,
  createTodo,
  createGroup,
  updateTodo,
  updateTodoStatus,
  deleteTodo,
  reorderTodos,
  updateGroup,
  deleteGroup,
  reorderGroups,
  getMobileUrl,
  type TodoItem,
  type GroupItem,
  type SortRule,
  type UpdateTodoParams,
} from "@/utils/request";
import Sortable from "sortablejs";

const { t } = useI18n();

const STORE_KEY = "app-settings.json";
const KEYS = {
  themeColor: "themeColor",
  backgroundColor: "backgroundColor",
  backgroundImage: "backgroundImage",
  sidebarPosition: "sidebarPosition",
  paginationEnabled: "paginationEnabled",
  paginationPageSize: "paginationPageSize",
} as const;

const isSidebarOnly = computed(() =>
  typeof document !== "undefined" && document.documentElement.classList.contains("sidebar-window")
);

const isTauri = ref(false);
const alwaysOnTop = ref(false);

const groups = ref<GroupItem[]>([]);
const todoList = ref<TodoItem[]>([]);
const loading = ref(false);
const currentGroupId = ref<string | "all" | "ungrouped">("all");
const sortRule = ref<SortRule>("comprehensive");
const newlyCreatedGroupId = ref<string | undefined>(undefined);
const syncServerUrl = ref("");

const addInputContent = ref("");
const taskDialogVisible = ref(false);
const taskDialogMode = ref<"add" | "edit">("add");
const taskForm = ref({
  content: "",
  status: "pending" as "pending" | "completed",
  importance: "normal" as "normal" | "important" | "urgent",
  group_id: undefined as string | undefined,
});
const taskTimeRange = ref<[string, string] | null>(null);
const taskSubmitting = ref(false);
const taskDeleting = ref(false);
let editingTodoId: string | null = null;

const manageGroupsDialogVisible = ref(false);
const manageGroupsDialogWidth = "min(420px, calc(100vw - 48px))";
const manageGroupsList = ref<GroupItem[]>([]);
const manageGroupsListRef = ref<HTMLElement | null>(null);
/** 分组名称编辑中的本地值，key 为分组 id，仅在名称实际变更时提交 */
const manageGroupsEditNames = ref<Record<string, string>>({});
const manageGroupsNewName = ref("");
const manageGroupsSubmitting = ref(false);
let sortableInstance: { destroy(): void } | null = null;

const themeColor = ref("rgba(64, 249, 255, 0.61)");
const backgroundColor = ref("rgba(26, 26, 26, 0.56)");
const backgroundImage = ref("");
const sidebarPosition = ref<"left" | "right">("left");
const paginationEnabled = ref(false);
const paginationPageSize = ref(10);
let settingsStore: { get: (k: string) => Promise<unknown> } | null = null;

function detectTauri(): boolean {
  return typeof window !== "undefined" && !!(window as Window & { __TAURI__?: unknown }).__TAURI__;
}

function getBackgroundImageUrl(path: string): string {
  if (!path) return "";
  if (isTauri.value) {
    const convert = (window as Window & { __TAURI_INTERNALS__?: { convertFileSrc?: (p: string) => string } })
      .__TAURI_INTERNALS__?.convertFileSrc;
    if (typeof convert === "function") {
      return `url(${convert(path)})`;
    }
  }
  return `url(${path})`;
}

function applyTheme() {
  document.documentElement.style.setProperty("--el-color-primary", themeColor.value);
  document.documentElement.style.setProperty("--app-border-color", themeColor.value);
  document.documentElement.style.setProperty("--app-bg-color", backgroundColor.value);
  document.documentElement.style.setProperty("--sidebar-item-bg", backgroundColor.value);
  document.documentElement.style.setProperty("--app-text-color", getContrastTextColor(backgroundColor.value));
  const bgColor = backgroundColor.value;
  const bgImage = getBackgroundImageUrl(backgroundImage.value);
  const mainWrap = document.querySelector(".main-wrap") as HTMLElement | null;
  const mainContent = document.querySelector(".main-content") as HTMLElement | null;
  if (mainWrap) {
    mainWrap.style.backgroundColor = bgColor;
    mainWrap.style.backgroundImage = bgImage;
    mainWrap.style.backgroundSize = "cover";
    mainWrap.style.backgroundPosition = "center";
  }
  if (mainContent) {
    mainContent.style.backgroundColor = bgColor;
    mainContent.style.backgroundImage = bgImage;
    mainContent.style.backgroundSize = "cover";
    mainContent.style.backgroundPosition = "center";
  }
  // 主窗体：同步设置窗体级背景色；下一帧再设一次后触发“微移再还原”，模拟跨屏拖回以让 DWM 正确合成透明度
  if (isTauri.value) {
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow())
      .then((win) => {
        if (win.label === "main") {
          win.setBackgroundColor(bgColor).catch(() => {});
          requestAnimationFrame(() => {
            win.setBackgroundColor(bgColor).catch(() => {});
            requestAnimationFrame(() => {
              import("@tauri-apps/api/core")
                .then(({ invoke }) => invoke("refresh_main_window_transparency"))
                .catch(() => {});
            });
          });
        }
      })
      .catch(() => {});
  }
}

function applySidebarPositionClass() {
  document.documentElement.classList.toggle("sidebar-on-right", sidebarPosition.value === "right");
}

async function loadSettings() {
  if (!detectTauri()) return;
  try {
    const { load } = await import("@tauri-apps/plugin-store");
    settingsStore = await load(STORE_KEY, { autoSave: false, defaults: {} });
    const tc = await settingsStore.get(KEYS.themeColor);
    const bc = await settingsStore.get(KEYS.backgroundColor);
    const bi = await settingsStore.get(KEYS.backgroundImage);
    const sp = await settingsStore.get(KEYS.sidebarPosition);
    const pe = await settingsStore.get(KEYS.paginationEnabled);
    const ps = await settingsStore.get(KEYS.paginationPageSize);
    if (typeof tc === "string") themeColor.value = tc;
    if (typeof bc === "string") backgroundColor.value = bc;
    if (typeof bi === "string") backgroundImage.value = bi;
    if (sp === "left" || sp === "right") sidebarPosition.value = sp;
    if (typeof pe === "boolean") paginationEnabled.value = pe;
    if (typeof ps === "number") paginationPageSize.value = ps;
    applyTheme();
    if (isSidebarOnly.value) applySidebarPositionClass();
    if (isTauri.value) {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_sidebar_position", { position: sidebarPosition.value });
      } catch {
        // sidebar window may not exist yet
      }
    }
  } catch {
    settingsStore = null;
  }
}

const filteredTodoList = computed(() => {
  const list = todoList.value;
  const id = currentGroupId.value;
  const gs = groups.value;
  if (id === "all") return list;
  if (id === "ungrouped") {
    return list.filter((t) => !t.group_id || !gs.some((x) => x.id === t.group_id));
  }
  return list.filter((t) => t.group_id === id);
});

function countByGroupId(groupId: string): number {
  return todoList.value.filter((t) => t.group_id === groupId).length;
}

const ungroupedCount = computed(() => {
  const gs = groups.value;
  return todoList.value.filter((t) => !t.group_id || !gs.some((x) => x.id === t.group_id)).length;
});

const currentGroupTitle = computed(() => {
  const id = currentGroupId.value;
  if (id === "all") return t("desktop.all");
  if (id === "ungrouped") return t("desktop.ungrouped");
  const g = groups.value.find((x) => x.id === id);
  return g?.name ?? t("desktop.all");
});

/** 新增表单中分组的默认值：当前选中的是具体分组时用该分组，全部/未分组时为空 */
const defaultGroupIdForForm = computed(() => {
  const id = currentGroupId.value;
  if (id === "all" || id === "ungrouped") return undefined;
  return id;
});

async function openQrcode() {
  if (!detectTauri()) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("open_qrcode_window", { url: window.location.href || "" });
  } catch (e) {
    console.error(e);
    ElMessage.error(t("desktop.message.openWindowFailed"));
  }
}

async function openSettings() {
  if (!detectTauri()) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("open_settings_window", { url: window.location.href || "" });
  } catch (e) {
    console.error(e);
    ElMessage.error(t("desktop.message.settingsWindowFailed"));
  }
}

async function windowMinimize() {
  if (!isTauri.value) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().minimize();
  } catch (e) {
    console.error(e);
  }
}

async function windowToggleMaximize() {
  if (!isTauri.value) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().toggleMaximize();
  } catch (e) {
    console.error(e);
  }
}

async function windowClose() {
  if (!isTauri.value) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  } catch (e) {
    console.error(e);
  }
}

async function toggleAlwaysOnTop() {
  if (!isTauri.value) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    const next = !alwaysOnTop.value;
    await win.setAlwaysOnTop(next);
    alwaysOnTop.value = next;
  } catch (e) {
    console.error(e);
  }
}

async function loadData() {
  loading.value = true;
  try {
    const [list, gs] = await Promise.all([
      getTodoList(sortRule.value),
      getGroups(),
    ]);
    todoList.value = list;
    groups.value = gs;
  } catch {
    todoList.value = [];
  } finally {
    loading.value = false;
  }
}

function openAddDialog() {
  taskDialogMode.value = "add";
  taskForm.value = {
    content: addInputContent.value.trim(),
    status: "pending",
    importance: "normal",
    group_id: defaultGroupIdForForm.value,
  };
  taskTimeRange.value = null;
  taskDialogVisible.value = true;
}

function onTaskDialogClosed() {
  taskForm.value = { content: "", status: "pending", importance: "normal", group_id: undefined };
  taskTimeRange.value = null;
  editingTodoId = null;
  if (taskDialogMode.value === "add") addInputContent.value = "";
}

function onTaskGroupChange(value: string | undefined) {
  if (!value) return;
  const isExisting = groups.value.some((g) => g.id === value);
  if (!isExisting) onCreateGroup(value.trim());
}

function onTaskSubmit() {
  const content = taskForm.value.content.trim();
  if (!content) {
    ElMessage.warning(t("list.pleaseEnterContent"));
    return;
  }
  const [start_time, end_time] = taskTimeRange.value ?? [undefined, undefined];
  if (taskDialogMode.value === "add") {
    taskSubmitting.value = true;
    createTodo({
      content,
      importance: taskForm.value.importance,
      group_id: taskForm.value.group_id,
      start_time,
      end_time,
    })
      .then((item) => {
        todoList.value = [...todoList.value, item];
        emitTodosUpdated();
        taskDialogVisible.value = false;
        addInputContent.value = "";
        ElMessage.success(t("desktop.message.added"));
      })
      .catch(() => {})
      .finally(() => {
        taskSubmitting.value = false;
      });
  } else {
    if (!editingTodoId) return;
    taskSubmitting.value = true;
    const payload: UpdateTodoParams = {
      content,
      status: taskForm.value.status,
      importance: taskForm.value.importance,
      group_id: taskForm.value.group_id ?? undefined,
      start_time,
      end_time,
    };
    updateTodo(editingTodoId, payload)
      .then((updated) => {
        const idx = todoList.value.findIndex((t) => t.id === editingTodoId);
        if (idx !== -1) {
          const arr = [...todoList.value];
          arr[idx] = updated;
          todoList.value = arr;
        }
        emitTodosUpdated();
        taskDialogVisible.value = false;
        editingTodoId = null;
        ElMessage.success(t("desktop.saved"));
      })
      .catch(() => {})
      .finally(() => {
        taskSubmitting.value = false;
      });
  }
}

async function onTaskDelete() {
  if (!editingTodoId) return;
  try {
    await ElMessageBox.confirm(
      t("desktop.deleteTaskConfirm"),
      t("common.delete"),
      { type: "warning" }
    );
  } catch {
    return;
  }
  taskDeleting.value = true;
  deleteTodo(editingTodoId)
    .then(() => {
      todoList.value = todoList.value.filter((t) => t.id !== editingTodoId);
      emitTodosUpdated();
      taskDialogVisible.value = false;
      taskForm.value = { content: "", status: "pending", importance: "normal", group_id: undefined };
      taskTimeRange.value = null;
      editingTodoId = null;
      ElMessage.success(t("desktop.deleted"));
    })
    .catch(() => {})
    .finally(() => {
      taskDeleting.value = false;
    });
}

function onCreateGroup(name: string) {
  createGroup(name.trim())
    .then((g) => {
      groups.value = [...groups.value, g];
      newlyCreatedGroupId.value = g.id;
      setTimeout(() => { newlyCreatedGroupId.value = undefined; }, 500);
      currentGroupId.value = g.id;
      ElMessage.success(t("desktop.message.added"));
    })
    .catch(() => {});
}

function emitGroupsUpdated() {
  if (!isTauri.value) return;
  import("@tauri-apps/api/event").then(({ emit }) => {
    emit("groups-updated", {}).catch(() => {});
  });
}

function emitTodosUpdated() {
  if (!isTauri.value) return;
  import("@tauri-apps/api/event").then(({ emit }) => {
    emit("todos-updated", {}).catch(() => {});
  });
}

async function openManageGroupsDialog() {
  const gs = await getGroups();
  manageGroupsList.value = [...gs];
  manageGroupsEditNames.value = Object.fromEntries(manageGroupsList.value.map((g) => [g.id, g.name]));
  manageGroupsDialogVisible.value = true;
  manageGroupsNewName.value = "";
}

function initManageGroupsSortable() {
  sortableInstance?.destroy();
  sortableInstance = null;
  nextTick(() => {
    const el = manageGroupsListRef.value;
    if (!el) return;
    const sortableOptions = {
      animation: 150,
      handle: ".manage-groups-drag-handle",
      forceFallback: true,
      fallbackOnBody: true,
      ghostClass: "sortable-ghost",
      chosenClass: "sortable-chosen",
      dragClass: "sortable-drag",
      onEnd(ev: { oldIndex: number; newIndex: number }) {
        const from = ev.oldIndex;
        const to = ev.newIndex;
        if (from === to) return;
        const list = [...manageGroupsList.value];
        const [item] = list.splice(from, 1);
        list.splice(to, 0, item);
        manageGroupsList.value = list;
      },
    };
    sortableInstance = Sortable.create(el, sortableOptions as Parameters<typeof Sortable.create>[1]);
  });
}

function onAddGroup() {
  if (isSidebarOnly.value && isTauri.value) {
    import("@tauri-apps/api/event").then(({ emitTo }) => {
      emitTo("main", "open-manage-groups", {}).catch(() => {});
    });
    return;
  }
  ElMessageBox.prompt(t("desktop.message.pleaseEnterGroupName"), t("desktop.addGroup"), {
    confirmButtonText: t("desktop.add"),
    cancelButtonText: t("common.cancel"),
    inputPattern: /\S+/,
    inputErrorMessage: t("desktop.message.pleaseEnterGroupName"),
  })
    .then(({ value }) => {
      if (value && value.trim()) onCreateGroup(value.trim());
    })
    .catch(() => {});
}

async function manageGroupsAdd() {
  const name = manageGroupsNewName.value.trim();
  if (!name) return;
  manageGroupsSubmitting.value = true;
  try {
    const g = await createGroup(name);
    manageGroupsList.value = [...manageGroupsList.value, g];
    manageGroupsEditNames.value[g.id] = g.name;
    manageGroupsNewName.value = "";
    emitGroupsUpdated();
    ElMessage.success(t("desktop.message.added"));
  } catch {
    // error already shown in request
  } finally {
    manageGroupsSubmitting.value = false;
  }
}

function onManageGroupNameBlur(g: GroupItem, value: string) {
  const trimmed = value.trim();
  if (trimmed === g.name) return;
  if (!trimmed) {
    manageGroupsEditNames.value[g.id] = g.name;
    return;
  }
  manageGroupsUpdate(g.id, trimmed);
}

function manageGroupsUpdate(id: string, name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;
  updateGroup(id, trimmed)
    .then((updated) => {
      const list = manageGroupsList.value.map((x) => (x.id === id ? updated : x));
      manageGroupsList.value = list;
      manageGroupsEditNames.value[id] = updated.name;
      emitGroupsUpdated();
      ElMessage.success(t("desktop.message.saved"));
    })
    .catch(() => {});
}

function manageGroupsDelete(g: GroupItem) {
  ElMessageBox.confirm(t("desktop.deleteGroupConfirm"), t("desktop.dialog.groups"), {
    confirmButtonText: t("common.ok"),
    cancelButtonText: t("common.cancel"),
    type: "warning",
  })
    .then(() =>
      deleteGroup(g.id).then(() => {
        manageGroupsList.value = manageGroupsList.value.filter((x) => x.id !== g.id);
        const next = { ...manageGroupsEditNames.value };
        delete next[g.id];
        manageGroupsEditNames.value = next;
        emitGroupsUpdated();
        ElMessage.success(t("desktop.message.deleted"));
      })
    )
    .catch(() => {});
}

async function manageGroupsConfirm() {
  const orderedIds = manageGroupsList.value.map((x) => x.id);
  if (orderedIds.length === 0) {
    manageGroupsDialogVisible.value = false;
    sortableInstance?.destroy();
    sortableInstance = null;
    return;
  }
  manageGroupsSubmitting.value = true;
  try {
    await reorderGroups(orderedIds);
    emitGroupsUpdated();
    manageGroupsDialogVisible.value = false;
    groups.value = [...manageGroupsList.value];
    ElMessage.success(t("desktop.message.saved"));
  } catch {
    // error already shown
  } finally {
    manageGroupsSubmitting.value = false;
    sortableInstance?.destroy();
    sortableInstance = null;
  }
}

function onManageGroupsDialogClosed() {
  manageGroupsEditNames.value = {};
  sortableInstance?.destroy();
  sortableInstance = null;
}

function isInCurrentGroup(t: TodoItem): boolean {
  const gid = currentGroupId.value;
  const gs = groups.value;
  if (gid === "all") return true;
  if (gid === "ungrouped") return !t.group_id || !gs.some((x) => x.id === t.group_id);
  return t.group_id === gid;
}

function onToggle(id: string, newStatus: "pending" | "completed") {
  updateTodoStatus(id, newStatus)
    .then((updated) => {
      const idx = todoList.value.findIndex((t) => t.id === id);
      if (idx !== -1) {
        const arr = [...todoList.value];
        arr[idx] = updated;
        todoList.value = arr;
      }
      emitTodosUpdated();

      if (newStatus === "completed" || newStatus === "pending") {
        const ordered = todoList.value;
        const currentIds = ordered.map((t) => t.id);
        const groupEntries = ordered.map((t, i) => ({ t, i })).filter(({ t }) => isInCurrentGroup(t));
        const groupIndices = groupEntries.map(({ i }) => i);
        const groupTasks = groupEntries.map(({ t }) => t);
        let groupNewOrder: string[];
        if (newStatus === "completed") {
          const pending = groupTasks.filter((t) => t.status === "pending");
          const completed = groupTasks.filter((t) => t.status === "completed");
          const completedOrder = [id, ...completed.filter((t) => t.id !== id).map((t) => t.id)];
          groupNewOrder = [...pending.map((t) => t.id), ...completedOrder];
        } else {
          groupNewOrder = [id, ...groupTasks.filter((t) => t.id !== id).map((t) => t.id)];
        }
        const newIds = currentIds.slice();
        groupIndices.forEach((pos, j) => {
          newIds[pos] = groupNewOrder[j];
        });
        reorderTodos(newIds)
          .then(() => loadData())
          .then(() => emitTodosUpdated())
          .catch(() => {});
      }
    })
    .catch(() => {});
}

function onReorder(orderedIds: string[]) {
  reorderTodos(orderedIds)
    .then(() => loadData())
    .then(() => emitTodosUpdated())
    .catch(() => {});
}

function onEdit(row: TodoItem) {
  taskDialogMode.value = "edit";
  editingTodoId = row.id;
  taskForm.value = {
    content: row.content,
    status: row.status,
    importance: row.importance,
    group_id: row.group_id ?? undefined,
  };
  taskTimeRange.value =
    row.start_time && row.end_time ? [row.start_time, row.end_time] : null;
  taskDialogVisible.value = true;
}

/** 任务弹窗打开时，将蒙层限制在非 header/footer 区域（左右全宽，上下各留 40px） */
const TASK_OVERLAY_TOP = 40;
const TASK_OVERLAY_BOTTOM = 40;
function applyTaskDialogOverlayInset() {
  nextTick(() => {
    setTimeout(() => {
      const dialogEl = document.querySelector(".task-dialog");
      if (!dialogEl) return;
      const wrapper = dialogEl.closest(".el-dialog__wrapper");
      if (!wrapper) return;
      let overlay: HTMLElement | null = wrapper.previousElementSibling as HTMLElement | null;
      if (!overlay?.classList.contains("el-overlay")) {
        overlay = wrapper.nextElementSibling as HTMLElement | null;
      }
      if (overlay?.classList.contains("el-overlay")) {
        overlay.style.top = `${TASK_OVERLAY_TOP}px`;
        overlay.style.bottom = `${TASK_OVERLAY_BOTTOM}px`;
      }
    }, 0);
  });
}
watch(taskDialogVisible, (visible) => {
  if (visible) applyTaskDialogOverlayInset();
});

// 侧栏窗口：分组选中变化时同步到主窗体
watch(
  currentGroupId,
  (groupId) => {
    if (isSidebarOnly.value && isTauri.value && groupId != null) {
      import("@tauri-apps/api/event").then(({ emitTo }) => {
        emitTo("main", "group-selected", { groupId }).catch(() => {});
      });
    }
  },
  { flush: "sync" }
);

let unlistenSettings: (() => void) | null = null;
let unlistenGroupSelected: (() => void) | null = null;
let unlistenOpenManageGroups: (() => void) | null = null;
let unlistenGroupsUpdated: (() => void) | null = null;
let unlistenTodosUpdated: (() => void) | null = null;

/** 手机端等外部修改数据后，主窗体定时拉取以同步（仅主窗体、Tauri 环境） */
const SYNC_POLL_INTERVAL_MS = 8000;
let syncPollTimer: ReturnType<typeof setInterval> | null = null;
function onVisibilityChange() {
  if (typeof document !== "undefined" && document.visibilityState === "visible") {
    loadData();
  }
}
function startSyncPoll() {
  if (syncPollTimer) return;
  syncPollTimer = setInterval(() => {
    if (typeof document !== "undefined" && document.visibilityState === "visible") {
      loadData();
    }
  }, SYNC_POLL_INTERVAL_MS);
  document.addEventListener("visibilitychange", onVisibilityChange);
}
function stopSyncPoll() {
  if (syncPollTimer) {
    clearInterval(syncPollTimer);
    syncPollTimer = null;
  }
  document.removeEventListener("visibilitychange", onVisibilityChange);
}

onMounted(async () => {
  isTauri.value = detectTauri();
  // 立即应用一次主题（含透明背景），避免启动首帧显示不透明
  applyTheme();
  if (isTauri.value) {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      alwaysOnTop.value = await getCurrentWindow().isAlwaysOnTop();
    } catch {
      // ignore
    }
  }
  if (isSidebarOnly.value) {
    currentGroupId.value = "all";
  }
  await loadSettings();
  await loadData();
  const url = await getMobileUrl();
  syncServerUrl.value = url || "http://127.0.0.1:8080/mobile";
  if (isTauri.value) {
    const { listen } = await import("@tauri-apps/api/event");
    unlistenSettings = await listen<{
      themeColor?: string;
      backgroundColor?: string;
      backgroundImage?: string;
      sidebarPosition?: "left" | "right";
      paginationEnabled?: boolean;
      paginationPageSize?: number;
    }>("settings-changed", (e) => {
      if (e.payload.themeColor) themeColor.value = e.payload.themeColor;
      if (e.payload.backgroundColor) backgroundColor.value = e.payload.backgroundColor;
      if (e.payload.backgroundImage !== undefined) backgroundImage.value = e.payload.backgroundImage;
      if (e.payload.sidebarPosition === "left" || e.payload.sidebarPosition === "right") {
        sidebarPosition.value = e.payload.sidebarPosition;
        if (isSidebarOnly.value) applySidebarPositionClass();
        import("@tauri-apps/api/core")
          .then(({ invoke }) => invoke("set_sidebar_position", { position: sidebarPosition.value }).catch(() => {}));
      }
      if (typeof e.payload.paginationEnabled === "boolean") paginationEnabled.value = e.payload.paginationEnabled;
      if (typeof e.payload.paginationPageSize === "number") paginationPageSize.value = e.payload.paginationPageSize;
      nextTick(() => applyTheme());
    });
    // 主窗体：监听侧栏发来的分组选中，同步当前分组
    unlistenGroupSelected = await listen<{ groupId: string | "all" | "ungrouped" }>("group-selected", (e) => {
      if (!isSidebarOnly.value && e.payload?.groupId != null) {
        currentGroupId.value = e.payload.groupId;
      }
    });
    // 主窗体：监听分组窗体发来的“打开分组管理”，弹出分组管理弹窗
    unlistenOpenManageGroups = await listen("open-manage-groups", () => {
      if (!isSidebarOnly.value) openManageGroupsDialog();
    });
    // 分组窗体：监听主窗体发来的“分组已更新”，刷新分组列表
    unlistenGroupsUpdated = await listen("groups-updated", async () => {
      if (isSidebarOnly.value) {
        const gs = await getGroups();
        groups.value = gs;
      }
    });
    // 分组窗体：监听主窗体发来的“任务列表已更新”，刷新任务列表以同步数量
    unlistenTodosUpdated = await listen("todos-updated", async () => {
      if (isSidebarOnly.value) {
        const list = await getTodoList(sortRule.value);
        todoList.value = list;
      }
    });
    if (!isSidebarOnly.value) {
      startSyncPoll();
    }
  }
});

onUnmounted(() => {
  unlistenSettings?.();
  unlistenGroupSelected?.();
  unlistenOpenManageGroups?.();
  unlistenGroupsUpdated?.();
  unlistenTodosUpdated?.();
  stopSyncPoll();
});
</script>

<style scoped>
.desktop-layout {
  display: flex;
  height: 100vh;
  width: 100%;
  min-height: 0;
  overflow: hidden; /* 避免启动时整页出现横/竖条，拖动后消失 */
  background: transparent;
}
.sidebar {
  flex-shrink: 0;
  width: 200px;
  display: flex;
  flex-direction: column;
  background: transparent;
  overflow: hidden;
}
.sidebar-groups {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
  padding: 12px 0 12px 0;
  overflow-y: auto;
}
/* 分组窗体：第一个分组项上边与主窗体 header 下边对齐（header 高度 40px） */
.desktop-layout.is-sidebar-only .sidebar-groups {
  padding-top: 40px;
}
.sidebar-item {
  --sidebar-item-border-width: 1px;
  --sidebar-item-border-left-width: 1px;
  --sidebar-item-border-bottom-width: 1px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 40px;
  padding: 8px 12px;
  border: none;
  border-top: var(--sidebar-item-border-width) solid var(--el-color-primary, #409eff);
  border-left: var(--sidebar-item-border-left-width) solid var(--el-color-primary, #409eff);
  border-bottom: var(--sidebar-item-border-bottom-width) solid var(--el-color-primary, #409eff);
  border-right: none;
  border-radius: 8px 0 0 8px;
  background: var(--sidebar-item-bg, rgba(0, 0, 0, 0.06));
  color: var(--app-text-color, #1a1a1a);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  box-shadow: none;
  transition:
    background 0.2s ease,
    color 0.2s ease,
    width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    max-width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    border-width 0.2s ease,
    box-shadow 0.2s ease;
  box-sizing: border-box;
}
/* 非新增按钮：未选中 80px，不显示数量、名称不省略；抽屉动画用 overflow 裁剪 */
.sidebar-item:not(.sidebar-item-add) {
  width: 80px;
  overflow: hidden;
}
.sidebar-item:not(.sidebar-item-add) .sidebar-item-name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: clip;
  flex: 0 1 auto;
  min-width: 0;
  line-height: 1;
  display: flex;
  align-items: center;
}
.sidebar-item:not(.sidebar-item-add) .sidebar-item-count {
  flex-shrink: 0;
  font-size: 12px;
  line-height: 1;
  display: flex;
  align-items: center;
  opacity: 0;
  max-width: 0;
  margin-left: 0;
  overflow: hidden;
  white-space: nowrap;
  transition:
    opacity 0.25s cubic-bezier(0.4, 0, 0.2, 1),
    max-width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    margin-left 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
/* 非新增按钮：选中 最长150px，名称+10px+数量，抽屉展开动画；左边加粗，取消下边框 */
.sidebar-item:not(.sidebar-item-add).active {
  --sidebar-item-border-width: 1px;
  --sidebar-item-border-left-width: 5px;
  --sidebar-item-border-bottom-width: 0;
  width: auto;
  max-width: 150px;
}
.sidebar-item:not(.sidebar-item-add).active .sidebar-item-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.sidebar-item:not(.sidebar-item-add).active .sidebar-item-count {
  opacity: 0.95;
  max-width: 48px;
  margin-left: 10px;
}
.sidebar-item:hover {
  background: var(--sidebar-item-bg, rgba(0, 0, 0, 0.06));
  color: var(--app-text-color, #1a1a1a);
}
.sidebar-item.active {
  background: var(--sidebar-item-bg, rgba(0, 0, 0, 0.06));
  color: var(--app-text-color, #1a1a1a);
  font-weight: bold;
  box-shadow:
    0 -2px 6px color-mix(in srgb, var(--el-color-primary, #409eff) 22%, transparent),
    -5px 0 10px color-mix(in srgb, var(--el-color-primary, #409eff) 28%, transparent),
    0 5px 10px color-mix(in srgb, var(--el-color-primary, #409eff) 28%, transparent);
}
.sidebar-item-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1;
  display: flex;
  align-items: center;
}
.sidebar-item-count {
  flex-shrink: 0;
  margin-left: 8px;
  font-size: 12px;
  opacity: 0.85;
  line-height: 1;
  display: flex;
  align-items: center;
}
.sidebar-item-add {
  width: auto;
  min-width: 0;
  flex-shrink: 0;
  justify-content: flex-start;
  align-self: flex-end;
}
.sidebar-item-add-text {
  flex: 0 1 auto;
  min-width: 0;
  overflow: visible;
  white-space: nowrap;
  line-height: 1;
  display: flex;
  align-items: center;
}
.sidebar-item-add-icon {
  flex-shrink: 0;
  margin-left: 10px;
  font-size: 18px;
  line-height: 1;
  display: flex;
  align-items: center;
}
.main-wrap {
  flex: 1;
  min-width: 0;
  min-height: 0; /* 避免启动时高度未收敛出现多余竖条，拖动后消失 */
  display: flex;
  flex-direction: column;
  overflow-x: hidden;
  overflow-y: auto;
  background: var(--sidebar-item-bg, #fff);
}
.desktop-title-bar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 12px 0 16px;
  background: var(--sidebar-item-bg, #f5f5f5);
  color: var(--app-text-color, #1a1a1a);
  border-bottom: 1px solid var(--app-border-color, #e8e8e8);
}
.desktop-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  flex: 1;
  text-align: left;
  pointer-events: none;
}
.desktop-title-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}
.pin-icon {
  width: 1em;
  height: 1em;
  font-size: 16px;
}
.title-bar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}
.title-bar-btn:hover {
  background: color-mix(in srgb, var(--el-color-primary, #409eff) 15%, transparent);
  color: var(--el-color-primary, #409eff);
}
.title-bar-btn-active {
  background: color-mix(in srgb, var(--el-color-primary, #409eff) 20%, transparent);
  color: var(--el-color-primary, #409eff);
}
.title-bar-btn-close:hover {
  background: #e81123;
  color: #fff;
}
.main-content {
  flex: 1;
  min-width: 0;
  min-height: 0; /* 与 .main-wrap 一致，避免启动时多余横/竖条 */
  display: flex;
  flex-direction: column;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 16px;
  background: var(--sidebar-item-bg, #fff);
}

/* 主窗体滚动条：无背景、无上下箭头 */
.main-wrap,
.main-content {
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--app-text-color, #333) 25%, transparent) transparent;
}
.main-wrap::-webkit-scrollbar,
.main-content::-webkit-scrollbar {
  width: 8px;
}
.main-wrap::-webkit-scrollbar-track,
.main-content::-webkit-scrollbar-track {
  background: transparent;
}
.main-wrap::-webkit-scrollbar-thumb,
.main-content::-webkit-scrollbar-thumb {
  border-radius: 4px;
  background: color-mix(in srgb, var(--app-text-color, #333) 25%, transparent);
}
.main-wrap::-webkit-scrollbar-thumb:hover,
.main-content::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--app-text-color, #333) 40%, transparent);
}
.main-wrap::-webkit-scrollbar-button,
.main-content::-webkit-scrollbar-button {
  display: none;
  height: 0;
  width: 0;
}
.form-card {
  flex-shrink: 0;
  margin-bottom: 0;
}
.form-card-simple {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.form-card-simple .add-input {
  flex: 1;
  min-width: 160px;
}
.form-card-simple .add-input :deep(.el-input__wrapper) {
  background-color: transparent !important;
  box-shadow: 0 0 0 1px var(--app-border-color, #dcdfe6) inset;
}
.form-card-simple .add-input :deep(.el-input__inner) {
  color: var(--app-text-color, #1a1a1a) !important;
}
.form-card-simple .add-input :deep(.el-input__inner::placeholder) {
  color: color-mix(in srgb, var(--app-text-color, #1a1a1a) 50%, transparent) !important;
}
.form-card-simple .add-input :deep(.el-input__count),
.form-card-simple .add-input :deep(.el-input__count-inner) {
  background: transparent !important;
}
.form-card-simple .add-input :deep(.el-input__count) {
  color: color-mix(in srgb, var(--app-text-color, #1a1a1a) 65%, transparent) !important;
  font-size: 12px;
}
/* 主窗体添加按钮禁用态：避免过白，使用主题色弱化 */
.form-card-simple .add-btn.is-disabled {
  background-color: color-mix(in srgb, var(--el-color-primary, #409eff) 22%, var(--app-bg-color, #f5f5f5)) !important;
  border-color: color-mix(in srgb, var(--el-color-primary, #409eff) 28%, var(--app-border-color, #dcdfe6)) !important;
  color: color-mix(in srgb, var(--app-text-color, #1a1a1a) 55%, transparent) !important;
}
.form-table-divider {
  flex-shrink: 0;
  height: 0;
  margin: 0;
  margin-top: 20px;
  margin-bottom: 16px;
  border: none;
  border-top: 1px solid var(--app-border-color, #e8e8e8);
}
.desktop-footer {
  flex-shrink: 0;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--app-text-color, #1a1a1a);
  background: var(--sidebar-item-bg, rgba(0, 0, 0, 0.06));
  border-top: 1px solid var(--app-border-color, #e8e8e8);
}
.footer-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--app-text-color, #1a1a1a);
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}
.footer-icon-btn:hover {
  background: color-mix(in srgb, var(--app-text-color, #1a1a1a) 12%, transparent);
  color: var(--el-color-primary, #409eff);
}
.desktop-footer-text {
  min-width: 0;
}
.desktop-footer-spacer {
  flex: 1;
  min-width: 0;
}

/* 新增/编辑任务弹窗（复用）：宽高随主窗体自适应，左右间距缩小 */
/* 在 .el-dialog 上覆盖 --el-dialog-width（Element 默认 50% 定义在 .el-dialog 自身），并强制 width */
.task-dialog :deep(.el-dialog) {
  --el-dialog-width: calc(100vw - 40px);
  width: var(--el-dialog-width) !important;
  max-width: var(--el-dialog-width) !important;
  max-height: calc(100vh - 40px);
  display: flex;
  flex-direction: column;
}
.task-dialog :deep(.el-dialog__body) {
  max-height: calc(100vh - 140px);
  overflow-y: auto;
}
.task-dialog .dialog-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  width: 100%;
}
.task-dialog .dialog-footer-right {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
/* 任务弹窗：重要性选项内的颜色点 */
.importance-option {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.importance-option-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.importance-option-dot-normal {
  background: var(--app-text-color, #909399);
  opacity: 0.7;
}
.importance-option-dot-important {
  background: var(--el-color-warning, #e6a23c);
}
.importance-option-dot-urgent {
  background: var(--el-color-danger, #f56c6c);
}
/* 分组管理弹窗：宽度随主窗体自适应，不超过主窗体（覆盖 .el-dialog 自身的 50% 默认） */
.manage-groups-dialog :deep(.el-dialog) {
  --el-dialog-width: min(420px, calc(100vw - 48px));
  width: var(--el-dialog-width) !important;
  max-width: calc(100vw - 48px) !important;
}
.manage-groups-add {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.manage-groups-add .el-input {
  flex: 1;
}
.manage-groups-list {
  max-height: 320px;
  overflow-y: auto;
  min-height: 40px;
}
.manage-groups-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid var(--el-border-color-lighter, #ebeef5);
}
.manage-groups-item:last-child {
  border-bottom: none;
}
.manage-groups-drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  cursor: grab;
  color: var(--el-text-color-secondary, #909399);
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
}
.manage-groups-drag-handle:active {
  cursor: grabbing;
}
.manage-groups-item .el-input {
  flex: 1;
}
</style>

<style>
/* 任务弹窗蒙层仅覆盖主内容区，不覆盖主窗体 header 与 footer（各 40px），便于拖拽标题栏 */
.el-overlay:has(.task-dialog) {
  top: 40px;
  bottom: 40px;
}
/* 弹窗 teleport 到 body 后，scoped 样式可能不生效，用全局样式强制宽度（覆盖 Element 默认 50%） */
.el-dialog.task-dialog,
.el-overlay:has(.task-dialog) .el-dialog {
  --el-dialog-width: calc(100vw - 40px) !important;
  width: var(--el-dialog-width) !important;
  max-width: var(--el-dialog-width) !important;
}
.el-dialog.manage-groups-dialog,
.el-overlay:has(.manage-groups-dialog) .el-dialog {
  --el-dialog-width: min(420px, calc(100vw - 48px)) !important;
  width: var(--el-dialog-width) !important;
  max-width: calc(100vw - 48px) !important;
}
</style>
