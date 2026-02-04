<template>
  <div class="todo-list">
    <div class="toolbar">
      <el-radio-group v-model="sortRule" size="small" @change="emit('sortChange', sortRule)">
        <el-radio-button value="comprehensive">{{ $t('list.sortComprehensive') }}</el-radio-button>
        <el-radio-button value="importance">{{ $t('list.sortImportance') }}</el-radio-button>
        <el-radio-button value="deadline">{{ $t('list.sortDeadline') }}</el-radio-button>
      </el-radio-group>
      <el-tooltip :content="$t('list.refresh')" placement="top">
        <el-button size="small" circle :loading="loading" @click="$emit('refresh')">
          <el-icon><Refresh /></el-icon>
        </el-button>
      </el-tooltip>
    </div>
    <el-table
      ref="tableRef"
      v-loading="loading"
      :data="pagedTodoList"
      stripe
      row-key="id"
      :show-header="false"
      style="width: 100%"
      :empty-text="$t('list.empty')"
      :row-class-name="rowClassName"
      @row-dblclick="onRowClick"
    >
      <el-table-column type="index" width="36" align="center">
        <template #default="{ row }">
          <span
            v-if="row.status !== 'completed'"
            class="drag-handle"
            data-drag-handle
          >⋮⋮</span>
          <span v-else class="drag-handle-placeholder">⋮⋮</span>
        </template>
      </el-table-column>
      <el-table-column prop="content" :label="$t('list.content')" min-width="120">
        <template #default="{ row }">
          <div class="content-cell">
            <span
              class="importance-dot"
              :class="`importance-dot-${row.importance}`"
              :title="importanceLabel(row.importance)"
              aria-hidden="true"
            />
            <div class="content-main">
              <span class="content-title" :class="{ 'content-completed': row.status === 'completed' }">{{ row.content }}</span>
              <span class="content-sub">
                {{ groupName(row.group_id) }}
                <el-icon
                  v-if="row.start_time || row.end_time"
                  class="sub-clock"
                  :title="timeRangeTitle(row.start_time, row.end_time)"
                >
                  <Clock />
                </el-icon>
              </span>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column :label="$t('list.status')" width="56" align="center">
        <template #default="{ row }">
          <button
            type="button"
            class="status-radio"
            :class="{ checked: row.status === 'completed' }"
            :aria-label="row.status === 'completed' ? $t('list.toggleToPending') : $t('list.toggleToCompleted')"
            @click.stop="emit('toggle', row.id, row.status === 'completed' ? 'pending' : 'completed')"
          >
            <span class="status-radio-inner" />
          </button>
        </template>
      </el-table-column>
    </el-table>
    <div v-if="paginationEnabled && totalItems > pageSize" class="pagination-wrap">
      <el-pagination
        v-model:current-page="currentPage"
        :page-size="pageSize"
        layout="prev, pager, next"
        :total="totalItems"
        size="small"
        background
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { Refresh, Clock } from "@element-plus/icons-vue";
import Sortable from "sortablejs";
import type { TodoItem, GroupItem, SortRule, Importance } from "@/utils/request";

const { t, locale } = useI18n();

const props = withDefaults(
  defineProps<{
    todoList: TodoItem[];
    loading: boolean;
    groups?: GroupItem[];
    sortRule?: SortRule;
    /** 选中分组时隐藏「分组」列 */
    hideGroupColumn?: boolean;
    /** 是否启用分页 */
    paginationEnabled?: boolean;
    /** 每页条数 */
    pageSize?: number;
  }>(),
  { groups: () => [], sortRule: "comprehensive", hideGroupColumn: false, paginationEnabled: false, pageSize: 10 }
);

const sortRule = ref<SortRule>(props.sortRule);
const tableRef = ref<{ $el: HTMLElement } | null>(null);
const isDragging = ref(false);
let sortable: Sortable | null = null;

const currentPage = ref(1);
const pageSize = computed(() => (props.pageSize && props.pageSize > 0 ? props.pageSize : 10));
const paginationEnabled = computed(() => !!props.paginationEnabled);
const totalItems = computed(() => props.todoList.length);
const pagedTodoList = computed(() => {
  if (!paginationEnabled.value) return props.todoList;
  const start = (currentPage.value - 1) * pageSize.value;
  return props.todoList.slice(start, start + pageSize.value);
});

