use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use md5::Md5;
use qrcode::{render::svg, QrCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::write::FileOptions;

use crate::config::{FavoriteDirectory, SRightConfig};
use crate::logging::read_recent_logs;
use crate::paths::log_path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action_id: String,
    pub paths: Vec<PathBuf>,
    #[serde(default)]
    pub confirmed_dangerous: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: String,
    pub selected_count: usize,
    pub message: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionDescriptor {
    pub id: String,
    pub title: String,
    pub dangerous: bool,
    pub selection: ActionSelection,
    pub default_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionSelection {
    Single,
    Multiple,
    Any,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("unknown action: {0}")]
    UnknownAction(String),
    #[error("action requires confirmation: {0}")]
    RequiresConfirmation(String),
    #[error("action requires at least one selected path: {0}")]
    EmptySelection(String),
    #[error("selected path is not a folder: {0}")]
    NotAFolder(String),
    #[error("selected path has no file name: {0}")]
    MissingFileName(String),
    #[error("target already exists: {0}")]
    TargetExists(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("archive operation failed for {path}: {source}")]
    Archive {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("walk operation failed for {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: walkdir::Error,
    },
    #[error("qr operation failed: {0}")]
    Qr(String),
    #[error("file operation failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type ActionExecutionResult<T> = Result<T, ActionError>;

pub fn action_descriptors() -> Vec<ActionDescriptor> {
    vec![
        descriptor("debug.echo", "sRight Debug Echo", false),
        descriptor("copy.path", "复制完整路径", false),
        descriptor("copy.real_path", "复制真实路径", false),
        descriptor("copy.name", "复制名称", false),
        descriptor("copy.parent_path", "复制父目录路径", false),
        descriptor("copy.shell_escaped_path", "复制 Shell Escaped Path", false),
        descriptor("file.move_to_trash", "移到废纸篓", true),
        descriptor("file.delete_permanently", "彻底删除", true),
        descriptor("folder.create_from_filename", "根据文件名创建文件夹", false),
        descriptor("folder.dissolve", "解散文件夹", false),
        descriptor("file.info", "查看文件信息", false),
        descriptor("new_file.custom", "自定义创建新文件", false),
        descriptor("new_file.text", "TXT", false),
        descriptor("new_file.rtf", "RTF", false),
        descriptor("new_file.xml", "XML", false),
        descriptor("new_file.word", "Word", false),
        descriptor("new_file.excel", "Excel", false),
        descriptor("new_file.ppt", "PPT", false),
        descriptor("new_file.wps_writer", "WPS 文字", false),
        descriptor("new_file.wps_spreadsheet", "WPS 表格", false),
        descriptor("new_file.wps_presentation", "WPS 演示", false),
        descriptor("new_file.pages", "Pages", false),
        descriptor("new_file.numbers", "Numbers", false),
        descriptor("new_file.keynote", "Keynote", false),
        descriptor("new_file.ai", "Ai", false),
        descriptor("new_file.psd", "PSD", false),
        descriptor("new_file.markdown", "Markdown", false),
        descriptor("new_file.json", "新建 JSON 文件", false),
        descriptor("open.terminal", "在 Terminal 中打开", false),
        descriptor("open.vscode", "在 VSCode 中打开", false),
        descriptor("open.cursor", "在 Cursor 中打开", false),
        descriptor("favorite.open.downloads", "打开下载", false),
        descriptor("favorite.open.pictures", "打开图片", false),
        descriptor("favorite.open.music", "打开音乐", false),
        descriptor("favorite.open.movies", "打开影片", false),
        descriptor("favorite.open.desktop", "打开桌面", false),
        descriptor("favorite.open.documents", "打开文稿", false),
        descriptor("send.copy_to.downloads", "复制到下载", false),
        descriptor("send.move_to.downloads", "移动到下载", false),
        descriptor("send.copy_to.pictures", "复制到图片", false),
        descriptor("send.move_to.pictures", "移动到图片", false),
        descriptor("send.copy_to.music", "复制到音乐", false),
        descriptor("send.move_to.music", "移动到音乐", false),
        descriptor("send.copy_to.movies", "复制到影片", false),
        descriptor("send.move_to.movies", "移动到影片", false),
        descriptor("send.copy_to.desktop", "复制到桌面", false),
        descriptor("send.move_to.desktop", "移动到桌面", false),
        descriptor("send.copy_to.documents", "复制到文稿", false),
        descriptor("send.move_to.documents", "移动到文稿", false),
        descriptor("archive.zip", "压缩为 ZIP", false),
        descriptor("archive.zip_each", "每项单独压缩为 ZIP", false),
        descriptor("archive.unzip_here", "解压到当前目录", false),
        descriptor("archive.unzip_to_folder", "解压到单独目录", false),
        descriptor("image.convert.png", "图片转 PNG", false),
        descriptor("image.convert.jpg", "图片转 JPG", false),
        descriptor("image.convert.webp", "图片转 WebP", false),
        descriptor("image.convert.heic", "图片转 HEIC", false),
        descriptor("icon.make_iconset", "生成 iconset", false),
        descriptor("icon.make_icns", "生成 ICNS", false),
        descriptor("icon.set_custom", "设置自定义图标", false),
        descriptor("icon.remove_custom", "删除自定义图标", false),
        descriptor("tool.hash.md5", "计算 MD5", false),
        descriptor("tool.hash.sha1", "计算 SHA1", false),
        descriptor("tool.hash.sha256", "计算 SHA256", false),
        descriptor("tool.qr.path", "文件路径生成二维码", false),
        descriptor("tool.open_parent", "打开所在目录", false),
        descriptor("tool.copy_summary", "复制文件摘要信息", false),
        descriptor("script.run.default", "运行默认脚本动作", false),
        descriptor("logs.search", "搜索动作日志", false),
        descriptor("logs.export", "导出动作日志", false),
    ]
}

pub fn execute_action(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    execute_action_with_favorite_dirs(request, &[])
}

pub fn execute_configured_action(
    request: ActionRequest,
    config: &SRightConfig,
) -> ActionExecutionResult<ActionResult> {
    execute_action_with_favorite_dirs(request, &config.favorite_dirs)
}

fn execute_action_with_favorite_dirs(
    request: ActionRequest,
    favorite_dirs: &[FavoriteDirectory],
) -> ActionExecutionResult<ActionResult> {
    let descriptor = action_descriptor_for(&request.action_id, favorite_dirs)
        .ok_or_else(|| ActionError::UnknownAction(request.action_id.clone()))?;

    if descriptor.dangerous && !request.confirmed_dangerous {
        return Err(ActionError::RequiresConfirmation(request.action_id));
    }

    if let Some(favorite_id) = request
        .action_id
        .strip_prefix("favorite.open.")
        .map(str::to_string)
    {
        return open_favorite(request, &favorite_id, favorite_dirs);
    }

    if let Some(favorite_id) = request
        .action_id
        .strip_prefix("send.copy_to.")
        .map(str::to_string)
    {
        return send_to_favorite(request, &favorite_id, SendMode::Copy, favorite_dirs);
    }

    if let Some(favorite_id) = request
        .action_id
        .strip_prefix("send.move_to.")
        .map(str::to_string)
    {
        return send_to_favorite(request, &favorite_id, SendMode::Move, favorite_dirs);
    }

    match request.action_id.as_str() {
        "debug.echo" => Ok(ActionResult {
            action_id: request.action_id,
            selected_count: request.paths.len(),
            message: debug_echo_message(&request.paths),
            payload: json!({ "paths": request.paths }),
        }),
        "copy.path" => copy_text_result(request, "Copied path", |path| {
            Ok(path.display().to_string())
        }),
        "copy.real_path" => copy_text_result(request, "Copied real path", |path| {
            path.canonicalize()
                .map(|path| path.display().to_string())
                .map_err(|source| ActionError::Io {
                    path: path.to_path_buf(),
                    source,
                })
        }),
        "copy.name" => copy_text_result(request, "Copied name", |path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))
        }),
        "copy.parent_path" => copy_text_result(request, "Copied parent path", |path| {
            path.parent()
                .map(|parent| parent.display().to_string())
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))
        }),
        "copy.shell_escaped_path" => {
            copy_text_result(request, "Copied shell escaped path", |path| {
                Ok(shell_escape(&path.display().to_string()))
            })
        }
        "file.info" => file_info(request),
        "new_file.custom" => create_new_file(request, "Untitled", ""),
        "new_file.text" => create_new_file(request, "Untitled.txt", ""),
        "new_file.rtf" => create_new_file(request, "Untitled.rtf", "{\\rtf1\\ansi\\deff0\n}\n"),
        "new_file.xml" => create_new_file(
            request,
            "Untitled.xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        ),
        "new_file.word" => create_new_file(request, "Untitled.docx", ""),
        "new_file.excel" => create_new_file(request, "Untitled.xlsx", ""),
        "new_file.ppt" => create_new_file(request, "Untitled.pptx", ""),
        "new_file.wps_writer" => create_new_file(request, "Untitled.wps", ""),
        "new_file.wps_spreadsheet" => create_new_file(request, "Untitled.et", ""),
        "new_file.wps_presentation" => create_new_file(request, "Untitled.dps", ""),
        "new_file.pages" => create_new_file(request, "Untitled.pages", ""),
        "new_file.numbers" => create_new_file(request, "Untitled.numbers", ""),
        "new_file.keynote" => create_new_file(request, "Untitled.key", ""),
        "new_file.ai" => create_new_file(request, "Untitled.ai", ""),
        "new_file.psd" => create_new_file(request, "Untitled.psd", ""),
        "new_file.markdown" => create_new_file(request, "Untitled.md", "# Untitled\n"),
        "new_file.json" => create_new_file(request, "Untitled.json", "{}\n"),
        "open.terminal" => open_with_app(request, "terminal", "com.apple.Terminal", true),
        "open.vscode" => open_with_app(request, "vscode", "com.microsoft.VSCode", false),
        "open.cursor" => open_with_app(request, "cursor", "com.todesktop.230313mzl4w4u92", false),
        "archive.zip" => zip_selection(request),
        "archive.zip_each" => zip_each(request),
        "archive.unzip_here" => unzip_selection(request, UnzipMode::Here),
        "archive.unzip_to_folder" => unzip_selection(request, UnzipMode::Folder),
        "image.convert.png" => convert_image(request, "png"),
        "image.convert.jpg" => convert_image(request, "jpeg"),
        "image.convert.webp" => convert_image(request, "webp"),
        "image.convert.heic" => convert_image(request, "heic"),
        "icon.make_iconset" => icon_tool_action(request, "make_iconset"),
        "icon.make_icns" => icon_tool_action(request, "make_icns"),
        "icon.set_custom" => icon_tool_action(request, "set_custom"),
        "icon.remove_custom" => icon_tool_action(request, "remove_custom"),
        "tool.hash.md5" => hash_files::<Md5>(request, "md5"),
        "tool.hash.sha1" => hash_files::<Sha1>(request, "sha1"),
        "tool.hash.sha256" => hash_files::<Sha256>(request, "sha256"),
        "tool.qr.path" => qr_for_paths(request),
        "tool.open_parent" => open_parent(request),
        "tool.copy_summary" => copy_summary(request),
        "script.run.default" => run_script_action(request),
        "logs.search" => search_logs(request),
        "logs.export" => export_logs(request),
        "folder.create_from_filename" => create_folders_from_filename(request),
        "folder.dissolve" => dissolve_folders(request),
        "file.delete_permanently" => delete_permanently(request),
        "file.move_to_trash" => move_to_trash(request),
        action_id => Err(ActionError::UnknownAction(action_id.to_string())),
    }
}

