#[allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tauri::Emitter;
use uuid::Uuid;

use crate::{AppState, Todo, DEFAULT_PORT};

#[derive(Deserialize)]
struct CreateTodoBody {
    content: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    start_time: Option<String>,
    #[serde(default)]
    end_time: Option<String>,
    #[serde(default)]
    importance: Option<String>,
    #[serde(default)]
    group_id: Option<String>,
}

#[derive(Deserialize)]
struct UpdateTodoBody {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub importance: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
}

#[derive(Deserialize)]
struct ReorderBody {
    ordered_ids: Vec<String>,
}

#[derive(Deserialize)]
struct ListTodosQuery {
    #[serde(default)]
    sort: String, // comprehensive | importance | deadline
}

#[derive(Deserialize)]
struct ContentSuggestionsQuery {
    #[serde(default)]
    q: String,
}

fn norm_importance(s: Option<&String>) -> &'static str {
    match s.map(|s| s.as_str()) {
        Some("urgent") | Some("紧急") => "urgent",
        Some("important") | Some("重要") => "important",
        _ => "normal",
    }
}

fn todo_status(s: &str) -> &'static str {
    if s == "completed" {
        "completed"
    } else {
        "pending"
    }
}

type TodoRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    i64,
);

async fn fetch_one_todo(pool: &sqlx::SqlitePool, id: &str) -> Option<Todo> {
    let row: Option<TodoRow> = sqlx::query_as(
        "SELECT id, content, status, reminder_time, start_time, end_time, importance, group_id, sort_order FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.map(|(id, content, status, reminder_time, start_time, end_time, importance, group_id, sort_order)| Todo {
        id,
        content,
        status,
        reminder_time,
        start_time,
        end_time,
        importance,
        group_id,
        sort_order,
    })
}

/// 返回手机扫码地址（局域网 IP），前端直接 fetch 即可，不依赖 Tauri 事件/invoke
async fn mobile_url() -> impl IntoResponse {
    let ip = local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let url = format!("http://{}:{}/mobile", ip, DEFAULT_PORT);
    Json(serde_json::json!({ "url": url }))
}

fn order_by_clause(sort: &str) -> &'static str {
    match sort {
        "importance" => "ORDER BY CASE importance WHEN 'urgent' THEN 0 WHEN 'important' THEN 1 ELSE 2 END ASC, sort_order ASC, id ASC",
        "deadline" => "ORDER BY CASE WHEN end_time IS NULL THEN 1 ELSE 0 END ASC, end_time ASC, sort_order ASC, id ASC",
        _ => "ORDER BY CASE WHEN end_time IS NULL THEN 1 ELSE 0 END ASC, end_time ASC, sort_order ASC, CASE importance WHEN 'urgent' THEN 0 WHEN 'important' THEN 1 ELSE 2 END ASC, id ASC",
    }
}

async fn content_suggestions(
    State(state): State<AppState>,
    Query(q): Query<ContentSuggestionsQuery>,
) -> impl IntoResponse {
    let q = q.q.trim();
    if q.is_empty() {
        return Json(Vec::<String>::new()).into_response();
    }
    let pattern = format!("%{}%", q.replace('%', "\\%").replace('_', "\\_"));
    let rows = match sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT content FROM todos WHERE content LIKE ?1 ESCAPE '\\' ORDER BY content LIMIT 20",
    )
    .bind(&pattern)
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("content_suggestions: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<String>::new())).into_response();
        }
    };
    Json(rows).into_response()
}

async fn list_todos(
    State(state): State<AppState>,
    Query(q): Query<ListTodosQuery>,
) -> impl IntoResponse {
    let order = order_by_clause(q.sort.as_str());
    let sql = format!(
        "SELECT id, content, status, reminder_time, start_time, end_time, importance, group_id, sort_order FROM todos {}",
        order
    );
    let rows = match sqlx::query_as::<_, (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        i64,
    )>(&sql)
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_todos: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "获取列表失败"})),
            )
                .into_response();
        }
    };
    let list: Vec<serde_json::Value> = rows
        .into_iter()
        .map(
            |(id, content, status, reminder_time, start_time, end_time, importance, group_id, sort_order)| {
                serde_json::json!({
                    "id": id,
                    "content": content,
                    "status": status,
                    "reminder_time": reminder_time,
                    "start_time": start_time,
                    "end_time": end_time,
                    "importance": importance,
                    "group_id": group_id,
                    "sort_order": sort_order
                })
            },
        )
        .collect();
    Json(list).into_response()
}

