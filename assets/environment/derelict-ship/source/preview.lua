local out='assets/environment/derelict-ship/'
local source=app.open(out..'ship.aseprite');local pal=source.palettes[1]
local function read(file) local f=assert(io.open(file));local v=json.decode(f:read('*a'));f:close();return v end
local manifest=read(out..'manifest.json');local scene=read(out..'example-room.json')
local lookup={};for _,t in ipairs(manifest.tiles) do lookup[t.id]=math.floor(t.atlas_index)+1 end
local hero=app.open(out..'preview/engineer.png');local herocel=hero.layers[1]:cel(1)
local function tile(im,index,x,y)
 local cel=source.layers[1]:cel(math.floor(index));im:drawImage(cel.image,Point(x+cel.position.x,y+cel.position.y))
end
local function room(phase,diagnostic)
 local im=Image(832,704,ColorMode.INDEXED)
 for _,layer in ipairs(scene.layers) do for i,gid in ipairs(layer.data) do
  local index=math.floor(gid);local x,y=(i-1)%13,math.floor((i-1)/13)
  if layer.name=='structure' then if x==6 and y==5 then index=lookup['door_ew_'..phase] elseif x==8 and y==7 then index=lookup['door_ns_'..(3-phase)] end end
  if layer.name=='prop' and x==3 and y==3 then index=lookup['diagnostic_'..diagnostic] end
  if index>0 then tile(im,index,x*64,y*64) end
 end end
 im:drawImage(herocel.image,Point(709-32+herocel.position.x,480-38+herocel.position.y))
 return im
end
local preview=Sprite(832,704,ColorMode.INDEXED);preview:setPalette(pal)
local still=room(3,0);still:saveAs{filename=out..'preview/room.png',palette=pal}
local phases={0,1,2,3,3,3,3,3,2,1,0,0}
for f,p in ipairs(phases) do if f>1 then preview:newEmptyFrame() end;preview.frames[f].duration=.12;preview:newCel(preview.layers[1],f,room(p,math.floor((f-1)/3)%2),Point(0,0)) end
preview:saveCopyAs(out..'preview/room-motion.gif')
-- Readable presentation contact sheet; exported game textures remain 64px.
local board=Image(768,768,ColorMode.INDEXED)
for y=0,767 do for x=0,767 do board:drawPixel(x,y,math.floor(x/12+y/12)%2==0 and 1 or 2) end end
for i=1,64 do tile(board,i,((i-1)%8)*96+16,math.floor((i-1)/8)*96+16) end
board:saveAs{filename=out..'preview/tiles.png',palette=pal}
print('Rendered assembled room, animated door/diagnostic proof and tile contact sheet.')