fn action_descriptor_for(
    action_id: &str,
    favorite_dirs: &[FavoriteDirectory],
) -> Option<ActionDescriptor> {
    if let Some(descriptor) = action_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.id == action_id)
    {
        return Some(descriptor);
    }

    for (prefix, verb) in [
        ("favorite.open.", "打开"),
        ("send.copy_to.", "复制到"),
        ("send.move_to.", "移动到"),
    ] {
        let Some(favorite_id) = action_id.strip_prefix(prefix) else {
            continue;
        };
        let Some(directory) = favorite_dirs
            .iter()
            .find(|directory| directory.id == favorite_id)
        else {
            continue;
        };
        return Some(descriptor(
            action_id,
            &format!("{verb}{}", directory.title),
            false,
        ));
    }

    None
}

fn descriptor(id: &str, title: &str, dangerous: bool) -> ActionDescriptor {
    ActionDescriptor {
        id: id.to_string(),
        title: title.to_string(),
        dangerous,
        selection: ActionSelection::Any,
        default_enabled: true,
    }
}

fn debug_echo_message(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "Debug echo received no selected paths".to_string();
    }

    let preview = paths
        .iter()
        .take(3)
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    if paths.len() > 3 {
        format!(
            "Debug echo received {} selected paths: {preview}, ...",
            paths.len()
        )
    } else {
        format!(
            "Debug echo received {} selected paths: {preview}",
            paths.len()
        )
    }
}