async fn create_todo(
    State(state): State<AppState>,
    Json(body): Json<CreateTodoBody>,
) -> impl IntoResponse {
    let content = body.content.trim();
    if content.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "内容不能为空"})),
        )
            .into_response();
    }
    let id = Uuid::new_v4().to_string();
    let status = todo_status(&body.status).to_string();
    let importance = norm_importance(body.importance.as_ref()).to_string();
    let sort_order: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM todos")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let todo = Todo {
        id: id.clone(),
        content: content.to_string(),
        status: status.clone(),
        reminder_time: None,
        start_time: body.start_time.clone(),
        end_time: body.end_time.clone(),
        importance: importance.clone(),
        group_id: body.group_id.clone(),
        sort_order,
    };
    match sqlx::query(
        "INSERT INTO todos (id, content, status, reminder_time, start_time, end_time, importance, group_id, sort_order) VALUES (?, ?, ?, NULL, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&todo.content)
    .bind(&status)
    .bind(&body.start_time)
    .bind(&body.end_time)
    .bind(&importance)
    .bind(&body.group_id)
    .bind(sort_order)
    .execute(&state.pool)
    .await
    {
        Ok(_) => (StatusCode::CREATED, Json(todo)).into_response(),
        Err(e) => {
            eprintln!("create_todo: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "新增失败"})),
            )
                .into_response()
        }
    }
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTodoBody>,
) -> impl IntoResponse {
    let mut updates: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref c) = body.content {
        updates.push("content = ?".to_string());
        binds.push(c.clone());
    }
    if let Some(ref s) = body.status {
        updates.push("status = ?".to_string());
        binds.push(todo_status(s).to_string());
    }
    if body.start_time.is_some() {
        updates.push("start_time = ?".to_string());
        binds.push(body.start_time.as_deref().unwrap_or("").to_string());
    }
    if body.end_time.is_some() {
        updates.push("end_time = ?".to_string());
        binds.push(body.end_time.as_deref().unwrap_or("").to_string());
    }
    if let Some(ref imp) = body.importance {
        updates.push("importance = ?".to_string());
        binds.push(norm_importance(Some(imp)).to_string());
    }
    if body.group_id.is_some() {
        updates.push("group_id = ?".to_string());
        binds.push(body.group_id.as_deref().unwrap_or("").to_string());
    }

    if updates.is_empty() {
        let row = fetch_one_todo(&state.pool, &id).await;
        return match row {
            Some(t) => (StatusCode::OK, Json(t)).into_response(),
            None => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "任务不存在"})),
            )
                .into_response(),
        };
    }

    let sql = format!("UPDATE todos SET {} WHERE id = ?", updates.join(", "));
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(&id);
    let result = q.execute(&state.pool).await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            let row = fetch_one_todo(&state.pool, &id).await;
            let out = match row {
                Some(t) => t,
                None => Todo {
                    id: id.clone(),
                    content: String::new(),
                    status: "pending".to_string(),
                    reminder_time: None,
                    start_time: None,
                    end_time: None,
                    importance: "normal".to_string(),
                    group_id: None,
                    sort_order: 0,
                },
            };
            (StatusCode::OK, Json(out)).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "任务不存在"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("update_todo: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "更新失败"})),
            )
                .into_response()
        }
    }
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "任务不存在"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("delete_todo: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "删除失败"})),
            )
                .into_response()
        }
    }
}

async fn reorder_todos(
    State(state): State<AppState>,
    Json(body): Json<ReorderBody>,
) -> impl IntoResponse {
    for (idx, id) in body.ordered_ids.iter().enumerate() {
        let _ = sqlx::query("UPDATE todos SET sort_order = ? WHERE id = ?")
            .bind(idx as i64)
            .bind(id)
            .execute(&state.pool)
            .await;
    }
    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

async fn list_groups(State(state): State<AppState>) -> impl IntoResponse {
    let rows = match sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM groups ORDER BY COALESCE(sort_order, 999999), id",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("list_groups: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "获取分组失败"})),
            )
                .into_response();
        }
    };
    let list: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, name)| serde_json::json!({ "id": id, "name": name }))
        .collect();
    Json(list).into_response()
}

