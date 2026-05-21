use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use sright_core::{default_config, execute_action, execute_configured_action, ActionRequest};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn default_config_includes_phase3_menus_templates_apps_and_favorites() {
    let config = default_config();
    let menu_ids = config
        .menus
        .iter()
        .map(|menu| menu.id.as_str())
        .collect::<Vec<_>>();

    for id in [
        "new_file.custom",
        "new_file.text",
        "new_file.rtf",
        "new_file.xml",
        "new_file.word",
        "new_file.excel",
        "new_file.ppt",
        "new_file.wps_writer",
        "new_file.wps_spreadsheet",
        "new_file.wps_presentation",
        "new_file.pages",
        "new_file.numbers",
        "new_file.keynote",
        "new_file.ai",
        "new_file.psd",
        "new_file.markdown",
        "favorite.open.downloads",
        "open.terminal",
        "open.vscode",
        "open.cursor",
        "send.copy_to.downloads",
        "send.move_to.downloads",
    ] {
        assert!(menu_ids.contains(&id), "missing default menu {id}");
    }

    let template_ids = config
        .file_templates
        .iter()
        .map(|template| template.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        template_ids,
        vec![
            "custom",
            "text",
            "rtf",
            "xml",
            "word",
            "excel",
            "ppt",
            "wps_writer",
            "wps_spreadsheet",
            "wps_presentation",
            "pages",
            "numbers",
            "keynote",
            "ai",
            "psd",
            "markdown",
        ]
    );
    assert!(config
        .file_templates
        .iter()
        .any(|template| template.id == "markdown" && template.file_name == "Untitled.md"));
    assert!(config
        .menus
        .iter()
        .filter(|menu| menu.id.starts_with("new_file."))
        .all(|menu| !menu.enabled));
    assert!(config
        .open_apps
        .iter()
        .any(|app| app.id == "vscode" && app.enabled));
    assert!(config
        .favorite_dirs
        .iter()
        .any(|directory| directory.id == "downloads" && directory.enabled));
}

#[test]
fn new_markdown_file_uses_unique_name_in_selected_folder() {
    let root = temp_dir("new-markdown");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("Untitled.md"), "existing").unwrap();

    let result = execute_action(request("new_file.markdown", &[root.clone()])).unwrap();

    let created = root.join("Untitled 2.md");
    assert_eq!(result.selected_count, 1);
    assert!(created.is_file());
    assert_eq!(fs::read_to_string(created).unwrap(), "# Untitled\n");
}

#[test]
fn new_office_file_types_create_files_with_expected_extensions() {
    let root = temp_dir("new-office-types");
    fs::create_dir_all(&root).unwrap();

    for (action_id, expected_file_name) in [
        ("new_file.word", "Untitled.docx"),
        ("new_file.excel", "Untitled.xlsx"),
        ("new_file.ppt", "Untitled.pptx"),
        ("new_file.pages", "Untitled.pages"),
        ("new_file.markdown", "Untitled.md"),
    ] {
        let result = execute_action(request(action_id, &[root.clone()])).unwrap();
        assert_eq!(result.selected_count, 1);
        assert!(
            root.join(expected_file_name).is_file(),
            "{action_id} should create {expected_file_name}"
        );
    }
}

#[test]
fn copy_and_move_to_downloads_use_configurable_destination_boundary() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("send-to");
    let downloads = root.join("Downloads");
    let source = root.join("source.txt");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, "data").unwrap();
    std::env::set_var("SRIGHT_FAVORITE_DOWNLOADS_DIR", &downloads);

    execute_action(request("send.copy_to.downloads", &[source.clone()])).unwrap();

    assert!(source.exists());
    assert_eq!(
        fs::read_to_string(downloads.join("source.txt")).unwrap(),
        "data"
    );

    execute_action(request("send.move_to.downloads", &[source.clone()])).unwrap();

    assert!(!source.exists());
    assert_eq!(
        fs::read_to_string(downloads.join("source 2.txt")).unwrap(),
        "data"
    );

    std::env::remove_var("SRIGHT_FAVORITE_DOWNLOADS_DIR");
}

#[test]
fn configured_favorite_directories_open_copy_and_move_by_id() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("configured-favorite");
    let favorite = root.join("Work");
    let source = root.join("source.txt");
    let open_log = root.join("open.log");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, "data").unwrap();
    std::env::set_var("SRIGHT_OPEN_LOG_FILE", &open_log);

    let mut config = default_config();
    config.favorite_dirs.push(sright_core::FavoriteDirectory {
        id: "work".to_string(),
        title: "Work".to_string(),
        path: favorite.display().to_string(),
        enabled: true,
    });

    execute_configured_action(request("favorite.open.work", &[]), &config).unwrap();
    execute_configured_action(request("send.copy_to.work", &[source.clone()]), &config).unwrap();
    execute_configured_action(request("send.move_to.work", &[source.clone()]), &config).unwrap();

    assert!(fs::read_to_string(open_log)
        .unwrap()
        .contains(&favorite.display().to_string()));
    assert_eq!(
        fs::read_to_string(favorite.join("source.txt")).unwrap(),
        "data"
    );
    assert_eq!(
        fs::read_to_string(favorite.join("source 2.txt")).unwrap(),
        "data"
    );
    assert!(!source.exists());

    std::env::remove_var("SRIGHT_OPEN_LOG_FILE");
}

#[test]
fn open_cursor_writes_test_log_instead_of_launching_app() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let root = temp_dir("open-cursor");
    let file = root.join("main.ts");
    let log = root.join("open.log");
    fs::create_dir_all(&root).unwrap();
    fs::write(&file, "console.log('hello');").unwrap();
    std::env::set_var("SRIGHT_OPEN_LOG_FILE", &log);

    let result = execute_action(request("open.cursor", &[file.clone()])).unwrap();

    let contents = fs::read_to_string(log).unwrap();
    assert_eq!(result.selected_count, 1);
    assert!(contents.contains("cursor"));
    assert!(contents.contains(&file.display().to_string()));

    std::env::remove_var("SRIGHT_OPEN_LOG_FILE");
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
    std::env::temp_dir().join(format!("sright-phase3-{label}-{suffix}"))
}