fn copy_text_result(
    request: ActionRequest,
    message: &str,
    format_path: impl Fn(&Path) -> ActionExecutionResult<String>,
) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let text = request
        .paths
        .iter()
        .map(|path| format_path(path))
        .collect::<ActionExecutionResult<Vec<_>>>()?
        .join("\n");

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("{message}: {text}"),
        payload: json!({ "text": text }),
    })
}

fn file_info(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let items = request
        .paths
        .iter()
        .map(|path| {
            let metadata = fs::metadata(path).map_err(|source| ActionError::Io {
                path: path.to_path_buf(),
                source,
            })?;
            Ok(json!({
                "path": path.display().to_string(),
                "real_path": path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).display().to_string(),
                "name": path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default(),
                "is_file": metadata.is_file(),
                "is_dir": metadata.is_dir(),
                "len": metadata.len(),
                "readonly": metadata.permissions().readonly()
            }))
        })
        .collect::<ActionExecutionResult<Vec<_>>>()?;

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Collected file info for {} item(s)", request.paths.len()),
        payload: json!({ "items": items }),
    })
}

fn create_new_file(
    request: ActionRequest,
    file_name: &str,
    contents: &str,
) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let directory = selected_output_dir(&request.paths)?;
    let target = unique_name_target(&directory, Path::new(file_name));
    fs::write(&target, contents).map_err(|source| ActionError::Io {
        path: target.clone(),
        source,
    })?;

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Created file {}", target.display()),
        payload: json!({ "created": [target.display().to_string()] }),
    })
}

