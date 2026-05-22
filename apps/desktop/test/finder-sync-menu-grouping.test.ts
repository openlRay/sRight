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
assert.match(finderSync, /item\.identifier = NSUserInterfaceItemIdentifier\(encodedActionID\(actionID\)\)/);
assert.match(finderSync, /item\.tag = tag/);
assert.match(finderSync, /menuActionIDsByTag\[tag\] = actionID/);
assert.match(finderSync, /item\.toolTip = encodedActionID\(actionID\)/);
assert.match(finderSync, /decodedActionID\(from: item\.identifier\?\.rawValue\)/);
assert.match(finderSync, /isKnownActionID\(actionID, config: config\)/);
assert.doesNotMatch(finderSync, /return item\.identifier\?\.rawValue/);
assert.doesNotMatch(finderSync, /item\.identifier = NSUserInterfaceItemIdentifier\(actionID\)/);
assert.match(finderSync, /menuActionIDsByTag\[item\.tag\]/);
assert.match(finderSync, /uniqueActionID\(forTitle:/);
assert.match(finderSync, /SelectionContext/);
assert.match(finderSync, /selectedContext\(for:/);
assert.match(finderSync, /shouldRenderMenuItem/);
assert.match(finderSync, /isImageFile/);
assert.match(finderSync, /case \.noneSelected/);
assert.match(finderSync, /case \.onlyFiles/);
assert.match(finderSync, /case \.onlyFolders/);
assert.match(finderSync, /case \.mixed/);
assert.match(finderSync, /if actionID\.hasPrefix\("new_file\."\) \{\s*return true\s*\}/);
assert.doesNotMatch(finderSync, /actionID == "folder\.dissolve" \|\| actionID\.hasPrefix\("new_file\."\)/);
assert.doesNotMatch(finderSync, /addNewFileSubmenu|addSendToSubmenu|addFavoriteDirsSubmenu|addToolboxSubmenu/);
assert.doesNotMatch(finderSync, /icon\.set_custom|icon\.remove_custom|tool\.copy_summary|script\.run\.default|logs\.search|logs\.export/);
