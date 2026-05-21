import { invoke } from "@tauri-apps/api/core";

export interface MenuItem {
    id: string;
    title: string;
    enabled: boolean;
    dangerous: boolean;
    file_kinds: string[];
    extensions: string[];
}

export interface DangerousConfirmationConfig {
    enabled: boolean;
    action_ids: string[];
}

export interface FileTemplate {
    id: string;
    title: string;
    file_name: string;
    enabled: boolean;
}

export interface OpenApp {
    id: string;
    title: string;
    bundle_id: string;
    enabled: boolean;
}

export interface FavoriteDirectory {
    id: string;
    title: string;
    path: string;
    enabled: boolean;
}

export interface ArchiveConfig {
    delete_source_after_archive: boolean;
    delete_archive_after_extract: boolean;
}

export interface ImageConfig {
    output_dir: string | null;
    quality: number;
}

export interface ToolboxConfig {
    translation_provider: string;
}

export interface CustomScript {
    id: string;
    title: string;
    command: string;
    enabled: boolean;
    dangerous: boolean;
}

export interface SRightConfig {
    enabled: boolean;
    show_icons: boolean;
    merge_groups: boolean;
    dangerous_confirmation: DangerousConfirmationConfig;
    file_templates: FileTemplate[];
    open_apps: OpenApp[];
    favorite_dirs: FavoriteDirectory[];
    archive: ArchiveConfig;
    image: ImageConfig;
    toolbox: ToolboxConfig;
    custom_scripts: CustomScript[];
    menus: MenuItem[];
}

export interface ActionLogEntry {
    timestamp: number;
    action_id: string;
    selected_count: number;
    status: "success" | "failure";
    message: string;
    error: string | null;
}

export interface Diagnostics {
    config_path: string;
    log_path: string;
}

export function loadConfig(): Promise<SRightConfig> {
    return invoke("load_config");
}

export function saveConfig(config: SRightConfig): Promise<void> {
    return invoke("save_config_command", { config });
}

export function runDebugAction(): Promise<string> {
    return invoke("run_debug_action");
}

export function recentLogs(limit: number): Promise<ActionLogEntry[]> {
    return invoke("recent_logs", { limit });
}

export function diagnostics(): Promise<Diagnostics> {
    return invoke("diagnostics");
}

export function openFinderExtensionSettings(): Promise<void> {
    return invoke("open_finder_extension_settings");
}

export function pickDirectory(): Promise<string | null> {
    return invoke("pick_directory");
}

export function openPathInFinder(path: string): Promise<void> {
    return invoke("open_path_in_finder", { path });
}
