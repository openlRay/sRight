import { Button, Checkbox, Input } from '@heroui/react';
import { Folder, Minus, Plus } from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { usePreferenceStore } from '../stores/preferences';

export default function SendToView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const addFavoriteDirFromPicker = usePreferenceStore(state => state.addFavoriteDirFromPicker);
    const refresh = usePreferenceStore(state => state.refresh);
    const removeFavoriteDir = usePreferenceStore(state => state.removeFavoriteDir);
    const renameFavoriteDir = usePreferenceStore(state => state.renameFavoriteDir);
    const reorderFavoriteDirs = usePreferenceStore(state => state.reorderFavoriteDirs);
    const setTopLevelFlag = usePreferenceStore(state => state.setTopLevelFlag);
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
        void renameFavoriteDir(directoryId, event.currentTarget.value);
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

        void reorderFavoriteDirs(draggingDirectoryId, targetDirectoryId);
        setDraggingDirectoryId(null);
    }

    function removeSelectedDirectory() {
        if (!selectedDirectoryId) {
            return;
        }

        const removedId = selectedDirectoryId;
        setSelectedDirectoryId(null);
        void removeFavoriteDir(removedId);
    }

    return (
        <section className="settings-page send-to-page">
            <div className="send-to-table" role="table" aria-label="发送文件到目录">
                <div className="send-to-row send-to-head" role="row">
                    <span role="columnheader">图标</span>
                    <span role="columnheader">真实路径</span>
                    <span role="columnheader">显示名称（双击编辑 / 按住拖拽）</span>
                </div>

                {config.favorite_dirs.map(directory => (
                    <div
                        key={directory.id}
                        className={`send-to-row send-to-data-row${selectedDirectoryId === directory.id ? ' selected' : ''}${
                            draggingDirectoryId === directory.id ? ' dragging' : ''
                        }`}
                        draggable
                        role="row"
                        tabIndex={0}
                        onClick={() => setSelectedDirectoryId(directory.id)}
                        onDragEnd={() => setDraggingDirectoryId(null)}
                        onDragOver={event => event.preventDefault()}
                        onDragStart={event => beginDirectoryDrag(directory.id, event)}
                        onDrop={() => dropDirectory(directory.id)}
                    >
                        <span role="cell">
                            <Folder size={30} />
                        </span>
                        <span className="path-cell" role="cell">
                            {directory.path}
                        </span>
                        <span className="directory-name-cell" role="cell">
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
                                <span
                                    className="directory-name-button"
                                    onDoubleClick={event => {
                                        event.stopPropagation();
                                        setEditingDirectoryId(directory.id);
                                    }}
                                >
                                    {directory.title}
                                </span>
                            )}
                        </span>
                    </div>
                ))}
            </div>

            <div className="send-to-actions">
                <Button
                    isIconOnly
                    aria-label="增加目录"
                    isDisabled={busy}
                    onPress={() => void addFavoriteDirFromPicker()}
                >
                    <Plus size={18} />
                </Button>
                <Button
                    isIconOnly
                    aria-label="删除目录"
                    isDisabled={busy || !selectedDirectoryId}
                    onPress={removeSelectedDirectory}
                >
                    <Minus size={18} />
                </Button>
                <Button isDisabled={busy} onPress={() => void refresh()}>
                    重置
                </Button>
            </div>

            <div className="send-to-options">
                <Checkbox
                    isSelected={config.show_icons}
                    onChange={enabled => void setTopLevelFlag('show_icons', enabled)}
                >
                    显示图标
                </Checkbox>
                <Checkbox isSelected={moveEnabled} onChange={enabled => void toggleSendMenus('send.move_to.', enabled)}>
                    启用移动文件到...
                </Checkbox>
                <Checkbox isSelected={copyEnabled} onChange={enabled => void toggleSendMenus('send.copy_to.', enabled)}>
                    启用复制文件到...
                </Checkbox>
            </div>
        </section>
    );
}
