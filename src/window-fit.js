// Shared by the options windows (index.html, filename-dialog.html).
// Loaded as a classic <script> in the webviews and with require() by node tests.
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) {
    module.exports = api;
  } else {
    root.WindowFit = api;
  }
})(typeof self !== 'undefined' ? self : this, function () {
  // Space kept free around the window (taskbar overlap, borders).
  const SCREEN_MARGIN = 40;
  const MIN_HEIGHT = 150;

  // Height of the window for a given content height: follow the content, but
  // never exceed the available screen height. When capped, the page scrolls
  // (html { overflow-y: auto }).
  function fitWindowHeight(contentHeight, availableHeight) {
    const content = Math.ceil(contentHeight);
    if (!availableHeight || availableHeight <= 0) {
      return content;
    }
    const max = Math.max(MIN_HEIGHT, Math.floor(availableHeight - SCREEN_MARGIN));
    return Math.min(content, max);
  }

  return { fitWindowHeight, SCREEN_MARGIN, MIN_HEIGHT };
});
