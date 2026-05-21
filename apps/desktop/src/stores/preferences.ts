import { create } from "zustand";
import {
    loadConfig,
    openFinderExtensionSettings,
    openPathInFinder,
    pickDirectory,
    runDebugAction,
    saveConfig,
    type FavoriteDirectory,
    type SRightConfig
} from "../lib/api";

type TopLevelFlag = "enabled" | "show_icons" | "merge_groups";

interface PreferenceState {
    busy: boolean;
    config: SRightConfig | null;
    settingsStatus: string;
    status: string;
    addFavoriteDirFromPicker: () => Promise<void>;
    openFavoriteDir: (directoryId: string) => Promise<void>;
    openExtensionSettings: () => Promise<void>;
    persist: () => Promise<void>;
    refresh: () => Promise<void>;
    removeFavoriteDir: (directoryId: string) => Promise<void>;
    renameFavoriteDir: (directoryId: string, title: string) => Promise<void>;
    renameTemplate: (templateId: string, title: string) => Promise<void>;
    reorderFavoriteDirs: (fromDirectoryId: string, targetDirectoryId: string) => Promise<void>;
    reorderTemplates: (fromTemplateId: string, targetTemplateId: string) => Promise<void>;
    runDebug: () => Promise<void>;
    setCustomScriptCommand: (scriptId: string, command: string) => Promise<void>;
    setDangerousConfirmationEnabled: (enabled: boolean) => Promise<void>;
    setMenuEnabled: (actionId: string, enabled: boolean) => Promise<void>;
    setTemplateEnabled: (templateId: string, enabled: boolean) => Promise<void>;
    setTemplateMainMenu: (templateId: string, enabled: boolean) => Promise<void>;
    setToolboxProvider: (provider: string) => Promise<void>;
    setTopLevelFlag: (key: TopLevelFlag, enabled: boolean) => Promise<void>;
    toggleAllFavorites: (enabled: boolean) => Promise<void>;
    toggleCustomScript: (scriptId: string, enabled: boolean) => Promise<void>;
    toggleFavoriteDir: (directoryId: string, enabled: boolean) => Promise<void>;
    toggleSendMenus: (prefix: "send.copy_to." | "send.move_to.", enabled: boolean) => Promise<void>;
}

function favoriteMenuIds(directoryId: string) {
    return [`favorite.open.${directoryId}`, `send.copy_to.${directoryId}`, `send.move_to.${directoryId}`];
}

function uniqueFavoriteDirectoryId(config: SRightConfig, path: string) {
    const baseId =
        path
            .split("/")
            .filter(Boolean)
            .at(-1)
            ?.toLowerCase()
            .replace(/[^a-z0-9]+/g, "_")
            .replace(/^_+|_+$/g, "") || "directory";
    const existingIds = new Set(config.favorite_dirs.map((directory) => directory.id));
    let id = baseId;
    let index = 2;

    while (existingIds.has(id)) {
        id = `${baseId}_${index}`;
        index += 1;
    }

    return id;
}

function directoryTitle(path: string) {
    return path.split("/").filter(Boolean).at(-1) || path;
}

function setMenuEnabledInConfig(config: SRightConfig, actionId: string, enabled: boolean) {
    const menu = config.menus.find((item) => item.id === actionId);
    if (menu) {
        menu.enabled = enabled;
    }
}

function addFavoriteMenus(config: SRightConfig, directoryId: string, title: string, enabled: boolean) {
    for (const [id, prefix] of [
        [`favorite.open.${directoryId}`, "打开"],
        [`send.copy_to.${directoryId}`, "复制到"],
        [`send.move_to.${directoryId}`, "移动到"]
    ] as const) {
        const menu = config.menus.find((item) => item.id === id);
        if (menu) {
            menu.title = `${prefix}${title}`;
            menu.enabled = enabled;
        } else {
            config.menus.push({
                id,
                title: `${prefix}${title}`,
                enabled,
                dangerous: false,
                file_kinds: [],
                extensions: []
            });
        }
    }
}

function moveItem<T extends { id: string }>(items: T[], fromId: string, targetId: string) {
    const fromIndex = items.findIndex((item) => item.id === fromId);
    const toIndex = items.findIndex((item) => item.id === targetId);
    if (fromIndex === -1 || toIndex === -1 || fromIndex === toIndex) {
        return;
    }

    const [movedItem] = items.splice(fromIndex, 1);
    items.splice(toIndex, 0, movedItem);
}

