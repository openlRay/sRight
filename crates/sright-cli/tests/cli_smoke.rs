use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn cli_config_init_and_print_use_override_dir() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("init");

    let init = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "init"])
        .output()
        .expect("config init should run");
    assert!(
        init.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let print = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "print"])
        .output()
        .expect("config print should run");
    assert!(
        print.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&print.stderr)
    );
    assert!(String::from_utf8_lossy(&print.stdout).contains("copy.path"));
}

#[test]
fn cli_action_run_writes_jsonl_log() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("action");

    let run = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "action",
            "run",
            "--id",
            "copy.path",
            "--path",
            "/tmp/sright.txt",
        ])
        .output()
        .expect("action run should execute");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("\"selected_count\": 1"));

    let tail = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["logs", "tail", "--limit", "5"])
        .output()
        .expect("logs tail should execute");
    assert!(
        tail.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&tail.stderr)
    );
    assert!(String::from_utf8_lossy(&tail.stdout).contains("copy.path"));
}

#[test]
fn cli_blocks_dangerous_actions_until_confirmed() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("dangerous");
    let target = support_dir.join("delete-me.txt");
    std::fs::create_dir_all(&support_dir).unwrap();
    std::fs::write(&target, "bye").unwrap();

    let blocked = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "action",
            "run",
            "--id",
            "file.delete_permanently",
            "--path",
            target.to_str().unwrap(),
        ])
        .output()
        .expect("dangerous action should run and be blocked");
    assert!(!blocked.status.success());
    assert!(target.exists());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("requires confirmation"));

    let confirmed = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "action",
            "run",
            "--id",
            "file.delete_permanently",
            "--confirmed-dangerous",
            "--path",
            target.to_str().unwrap(),
        ])
        .output()
        .expect("confirmed dangerous action should execute");
    assert!(
        confirmed.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
    assert!(!target.exists());
}

#[test]
fn cli_copy_action_writes_clipboard_text() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("clipboard");
    let target = support_dir.join("copy-me.txt");
    let clipboard = support_dir.join("clipboard.txt");
    std::fs::create_dir_all(&support_dir).unwrap();
    std::fs::write(&target, "copy").unwrap();

    let run = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .env("SRIGHT_CLIPBOARD_FILE", &clipboard)
        .args([
            "action",
            "run",
            "--id",
            "copy.name",
            "--path",
            target.to_str().unwrap(),
        ])
        .output()
        .expect("copy action should execute");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(std::fs::read_to_string(clipboard).unwrap(), "copy-me.txt");
    assert!(String::from_utf8_lossy(&run.stdout).contains("\"payload\""));
}

#[test]
fn cli_action_run_can_write_result_file() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("result-file");
    let result_file = support_dir.join("result.json");
    std::fs::create_dir_all(&support_dir).unwrap();

    let run = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "action",
            "run",
            "--id",
            "copy.path",
            "--result-path",
            result_file.to_str().unwrap(),
            "--path",
            "/tmp/result-file.txt",
        ])
        .output()
        .expect("action run should execute");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let contents = std::fs::read_to_string(result_file).unwrap();
    assert!(contents.contains("\"success\": true"));
    assert!(contents.contains("\"action_id\": \"copy.path\""));
}

#[test]
fn cli_exports_imports_config_and_searches_logs() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let support_dir = temp_support_dir("import-export");
    let export_path = support_dir.join("config-export.json");
    let log_export_path = support_dir.join("logs-export.jsonl");

    let init = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "init"])
        .output()
        .expect("config init should run");
    assert!(init.status.success());

    let export = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "export", "--path", export_path.to_str().unwrap()])
        .output()
        .expect("config export should run");
    assert!(
        export.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(export_path.exists());

    let disabled = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "set-enabled", "false"])
        .output()
        .expect("config set-enabled should run");
    assert!(disabled.status.success());

    let import = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["config", "import", "--path", export_path.to_str().unwrap()])
        .output()
        .expect("config import should run");
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let run = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "action",
            "run",
            "--id",
            "copy.path",
            "--path",
            "/tmp/log-me",
        ])
        .output()
        .expect("debug action should run");
    assert!(run.status.success());

    let search = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args(["logs", "search", "copy.path"])
        .output()
        .expect("logs search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("copy.path"));

    let export_logs = sright_cli()
        .env("SRIGHT_APP_SUPPORT_DIR", &support_dir)
        .args([
            "logs",
            "export",
            "--path",
            log_export_path.to_str().unwrap(),
        ])
        .output()
        .expect("logs export should run");
    assert!(export_logs.status.success());
    assert!(log_export_path.exists());
}

fn sright_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sright-cli"))
}

fn temp_support_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sright-cli-{label}-{suffix}"))
}
