import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { defaultDangerousActionIds, defaultToolboxMenus, resetToolboxMenus, toolboxMenuItems } from '../src/lib/toolbox-actions.ts';
import { type SRightConfig } from '../src/lib/api.ts';

const menus = [
    { id: 'copy.path', title: '拷贝路径', enabled: true, dangerous: false, file_kinds: [], extensions: [] },
    { id: 'new_file.text', title: 'TXT', enabled: true, dangerous: false, file_kinds: [], extensions: [] },
    { id: 'file.delete_permanently', title: '彻底删除', enabled: true, dangerous: true, file_kinds: [], extensions: [] },
    { id: 'image.convert.png', title: '图片转 PNG', enabled: true, dangerous: false, file_kinds: [], extensions: [] },
    { id: 'send.copy_to.downloads', title: '复制到下载', enabled: false, dangerous: false, file_kinds: [], extensions: [] },
    { id: 'favorite.open.downloads', title: '打开下载', enabled: true, dangerous: false, file_kinds: [], extensions: [] }
];

assert.deepEqual(
    toolboxMenuItems(menus).map(menu => [menu.id, menu.title]),
    [
        ['copy.path', '拷贝路径'],
        ['file.delete_permanently', '彻底删除'],
        ['image.convert.png', '图片转 PNG']
    ]
);

const toolboxView = readFileSync(new URL('../src/views/ToolboxView.tsx', import.meta.url), 'utf8');
const toolboxStyles = readFileSync(new URL('../src/styles/pages/toolbox.less', import.meta.url), 'utf8');
const store = readFileSync(new URL('../src/stores/preferences.ts', import.meta.url), 'utf8');

assert.match(toolboxView, /setMenuIconVisibility\('toolbox'/);
assert.match(toolboxView, /显示图标/);
assert.match(toolboxView, /resetToolbox/);
assert.match(toolboxView, /removeMenus/);
assert.match(toolboxView, /selectedActionIds/);
assert.match(toolboxView, /<Table\.Column>选择<\/Table\.Column>/);
assert.match(toolboxView, /<Table\.Column>开启<\/Table\.Column>/);
assert.match(toolboxView, /<Table\.Column>主菜单<\/Table\.Column>/);
assert.match(toolboxView, /setMenuMainMenu/);
assert.match(toolboxView, /VisibleSwitch/);
assert.match(toolboxView, /RefreshCcw/);
assert.match(toolboxView, /Select/);
assert.match(toolboxView, /ListBox/);
assert.match(toolboxStyles, /\.settings-table-scroll th:nth-child\(4\),[\s\S]*?\.settings-table-scroll td:nth-child\(4\)[\s\S]*?width: 240px/);
assert.match(toolboxStyles, /\.toolbox-option-select[\s\S]*?width: 100%[\s\S]*?min-width: 190px/);
assert.doesNotMatch(toolboxView, /aria-label=\{`启用 \$\{menu\.title\}`\}[\s\S]*?<VisibleCheckbox/);
assert.doesNotMatch(toolboxView, /<select/);
assert.doesNotMatch(toolboxView, /<option/);
assert.match(store, /config\.dangerous_confirmation\.enabled = true/);
assert.match(store, /removeMenus/);
assert.match(store, /menu\.main_menu = enabled/);

function config(): SRightConfig {
    return {
        enabled: true,
        show_icons: true,
        menu_icons: { new_file: true, send_to: true, favorite_dirs: true, toolbox: true },
        show_menu_bar_icon: true,
        settings_shortcut: '',
        dangerous_confirmation: { enabled: false, action_ids: [] },
        file_templates: [],
        open_apps: [],
        favorite_dirs: [],
        send_dirs: [],
        archive: { delete_source_after_archive: false, delete_archive_after_extract: false },
        image: { output_dir: null, quality: 90 },
        toolbox: { translation_provider: 'apple' },
        removed_menus: [],
        menus: [
            { id: 'new_file.text', title: 'TXT', enabled: true, dangerous: false, file_kinds: [], extensions: [] },
            { id: 'copy.path', title: '改名路径', enabled: false, dangerous: false, file_kinds: [], extensions: [] }
        ],
        menu_tree: []
    };
}

{
    const nextConfig = config();

    resetToolboxMenus(nextConfig);

    assert.equal(nextConfig.menus.find(menu => menu.id === 'copy.path')?.title, '拷贝路径');
    assert.equal(nextConfig.menus.find(menu => menu.id === 'copy.path')?.enabled, true);
    assert.equal(nextConfig.menus.find(menu => menu.id === 'copy.path')?.main_menu, false);
    assert.equal(nextConfig.menus.find(menu => menu.id === 'new_file.text')?.title, 'TXT');
    assert.equal(nextConfig.dangerous_confirmation.enabled, true);
    assert.deepEqual(nextConfig.dangerous_confirmation.action_ids, defaultDangerousActionIds());
    assert.deepEqual(defaultDangerousActionIds(), ['file.delete_permanently']);
}

{
    const menus = defaultToolboxMenus();
    const ids = menus.map(menu => menu.id);

    for (const id of [
        'icon.set_custom',
        'icon.remove_custom',
        'tool.copy_summary',
        'script.run.default',
        'logs.search',
        'logs.export'
    ]) {
        assert.equal(ids.includes(id), false, `removed toolbox action should not exist ${id}`);
    }

    for (const id of [
        'file.shortcut_desktop',
        'share.airdrop',
        'tool.hash.sha512',
        'tool.qr.file',
        'file.cut',
        'favorite.add_selected',
        'permission.grant_write',
        'visibility.unhide_all',
        'visibility.hide_all',
        'visibility.unhide_selected',
        'visibility.hide_selected',
        'finder.show_extensions',
        'finder.hide_extensions'
    ]) {
        assert.ok(ids.includes(id), `missing toolbox action ${id}`);
    }
}
