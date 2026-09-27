const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { PNG } = require(process.env.PNGJS_PATH || 'C:/Users/jbxcl/Projects/sprite-axi/node_modules/pngjs');
const root = path.resolve(__dirname, '..');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
const colors = fs.readFileSync(path.resolve(root, manifest.palette), 'utf8').trim().split(/\r?\n/).slice(4).map(s => s.trim().split(/\s+/).join(','));
let pixels = 0, frames = 0;
for (const asset of manifest.assets) {
  const file = fs.readFileSync(path.join(root, asset.file));
  const sheet = PNG.sync.read(file);
  const [w, h] = asset.frame_size;
  assert.equal(file[25], 3, `${asset.id}: indexed PNG`);
  const cols = asset.columns || asset.frames;
  assert.equal(sheet.width, w * cols);
  assert.equal(sheet.height, h * Math.ceil(asset.frames / cols));
  assert(fs.existsSync(path.join(root, `${asset.id}.aseprite`)), 'editable source');
  for (let f = 0; f < asset.frames; f++) {
    const frame = PNG.sync.read(fs.readFileSync(path.join(root, 'png', `${asset.id}_${f}.png`)));
    assert.equal(frame.width, w); assert.equal(frame.height, h);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4, j = ((y + Math.floor(f / cols) * h) * sheet.width + (f % cols) * w + x) * 4;
      const a = frame.data[i + 3];
      assert(a === 0 || a === 255, `${asset.id}: binary alpha`);
      if (a) assert(colors.includes([...frame.data.subarray(i, i + 3)].join(',')), `${asset.id}: shared palette`);
      assert(frame.data.subarray(i, i + 4).equals(sheet.data.subarray(j, j + 4)), `${asset.id}: sheet frame ${f}`);
      pixels++;
    }
    frames++;
  }
}
const report = { status: 'PASS', assets: manifest.assets.length, frames, pixels, checks: ['indexed PNG', 'dimensions', 'shared engineer palette', 'binary alpha', 'sheet/frame equality', 'editable sources'] };
fs.writeFileSync(path.join(root, 'validation.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
