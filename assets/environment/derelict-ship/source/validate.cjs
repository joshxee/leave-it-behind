const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const {PNG}=require(process.env.PNGJS_PATH||'C:/Users/jbxcl/Projects/sprite-axi/node_modules/pngjs');
const root=path.resolve(__dirname,'..'),read=f=>JSON.parse(fs.readFileSync(path.join(root,f),'utf8'));
const m=read('manifest.json'),atlas=PNG.sync.read(fs.readFileSync(path.join(root,'ship-atlas.png'))),meta=read('ship-atlas.aseprite.json'),nativeFrames=Object.values(meta.frames);
const pal=fs.readFileSync(path.join(root,'ship-16.pal'),'utf8').trim().split(/\r?\n/).slice(3).map(s=>s.split(' ').map(Number));
const allowed=new Set(pal.slice(1).map(c=>c.join(','))),used=new Set(),images=new Map(),issues=[],groups={},hashes=new Set();
const equal=(a,i,b,j)=>a[i+3]===b[j+3]&&(!a[i+3]||[0,1,2].every(c=>a[i+c]===b[j+c]));
if(pal.length!==16||new Set(pal.map(c=>c.join(','))).size!==16)issues.push('Palette must have 16 unique entries');
const engineerPal=path.resolve(root,'../../characters/engineer/engineer-16.pal');
if(fs.existsSync(engineerPal)&&!fs.readFileSync(engineerPal).equals(fs.readFileSync(path.join(root,'ship-16.pal'))))issues.push('Palette differs from engineer');
if(atlas.width!==512||atlas.height!==512)issues.push('Incorrect atlas dimensions');
for(const t of m.tiles){
 const bytes=fs.readFileSync(path.join(root,t.file)),im=PNG.sync.read(bytes);images.set(t.id,im);groups[t.category]=(groups[t.category]||0)+1;
 hashes.add(crypto.createHash('sha256').update(im.data).digest('hex'));
 if(im.width!==64||im.height!==64||bytes[25]!==3)issues.push(`${t.id}: expected indexed 64x64 PNG`);
 let opaque=0,partial=false,off=false,mismatch=false;
 for(let y=0;y<64;y++)for(let x=0;x<64;x++){
  const k=(y*64+x)*4,a=im.data[k+3];if(a!==0&&a!==255)partial=true;
  if(a){opaque++;const c=Array.from(im.data.subarray(k,k+3)).join(',');used.add(c);if(!allowed.has(c))off=true;}
  const q=((y+t.atlas_rect[1])*512+x+t.atlas_rect[0])*4;if(!equal(im.data,k,atlas.data,q))mismatch=true;
 }
 if(!opaque)issues.push(`${t.id}: blank`);if(partial)issues.push(`${t.id}: partial alpha`);if(off)issues.push(`${t.id}: off palette`);if(mismatch)issues.push(`${t.id}: atlas differs`);
 if((t.category==='floor'||t.category==='space')&&opaque!==4096)issues.push(`${t.id}: base tile must be opaque`);
 if(t.category!=='floor'&&t.category!=='space'&&opaque===4096)issues.push(`${t.id}: overlay lacks transparency`);
 const nf=nativeFrames[t.atlas_index];if(!nf||nf.trimmed||nf.rotated||nf.frame.w!==64||nf.frame.h!==64)issues.push(`${t.id}: incorrect native atlas entry`);
 if(t.anchor_pixels[0]!==32||t.anchor_pixels[1]!==32)issues.push(`${t.id}: grid anchor drift`);
 for(const [x,y,w,h]of t.collision)if(x<0||y<0||w<=0||h<=0||x+w>64||y+h>64)issues.push(`${t.id}: invalid collider`);
}
const count=fs.readdirSync(path.join(root,'png')).filter(x=>x.endsWith('.png')).length;if(count!==64||m.tiles.length!==64||hashes.size!==64)issues.push('Expected 64 distinct individual tiles');
const expected={floor:9,floor_edge:8,space:2,wall:16,breach:2,patched_wall:2,door:10,cockpit:6,diagnostic:3,prop:6};
for(const [category,n]of Object.entries(expected))if(groups[category]!==n)issues.push(`${category}: expected ${n} tiles`);
let seamPairs=0;
const structural=m.tiles.filter(t=>t.connection_mask!==undefined),wall=m.tiles.filter(t=>t.category==='wall');
for(const a of structural)for(const b of wall){
 const pa=images.get(a.id).data,pb=images.get(b.id).data;
 if((a.connection_mask&2)&&(b.connection_mask&8)){seamPairs++;if(Array.from({length:64},(_,y)=>equal(pa,(y*64+63)*4,pb,y*64*4)).some(v=>!v))issues.push(`EW seam: ${a.id}/${b.id}`);}
 if((a.connection_mask&4)&&(b.connection_mask&1)){seamPairs++;if(Array.from({length:64},(_,x)=>equal(pa,(63*64+x)*4,pb,x*4)).some(v=>!v))issues.push(`NS seam: ${a.id}/${b.id}`);}
}
const floorBase=images.get('floor_panel_a').data;
for(const t of m.tiles.filter(t=>t.category==='floor')){
 const p=images.get(t.id).data;let mismatch=false;
 for(let n=0;n<64;n++)for(const [x,y]of [[n,0],[n,63],[0,n],[63,n]])if(!equal(p,(y*64+x)*4,floorBase,(y*64+x)*4))mismatch=true;
 if(mismatch)issues.push(`${t.id}: floor border mismatch`);
}
const cockpitMaster=PNG.sync.read(fs.readFileSync(path.join(root,'cockpit-module.png')));
for(const t of m.tiles.filter(t=>t.category==='cockpit')){const p=images.get(t.id).data,[cx,cy]=t.module_cell;for(let y=0;y<64;y++)for(let x=0;x<64;x++)if(!equal(p,(y*64+x)*4,cockpitMaster.data,((cy*64+y)*192+cx*64+x)*4)){issues.push(`${t.id}: cockpit slice differs`);y=64;break;}}
const wEw=images.get('wall_ew').data;
for(const [id,x,bx]of [['cockpit_0_0',0,63],['cockpit_2_0',63,0]]){const p=images.get(id).data;for(let y=0;y<64;y++)if(!equal(p,(y*64+x)*4,wEw,(y*64+bx)*4)){issues.push(`${id}: hull connection differs`);break;}}
for(const a of m.animations){const tiles=a.files.map(f=>m.tiles.find(t=>t.file===f)),tag=meta.meta.frameTags.find(t=>t.name===a.id);if(!tag||tag.from!==tiles[0].atlas_index||tag.to!==tiles.at(-1).atlas_index)issues.push(`${a.id}: tag range differs`);for(const t of tiles)if(nativeFrames[t.atlas_index].duration!==a.duration_ms)issues.push(`${a.id}: wrong timing`);}
const scene=read('example-room.json'),tmj=read('example-room.tmj'),tsj=read('ship.tsj');
for(const l of scene.layers){if(l.data.length!==scene.width*scene.height||l.data.some(g=>g<0||g>64||!Number.isInteger(g)))issues.push(`${l.name}: invalid map layer`);if(JSON.stringify(l.data)!==JSON.stringify(tmj.layers.find(t=>t.name===l.name).data))issues.push(`${l.name}: Tiled map differs`);}
if(tsj.tiles.length!==64||tsj.tilecount!==64)issues.push('Tiled tileset incomplete');
const report={passed:issues.length===0,individual_tiles:count,distinct_tiles:hashes.size,tile_dimensions:[64,64],atlas_dimensions:[512,512],palette_entries:pal.length,opaque_colours_used:used.size,binary_alpha:true,wall_connection_patterns:wall.length,matching_wall_seam_pairs:seamPairs,cockpit_master_matches_six_tiles:!issues.some(s=>s.includes('cockpit')),groups,issues};
fs.writeFileSync(path.join(root,'validation.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));if(issues.length)process.exitCode=1;
