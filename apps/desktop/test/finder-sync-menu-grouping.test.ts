import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const finderSync = readFileSync(
    new URL('../../../native/macos/FinderSyncExtension/FinderSync.swift', import.meta.url),
    'utf8'
);

assert.equal(finderSync.includes('parent.submenu = submenu'), false);
assert.match(finderSync, /menu\.setSubmenu\(submenu, for: parent\)/);
assert.match(finderSync, /struct MenuTreeItem: Decodable/);
assert.match(finderSync, /renderMenuTree/);
assert.doesNotMatch(finderSync, /addNewFileSubmenu|addSendToSubmenu|addFavoriteDirsSubmenu|addToolboxSubmenu/);
