// Static checks: the options windows must scroll vertically when their content
// is taller than the window (issue: no scroll bar, content cut off).
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const pages = ['index.html', 'filename-dialog.html'];

function styleOf(html) {
  return html.match(/<style>([\s\S]*?)<\/style>/)[1];
}

function rules(css, selectorTest) {
  const out = [];
  const re = /([^{}]+)\{([^{}]*)\}/g;
  let m;
  while ((m = re.exec(css))) {
    const selectors = m[1].split(',').map((s) => s.trim());
    if (selectors.some(selectorTest)) out.push(m[2]);
  }
  return out.join(';');
}

for (const page of pages) {
  const html = fs.readFileSync(path.join(__dirname, '..', 'src', page), 'utf8');
  const css = styleOf(html);

  test(`${page}: root element scrolls vertically`, () => {
    const htmlRule = rules(css, (s) => s === 'html');
    assert.match(htmlRule, /overflow-y:\s*auto/, 'html must have overflow-y: auto');
    const bodyRule = rules(css, (s) => s === 'body');
    assert.doesNotMatch(bodyRule, /overflow(-y)?:\s*hidden/, 'body must not clip vertically');
  });

  test(`${page}: window height is capped to the screen`, () => {
    assert.match(html, /<script src="window-fit.js"><\/script>/);
    assert.match(html, /WindowFit\.fitWindowHeight\(/);
  });
}
