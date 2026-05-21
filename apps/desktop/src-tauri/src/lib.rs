use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sright_core::{
    append_action_log, execute_configured_action, load_or_init_config, read_recent_logs,
    save_config, ActionLogEntry, ActionRequest, SRightConfig,
};
use tauri::{image::Image, menu::MenuBuilder, tray::TrayIconBuilder, Manager};

#[derive(Debug, Serialize)]
struct Diagnostics {
    config_path: String,
    log_path: String,
}

#[derive(Debug, Deserialize)]
struct PendingFinderAction {
    request_id: String,
    action_id: String,
    paths: Vec<PathBuf>,
    confirmed_dangerous: bool,
}

#[tauri::command]
fn load_config() -> Result<SRightConfig, String> {
    load_or_init_config().map_err(|error| error.to_string())
}

#[tauri::command]
fn save_config_command(app: tauri::AppHandle, config: SRightConfig) -> Result<(), String> {
    save_config(&config).map_err(|error| error.to_string())?;
    sync_tray_visibility(&app, config.show_menu_bar_icon)
}

#[tauri::command]
fn recent_logs(limit: usize) -> Result<Vec<ActionLogEntry>, String> {
    read_recent_logs(limit).map_err(|error| error.to_string())
}

#[tauri::command]
fn diagnostics() -> Diagnostics {
    Diagnostics {
        config_path: sright_core::paths::config_path().display().to_string(),
        log_path: sright_core::paths::log_path().display().to_string(),
    }
}

#[tauri::command]
fn open_finder_extension_settings() -> Result<(), String> {
    open_system_settings(&[
        "x-apple.systempreferences:com.apple.ExtensionsPreferences?Finder",
        "x-apple.systempreferences:com.apple.ExtensionsPreferences",
    ])
}

#[tauri::command]
fn open_full_disk_access_settings() -> Result<(), String> {
    open_system_settings(&[
        "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
        "x-apple.systempreferences:com.apple.Security-Privacy?Privacy_AllFiles",
        "x-apple.systempreferences:com.apple.preference.security",
    ])
}

fn open_system_settings(candidates: &[&str]) -> Result<(), String> {
    for target in candidates {
        if Command::new("open")
            .arg(target)
            .status()
            .map_err(|error| error.to_string())?
            .success()
        {
            return Ok(());
        }
    }

    Command::new("open")
        .arg("-b")
        .arg("com.apple.systempreferences")
        .status()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|error| error.to_string())
}

#[tauri::command]
fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())
}

#[tauri::command]
fn pick_directory() -> Result<Option<String>, String> {
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(r#"POSIX path of (choose folder with prompt "选择发送文件到目录")"#)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(Some(normalize_chosen_directory(&path)));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("-128") || stderr.contains("User canceled") {
        return Ok(None);
    }

    Err(stderr.trim().to_string())
}

#[tauri::command]
fn pick_template_file() -> Result<Option<String>, String> {
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(r#"POSIX path of (choose file with prompt "选择新建文件模板")"#)
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(Some(path));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("-128") || stderr.contains("User canceled") {
        return Ok(None);
    }

    Err(stderr.trim().to_string())
}

#[tauri::command]
fn open_path_in_finder(path: String) -> Result<(), String> {
    let status = Command::new("open")
        .arg(expand_home_path(&path))
        .status()
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("open exited with status {status}"));
    }
    Ok(())
}

pub fn run() {
    spawn_pending_action_worker();
    tauri::Builder::default()
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config_command,
            recent_logs,
            diagnostics,
            open_finder_extension_settings,
            open_full_disk_access_settings,
            close_window,
            minimize_window,
            pick_directory,
            pick_template_file,
            open_path_in_finder
        ])
        .run(tauri::generate_context!())
        .expect("error while running sRight desktop app");
}

fn normalize_chosen_directory(path: &str) -> String {
    if path == "/" {
        return path.to_string();
    }

    path.trim_end_matches('/').to_string()
}

fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(path)
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = MenuBuilder::new(app)
        .text("preferences", "偏好设置")
        .separator()
        .text("quit", "退出")
        .build()?;
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("sRight")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "preferences" => show_preferences(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    let config = load_or_init_config()?;
    tray.set_visible(config.show_menu_bar_icon)?;

    Ok(())
}

fn sync_tray_visibility(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_visible(visible)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn show_preferences(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn spawn_pending_action_worker() {
    std::thread::spawn(|| loop {
        if let Err(error) = process_pending_actions_once() {
            eprintln!("sRight pending action worker failed: {error:#}");
        }
        std::thread::sleep(Duration::from_millis(300));
    });
}

fn process_pending_actions_once() -> Result<(), String> {
    let pending_dir = pending_actions_dir();
    if !pending_dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&pending_dir).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        let processing_path = path.with_extension("processing");
        if std::fs::rename(&path, &processing_path).is_err() {
            continue;
        }

        let result = process_pending_action_file(&processing_path);
        let finished_path =
            processing_path.with_extension(if result.is_ok() { "done" } else { "failed" });
        let _ = std::fs::rename(&processing_path, finished_path);
        result?;
    }

    Ok(())
}

fn process_pending_action_file(path: &Path) -> Result<(), String> {
    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let action: PendingFinderAction =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    run_pending_finder_action(action)
}