fn open_with_app(
    request: ActionRequest,
    app_id: &str,
    bundle_id: &str,
    use_parent_for_files: bool,
) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let targets = request
        .paths
        .iter()
        .map(|path| {
            if use_parent_for_files && path.is_file() {
                path.parent()
                    .map(Path::to_path_buf)
                    .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))
            } else {
                Ok(path.clone())
            }
        })
        .collect::<ActionExecutionResult<Vec<_>>>()?;

    if let Some(log_path) = env::var_os("SRIGHT_OPEN_LOG_FILE").map(PathBuf::from) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|source| ActionError::Io {
                path: log_path.clone(),
                source,
            })?;
        writeln!(
            file,
            "{}\t{}\t{}",
            app_id,
            bundle_id,
            targets
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\t")
        )
        .map_err(|source| ActionError::Io {
            path: log_path,
            source,
        })?;
    } else {
        let status = Command::new("/usr/bin/open")
            .arg("-b")
            .arg(bundle_id)
            .args(&targets)
            .status()
            .map_err(|source| ActionError::Io {
                path: PathBuf::from("/usr/bin/open"),
                source,
            })?;
        if !status.success() {
            return Err(ActionError::CommandFailed(format!(
                "open -b {bundle_id} exited with status {status}"
            )));
        }
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Opened {} item(s) with {app_id}", targets.len()),
        payload: json!({
            "app_id": app_id,
            "bundle_id": bundle_id,
            "targets": targets.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()
        }),
    })
}

#[derive(Debug, Clone, Copy)]
enum SendMode {
    Copy,
    Move,
}

fn send_to_favorite(
    request: ActionRequest,
    favorite_id: &str,
    mode: SendMode,
    favorite_dirs: &[FavoriteDirectory],
) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let destination_dir = favorite_directory(favorite_id, favorite_dirs);
    fs::create_dir_all(&destination_dir).map_err(|source| ActionError::Io {
        path: destination_dir.clone(),
        source,
    })?;

    let mut sent = Vec::new();
    for source in &request.paths {
        let file_name = source
            .file_name()
            .ok_or_else(|| ActionError::MissingFileName(source.display().to_string()))?;
        let target = unique_name_target(&destination_dir, Path::new(file_name));
        match mode {
            SendMode::Copy => copy_item(source, &target)?,
            SendMode::Move => {
                fs::rename(source, &target).map_err(|source_error| ActionError::Io {
                    path: source.clone(),
                    source: source_error,
                })?
            }
        }
        sent.push(target.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Sent {} item(s) to {favorite_id}", sent.len()),
        payload: json!({ "targets": sent }),
    })
}

fn open_favorite(
    request: ActionRequest,
    favorite_id: &str,
    favorite_dirs: &[FavoriteDirectory],
) -> ActionExecutionResult<ActionResult> {
    let directory = favorite_directory(favorite_id, favorite_dirs);
    fs::create_dir_all(&directory).map_err(|source| ActionError::Io {
        path: directory.clone(),
        source,
    })?;
    open_with_app(
        ActionRequest {
            paths: vec![directory],
            ..request
        },
        "finder",
        "com.apple.finder",
        false,
    )
}

fn zip_selection(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let parent = common_parent(&request.paths)?;
    let archive_name = if request.paths.len() == 1 {
        format!(
            "{}.zip",
            request.paths[0]
                .file_stem()
                .unwrap_or_else(|| request.paths[0].as_os_str())
                .to_string_lossy()
        )
    } else {
        "Archive.zip".to_string()
    };
    let archive_path = unique_name_target(&parent, Path::new(&archive_name));
    write_zip(&archive_path, &request.paths, &parent)?;

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Created archive {}", archive_path.display()),
        payload: json!({ "archives": [archive_path.display().to_string()] }),
    })
}

