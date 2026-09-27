// Read-only delivery audit: node --preserve-symlinks source/validate.cjs
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
// Reuse sprite-axi's installed PNG decoder; no image editing occurs here.
const { PNG } = require(process.env.PNGJS_PATH || 'C:/Users/jbxcl/Projects/sprite-axi/node_modules/pngjs');
const root = path.resolve(__dirname, '..');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json')));
const palette = fs.readFileSync(path.join(root, 'engineer-16.pal'), 'utf8').trim().split(/\r?\n/).slice(3).map(s => s.split(' ').map(Number));
const valid = new Set(palette.slice(1).map(c => c.join(',')));
const issues = [], images = new Map(), colors = new Set(), groups = {};
const expected = { idle: 16, walk: 32, wrench_hold: 16, tape_hold: 16, wrench_use: 24, tape_use: 24, wrench_walk: 32, tape_walk: 32 };
const atlas=PNG.sync.read(fs.readFileSync(path.join(root,'engineer-atlas.png')));
const atlasMeta=JSON.parse(fs.readFileSync(path.join(root,'engineer-atlas.aseprite.json'),'utf8'));
const atlasFrames=Object.values(atlasMeta.frames);
if(atlas.width!==1024||atlas.height!==Math.ceil(manifest.frames.length/16)*64)issues.push('Incorrect atlas size');
if(palette.length!==16||new Set(palette.map(c=>c.join(','))).size!==16)issues.push('Palette must contain 16 unique entries');
if(palette.slice(1).some(([r,g,b])=>r>g||g>b))issues.push('Palette contains a non-cold colour');
for (const frame of manifest.frames) {
  const file = path.join(root, frame.file), bytes=fs.readFileSync(file),png = PNG.sync.read(bytes);
  if(bytes[25]!==3)issues.push(`${frame.file}: not an indexed PNG`);
  if (png.width !== 64 || png.height !== 64) issues.push(`${frame.file}: incorrect size`);
  let opaque = 0; const bounds = [64, 64, -1, -1];
  for (let y=0;y<64;y++) for (let x=0;x<64;x++) {
    const i=(y*64+x)*4, a=png.data[i+3];
    if (a!==0 && a!==255) issues.push(`${frame.file}: partial alpha`);
    if (!a) continue;
    opaque++; const c=Array.from(png.data.subarray(i,i+3)).join(',');colors.add(c);
    if (!valid.has(c)) issues.push(`${frame.file}: off-palette colour ${c}`);
    bounds[0]=Math.min(bounds[0],x);bounds[1]=Math.min(bounds[1],y);bounds[2]=Math.max(bounds[2],x);bounds[3]=Math.max(bounds[3],y);
  }
  if (!opaque || opaque===4096) issues.push(`${frame.file}: empty or lacks transparency`);
  const af=atlasFrames[frame.atlas_index];
  if(!af||af.trimmed||af.rotated||af.frame.w!==64||af.frame.h!==64||af.duration!==frame.duration_ms)issues.push(`${frame.file}: incorrect atlas metadata`);
  let mismatch=0;
  for(let y=0;y<64;y++)for(let x=0;x<64;x++) {
    const k=(y*64+x)*4,q=((y+af.frame.y)*atlas.width+x+af.frame.x)*4;
    if(png.data[k+3]!==atlas.data[q+3]||(png.data[k+3]&&[0,1,2].some(c=>png.data[k+c]!==atlas.data[q+c])))mismatch++;
  }
  if(mismatch)issues.push(`${frame.file}: ${mismatch} atlas pixels differ`);
  images.set(frame.file,{png,hash:crypto.createHash('sha256').update(png.data).digest('hex'),bounds});
  groups[frame.action]=(groups[frame.action]||0)+1;
  if (frame.contact_pixels) {
    const [cx,cy]=frame.contact_pixels; let near=false;
    for(let y=Math.max(0,cy-3);y<=Math.min(63,cy+3);y++) for(let x=Math.max(0,cx-3);x<=Math.min(63,cx+3);x++) if(png.data[(y*64+x)*4+3]) near=true;
    if(!near)issues.push(`${frame.file}: contact has no nearby tool pixels`);
    if(Math.min(bounds[0],bounds[1],63-bounds[2],63-bounds[3])>3)issues.push(`${frame.file}: tool does not reach the canvas edge`);
  }
}
for(const [name,n] of Object.entries(expected))if(groups[name]!==n)issues.push(`${name}: expected ${n}, got ${groups[name]}`);
const animations=[];
for(const a of manifest.animations.filter(a=>a.files.length>1)) {
  const distinct=new Set(a.files.map(f=>images.get(f).hash)).size;
  if(distinct!==a.files.length)issues.push(`${a.name}: ${distinct}/${a.files.length} distinct frames`);
  const changes=a.files.slice(1).map((file,i)=>{
    const p=images.get(a.files[i]).png.data,q=images.get(file).png.data;let count=0;
    for(let k=0;k<p.length;k+=4)if(p[k]!==q[k]||p[k+1]!==q[k+1]||p[k+2]!==q[k+2]||p[k+3]!==q[k+3])count++;
    return count;
  });
  animations.push({name:a.name,distinct_frames:distinct,changed_pixels_between_frames:changes});
  if(a.name.includes('_use_')) {
    const contacts=manifest.frames.filter(f=>a.files.includes(f.file)).map(f=>JSON.stringify(f.contact_pixels));
    if(new Set(contacts).size!==1)issues.push(`${a.name}: contact moves`);
    const first=manifest.frames.find(f=>f.file===a.files[0]);
    const held=manifest.frames.find(f=>f.action===first.action.replace('_use','_hold')&&f.direction===first.direction);
    if(JSON.stringify(held.contact_pixels)!==contacts[0])issues.push(`${a.name}: contact differs from held pose`);
  }
}
const idleHashes=new Set(manifest.frames.filter(f=>f.action==='idle').map(f=>images.get(f.file).hash));
if(idleHashes.size!==16)issues.push('Idle directions are duplicated');
const actual=fs.readdirSync(path.join(root,'png')).filter(s=>s.endsWith('.png')).length;
if(actual!==manifest.frames.length)issues.push(`Expected ${manifest.frames.length} individual PNGs, found ${actual}`);
for(const a of manifest.animations) {
  const t=atlasMeta.meta.frameTags.find(t=>t.name===a.name);
  if(!t||t.from!==a.from_frame-1||t.to!==a.to_frame-1)issues.push(`${a.name}: incorrect Aseprite tag range`);
}
const pivot=atlasMeta.meta.slices.find(s=>s.name==='engineer_root')?.keys[0].pivot;
if(!pivot||pivot.x!==32||pivot.y!==38||JSON.stringify(manifest.pivot_pixels)!=='[32,38]')issues.push('Incorrect root pivot');
const report={passed:issues.length===0,individual_pngs:actual,dimensions:[64,64],palette_entries:palette.length,opaque_colours_used:colors.size,alpha_values:[0,255],pivot_pixels:manifest.pivot_pixels,atlas_matches_individual_pngs:!issues.some(s=>s.includes('atlas')),aseprite_animation_tags:atlasMeta.meta.frameTags.length,groups,idle_directions_distinct:idleHashes.size,animations,issues};
fs.writeFileSync(path.join(root,'validation.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
if(issues.length)process.exitCode=1;
