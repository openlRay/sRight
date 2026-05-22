import type { MenuItem, SRightConfig } from './api';

interface ToolboxMenuItem {
    id: string;
}

interface DefaultToolboxAction {
    id: string;
    title: string;
    dangerous: boolean;
    enabled: boolean;
}

const defaultToolboxActions: DefaultToolboxAction[] = [
    action('copy.path', '拷贝路径'),
    action('copy.name', '拷贝文件(夹)名称'),
    action('file.delete_permanently', '彻底删除', true),
    action('folder.create_from_filename', '根据文件名创建文件夹'),
    action('folder.dissolve', '解散文件夹'),
    action('file.info', '查看文件信息'),
    action('file.shortcut_desktop', '发送快捷方式到桌面'),
    action('share.airdrop', '隔空投送'),
    action('file.cut', '剪切'),
    action('favorite.add_selected', '添加到常用目录'),
    action('permission.grant_write', '授予选择的文件写入权限'),
    action('visibility.unhide_all', '取消隐藏所有文件'),
    action('visibility.hide_all', '隐藏所有文件'),
    action('visibility.unhide_selected', '取消隐藏已选文件'),
    action('visibility.hide_selected', '隐藏已选文件'),
    action('finder.show_extensions', '显示文件扩展名'),
    action('finder.hide_extensions', '隐藏文件扩展名'),
    action('open.terminal', '在 Terminal 中打开'),
    action('open.vscode', '在 VSCode 中打开'),
    action('open.cursor', '在 Cursor 中打开'),
    action('archive.zip', '压缩为 ZIP'),
    action('archive.zip_each', '每项单独压缩为 ZIP'),
    action('archive.unzip_here', '解压到当前目录'),
    action('archive.unzip_to_folder', '解压到单独目录'),
    action('image.convert.png', '图片转 PNG'),
    action('image.convert.jpg', '图片转 JPG'),
    action('image.convert.webp', '图片转 WebP'),
    action('image.convert.heic', '图片转 HEIC'),
    action('icon.make_iconset', '生成 iconset'),
    action('icon.make_icns', '生成 ICNS'),
    action('tool.hash.md5', '计算 MD5'),
    action('tool.hash.sha1', '计算 SHA1'),
    action('tool.hash.sha256', '计算 SHA256'),
    action('tool.hash.sha512', '计算 SHA512'),
    action('tool.qr.file', '选中图片/文件生成二维码'),
    action('tool.open_parent', '打开所在目录')
];

function action(id: string, title: string, dangerous = false): DefaultToolboxAction {
    return { id, title, dangerous, enabled: true };
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

export function defaultToolboxMenus(): MenuItem[] {
    return defaultToolboxActions.map(item => ({
        id: item.id,
        title: item.title,
        enabled: item.enabled,
        main_menu: false,
        dangerous: item.dangerous,
        file_kinds: [],
        extensions: []
    }));
}

export function defaultDangerousActionIds() {
    return defaultToolboxActions.filter(item => item.dangerous).map(item => item.id);
}

export function resetToolboxMenus(config: SRightConfig) {
    config.menus = config.menus.filter(item => !isToolboxMenuItem(item));
    config.removed_menus = [];
    config.menus.push(...defaultToolboxMenus());
    config.dangerous_confirmation.enabled = true;
    config.dangerous_confirmation.action_ids = defaultDangerousActionIds();
}
