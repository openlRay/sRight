import assert from 'node:assert/strict';
import { toolboxMenuItems } from '../src/lib/toolbox-actions.ts';

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
