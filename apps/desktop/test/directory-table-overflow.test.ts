import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const favoritesView = readFileSync(new URL('../src/views/FavoritesView.tsx', import.meta.url), 'utf8');
const sendToView = readFileSync(new URL('../src/views/SendToView.tsx', import.meta.url), 'utf8');
const sendToStyles = readFileSync(new URL('../src/styles/pages/send-to.less', import.meta.url), 'utf8');

for (const view of [favoritesView, sendToView]) {
    assert.match(view, /import \{ Button, Input, Table, TableLayout, Tooltip, Virtualizer \} from '@heroui\/react';/);
    assert.match(view, /<Virtualizer\s+layout=\{TableLayout\}\s+layoutOptions=\{\{\s+headingHeight: 42,\s+rowHeight: 42,\s+\}\}/);
    assert.match(view, /<Virtualizer[\s\S]*<Table className="settings-table"[\s\S]*<Table\.ScrollContainer className="settings-table-scroll">/);
    assert.match(view, /<Table\.ScrollContainer className="settings-table-scroll">/);
    assert.match(view, /<Tooltip[\s\S]*<Tooltip\.Trigger[\s\S]*className="path-cell"[\s\S]*<Tooltip\.Content[\s\S]*\{directory\.path\}/);
    assert.match(view, /<Tooltip[\s\S]*<Tooltip\.Trigger[\s\S]*className="directory-name-button"[\s\S]*<Tooltip\.Content[\s\S]*\{directory\.title\}/);
}

assert.match(sendToStyles, /\.path-cell[\s\S]*max-width:/);
assert.match(sendToStyles, /\.directory-name-button[\s\S]*max-width:/);
assert.match(sendToStyles, /\.settings-table-scroll[\s\S]*table[\s\S]*table-layout: fixed/);
assert.match(sendToStyles, /\.settings-table-scroll th[\s\S]*position: sticky[\s\S]*top: 0/);
