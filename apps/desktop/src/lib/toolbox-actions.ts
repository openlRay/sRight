interface ToolboxMenuItem {
    id: string;
}

export function isToolboxMenuItem(menu: ToolboxMenuItem) {
    return !menu.id.startsWith('new_file.')
        && !menu.id.startsWith('favorite.open.')
        && !menu.id.startsWith('send.copy_to.')
        && !menu.id.startsWith('send.move_to.');
}

export function toolboxMenuItems<T extends ToolboxMenuItem>(menus: T[]) {
    return menus.filter(isToolboxMenuItem);
}
