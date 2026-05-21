import type { FileTemplate, SRightConfig } from './api';

export function templateMenuId(templateId: string) {
    return templateId.startsWith('new_file.') ? templateId : `new_file.${templateId}`;
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
        return;
    }

    const template = config.file_templates.find(item => item.id === templateId || id === templateMenuId(item.id));
    if (!template) {
        return;
    }

    syncTemplateMenuTitle(config, template, enabled);
}
