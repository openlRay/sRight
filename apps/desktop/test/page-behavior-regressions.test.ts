import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const preferenceShell = readFileSync(new URL('../src/components/PreferenceShell.tsx', import.meta.url), 'utf8');
const favoritesView = readFileSync(new URL('../src/views/FavoritesView.tsx', import.meta.url), 'utf8');
const newFileTemplatesView = readFileSync(new URL('../src/views/NewFileTemplatesView.tsx', import.meta.url), 'utf8');
const sendToView = readFileSync(new URL('../src/views/SendToView.tsx', import.meta.url), 'utf8');
const store = readFileSync(new URL('../src/stores/preferences.ts', import.meta.url), 'utf8');
const api = readFileSync(new URL('../src/lib/api.ts', import.meta.url), 'utf8');
const globalsStyles = readFileSync(new URL('../src/styles/globals.less', import.meta.url), 'utf8');
const layoutStyles = readFileSync(new URL('../src/styles/layout.less', import.meta.url), 'utf8');

assert.match(preferenceShell, /startDragging/);
assert.match(preferenceShell, /dragStartDelay/);
assert.match(preferenceShell, /new URL\('\.\.\/\.\.\/src-tauri\/icons\/icon\.png', import\.meta\.url\)\.href/);
assert.doesNotMatch(preferenceShell, /src="\/src-tauri\/icons\/icon\.png"/);
assert.match(globalsStyles, /#app\s*\{[\s\S]*padding: @space-xl/);
assert.match(layoutStyles, /\.preference-window\s*\{[\s\S]*box-shadow: @shadow-layered/);
assert.match(layoutStyles, /\.preference-window\s*\{[\s\S]*border: 1px solid rgba\(8, 8, 8, 0\.12\)/);

assert.match(favoritesView, /resetFavoriteDirs/);
assert.match(favoritesView, /RefreshCcw/);
assert.match(favoritesView, /setMenuIconVisibility\('favorite_dirs'/);
assert.doesNotMatch(favoritesView, /setTopLevelFlag\('show_icons'/);
assert.match(newFileTemplatesView, /setMenuIconVisibility\('new_file'/);
assert.match(newFileTemplatesView, /resetTemplates/);
assert.match(newFileTemplatesView, /RefreshCcw/);
assert.doesNotMatch(newFileTemplatesView, /setTopLevelFlag\('show_icons'/);
assert.doesNotMatch(favoritesView, /onPress=\{\(\) => void refresh\(\)\}/);
assert.match(sendToView, /resetSendDirs/);
assert.match(sendToView, /RefreshCcw/);
assert.match(sendToView, /setMenuIconVisibility\('send_to'/);
assert.doesNotMatch(sendToView, /setTopLevelFlag\('show_icons'/);
assert.match(sendToView, /config\.send_dirs/);
assert.doesNotMatch(sendToView, /config\.favorite_dirs\.map/);
assert.doesNotMatch(sendToView, /addFavoriteDirFromPicker/);
assert.doesNotMatch(sendToView, /removeFavoriteDir/);
assert.doesNotMatch(sendToView, /renameFavoriteDir/);
assert.doesNotMatch(sendToView, /reorderFavoriteDirs/);
assert.doesNotMatch(sendToView, /onPress=\{\(\) => void refresh\(\)\}/);

assert.match(store, /resetFavoriteDirs/);
assert.match(store, /resetSendDirs/);
assert.match(store, /resetTemplates/);
assert.match(store, /resetToolbox/);
assert.match(store, /addSendDirFromPicker/);
assert.match(store, /setMenuIconVisibility/);
assert.match(api, /send_dirs/);
assert.match(api, /menu_icons/);
