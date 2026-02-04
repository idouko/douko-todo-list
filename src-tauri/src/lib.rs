#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod server;

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri::webview::WebviewWindowBuilder;
use tauri::WebviewUrl;

/// 主窗体首次启动时的宽度（逻辑像素）
const MAIN_WINDOW_INITIAL_WIDTH: f64 = 375.0;
/// 主窗体首次启动时高度为屏幕高度的比例
const MAIN_WINDOW_INITIAL_HEIGHT_RATIO: f64 = 0.75;
const WINDOW_STATE_FILENAME: &str = "window-state.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct MainWindowState {
    width: f64,
    height: f64,
}

fn main_window_state_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(WINDOW_STATE_FILENAME)
}

fn load_main_window_state(app: &AppHandle) -> Option<MainWindowState> {
    let path = main_window_state_path(app);
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_main_window_state(app: &AppHandle, state: &MainWindowState) -> Result<(), String> {
    let path = main_window_state_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

pub const DEFAULT_PORT: u16 = 8080;

/// 供前端获取应用版本号（关于我们等页面使用）
#[tauri::command]
fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 主窗背景透明度刷新：微移窗口再还原，触发 DWM 重新合成（解决启动后透明度不生效、需拖到别屏再拖回才正常的问题）
#[tauri::command]
fn refresh_main_window_transparency(app: AppHandle) -> Result<(), String> {
    let Some(main_win) = app.get_webview_window("main") else { return Ok(()) };
    let Ok(pos) = main_win.outer_position() else { return Ok(()) };
    let orig = tauri::PhysicalPosition::new(pos.x, pos.y);
    let _ = main_win.set_position(tauri::PhysicalPosition::new(pos.x + 1, pos.y));
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let value = app_clone.clone();
        let _ = app_clone.run_on_main_thread(move || {
            if let Some(w) = value.get_webview_window("main") {
                let _ = w.set_position(orig);
            }
        });
    });
    Ok(())
}

/// 供前端在未收到 web-server-url 事件时主动获取扫码地址（服务就绪后调用）
#[tauri::command]
fn get_web_server_url() -> String {
    let ip = local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    format!("http://{}:{}/mobile", ip, DEFAULT_PORT)
}

/// 背景图缓存文件名（保存在应用数据目录）
const BACKGROUND_IMAGE_FILENAME: &str = "background_image";

/// 将用户选择的背景图复制到应用数据目录，返回新路径；下次启动直接读该路径即可生效。
#[tauri::command]
fn save_background_image(app: AppHandle, source_path: String) -> Result<String, String> {
    use std::path::Path;
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    let ext = Path::new(&source_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let dest_name = format!("{}.{}", BACKGROUND_IMAGE_FILENAME, ext);
    let dest = app_data.join(&dest_name);
    std::fs::copy(&source_path, &dest).map_err(|e| e.to_string())?;
    dest.into_os_string()
        .into_string()
        .map_err(|_| "路径编码错误".to_string())
}

/// 清除应用数据目录中的背景图缓存（用户点击「清除背景图」时调用）。
#[tauri::command]
fn clear_background_image(app: AppHandle) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    for ext in ["png", "jpg", "jpeg", "gif", "webp", "bmp"] {
        let p = app_data.join(format!("{}.{}", BACKGROUND_IMAGE_FILENAME, ext));
        let _ = std::fs::remove_file(&p);
    }
    Ok(())
}

/// 存储信息：供设置「数据与安全」面板展示
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageInfo {
    /// SQLite 数据库文件路径
    sqlite_path: String,
    /// SQLite 文件大小（字节）
    sqlite_size_bytes: u64,
    /// 应用数据目录总大小（字节，含数据库、配置、缓存等）
    app_data_size_bytes: u64,
}

fn file_size(path: &std::path::Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                total += dir_size(&entry.path());
            } else {
                total += meta.len();
            }
        }
    }
    total
}