function initSortable() {
  if (isDragging.value) return;
  sortable?.destroy();
  sortable = null;
  if (props.loading || props.todoList.length === 0) return;
  const table = tableRef.value;
  if (!table) return;
  const el = (table as { $el?: HTMLElement }).$el ?? (table as unknown as HTMLElement);
  if (!el || typeof el.querySelector !== "function") return;
  const bodyWrapper = el.querySelector(".el-table__body-wrapper");
  const tbody = bodyWrapper?.querySelector("tbody") ?? el.querySelector("table tbody") ?? el.querySelector("tbody");
  if (!tbody || !(tbody instanceof HTMLElement)) return;
  if (tbody.querySelector("tr.sortable-ghost")) return;
  sortable = Sortable.create(tbody, {
    handle: "[data-drag-handle]",
    draggable: "tr",
    animation: 150,
    ghostClass: "sortable-ghost",
    chosenClass: "sortable-chosen",
    dragClass: "sortable-drag",
    forceFallback: true,
    fallbackOnBody: true,
    fallbackTolerance: 5,
    swapThreshold: 0.5,
    invertSwap: true,
    filter: ".el-table__empty-block, .row-completed",
    preventOnFilter: false,
    onStart() {
      isDragging.value = true;
    },
    onEnd(evt) {
      isDragging.value = false;
      const oldIndex = evt.oldIndex;
      const newIndex = evt.newIndex;
      sortable?.destroy();
      sortable = null;
      if (oldIndex == null || newIndex == null || oldIndex === newIndex) return;
      const fullList = [...props.todoList];
      let orderedIds: string[];
      if (paginationEnabled.value) {
        const start = (currentPage.value - 1) * pageSize.value;
        const pageList = fullList.slice(start, start + pageSize.value);
        const [item] = pageList.splice(oldIndex, 1);
        pageList.splice(newIndex, 0, item);
        orderedIds = [
          ...fullList.slice(0, start).map((r) => r.id),
          ...pageList.map((r) => r.id),
          ...fullList.slice(start + pageSize.value).map((r) => r.id),
        ];
      } else {
        const [item] = fullList.splice(oldIndex, 1);
        fullList.splice(newIndex, 0, item);
        orderedIds = fullList.map((r) => r.id);
      }
      emit("reorder", orderedIds);
    },
  });
}

watch(
  () => [props.todoList.length, props.loading, props.todoList.map((t) => t.id).join(","), props.paginationEnabled, props.pageSize] as const,
  () => {
    if (!paginationEnabled.value) {
      currentPage.value = 1;
    } else {
      // 列表变化时，确保页码不越界
      const maxPage = Math.max(1, Math.ceil(totalItems.value / pageSize.value));
      currentPage.value = Math.min(currentPage.value, maxPage);
    }
    nextTick(() => {
      setTimeout(() => initSortable(), 120);
    });
  },
  { immediate: true }
);

onMounted(() => {
  nextTick(() => {
    setTimeout(() => initSortable(), 200);
  });
});

function rowClassName({ row }: { row: TodoItem }) {
  return row.status === "completed" ? "row-completed" : "";
}

function onRowClick(row: TodoItem, _column: unknown, evt: MouseEvent) {
  if (isDragging.value) return;
  const target = evt.target as HTMLElement;
  if (target.closest("button") || target.closest(".drag-handle") || target.closest("[data-drag-handle]")) return;
  emit("edit", row);
}

const emit = defineEmits<{
  (e: "toggle", id: string, newStatus: "pending" | "completed"): void;
  (e: "refresh"): void;
  (e: "reorder", orderedIds: string[]): void;
  (e: "sortChange", sort: SortRule): void;
  (e: "edit", row: TodoItem): void;
}>();

watch(
  () => props.sortRule,
  (v) => (sortRule.value = v)
);

function groupName(groupId?: string) {
  if (!groupId) return "—";
  const g = props.groups.find((x) => x.id === groupId);
  return g?.name ?? "—";
}

