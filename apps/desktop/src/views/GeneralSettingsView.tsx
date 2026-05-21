import { Button, Card } from '@heroui/react';
import { ExternalLink, KeyRound, SlidersHorizontal } from 'lucide-react';
import { type ReactNode } from 'react';
import { VisibleSwitch } from '../components/VisibleControls';
import { usePreferenceStore } from '../stores/preferences';

export default function GeneralSettingsView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const openExtensionSettings = usePreferenceStore(state => state.openExtensionSettings);
    const openPermissionSettings = usePreferenceStore(state => state.openPermissionSettings);
    const setDangerousConfirmationEnabled = usePreferenceStore(state => state.setDangerousConfirmationEnabled);
    const setTopLevelFlag = usePreferenceStore(state => state.setTopLevelFlag);

    if (!config) {
        return null;
    }

    return (
        <section className="settings-page general-page">
            <GeneralSection icon={<SlidersHorizontal size={16} />} iconClassName="basic" title="基础设置">
                <Card className="general-card settings-card">
                    <Card.Content>
                        <SwitchRow
                            label="显示顶部菜单栏图标"
                            description="控制 macOS 菜单栏中的 sRight 图标。"
                            isSelected={config.show_menu_bar_icon}
                            onValueChange={enabled => void setTopLevelFlag('show_menu_bar_icon', enabled)}
                        />
                        <SwitchRow
                            label="启用 Finder 右键菜单"
                            description="总开关，关闭后 Finder 右键菜单不再显示 sRight 动作。"
                            isSelected={config.enabled}
                            onValueChange={enabled => void setTopLevelFlag('enabled', enabled)}
                        />
                        <SwitchRow
                            label="显示右键菜单图标"
                            description="在 Finder 右键菜单项前显示文件、目录或功能图标。"
                            isSelected={config.show_icons}
                            onValueChange={enabled => void setTopLevelFlag('show_icons', enabled)}
                        />
                        <SwitchRow
                            label="危险动作二次确认"
                            description="彻底删除默认需要确认。"
                            isSelected={config.dangerous_confirmation.enabled}
                            onValueChange={enabled => void setDangerousConfirmationEnabled(enabled)}
                        />
                    </Card.Content>
                </Card>
            </GeneralSection>

            <GeneralSection icon={<KeyRound size={16} />} iconClassName="permission" title="权限">
                <Card className="general-card settings-card">
                    <Card.Content>
                        <SettingRow
                            title="Finder 右键扩展"
                            description="在系统设置中启用 Finder Extension 后，Finder 右键菜单会显示 sRight。"
                            action={
                                <Button variant="secondary" isDisabled={busy} onPress={() => void openExtensionSettings()}>
                                    <ExternalLink size={16} />
                                    扩展启用引导
                                </Button>
                            }
                        />
                        <SettingRow
                            title="磁盘访问权限"
                            description="部分功能无法使用时，您可以授予完全磁盘访问权限来解决。"
                            action={
                                <Button variant="secondary" isDisabled={busy} onPress={() => void openPermissionSettings()}>
                                    <ExternalLink size={16} />
                                    权限设置引导
                                </Button>
                            }
                        />
                    </Card.Content>
                </Card>
            </GeneralSection>
        </section>
    );
}

function SettingRow({ action, description, title }: { action: ReactNode; description: string; title: string }) {
    return (
        <div className="setting-row">
            <span>
                <strong>{title}</strong>
                <small>{description}</small>
            </span>
            <span className="setting-row-action">{action}</span>
        </div>
    );
}

function SwitchRow({
    description,
    isSelected,
    label,
    onValueChange
}: {
    description: string;
    isSelected: boolean;
    label: string;
    onValueChange: (enabled: boolean) => void;
}) {
    return (
        <SettingRow
            title={label}
            description={description}
            action={<VisibleSwitch isSelected={isSelected} onChange={onValueChange} />}
        />
    );
}

function GeneralSection({
    children,
    icon,
    iconClassName,
    title
}: {
    children: ReactNode;
    icon: ReactNode;
    iconClassName: string;
    title: string;
}) {
    return (
        <section className="general-card-section">
            <div className="general-section-title">
                <span className={`section-icon ${iconClassName}`}>{icon}</span>
                <h2>{title}</h2>
            </div>
            {children}
        </section>
    );
}