/// 获取存储信息：SQLite 路径与大小、应用数据目录总大小。
#[tauri::command]
fn get_storage_info(app: AppHandle) -> Result<StorageInfo, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let sqlite_path = app_data.join("todos.db");
    let sqlite_size_bytes = file_size(&sqlite_path);
    let app_data_size_bytes = dir_size(&app_data);
    let sqlite_path_str = sqlite_path
        .into_os_string()
        .into_string()
        .map_err(|_| "路径编码错误")?;
    Ok(StorageInfo {
        sqlite_path: sqlite_path_str,
        sqlite_size_bytes,
        app_data_size_bytes,
    })
}

/// 在系统文件管理器中显示并选中指定路径（文件或目录）。
#[tauri::command]
fn open_path_in_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("路径不存在".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        let path_abs = std::fs::canonicalize(p).map_err(|e| e.to_string())?;
        let path_str = path_abs
            .into_os_string()
            .into_string()
            .map_err(|_| "路径编码错误")?;
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path_str))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        let parent = p
            .parent()
            .ok_or("无法获取父目录")?;
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 与 Vue Router History 模式一致，使用路径 /qrcode-window（不用 hash）
const QRCODE_WINDOW_PATH: &str = "/qrcode-window";
/// 设置窗口路径
const SETTINGS_WINDOW_PATH: &str = "/settings-window";

/// 根据主窗口位置与尺寸，计算子窗口居中时的逻辑坐标 (x, y)。
/// 用于二维码、设置等窗口在创建时直接指定位置，避免先出现再移动的闪烁。
fn center_position_on_main(
    app: &AppHandle,
    window_width: f64,
    window_height: f64,
) -> Option<(f64, f64)> {
    let main_win = app.get_webview_window("main")?;
    let main_pos = main_win.outer_position().ok()?;
    let main_size = main_win.outer_size().ok()?;
    let scale = main_win.scale_factor().unwrap_or(1.0);
    let center_x = (main_pos.x as f64 + main_size.width as f64 / 2.0) / scale;
    let center_y = (main_pos.y as f64 + main_size.height as f64 / 2.0) / scale;
    let x = center_x - window_width / 2.0;
    let y = center_y - window_height / 2.0;
    Some((x, y))
}

/// 二维码窗口逻辑尺寸（与 inner_size 一致）
const QRCODE_WINDOW_WIDTH: f64 = 360.0;
const QRCODE_WINDOW_HEIGHT: f64 = 420.0;

/// 打开二维码独立窗口。若已存在则复用并聚焦，不销毁重建；否则创建并居中于主窗。
#[tauri::command]
async fn open_qrcode_window(app: AppHandle, url: String) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("qrcode") {
        let _ = w.set_focus();
        return Ok(());
    }

    let resolved = resolve_qrcode_window_url(&app, url);
    let parsed = url::Url::parse(&resolved).map_err(|e| e.to_string())?;
    let webview_url = WebviewUrl::External(parsed);

    let mut builder = WebviewWindowBuilder::new(&app, "qrcode", webview_url)
        .title("扫码用手机编辑")
        .inner_size(QRCODE_WINDOW_WIDTH, QRCODE_WINDOW_HEIGHT)
        .resizable(false)
        .always_on_top(true);

    if let Some((x, y)) = center_position_on_main(&app, QRCODE_WINDOW_WIDTH, QRCODE_WINDOW_HEIGHT)
    {
        builder = builder.position(x, y);
    }

    let window = builder.build().map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

