import { Input, Table } from '@heroui/react';
import {
    Archive,
    Code2,
    Copy,
    FileText,
    FolderPlus,
    Hash,
    Image,
    Info,
    QrCode,
    Terminal,
    Trash2,
    Wrench,
    type LucideIcon
} from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { VisibleCheckbox } from '../components/VisibleControls';
import { toolboxMenuItems } from '../lib/toolbox-actions';
import { usePreferenceStore } from '../stores/preferences';

export default function ToolboxView() {
    const config = usePreferenceStore(state => state.config);
    const renameMenu = usePreferenceStore(state => state.renameMenu);
    const reorderMenus = usePreferenceStore(state => state.reorderMenus);
    const setCustomScriptCommand = usePreferenceStore(state => state.setCustomScriptCommand);
    const setDangerousActionConfirmation = usePreferenceStore(state => state.setDangerousActionConfirmation);
    const setMenuEnabled = usePreferenceStore(state => state.setMenuEnabled);
    const [editingActionId, setEditingActionId] = useState<string | null>(null);
    const [draggingActionId, setDraggingActionId] = useState<string | null>(null);

    if (!config) {
        return null;
    }

    const menus = toolboxMenuItems(config.menus);

    function finishActionNameEdit(actionId: string, event: FocusEvent<HTMLInputElement>) {
        setEditingActionId(null);
        void renameMenu(actionId, event.currentTarget.value);
    }

    function commitNameOnEnter(event: KeyboardEvent<HTMLInputElement>) {
        if (event.key === 'Enter') {
            event.currentTarget.blur();
        }
    }

    function beginActionDrag(actionId: string, event: DragEvent) {
        setDraggingActionId(actionId);
        event.dataTransfer.effectAllowed = 'move';
    }

    function dropAction(targetActionId: string) {
        if (!draggingActionId || draggingActionId === targetActionId) {
            setDraggingActionId(null);
            return;
        }

        void reorderMenus(draggingActionId, targetActionId);
        setDraggingActionId(null);
    }

    return (
        <section className="settings-page toolbox-page">
            <Table className="settings-table" aria-label="工具箱右键能力">
                <Table.ScrollContainer className="settings-table-scroll">
                    <Table.Content>
                        <Table.Header>
                            <Table.Column>启用</Table.Column>
                            <Table.Column>图标</Table.Column>
                            <Table.Column isRowHeader>显示名称（双击编辑 / 按住拖拽）</Table.Column>
                            <Table.Column>选项</Table.Column>
                        </Table.Header>
                        <Table.Body>
                            {menus.map(menu => {
                                const scriptId = scriptIdFromActionId(menu.id);

                                return (
                                    <Table.Row key={menu.id} className="toolbox-action-row">
                                        <Table.Cell>
                                            <VisibleCheckbox
                                                aria-label={`启用 ${menu.title}`}
                                                isSelected={menu.enabled}
                                                onChange={enabled => void setMenuEnabled(menu.id, enabled)}
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <ActionIcon actionId={menu.id} />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <div
                                                className="toolbox-drag-handle"
                                                draggable
                                                onDragEnd={() => setDraggingActionId(null)}
                                                onDragOver={event => event.preventDefault()}
                                                onDragStart={event => beginActionDrag(menu.id, event)}
                                                onDrop={() => dropAction(menu.id)}
                                            >
                                                {editingActionId === menu.id ? (
                                                    <Input
                                                        autoFocus
                                                        className="toolbox-name-input"
                                                        defaultValue={menu.title}
                                                        onBlur={event => finishActionNameEdit(menu.id, event)}
                                                        onClick={event => event.stopPropagation()}
                                                        onKeyDown={commitNameOnEnter}
                                                    />
                                                ) : (
                                                    <button
                                                        type="button"
                                                        className="toolbox-name-button"
                                                        onDoubleClick={event => {
                                                            event.stopPropagation();
                                                            setEditingActionId(menu.id);
                                                        }}
                                                    >
                                                        {menu.title}
                                                    </button>
                                                )}
                                            </div>
                                        </Table.Cell>
                                        <Table.Cell>
                                            <ActionOptions
                                                actionId={menu.id}
                                                command={
                                                    scriptId
                                                        ? config.custom_scripts.find(script => script.id === scriptId)
                                                              ?.command
                                                        : undefined
                                                }
                                                isDangerous={menu.dangerous}
                                                requiresConfirmation={config.dangerous_confirmation.action_ids.includes(
                                                    menu.id
                                                )}
                                                onCommandChange={command => {
                                                    if (scriptId) {
                                                        void setCustomScriptCommand(scriptId, command);
                                                    }
                                                }}
                                                onConfirmationChange={enabled =>
                                                    void setDangerousActionConfirmation(menu.id, enabled)
                                                }
                                            />
                                        </Table.Cell>
                                    </Table.Row>
                                );
                            })}
                        </Table.Body>
                    </Table.Content>
                </Table.ScrollContainer>
            </Table>
        </section>
    );
}

function ActionOptions({
    actionId,
    command,
    isDangerous,
    onCommandChange,
    onConfirmationChange,
    requiresConfirmation
}: {
    actionId: string;
    command?: string;
    isDangerous: boolean;
    onCommandChange: (command: string) => void;
    onConfirmationChange: (enabled: boolean) => void;
    requiresConfirmation: boolean;
}) {
    if (isDangerous) {
        return (
            <select
                className="toolbox-option-select"
                aria-label={`${actionId} 确认方式`}
                value={requiresConfirmation ? 'confirm' : 'none'}
                onChange={event => onConfirmationChange(event.currentTarget.value === 'confirm')}
            >
                <option value="confirm">需要再次确认</option>
                <option value="none">不需要确认</option>
            </select>
        );
    }

    if (command !== undefined) {
        return (
            <Input
                className="toolbox-command-input"
                defaultValue={command}
                placeholder="脚本路径或命令"
                onBlur={event => onCommandChange(event.currentTarget.value)}
            />
        );
    }

    return <span className="toolbox-option-empty">--</span>;
}

function ActionIcon({ actionId }: { actionId: string }) {
    const Icon = actionIconFor(actionId);
    return (
        <span className={`toolbox-action-icon ${actionIconClass(actionId)}`}>
            <Icon size={20} />
        </span>
    );
}

function actionIconFor(actionId: string): LucideIcon {
    if (actionId.startsWith('copy.')) {
        return Copy;
    }
    if (actionId.startsWith('file.delete')) {
        return Trash2;
    }
    if (actionId.startsWith('folder.')) {
        return FolderPlus;
    }
    if (actionId.startsWith('open.')) {
        return actionId === 'open.terminal' ? Terminal : Code2;
    }
    if (actionId.startsWith('archive.')) {
        return Archive;
    }
    if (actionId.startsWith('image.') || actionId.startsWith('icon.')) {
        return Image;
    }
    if (actionId.startsWith('tool.hash.')) {
        return Hash;
    }
    if (actionId.startsWith('tool.qr.')) {
        return QrCode;
    }
    if (actionId.startsWith('logs.')) {
        return FileText;
    }
    if (actionId === 'file.info') {
        return Info;
    }
    return Wrench;
}

function actionIconClass(actionId: string) {
    if (actionId.startsWith('image.') || actionId.startsWith('icon.')) {
        return 'image';
    }
    if (actionId.startsWith('archive.')) {
        return 'archive';
    }
    if (actionId.startsWith('file.delete')) {
        return 'danger';
    }
    return 'tool';
}

function scriptIdFromActionId(actionId: string) {
    return actionId.startsWith('script.run.') ? actionId.slice('script.run.'.length) : null;
}
