const test = require('node:test');
const assert = require('node:assert/strict');
const G = require('../src/selection-geometry.js');

// Composed desktop 4480x1640 px: secondary 1920x1080 on the left (raised by
// 200 px, so negative virtual coordinates), primary 2560x1440 on the right.
const monitors = [
  { x: 1920, y: 200, width: 2560, height: 1440, primary: true },
  { x: 0, y: 0, width: 1920, height: 1080, primary: false },
];

test('monitors are scaled from image pixels to the window viewport (150% scaling)', () => {
  const round = (m) => ({ ...m, x: Math.round(m.x), y: Math.round(m.y), width: Math.round(m.width), height: Math.round(m.height) });
  const css = G.monitorsToCss(monitors, 4480, 1640, 4480 / 1.5, 1640 / 1.5).map(round);
  assert.deepEqual(css[1], { x: 0, y: 0, width: 1280, height: 720, primary: false });
  assert.deepEqual(css[0], { x: 1280, y: 133, width: 1707, height: 960, primary: true });
});

test('a click selects the whole monitor under the cursor', () => {
  const css = G.monitorsToCss(monitors, 4480, 1640, 4480, 1640);
  const view = { width: 4480, height: 1640 };
  assert.deepEqual(G.fullMonitorSelection(css, { x: 100, y: 100 }, view), { x: 0, y: 0, width: 1920, height: 1080 });
  assert.deepEqual(G.fullMonitorSelection(css, { x: 3000, y: 900 }, view), { x: 1920, y: 200, width: 2560, height: 1440 });
});

test('full-screen selection falls back to the primary monitor, then the viewport', () => {
  const css = G.monitorsToCss(monitors, 4480, 1640, 4480, 1640);
  const view = { width: 4480, height: 1640 };
  // Point in the gap below the secondary monitor.
  assert.deepEqual(G.fullMonitorSelection(css, { x: 10, y: 1500 }, view), { x: 1920, y: 200, width: 2560, height: 1440 });
  assert.deepEqual(G.fullMonitorSelection([], null, view), { x: 0, y: 0, width: 4480, height: 1640 });
});
