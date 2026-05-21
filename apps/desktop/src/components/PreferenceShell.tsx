import { Spinner } from '@heroui/react';
import { useEffect } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { preferenceSections } from '../router/sections';
import { usePreferenceStore } from '../stores/preferences';

export default function PreferenceShell() {
    const config = usePreferenceStore(state => state.config);
    const refresh = usePreferenceStore(state => state.refresh);

    useEffect(() => {
        void refresh();
    }, [refresh]);

    return (
        <main className="preference-window">
            <aside className="sidebar">
                <div className="traffic-lights" aria-hidden="true">
                    <span className="traffic-light close" />
                    <span className="traffic-light minimize" />
                </div>

                <div className="brand">
                    <img src="/src-tauri/icons/icon.png" alt="" />
                    <strong>超级右键：2.4.7</strong>
                    <a href="https://apps.apple.com/" target="_blank" rel="noreferrer">
                        评分
                    </a>
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
