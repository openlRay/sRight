use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use sright_core::{
    append_action_log, apply_action_result_updates, default_config, execute_configured_action,
    load_or_init_config, read_recent_logs, save_config, ActionLogEntry, ActionRequest,
};

#[derive(Debug, Parser)]
#[command(name = "sright-cli", version, about = "sRight Finder action runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Action {
        #[command(subcommand)]
        command: ActionCommand,
    },
    Logs {
        #[command(subcommand)]
        command: LogsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Init,
    Print,
    SetEnabled {
        enabled: String,
    },
    Export {
        #[arg(long)]
        path: PathBuf,
    },
    Import {
        #[arg(long)]
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum ActionCommand {
    Run(ActionRunArgs),
}

#[derive(Debug, Args)]
struct ActionRunArgs {
    #[arg(long = "id")]
    action_id: String,
    #[arg(long = "path")]
    paths: Vec<PathBuf>,
    #[arg(long)]
    confirmed_dangerous: bool,
    #[arg(long)]
    result_path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum LogsCommand {
    Tail {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Export {
        #[arg(long)]
        path: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Config { command } => run_config(command),
        Command::Action { command } => run_action(command),
        Command::Logs { command } => run_logs(command),
    }
}

fn run_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Init => {
            let config = load_or_init_config().context("could not initialize config")?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Print => {
            let config = load_or_init_config().context("could not load config")?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::SetEnabled { enabled } => {
            let mut config = load_or_init_config().context("could not load config")?;
            let enabled = enabled
                .parse::<bool>()
                .context("enabled must be true or false")?;
            config.enabled = enabled;
            save_config(&config).context("could not save config")?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigCommand::Export { path } => {
            let config = load_or_init_config().context("could not load config")?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("could not create {}", parent.display()))?;
            }
            std::fs::write(
                &path,
                format!("{}\n", serde_json::to_string_pretty(&config)?),
            )
            .with_context(|| format!("could not write {}", path.display()))?;
            println!("{}", path.display());
        }
        ConfigCommand::Import { path } => {
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("could not read {}", path.display()))?;
            let config = serde_json::from_str(&contents).context("could not parse config")?;
            save_config(&config).context("could not save imported config")?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
    }

    Ok(())
}

fn run_action(command: ActionCommand) -> Result<()> {
    match command {
        ActionCommand::Run(args) => run_action_request(args),
    }
}

fn run_action_request(args: ActionRunArgs) -> Result<()> {
    let result_path = args.result_path.clone();
    let selected_count = args.paths.len();
    let mut config = load_or_init_config().context("could not load config")?;
    if !config.enabled {
        let message = "sRight is disabled in config";
        append_action_log(&ActionLogEntry::failure(
            &args.action_id,
            selected_count,
            message,
            message,
        ))?;
        write_action_result_file(
            result_path.as_deref(),
            false,
            &args.action_id,
            selected_count,
            None,
            Some(message),
        )?;
        bail!(message);
    }

    let Some(menu) = config.menus.iter().find(|menu| menu.id == args.action_id) else {
        let message = format!("action is not present in config: {}", args.action_id);
        append_action_log(&ActionLogEntry::failure(
            &args.action_id,
            selected_count,
            "Action blocked before execution",
            &message,
        ))?;
        write_action_result_file(
            result_path.as_deref(),
            false,
            &args.action_id,
            selected_count,
            None,
            Some(&message),
        )?;
        bail!(message);
    };

    if !menu.enabled {
        let message = format!("action is disabled in config: {}", args.action_id);
        append_action_log(&ActionLogEntry::failure(
            &args.action_id,
            selected_count,
            "Action blocked before execution",
            &message,
        ))?;
        write_action_result_file(
            result_path.as_deref(),
            false,
            &args.action_id,
            selected_count,
            None,
            Some(&message),
        )?;
        bail!(message);
    }

    let confirmed_dangerous = args.confirmed_dangerous
        || !config
            .dangerous_confirmation
            .requires_confirmation(&args.action_id);

    match execute_configured_action(
        ActionRequest {
            action_id: args.action_id.clone(),
            paths: args.paths,
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
                if let Err(error) = write_cut_clipboard(&paths) {
                    append_action_log(&ActionLogEntry::failure(
                        &result.action_id,
                        result.selected_count,
                        "Action failed",
                        format!("could not write cut files to clipboard: {error:#}"),
                    ))?;
                    return Err(error).context("could not write cut files to clipboard");
                }
            } else if let Some(text) = result.payload.get("text").and_then(|value| value.as_str()) {
                if let Err(error) = write_clipboard_text(text) {
                    append_action_log(&ActionLogEntry::failure(
                        &result.action_id,
                        result.selected_count,
                        "Action failed",
                        format!("could not write action text to clipboard: {error:#}"),
                    ))?;
                    return Err(error).context("could not write action text to clipboard");
                }
            }
            if apply_action_result_updates(&mut config, &result) {
                save_config(&config).context("could not save config updates from action")?;
            }

            append_action_log(&ActionLogEntry::success(
                &result.action_id,
                result.selected_count,
                &result.message,
            ))?;
            let response = serde_json::json!({
                "action_id": result.action_id,
                "selected_count": result.selected_count,
                "message": result.message,
                "payload": result.payload
            });
            write_action_result_file(
                result_path.as_deref(),
                true,
                &args.action_id,
                response["selected_count"].as_u64().unwrap_or_default() as usize,
                Some(&response),
                None,
            )?;
            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Err(error) => {
            append_action_log(&ActionLogEntry::failure(
                &args.action_id,
                selected_count,
                "Action failed",
                error.to_string(),
            ))?;
            write_action_result_file(
                result_path.as_deref(),
                false,
                &args.action_id,
                selected_count,
                None,
                Some(&error.to_string()),
            )?;
            Err(error).context("could not execute action")
        }
    }
}

fn run_logs(command: LogsCommand) -> Result<()> {
    match command {
        LogsCommand::Tail { limit } => {
            for entry in read_recent_logs(limit).context("could not read logs")? {
                println!("{}", serde_json::to_string(&entry)?);
            }
        }
        LogsCommand::Search { query, limit } => {
            let query = query.to_lowercase();
            for entry in read_recent_logs(limit).context("could not read logs")? {
                let haystack = format!(
                    "{} {} {}",
                    entry.action_id,
                    entry.message,
                    entry.error.clone().unwrap_or_default()
                )
                .to_lowercase();
                if haystack.contains(&query) {
                    println!("{}", serde_json::to_string(&entry)?);
                }
            }
        }
        LogsCommand::Export { path } => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("could not create {}", parent.display()))?;
            }
            std::fs::copy(sright_core::paths::log_path(), &path)
                .with_context(|| format!("could not write {}", path.display()))?;
            println!("{}", path.display());
        }
    }

    Ok(())
}

