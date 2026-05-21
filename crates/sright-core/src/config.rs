use std::{collections::HashSet, fs};

use serde::{Deserialize, Serialize};

use crate::actions::action_descriptors;
use crate::paths::{app_support_dir, config_path, finder_sync_app_support_dir};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SRightConfig {
    pub enabled: bool,
    pub show_icons: bool,
    #[serde(default = "default_show_menu_bar_icon")]
    pub show_menu_bar_icon: bool,
    #[serde(default)]
    pub settings_shortcut: String,
    pub merge_groups: bool,
    #[serde(default)]
    pub dangerous_confirmation: DangerousConfirmationConfig,
    #[serde(default = "default_file_templates")]
    pub file_templates: Vec<FileTemplate>,
    #[serde(default = "default_open_apps")]
    pub open_apps: Vec<OpenApp>,
    #[serde(default = "default_favorite_dirs")]
    pub favorite_dirs: Vec<FavoriteDirectory>,
    #[serde(default = "default_send_dirs")]
    pub send_dirs: Vec<FavoriteDirectory>,
    #[serde(default)]
    pub archive: ArchiveConfig,
    #[serde(default)]
    pub image: ImageConfig,
    #[serde(default)]
    pub toolbox: ToolboxConfig,
    #[serde(default = "default_custom_scripts")]
    pub custom_scripts: Vec<CustomScript>,
    pub menus: Vec<MenuItem>,
    #[serde(default)]
    pub menu_tree: Vec<MenuTreeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    #[serde(default)]
    pub dangerous: bool,
    #[serde(default)]
    pub file_kinds: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuTreeItem {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<MenuTreeItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DangerousConfirmationConfig {
    pub enabled: bool,
    pub action_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTemplate {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenApp {
    pub id: String,
    pub title: String,
    pub bundle_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteDirectory {
    pub id: String,
    pub title: String,
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub delete_source_after_archive: bool,
    pub delete_archive_after_extract: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageConfig {
    pub output_dir: Option<String>,
    pub quality: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolboxConfig {
    pub translation_provider: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomScript {
    pub id: String,
    pub title: String,
    pub command: String,
    pub enabled: bool,
    pub dangerous: bool,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            delete_source_after_archive: false,
            delete_archive_after_extract: false,
        }
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            quality: 90,
        }
    }
}

impl Default for ToolboxConfig {
    fn default() -> Self {
        Self {
            translation_provider: "none".to_string(),
        }
    }
}

impl Default for DangerousConfirmationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            action_ids: action_descriptors()
                .into_iter()
                .filter(|descriptor| descriptor.dangerous)
                .map(|descriptor| descriptor.id)
                .collect(),
        }
    }
}

impl DangerousConfirmationConfig {
    pub fn requires_confirmation(&self, action_id: &str) -> bool {
        self.enabled && self.action_ids.iter().any(|id| id == action_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to create app support directory: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("failed to read config.json: {0}")]
    Read(#[source] std::io::Error),
    #[error("failed to write config.json: {0}")]
    Write(#[source] std::io::Error),
    #[error("failed to sync FinderSync config.json: {0}")]
    SyncFinderSync(#[source] std::io::Error),
    #[error("failed to parse config.json: {0}")]
    Parse(#[source] serde_json::Error),
    #[error("failed to serialize config.json: {0}")]
    Serialize(#[source] serde_json::Error),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

pub fn default_config() -> SRightConfig {
    let config = SRightConfig {
        enabled: true,
        show_icons: true,
        show_menu_bar_icon: default_show_menu_bar_icon(),
        settings_shortcut: String::new(),
        merge_groups: false,
        dangerous_confirmation: DangerousConfirmationConfig::default(),
        file_templates: default_file_templates(),
        open_apps: default_open_apps(),
        favorite_dirs: default_favorite_dirs(),
        send_dirs: default_send_dirs(),
        archive: ArchiveConfig::default(),
        image: ImageConfig::default(),
        toolbox: ToolboxConfig::default(),
        custom_scripts: default_custom_scripts(),
        menus: default_menus(),
        menu_tree: Vec::new(),
    };
    with_menu_tree(config)
}

fn default_show_menu_bar_icon() -> bool {
    true
}

pub fn load_or_init_config() -> ConfigResult<SRightConfig> {
    let path = config_path();
    if !path.exists() {
        let config = default_config();
        save_config(&config)?;
        return Ok(config);
    }

    let contents = fs::read_to_string(path).map_err(ConfigError::Read)?;
    let config = serde_json::from_str(&contents).map_err(ConfigError::Parse)?;
    let config = ensure_default_menus(config);
    sync_finder_sync_config(&config)?;
    Ok(config)
}

pub fn save_config(config: &SRightConfig) -> ConfigResult<()> {
    let config = ensure_default_menus(config.clone());
    fs::create_dir_all(app_support_dir()).map_err(ConfigError::CreateDir)?;
    let contents = serde_json::to_string_pretty(&config).map_err(ConfigError::Serialize)?;
    fs::write(config_path(), format!("{contents}\n")).map_err(ConfigError::Write)?;
    sync_finder_sync_config(&config)
}

fn sync_finder_sync_config(config: &SRightConfig) -> ConfigResult<()> {
    if matches!(
        std::env::var("SRIGHT_SKIP_FINDER_SYNC_SYNC").as_deref(),
        Ok("1" | "true" | "yes")
    ) {
        return Ok(());
    }

    let Some(dir) = finder_sync_app_support_dir() else {
        return Ok(());
    };

    fs::create_dir_all(&dir).map_err(ConfigError::SyncFinderSync)?;
    let contents = serde_json::to_string_pretty(config).map_err(ConfigError::Serialize)?;
    fs::write(dir.join("config.json"), format!("{contents}\n")).map_err(ConfigError::SyncFinderSync)
}

fn default_menus() -> Vec<MenuItem> {
    action_descriptors()
        .into_iter()
        .map(|descriptor| {
            let default_enabled =
                !descriptor.id.starts_with("new_file.") && descriptor.default_enabled;
            MenuItem {
                id: descriptor.id,
                title: descriptor.title,
                enabled: default_enabled,
                dangerous: descriptor.dangerous,
                file_kinds: Vec::new(),
                extensions: Vec::new(),
            }
        })
        .collect()
}

fn ensure_default_menus(mut config: SRightConfig) -> SRightConfig {
    ensure_open_apps(&mut config.open_apps);
    ensure_favorite_dirs(&mut config.favorite_dirs);
    ensure_send_dirs(&mut config.send_dirs);
    ensure_custom_scripts(&mut config.custom_scripts);
    retain_known_menu_items(&mut config);

    for default_menu in default_menus() {
        if config.menus.iter().any(|menu| menu.id == default_menu.id) {
            continue;
        }
        config.menus.push(default_menu);
    }

    for menu in &mut config.menus {
        if let Some(descriptor) = action_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.id == menu.id)
        {
            menu.dangerous = descriptor.dangerous;
        }
    }

    ensure_favorite_menus(&mut config.menus, &config.favorite_dirs);
    ensure_send_menus(&mut config.menus, &config.send_dirs);

    config.menu_tree = build_menu_tree(&config);
    config
}

fn with_menu_tree(mut config: SRightConfig) -> SRightConfig {
    config.menu_tree = build_menu_tree(&config);
    config
}

fn retain_known_menu_items(config: &mut SRightConfig) {
    let menu_ids = known_menu_ids(&config.favorite_dirs, &config.send_dirs);
    config.menus.retain(|menu| menu_ids.contains(&menu.id));

    let dangerous_action_ids = action_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.dangerous)
        .map(|descriptor| descriptor.id)
        .collect::<HashSet<_>>();
    config
        .dangerous_confirmation
        .action_ids
        .retain(|action_id| dangerous_action_ids.contains(action_id));
}

fn build_menu_tree(config: &SRightConfig) -> Vec<MenuTreeItem> {
    let mut tree = Vec::new();

    if let Some(group) = new_file_menu_tree(config) {
        tree.push(group);
    }
    if let Some(group) = send_to_menu_tree(config) {
        tree.push(group);
    }
    if let Some(group) = favorite_dirs_menu_tree(config) {
        tree.push(group);
    }
    if let Some(group) = toolbox_menu_tree(config) {
        tree.push(group);
    }

    tree
}

fn new_file_menu_tree(config: &SRightConfig) -> Option<MenuTreeItem> {
    let children = config
        .file_templates
        .iter()
        .filter(|template| template.enabled)
        .map(|template| {
            leaf_menu_item(
                &template.title,
                &format!("new_file.{}", template.id),
                Some(format!("file:{}", template.file_name)),
            )
        })
        .collect::<Vec<_>>();

    group_menu_item("新建文件", Some("file:Untitled.txt".to_string()), children)
}

fn send_to_menu_tree(config: &SRightConfig) -> Option<MenuTreeItem> {
    let move_children = send_dir_children(config, "send.move_to.");
    let copy_children = send_dir_children(config, "send.copy_to.");
    let mut children = Vec::new();

    if let Some(group) = group_menu_item("移动文件到...", None, move_children) {
        children.push(group);
    }
    if let Some(group) = group_menu_item("复制文件到...", None, copy_children) {
        children.push(group);
    }

    group_menu_item("发送文件到", Some("home".to_string()), children)
}

fn send_dir_children(config: &SRightConfig, prefix: &str) -> Vec<MenuTreeItem> {
    config
        .send_dirs
        .iter()
        .filter(|directory| directory.enabled)
        .filter_map(|directory| {
            let action_id = format!("{prefix}{}", directory.id);
            enabled_menu(config, &action_id).map(|_| {
                leaf_menu_item(
                    &directory.title,
                    &action_id,
                    Some(format!("path:{}", directory.path)),
                )
            })
        })
        .collect()
}

fn favorite_dirs_menu_tree(config: &SRightConfig) -> Option<MenuTreeItem> {
    let children = config
        .favorite_dirs
        .iter()
        .filter(|directory| directory.enabled)
        .filter_map(|directory| {
            let action_id = format!("favorite.open.{}", directory.id);
            enabled_menu(config, &action_id).map(|_| {
                leaf_menu_item(
                    &directory.title,
                    &action_id,
                    Some(format!("path:{}", directory.path)),
                )
            })
        })
        .collect::<Vec<_>>();

    group_menu_item("常用目录", Some("home".to_string()), children)
}

fn toolbox_menu_tree(config: &SRightConfig) -> Option<MenuTreeItem> {
    let children = config
        .menus
        .iter()
        .filter(|menu| menu.enabled)
        .filter(|menu| is_toolbox_menu_item(&menu.id))
        .map(|menu| leaf_menu_item(&menu.title, &menu.id, None))
        .collect::<Vec<_>>();

    group_menu_item(
        "工具箱",
        Some("system:wrench.and.screwdriver".to_string()),
        children,
    )
}

fn is_toolbox_menu_item(action_id: &str) -> bool {
    !action_id.starts_with("new_file.")
        && !action_id.starts_with("favorite.open.")
        && !action_id.starts_with("send.copy_to.")
        && !action_id.starts_with("send.move_to.")
}

fn enabled_menu<'a>(config: &'a SRightConfig, action_id: &str) -> Option<&'a MenuItem> {
    config
        .menus
        .iter()
        .find(|menu| menu.id == action_id && menu.enabled)
}

fn group_menu_item(
    title: &str,
    icon: Option<String>,
    children: Vec<MenuTreeItem>,
) -> Option<MenuTreeItem> {
    if children.is_empty() {
        return None;
    }

    Some(MenuTreeItem {
        title: title.to_string(),
        action_id: None,
        icon,
        children,
    })
}

fn leaf_menu_item(title: &str, action_id: &str, icon: Option<String>) -> MenuTreeItem {
    MenuTreeItem {
        title: title.to_string(),
        action_id: Some(action_id.to_string()),
        icon,
        children: Vec::new(),
    }
}

fn known_menu_ids(
    favorite_dirs: &[FavoriteDirectory],
    send_dirs: &[FavoriteDirectory],
) -> HashSet<String> {
    let mut ids = action_descriptors()
        .into_iter()
        .map(|descriptor| descriptor.id)
        .collect::<HashSet<_>>();

    for directory in favorite_dirs {
        ids.insert(format!("favorite.open.{}", directory.id));
    }

    for directory in send_dirs {
        for (id, _) in send_menu_items(directory) {
            ids.insert(id);
        }
    }

    ids
}

fn default_file_templates() -> Vec<FileTemplate> {
    vec![
        file_template("custom", "自定义创建新文件", "Untitled"),
        file_template("text", "TXT", "Untitled.txt"),
        file_template("rtf", "RTF", "Untitled.rtf"),
        file_template("xml", "XML", "Untitled.xml"),
        file_template("word", "Word", "Untitled.docx"),
        file_template("excel", "Excel", "Untitled.xlsx"),
        file_template("ppt", "PPT", "Untitled.pptx"),
        file_template("wps_writer", "WPS 文字", "Untitled.wps"),
        file_template("wps_spreadsheet", "WPS 表格", "Untitled.et"),
        file_template("wps_presentation", "WPS 演示", "Untitled.dps"),
        file_template("pages", "Pages", "Untitled.pages"),
        file_template("numbers", "Numbers", "Untitled.numbers"),
        file_template("keynote", "Keynote", "Untitled.key"),
        file_template("ai", "Ai", "Untitled.ai"),
        file_template("psd", "PSD", "Untitled.psd"),
        file_template("markdown", "Markdown", "Untitled.md"),
    ]
}

fn file_template(id: &str, title: &str, file_name: &str) -> FileTemplate {
    FileTemplate {
        id: id.to_string(),
        title: title.to_string(),
        file_name: file_name.to_string(),
        enabled: true,
    }
}

fn default_open_apps() -> Vec<OpenApp> {
    vec![
        open_app("terminal", "Terminal", "com.apple.Terminal"),
        open_app("vscode", "Visual Studio Code", "com.microsoft.VSCode"),
        open_app("cursor", "Cursor", "com.todesktop.230313mzl4w4u92"),
    ]
}

fn open_app(id: &str, title: &str, bundle_id: &str) -> OpenApp {
    OpenApp {
        id: id.to_string(),
        title: title.to_string(),
        bundle_id: bundle_id.to_string(),
        enabled: true,
    }
}

fn default_favorite_dirs() -> Vec<FavoriteDirectory> {
    default_directory_presets()
}

fn default_send_dirs() -> Vec<FavoriteDirectory> {
    default_directory_presets()
}

fn default_directory_presets() -> Vec<FavoriteDirectory> {
    vec![
        favorite_dir("downloads", "下载", "~/Downloads"),
        favorite_dir("pictures", "图片", "~/Pictures"),
        favorite_dir("music", "音乐", "~/Music"),
        favorite_dir("movies", "影片", "~/Movies"),
        favorite_dir("desktop", "桌面", "~/Desktop"),
        favorite_dir("documents", "文稿", "~/Documents"),
    ]
}

fn favorite_dir(id: &str, title: &str, path: &str) -> FavoriteDirectory {
    FavoriteDirectory {
        id: id.to_string(),
        title: title.to_string(),
        path: path.to_string(),
        enabled: true,
    }
}

fn ensure_open_apps(apps: &mut Vec<OpenApp>) {
    for default_app in default_open_apps() {
        if apps.iter().any(|app| app.id == default_app.id) {
            continue;
        }
        apps.push(default_app);
    }
}

fn ensure_favorite_dirs(directories: &mut Vec<FavoriteDirectory>) {
    ensure_directory_presets(directories);
}

fn ensure_send_dirs(directories: &mut Vec<FavoriteDirectory>) {
    ensure_directory_presets(directories);
}

fn ensure_directory_presets(directories: &mut Vec<FavoriteDirectory>) {
    for default_directory in default_directory_presets() {
        if directories
            .iter()
            .any(|directory| directory.id == default_directory.id)
        {
            continue;
        }
        directories.push(default_directory);
    }
}

fn ensure_favorite_menus(menus: &mut Vec<MenuItem>, directories: &[FavoriteDirectory]) {
    for directory in directories {
        let id = format!("favorite.open.{}", directory.id);
        let title = format!("打开{}", directory.title);
        if let Some(menu) = menus.iter_mut().find(|menu| menu.id == id) {
            menu.title = title;
            menu.dangerous = false;
            continue;
        }

        menus.push(MenuItem {
            id,
            title,
            enabled: directory.enabled,
            dangerous: false,
            file_kinds: Vec::new(),
            extensions: Vec::new(),
        });
    }
}

fn ensure_send_menus(menus: &mut Vec<MenuItem>, directories: &[FavoriteDirectory]) {
    for directory in directories {
        for (id, title) in send_menu_items(directory) {
            if let Some(menu) = menus.iter_mut().find(|menu| menu.id == id) {
                menu.title = title;
                menu.dangerous = false;
                continue;
            }

            menus.push(MenuItem {
                id,
                title,
                enabled: directory.enabled,
                dangerous: false,
                file_kinds: Vec::new(),
                extensions: Vec::new(),
            });
        }
    }
}

fn send_menu_items(directory: &FavoriteDirectory) -> [(String, String); 2] {
    [
        (
            format!("send.copy_to.{}", directory.id),
            format!("复制到{}", directory.title),
        ),
        (
            format!("send.move_to.{}", directory.id),
            format!("移动到{}", directory.title),
        ),
    ]
}

fn default_custom_scripts() -> Vec<CustomScript> {
    vec![CustomScript {
        id: "default".to_string(),
        title: "默认脚本动作".to_string(),
        command: String::new(),
        enabled: false,
        dangerous: false,
    }]
}

fn ensure_custom_scripts(scripts: &mut Vec<CustomScript>) {
    for default_script in default_custom_scripts() {
        if scripts.iter().any(|script| script.id == default_script.id) {
            continue;
        }
        scripts.push(default_script);
    }
}
