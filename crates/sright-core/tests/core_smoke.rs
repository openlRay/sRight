use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use sright_core::{
    append_action_log, default_config, load_or_init_config, read_recent_logs, save_config,
    ActionLogEntry,
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
fn save_config_preserves_removed_file_templates() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("removed-templates");
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

    let mut config = default_config();
    config.file_templates.clear();
    save_config(&config).expect("config should save");

    let loaded = load_or_init_config().expect("config should load");
    assert!(loaded.file_templates.is_empty());

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn default_config_includes_general_interaction_preferences() {
    let config = default_config();

    assert!(config.show_menu_bar_icon);
    assert_eq!(config.settings_shortcut, "");
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
    assert!(config
        .send_dirs
        .iter()
        .any(|directory| directory.id == "downloads"));
    assert!(config.show_menu_bar_icon);
    assert_eq!(config.settings_shortcut, "");
    assert!(config.menu_tree.iter().any(|item| item.title == "工具箱"));

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn load_or_init_config_removes_unknown_menu_actions() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("unknown-actions");
    fs::create_dir_all(&support_dir).unwrap();
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
    fs::write(
        support_dir.join("config.json"),
        r#"{
  "enabled": true,
  "show_icons": true,
  "merge_groups": false,
  "dangerous_confirmation": { "enabled": true, "action_ids": ["legacy.dangerous", "file.delete_permanently"] },
  "menus": [
    { "id": "legacy.action", "title": "Legacy Action", "enabled": true, "dangerous": false, "file_kinds": [], "extensions": [] },
    { "id": "copy.path", "title": "拷贝路径", "enabled": true, "dangerous": false, "file_kinds": [], "extensions": [] }
  ]
}
"#,
    )
    .unwrap();

    let config = load_or_init_config().expect("config should load");
    assert!(config.menus.iter().all(|menu| menu.id != "legacy.action"));
    assert!(config.menus.iter().any(|menu| menu.id == "copy.path"));
    assert!(config
        .dangerous_confirmation
        .action_ids
        .iter()
        .all(|action_id| action_id != "legacy.dangerous"));
    assert!(config
        .dangerous_confirmation
        .action_ids
        .iter()
        .any(|action_id| action_id == "file.delete_permanently"));

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn jsonl_logs_append_and_tail() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("logs");
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);

    append_action_log(&ActionLogEntry::success("copy.path", 1, "first")).unwrap();
    append_action_log(&ActionLogEntry::success("copy.path", 2, "second")).unwrap();

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
