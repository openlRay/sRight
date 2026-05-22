import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const app = readFileSync(new URL('../src/App.tsx', import.meta.url), 'utf8');
const api = readFileSync(new URL('../src/lib/api.ts', import.meta.url), 'utf8');
const tauriLib = readFileSync(new URL('../src-tauri/src/lib.rs', import.meta.url), 'utf8');
const finderSync = readFileSync(new URL('../../../native/macos/FinderSyncExtension/FinderSync.swift', import.meta.url), 'utf8');
const capability = JSON.parse(
    readFileSync(new URL('../src-tauri/capabilities/main.json', import.meta.url), 'utf8')
);
const tauriConfig = JSON.parse(
    readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8')
);
const globalsStyles = readFileSync(new URL('../src/styles/globals.less', import.meta.url), 'utf8');

assert.doesNotMatch(api, /export interface ActionResult/);
assert.doesNotMatch(app, /Modal/);
assert.doesNotMatch(app, /Button/);
assert.doesNotMatch(app, /getCurrentWindow/);
assert.doesNotMatch(app, /__SRIGHT_ACTION_RESULT__/);
assert.doesNotMatch(app, /isActionResultWindow/);
assert.doesNotMatch(app, /ActionResultDialog/);
assert.doesNotMatch(app, /action-result-dialog/);
assert.doesNotMatch(app, /navigator\.clipboard\.writeText/);
assert.doesNotMatch(app, /listen<ActionResult>\('sright:\/\/action-result'/);
assert.doesNotMatch(globalsStyles, /action-result/);

assert.equal(tauriConfig.app.windows.find((window: { label: string }) => window.label === 'main')?.visible, false);

assert.deepEqual(capability.windows, ['main']);
assert.doesNotMatch(tauriLib, /WebviewWindowBuilder::new/);
assert.doesNotMatch(tauriLib, /WebviewUrl::App\("index\.html\?action-result"/);
assert.doesNotMatch(tauriLib, /open_action_result_window/);
assert.match(tauriLib, /present_system_action_result/);
assert.match(tauriLib, /display alert/);
assert.match(tauriLib, /set the clipboard to/);
assert.doesNotMatch(tauriLib, /^repeat$/m);
assert.match(tauriLib, /SRIGHT_ACTION_RESULT_FILE/);
assert.match(tauriLib, /ACTION_RESULT_REOPEN_SUPPRESSED/);
assert.match(tauriLib, /should_show_preferences_on_startup/);
assert.match(tauriLib, /should_show_preferences_on_startup_with_args/);
assert.match(tauriLib, /BACKGROUND_ACTION_ARG/);
assert.match(tauriLib, /has_pending_finder_actions/);
assert.match(tauriLib, /std::thread::sleep/);
assert.match(tauriLib, /Ordering::SeqCst/);
assert.match(tauriLib, /should_present_action_result/);
assert.match(tauriLib, /confirm_dangerous_action/);
assert.match(tauriLib, /SRIGHT_CONFIRM_DANGEROUS_RESPONSE/);
assert.doesNotMatch(tauriLib, /show_preferences\(app\);\n\s*app\.emit\("sright:\/\/action-result"/);
assert.match(finderSync, /"--args", "--sright-background-action"/);
