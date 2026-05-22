import type { FileTemplate, SRightConfig } from './api';

export function defaultFileTemplates(): FileTemplate[] {
    return [
        fileTemplate('custom', '自定义创建新文件', 'Untitled'),
        fileTemplate('text', 'TXT', 'Untitled.txt'),
        fileTemplate('rtf', 'RTF', 'Untitled.rtf'),
        fileTemplate('xml', 'XML', 'Untitled.xml'),
        fileTemplate('word', 'Word', 'Untitled.docx'),
        fileTemplate('excel', 'Excel', 'Untitled.xlsx'),
        fileTemplate('ppt', 'PPT', 'Untitled.pptx'),
        fileTemplate('wps_writer', 'WPS 文字', 'Untitled.wps'),
        fileTemplate('wps_spreadsheet', 'WPS 表格', 'Untitled.et'),
        fileTemplate('wps_presentation', 'WPS 演示', 'Untitled.dps'),
        fileTemplate('pages', 'Pages', 'Untitled.pages'),
        fileTemplate('numbers', 'Numbers', 'Untitled.numbers'),
        fileTemplate('keynote', 'Keynote', 'Untitled.key'),
        fileTemplate('ai', 'Ai', 'Untitled.ai'),
        fileTemplate('psd', 'PSD', 'Untitled.psd'),
        fileTemplate('markdown', 'Markdown', 'Untitled.md')
    ];
}

function fileTemplate(id: string, title: string, fileName: string): FileTemplate {
    return {
        id,
        title,
        file_name: fileName,
        enabled: true
    };
}

export function templateMenuId(templateId: string) {
    return templateId.startsWith('new_file.') ? templateId : `new_file.${templateId}`;
}

export function resetTemplateMenus(config: SRightConfig) {
    config.file_templates = defaultFileTemplates();
    config.menus = config.menus.filter(item => !item.id.startsWith('new_file.'));

    for (const template of config.file_templates) {
        syncTemplateMenuTitle(config, template, false);
    }
}

export function syncTemplateMenuTitle(config: SRightConfig, template: FileTemplate, defaultEnabled: boolean) {
    const id = templateMenuId(template.id);
    const menu = config.menus.find(item => item.id === id);

    if (menu) {
        menu.title = template.title;
        return;
    }

    config.menus.push({
        id,
        title: template.title,
        enabled: defaultEnabled,
        main_menu: defaultEnabled,
        dangerous: false,
        file_kinds: [],
        extensions: []
    });
}

export function setTemplateMenuEnabled(config: SRightConfig, templateId: string, enabled: boolean) {
    const id = templateMenuId(templateId);
    const menu = config.menus.find(item => item.id === id);

    if (menu) {
        menu.enabled = enabled;
        menu.main_menu = enabled;
        return;
    }

    const template = config.file_templates.find(item => item.id === templateId || id === templateMenuId(item.id));
    if (!template) {
        return;
    }

    syncTemplateMenuTitle(config, template, enabled);
}

export function isTemplateMenuInMainMenu(config: SRightConfig, templateId: string) {
    const id = templateMenuId(templateId);
    const menu = config.menus.find(item => item.id === id);

    return Boolean(menu?.main_menu ?? menu?.enabled);
}