fn run_pending_finder_action(action: PendingFinderAction) -> Result<(), String> {
    let selected_count = action.paths.len();
    let config = load_or_init_config().map_err(|error| error.to_string())?;
    if !config.enabled {
        let message = "sRight is disabled in config";
        append_action_log(&ActionLogEntry::failure(
            &action.action_id,
            selected_count,
            message,
            message,
        ))
        .map_err(|error| error.to_string())?;
        return Err(message.to_string());
    }

    let action_present = config.menus.iter().any(|menu| menu.id == action.action_id)
        || new_file_template_enabled(&config, &action.action_id).is_some();
    if !action_present {
        let message = format!("action is not present in config: {}", action.action_id);
        append_action_log(&ActionLogEntry::failure(
            &action.action_id,
            selected_count,
            "Action blocked before execution",
            &message,
        ))
        .map_err(|error| error.to_string())?;
        return Err(message);
    }

    let action_enabled = if action.action_id.starts_with("new_file.") {
        new_file_template_enabled(&config, &action.action_id).unwrap_or(false)
    } else {
        config
            .menus
            .iter()
            .find(|menu| menu.id == action.action_id)
            .map(|menu| menu.enabled)
            .unwrap_or(false)
    };
    if !action_enabled {
        let message = format!("action is disabled in config: {}", action.action_id);
        append_action_log(&ActionLogEntry::failure(
            &action.action_id,
            selected_count,
            "Action blocked before execution",
            &message,
        ))
        .map_err(|error| error.to_string())?;
        return Err(message);
    }

    let confirmed_dangerous = action.confirmed_dangerous
        || !config
            .dangerous_confirmation
            .requires_confirmation(&action.action_id);
    match execute_configured_action(
        ActionRequest {
            action_id: action.action_id.clone(),
            paths: action.paths,
            confirmed_dangerous,
        },
        &config,
    ) {
        Ok(result) => {
            if let Some(text) = result.payload.get("text").and_then(|value| value.as_str()) {
                write_clipboard_text(text).map_err(|error| error.to_string())?;
            }
            append_action_log(&ActionLogEntry::success(
                &result.action_id,
                result.selected_count,
                &format!("{} (request {})", result.message, action.request_id),
            ))
            .map_err(|error| error.to_string())?;
            Ok(())
        }
        Err(error) => {
            append_action_log(&ActionLogEntry::failure(
                &action.action_id,
                selected_count,
                "Action failed",
                error.to_string(),
            ))
            .map_err(|log_error| log_error.to_string())?;
            Err(error.to_string())
        }
    }
}

fn new_file_template_enabled(config: &SRightConfig, action_id: &str) -> Option<bool> {
    let template_id = action_id.strip_prefix("new_file.")?;
    config
        .file_templates
        .iter()
        .find(|template| template.id == template_id)
        .map(|template| template.enabled)
}

fn pending_actions_dir() -> PathBuf {
    if let Ok(path) = std::env::var("SRIGHT_PENDING_ACTIONS_DIR") {
        return PathBuf::from(path);
    }

    sright_core::paths::finder_sync_app_support_dir()
        .unwrap_or_else(sright_core::paths::app_support_dir)
        .join("pending-actions")
}

fn write_clipboard_text(text: &str) -> Result<(), String> {
    if let Ok(path) = std::env::var("SRIGHT_CLIPBOARD_FILE") {
        std::fs::write(&path, text).map_err(|error| format!("could not write {path}: {error}"))?;
        return Ok(());
    }

    let mut child = Command::new("/usr/bin/pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start pbcopy: {error}"))?;
    let Some(stdin) = child.stdin.as_mut() else {
        return Err("could not open pbcopy stdin".to_string());
    };
    stdin
        .write_all(text.as_bytes())
        .map_err(|error| format!("could not send text to pbcopy: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("could not wait for pbcopy: {error}"))?;
    if !status.success() {
        return Err(format!("pbcopy exited with status {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sright-desktop-{label}-{nanos}"))
    }

    #[test]
    fn pending_finder_action_creates_file_through_desktop_worker() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support");
        let pending_dir = temp_dir("pending");
        let selected_dir = temp_dir("selected");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::fs::create_dir_all(&selected_dir).unwrap();
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
        std::env::set_var("SRIGHT_PENDING_ACTIONS_DIR", &pending_dir);
        std::env::set_var("SRIGHT_SKIP_FINDER_SYNC_SYNC", "1");

        let request = serde_json::json!({
            "request_id": "test-request",
            "action_id": "new_file.text",
            "paths": [selected_dir.display().to_string()],
            "confirmed_dangerous": false
        });
        std::fs::write(pending_dir.join("test-request.json"), request.to_string()).unwrap();

        process_pending_actions_once().unwrap();

        assert!(selected_dir.join("Untitled.txt").exists());
        assert!(pending_dir.join("test-request.done").exists());
    }

    #[test]
    fn pending_finder_action_runs_dangerous_file_operation() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-dangerous");
        let pending_dir = temp_dir("pending-dangerous");
        let selected_dir = temp_dir("selected-dangerous");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::fs::create_dir_all(&selected_dir).unwrap();
        let delete_target = selected_dir.join("delete-me.txt");
        std::fs::write(&delete_target, "delete").unwrap();
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
        std::env::set_var("SRIGHT_PENDING_ACTIONS_DIR", &pending_dir);
        std::env::set_var("SRIGHT_SKIP_FINDER_SYNC_SYNC", "1");

        let request = serde_json::json!({
            "request_id": "delete-request",
            "action_id": "file.delete_permanently",
            "paths": [delete_target.display().to_string()],
            "confirmed_dangerous": true
        });
        std::fs::write(pending_dir.join("delete-request.json"), request.to_string()).unwrap();

        process_pending_actions_once().unwrap();

        assert!(!delete_target.exists());
        assert!(pending_dir.join("delete-request.done").exists());
    }
}
