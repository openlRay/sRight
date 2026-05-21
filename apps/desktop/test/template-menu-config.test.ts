import assert from 'node:assert/strict';
import { setTemplateMenuEnabled, syncTemplateMenuTitle } from '../src/lib/template-menu-config.ts';
import { type SRightConfig } from '../src/lib/api.ts';

function config(): SRightConfig {
    return {
        enabled: true,
        show_icons: true,
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
        custom_scripts: [],
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
}
