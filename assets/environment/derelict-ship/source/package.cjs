const fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const m=JSON.parse(fs.readFileSync(path.join(root,'manifest.json'),'utf8'));
const byId=new Map(m.tiles.map(t=>[t.id,t]));
const width=13,height=11, layers=['background','floor','structure','prop'].map(name=>({name,data:Array(width*height).fill(0)}));
const put=(layer,x,y,id)=>{const t=byId.get(id);if(!t)throw Error(id);layers.find(l=>l.name===layer).data[y*width+x]=t.atlas_index+1;};
for(let y=0;y<height;y++)for(let x=0;x<width;x++)put('background',x,y,'space_'+((x*3+y)%2));
for(let y=1;y<=9;y++)for(let x=1;x<=11;x++){
 let edge=(y===1?'n':y===9?'s':'')+(x===11?'e':x===1?'w':'');
 const id=edge?'floor_edge_'+edge:(y>5?(x>8?'floor_grate':((x+y)%4===0?'floor_panel_b':'floor_panel_a')):'floor_panel_'+['a','a','b','c'][(x*3+y)%4]);
 put('floor',x,y,id);
}
const walls=new Set(),pos=(x,y)=>`${x},${y}`;
for(let x=1;x<=11;x++){walls.add(pos(x,1));walls.add(pos(x,9));walls.add(pos(x,5));}
for(let y=1;y<=9;y++){walls.add(pos(1,y));walls.add(pos(11,y));}
for(let y=5;y<=9;y++)walls.add(pos(8,y));
const names=['pillar','n','e','ne','s','ns','es','nes','w','nw','ew','new','sw','nsw','esw','nesw'];
for(const p of walls){const [x,y]=p.split(',').map(Number);const mask=(walls.has(pos(x,y-1))?1:0)|(walls.has(pos(x+1,y))?2:0)|(walls.has(pos(x,y+1))?4:0)|(walls.has(pos(x-1,y))?8:0);put('structure',x,y,'wall_'+names[mask]);}
for(let y=0;y<2;y++)for(let x=0;x<3;x++){put('prop',x+5,y+1,`cockpit_${x}_${y}`);if(y===0)layers.find(l=>l.name==='structure').data[(y+1)*width+x+5]=0;}
put('structure',6,5,'door_ew_3');put('structure',8,7,'door_ns_0');put('structure',11,7,'wall_breach_ns');
put('prop',3,3,'diagnostic_0');put('prop',9,3,'prop_control_console');put('prop',2,7,'prop_crate');put('prop',3,8,'prop_crate');put('prop',5,7,'prop_oxygen_rack');put('prop',6,7,'prop_locker');put('prop',10,6,'prop_pipe_stack');
put('floor',6,4,'floor_drain');put('floor',3,4,'floor_conduit_h');put('floor',4,4,'floor_conduit_h');put('floor',9,8,'floor_hatch');
const scene={name:'Bridge / cargo / damaged engineering bay',width,height,tile_size:64,layers,dynamic_cells:{door_ew:[6,5],door_ns:[8,7],diagnostic:[3,3],breach:[11,7]},engineer:{file:'preview/engineer.png',position:[709,480],pivot:[32,38]},notes:'Demonstration layout, not a game level. Structure and prop collision rectangles are in manifest.json.'};
fs.writeFileSync(path.join(root,'example-room.json'),JSON.stringify(scene,null,2)+'\n');
const tiledTiles=m.tiles.map(t=>({id:t.atlas_index,type:t.category,properties:[{name:'asset_id',type:'string',value:t.id},{name:'layer',type:'string',value:t.layer},...(t.connection_mask!==undefined?[{name:'connection_mask',type:'int',value:t.connection_mask}]:[]),...(t.passable!==undefined?[{name:'passable',type:'bool',value:t.passable}]:[]),...(t.vacuum_hazard!==undefined?[{name:'vacuum_hazard',type:'bool',value:t.vacuum_hazard}]:[])],...(t.collision.length?{objectgroup:{draworder:'index',objects:t.collision.map(([x,y,width,height],i)=>({id:i+1,name:'collision',type:'solid',x,y,width,height,rotation:0,visible:true}))}}:{})}));
for(const a of m.animations){const first=m.tiles.find(t=>t.file===a.files[0]);tiledTiles[first.atlas_index].animation=a.files.map(file=>({tileid:m.tiles.find(t=>t.file===file).atlas_index,duration:a.duration_ms}));}
const tileset={type:'tileset',version:'1.10',tiledversion:'1.11.0',name:'Derelict ship',tilewidth:64,tileheight:64,tilecount:64,columns:8,image:'ship-atlas.png',imagewidth:512,imageheight:512,margin:0,spacing:0,tiles:tiledTiles};
// Tiled loops native tile animations: door transitions are listed only in the custom manifest.
for(const t of tileset.tiles)if(t.type==='door')delete t.animation;
fs.writeFileSync(path.join(root,'ship.tsj'),JSON.stringify(tileset,null,2)+'\n');
const tmj={type:'map',version:'1.10',tiledversion:'1.11.0',orientation:'orthogonal',renderorder:'right-down',infinite:false,width,height,tilewidth:64,tileheight:64,nextlayerid:5,nextobjectid:1,tilesets:[{firstgid:1,source:'ship.tsj'}],layers:layers.map((l,i)=>({id:i+1,name:l.name,type:'tilelayer',width,height,x:0,y:0,opacity:1,visible:true,data:l.data}))};
fs.writeFileSync(path.join(root,'example-room.tmj'),JSON.stringify(tmj,null,2)+'\n');
const palette=fs.readFileSync(path.join(root,'ship-16.pal'),'utf8').trim().split(/\r?\n/).slice(3).map(s=>s.split(' ').map(n=>Number(n).toString(16).padStart(2,'0')).join(''));
fs.writeFileSync(path.join(root,'preview','preview-data.js'),'window.SHIP_DATA = '+JSON.stringify({manifest:m,scene,palette})+';\n');
fs.copyFileSync(path.resolve(root,'../../characters/engineer/png/engineer_wrench_hold_e_00.png'),path.join(root,'preview','engineer.png'));
console.log('Packaged room layout, Tiled tileset/map and offline preview metadata.');