fn resolve_qrcode_window_url(app: &AppHandle, frontend_url: String) -> String {
    if let Ok(parsed) = url::Url::parse(&frontend_url) {
        let is_bad_localhost = parsed.host_str() == Some("localhost")
            && parsed.port().map_or(true, |p| p == 80);
        if is_bad_localhost {
            return backend_qrcode_url(app);
        }
        if parsed.scheme() == "tauri" || parsed.scheme().starts_with("asset") {
            return backend_qrcode_url(app);
        }
    }
    if frontend_url.is_empty() {
        return backend_qrcode_url(app);
    }
    let trimmed = frontend_url
        .trim_end_matches('/')
        .trim_end_matches("#/qrcode-window");
    let base = trimmed.trim_end_matches('/');
    if base.is_empty() {
        return backend_qrcode_url(app);
    }
    if base.ends_with(QRCODE_WINDOW_PATH) {
        return base.to_string();
    }
    format!("{}{}", base, QRCODE_WINDOW_PATH)
}

fn backend_qrcode_url(app: &AppHandle) -> String {
    if cfg!(debug_assertions) {
        if let Some(ref u) = app.config().build.dev_url {
            let base = u.as_str().trim_end_matches('/');
            return format!("{}{}", base, QRCODE_WINDOW_PATH);
        }
    }
    format!("tauri://localhost{}", QRCODE_WINDOW_PATH)
}

/// 设置窗口逻辑尺寸
const SETTINGS_WINDOW_WIDTH: f64 = 640.0;
const SETTINGS_WINDOW_HEIGHT: f64 = 520.0;

/// 打开设置独立窗口。若已存在则复用并聚焦，不销毁重建；否则创建并居中于主窗。
#[tauri::command]
async fn open_settings_window(app: AppHandle, url: String) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_focus();
        return Ok(());
    }

    let resolved = resolve_settings_window_url(&app, url);
    let parsed = url::Url::parse(&resolved).map_err(|e| e.to_string())?;
    let webview_url = WebviewUrl::External(parsed);

    let mut builder = WebviewWindowBuilder::new(&app, "settings", webview_url)
        .title("设置")
        .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
        .resizable(false)
        .always_on_top(true);

    if let Some((x, y)) =
        center_position_on_main(&app, SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
    {
        builder = builder.position(x, y);
    }

    let window = builder.build().map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

fn resolve_settings_window_url(app: &AppHandle, frontend_url: String) -> String {
    if let Ok(parsed) = url::Url::parse(&frontend_url) {
        if parsed.host_str() == Some("localhost") && parsed.port().map_or(true, |p| p == 80) {
            return backend_settings_url(app);
        }
        if parsed.scheme() == "tauri" || parsed.scheme().starts_with("asset") {
            return backend_settings_url(app);
        }
    }
    if frontend_url.is_empty() {
        return backend_settings_url(app);
    }
    let trimmed = frontend_url
        .trim_end_matches('/')
        .trim_end_matches("#/settings-window");
    let base = trimmed.trim_end_matches('/');
    if base.is_empty() {
        return backend_settings_url(app);
    }
    if base.ends_with(SETTINGS_WINDOW_PATH) {
        return base.to_string();
    }
    format!("{}{}", base, SETTINGS_WINDOW_PATH)
}

fn backend_settings_url(app: &AppHandle) -> String {
    if cfg!(debug_assertions) {
        if let Some(ref u) = app.config().build.dev_url {
            let base = u.as_str().trim_end_matches('/');
            return format!("{}{}", base, SETTINGS_WINDOW_PATH);
        }
    }
    format!("tauri://localhost{}", SETTINGS_WINDOW_PATH)
}

/// 侧栏窗口逻辑宽度：收起 80px，展开由前端传入。与主窗左边缘紧密贴合（主窗口 x = 侧栏右边缘）
const SIDEBAR_WIDTH_COLLAPSED: u32 = 80;
const SIDEBAR_WIDTH_EXPANDED_DEFAULT: u32 = 200;
static SIDEBAR_WIDTH_CURRENT: AtomicU32 = AtomicU32::new(SIDEBAR_WIDTH_EXPANDED_DEFAULT);
/// 分组窗体（副窗体）对主窗体的吸附位置：0 = 吸附在左侧，1 = 吸附在右侧。
/// 所有副窗体位置由 sync_sidebar_to_main 根据此值 + 主窗位置/尺寸计算，实现“吸附位置”方案。
static SIDEBAR_POSITION: AtomicU8 = AtomicU8::new(0);
/// Moved/Resized 防抖：仅在手松/拖拽结束后做一次“置顶”同步，避免每帧 set_focus 导致卡顿。
static MOVE_DEBOUNCE: AtomicU64 = AtomicU64::new(0);
static RESIZE_DEBOUNCE: AtomicU64 = AtomicU64::new(0);
/// 节流：上次执行 sync 的时间（毫秒），避免 Resized/Moved 每帧都同步导致卡顿。
static LAST_SIDEBAR_SYNC_MS: AtomicU64 = AtomicU64::new(0);

fn sidebar_width() -> f64 {
    SIDEBAR_WIDTH_CURRENT.load(Ordering::Relaxed) as f64
}

fn sidebar_on_right() -> bool {
    SIDEBAR_POSITION.load(Ordering::Relaxed) == 1
}

/// 前端调用：设置侧栏宽度（收起 80，展开为测量值），并立即同步位置与尺寸，保证与主窗无间隙
#[tauri::command]
fn set_sidebar_width(app: AppHandle, width: u32) -> Result<(), String> {
    let w = width.clamp(SIDEBAR_WIDTH_COLLAPSED, 150);
    SIDEBAR_WIDTH_CURRENT.store(w, Ordering::Relaxed);
    sync_sidebar_to_main(&app, true);
    Ok(())
}

/// 设置分组窗体对主窗体的吸附位置（"left" | "right"），在主线程执行一次 sync。
#[tauri::command]
fn set_sidebar_position(app: AppHandle, position: String) -> Result<(), String> {
    let pos_trimmed = position.trim();
    let on_right = pos_trimmed.eq_ignore_ascii_case("right");
    SIDEBAR_POSITION.store(if on_right { 1 } else { 0 }, Ordering::Relaxed);
    println!("Setting sidebar position to: {}", if on_right { "right" } else { "left" });
    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        // 确保 sidebar 窗口存在，如果不存在则创建
        if app_clone.get_webview_window("sidebar").is_none() {
            println!("Sidebar window does not exist, creating...");
            if let Err(e) = create_sidebar_window(&app_clone) {
                eprintln!("Failed to create sidebar window: {}", e);
                return;
            }
            println!("Sidebar window created successfully");
        } else {
            println!("Sidebar window already exists");
        }
        sync_sidebar_to_main(&app_clone, true);
        println!("Sidebar position synced");
    });
    Ok(())
}

