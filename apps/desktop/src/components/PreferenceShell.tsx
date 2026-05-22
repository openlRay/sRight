import { Spinner } from '@heroui/react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useRef, type PointerEvent } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { closeWindow, minimizeWindow } from '../lib/api';
import { preferenceSections } from '../router/sections';
import { usePreferenceStore } from '../stores/preferences';

const appIconUrl = new URL('../../src-tauri/icons/icon.png', import.meta.url).href;
const dragStartDelay = 180;
const panelDragBlockedSelector = 'a, button, input, select, textarea, [role="button"], [data-no-panel-drag]';

export default function PreferenceShell() {
    const config = usePreferenceStore(state => state.config);
    const refresh = usePreferenceStore(state => state.refresh);
    const dragTimer = useRef<number | null>(null);

    useEffect(() => {
        void refresh();
    }, [refresh]);

    useEffect(() => clearPanelDragTimer, []);

    function clearPanelDragTimer() {
        if (dragTimer.current === null) {
            return;
        }

        window.clearTimeout(dragTimer.current);
        dragTimer.current = null;
    }

    function beginPanelDrag(event: PointerEvent<HTMLElement>) {
        if (event.button !== 0) {
            return;
        }

        const target = event.target as HTMLElement;
        if (target.closest(panelDragBlockedSelector)) {
            return;
        }

        clearPanelDragTimer();
        dragTimer.current = window.setTimeout(() => {
            dragTimer.current = null;
            void getCurrentWindow().startDragging();
        }, dragStartDelay);
    }

    return (
        <main
            className="preference-window"
            onPointerCancel={clearPanelDragTimer}
            onPointerDown={beginPanelDrag}
            onPointerLeave={clearPanelDragTimer}
            onPointerUp={clearPanelDragTimer}
        >
            <aside className="sidebar">
                <div className="traffic-lights" data-tauri-drag-region>
                    <button
                        aria-label="关闭窗口"
                        className="traffic-light close"
                        type="button"
                        onClick={() => void closeWindow()}
                    />
                    <button
                        aria-label="最小化窗口"
                        className="traffic-light minimize"
                        type="button"
                        onClick={() => void minimizeWindow()}
                    />
                </div>

                <div className="brand">
                    <img src={appIconUrl} alt="" />
                </div>

                <nav className="sidebar-nav" aria-label="偏好设置分类">
                    {preferenceSections.map(item => {
                        const Icon = item.icon;
                        return (
                            <NavLink
                                key={item.id}
                                to={item.path}
                                className={({ isActive }) => `sidebar-nav-link${isActive ? ' active' : ''}`}
                            >
                                <span className={`nav-icon ${item.iconClass}`}>
                                    <Icon size={21} />
                                </span>
                                {item.label}
                            </NavLink>
                        );
                    })}
                </nav>
            </aside>

            <section className="content-pane">
                {!config ? (
                    <div className="loading-state">
                        <Spinner />
                        <span>加载偏好设置...</span>
                    </div>
                ) : (
                    <Outlet />
                )}
            </section>
        </main>
    );
}
