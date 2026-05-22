import assert from 'node:assert/strict';
import {
    isTemplateMenuInMainMenu,
    resetTemplateMenus,
    setTemplateMenuEnabled,
    syncTemplateMenuTitle
} from '../src/lib/template-menu-config.ts';
import { type SRightConfig } from '../src/lib/api.ts';

function config(): SRightConfig {
    return {
        enabled: true,
        show_icons: true,
        menu_icons: { new_file: true, send_to: true, favorite_dirs: true, toolbox: true },
        show_menu_bar_icon: true,
        settings_shortcut: '',
        dangerous_confirmation: { enabled: true, action_ids: [] },
        file_templates: [{ id: 'psd', title: 'PSD', file_name: 'Untitled.psd', enabled: true }],
        open_apps: [],
        favorite_dirs: [],
        send_dirs: [],
        archive: { delete_source_after_archive: false, delete_archive_after_extract: false },
        image: { output_dir: null, quality: 90 },
        toolbox: { translation_provider: 'apple' },
        removed_menus: [],
        menus: [],
        menu_tree: []
    };
}

{
    const nextConfig = config();
    syncTemplateMenuTitle(nextConfig, nextConfig.file_templates[0], false);

    assert.deepEqual(nextConfig.menus.find(menu => menu.id === 'new_file.psd'), {
        id: 'new_file.psd',
        title: 'PSD',
        enabled: false,
        main_menu: false,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });
}

{
    const nextConfig = config();
    nextConfig.menus.push({
        id: 'new_file.psd',
        title: 'PSD',
        enabled: false,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });

    nextConfig.file_templates[0].title = 'Photoshop';
    syncTemplateMenuTitle(nextConfig, nextConfig.file_templates[0], false);

    assert.equal(nextConfig.menus[0].title, 'Photoshop');
    assert.equal(nextConfig.menus[0].enabled, false);
}

{
    const nextConfig = config();
    setTemplateMenuEnabled(nextConfig, 'psd', true);

    assert.equal(nextConfig.menus.find(menu => menu.id === 'new_file.psd')?.enabled, true);
    assert.equal(nextConfig.menus.find(menu => menu.id === 'new_file.psd')?.main_menu, true);
    assert.equal(isTemplateMenuInMainMenu(nextConfig, 'psd'), true);
}

{
    const nextConfig = config();
    nextConfig.menus.push({
        id: 'new_file.psd',
        title: 'PSD',
        enabled: true,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });

    assert.equal(isTemplateMenuInMainMenu(nextConfig, 'psd'), true);
}

{
    const nextConfig = config();
    nextConfig.file_templates.push({ id: 'custom_2', title: 'Sketch', file_name: 'Untitled.sketch', enabled: true });
    nextConfig.menus.push({
        id: 'new_file.custom_2',
        title: 'Sketch',
        enabled: true,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });

    resetTemplateMenus(nextConfig);

    assert.equal(nextConfig.file_templates.some(template => template.id === 'custom_2'), false);
    assert.equal(nextConfig.menus.some(menu => menu.id === 'new_file.custom_2'), false);
    assert.equal(nextConfig.file_templates.find(template => template.id === 'text')?.file_name, 'Untitled.txt');
    assert.equal(nextConfig.menus.find(menu => menu.id === 'new_file.text')?.enabled, false);
    assert.equal(nextConfig.menus.find(menu => menu.id === 'new_file.text')?.main_menu, false);
}
