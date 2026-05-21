import { Button, Checkbox, Input } from '@heroui/react';
import { Play } from 'lucide-react';
import { usePreferenceStore } from '../stores/preferences';

export default function ToolboxView() {
    const busy = usePreferenceStore(state => state.busy);
    const config = usePreferenceStore(state => state.config);
    const status = usePreferenceStore(state => state.status);
    const runDebug = usePreferenceStore(state => state.runDebug);
    const setCustomScriptCommand = usePreferenceStore(state => state.setCustomScriptCommand);
    const setToolboxProvider = usePreferenceStore(state => state.setToolboxProvider);
    const toggleCustomScript = usePreferenceStore(state => state.toggleCustomScript);

    if (!config) {
        return null;
    }

    return (
        <section className="settings-page toolbox-page">
            <div className="page-header">
                <h1>工具箱</h1>
                <Button isDisabled={busy} onPress={() => void runDebug()}>
                    <Play size={16} />
                    运行调试动作
                </Button>
            </div>
            <div className="preference-group">
                <label className="field-row">
                    <span>翻译 Provider</span>
                    <Input
                        defaultValue={config.toolbox.translation_provider}
                        placeholder="none"
                        onBlur={event => void setToolboxProvider(event.currentTarget.value)}
                    />
                </label>
                {config.custom_scripts.map(script => (
                    <label key={script.id} className="menu-row">
                        <span>
                            <strong>{script.title}</strong>
                            <small>{script.command || '未配置命令'}</small>
                        </span>
                        <span className="row-actions">
                            <Input
                                defaultValue={script.command}
                                placeholder="脚本路径或命令"
                                onBlur={event => void setCustomScriptCommand(script.id, event.currentTarget.value)}
                            />
                            <Checkbox
                                isSelected={script.enabled}
                                onChange={enabled => void toggleCustomScript(script.id, enabled)}
                            />
                        </span>
                    </label>
                ))}
                {status ? <p className="status">{status}</p> : null}
            </div>
        </section>
    );
}
