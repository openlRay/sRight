import { Button, Input, Table, Tooltip } from '@heroui/react';
import { FolderOpen, FolderPlus, RefreshCcw, Trash2 } from 'lucide-react';
import { useState, type DragEvent, type FocusEvent, type KeyboardEvent } from 'react';
import { VisibleCheckbox, VisibleSwitch } from '../components/VisibleControls';
import { usePreferenceStore } from '../stores/preferences';

export default function FavoritesView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const addFavoriteDirFromPicker = usePreferenceStore(state => state.addFavoriteDirFromPicker);
    const openFavoriteDir = usePreferenceStore(state => state.openFavoriteDir);
    const removeFavoriteDir = usePreferenceStore(state => state.removeFavoriteDir);
    const resetFavoriteDirs = usePreferenceStore(state => state.resetFavoriteDirs);
    const renameFavoriteDir = usePreferenceStore(state => state.renameFavoriteDir);
    const reorderFavoriteDirs = usePreferenceStore(state => state.reorderFavoriteDirs);
    const setMenuIconVisibility = usePreferenceStore(state => state.setMenuIconVisibility);
    const toggleAllFavorites = usePreferenceStore(state => state.toggleAllFavorites);
    const [selectedDirectoryId, setSelectedDirectoryId] = useState<string | null>(null);
    const [editingDirectoryId, setEditingDirectoryId] = useState<string | null>(null);
    const [draggingDirectoryId, setDraggingDirectoryId] = useState<string | null>(null);

    if (!config) {
        return null;
    }

    const openMenus = config.menus.filter(menu => menu.id.startsWith('favorite.open.'));
    const favoritesEnabled = openMenus.length > 0 && openMenus.every(menu => menu.enabled);

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

    function openDirectory(directoryId: string) {
        void openFavoriteDir(directoryId);
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
        <section className="settings-page favorites-page">
            <Table className="settings-table" aria-label="常用目录">
                <Table.ScrollContainer className="settings-table-scroll">
                    <Table.Content>
                        <Table.Header>
                            <Table.Column>选择</Table.Column>
                            <Table.Column>图标</Table.Column>
                            <Table.Column>真实路径</Table.Column>
                            <Table.Column isRowHeader>显示名称（双击编辑 / 按住拖拽）</Table.Column>
                        </Table.Header>
                        <Table.Body>
                            {config.favorite_dirs.map(directory => (
                                <Table.Row
                                    key={directory.id}
                                    aria-selected={selectedDirectoryId === directory.id}
                                    onClick={() => openDirectory(directory.id)}
                                >
                                    <Table.Cell>
                                        <span onClick={event => event.stopPropagation()}>
                                            <VisibleCheckbox
                                                aria-label={`选择 ${directory.title}`}
                                                isSelected={selectedDirectoryId === directory.id}
                                                onChange={selected =>
                                                    setSelectedDirectoryId(selected ? directory.id : null)
                                                }
                                            />
                                        </span>
                                    </Table.Cell>
                                    <Table.Cell>
                                        <span className="directory-icon favorite">
                                            <FolderOpen size={22} />
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
                <Button aria-label="增加目录" isDisabled={busy} onPress={() => void addFavoriteDirFromPicker()}>
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
                <Button isDisabled={busy} onPress={() => void resetFavoriteDirs()}>
                    <RefreshCcw size={18} />
                    重置
                </Button>
            </div>

            <div className="template-options">
                <VisibleSwitch
                    size="sm"
                    isSelected={config.menu_icons.favorite_dirs}
                    onChange={enabled => void setMenuIconVisibility('favorite_dirs', enabled)}
                >
                    显示图标
                </VisibleSwitch>
                <VisibleSwitch
                    size="sm"
                    isSelected={favoritesEnabled}
                    onChange={enabled => void toggleAllFavorites(enabled)}
                >
                    启用常用目录
                </VisibleSwitch>
            </div>
        </section>
    );
}
