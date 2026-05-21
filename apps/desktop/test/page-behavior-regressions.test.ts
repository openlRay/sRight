import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const preferenceShell = readFileSync(new URL('../src/components/PreferenceShell.tsx', import.meta.url), 'utf8');
const favoritesView = readFileSync(new URL('../src/views/FavoritesView.tsx', import.meta.url), 'utf8');
const sendToView = readFileSync(new URL('../src/views/SendToView.tsx', import.meta.url), 'utf8');
const store = readFileSync(new URL('../src/stores/preferences.ts', import.meta.url), 'utf8');
const api = readFileSync(new URL('../src/lib/api.ts', import.meta.url), 'utf8');

assert.match(preferenceShell, /startDragging/);
assert.match(preferenceShell, /dragStartDelay/);

assert.match(favoritesView, /resetFavoriteDirs/);
assert.doesNotMatch(favoritesView, /onPress=\{\(\) => void refresh\(\)\}/);
assert.match(sendToView, /resetSendDirs/);
assert.match(sendToView, /config\.send_dirs/);
assert.doesNotMatch(sendToView, /config\.favorite_dirs\.map/);
assert.doesNotMatch(sendToView, /addFavoriteDirFromPicker/);
assert.doesNotMatch(sendToView, /removeFavoriteDir/);
assert.doesNotMatch(sendToView, /renameFavoriteDir/);
assert.doesNotMatch(sendToView, /reorderFavoriteDirs/);
assert.doesNotMatch(sendToView, /onPress=\{\(\) => void refresh\(\)\}/);

assert.match(store, /resetFavoriteDirs/);
assert.match(store, /resetSendDirs/);
assert.match(store, /addSendDirFromPicker/);
assert.match(api, /send_dirs/);
