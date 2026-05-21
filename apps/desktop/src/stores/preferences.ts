import { create } from "zustand";
import {
    loadConfig,
    openFinderExtensionSettings,
    openFullDiskAccessSettings,
    openPathInFinder,
    pickDirectory,
    pickTemplateFile,
    saveConfig,
    type FavoriteDirectory,
    type SRightConfig
} from "../lib/api";
import {
    addFavoriteMenu,
    addSendMenus,
    defaultDirectoryPresets,
    directoryTitle,
    favoriteMenuId,
    resetFavoriteMenus,
    resetSendMenus,
    sendMenuIds,
    uniqueDirectoryId
} from "../lib/directory-config";
import { templateInfoFromPath } from "../lib/file-template-utils";
import { setTemplateMenuEnabled, syncTemplateMenuTitle } from "../lib/template-menu-config";

type TopLevelFlag = "enabled" | "show_icons" | "show_menu_bar_icon";

interface PreferenceState {
    busy: boolean;
    config: SRightConfig | null;
    status: string;
    addFavoriteDirFromPicker: () => Promise<void>;
    addSendDirFromPicker: () => Promise<void>;
    addTemplateFromPicker: () => Promise<void>;
    openFavoriteDir: (directoryId: string) => Promise<void>;
    openExtensionSettings: () => Promise<void>;
    openPermissionSettings: () => Promise<void>;
    persist: () => Promise<void>;
    refresh: () => Promise<void>;
    removeFavoriteDir: (directoryId: string) => Promise<void>;
    removeSendDir: (directoryId: string) => Promise<void>;
    removeTemplate: (templateId: string) => Promise<void>;
    removeTemplates: (templateIds: string[]) => Promise<void>;
    resetFavoriteDirs: () => Promise<void>;
    resetSendDirs: () => Promise<void>;
    renameFavoriteDir: (directoryId: string, title: string) => Promise<void>;
    renameSendDir: (directoryId: string, title: string) => Promise<void>;
    renameTemplate: (templateId: string, title: string) => Promise<void>;
    reorderFavoriteDirs: (fromDirectoryId: string, targetDirectoryId: string) => Promise<void>;
    reorderSendDirs: (fromDirectoryId: string, targetDirectoryId: string) => Promise<void>;
    reorderTemplates: (fromTemplateId: string, targetTemplateId: string) => Promise<void>;
    setCustomScriptCommand: (scriptId: string, command: string) => Promise<void>;
    setDangerousActionConfirmation: (actionId: string, enabled: boolean) => Promise<void>;
    setDangerousConfirmationEnabled: (enabled: boolean) => Promise<void>;
    setSettingsShortcut: (shortcut: string) => Promise<void>;
    setMenuEnabled: (actionId: string, enabled: boolean) => Promise<void>;
    renameMenu: (actionId: string, title: string) => Promise<void>;
    reorderMenus: (fromActionId: string, targetActionId: string) => Promise<void>;
    setTemplateEnabled: (templateId: string, enabled: boolean) => Promise<void>;
    setTemplateMainMenu: (templateId: string, enabled: boolean) => Promise<void>;
    setTopLevelFlag: (key: TopLevelFlag, enabled: boolean) => Promise<void>;
    toggleAllFavorites: (enabled: boolean) => Promise<void>;
    toggleCustomScript: (scriptId: string, enabled: boolean) => Promise<void>;
    toggleFavoriteDir: (directoryId: string, enabled: boolean) => Promise<void>;
    toggleSendMenus: (prefix: "send.copy_to." | "send.move_to.", enabled: boolean) => Promise<void>;
}

function uniqueTemplateId(config: SRightConfig) {
    const existingIds = new Set(config.file_templates.map((template) => template.id));
    let index = config.file_templates.length + 1;
    let id = `custom_${index}`;

    while (existingIds.has(id)) {
        index += 1;
        id = `custom_${index}`;
    }

    return id;
}