fn zip_each(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut archives = Vec::new();
    for path in &request.paths {
        let parent = path
            .parent()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let archive_name = format!(
            "{}.zip",
            path.file_stem()
                .unwrap_or_else(|| path.as_os_str())
                .to_string_lossy()
        );
        let archive_path = unique_name_target(parent, Path::new(&archive_name));
        write_zip(&archive_path, &[path.clone()], parent)?;
        archives.push(archive_path.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Created {} archive(s)", archives.len()),
        payload: json!({ "archives": archives }),
    })
}

#[derive(Debug, Clone, Copy)]
enum UnzipMode {
    Here,
    Folder,
}

fn unzip_selection(request: ActionRequest, mode: UnzipMode) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut extracted = Vec::new();
    for archive in &request.paths {
        let parent = archive
            .parent()
            .ok_or_else(|| ActionError::MissingFileName(archive.display().to_string()))?;
        let output_dir = match mode {
            UnzipMode::Here => parent.to_path_buf(),
            UnzipMode::Folder => {
                let stem = archive
                    .file_stem()
                    .ok_or_else(|| ActionError::MissingFileName(archive.display().to_string()))?;
                let target = unique_name_target(parent, Path::new(stem));
                fs::create_dir(&target).map_err(|source| ActionError::Io {
                    path: target.clone(),
                    source,
                })?;
                target
            }
        };
        extract_zip(archive, &output_dir)?;
        extracted.push(output_dir.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Extracted {} archive(s)", request.paths.len()),
        payload: json!({ "directories": extracted }),
    })
}

fn convert_image(request: ActionRequest, format: &str) -> ActionExecutionResult<ActionResult> {
    run_tool_or_log(
        request,
        "SRIGHT_IMAGE_LOG_FILE",
        "image",
        |path| {
            let parent = path
                .parent()
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
            let stem = path
                .file_stem()
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
            Ok(unique_name_target(
                parent,
                Path::new(&format!("{}.{}", stem.to_string_lossy(), format)),
            ))
        },
        |source, target| {
            let status = Command::new("/usr/bin/sips")
                .args(["-s", "format", format])
                .arg(source)
                .args(["--out"])
                .arg(target)
                .status()
                .map_err(|error| ActionError::Io {
                    path: PathBuf::from("/usr/bin/sips"),
                    source: error,
                })?;
            if status.success() {
                Ok(())
            } else {
                Err(ActionError::CommandFailed(format!(
                    "sips convert to {format} exited with status {status}"
                )))
            }
        },
    )
}

fn icon_tool_action(request: ActionRequest, tool_id: &str) -> ActionExecutionResult<ActionResult> {
    run_tool_or_log(
        request,
        "SRIGHT_ICON_LOG_FILE",
        tool_id,
        |path| {
            let parent = path
                .parent()
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
            let stem = path
                .file_stem()
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
            Ok(unique_name_target(
                parent,
                Path::new(&format!("{}.{}", stem.to_string_lossy(), tool_id)),
            ))
        },
        |source, target| {
            fs::write(target, source.display().to_string()).map_err(|error| ActionError::Io {
                path: target.to_path_buf(),
                source: error,
            })
        },
    )
}

fn hash_files<D>(request: ActionRequest, algorithm: &str) -> ActionExecutionResult<ActionResult>
where
    D: Digest + Default,
{
    require_selection(&request)?;
    let lines = request
        .paths
        .iter()
        .map(|path| {
            let digest = hash_file::<D>(path)?;
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            Ok(format!("{digest}  {name}"))
        })
        .collect::<ActionExecutionResult<Vec<_>>>()?;
    let text = lines.join("\n");

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Calculated {algorithm} for {} item(s)", request.paths.len()),
        payload: json!({ "text": text }),
    })
}

fn qr_for_paths(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut created = Vec::new();
    for path in &request.paths {
        let parent = path
            .parent()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let stem = path
            .file_stem()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let target = unique_name_target(
            parent,
            Path::new(&format!("{}.path-qr.svg", stem.to_string_lossy())),
        );
        let code = QrCode::new(path.display().to_string())
            .map_err(|error| ActionError::Qr(error.to_string()))?;
        let image = code.render::<svg::Color>().min_dimensions(256, 256).build();
        fs::write(&target, image).map_err(|source| ActionError::Io {
            path: target.clone(),
            source,
        })?;
        created.push(target.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Created {} QR file(s)", created.len()),
        payload: json!({ "created": created }),
    })
}

fn open_parent(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    let parent_paths = request
        .paths
        .iter()
        .map(|path| {
            path.parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))
        })
        .collect::<ActionExecutionResult<Vec<_>>>()?;
    open_with_app(
        ActionRequest {
            paths: parent_paths,
            ..request
        },
        "finder",
        "com.apple.finder",
        false,
    )
}

