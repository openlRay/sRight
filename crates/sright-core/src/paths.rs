use std::env;
use std::path::PathBuf;

const APP_SUPPORT_OVERRIDE: &str = "SRIGHT_APP_SUPPORT_DIR";
const FINDER_SYNC_BUNDLE_ID: &str = "dev.sright.preferences.findersync";

pub fn app_support_dir() -> PathBuf {
    if let Ok(path) = env::var(APP_SUPPORT_OVERRIDE) {
        return PathBuf::from(path);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("sRight")
}

pub fn config_path() -> PathBuf {
    app_support_dir().join("config.json")
}

pub fn log_path() -> PathBuf {
    app_support_dir().join("actions.jsonl")
}

pub fn finder_sync_app_support_dir() -> Option<PathBuf> {
    if env::var(APP_SUPPORT_OVERRIDE).is_ok() {
        return None;
    }

    let home = env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Containers")
            .join(FINDER_SYNC_BUNDLE_ID)
            .join("Data")
            .join("Library")
            .join("Application Support")
            .join("sRight"),
    )
}
