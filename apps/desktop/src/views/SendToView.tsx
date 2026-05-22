import { Button, Input, Table, Tooltip } from '@heroui/react';
import { FolderInput, FolderPlus, RefreshCcw, Trash2 } from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { VisibleCheckbox, VisibleSwitch } from '../components/VisibleControls';
import { usePreferenceStore } from '../stores/preferences';

export default function SendToView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const addSendDirFromPicker = usePreferenceStore(state => state.addSendDirFromPicker);
    const removeSendDir = usePreferenceStore(state => state.removeSendDir);
    const renameSendDir = usePreferenceStore(state => state.renameSendDir);
    const reorderSendDirs = usePreferenceStore(state => state.reorderSendDirs);
    const resetSendDirs = usePreferenceStore(state => state.resetSendDirs);
    const setMenuIconVisibility = usePreferenceStore(state => state.setMenuIconVisibility);
    const toggleSendMenus = usePreferenceStore(state => state.toggleSendMenus);
    const [selectedDirectoryId, setSelectedDirectoryId] = useState<string | null>(null);
    const [editingDirectoryId, setEditingDirectoryId] = useState<string | null>(null);
    const [draggingDirectoryId, setDraggingDirectoryId] = useState<string | null>(null);

    if (!config) {
        return null;
    }

    const copyMenus = config.menus.filter(menu => menu.id.startsWith('send.copy_to.'));
    const copyEnabled = copyMenus.length > 0 && copyMenus.every(menu => menu.enabled);
    const moveMenus = config.menus.filter(menu => menu.id.startsWith('send.move_to.'));
    const moveEnabled = moveMenus.length > 0 && moveMenus.every(menu => menu.enabled);

    function finishDirectoryNameEdit(directoryId: string, event: FocusEvent<HTMLInputElement>) {
        setEditingDirectoryId(null);
        void renameSendDir(directoryId, event.currentTarget.value);
    }

    function commitDirectoryNameOnEnter(event: KeyboardEvent<HTMLInputElement>) {
        if (event.key === 'Enter') {
            event.currentTarget.blur();
        }
    }

    function beginDirectoryDrag(directoryId: string, event: DragEvent) {
        setDraggingDirectoryId(directoryId);
        event.dataTransfer.effectAllowed = 'move';
    }

    function dropDirectory(targetDirectoryId: string) {
        if (!draggingDirectoryId || draggingDirectoryId === targetDirectoryId) {
            setDraggingDirectoryId(null);
            return;
        }

        void reorderSendDirs(draggingDirectoryId, targetDirectoryId);
        setDraggingDirectoryId(null);
    }

    function removeSelectedDirectory() {
        if (!selectedDirectoryId) {
            return;
        }

        const removedId = selectedDirectoryId;
        setSelectedDirectoryId(null);
        void removeSendDir(removedId);
    }

    return (
        <section className="settings-page send-to-page">
            <Table className="settings-table" aria-label="发送文件到目录">
                <Table.ScrollContainer className="settings-table-scroll">
                    <Table.Content>
                        <Table.Header>
                            <Table.Column>选择</Table.Column>
                            <Table.Column>图标</Table.Column>
                            <Table.Column>真实路径</Table.Column>
                            <Table.Column isRowHeader>显示名称（双击编辑 / 按住拖拽）</Table.Column>
                        </Table.Header>
                        <Table.Body>
                            {config.send_dirs.map(directory => (
                                <Table.Row key={directory.id}>
                                    <Table.Cell>
                                        <VisibleCheckbox
                                            aria-label={`选择 ${directory.title}`}
                                            isSelected={selectedDirectoryId === directory.id}
                                            onChange={selected =>
                                                setSelectedDirectoryId(selected ? directory.id : null)
                                            }
                                        />
                                    </Table.Cell>
                                    <Table.Cell>
                                        <span className="directory-icon send">
                                            <FolderInput size={22} />
                                        </span>
                                    </Table.Cell>
                                    <Table.Cell>
                                        <Tooltip delay={0}>
                                            <Tooltip.Trigger className="path-cell">
                                                <span>{directory.path}</span>
                                            </Tooltip.Trigger>
                                            <Tooltip.Content showArrow>
                                                <Tooltip.Arrow />
                                                {directory.path}
                                            </Tooltip.Content>
                                        </Tooltip>
                                    </Table.Cell>
                                    <Table.Cell>
                                        <div
                                            className="directory-drag-handle"
                                            draggable
                                            onDragEnd={() => setDraggingDirectoryId(null)}
                                            onDragOver={event => event.preventDefault()}
                                            onDragStart={event => beginDirectoryDrag(directory.id, event)}
                                            onDrop={() => dropDirectory(directory.id)}
                                        >
                                            {editingDirectoryId === directory.id ? (
                                                <Input
                                                    autoFocus
                                                    className="directory-name-input"
                                                    defaultValue={directory.title}
                                                    onBlur={event => finishDirectoryNameEdit(directory.id, event)}
                                                    onClick={event => event.stopPropagation()}
                                                    onKeyDown={commitDirectoryNameOnEnter}
                                                />
                                            ) : (
                                                <Tooltip delay={0}>
                                                    <Tooltip.Trigger
                                                        className="directory-name-button"
                                                        onDoubleClick={event => {
                                                            event.stopPropagation();
                                                            setEditingDirectoryId(directory.id);
                                                        }}
                                                    >
                                                        <span>{directory.title}</span>
                                                    </Tooltip.Trigger>
                                                    <Tooltip.Content showArrow>
                                                        <Tooltip.Arrow />
                                                        {directory.title}
                                                    </Tooltip.Content>
                                                </Tooltip>
                                            )}
                                        </div>
                                    </Table.Cell>
                                </Table.Row>
                            ))}
                        </Table.Body>
                    </Table.Content>
                </Table.ScrollContainer>
            </Table>

            <div className="template-actions">
                <Button aria-label="增加目录" isDisabled={busy} onPress={() => void addSendDirFromPicker()}>
                    <FolderPlus size={18} />
                    增加目录
                </Button>
                <Button
                    aria-label="删除目录"
                    isDisabled={busy || !selectedDirectoryId}
                    onPress={removeSelectedDirectory}
                >
                    <Trash2 size={18} />
                    删除目录
                </Button>
                <Button isDisabled={busy} onPress={() => void resetSendDirs()}>
                    <RefreshCcw size={18} />
                    重置
                </Button>
            </div>

            <div className="template-options">
                <VisibleSwitch
                    isSelected={config.menu_icons.send_to}
                    onChange={enabled => void setMenuIconVisibility('send_to', enabled)}
                >
                    显示图标
                </VisibleSwitch>
                <VisibleSwitch
                    isSelected={moveEnabled}
                    onChange={enabled => void toggleSendMenus('send.move_to.', enabled)}
                >
                    启用移动文件到...
                </VisibleSwitch>
                <VisibleSwitch
                    isSelected={copyEnabled}
                    onChange={enabled => void toggleSendMenus('send.copy_to.', enabled)}
                >
                    启用复制文件到...
                </VisibleSwitch>
            </div>
        </section>
    );
}