fn write_cut_clipboard(paths: &[String]) -> Result<()> {
    if let Ok(path) = std::env::var("SRIGHT_CLIPBOARD_FILE") {
        std::fs::write(Path::new(&path), format!("cut\n{}\n", paths.join("\n")))
            .with_context(|| format!("could not write {path}"))?;
        return Ok(());
    }

    let script = r#"
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
"#;
    let status = ProcessCommand::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", script])
        .args(paths)
        .status()
        .context("could not start osascript")?;
    if !status.success() {
        bail!("osascript exited with status {status}");
    }

    Ok(())
}

fn write_clipboard_text(text: &str) -> Result<()> {
    if let Ok(path) = std::env::var("SRIGHT_CLIPBOARD_FILE") {
        std::fs::write(Path::new(&path), text)
            .with_context(|| format!("could not write {path}"))?;
        return Ok(());
    }

    let mut child = ProcessCommand::new("/usr/bin/pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("could not start pbcopy")?;

    let mut stdin = child.stdin.take().context("could not open pbcopy stdin")?;
    stdin
        .write_all(text.as_bytes())
        .context("could not send text to pbcopy")?;
    drop(stdin);

    let status = child.wait().context("could not wait for pbcopy")?;
    if !status.success() {
        bail!("pbcopy exited with status {status}");
    }

    Ok(())
}

fn write_action_result_file(
    path: Option<&Path>,
    success: bool,
    action_id: &str,
    selected_count: usize,
    response: Option<&serde_json::Value>,
    error: Option<&str>,
) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }

    let payload = serde_json::json!({
        "success": success,
        "action_id": action_id,
        "selected_count": selected_count,
        "response": response.cloned(),
        "error": error
    });
    std::fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&payload)?),
    )
    .with_context(|| format!("could not write {}", path.display()))?;
    Ok(())
}

#[allow(dead_code)]
fn _ensure_default_config_is_linked() {
    let _ = default_config();
}
