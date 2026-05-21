use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use sright_core::{
    action_descriptors, default_config, execute_action, ActionRequest, DangerousConfirmationConfig,
};

#[test]
fn default_config_includes_phase2_menus_and_dangerous_confirmation() {
    let config = default_config();
    let menu_ids = config
        .menus
        .iter()
        .map(|menu| menu.id.as_str())
        .collect::<Vec<_>>();

    for id in [
        "copy.path",
        "copy.name",
        "file.delete_permanently",
        "folder.create_from_filename",
        "folder.dissolve",
        "file.info",
    ] {
        assert!(menu_ids.contains(&id), "missing default menu {id}");
    }

    assert!(config.dangerous_confirmation.enabled);
    assert!(config
        .dangerous_confirmation
        .action_ids
        .contains(&"file.delete_permanently".to_string()));
}

#[test]
fn action_registry_marks_dangerous_actions() {
    let descriptors = action_descriptors();

    assert!(descriptors
        .iter()
        .any(|action| action.id == "file.delete_permanently" && action.dangerous));
}

#[test]
fn copy_variants_return_expected_payload_text() {
    let root = temp_dir("copy");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("hello world.txt");
    fs::write(&file, "hello").unwrap();

    assert_payload_text("copy.path", &[file.clone()], &file.display().to_string());
    assert_payload_text("copy.name", &[file.clone()], "hello world.txt");
}

#[test]
fn file_info_returns_metadata_payload() {
    let root = temp_dir("info");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("info.txt");
    fs::write(&file, "hello").unwrap();

    let result = execute_action(request("file.info", &[file.clone()], false)).unwrap();

    assert_eq!(result.selected_count, 1);
    assert_eq!(result.payload["items"][0]["name"], "info.txt");
    assert_eq!(result.payload["items"][0]["is_file"], true);
    assert_eq!(
        result.payload["items"][0]["path"],
        file.display().to_string()
    );
}

#[test]
fn create_folder_from_filename_copies_file_into_created_folder() {
    let root = temp_dir("folder-from-file");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("archive.tar.gz");
    fs::write(&file, "data").unwrap();

    let result = execute_action(request(
        "folder.create_from_filename",
        &[file.clone()],
        false,
    ))
    .unwrap();

    assert_eq!(result.selected_count, 1);
    let folder = root.join("archive.tar");
    assert!(folder.is_dir());
    assert!(file.exists());
    assert_eq!(fs::read_to_string(&file).unwrap(), "data");
    assert_eq!(
        fs::read_to_string(folder.join("archive.tar.gz")).unwrap(),
        "data"
    );
    assert_eq!(
        result.payload["copied"][0],
        folder.join("archive.tar.gz").display().to_string()
    );
}

#[test]
fn dissolve_folder_moves_children_to_parent_and_rejects_collisions() {
    let root = temp_dir("dissolve");
    let folder = root.join("Folder");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("child.txt"), "child").unwrap();

    execute_action(request("folder.dissolve", &[folder.clone()], false)).unwrap();

    assert!(root.join("child.txt").exists());
    assert!(!folder.exists());

    let collision_folder = root.join("Collision");
    fs::create_dir_all(&collision_folder).unwrap();
    fs::write(collision_folder.join("child.txt"), "new").unwrap();
    let error = execute_action(request("folder.dissolve", &[collision_folder], false))
        .expect_err("collision should fail");
    assert!(error.to_string().contains("already exists"));
}

#[test]
fn dangerous_actions_require_confirmation() {
    let root = temp_dir("dangerous");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("delete-me.txt");
    fs::write(&file, "bye").unwrap();

    let error = execute_action(request("file.delete_permanently", &[file.clone()], false))
        .expect_err("delete should require confirmation");
    assert!(error.to_string().contains("requires confirmation"));
    assert!(file.exists());

    execute_action(request("file.delete_permanently", &[file.clone()], true)).unwrap();
    assert!(!file.exists());
}

#[test]
fn dangerous_confirmation_config_defaults_to_all_dangerous_actions() {
    let config = DangerousConfirmationConfig::default();

    assert!(config.requires_confirmation("file.delete_permanently"));
    assert!(!config.requires_confirmation("copy.path"));
}

#[test]
fn unknown_action_is_rejected() {
    let error = execute_action(request(
        "legacy.action",
        &[PathBuf::from("/tmp/example.txt")],
        true,
    ))
    .expect_err("unknown action should not execute");
    assert!(error.to_string().contains("unknown action"));
}

fn assert_payload_text(action_id: &str, paths: &[PathBuf], expected: &str) {
    let result = execute_action(request(action_id, paths, false)).unwrap();
    assert_eq!(result.payload["text"], Value::String(expected.to_string()));
}

fn request(action_id: &str, paths: &[PathBuf], confirmed_dangerous: bool) -> ActionRequest {
    ActionRequest {
        action_id: action_id.to_string(),
        paths: paths.to_vec(),
        confirmed_dangerous,
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sright-phase2-{label}-{suffix}"))
}
