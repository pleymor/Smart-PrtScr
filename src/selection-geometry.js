// Multi-monitor geometry for the selection window (selection.html).
// The selection window covers the whole virtual desktop; the captured image is
// stretched over its viewport. Monitors come from the backend in image pixels.
// Loaded as a classic <script> in the webview and with require() by node tests.
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) {
    module.exports = api;
  } else {
    root.SelectionGeometry = api;
  }
})(typeof self !== 'undefined' ? self : this, function () {
  // Convert monitor rectangles from image pixels to viewport (CSS) pixels.
  function monitorsToCss(monitors, imageWidth, imageHeight, viewWidth, viewHeight) {
    if (!monitors || !imageWidth || !imageHeight) return [];
    const sx = viewWidth / imageWidth;
    const sy = viewHeight / imageHeight;
    return monitors.map((m) => ({
      x: m.x * sx,
      y: m.y * sy,
      width: m.width * sx,
      height: m.height * sy,
      primary: !!m.primary,
    }));
  }

  function monitorAt(monitors, x, y) {
    return (
      monitors.find((m) => x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height) || null
    );
  }

  function primaryMonitor(monitors) {
    return monitors.find((m) => m.primary) || monitors[0] || null;
  }

  // "Whole screen" selection: the monitor under the point, else the primary
  // monitor, else the whole viewport.
  function fullMonitorSelection(monitors, point, viewport) {
    const m =
      (point && monitorAt(monitors, point.x, point.y)) || primaryMonitor(monitors);
    if (!m) return { x: 0, y: 0, width: viewport.width, height: viewport.height };
    return { x: m.x, y: m.y, width: m.width, height: m.height };
  }

  return { monitorsToCss, monitorAt, primaryMonitor, fullMonitorSelection };
});