fn sidebar_window_url(app: &AppHandle) -> String {
    if cfg!(debug_assertions) {
        if let Some(ref u) = app.config().build.dev_url {
            let base = u.as_str().trim_end_matches('/');
            return format!("{}?window=sidebar", base);
        }
    }
    "tauri://localhost/?window=sidebar".to_string()
}


const APP_SETTINGS_FILENAME: &str = "app-settings.json";

/// 启动时从设置文件读取吸附位置（左/右），使分组窗体在首次显示前就按上次设置吸附在正确侧。
fn load_sidebar_position_from_store(app: &AppHandle) {
    let Ok(app_data) = app.path().app_data_dir() else { return };
    let path = app_data.join(APP_SETTINGS_FILENAME);
    let Ok(data) = std::fs::read_to_string(&path) else { return };
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&data) else { return };
    let Some(pos) = root.get("sidebarPosition").and_then(|v| v.as_str()) else { return };
    if pos.eq_ignore_ascii_case("right") {
        SIDEBAR_POSITION.store(1, Ordering::Relaxed);
    } else {
        SIDEBAR_POSITION.store(0, Ordering::Relaxed);
    }
}

/// 主窗体首次启动：宽 375px，高为屏幕高度的 3/4；后续启动使用上次保存的尺寸。
fn apply_main_window_initial_size(app: &AppHandle) {
    let Some(main_win) = app.get_webview_window("main") else { return };

    if let Some(state) = load_main_window_state(app) {
        if state.width > 0.0 && state.height > 0.0 {
            let _ = main_win.set_size(tauri::LogicalSize::new(state.width, state.height));
            return;
        }
    }

    let scale = main_win.scale_factor().unwrap_or(1.0);
    let initial_height = if let Ok(Some(monitor)) = main_win.current_monitor() {
        let phys = monitor.size();
        (phys.height as f64 / scale) * MAIN_WINDOW_INITIAL_HEIGHT_RATIO
    } else {
        600.0 * MAIN_WINDOW_INITIAL_HEIGHT_RATIO
    };

    let _ = main_win.set_size(tauri::LogicalSize::new(
        MAIN_WINDOW_INITIAL_WIDTH,
        initial_height,
    ));
    let _ = save_main_window_state(app, &MainWindowState {
        width: MAIN_WINDOW_INITIAL_WIDTH,
        height: initial_height,
    });
}

