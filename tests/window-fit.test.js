const test = require('node:test');
const assert = require('node:assert/strict');
const { fitWindowHeight, SCREEN_MARGIN } = require('../src/window-fit.js');

test('window follows its content when it fits on screen', () => {
  assert.equal(fitWindowHeight(520, 1032), 520);
});

test('window never grows taller than the available screen height', () => {
  // e.g. 1366x768 at 125%: ~582 CSS px available, content 760 px tall.
  const h = fitWindowHeight(760, 582);
  assert.ok(h <= 582 - SCREEN_MARGIN, `height ${h} exceeds the screen`);
});

test('a missing screen height falls back to the content height', () => {
  assert.equal(fitWindowHeight(300, 0), 300);
  assert.equal(fitWindowHeight(300, undefined), 300);
});
