import { Button, Input, ListBox, Select, Table } from '@heroui/react';
import {
    Archive,
    Code2,
    Copy,
    FolderPlus,
    Hash,
    Image,
    Info,
    QrCode,
    RefreshCcw,
    Terminal,
    Trash2,
    Wrench,
    type LucideIcon
} from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { VisibleCheckbox, VisibleSwitch } from '../components/VisibleControls';
import { toolboxMenuItems } from '../lib/toolbox-actions';
import { usePreferenceStore } from '../stores/preferences';

export default function ToolboxView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const renameMenu = usePreferenceStore(state => state.renameMenu);
    const removeMenus = usePreferenceStore(state => state.removeMenus);
    const reorderMenus = usePreferenceStore(state => state.reorderMenus);
    const resetToolbox = usePreferenceStore(state => state.resetToolbox);
    const setDangerousActionConfirmation = usePreferenceStore(state => state.setDangerousActionConfirmation);
    const setMenuIconVisibility = usePreferenceStore(state => state.setMenuIconVisibility);
    const setMenuEnabled = usePreferenceStore(state => state.setMenuEnabled);
    const setMenuMainMenu = usePreferenceStore(state => state.setMenuMainMenu);
    const [editingActionId, setEditingActionId] = useState<string | null>(null);
    const [draggingActionId, setDraggingActionId] = useState<string | null>(null);
    const [selectedActionIds, setSelectedActionIds] = useState<string[]>([]);

    if (!config) {
        return null;
    }

    const menus = toolboxMenuItems(config.menus);
    const selectedActionIdSet = new Set(selectedActionIds);

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

    function toggleActionSelection(actionId: string, selected: boolean) {
        setSelectedActionIds(current => {
            if (selected) {
                return current.includes(actionId) ? current : [...current, actionId];
            }

            return current.filter(id => id !== actionId);
        });
    }

    function removeSelectedActions() {
        if (selectedActionIds.length === 0) {
            return;
        }

        void removeMenus(selectedActionIds);
        setSelectedActionIds([]);
    }

    return (
        <section className="settings-page toolbox-page">
            <Table className="settings-table" aria-label="工具箱右键能力">
                <Table.ScrollContainer className="settings-table-scroll">
                    <Table.Content>
                        <Table.Header>
                            <Table.Column>选择</Table.Column>
                            <Table.Column>图标</Table.Column>
                            <Table.Column isRowHeader>显示名称（双击编辑 / 按住拖拽）</Table.Column>
                            <Table.Column>选项</Table.Column>
                            <Table.Column>开启</Table.Column>
                            <Table.Column>主菜单</Table.Column>
                        </Table.Header>
                        <Table.Body>
                            {menus.map(menu => (
                                    <Table.Row key={menu.id} className="toolbox-action-row">
                                        <Table.Cell>
                                            <VisibleCheckbox
                                                aria-label={`选择 ${menu.title}`}
                                                isSelected={selectedActionIdSet.has(menu.id)}
                                                onChange={selected => toggleActionSelection(menu.id, selected)}
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
                                                isDangerous={menu.dangerous}
                                                requiresConfirmation={
                                                    config.dangerous_confirmation.enabled
                                                    && config.dangerous_confirmation.action_ids.includes(menu.id)
                                                }
                                                onConfirmationChange={enabled =>
                                                    void setDangerousActionConfirmation(menu.id, enabled)
                                                }
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <VisibleSwitch
                                                aria-label={`开启 ${menu.title}`}
                                                size="sm"
                                                isSelected={menu.enabled}
                                                onChange={enabled => void setMenuEnabled(menu.id, enabled)}
                                            />
                                        </Table.Cell>
                                        <Table.Cell>
                                            <VisibleSwitch
                                                aria-label={`${menu.title} 显示在主菜单`}
                                                size="sm"
                                                isDisabled={!menu.enabled}
                                                isSelected={Boolean(menu.main_menu)}
                                                onChange={enabled => void setMenuMainMenu(menu.id, enabled)}
                                            />
                                        </Table.Cell>
                                    </Table.Row>
                            ))}
                        </Table.Body>
                    </Table.Content>
                </Table.ScrollContainer>
            </Table>

            <div className="template-actions">
                <Button isDisabled={busy || selectedActionIds.length === 0} onPress={removeSelectedActions}>
                    <Trash2 size={18} />
                    删除选中
                </Button>
                <Button isDisabled={busy} onPress={() => void resetToolbox()}>
                    <RefreshCcw size={18} />
                    重置
                </Button>
            </div>

            <div className="template-options">
                <VisibleSwitch
                    isSelected={config.menu_icons.toolbox}
                    onChange={enabled => void setMenuIconVisibility('toolbox', enabled)}
                >
                    显示图标
                </VisibleSwitch>
            </div>
        </section>
    );
}

function ActionOptions({
    actionId,
    isDangerous,
    onConfirmationChange,
    requiresConfirmation
}: {
    actionId: string;
    isDangerous: boolean;
    onConfirmationChange: (enabled: boolean) => void;
    requiresConfirmation: boolean;
}) {
    if (isDangerous) {
        return (
            <Select
                className="toolbox-option-select"
                aria-label={`${actionId} 确认方式`}
                selectedKey={requiresConfirmation ? 'confirm' : 'none'}
                onSelectionChange={key => onConfirmationChange(key === 'confirm')}
            >
                <Select.Trigger>
                    <Select.Value />
                    <Select.Indicator />
                </Select.Trigger>
                <Select.Popover>
                    <ListBox>
                        <ListBox.Item id="confirm" textValue="需要再次确认">
                            需要再次确认
                        </ListBox.Item>
                        <ListBox.Item id="none" textValue="不需要确认">
                            不需要确认
                        </ListBox.Item>
                    </ListBox>
                </Select.Popover>
            </Select>
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