/// 主窗体尺寸变化时保存到文件，供下次启动恢复。
fn save_main_window_size_on_resize(app: &AppHandle) {
    let Some(main_win) = app.get_webview_window("main") else { return };
    let Ok(phys) = main_win.outer_size() else { return };
    let scale = main_win.scale_factor().unwrap_or(1.0);
    let state = MainWindowState {
        width: phys.width as f64 / scale,
        height: phys.height as f64 / scale,
    };
    let _ = save_main_window_state(app, &state);
}

/// 创建侧栏窗口（先不显示，由 sync_sidebar_to_main 在位置就绪后显示）。
/// 主窗尚未 show 时 outer_position/outer_size 可能失败或为 0，用默认值仍创建侧栏，避免创建失败导致分组窗体永远不出现。
fn create_sidebar_window(app: &AppHandle) -> Result<(), String> {
    let main_win = app
        .get_webview_window("main")
        .ok_or("main window not found")?;
    let scale = main_win.scale_factor().unwrap_or(1.0);
    let default_w = (MAIN_WINDOW_INITIAL_WIDTH * scale) as u32;
    let default_h = (600.0_f64 * MAIN_WINDOW_INITIAL_HEIGHT_RATIO * scale) as u32;
    let pos = main_win
        .outer_position()
        .ok()
        .unwrap_or_else(|| tauri::PhysicalPosition::new(0, 0));
    let size = main_win
        .outer_size()
        .ok()
        .unwrap_or_else(|| tauri::PhysicalSize::new(default_w, default_h));
    let main_x = pos.x as f64 / scale;
    let main_y = pos.y as f64 / scale;
    let main_w_logical = size.width as f64 / scale;
    let main_physical_h = size.height as f64;
    let main_h_logical = main_physical_h / scale;
    let w = sidebar_width();
    let sidebar_x = if sidebar_on_right() {
        main_x + main_w_logical - 16.0
    } else {
        (main_x - w).max(0.0)
    };
    let sidebar_y = main_y.max(0.0);

    let url = sidebar_window_url(app);
    let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
    let webview_url = WebviewUrl::External(parsed);

    let builder = WebviewWindowBuilder::new(app, "sidebar", webview_url)
        .title("")
        .inner_size(w, main_h_logical.max(100.0))
        .position(sidebar_x, sidebar_y)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .visible(true)
        .always_on_top(false)
        .focused(false);
    builder.build().map_err(|e| e.to_string())?;

    if let Some(sidebar) = app.get_webview_window("sidebar") {
        let sidebar_scale = sidebar.scale_factor().unwrap_or(scale);
        let w_phys = (w * sidebar_scale).round() as u32;
        let size_physical = tauri::PhysicalSize {
            width: (w_phys + 2).max(1),
            height: size.height.max(1),
        };
        let _ = sidebar.set_size(size_physical);
    }
    Ok(())
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 节流间隔（毫秒）：Resized/Moved 期间最多每这么久同步一次位置/尺寸，减轻卡顿。
const SIDEBAR_SYNC_THROTTLE_MS: u64 = 80;

/// 按“吸附位置”方案更新副窗体位置与尺寸；bring_to_front 为 true 时再执行 set_focus 保证层级，拖动时传 false 避免卡顿。
fn sync_sidebar_to_main(app: &AppHandle, bring_to_front: bool) {
    let Some(main_win) = app.get_webview_window("main") else { return };
    let Some(sidebar) = app.get_webview_window("sidebar") else { return };
    LAST_SIDEBAR_SYNC_MS.store(now_millis(), Ordering::Relaxed);
    let Ok(main_pos) = main_win.outer_position() else { return };
    let Ok(main_outer) = main_win.outer_size() else { return };
    if main_outer.height == 0 || main_outer.width == 0 {
        let _ = sidebar.show();
        return;
    }
    let w = sidebar_width();
    let sidebar_scale = sidebar.scale_factor().unwrap_or_else(|_| main_win.scale_factor().unwrap_or(1.0));
    let sidebar_width_physical = (w * sidebar_scale).round() as i32;
    let overlap_left = 16;
    let (sidebar_x_physical, width_physical) = if sidebar_on_right() {
        let x = main_pos.x + main_outer.width as i32;
        let w = (sidebar_width_physical + 4).max(0) as u32;
        (x, w)
    } else {
        let x = (main_pos.x - sidebar_width_physical).max(0);
        let w = (sidebar_width_physical + overlap_left) as u32;
        (x, w)
    };
    let sidebar_y = main_pos.y;
    let sidebar_pos = tauri::PhysicalPosition::new(sidebar_x_physical, sidebar_y);
    let _ = sidebar.set_position(sidebar_pos);
    let size_physical = tauri::PhysicalSize {
        width: width_physical,
        height: main_outer.height,
    };
    let _ = sidebar.set_size(size_physical);
    let _ = sidebar.show();
    if bring_to_front {
        let _ = sidebar.set_focus();
        let _ = main_win.set_focus();
    }
}

/// 主窗 Moved 时：延迟 16ms 再读主窗位置并同步；同时做置顶，避免主窗向左拖覆盖左侧分组窗后松手时分组窗被压在下面。
const MOVE_SYNC_DEFER_MS: u64 = 16;

fn sync_sidebar_on_moved(app: &AppHandle) {
    let my_count = MOVE_DEBOUNCE.fetch_add(1, Ordering::SeqCst) + 1;
    let app2 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(MOVE_SYNC_DEFER_MS));
        let app3 = app2.clone();
        let _ = app2.run_on_main_thread(move || {
            if MOVE_DEBOUNCE.load(Ordering::SeqCst) == my_count {
                sync_sidebar_to_main(&app3, true);
            }
        });
    });
    let app4 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let app5 = app4.clone();
        let _ = app4.run_on_main_thread(move || {
            if MOVE_DEBOUNCE.load(Ordering::SeqCst) == my_count {
                sync_sidebar_to_main(&app5, true);
            }
        });
    });
}

