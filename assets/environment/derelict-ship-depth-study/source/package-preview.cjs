const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');

const root = path.resolve(__dirname, '..');
const assets = ['current.png', 'raised-cutaway.png', 'raised-solid.png'];
let html = fs.readFileSync(path.join(__dirname, 'preview-template.html'), 'utf8');

// Embed every displayed view so switching still works after the preview server stops.
for (const name of assets) {
  const marker = `src="${name}"`;
  assert.equal(html.split(marker).length, 2, `Expected one image for ${name}`);
  const data = fs.readFileSync(path.join(root, name));
  html = html.replace(marker, `src="data:image/png;base64,${data.toString('base64')}"`);
}
const images = [...html.matchAll(/<img\b[^>]*\bsrc="([^"]+)"/g)];
assert.equal(images.length, assets.length);
assert(images.every(([, src]) => src.startsWith('data:image/png;base64,')));
fs.writeFileSync(path.join(root, 'index.html'), html);
console.log('Packaged preview with all three PNG views embedded.');