function importanceLabel(imp: Importance) {
  if (imp === "normal") return t("common.normal");
  if (imp === "important") return t("common.important");
  if (imp === "urgent") return t("common.urgent");
  return imp;
}

function formatTime(s: string) {
  if (!s) return "—";
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

function timeRangeTitle(start?: string, end?: string) {
  if (!start && !end) return "";
  const a = start ? formatTime(start) : "—";
  const b = end ? formatTime(end) : "—";
  return `${a} ~ ${b}`;
}

onUnmounted(() => {
  sortable?.destroy();
  sortable = null;
});
</script>

<style scoped>
.todo-list {
  background: transparent;
  border: none;
  border-radius: 8px;
  padding: 16px 0;
  box-shadow: none;
  color: var(--app-text-color, #1a1a1a);
  --el-table-bg-color: transparent;
  --el-table-tr-bg-color: transparent;
  --el-table-header-bg-color: transparent;
  --el-table-row-hover-bg-color: color-mix(in srgb, var(--app-border-color, #409eff) 12%, transparent);
  /* 表格分割线使用主题色 */
  --el-table-border-color: var(--app-border-color, #ebeef5);
  --el-table-border: 1px solid var(--app-border-color, #ebeef5);
}
.toolbar {
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  /* 让筛选条件组内 Element 组件的边框变量使用主题色 */
  --el-border-color: var(--app-border-color, #dcdfe6);
  --el-border-color-hover: var(--app-border-color, #dcdfe6);
}
.drag-handle {
  display: inline-block;
  cursor: grab !important;
  color: var(--app-text-color, #999);
  opacity: 0.7;
  user-select: none;
  pointer-events: auto;
}
.drag-handle:active {
  cursor: grabbing !important;
}
.drag-handle-placeholder {
  color: var(--app-text-color, #999);
  opacity: 0.4;
  user-select: none;
  cursor: default;
}
.content-cell {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  min-width: 0;
}
.importance-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-top: 6px;
}
.importance-dot-normal {
  background: var(--app-text-color, #909399);
  opacity: 0.7;
}
.importance-dot-important {
  background: var(--el-color-warning, #e6a23c);
}
.importance-dot-urgent {
  background: var(--el-color-danger, #f56c6c);
}
.content-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.content-title {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}
.content-sub {
  font-size: 12px;
  color: var(--app-text-color, #909399);
  opacity: 0.85;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.sub-clock {
  font-size: 12px;
  opacity: 0.9;
}
.content-completed {
  text-decoration: line-through;
  color: var(--app-text-color, #666);
  opacity: 0.85;
}
/* 状态列：单个 radio 样式，勾选=已完成，未勾选=待处理 */
.status-radio {
  width: 20px;
  height: 20px;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.status-radio-inner {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--app-border-color, #dcdfe6);
  background: transparent;
  color: var(--app-text-color, #1a1a1a);
  transition: border-color 0.2s, background 0.2s;
}
.status-radio.checked .status-radio-inner {
  border-color: var(--el-color-primary, #409eff);
  background: var(--el-color-primary, #409eff);
  box-shadow: inset 0 0 0 3px var(--el-bg-color, #fff);
}
.status-radio:hover .status-radio-inner {
  border-color: var(--el-color-primary, #409eff);
}
.pagination-wrap {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
/* 分页：非选中项背景透明、边框主题色、文字反色（主题色） */
.pagination-wrap :deep(.el-pager li:not(.is-active)),
.pagination-wrap :deep(.btn-prev:not(:disabled)),
.pagination-wrap :deep(.btn-next:not(:disabled)) {
  background: transparent !important;
  border: 1px solid var(--el-color-primary, #409eff) !important;
  color: var(--el-color-primary, #409eff) !important;
}
.pagination-wrap :deep(.el-pager li:not(.is-active):hover),
.pagination-wrap :deep(.btn-prev:not(:disabled):hover),
.pagination-wrap :deep(.btn-next:not(:disabled):hover) {
  color: var(--el-color-primary, #409eff) !important;
}
.pagination-wrap :deep(.el-pager li.is-active) {
  background: var(--el-color-primary, #409eff) !important;
  border-color: var(--el-color-primary, #409eff) !important;
  color: #fff !important;
}
/* 分页禁用态：使用背景色配置，并与背景区分（略混入文字色/降不透明度） */
.pagination-wrap :deep(.btn-prev:disabled),
.pagination-wrap :deep(.btn-next:disabled) {
  background: color-mix(in srgb, var(--app-text-color, #1a1a1a) 18%, var(--app-bg-color, #f5f5f5)) !important;
  border-color: color-mix(in srgb, var(--app-text-color, #1a1a1a) 25%, var(--app-bg-color, #f5f5f5)) !important;
  color: color-mix(in srgb, var(--app-text-color, #1a1a1a) 45%, var(--app-bg-color, #f5f5f5)) !important;
  cursor: not-allowed;
}
/* 表格背景透明，文字随背景反色 */
:deep(.el-table),
:deep(.el-table__header-wrapper),
:deep(.el-table__body-wrapper),
:deep(.el-table th.el-table__cell),
:deep(.el-table td.el-table__cell),
:deep(.el-table tr),
:deep(.el-table--striped .el-table__body tr.el-table__row--striped td.el-table__cell) {
  background: transparent !important;
}
:deep(.el-table th.el-table__cell),
:deep(.el-table td.el-table__cell),
:deep(.el-table__empty-block) {
  color: var(--app-text-color, #1a1a1a);
}
:deep(.el-table th.el-table__cell) {
  border-bottom: 1px solid var(--app-border-color, #ebeef5) !important;
  border-right: none !important;
}
:deep(.el-table td.el-table__cell) {
  border-bottom: 1px solid color-mix(in srgb, var(--app-border-color, #ebeef5) 35%, transparent) !important;
  border-right: none !important;
}
:deep(.el-table__inner-wrapper::before) {
  background: transparent;
}
/* 筛选条件组（综合/重要性/截止时间）：边框使用主题色 */
:deep(.el-radio-group .el-radio-button__inner) {
  background: transparent !important;
  border: 1px solid var(--app-border-color, #dcdfe6) !important;
  border-color: var(--app-border-color, #dcdfe6) !important;
  outline: 1px solid var(--app-border-color, #dcdfe6) !important;
  outline-offset: -1px;
  color: var(--app-text-color, #1a1a1a);
}
:deep(.el-radio-group .el-radio-button:first-child .el-radio-button__inner),
:deep(.el-radio-group .el-radio-button .el-radio-button__inner) {
  box-shadow: none !important;
}
:deep(.el-radio-group .el-radio-button__original-radio:checked + .el-radio-button__inner) {
  background: var(--el-color-primary) !important;
  border-color: var(--el-color-primary) !important;
  color: #fff;
  box-shadow: -1px 0 0 0 var(--el-color-primary) !important;
}
:deep(.el-radio-group .el-radio-button:hover .el-radio-button__inner) {
  color: var(--el-color-primary);
}
/* 刷新列表：主题色图标按钮（circle 使用 type="primary" 在模板中设置） */
:deep(.toolbar .el-button.is-circle) {
  color: var(--el-color-primary);
  border-color: var(--app-border-color, #dcdfe6);
  background: transparent;
}
:deep(.toolbar .el-button.is-circle:hover) {
  color: #fff;
  background: var(--el-color-primary);
  border-color: var(--el-color-primary);
}
:deep(.el-table .el-button--small) {
  color: inherit;
}
/* 列表中重要性、状态等标签：背景透明，边框主题色，文字反色 */
:deep(.el-tag),
:deep(.el-tag.el-tag--primary),
:deep(.el-tag.el-tag--info),
:deep(.el-tag.el-tag--success),
:deep(.el-tag.el-tag--warning) {
  background-color: transparent !important;
  background: transparent !important;
  border: 1px solid var(--app-border-color, #dcdfe6) !important;
  border-color: var(--app-border-color, #dcdfe6) !important;
  color: var(--app-text-color, #1a1a1a) !important;
}
:deep(.sortable-ghost) {
  opacity: 0.5;
  background: transparent;
}
:deep(.sortable-ghost td) {
  background: transparent !important;
}
</style>