export const usePreferenceStore = create<PreferenceState>((set, get) => {
    async function saveNextConfig(nextConfig: SRightConfig) {
        set({ busy: true, config: nextConfig });
        try {
            await saveConfig(nextConfig);
            set({ status: "已保存" });
        } finally {
            set({ busy: false });
        }
    }

    async function updateConfig(mutator: (config: SRightConfig) => void) {
        const currentConfig = get().config;
        if (!currentConfig) {
            return;
        }

        const nextConfig = structuredClone(currentConfig);
        mutator(nextConfig);
        await saveNextConfig(nextConfig);
    }

    return {
        busy: false,
        config: null,
        settingsStatus: "",
        status: "",

        async refresh() {
            set({ status: "" });
            const config = await loadConfig();
            set({ config });
        },

        async persist() {
            const config = get().config;
            if (!config) {
                return;
            }

            await saveNextConfig(config);
        },

        async runDebug() {
            set({ busy: true });
            try {
                const status = await runDebugAction();
                set({ status });
            } finally {
                set({ busy: false });
            }
        },

        async openExtensionSettings() {
            set({ busy: true, settingsStatus: "" });
            try {
                await openFinderExtensionSettings();
                set({ settingsStatus: "已打开系统设置，请在 Finder Extensions 中启用 SRightFinderSync。" });
            } finally {
                set({ busy: false });
            }
        },

        async setMenuEnabled(actionId, enabled) {
            await updateConfig((config) => {
                setMenuEnabledInConfig(config, actionId, enabled);
            });
        },

        async setTopLevelFlag(key, enabled) {
            await updateConfig((config) => {
                config[key] = enabled;
            });
        },

        async setDangerousConfirmationEnabled(enabled) {
            await updateConfig((config) => {
                config.dangerous_confirmation.enabled = enabled;
            });
        },

        async setTemplateEnabled(templateId, enabled) {
            await updateConfig((config) => {
                const template = config.file_templates.find((item) => item.id === templateId);
                if (!template) {
                    return;
                }

                template.enabled = enabled;
                if (!enabled) {
                    setMenuEnabledInConfig(config, `new_file.${templateId}`, false);
                }
            });
        },

        async setTemplateMainMenu(templateId, enabled) {
            await updateConfig((config) => {
                setMenuEnabledInConfig(config, templateId.startsWith("new_file.") ? templateId : `new_file.${templateId}`, enabled);
            });
        },

        async renameTemplate(templateId, title) {
            await updateConfig((config) => {
                const template = config.file_templates.find((item) => item.id === templateId);
                if (!template) {
                    return;
                }

                template.title = title.trim() || template.file_name;
            });
        },

        async reorderTemplates(fromTemplateId, targetTemplateId) {
            await updateConfig((config) => {
                moveItem(config.file_templates, fromTemplateId, targetTemplateId);
            });
        },

        async toggleFavoriteDir(directoryId, enabled) {
            await updateConfig((config) => {
                for (const id of favoriteMenuIds(directoryId)) {
                    setMenuEnabledInConfig(config, id, enabled);
                }
            });
        },

        async openFavoriteDir(directoryId) {
            const directory = get().config?.favorite_dirs.find((item) => item.id === directoryId);
            if (!directory) {
                return;
            }

            await openPathInFinder(directory.path);
        },

        async addFavoriteDirFromPicker() {
            const currentConfig = get().config;
            if (!currentConfig) {
                return;
            }

            const path = await pickDirectory();
            if (!path || currentConfig.favorite_dirs.some((directory) => directory.path === path)) {
                return;
            }

            await updateConfig((config) => {
                const id = uniqueFavoriteDirectoryId(config, path);
                const title = directoryTitle(path);
                config.favorite_dirs.push({ id, title, path, enabled: true });
                addFavoriteMenus(config, id, title, true);
            });
        },

        async removeFavoriteDir(directoryId) {
            await updateConfig((config) => {
                config.favorite_dirs = config.favorite_dirs.filter((directory) => directory.id !== directoryId);
                const menuIds = new Set(favoriteMenuIds(directoryId));
                config.menus = config.menus.filter((menu) => !menuIds.has(menu.id));
            });
        },

        async renameFavoriteDir(directoryId, title) {
            await updateConfig((config) => {
                const directory = config.favorite_dirs.find((item) => item.id === directoryId);
                if (!directory) {
                    return;
                }

                directory.title = title.trim() || directoryTitle(directory.path);
                addFavoriteMenus(config, directory.id, directory.title, directory.enabled);
            });
        },

        async reorderFavoriteDirs(fromDirectoryId, targetDirectoryId) {
            await updateConfig((config) => {
                moveItem(config.favorite_dirs, fromDirectoryId, targetDirectoryId);
            });
        },

        async toggleSendMenus(prefix, enabled) {
            await updateConfig((config) => {
                for (const menu of config.menus) {
                    if (menu.id.startsWith(prefix)) {
                        menu.enabled = enabled;
                    }
                }
            });
        },

        async toggleAllFavorites(enabled) {
            await updateConfig((config) => {
                for (const directory of config.favorite_dirs) {
                    directory.enabled = enabled;
                    for (const menuId of favoriteMenuIds(directory.id)) {
                        setMenuEnabledInConfig(config, menuId, enabled);
                    }
                }
            });
        },

        async setToolboxProvider(provider) {
            await updateConfig((config) => {
                config.toolbox.translation_provider = provider;
            });
        },

        async setCustomScriptCommand(scriptId, command) {
            await updateConfig((config) => {
                const script = config.custom_scripts.find((item) => item.id === scriptId);
                if (script) {
                    script.command = command;
                }
            });
        },

        async toggleCustomScript(scriptId, enabled) {
            await updateConfig((config) => {
                const script = config.custom_scripts.find((item) => item.id === scriptId);
                if (script) {
                    script.enabled = enabled;
                }
                setMenuEnabledInConfig(config, `script.run.${scriptId}`, enabled);
            });
        }
    };
});

export type { FavoriteDirectory };
