import { Button, Input, Table } from '@heroui/react';
import { FilePlus2, RefreshCcw, Trash2 } from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { VisibleCheckbox, VisibleSwitch } from '../components/VisibleControls';
import { type FileTemplate } from '../lib/api';
import { isTemplateMenuInMainMenu } from '../lib/template-menu-config';
import { usePreferenceStore } from '../stores/preferences';

const templateIconMeta: Record<string, { className: string; label: string; mark: string }> = {
    ai: { className: 'ai', label: 'Illustrator 文件', mark: 'Ai' },
    docx: { className: 'word', label: 'Word 文件', mark: 'W' },
    dps: { className: 'presentation', label: 'WPS 演示文件', mark: 'D' },
    et: { className: 'sheet', label: 'WPS 表格文件', mark: 'E' },
    json: { className: 'json', label: 'JSON 文件', mark: '{}' },
    key: { className: 'keynote', label: 'Keynote 文件', mark: 'K' },
    markdown: { className: 'markdown', label: 'Markdown 文件', mark: 'M' },
    md: { className: 'markdown', label: 'Markdown 文件', mark: 'M' },
    numbers: { className: 'numbers', label: 'Numbers 文件', mark: 'N' },
    pages: { className: 'pages', label: 'Pages 文件', mark: 'P' },
    pptx: { className: 'presentation', label: 'PPT 文件', mark: 'P' },
    psd: { className: 'psd', label: 'Photoshop 文件', mark: 'Ps' },
    rtf: { className: 'rtf', label: 'RTF 文件', mark: 'R' },
    txt: { className: 'text', label: '文本文件', mark: 'T' },
    wps: { className: 'word', label: 'WPS 文字文件', mark: 'W' },
    xlsx: { className: 'sheet', label: 'Excel 文件', mark: 'X' },
    xml: { className: 'xml', label: 'XML 文件', mark: '</>' }
};

