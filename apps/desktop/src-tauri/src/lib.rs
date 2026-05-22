use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sright_core::{
    append_action_log, apply_action_result_updates, execute_configured_action, load_or_init_config,
    read_recent_logs, save_config, ActionLogEntry, ActionRequest, SRightConfig,
};
use tauri::{image::Image, menu::MenuBuilder, tray::TrayIconBuilder, Manager, RunEvent};

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

static ACTION_RESULT_REOPEN_SUPPRESSED: AtomicBool = AtomicBool::new(false);
const BACKGROUND_ACTION_ARG: &str = "--sright-background-action";
const FINDER_ACTION_REOPEN_SUPPRESSION_WINDOW: Duration = Duration::from_secs(30);

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
    window.hide().map_err(|error| error.to_string())?;
    set_dock_visible(window.app_handle(), false)
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
    tauri::Builder::default()
        .setup(|app| {
            setup_tray(app)?;
            if should_show_preferences_on_startup() {
                show_preferences(app.handle());
            }
            spawn_pending_action_worker(app.handle().clone());
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
        .build(tauri::generate_context!())
        .expect("error while building sRight desktop app")
        .run(|app, event| handle_run_event(app, event));
}

fn handle_run_event(app: &tauri::AppHandle, event: RunEvent) {
    #[cfg(target_os = "macos")]
    if let RunEvent::Reopen { .. } = event {
        if !should_show_preferences_on_reopen() {
            return;
        }
        show_preferences(app);
    }
}

fn should_show_preferences_on_startup() -> bool {
    should_show_preferences_on_startup_with_args(std::env::args())
}

fn should_show_preferences_on_startup_with_args<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let is_background_action = args
        .into_iter()
        .any(|arg| arg.as_ref() == BACKGROUND_ACTION_ARG);
    !is_background_action && !has_pending_finder_actions()
}

fn should_show_preferences_on_reopen() -> bool {
    !ACTION_RESULT_REOPEN_SUPPRESSED.load(Ordering::SeqCst)
        && !has_pending_finder_actions()
        && !has_recent_finder_action_files(FINDER_ACTION_REOPEN_SUPPRESSION_WINDOW)
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

fn set_dock_visible(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return app
            .set_dock_visibility(visible)
            .map_err(|error| error.to_string());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, visible);
        Ok(())
    }
}

fn show_preferences(app: &tauri::AppHandle) {
    let _ = set_dock_visible(app, true);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn spawn_pending_action_worker(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        if let Err(error) = process_pending_actions_once_with_app(Some(&app)) {
            eprintln!("sRight pending action worker failed: {error:#}");
        }
        std::thread::sleep(Duration::from_millis(300));
    });
}

fn has_pending_finder_actions() -> bool {
    let pending_dir = pending_actions_dir();
    let Ok(entries) = std::fs::read_dir(&pending_dir) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("json" | "processing")
        )
    })
}

fn has_recent_finder_action_files(window: Duration) -> bool {
    let pending_dir = pending_actions_dir();
    let Ok(entries) = std::fs::read_dir(&pending_dir) else {
        return false;
    };

    let now = std::time::SystemTime::now();
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("json" | "processing" | "done" | "failed")
        ) {
            return false;
        }

        entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age <= window)
    })
}

#[cfg(test)]
fn process_pending_actions_once() -> Result<(), String> {
    process_pending_actions_once_with_app(None)
}

fn process_pending_actions_once_with_app(app: Option<&tauri::AppHandle>) -> Result<(), String> {
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

        let result = process_pending_action_file(&processing_path, app);
        let finished_path =
            processing_path.with_extension(if result.is_ok() { "done" } else { "failed" });
        let _ = std::fs::rename(&processing_path, finished_path);
        result?;
    }

    Ok(())
}

fn process_pending_action_file(path: &Path, app: Option<&tauri::AppHandle>) -> Result<(), String> {
    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let action: PendingFinderAction =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    run_pending_finder_action(action, app)
}