fn copy_summary(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let text = request
        .paths
        .iter()
        .map(|path| {
            let metadata = fs::metadata(path).map_err(|source| ActionError::Io {
                path: path.clone(),
                source,
            })?;
            Ok(format!(
                "{}\t{}\t{} bytes",
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string()),
                if metadata.is_dir() { "folder" } else { "file" },
                metadata.len()
            ))
        })
        .collect::<ActionExecutionResult<Vec<_>>>()?
        .join("\n");

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Copied summary for {} item(s)", request.paths.len()),
        payload: json!({ "text": text }),
    })
}

fn run_script_action(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    if let Some(log_path) = env::var_os("SRIGHT_SCRIPT_LOG_FILE").map(PathBuf::from) {
        append_tool_log(&log_path, "script.run.default", &request.paths)?;
    } else if let Ok(script) = env::var("SRIGHT_DEFAULT_SCRIPT") {
        let status = Command::new(script)
            .args(&request.paths)
            .status()
            .map_err(|source| ActionError::Io {
                path: PathBuf::from("SRIGHT_DEFAULT_SCRIPT"),
                source,
            })?;
        if !status.success() {
            return Err(ActionError::CommandFailed(format!(
                "default script exited with status {status}"
            )));
        }
    } else {
        return Err(ActionError::CommandFailed(
            "SRIGHT_DEFAULT_SCRIPT is not configured".to_string(),
        ));
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Ran script for {} item(s)", request.paths.len()),
        payload: json!({ "paths": request.paths }),
    })
}

fn search_logs(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    let query = request
        .paths
        .first()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let query_lower = query.to_lowercase();
    let matches = read_recent_logs(500)
        .map_err(|error| ActionError::CommandFailed(error.to_string()))?
        .into_iter()
        .filter(|entry| {
            let haystack = format!(
                "{} {} {}",
                entry.action_id,
                entry.message,
                entry.error.clone().unwrap_or_default()
            )
            .to_lowercase();
            haystack.contains(&query_lower)
        })
        .collect::<Vec<_>>();

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Found {} log match(es)", matches.len()),
        payload: json!({ "matches": matches }),
    })
}

fn export_logs(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    let target = env::var_os("SRIGHT_LOG_EXPORT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| unique_name_target(&env::temp_dir(), Path::new("sright-actions.jsonl")));
    let source = log_path();
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| ActionError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(&source, &target).map_err(|source_error| ActionError::Io {
        path: source,
        source: source_error,
    })?;

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Exported logs to {}", target.display()),
        payload: json!({ "path": target.display().to_string() }),
    })
}

fn create_folders_from_filename(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut created = Vec::new();
    let mut copied = Vec::new();

    for path in &request.paths {
        let file_name = path
            .file_name()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let stem = path
            .file_stem()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let parent = path
            .parent()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let target = parent.join(stem);
        if target.exists() {
            return Err(ActionError::TargetExists(target.display().to_string()));
        }
        fs::create_dir(&target).map_err(|source| ActionError::Io {
            path: target.clone(),
            source,
        })?;
        let destination = target.join(file_name);
        fs::copy(path, &destination).map_err(|source| ActionError::Io {
            path: path.clone(),
            source,
        })?;
        created.push(target.display().to_string());
        copied.push(destination.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Created {} folder(s)", created.len()),
        payload: json!({ "created": created, "copied": copied }),
    })
}

fn dissolve_folders(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut moves = Vec::new();

    for folder in &request.paths {
        if !folder.is_dir() {
            return Err(ActionError::NotAFolder(folder.display().to_string()));
        }
        let parent = folder
            .parent()
            .ok_or_else(|| ActionError::MissingFileName(folder.display().to_string()))?;

        for entry in fs::read_dir(folder).map_err(|source| ActionError::Io {
            path: folder.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| ActionError::Io {
                path: folder.clone(),
                source,
            })?;
            let target = parent.join(entry.file_name());
            if target.exists() {
                return Err(ActionError::TargetExists(target.display().to_string()));
            }
            moves.push((entry.path(), target));
        }
    }

    let mut moved = Vec::new();
    for (source, target) in moves {
        fs::rename(&source, &target).map_err(|error| ActionError::Io {
            path: source,
            source: error,
        })?;
        moved.push(target.display().to_string());
    }

    for folder in &request.paths {
        fs::remove_dir(folder).map_err(|source| ActionError::Io {
            path: folder.clone(),
            source,
        })?;
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Dissolved {} folder(s)", request.paths.len()),
        payload: json!({ "moved": moved }),
    })
}

