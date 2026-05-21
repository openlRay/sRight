import { Button, Card, Switch } from '@heroui/react';
import { ExternalLink, RefreshCw } from 'lucide-react';
import { usePreferenceStore } from '../stores/preferences';

export default function GeneralSettingsView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const settingsStatus = usePreferenceStore(state => state.settingsStatus);
    const openExtensionSettings = usePreferenceStore(state => state.openExtensionSettings);
    const refresh = usePreferenceStore(state => state.refresh);
    const setDangerousConfirmationEnabled = usePreferenceStore(state => state.setDangerousConfirmationEnabled);
    const setTopLevelFlag = usePreferenceStore(state => state.setTopLevelFlag);

    if (!config) {
        return null;
    }

    return (
        <section className="settings-page general-page">
            <div className="page-header">
                <h1>通用设置</h1>
                <Button isDisabled={busy} onPress={() => void refresh()}>
                    <RefreshCw size={16} />
                    刷新
                </Button>
            </div>

            <Card className="preference-group">
                <Card.Content>
                    <SwitchRow
                        label="启用 Finder 右键菜单"
                        isSelected={config.enabled}
                        onValueChange={enabled => void setTopLevelFlag('enabled', enabled)}
                    />
                    <SwitchRow
                        label="显示菜单图标"
                        isSelected={config.show_icons}
                        onValueChange={enabled => void setTopLevelFlag('show_icons', enabled)}
                    />
                    <SwitchRow
                        label="合并菜单分组"
                        isSelected={config.merge_groups}
                        onValueChange={enabled => void setTopLevelFlag('merge_groups', enabled)}
                    />
                    <SwitchRow
                        label="危险动作二次确认"
                        description="移到废纸篓、彻底删除默认需要确认。"
                        isSelected={config.dangerous_confirmation.enabled}
                        onValueChange={enabled => void setDangerousConfirmationEnabled(enabled)}
                    />
                </Card.Content>
            </Card>

            <Card className="preference-group guide">
                <Card.Content>
                    <div>
                        <h2>Finder 右键扩展</h2>
                        <p className="hint">
                            安装后需要在系统设置中手动启用 Finder Extension。启用后，Finder 右键菜单会显示 sRight。
                        </p>
                    </div>
                    <Button isDisabled={busy} onPress={() => void openExtensionSettings()}>
                        <ExternalLink size={16} />
                        打开启用位置
                    </Button>
                    <ol className="steps">
                        <li>在系统设置中进入“扩展”或“登录项与扩展”。</li>
                        <li>找到 Finder Extensions。</li>
                        <li>启用 SRightFinderSync。</li>
                        <li>如果 Finder 右键没有刷新，请重新启动 Finder。</li>
                    </ol>
                    {settingsStatus ? <p className="status">{settingsStatus}</p> : null}
                </Card.Content>
            </Card>
        </section>
    );
}

function SwitchRow({
    description,
    isSelected,
    label,
    onValueChange
}: {
    description?: string;
    isSelected: boolean;
    label: string;
    onValueChange: (enabled: boolean) => void;
}) {
    return (
        <div className="switch-row">
            <span>
                {label}
                {description ? <small>{description}</small> : null}
            </span>
            <Switch isSelected={isSelected} onChange={onValueChange} />
        </div>
    );
}
