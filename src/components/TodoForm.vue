<template>
  <div class="todo-form">
    <el-autocomplete
      v-model="content"
      :placeholder="$t('form.placeholder')"
      clearable
      maxlength="500"
      show-word-limit
      :fetch-suggestions="fetchContentSuggestions"
      style="flex: 1; min-width: 160px"
      @keyup.enter="handleSubmit"
    >
      <template #default="{ item }">
        <span v-for="(part, i) in getHighlightParts(item, content)" :key="i" :class="{ 'suggestion-highlight': part.match }">{{ part.text }}</span>
      </template>
    </el-autocomplete>
    <el-select v-model="importance" :placeholder="$t('form.importance')" clearable style="width: 100px">
      <el-option :label="$t('form.importanceNormal')" value="normal" />
      <el-option :label="$t('form.importanceImportant')" value="important" />
      <el-option :label="$t('form.importanceUrgent')" value="urgent" />
    </el-select>
    <el-select
      v-model="group_id"
      :placeholder="$t('form.groupPlaceholder')"
      clearable
      filterable
      allow-create
      default-first-option
      style="width: 140px"
      @change="onGroupChange"
    >
      <el-option
        v-for="g in props.groups"
        :key="g.id"
        :label="g.name"
        :value="g.id"
      />
    </el-select>
    <el-button type="primary" :loading="submitting" @click="handleSubmit">
      {{ $t('form.add') }}
    </el-button>
  </div>
  <div class="todo-form-row2">
    <el-date-picker
      v-model="start_time"
      type="datetime"
      :placeholder="$t('form.startPlaceholder')"
      value-format="YYYY-MM-DDTHH:mm:ss"
      style="width: 200px"
    />
    <el-date-picker
      v-model="end_time"
      type="datetime"
      :placeholder="$t('form.endPlaceholder')"
      value-format="YYYY-MM-DDTHH:mm:ss"
      style="width: 200px"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { getContentSuggestions, type CreateTodoParams, type GroupItem, type Importance } from "@/utils/request";

const props = defineProps<{
  groups: GroupItem[];
  /** 父组件新建分组后传入新分组 id，表单会自动选中该分组 */
  newlyCreatedGroupId?: string;
  /** 当前选中的分组 id，新建任务时默认使用该分组 */
  defaultGroupId?: string;
}>();

const emit = defineEmits<{
  (e: "submit", params: CreateTodoParams): void;
  (e: "createGroup", name: string): void;
}>();
const content = ref("");
const submitting = ref(false);
const importance = ref<Importance>("normal");
const group_id = ref<string | undefined>(undefined);
const start_time = ref<string | undefined>(undefined);
const end_time = ref<string | undefined>(undefined);

watch(
  () => props.newlyCreatedGroupId,
  (id) => {
    if (id) group_id.value = id;
  }
);

watch(
  () => props.defaultGroupId,
  (id) => {
    group_id.value = id;
  },
  { immediate: true }
);

function onGroupChange(value: string | undefined) {
  if (!value) return;
  const isExistingId = props.groups.some((g) => g.id === value);
  if (!isExistingId) {
    emit("createGroup", value.trim());
  }
}

function fetchContentSuggestions(
  queryString: string,
  callback: (suggestions: string[]) => void
) {
  if (!queryString.trim()) {
    callback([]);
    return;
  }
  getContentSuggestions(queryString).then(callback);
}

/** 将联想项按输入内容拆成片段，匹配部分标蓝 */
function getHighlightParts(item: string, query: string): { text: string; match: boolean }[] {
  if (!query || !query.trim()) {
    return [{ text: item, match: false }];
  }
  const q = query.trim().toLowerCase();
  const lower = item.toLowerCase();
  const idx = lower.indexOf(q);
  if (idx === -1) {
    return [{ text: item, match: false }];
  }
  const parts: { text: string; match: boolean }[] = [];
  if (idx > 0) {
    parts.push({ text: item.slice(0, idx), match: false });
  }
  parts.push({ text: item.slice(idx, idx + q.length), match: true });
  if (idx + q.length < item.length) {
    parts.push({ text: item.slice(idx + q.length), match: false });
  }
  return parts;
}

function handleSubmit() {
  const text = content.value.trim();
  if (!text) {
    ElMessage.warning("请输入任务内容");
    return;
  }
  submitting.value = true;
  emit("submit", {
    content: text,
    importance: importance.value,
    group_id: group_id.value,
    start_time: start_time.value,
    end_time: end_time.value,
  });
  content.value = "";
  submitting.value = false;
}
</script>

<style scoped>
.todo-form,
.todo-form-row2 {
  color: var(--app-text-color, #1a1a1a);
}
.todo-form {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.todo-form-row2 {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}
.suggestion-highlight {
  color: var(--el-color-primary, #409eff);
  font-weight: 500;
}
/* 表单项背景透明、边框主题色、文字反色（兜底覆盖） */
:deep(.el-input),
:deep(.el-select .el-input),
:deep(.el-date-editor .el-input) {
  --el-fill-color-blank: transparent !important;
  --el-input-bg-color: transparent !important;
}
:deep(.el-input__wrapper),
:deep(.el-select .el-input__wrapper),
:deep(.el-date-editor .el-input__wrapper) {
  background: transparent !important;
  background-color: transparent !important;
  box-shadow: 0 0 0 1px var(--app-border-color, #dcdfe6) inset !important;
}
:deep(.el-input__wrapper:hover),
:deep(.el-input__wrapper.is-focus),
:deep(.el-select .el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--app-border-color, #dcdfe6) inset !important;
}
:deep(.el-input__inner),
:deep(.el-select .el-input__inner),
:deep(.el-date-editor .el-input__inner),
:deep(.el-date-editor .el-range-separator) {
  color: var(--app-text-color, #1a1a1a) !important;
}
/* 占位文字带透明度 */
:deep(.el-input__inner::placeholder),
:deep(.el-input__wrapper .el-input__inner::placeholder) {
  color: var(--app-text-color, #1a1a1a) !important;
  opacity: 0.65;
}
/* 重要性、分组 el-select 使用 .el-select__wrapper 边框 */
:deep(.el-select__wrapper) {
  background: transparent !important;
  background-color: transparent !important;
  box-shadow: 0 0 0 1px var(--app-border-color, #dcdfe6) inset !important;
}
:deep(.el-select__wrapper:hover),
:deep(.el-select__wrapper.is-focused) {
  box-shadow: 0 0 0 1px var(--app-border-color, #dcdfe6) inset !important;
}
:deep(.el-select__placeholder),
:deep(.el-select__placeholder.is-transparent),
:deep(.el-select__selected-item),
:deep(.el-select__input) {
  color: var(--app-text-color, #1a1a1a) !important;
}
:deep(.el-select__placeholder.is-transparent) {
  opacity: 0.65;
}
:deep(.el-input .el-input__count),
:deep(.el-input__suffix .el-icon),
:deep(.el-select .el-select__suffix .el-icon),
:deep(.el-select .el-select__caret) {
  color: var(--app-text-color, #1a1a1a) !important;
  opacity: 0.8;
}
</style>
