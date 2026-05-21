use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use sright_core::{
    append_action_log, default_config, execute_action, load_or_init_config, read_recent_logs,
    save_config, ActionLogEntry, ActionRequest,
};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn load_or_init_config_creates_default_json() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("config");
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

    let config = load_or_init_config().expect("default config should be created");

    assert_eq!(config, default_config());
    assert!(support_dir.join("config.json").exists());

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn save_config_round_trips_enabled_flag() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("round-trip");
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

    let mut config = default_config();
    config.enabled = false;
    save_config(&config).expect("config should save");

    let loaded = load_or_init_config().expect("config should load");
    assert!(!loaded.enabled);

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn load_or_init_config_upgrades_older_config_with_phase3_defaults() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("upgrade");
    fs::create_dir_all(&support_dir).unwrap();
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
    fs::write(
        support_dir.join("config.json"),
        r#"{
  "enabled": true,
  "show_icons": true,
  "merge_groups": false,
  "dangerous_confirmation": { "enabled": true, "action_ids": [] },
  "menus": []
}
"#,
    )
    .unwrap();

    let config = load_or_init_config().expect("older config should be upgraded");

    assert!(config.menus.iter().any(|menu| menu.id == "new_file.text"));
    assert!(config
        .file_templates
        .iter()
        .any(|template| template.id == "text"));
    assert!(config.open_apps.iter().any(|app| app.id == "terminal"));
    assert!(config
        .favorite_dirs
        .iter()
        .any(|directory| directory.id == "downloads"));

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn debug_echo_reports_selected_count() {
    let result = execute_action(ActionRequest {
        action_id: "debug.echo".to_string(),
        paths: vec![PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")],
        confirmed_dangerous: false,
    })
    .expect("debug action should execute");

    assert_eq!(result.action_id, "debug.echo");
    assert_eq!(result.selected_count, 2);
    assert!(result.message.contains("/tmp/a.txt"));
}

#[test]
fn jsonl_logs_append_and_tail() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("logs");
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

    append_action_log(&ActionLogEntry::success("debug.echo", 1, "first")).unwrap();
    append_action_log(&ActionLogEntry::success("debug.echo", 2, "second")).unwrap();

    let raw = fs::read_to_string(support_dir.join("actions.jsonl")).unwrap();
    assert_eq!(raw.lines().count(), 2);

    let recent = read_recent_logs(1).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].selected_count, 2);

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

fn temp_support_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sright-{label}-{suffix}"))
}