/// 主窗 Resized 时：节流 + 防抖。节流避免每帧同步卡顿，松手后 150ms 再做一次“置顶”同步。
fn sync_sidebar_on_resized(app: &AppHandle) {
    save_main_window_size_on_resize(app);
    let now = now_millis();
    if now.saturating_sub(LAST_SIDEBAR_SYNC_MS.load(Ordering::Relaxed)) >= SIDEBAR_SYNC_THROTTLE_MS {
        sync_sidebar_to_main(app, false);
    }
    let my_count = RESIZE_DEBOUNCE.fetch_add(1, Ordering::SeqCst) + 1;
    let app2 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let app3 = app2.clone();
        let _ = app2.run_on_main_thread(move || {
            if RESIZE_DEBOUNCE.load(Ordering::SeqCst) == my_count {
                sync_sidebar_to_main(&app3, true);
            }
        });
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    pub importance: String, // normal | important | urgent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub sort_order: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub app_handle: AppHandle,
}

const CREATE_GROUPS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
)"#;

const CREATE_TODOS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS todos (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    status TEXT NOT NULL,
    reminder_time TEXT,
    start_time TEXT,
    end_time TEXT,
    importance TEXT NOT NULL DEFAULT 'normal',
    group_id TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
)"#;

const MIGRATE_TODOS_COLUMNS: &[&str] = &[
    "ALTER TABLE todos ADD COLUMN start_time TEXT",
    "ALTER TABLE todos ADD COLUMN end_time TEXT",
    "ALTER TABLE todos ADD COLUMN importance TEXT NOT NULL DEFAULT 'normal'",
    "ALTER TABLE todos ADD COLUMN group_id TEXT",
    "ALTER TABLE todos ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
];