export default function NewFileTemplatesView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const renameTemplate = usePreferenceStore(state => state.renameTemplate);
    const reorderTemplates = usePreferenceStore(state => state.reorderTemplates);
    const setTemplateEnabled = usePreferenceStore(state => state.setTemplateEnabled);
    const setTemplateMainMenu = usePreferenceStore(state => state.setTemplateMainMenu);
    const setMenuIconVisibility = usePreferenceStore(state => state.setMenuIconVisibility);
    const addTemplateFromPicker = usePreferenceStore(state => state.addTemplateFromPicker);
    const removeTemplates = usePreferenceStore(state => state.removeTemplates);
    const resetTemplates = usePreferenceStore(state => state.resetTemplates);
    const [selectedTemplateIds, setSelectedTemplateIds] = useState<string[]>([]);
    const [editingTemplateId, setEditingTemplateId] = useState<string | null>(null);
    const [draggingTemplateId, setDraggingTemplateId] = useState<string | null>(null);
    const [openAfterCreate, setOpenAfterCreate] = useState(false);

    if (!config) {
        return null;
    }

    function isTemplateInMainMenu(templateId: string) {
        return config ? isTemplateMenuInMainMenu(config, templateId) : false;
    }

    function templateExtension(fileName: string) {
        const parts = fileName.split('.');
        return parts.length > 1 ? parts[parts.length - 1] : '';
    }

    function templateIcon(template: FileTemplate) {
        const extension = templateExtension(template.file_name).toLowerCase();
        return (
            templateIconMeta[extension] ?? {
                className: 'custom',
                label: '自定义模板文件',
                mark: extension.slice(0, 2).toUpperCase() || '+'
            }
        );
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

    function toggleTemplateSelection(templateId: string, selected: boolean) {
        setSelectedTemplateIds(currentIds => {
            if (selected) {
                return currentIds.includes(templateId) ? currentIds : [...currentIds, templateId];
            }

            return currentIds.filter(id => id !== templateId);
        });
    }

    function removeSelectedTemplates() {
        if (selectedTemplateIds.length === 0) {
            return;
        }

        const removedIds = selectedTemplateIds;
        setSelectedTemplateIds([]);
        void removeTemplates(removedIds);
    }

    return (
        <section className="settings-page new-file-page">
            <Table className="settings-table" aria-label="新建文件模板">
                <Table.ScrollContainer className="settings-table-scroll">
                    <Table.Content>
                        <Table.Header>
                            <Table.Column>选择</Table.Column>
                            <Table.Column>图标</Table.Column>
                            <Table.Column isRowHeader>显示名称（双击编辑 / 按住拖拽）</Table.Column>
                            <Table.Column>后缀</Table.Column>
                            <Table.Column>启用</Table.Column>
                            <Table.Column>主菜单</Table.Column>
                        </Table.Header>
                        <Table.Body>
                            {config.file_templates.map(template => {
                                const icon = templateIcon(template);

                                return (
                                    <Table.Row key={template.id}>
                                        <Table.Cell>
                                            <VisibleCheckbox
                                                aria-label={`选择 ${template.title}`}
                                                isSelected={selectedTemplateIds.includes(template.id)}
                                                onChange={selected => toggleTemplateSelection(template.id, selected)}
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <FileTemplateIcon
                                                className={icon.className}
                                                label={icon.label}
                                                mark={icon.mark}
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <div
                                                className="template-drag-handle"
                                                draggable
                                                onDragEnd={() => setDraggingTemplateId(null)}
                                                onDragOver={event => event.preventDefault()}
                                                onDragStart={event => beginTemplateDrag(template.id, event)}
                                                onDrop={() => dropTemplate(template.id)}
                                            >
                                                {editingTemplateId === template.id ? (
                                                    <Input
                                                        autoFocus
                                                        className="template-name-input"
                                                        defaultValue={template.title}
                                                        onBlur={event => finishTemplateNameEdit(template.id, event)}
                                                        onClick={event => event.stopPropagation()}
                                                        onKeyDown={commitTemplateNameOnEnter}
                                                    />
                                                ) : (
                                                    <button
                                                        type="button"
                                                        className="template-name-button"
                                                        onDoubleClick={event => {
                                                            event.stopPropagation();
                                                            setEditingTemplateId(template.id);
                                                        }}
                                                    >
                                                        {template.title}
                                                    </button>
                                                )}
                                            </div>
                                        </Table.Cell>
                                        <Table.Cell>
                                            <span className="suffix-cell">{templateExtension(template.file_name)}</span>
                                        </Table.Cell>
                                        <Table.Cell>
                                            <VisibleSwitch
                                                aria-label={`启用 ${template.title}`}
                                                size="sm"
                                                isSelected={template.enabled}
                                                onChange={enabled => void setTemplateEnabled(template.id, enabled)}
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <VisibleSwitch
                                                aria-label={`${template.title} 显示在主菜单`}
                                                size="sm"
                                                isDisabled={!template.enabled}
                                                isSelected={isTemplateInMainMenu(template.id)}
                                                onChange={enabled => void setTemplateMainMenu(template.id, enabled)}
                                            />
                                        </Table.Cell>
                                    </Table.Row>
                                );
                            })}
                        </Table.Body>
                    </Table.Content>
                </Table.ScrollContainer>
            </Table>

            <div className="template-actions">
                <Button isDisabled={busy} onPress={() => void addTemplateFromPicker()}>
                    <FilePlus2 size={16} />
                    添加模板文件
                </Button>
                <Button
                    variant="danger"
                    aria-label="删除模板"
                    isDisabled={busy || selectedTemplateIds.length === 0}
                    onPress={removeSelectedTemplates}
                >
                    <Trash2 size={18} />
                    删除选中行
                </Button>
                <Button isDisabled={busy} onPress={() => void resetTemplates()}>
                    <RefreshCcw size={18} />
                    重置
                </Button>
            </div>

            <div className="template-options">
                <VisibleSwitch
                    isSelected={config.menu_icons.new_file}
                    onChange={enabled => void setMenuIconVisibility('new_file', enabled)}
                >
                    显示图标
                </VisibleSwitch>
                <VisibleSwitch isSelected={openAfterCreate} onChange={setOpenAfterCreate}>
                    新建文件后自动打开
                </VisibleSwitch>
            </div>
        </section>
    );
}

function FileTemplateIcon({ className, label, mark }: { className: string; label: string; mark: string }) {
    return (
        <span className={`template-file-icon ${className}`} aria-label={label} role="img">
            <span className="file-fold" />
            <span className="file-lines" />
            <span className="file-mark">{mark}</span>
        </span>
    );
}
