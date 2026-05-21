import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const finderSync = readFileSync(new URL('../../../native/macos/FinderSyncExtension/FinderSync.swift', import.meta.url), 'utf8');

assert.doesNotMatch(finderSync, /menuTitle\(for:/);
assert.doesNotMatch(finderSync, /excludedToolboxActionIDs|hiddenToolboxActionIds/);
