use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use sright_core::{append_action_log, execute_action, ActionLogEntry, ActionRequest};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn default_registry_includes_phase4_and_phase5_actions() {
    let descriptors = sright_core::action_descriptors();
    let ids = descriptors
        .iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<Vec<_>>();

    for id in [
        "archive.zip",
        "archive.zip_each",
        "archive.unzip_here",
        "archive.unzip_to_folder",
        "image.convert.png",
        "image.convert.jpg",
        "image.convert.webp",
        "image.convert.heic",
        "icon.make_iconset",
        "icon.make_icns",
        "icon.set_custom",
        "icon.remove_custom",
        "tool.hash.md5",
        "tool.hash.sha1",
        "tool.hash.sha256",
        "tool.qr.path",
        "tool.open_parent",
        "tool.copy_summary",
        "script.run.default",
    ] {
        assert!(ids.contains(&id), "missing action {id}");
    }
}

#[test]
fn default_config_includes_phase4_and_phase5_settings() {
    let config = sright_core::default_config();

    assert!(!config.archive.delete_source_after_archive);
    assert!(!config.archive.delete_archive_after_extract);
    assert_eq!(config.image.output_dir, None);
    assert_eq!(config.toolbox.translation_provider, "none");
    assert!(config
        .custom_scripts
        .iter()
        .any(|script| script.id == "default"));
}

#[test]
fn zip_then_unzip_to_folder_round_trips_selected_file() {
    let root = temp_dir("zip-roundtrip");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("note.txt");
    fs::write(&file, "hello").unwrap();

    execute_action(request("archive.zip", &[file.clone()])).unwrap();
    let archive = root.join("note.zip");
    assert!(archive.is_file());

    execute_action(request("archive.unzip_to_folder", &[archive])).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("note").join("note.txt")).unwrap(),
        "hello"
    );
}

#[test]
fn hash_actions_return_expected_clipboard_text() {
    let root = temp_dir("hash");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("hello.txt");
    fs::write(&file, "hello").unwrap();

    let md5 = execute_action(request("tool.hash.md5", &[file.clone()])).unwrap();
    let sha1 = execute_action(request("tool.hash.sha1", &[file.clone()])).unwrap();
    let sha256 = execute_action(request("tool.hash.sha256", &[file])).unwrap();

    assert_eq!(
        md5.payload["text"],
        "5d41402abc4b2a76b9719d911017c592  hello.txt"
    );
    assert_eq!(
        sha1.payload["text"],
        "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d  hello.txt"
    );
    assert_eq!(
        sha256.payload["text"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  hello.txt"
    );
}

#[test]
fn qr_path_creates_svg_next_to_selected_file() {
    let root = temp_dir("qr");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("target.txt");
    fs::write(&file, "target").unwrap();

    let result = execute_action(request("tool.qr.path", &[file.clone()])).unwrap();

    let qr = root.join("target.path-qr.svg");
    assert_eq!(result.selected_count, 1);
    assert!(qr.is_file());
    assert!(fs::read_to_string(qr).unwrap().contains("<svg"));
}

#[test]
fn script_action_uses_configurable_test_boundary() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("script");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("input.txt");
    let log = root.join("script.log");
    fs::write(&target, "input").unwrap();
    std::env::set_var("SRIGHT_SCRIPT_LOG_FILE", &log);

    execute_action(request("script.run.default", &[target.clone()])).unwrap();

    let contents = fs::read_to_string(log).unwrap();
    assert!(contents.contains("script.run.default"));
    assert!(contents.contains(&target.display().to_string()));

    std::env::remove_var("SRIGHT_SCRIPT_LOG_FILE");
}

#[test]
fn logs_can_be_searched_and_exported() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_dir("logs");
    let export_path = support_dir.join("export.jsonl");
    fs::create_dir_all(&support_dir).unwrap();
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
    std::env::set_var("SRIGHT_LOG_EXPORT_PATH", &export_path);
    append_action_log(&ActionLogEntry::success("tool.hash.sha256", 1, "hash ok")).unwrap();
    append_action_log(&ActionLogEntry::failure(
        "archive.zip",
        1,
        "zip failed",
        "boom",
    ))
    .unwrap();

    let search = execute_action(request("logs.search", &[PathBuf::from("hash")])).unwrap();
    let export = execute_action(request("logs.export", &[])).unwrap();

    assert_eq!(
        search.payload["matches"][0]["action_id"],
        "tool.hash.sha256"
    );
    assert_eq!(export.payload["path"], export_path.display().to_string());
    assert!(fs::read_to_string(export_path)
        .unwrap()
        .contains("archive.zip"));

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
    std::env::remove_var("SRIGHT_LOG_EXPORT_PATH");
}

fn request(action_id: &str, paths: &[PathBuf]) -> ActionRequest {
    ActionRequest {
        action_id: action_id.to_string(),
        paths: paths.to_vec(),
        confirmed_dangerous: false,
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sright-phase45-{label}-{suffix}"))
}