fn delete_permanently(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut deleted = Vec::new();

    for path in &request.paths {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|source| ActionError::Io {
                path: path.clone(),
                source,
            })?;
        } else {
            fs::remove_file(path).map_err(|source| ActionError::Io {
                path: path.clone(),
                source,
            })?;
        }
        deleted.push(path.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Deleted {} item(s)", deleted.len()),
        payload: json!({ "deleted": deleted }),
    })
}

fn move_to_trash(request: ActionRequest) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut trashed = Vec::new();
    let trash_dir = user_trash_dir();
    fs::create_dir_all(&trash_dir).map_err(|source| ActionError::Io {
        path: trash_dir.clone(),
        source,
    })?;

    for path in &request.paths {
        let file_name = path
            .file_name()
            .ok_or_else(|| ActionError::MissingFileName(path.display().to_string()))?;
        let target = unique_trash_target(&trash_dir, Path::new(file_name));
        fs::rename(path, &target).map_err(|source| ActionError::Io {
            path: path.clone(),
            source,
        })?;
        trashed.push(target.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Moved {} item(s) to trash", trashed.len()),
        payload: json!({ "trashed": trashed }),
    })
}

fn user_trash_dir() -> PathBuf {
    if let Some(path) = env::var_os("SRIGHT_TRASH_DIR") {
        return PathBuf::from(path);
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".Trash")
}

fn unique_trash_target(trash_dir: &Path, file_name: &Path) -> PathBuf {
    let target = trash_dir.join(file_name);
    if !target.exists() {
        return target;
    }

    let stem = file_name
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_else(|| file_name.as_os_str().to_string_lossy());
    let extension = file_name
        .extension()
        .map(|extension| extension.to_string_lossy());

    for index in 2.. {
        let candidate_name = match &extension {
            Some(extension) => format!("{stem} {index}.{extension}"),
            None => format!("{stem} {index}"),
        };
        let candidate = trash_dir.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded trash target search should always return");
}

fn selected_output_dir(paths: &[PathBuf]) -> ActionExecutionResult<PathBuf> {
    let selected = paths
        .first()
        .ok_or_else(|| ActionError::EmptySelection("new_file".to_string()))?;
    if selected.is_dir() {
        return Ok(selected.clone());
    }

    selected
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| ActionError::MissingFileName(selected.display().to_string()))
}

fn favorite_directory(favorite_id: &str, favorite_dirs: &[FavoriteDirectory]) -> PathBuf {
    let env_key = format!("SRIGHT_FAVORITE_{}_DIR", favorite_id.to_ascii_uppercase());
    if let Some(path) = env::var_os(env_key) {
        return PathBuf::from(path);
    }

    if let Some(directory) = favorite_dirs
        .iter()
        .find(|directory| directory.id == favorite_id)
    {
        return expand_home_path(&directory.path);
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    match favorite_id {
        "downloads" => home.join("Downloads"),
        "pictures" => home.join("Pictures"),
        "music" => home.join("Music"),
        "movies" => home.join("Movies"),
        "desktop" => home.join("Desktop"),
        "documents" => home.join("Documents"),
        _ => home,
    }
}

fn expand_home_path(path: &str) -> PathBuf {
    if path == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(path)
}

fn copy_item(source: &Path, target: &Path) -> ActionExecutionResult<()> {
    if source.is_dir() {
        fs::create_dir(target).map_err(|source| ActionError::Io {
            path: target.to_path_buf(),
            source,
        })?;
        for entry in fs::read_dir(source).map_err(|source_error| ActionError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })? {
            let entry = entry.map_err(|source_error| ActionError::Io {
                path: source.to_path_buf(),
                source: source_error,
            })?;
            copy_item(&entry.path(), &target.join(entry.file_name()))?;
        }
        return Ok(());
    }

    fs::copy(source, target)
        .map(|_| ())
        .map_err(|source_error| ActionError::Io {
            path: source.to_path_buf(),
            source: source_error,
        })
}

