import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const toolboxView = readFileSync(new URL('../src/views/ToolboxView.tsx', import.meta.url), 'utf8');
const api = readFileSync(new URL('../src/lib/api.ts', import.meta.url), 'utf8');
const preferenceStore = readFileSync(new URL('../src/stores/preferences.ts', import.meta.url), 'utf8');
const toolboxStyles = readFileSync(new URL('../src/styles/pages/toolbox.less', import.meta.url), 'utf8');

assert.equal(toolboxView.includes('运行调试动作'), false);
assert.equal(toolboxView.includes('扩展'), false);
assert.equal(toolboxView.includes('翻译 Provider'), false);
assert.equal(toolboxView.includes('setToolboxProvider'), false);
assert.equal(api.includes('runDebugAction'), false);
assert.equal(preferenceStore.includes('runDebugAction'), false);
assert.equal(preferenceStore.includes('runDebug'), false);
assert.equal(preferenceStore.includes('setToolboxProvider'), false);
assert.equal(toolboxStyles.includes('toolbox-provider-section'), false);