fn run_pending_finder_action(
    action: PendingFinderAction,
    app: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    suppress_action_result_reopen();
    let selected_count = action.paths.len();
    let mut config = load_or_init_config().map_err(|error| error.to_string())?;
    if !config.enabled {
        let message = "sRight is disabled in config";
        append_action_log(&ActionLogEntry::failure(
            &action.action_id,
            selected_count,
            message,
            message,
        ))
        .map_err(|error| error.to_string())?;
        let _ = present_system_error_toast("操作失败", message);
        schedule_action_result_reopen_unsuppress();
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
        let _ = present_system_error_toast("操作失败", &message);
        schedule_action_result_reopen_unsuppress();
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
        let _ = present_system_error_toast("操作失败", &message);
        schedule_action_result_reopen_unsuppress();
        return Err(message);
    }

    let requires_confirmation = config
        .dangerous_confirmation
        .requires_confirmation(&action.action_id);
    let confirmed_dangerous =
        action.confirmed_dangerous || !requires_confirmation || confirm_dangerous_action(&action)?;
    match execute_configured_action(
        ActionRequest {
            action_id: action.action_id.clone(),
            paths: action.paths,
            confirmed_dangerous,
        },
        &config,
    ) {
        Ok(result) => {
            if result
                .payload
                .get("clipboard_operation")
                .and_then(|value| value.as_str())
                == Some("cut")
            {
                let paths = result
                    .payload
                    .get("paths")
                    .and_then(|value| value.as_array())
                    .map(|paths| {
                        paths
                            .iter()
                            .filter_map(|path| path.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                write_cut_clipboard(&paths).map_err(|error| error.to_string())?;
            } else if let Some(text) = result.payload.get("text").and_then(|value| value.as_str()) {
                write_clipboard_text(text).map_err(|error| error.to_string())?;
            }
            if apply_action_result_updates(&mut config, &result) {
                save_config(&config).map_err(|error| error.to_string())?;
            }
            present_action_result(app, &result)?;
            append_action_log(&ActionLogEntry::success(
                &result.action_id,
                result.selected_count,
                &format!("{} (request {})", result.message, action.request_id),
            ))
            .map_err(|error| error.to_string())?;
            schedule_action_result_reopen_unsuppress();
            Ok(())
        }
        Err(error) => {
            let error_message = error.to_string();
            append_action_log(&ActionLogEntry::failure(
                &action.action_id,
                selected_count,
                "Action failed",
                &error_message,
            ))
            .map_err(|log_error| log_error.to_string())?;
            let _ = present_system_error_toast("操作失败", &error_message);
            schedule_action_result_reopen_unsuppress();
            Err(error_message)
        }
    }
}

fn confirm_dangerous_action(action: &PendingFinderAction) -> Result<bool, String> {
    if let Ok(response) = std::env::var("SRIGHT_CONFIRM_DANGEROUS_RESPONSE") {
        return Ok(matches!(
            response.as_str(),
            "confirm" | "ok" | "yes" | "true" | "1"
        ));
    }

    let message = format!(
        "彻底删除将作用于 {} 个选中项。此操作不可撤销。",
        action.paths.len()
    );
    let script = format!(
        "display alert \"确认执行危险动作？\" message \"{}\" as critical buttons {{\"取消\", \"确认\"}} default button \"取消\" cancel button \"取消\"",
        applescript_string_literal(&message)
    );
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|error| format!("could not start osascript: {error}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).contains("确认"));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("-128") || stderr.contains("User canceled") {
        return Ok(false);
    }

    Err(stderr.trim().to_string())
}

fn applescript_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn present_action_result(
    app: Option<&tauri::AppHandle>,
    result: &sright_core::ActionResult,
) -> Result<(), String> {
    if !should_present_action_result(result) {
        return Ok(());
    }

    let _ = app;
    present_system_action_result(result)
}

fn should_present_action_result(result: &sright_core::ActionResult) -> bool {
    result
        .payload
        .get("presentation")
        .and_then(|value| value.as_str())
        == Some("dialog")
}

fn present_system_action_result(result: &sright_core::ActionResult) -> Result<(), String> {
    let title = result
        .payload
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("操作结果");
    let display_text = result
        .payload
        .get("display_text")
        .and_then(|value| value.as_str())
        .unwrap_or(&result.message);
    let copy_text = result
        .payload
        .get("copy_text")
        .and_then(|value| value.as_str());

    if let Ok(path) = std::env::var("SRIGHT_ACTION_RESULT_FILE") {
        let contents = format!("{title}\n\n{display_text}");
        std::fs::write(&path, contents)
            .map_err(|error| format!("could not write {path}: {error}"))?;
        return Ok(());
    }

    let script = system_action_result_script(title, display_text, copy_text);
    suppress_action_result_reopen();
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output();
    schedule_action_result_reopen_unsuppress();
    let output = output.map_err(|error| format!("could not start osascript: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("-128") || stderr.contains("User canceled") {
        return Ok(());
    }

    Err(stderr.trim().to_string())
}

fn present_system_error_toast(title: &str, message: &str) -> Result<(), String> {
    if let Ok(path) = std::env::var("SRIGHT_ACTION_ERROR_TOAST_FILE") {
        let contents = format!("{title}\n\n{message}");
        std::fs::write(&path, contents)
            .map_err(|error| format!("could not write {path}: {error}"))?;
        return Ok(());
    }

    let script = format!(
        "display notification \"{}\" with title \"sRight\" subtitle \"{}\"",
        applescript_string_literal(message),
        applescript_string_literal(title)
    );
    suppress_action_result_reopen();
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output();
    schedule_action_result_reopen_unsuppress();
    let output = output.map_err(|error| format!("could not start osascript: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("-128") || stderr.contains("User canceled") {
        return Ok(());
    }

    Err(stderr.trim().to_string())
}

fn system_action_result_script(title: &str, display_text: &str, copy_text: Option<&str>) -> String {
    let title = applescript_string_literal(title);
    let display_text = applescript_string_literal(display_text);
    let Some(copy_text) = copy_text else {
        return format!(
            r#"display alert "{}" message "{}" as informational buttons {{"关闭"}} default button "关闭""#,
            title, display_text
        );
    };

    format!(
        r#"set resultTitle to "{}"
set resultMessage to "{}"
set copyText to "{}"
set dialogResult to display alert resultTitle message resultMessage as informational buttons {{"关闭", "复制"}} default button "关闭"
if button returned of dialogResult is "复制" then
    set the clipboard to copyText
end if"#,
        title,
        display_text,
        applescript_string_literal(copy_text)
    )
}

fn suppress_action_result_reopen() {
    ACTION_RESULT_REOPEN_SUPPRESSED.store(true, Ordering::SeqCst);
}

fn schedule_action_result_reopen_unsuppress() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(1500));
        ACTION_RESULT_REOPEN_SUPPRESSED.store(false, Ordering::SeqCst);
    });
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

fn write_cut_clipboard(paths: &[String]) -> Result<(), String> {
    if let Ok(path) = std::env::var("SRIGHT_CLIPBOARD_FILE") {
        std::fs::write(&path, format!("cut\n{}\n", paths.join("\n")))
            .map_err(|error| format!("could not write {path}: {error}"))?;
        return Ok(());
    }

    let script = cut_clipboard_script();
    let status = Command::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", script])
        .args(paths)
        .status()
        .map_err(|error| format!("could not start osascript: {error}"))?;
    if !status.success() {
        return Err(format!("osascript exited with status {status}"));
    }

    Ok(())
}

fn cut_clipboard_script() -> &'static str {
    r#"
function run(argv) {
    ObjC.import('AppKit');
    ObjC.import('Foundation');
    const pasteboard = $.NSPasteboard.generalPasteboard;
    const urls = $.NSMutableArray.array;
    argv.forEach(path => urls.addObject($.NSURL.fileURLWithPath(path)));
    pasteboard.clearContents();
    pasteboard.writeObjects(urls);
    pasteboard.setPropertyListForType($(['move']), 'com.apple.finder.pasteboard.operations');
}
"#
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
        assert!(ACTION_RESULT_REOPEN_SUPPRESSED.load(Ordering::SeqCst));

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::remove_var("SRIGHT_SKIP_FINDER_SYNC_SYNC");
        ACTION_RESULT_REOPEN_SUPPRESSED.store(false, Ordering::SeqCst);
    }

    #[test]
    fn pending_finder_action_failure_writes_system_toast_without_window() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-toast");
        let pending_dir = temp_dir("pending-toast");
        let selected_dir = temp_dir("selected-toast");
        let toast_file = temp_dir("toast-output").join("toast.txt");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::fs::create_dir_all(&selected_dir).unwrap();
        std::fs::create_dir_all(toast_file.parent().unwrap()).unwrap();
        let folder = selected_dir.join("Folder");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("note.txt"), "child").unwrap();
        std::fs::write(selected_dir.join("note.txt"), "existing").unwrap();
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
        std::env::set_var("SRIGHT_PENDING_ACTIONS_DIR", &pending_dir);
        std::env::set_var("SRIGHT_SKIP_FINDER_SYNC_SYNC", "1");
        std::env::set_var("SRIGHT_ACTION_ERROR_TOAST_FILE", &toast_file);

        let request = serde_json::json!({
            "request_id": "toast-request",
            "action_id": "folder.dissolve",
            "paths": [folder.display().to_string()],
            "confirmed_dangerous": false
        });
        std::fs::write(pending_dir.join("toast-request.json"), request.to_string()).unwrap();

        assert!(process_pending_actions_once().is_err());

        let toast = std::fs::read_to_string(&toast_file).unwrap();
        assert!(toast.contains("操作失败"));
        assert!(toast.contains("target already exists"));
        assert!(pending_dir.join("toast-request.failed").exists());

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::remove_var("SRIGHT_SKIP_FINDER_SYNC_SYNC");
        std::env::remove_var("SRIGHT_ACTION_ERROR_TOAST_FILE");
        ACTION_RESULT_REOPEN_SUPPRESSED.store(false, Ordering::SeqCst);
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

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::remove_var("SRIGHT_SKIP_FINDER_SYNC_SYNC");
    }

    #[test]
    fn pending_finder_action_cancels_dangerous_operation_without_confirmation() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-dangerous-cancel");
        let pending_dir = temp_dir("pending-dangerous-cancel");
        let selected_dir = temp_dir("selected-dangerous-cancel");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::fs::create_dir_all(&selected_dir).unwrap();
        let delete_target = selected_dir.join("delete-me.txt");
        std::fs::write(&delete_target, "delete").unwrap();
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
        std::env::set_var("SRIGHT_PENDING_ACTIONS_DIR", &pending_dir);
        std::env::set_var("SRIGHT_SKIP_FINDER_SYNC_SYNC", "1");
        std::env::set_var("SRIGHT_CONFIRM_DANGEROUS_RESPONSE", "cancel");

        let request = serde_json::json!({
            "request_id": "delete-request",
            "action_id": "file.delete_permanently",
            "paths": [delete_target.display().to_string()],
            "confirmed_dangerous": false
        });
        std::fs::write(pending_dir.join("delete-request.json"), request.to_string()).unwrap();

        assert!(process_pending_actions_once().is_err());

        assert!(delete_target.exists());
        assert!(pending_dir.join("delete-request.failed").exists());

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::remove_var("SRIGHT_SKIP_FINDER_SYNC_SYNC");
        std::env::remove_var("SRIGHT_CONFIRM_DANGEROUS_RESPONSE");
    }

    #[test]
    fn system_action_result_can_write_result_without_window() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let output_dir = temp_dir("action-result-output");
        std::fs::create_dir_all(&output_dir).unwrap();
        let output_file = output_dir.join("result.txt");
        std::env::set_var("SRIGHT_ACTION_RESULT_FILE", &output_file);

        let result = sright_core::ActionResult {
            action_id: "file.info".to_string(),
            selected_count: 1,
            message: "Collected file info".to_string(),
            payload: serde_json::json!({
                "presentation": "dialog",
                "title": "查看文件信息",
                "display_text": "name: example.txt"
            }),
        };

        present_system_action_result(&result).unwrap();

        assert_eq!(
            std::fs::read_to_string(output_file).unwrap(),
            "查看文件信息\n\nname: example.txt"
        );

        std::env::remove_var("SRIGHT_ACTION_RESULT_FILE");
    }

    #[test]
    fn system_action_result_copy_script_closes_after_copy() {
        let script = system_action_result_script("计算 MD5", "hash  file.txt", Some("hash"));

        assert!(script.contains("set the clipboard to copyText"));
        assert!(!script.contains("repeat"));
        assert!(!script.contains("exit repeat"));
    }

    #[test]
    fn cut_clipboard_script_marks_finder_operation_as_move() {
        let script = cut_clipboard_script();

        assert!(script.contains("pasteboard.clearContents();"));
        assert!(script.contains("com.apple.finder.pasteboard.operations"));
        assert!(script.contains("move"));
    }

    #[test]
    fn startup_preferences_are_suppressed_for_pending_finder_actions() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-startup-pending");
        let pending_dir = support_dir.join("pending-actions");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

        assert!(should_show_preferences_on_startup());

        std::fs::write(pending_dir.join("request.json"), "{}").unwrap();
        assert!(!should_show_preferences_on_startup());

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
    }

    #[test]
    fn startup_preferences_are_suppressed_for_background_action_arg() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-startup-background-arg");
        std::fs::create_dir_all(&support_dir).unwrap();
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

        assert!(should_show_preferences_on_startup_with_args(["sRight"]));
        assert!(!should_show_preferences_on_startup_with_args([
            "sRight",
            BACKGROUND_ACTION_ARG
        ]));

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
    }

    #[test]
    fn reopen_preferences_are_suppressed_after_fast_completed_finder_action() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let support_dir = temp_dir("support-reopen-done");
        let pending_dir = support_dir.join("pending-actions");
        std::fs::create_dir_all(&pending_dir).unwrap();
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
        std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
        ACTION_RESULT_REOPEN_SUPPRESSED.store(false, Ordering::SeqCst);

        std::fs::write(pending_dir.join("new-file.done"), "{}").unwrap();

        assert!(!has_pending_finder_actions());
        assert!(!should_show_preferences_on_reopen());

        std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
        std::env::remove_var("SRIGHT_PENDING_ACTIONS_DIR");
    }
}
