/**
 * 前端请求封装：统一请求头、响应拦截、错误提示
 * 桌面端请求局域网 Axum 服务；移动端使用当前 origin 作为 baseURL
 */

import { ElMessage } from "element-plus";

export function getBaseUrl(): string {
  if (typeof window === "undefined") return "";
  const origin = window.location.origin;
  if (origin.includes(":5173") || (window as Window & { __TAURI__?: unknown }).__TAURI__) {
    return "http://127.0.0.1:8080";
  }
  return origin;
}

export async function getMobileUrl(): Promise<string> {
  const base = getBaseUrl();
  if (!base) return "";
  const res = await fetch(`${base}/api/mobile-url`);
  if (!res.ok) return "";
  const data = (await res.json()) as { url?: string };
  return data?.url ?? "";
}

export type Importance = "normal" | "important" | "urgent";
export type SortRule = "comprehensive" | "importance" | "deadline";

export interface GroupItem {
  id: string;
  name: string;
}

export interface TodoItem {
  id: string;
  content: string;
  status: "pending" | "completed";
  reminder_time?: string;
  start_time?: string;
  end_time?: string;
  importance: Importance;
  group_id?: string;
  sort_order: number;
}

export interface CreateTodoParams {
  content: string;
  status?: "pending" | "completed";
  start_time?: string;
  end_time?: string;
  importance?: Importance;
  group_id?: string;
}

export async function getTodoList(sort: SortRule = "comprehensive"): Promise<TodoItem[]> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/todo?sort=${encodeURIComponent(sort)}`, { method: "GET" });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "获取任务列表失败");
    throw new Error(text || "获取任务列表失败");
  }
  return res.json();
}

/** 根据输入获取任务内容联想（历史数据） */
export async function getContentSuggestions(q: string): Promise<string[]> {
  const base = getBaseUrl();
  if (!base) return [];
  const res = await fetch(
    `${base}/api/todo/content-suggestions?q=${encodeURIComponent(q.trim())}`,
    { method: "GET" }
  );
  if (!res.ok) return [];
  return res.json();
}

export async function createTodo(params: CreateTodoParams): Promise<TodoItem> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/todo`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      content: params.content,
      status: params.status ?? "pending",
      start_time: params.start_time ?? null,
      end_time: params.end_time ?? null,
      importance: params.importance ?? "normal",
      group_id: params.group_id ?? null,
    }),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "新增任务失败");
    throw new Error(text || "新增任务失败");
  }
  return res.json();
}

export type UpdateTodoParams = Partial<{
  content: string;
  status: "pending" | "completed";
  start_time: string;
  end_time: string;
  importance: Importance;
  group_id: string;
}>;

export async function updateTodo(id: string, params: UpdateTodoParams): Promise<TodoItem> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/todo/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(params),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "更新失败");
    throw new Error(text || "更新失败");
  }
  return res.json();
}

export async function updateTodoStatus(
  id: string,
  status: "pending" | "completed"
): Promise<TodoItem> {
  return updateTodo(id, { status });
}

export async function deleteTodo(id: string): Promise<void> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/todo/${id}`, { method: "DELETE" });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "删除任务失败");
    throw new Error(text || "删除任务失败");
  }
}

export async function reorderTodos(orderedIds: string[]): Promise<void> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/todo/reorder`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ ordered_ids: orderedIds }),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "排序失败");
    throw new Error(text || "排序失败");
  }
}

export async function getGroups(): Promise<GroupItem[]> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/groups`, { method: "GET" });
  if (!res.ok) return [];
  return res.json();
}

export async function createGroup(name: string): Promise<GroupItem> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/groups`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "新增分组失败");
    throw new Error(text || "新增分组失败");
  }
  return res.json();
}

export async function updateGroup(id: string, name: string): Promise<GroupItem> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/groups/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "更新分组失败");
    throw new Error(text || "更新分组失败");
  }
  return res.json();
}

export async function deleteGroup(id: string): Promise<void> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/groups/${id}`, { method: "DELETE" });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "删除分组失败");
    throw new Error(text || "删除分组失败");
  }
}

export async function reorderGroups(orderedIds: string[]): Promise<void> {
  const base = getBaseUrl();
  const res = await fetch(`${base}/api/groups/reorder`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ ordered_ids: orderedIds }),
  });
  if (!res.ok) {
    const text = await res.text();
    ElMessage.error(text || "分组排序失败");
    throw new Error(text || "分组排序失败");
  }
}
