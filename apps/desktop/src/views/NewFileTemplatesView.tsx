import { Button, Checkbox, Input, Link } from '@heroui/react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { type FileTemplate } from '../lib/api';
import { usePreferenceStore } from '../stores/preferences';

export default function NewFileTemplatesView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const refresh = usePreferenceStore(state => state.refresh);
    const renameTemplate = usePreferenceStore(state => state.renameTemplate);
    const reorderTemplates = usePreferenceStore(state => state.reorderTemplates);
    const setTemplateEnabled = usePreferenceStore(state => state.setTemplateEnabled);
    const setTemplateMainMenu = usePreferenceStore(state => state.setTemplateMainMenu);
    const setTopLevelFlag = usePreferenceStore(state => state.setTopLevelFlag);
    const [editingTemplateId, setEditingTemplateId] = useState<string | null>(null);
    const [draggingTemplateId, setDraggingTemplateId] = useState<string | null>(null);
    const [soundEnabled, setSoundEnabled] = useState(true);
    const [openAfterCreate, setOpenAfterCreate] = useState(false);

    if (!config) {
        return null;
    }

    function templateMenu(templateId: string) {
        return config?.menus.find(item => item.id === `new_file.${templateId}`) ?? null;
    }

    function isTemplateInMainMenu(templateId: string) {
        return templateMenu(templateId)?.enabled ?? false;
    }

    function templateExtension(fileName: string) {
        const parts = fileName.split('.');
        return parts.length > 1 ? parts[parts.length - 1] : '';
    }

    function templateIconLabel(template: FileTemplate) {
        const extension = templateExtension(template.file_name);
        if (extension.length > 0) {
            return extension.slice(0, 3).toUpperCase();
        }
        return template.title.slice(0, 1).toUpperCase();
    }

    function finishTemplateNameEdit(templateId: string, event: FocusEvent<HTMLInputElement>) {
        setEditingTemplateId(null);
        void renameTemplate(templateId, event.currentTarget.value);
    }

    function commitTemplateNameOnEnter(event: KeyboardEvent<HTMLInputElement>) {
        if (event.key === 'Enter') {
            event.currentTarget.blur();
        }
    }

    function beginTemplateDrag(templateId: string, event: DragEvent) {
        setDraggingTemplateId(templateId);
        event.dataTransfer.effectAllowed = 'move';
    }

    function dropTemplate(targetTemplateId: string) {
        if (!draggingTemplateId || draggingTemplateId === targetTemplateId) {
            setDraggingTemplateId(null);
            return;
        }

        void reorderTemplates(draggingTemplateId, targetTemplateId);
        setDraggingTemplateId(null);
    }

    return (
        <section className="settings-page new-file-page">
            <div className="template-table" role="table" aria-label="新建文件模板">
                <div className="template-row template-head" role="row">
                    <span className="enable-cell" role="columnheader">
                        启用
                    </span>
                    <span role="columnheader">图标</span>
                    <span role="columnheader">显示名称（双击编辑 / 按住拖拽）</span>
                    <span role="columnheader">后缀</span>
                    <span role="columnheader">主菜单</span>
                </div>

                {config.file_templates.map(template => (
                    <div
                        key={template.id}
                        className={`template-row${draggingTemplateId === template.id ? ' dragging' : ''}`}
                        draggable
                        role="row"
                        onDragEnd={() => setDraggingTemplateId(null)}
                        onDragOver={event => event.preventDefault()}
                        onDragStart={event => beginTemplateDrag(template.id, event)}
                        onDrop={() => dropTemplate(template.id)}
                    >
                        <span className="enable-cell" role="cell">
                            <Checkbox
                                isSelected={template.enabled}
                                onChange={enabled => void setTemplateEnabled(template.id, enabled)}
                            />
                        </span>
                        <span role="cell">
                            <span className="template-icon">{templateIconLabel(template)}</span>
                        </span>
                        <span className="template-name-cell" role="cell">
                            {editingTemplateId === template.id ? (
                                <Input
                                    autoFocus
                                    className="template-name-input"
                                    defaultValue={template.title}
                                    onBlur={event => finishTemplateNameEdit(template.id, event)}
                                    onKeyDown={commitTemplateNameOnEnter}
                                />
                            ) : (
                                <Button
                                    className="template-name-button"
                                    onDoubleClick={() => setEditingTemplateId(template.id)}
                                >
                                    {template.title}
                                </Button>
                            )}
                        </span>
                        <span role="cell">
                            <span className="suffix-cell">{templateExtension(template.file_name)}</span>
                        </span>
                        <span className="main-menu-cell" role="cell">
                            <Checkbox
                                isDisabled={!template.enabled}
                                isSelected={isTemplateInMainMenu(template.id)}
                                onChange={enabled => void setTemplateMainMenu(template.id, enabled)}
                            />
                        </span>
                    </div>
                ))}
            </div>

            <div className="template-actions">
                <Button isDisabled>添加模板文件</Button>
                <Button isIconOnly aria-label="删除模板" isDisabled>
                    -
                </Button>
                <Button isIconOnly aria-label="帮助">
                    ?
                </Button>
                <Link href="https://support.apple.com/" target="_blank" rel="noreferrer">
                    右键菜单失效的解决办法 &gt;&gt;
                </Link>
                <Button isDisabled={busy} onPress={() => void refresh()}>
                    重置
                </Button>
            </div>

            <div className="template-options">
                <Checkbox
                    isSelected={config.show_icons}
                    onChange={enabled => void setTopLevelFlag('show_icons', enabled)}
                >
                    显示图标
                </Checkbox>
                <Checkbox isSelected={soundEnabled} onChange={setSoundEnabled}>
                    开启提示音
                </Checkbox>
                <Checkbox isSelected={openAfterCreate} onChange={setOpenAfterCreate}>
                    新建文件后自动打开
                </Checkbox>
            </div>
        </section>
    );
}
