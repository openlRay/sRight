import assert from 'node:assert/strict';
import { templateInfoFromPath } from '../src/lib/file-template-utils.ts';

assert.deepEqual(templateInfoFromPath('/Users/me/Templates/Invoice.xlsx'), {
    title: 'Invoice',
    fileName: 'Invoice.xlsx'
});

assert.deepEqual(templateInfoFromPath('/Users/me/Templates/README'), {
    title: 'README',
    fileName: 'README'
});

assert.deepEqual(templateInfoFromPath('/Users/me/Templates/archive.tar.gz'), {
    title: 'archive.tar',
    fileName: 'archive.tar.gz'
});
