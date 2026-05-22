use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use sright_core::{
    apply_action_result_updates, default_config, execute_action, execute_configured_action,
    save_config, ActionRequest,
};

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
        "file.shortcut_desktop",
        "share.airdrop",
        "tool.hash.md5",
        "tool.hash.sha1",
        "tool.hash.sha256",
        "tool.hash.sha512",
        "tool.qr.file",
        "tool.open_parent",
        "file.cut",
        "favorite.add_selected",
        "permission.grant_write",
        "visibility.unhide_all",
        "visibility.hide_all",
        "visibility.unhide_selected",
        "visibility.hide_selected",
        "finder.show_extensions",
        "finder.hide_extensions",
    ] {
        assert!(ids.contains(&id), "missing action {id}");
    }

    for id in [
        "icon.set_custom",
        "icon.remove_custom",
        "tool.copy_summary",
        "script.run.default",
        "logs.search",
        "logs.export",
    ] {
        assert!(
            !ids.contains(&id),
            "removed toolbox action should not be registered: {id}"
        );
    }
}

#[test]
fn default_config_includes_phase4_and_phase5_settings() {
    let config = sright_core::default_config();

    assert!(!config.archive.delete_source_after_archive);
    assert!(!config.archive.delete_archive_after_extract);
    assert_eq!(config.image.output_dir, None);
    assert_eq!(config.toolbox.translation_provider, "none");
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
fn unzip_here_rejects_existing_output_file_without_overwriting() {
    let root = temp_dir("unzip-collision");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("note.txt");
    fs::write(&file, "from archive").unwrap();
    execute_action(request("archive.zip", &[file.clone()])).unwrap();
    let archive = root.join("note.zip");
    fs::write(&file, "existing").unwrap();

    let error = execute_action(request("archive.unzip_here", &[archive]))
        .expect_err("existing output file should stop extraction");

    assert!(error.to_string().contains("target already exists"));
    assert_eq!(fs::read_to_string(file).unwrap(), "existing");
}

#[test]
fn hash_actions_return_expected_clipboard_text() {
    let root = temp_dir("hash");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("hello.txt");
    fs::write(&file, "hello").unwrap();

    let md5 = execute_action(request("tool.hash.md5", &[file.clone()])).unwrap();
    let sha1 = execute_action(request("tool.hash.sha1", &[file.clone()])).unwrap();
    let sha256 = execute_action(request("tool.hash.sha256", &[file.clone()])).unwrap();
    let sha512 = execute_action(request("tool.hash.sha512", &[file])).unwrap();

    assert_eq!(
        md5.payload["display_text"],
        "5d41402abc4b2a76b9719d911017c592  hello.txt"
    );
    assert_eq!(
        md5.payload["copy_text"],
        "5d41402abc4b2a76b9719d911017c592  hello.txt"
    );
    assert_eq!(md5.payload["presentation"], "dialog");
    assert_eq!(md5.payload.get("text"), None);
    assert_eq!(
        sha1.payload["display_text"],
        "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d  hello.txt"
    );
    assert_eq!(
        sha256.payload["display_text"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824  hello.txt"
    );
    assert_eq!(
        sha512.payload["display_text"],
        "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043  hello.txt"
    );
}

#[test]
fn file_info_menu_groups_hash_actions() {
    let config = default_config();
    let toolbox = config
        .menu_tree
        .iter()
        .find(|item| item.title == "工具箱")
        .expect("toolbox group should exist");
    let file_info = toolbox
        .children
        .iter()
        .find(|item| item.title == "文件信息")
        .expect("file info group should exist");
    let child_ids = file_info
        .children
        .iter()
        .filter_map(|item| item.action_id.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(
        child_ids,
        [
            "file.info",
            "tool.hash.md5",
            "tool.hash.sha1",
            "tool.hash.sha256",
            "tool.hash.sha512"
        ]
    );
    assert!(!toolbox
        .children
        .iter()
        .any(|item| item.action_id.as_deref() == Some("tool.hash.md5")));
}

#[test]
fn shortcut_to_desktop_creates_unique_symlink() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("shortcut");
    let desktop = root.join("Desktop");
    fs::create_dir_all(&desktop).unwrap();
    std::env::set_var("SRIGHT_FAVORITE_DESKTOP_DIR", &desktop);
    let file = root.join("note.txt");
    fs::write(&file, "hello").unwrap();

    let result = execute_action(request("file.shortcut_desktop", &[file.clone()])).unwrap();

    let shortcut = desktop.join("note.txt");
    assert_eq!(result.payload["created"][0], shortcut.display().to_string());
    assert_eq!(fs::read_link(shortcut).unwrap(), file);

    std::env::remove_var("SRIGHT_FAVORITE_DESKTOP_DIR");
}

#[test]
fn favorite_add_selected_persists_directory_in_config() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_dir("favorite-add-support");
    let root = temp_dir("favorite-add");
    let folder = root.join("Work");
    fs::create_dir_all(&folder).unwrap();
    std::env::set_var("SRIGHT_APP_SUPPORT_DIR", &support_dir);
    let mut config = default_config();
    save_config(&config).unwrap();

    let result =
        execute_configured_action(request("favorite.add_selected", &[folder.clone()]), &config)
            .unwrap();

    assert_eq!(
        result.payload["favorite_dirs"][0]["path"],
        folder.canonicalize().unwrap().display().to_string()
    );
    assert!(apply_action_result_updates(&mut config, &result));
    assert!(config
        .favorite_dirs
        .iter()
        .any(|directory| directory.path == folder.canonicalize().unwrap().display().to_string()));

    std::env::remove_var("SRIGHT_APP_SUPPORT_DIR");
}

#[test]
fn file_visibility_and_extension_actions_use_system_boundaries() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("visibility");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("visible.txt");
    let chflags_log = root.join("chflags.log");
    let extension_log = root.join("extensions.log");
    fs::write(&file, "hello").unwrap();
    std::env::set_var("SRIGHT_CHFLAGS_LOG_FILE", &chflags_log);
    std::env::set_var("SRIGHT_EXTENSION_VISIBILITY_LOG_FILE", &extension_log);

    execute_action(request("visibility.hide_selected", &[file.clone()])).unwrap();
    execute_action(request("visibility.unhide_all", &[file.clone()])).unwrap();
    execute_action(request("finder.hide_extensions", &[file.clone()])).unwrap();
    execute_action(request("finder.show_extensions", &[file])).unwrap();

    let chflags = fs::read_to_string(chflags_log).unwrap();
    assert!(chflags.contains("hidden"));
    assert!(chflags.contains("-R\tnohidden"));
    let extensions = fs::read_to_string(extension_log).unwrap();
    assert!(extensions.contains("hide"));
    assert!(extensions.contains("show"));

    std::env::remove_var("SRIGHT_CHFLAGS_LOG_FILE");
    std::env::remove_var("SRIGHT_EXTENSION_VISIBILITY_LOG_FILE");
}

#[test]
fn qr_file_creates_svg_next_to_selected_file() {
    let root = temp_dir("qr");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("target.txt");
    fs::write(&file, "target").unwrap();

    let result = execute_action(request("tool.qr.file", &[file.clone()])).unwrap();

    let qr = root.join("target.file-qr.svg");
    assert_eq!(result.selected_count, 1);
    assert_eq!(
        result.payload["sources"][0],
        format!("file://{}", file.display())
    );
    assert!(qr.is_file());
    assert!(fs::read_to_string(qr).unwrap().contains("<svg"));
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