fn unique_name_target(directory: &Path, file_name: &Path) -> PathBuf {
    let target = directory.join(file_name);
    if !target.exists() {
        return target;
    }

    let stem = file_name
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_else(|| file_name.as_os_str().to_string_lossy());
    let extension = file_name
        .extension()
        .map(|extension| extension.to_string_lossy());

    for index in 2.. {
        let candidate_name = match &extension {
            Some(extension) => format!("{stem} {index}.{extension}"),
            None => format!("{stem} {index}"),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded target search should always return");
}

fn common_parent(paths: &[PathBuf]) -> ActionExecutionResult<PathBuf> {
    let first = paths
        .first()
        .ok_or_else(|| ActionError::EmptySelection("archive".to_string()))?;
    first
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| ActionError::MissingFileName(first.display().to_string()))
}

fn write_zip(archive_path: &Path, sources: &[PathBuf], base: &Path) -> ActionExecutionResult<()> {
    let file = File::create(archive_path).map_err(|source| ActionError::Io {
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for source in sources {
        if source.is_dir() {
            for entry in WalkDir::new(source) {
                let entry = entry.map_err(|source| ActionError::Walk {
                    path: source
                        .path()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| source.to_string().into()),
                    source,
                })?;
                add_zip_entry(&mut zip, entry.path(), base, options)?;
            }
        } else {
            add_zip_entry(&mut zip, source, base, options)?;
        }
    }

    zip.finish().map_err(|source| ActionError::Archive {
        path: archive_path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn add_zip_entry(
    zip: &mut zip::ZipWriter<File>,
    path: &Path,
    base: &Path,
    options: FileOptions,
) -> ActionExecutionResult<()> {
    let name = path
        .strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    if name.is_empty() {
        return Ok(());
    }

    if path.is_dir() {
        zip.add_directory(format!("{name}/"), options)
            .map_err(|source| ActionError::Archive {
                path: path.to_path_buf(),
                source,
            })?;
        return Ok(());
    }

    zip.start_file(name, options)
        .map_err(|source| ActionError::Archive {
            path: path.to_path_buf(),
            source,
        })?;
    let mut file = File::open(path).map_err(|source| ActionError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    std::io::copy(&mut file, zip).map_err(|source| ActionError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn extract_zip(archive_path: &Path, output_dir: &Path) -> ActionExecutionResult<()> {
    let file = File::open(archive_path).map_err(|source| ActionError::Io {
        path: archive_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|source| ActionError::Archive {
        path: archive_path.to_path_buf(),
        source,
    })?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|source| ActionError::Archive {
                path: archive_path.to_path_buf(),
                source,
            })?;
        let Some(enclosed) = entry.enclosed_name().map(Path::to_path_buf) else {
            continue;
        };
        let target = output_dir.join(enclosed);
        if entry.name().ends_with('/') {
            fs::create_dir_all(&target).map_err(|source| ActionError::Io {
                path: target,
                source,
            })?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| ActionError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let mut output = File::create(&target).map_err(|source| ActionError::Io {
            path: target.clone(),
            source,
        })?;
        std::io::copy(&mut entry, &mut output).map_err(|source| ActionError::Io {
            path: target,
            source,
        })?;
    }

    Ok(())
}

fn hash_file<D>(path: &Path) -> ActionExecutionResult<String>
where
    D: Digest + Default,
{
    let mut file = File::open(path).map_err(|source| ActionError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = D::default();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|source| ActionError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

fn run_tool_or_log(
    request: ActionRequest,
    log_env: &str,
    tool_id: &str,
    target_for: impl Fn(&Path) -> ActionExecutionResult<PathBuf>,
    run: impl Fn(&Path, &Path) -> ActionExecutionResult<()>,
) -> ActionExecutionResult<ActionResult> {
    require_selection(&request)?;
    let mut outputs = Vec::new();

    for path in &request.paths {
        let target = target_for(path)?;
        if let Some(log_path) = env::var_os(log_env).map(PathBuf::from) {
            append_tool_log(&log_path, tool_id, std::slice::from_ref(path))?;
            fs::write(&target, path.display().to_string()).map_err(|source| ActionError::Io {
                path: target.clone(),
                source,
            })?;
        } else {
            run(path, &target)?;
        }
        outputs.push(target.display().to_string());
    }

    Ok(ActionResult {
        action_id: request.action_id,
        selected_count: request.paths.len(),
        message: format!("Ran {tool_id} for {} item(s)", request.paths.len()),
        payload: json!({ "outputs": outputs }),
    })
}

fn append_tool_log(log_path: &Path, tool_id: &str, paths: &[PathBuf]) -> ActionExecutionResult<()> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|source| ActionError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|source| ActionError::Io {
            path: log_path.to_path_buf(),
            source,
        })?;
    writeln!(
        file,
        "{}\t{}",
        tool_id,
        paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\t")
    )
    .map_err(|source| ActionError::Io {
        path: log_path.to_path_buf(),
        source,
    })
}

fn require_selection(request: &ActionRequest) -> ActionExecutionResult<()> {
    if request.paths.is_empty() {
        return Err(ActionError::EmptySelection(request.action_id.clone()));
    }

    Ok(())
}

fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '/' | '.' | '_' | '-' | ':'))
    {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\\''"))
}