const MIGRATE_GROUPS_COLUMNS: &[&str] = &["ALTER TABLE groups ADD COLUMN sort_order INTEGER DEFAULT 0"];

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            get_web_server_url,
            refresh_main_window_transparency,
            open_qrcode_window,
            open_settings_window,
            set_sidebar_width,
            set_sidebar_position,
            save_background_image,
            clear_background_image,
            get_storage_info,
            open_path_in_folder
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let db_path = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("todos.db");

            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).expect("创建应用数据目录失败");
            }

            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let pool = rt.block_on(async {
                let opts = sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true);
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .connect_with(opts)
                    .await
                    .expect("SQLite 连接失败");
                sqlx::query(CREATE_GROUPS_SQL)
                    .execute(&pool)
                    .await
                    .expect("创建 groups 表失败");
                sqlx::query(CREATE_TODOS_SQL)
                    .execute(&pool)
                    .await
                    .expect("创建 todos 表失败");
                for sql in MIGRATE_TODOS_COLUMNS {
                    let _ = sqlx::query(*sql).execute(&pool).await;
                }
                for sql in MIGRATE_GROUPS_COLUMNS {
                    let _ = sqlx::query(*sql).execute(&pool).await;
                }
                pool
            });

            let state = AppState {
                pool,
                app_handle: app_handle.clone(),
            };

            let stop = Arc::new(AtomicBool::new(false));
            let stop_clone = stop.clone();
            let state_clone = state.clone();
            std::thread::spawn(move || {
                server::run_axum(state_clone, stop_clone);
            });

            app.manage(state);
            app.manage(stop);

            let handle = app.handle().clone();
            let _ = app.run_on_main_thread(move || {
                load_sidebar_position_from_store(&handle);
                apply_main_window_initial_size(&handle);
                // 先显示主窗，再创建并显示侧栏，避免主窗未 show 时创建侧栏导致一直不出现
                if let Some(main_win) = handle.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
                if create_sidebar_window(&handle).is_err() {
                    eprintln!("sidebar window create failed (retry on first move/resize)");
                }
                sync_sidebar_to_main(&handle, true);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            let label = window.label();
            match event {
                tauri::WindowEvent::Destroyed => {
                    if label == "main" {
                        if let Some(sidebar) = window.app_handle().get_webview_window("sidebar") {
                            let _ = sidebar.close();
                        }
                    }
                    if let Some(stop) = window.app_handle().try_state::<Arc<AtomicBool>>() {
                        stop.store(true, Ordering::SeqCst);
                    }
                }
                tauri::WindowEvent::Resized(_) => {
                    if label == "main" {
                        sync_sidebar_on_resized(window.app_handle());
                    }
                }
                tauri::WindowEvent::Moved(_) => {
                    if label == "main" {
                        sync_sidebar_on_moved(window.app_handle());
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