function setMenuEnabledInConfig(config: SRightConfig, actionId: string, enabled: boolean) {
    const menu = config.menus.find((item) => item.id === actionId);
    if (menu) {
        menu.enabled = enabled;
    }

    const scriptId = actionId.startsWith("script.run.") ? actionId.slice("script.run.".length) : null;
    const script = scriptId ? config.custom_scripts.find((item) => item.id === scriptId) : null;
    if (script) {
        script.enabled = enabled;
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

        async openExtensionSettings() {
            try {
                await openFinderExtensionSettings();
            } finally {
            }
        },

        async openPermissionSettings() {
            try {
                await openFullDiskAccessSettings();
            } finally {
            }
        },

        async setMenuEnabled(actionId, enabled) {
            await updateConfig((config) => {
                setMenuEnabledInConfig(config, actionId, enabled);
            });
        },

        async renameMenu(actionId, title) {
            await updateConfig((config) => {
                const menu = config.menus.find((item) => item.id === actionId);
                if (menu) {
                    menu.title = title.trim() || menu.title;
                }
            });
        },

        async reorderMenus(fromActionId, targetActionId) {
            await updateConfig((config) => {
                moveItem(config.menus, fromActionId, targetActionId);
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

        async setDangerousActionConfirmation(actionId, enabled) {
            await updateConfig((config) => {
                const actionIds = new Set(config.dangerous_confirmation.action_ids);
                if (enabled) {
                    actionIds.add(actionId);
                } else {
                    actionIds.delete(actionId);
                }
                config.dangerous_confirmation.action_ids = Array.from(actionIds);
            });
        },

        async setSettingsShortcut(shortcut) {
            await updateConfig((config) => {
                config.settings_shortcut = shortcut;
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
                    setTemplateMenuEnabled(config, templateId, false);
                }
            });
        },

        async setTemplateMainMenu(templateId, enabled) {
            await updateConfig((config) => {
                setTemplateMenuEnabled(config, templateId, enabled);
            });
        },

        async renameTemplate(templateId, title) {
            await updateConfig((config) => {
                const template = config.file_templates.find((item) => item.id === templateId);
                if (!template) {
                    return;
                }

                template.title = title.trim() || template.file_name;
                syncTemplateMenuTitle(config, template, false);
            });
        },

        async reorderTemplates(fromTemplateId, targetTemplateId) {
            await updateConfig((config) => {
                moveItem(config.file_templates, fromTemplateId, targetTemplateId);
            });
        },

        async toggleFavoriteDir(directoryId, enabled) {
            await updateConfig((config) => {
                setMenuEnabledInConfig(config, favoriteMenuId(directoryId), enabled);
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
                const id = uniqueDirectoryId(config.favorite_dirs, path);
                const title = directoryTitle(path);
                const directory = { id, title, path, enabled: true };
                config.favorite_dirs.push(directory);
                addFavoriteMenu(config, directory);
            });
        },

        async addSendDirFromPicker() {
            const currentConfig = get().config;
            if (!currentConfig) {
                return;
            }

            const path = await pickDirectory();
            if (!path || currentConfig.send_dirs.some((directory) => directory.path === path)) {
                return;
            }

            await updateConfig((config) => {
                const id = uniqueDirectoryId(config.send_dirs, path);
                const title = directoryTitle(path);
                const directory = { id, title, path, enabled: true };
                config.send_dirs.push(directory);
                addSendMenus(config, directory);
            });
        },

        async addTemplateFromPicker() {
            const currentConfig = get().config;
            if (!currentConfig) {
                return;
            }

            const path = await pickTemplateFile();
            if (!path) {
                return;
            }

            const templateInfo = templateInfoFromPath(path);
            await updateConfig((config) => {
                const id = uniqueTemplateId(config);
                const template = {
                    id,
                    title: templateInfo.title,
                    file_name: templateInfo.fileName,
                    enabled: true
                };

                config.file_templates.push(template);
                syncTemplateMenuTitle(config, template, false);
            });
        },

        async removeTemplate(templateId) {
            await updateConfig((config) => {
                config.file_templates = config.file_templates.filter((template) => template.id !== templateId);
                config.menus = config.menus.filter((menu) => menu.id !== `new_file.${templateId}`);
            });
        },

        async removeTemplates(templateIds) {
            await updateConfig((config) => {
                const ids = new Set(templateIds);
                config.file_templates = config.file_templates.filter((template) => !ids.has(template.id));
                config.menus = config.menus.filter((menu) => {
                    const templateId = menu.id.startsWith("new_file.") ? menu.id.slice("new_file.".length) : menu.id;
                    return !ids.has(templateId);
                });
            });
        },

        async removeFavoriteDir(directoryId) {
            await updateConfig((config) => {
                config.favorite_dirs = config.favorite_dirs.filter((directory) => directory.id !== directoryId);
                config.menus = config.menus.filter((menu) => menu.id !== favoriteMenuId(directoryId));
            });
        },

        async removeSendDir(directoryId) {
            await updateConfig((config) => {
                config.send_dirs = config.send_dirs.filter((directory) => directory.id !== directoryId);
                const menuIds = new Set(sendMenuIds(directoryId));
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
                addFavoriteMenu(config, directory);
            });
        },

        async renameSendDir(directoryId, title) {
            await updateConfig((config) => {
                const directory = config.send_dirs.find((item) => item.id === directoryId);
                if (!directory) {
                    return;
                }

                directory.title = title.trim() || directoryTitle(directory.path);
                addSendMenus(config, directory);
            });
        },

        async reorderFavoriteDirs(fromDirectoryId, targetDirectoryId) {
            await updateConfig((config) => {
                moveItem(config.favorite_dirs, fromDirectoryId, targetDirectoryId);
            });
        },

        async reorderSendDirs(fromDirectoryId, targetDirectoryId) {
            await updateConfig((config) => {
                moveItem(config.send_dirs, fromDirectoryId, targetDirectoryId);
            });
        },

        async resetFavoriteDirs() {
            await updateConfig((config) => {
                config.favorite_dirs = defaultDirectoryPresets();
                resetFavoriteMenus(config);
            });
        },

        async resetSendDirs() {
            await updateConfig((config) => {
                config.send_dirs = defaultDirectoryPresets();
                resetSendMenus(config);
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
                    setMenuEnabledInConfig(config, favoriteMenuId(directory.id), enabled);
                }
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