#[derive(Deserialize)]
struct CreateGroupBody {
    name: String,
}

async fn create_group(
    State(state): State<AppState>,
    Json(body): Json<CreateGroupBody>,
) -> impl IntoResponse {
    let name = body.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "分组名称不能为空"})),
        )
            .into_response();
    }
    let next_order: (Option<i64>,) = sqlx::query_as("SELECT MAX(sort_order) FROM groups")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((Some(0),));
    let sort_order = next_order.0.unwrap_or(-1) + 1;
    let id = Uuid::new_v4().to_string();
    match sqlx::query("INSERT INTO groups (id, name, sort_order) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(sort_order)
        .execute(&state.pool)
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "id": id, "name": name }))).into_response(),
        Err(e) => {
            eprintln!("create_group: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "新增分组失败"})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct UpdateGroupBody {
    name: String,
}

async fn update_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateGroupBody>,
) -> impl IntoResponse {
    let name = body.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "分组名称不能为空"})),
        )
            .into_response();
    }
    let result = sqlx::query("UPDATE groups SET name = ? WHERE id = ?")
        .bind(name)
        .bind(&id)
        .execute(&state.pool)
        .await;
    match result {
        Ok(r) if r.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({ "id": id, "name": name }))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "分组不存在"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("update_group: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "更新分组失败"})),
            )
                .into_response()
        }
    }
}

async fn delete_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE todos SET group_id = NULL WHERE group_id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await;
    let result = sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await;
    match result {
        Ok(r) if r.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "分组不存在"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("delete_group: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "删除分组失败"})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct ReorderGroupsBody {
    ordered_ids: Vec<String>,
}

async fn reorder_groups(
    State(state): State<AppState>,
    Json(body): Json<ReorderGroupsBody>,
) -> impl IntoResponse {
    if body.ordered_ids.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response();
    }
    for (i, id) in body.ordered_ids.iter().enumerate() {
        let sort_order = i as i64;
        if let Err(e) = sqlx::query("UPDATE groups SET sort_order = ? WHERE id = ?")
            .bind(sort_order)
            .bind(id)
            .execute(&state.pool)
            .await
        {
            eprintln!("reorder_groups: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "分组排序失败"})),
            )
                .into_response();
        }
    }
    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

fn static_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    PathBuf::from(manifest).join("../dist")
}

async fn serve_index() -> Response {
    let path = static_dir().join("index.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(html) => (StatusCode::OK, [("Content-Type", "text/html")], html).into_response(),
        Err(_) => {
            let fallback = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>手机端</title></head>
<body style="font-family:sans-serif;padding:20px;max-width:400px;margin:0 auto;">
  <h2>开发模式</h2>
  <p>请先在电脑上执行：<code>pnpm run build</code></p>
  <p>构建完成后刷新本页即可使用手机端。</p>
</body>
</html>"#;
            (StatusCode::OK, [("Content-Type", "text/html; charset=utf-8")], fallback.to_string()).into_response()
        }
    }
}

pub fn run_axum(state: AppState, _stop: Arc<AtomicBool>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/mobile-url", get(mobile_url))
        .route("/todo/reorder", post(reorder_todos))
        .route("/todo/content-suggestions", get(content_suggestions))
        .route("/todo", get(list_todos).post(create_todo))
        .route("/todo/:id", patch(update_todo).delete(delete_todo))
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/reorder", post(reorder_groups))
        .route("/groups/:id", patch(update_group).delete(delete_group))
        .with_state(state.clone());

    let static_path = static_dir();
    let serve_dir = ServeDir::new(static_path);
    let app = Router::new()
        .nest("/api", api)
        .route("/mobile", get(serve_index))
        .route("/mobile/*_", get(serve_index))
        .fallback_service(serve_dir)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    let app_handle = state.app_handle.clone();

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _ = rt.block_on(async {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Axum bind failed: {}", e);
                return;
            }
        };
        let mobile_url = {
            let ip = local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
            format!("http://{}:{}/mobile", ip, DEFAULT_PORT)
        };
        if let Err(e) = app_handle.emit("web-server-url", &mobile_url) {
            eprintln!("emit web-server-url failed: {}", e);
        }
        let _ = axum::serve(listener, app).await;
    });
}
