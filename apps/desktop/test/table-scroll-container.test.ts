import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

for (const viewName of ['FavoritesView', 'NewFileTemplatesView', 'SendToView', 'ToolboxView']) {
    const source = readFileSync(new URL(`../src/views/${viewName}.tsx`, import.meta.url), 'utf8');

    assert.match(source, /<Table[\s>]/, `${viewName} should render a HeroUI Table`);
    assert.match(source, /<Table\.ScrollContainer className="settings-table-scroll">/, `${viewName} should use Table.ScrollContainer`);
    assert.doesNotMatch(source, /<div className="settings-table-scroll">/, `${viewName} should not wrap Table in a scrolling div`);
}

const formStyles = readFileSync(new URL('../src/styles/forms.less', import.meta.url), 'utf8');

assert.match(formStyles, /\.settings-table[\s\S]*display: flex[\s\S]*flex-direction: column/);
assert.match(formStyles, /\.settings-table-scroll[\s\S]*flex: 1 1 auto[\s\S]*overflow: auto/);
