import type { FavoriteDirectory, SRightConfig } from './api';

export function defaultDirectoryPresets(): FavoriteDirectory[] {
    return [
        directoryPreset('downloads', '下载', '~/Downloads'),
        directoryPreset('pictures', '图片', '~/Pictures'),
        directoryPreset('music', '音乐', '~/Music'),
        directoryPreset('movies', '影片', '~/Movies'),
        directoryPreset('desktop', '桌面', '~/Desktop'),
        directoryPreset('documents', '文稿', '~/Documents')
    ];
}

export function directoryTitle(path: string) {
    return path.split('/').filter(Boolean).at(-1) || path;
}

export function uniqueDirectoryId(directories: FavoriteDirectory[], path: string) {
    const baseId =
        path
            .split('/')
            .filter(Boolean)
            .at(-1)
            ?.toLowerCase()
            .replace(/[^a-z0-9]+/g, '_')
            .replace(/^_+|_+$/g, '') || 'directory';
    const existingIds = new Set(directories.map(directory => directory.id));
    let id = baseId;
    let index = 2;

    while (existingIds.has(id)) {
        id = `${baseId}_${index}`;
        index += 1;
    }

    return id;
}

export function favoriteMenuId(directoryId: string) {
    return `favorite.open.${directoryId}`;
}

export function sendMenuIds(directoryId: string) {
    return [`send.copy_to.${directoryId}`, `send.move_to.${directoryId}`];
}

export function addFavoriteMenu(config: SRightConfig, directory: FavoriteDirectory) {
    upsertMenu(config, favoriteMenuId(directory.id), `打开${directory.title}`, directory.enabled);
}

export function addSendMenus(config: SRightConfig, directory: FavoriteDirectory) {
    for (const [id, prefix] of [
        [`send.copy_to.${directory.id}`, '复制到'],
        [`send.move_to.${directory.id}`, '移动到']
    ] as const) {
        upsertMenu(config, id, `${prefix}${directory.title}`, directory.enabled);
    }
}

export function resetFavoriteMenus(config: SRightConfig) {
    config.menus = config.menus.filter(menu => !menu.id.startsWith('favorite.open.'));
    for (const directory of config.favorite_dirs) {
        addFavoriteMenu(config, directory);
    }
}

export function resetSendMenus(config: SRightConfig) {
    config.menus = config.menus.filter(
        menu => !menu.id.startsWith('send.copy_to.') && !menu.id.startsWith('send.move_to.')
    );
    for (const directory of config.send_dirs) {
        addSendMenus(config, directory);
    }
}

function directoryPreset(id: string, title: string, path: string): FavoriteDirectory {
    return { id, title, path, enabled: true };
}

function upsertMenu(config: SRightConfig, id: string, title: string, enabled: boolean) {
    const menu = config.menus.find(item => item.id === id);
    if (menu) {
        menu.title = title;
        menu.enabled = enabled;
        menu.dangerous = false;
        return;
    }

    config.menus.push({
        id,
        title,
        enabled,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });
}
