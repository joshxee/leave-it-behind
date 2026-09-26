-- Rebuild from repository root with sprite-axi run <this file>.
local OUT='assets/environment/derelict-ship/'
local hex={'000000','101c29','1b2d3e','2c4255','405c70','56778b','7595a5','9ab3bd','bfd0d6','e6eff1','12354f','1c587b','3388ad','61bbd2','a0e0eb','8198a3'}
local pal=Palette(16)
for i,h in ipairs(hex) do pal:setColor(i-1,Color{r=tonumber(h:sub(1,2),16),g=tonumber(h:sub(3,4),16),b=tonumber(h:sub(5,6),16),a=i==1 and 0 or 255}) end
local im
local function px(x,y,c) x,y=math.floor(x),math.floor(y);if x>=0 and y>=0 and x<im.width and y<im.height then im:drawPixel(x,y,c) end end
local function rect(x,y,w,h,c) for yy=y,y+h-1 do for xx=x,x+w-1 do px(xx,yy,c) end end end
local function line(x1,y1,x2,y2,c,w)
 local n=math.max(1,math.ceil(math.max(math.abs(x2-x1),math.abs(y2-y1))))
 for i=0,n do local t=i/n;rect(math.floor(x1+(x2-x1)*t+.5),math.floor(y1+(y2-y1)*t+.5),w or 1,w or 1,c) end
end
local function poly(p,c)
 local ymin,ymax=999,-999;for _,v in ipairs(p) do ymin=math.min(ymin,v[2]);ymax=math.max(ymax,v[2]) end
 for y=math.floor(ymin),math.ceil(ymax) do local xs={};for i=1,#p do local a,b=p[i],p[i%#p+1];if (a[2]<=y+.5 and b[2]>y+.5) or (b[2]<=y+.5 and a[2]>y+.5) then xs[#xs+1]=a[1]+(y+.5-a[2])*(b[1]-a[1])/(b[2]-a[2]) end end;table.sort(xs);for i=1,#xs-1,2 do for x=math.ceil(xs[i]-.5),math.floor(xs[i+1]-.5) do px(x,y,c) end end end
end
local function ellipse(cx,cy,rx,ry,c) for y=math.floor(cy-ry),math.ceil(cy+ry) do for x=math.floor(cx-rx),math.ceil(cx+rx) do if (x-cx)^2/rx^2+(y-cy)^2/ry^2<=1 then px(x,y,c) end end end end
local function plate(x,y,w,h,base)
 poly({{x+3,y},{x+w-4,y},{x+w-1,y+3},{x+w-1,y+h-4},{x+w-4,y+h-1},{x+3,y+h-1},{x,y+h-4},{x,y+3}},1)
 rect(x+2,y+3,w-4,h-6,base);rect(x+3,y+2,w-6,1,7);rect(x+3,y+h-4,w-6,2,3)
end
local function bolt(x,y) rect(x,y,3,3,1);rect(x,y,2,1,7);px(x+1,y+1,4) end
local function scar(x,y,w) line(x,y,x+w,y-1,2);line(x+1,y+1,x+w-1,y,7) end
local function smallscreen(x,y,w,h,on)
 rect(x,y,w,h,1);rect(x+1,y+1,w-2,h-2,on and 10 or 2)
 rect(x+2,y+2,w-4,1,on and 12 or 3)
 if on then for yy=y+5,y+h-3,4 do rect(x+3,yy,math.max(2,w-8-(yy%3)*2),1,11) end;rect(x+w-4,y+4,1,h-7,12) end
end
local function basefloor(kind)
 rect(0,0,64,64,2);rect(0,0,64,1,1);rect(0,63,64,1,1);rect(0,0,1,64,1);rect(63,0,1,64,1)
 rect(2,2,60,1,3);rect(2,2,1,60,3);rect(2,61,60,1,1);rect(61,2,1,60,1)
 for _,p in ipairs({{5,5},{56,5},{5,56},{56,56}}) do rect(p[1],p[2],3,3,1);px(p[1],p[2],4) end
 if kind=='panel_a' then line(11,19,20,17,3);line(12,20,17,19,4);line(40,42,47,41,1)
 elseif kind=='panel_b' then rect(10,11,44,42,3);rect(12,13,40,38,2);line(14,42,23,41,3);line(42,18,47,16,4);rect(28,48,7,1,4)
 elseif kind=='panel_c' then line(11,31,52,31,1);line(11,32,52,32,3);rect(18,12,5,2,3);rect(37,43,12,2,1);rect(39,45,7,1,3)
 elseif kind=='grate' then plate(7,7,50,50,3);for y=12,51,5 do for x=11,50,8 do rect(x,y,6,3,1);rect(x,y,6,1,4) end end;rect(29,10,5,44,2);rect(30,10,1,43,5)
 elseif kind=='hatch' then plate(9,8,46,48,4);rect(14,13,36,37,2);rect(16,15,32,33,3);rect(29,24,6,16,1);rect(30,25,3,13,5);for _,x in ipairs({17,43}) do bolt(x,16);bolt(x,43) end;scar(20,20,8)
 elseif kind=='reinforced' then for _,y in ipairs({12,45}) do rect(8,y,48,7,1);rect(9,y+1,46,4,4);rect(9,y+1,46,1,5) end;for _,x in ipairs({12,48}) do bolt(x,13);bolt(x,46) end;rect(17,26,30,10,3);rect(21,29,22,1,4)
 elseif kind=='drain' then plate(18,18,28,28,3);for x=22,40,4 do rect(x,23,2,18,1);rect(x+1,23,1,17,4) end;line(12,45,17,43,1)
 elseif kind=='conduit_h' or kind=='conduit_v' then
  local v=kind=='conduit_v';for b=22,40 do for a=5,58 do local c=(b==22 or b==40) and 4 or (b==25 or b==35) and 5 or (b==26 or b==36) and 3 or 1;if v then px(b,a,c) else px(a,b,c) end end end
  for _,a in ipairs({10,49}) do if v then rect(20,a,23,5,2);rect(21,a,21,1,6) else rect(a,20,5,23,2);rect(a,21,1,21,6) end end
 end
end
local function wallmask(mask,x,y)
 if x>=20 and x<=43 and y>=20 and y<=43 then return true end
 if mask&1~=0 and x>=20 and x<=43 and y<32 then return true end
 if mask&2~=0 and y>=20 and y<=43 and x>=32 then return true end
 if mask&4~=0 and x>=20 and x<=43 and y>=32 then return true end
 if mask&8~=0 and y>=20 and y<=43 and x<32 then return true end
 return false
end
local function wall(mask)
 -- Extruded dark underside, bright flat top: the light remains overhead.
 for y=0,63 do for x=0,63 do
  if wallmask(mask,x,y-5) then px(x,y,wallmask(mask,x,y-4) and 3 or 1) end
  if wallmask(mask,x,y) then
   local edge=false;local rim=false
   for _,d in ipairs({{-1,0},{1,0},{0,-1},{0,1}}) do if not wallmask(mask,x+d[1],y+d[2]) then edge=true end;if not wallmask(mask,x+d[1]*2,y+d[2]*2) then rim=true end end
   px(x,y,edge and 1 or rim and 7 or 5)
  end
 end end
 -- Panel divisions and cable channels do not touch the matching tile edges.
 if mask&2~=0 or mask&8~=0 then
  for _,x in ipairs({8,51}) do if wallmask(mask,x,30) then rect(x,24,4,15,3);rect(x,24,1,15,6);bolt(x,27) end end
 end
 if mask&1~=0 or mask&4~=0 then
  for _,y in ipairs({8,51}) do if wallmask(mask,30,y) then rect(24,y,15,4,3);rect(24,y,15,1,6);bolt(27,y) end end
 end
 rect(25,25,14,14,4);rect(26,26,12,10,5);rect(28,28,8,1,6)
 scar(29,33,6)
 if mask==0 then bolt(23,23);bolt(38,38) end
end
local function breach(vertical,repaired)
 wall(vertical and 5 or 10)
 -- A serrated cut runs right through the load-bearing band.
 for y=0,63 do for x=0,63 do local u,v=vertical and y or x,vertical and x or y
  local left=23+({0,2,-1,4,1,-2,2,0})[math.floor(v/8)+1]
  local right=42+({0,-2,2,-1,3,0,-2,1})[math.floor(v/8)+1]
  if u>=left and u<=right then px(x,y,0) end
 end end
 local function p(u,v,c) if vertical then px(v,u,c) else px(u,v,c) end end
 local function l(u1,v1,u2,v2,c,w) if vertical then line(v1,u1,v2,u2,c,w) else line(u1,v1,u2,v2,c,w) end end
 for _,v in ipairs({22,29,36,43}) do l(17,v,25+(v%3),v-3,8,2);l(40,v+2,48,v,6,2);p(26,v-3,9) end
 l(22,41,28,46,1,2);l(28,46,32,44,1,2);l(22,40,28,45,12);l(28,45,32,43,12)
 l(44,21,37,15,1,2);l(44,20,37,14,6);l(38,14,35,16,6)
 if repaired then
  for v=19,47 do for u=22,43 do p(u,v,(v==19 or v==47 or u==22 or u==43) and 1 or 4) end end
  for _,v in ipairs({21,40}) do for u=18,47 do for b=0,4 do p(u,v+b,b==0 and 8 or b==4 and 6 or 15) end end end
  for u=27,39 do p(u,31,2);p(u,32,6) end
 end
end
local function door(vertical,phase,locked)
 wall(vertical and 5 or 10)
 local function p(u,v,c) if vertical then px(v,u,c) else px(u,v,c) end end
 local function r(u,v,w,h,c) for b=v,v+h-1 do for a=u,u+w-1 do p(a,b,c) end end end
 -- Clear aperture, retain a rail and two matching bulkhead ends.
 r(12,20,40,30,0);r(10,15,44,5,1);r(12,16,40,2,6)
 r(10,20,5,29,1);r(49,20,5,29,1);r(11,21,2,24,6);r(51,21,2,24,6)
 local retract=({0,6,12,19})[phase+1]
 for u=14,49 do for v=21,43 do
  local closed=(u<=31-retract or u>=32+retract)
  if closed then local seam=math.abs(u-31.5);local c=(v==21 or v==43) and 1 or (v==22 or v==41) and 7 or 4
   if seam<1 then c=2 elseif seam<2 then c=8 end
   if v>=29 and v<=35 and (u<25-retract or u>38+retract) then c=3 end
   p(u,v,c)
  end
 end end
 r(4,25,4,12,1);r(5,26,2,locked and 8 or 5,locked and 8 or 13)
 r(56,25,4,12,1);r(57,26,2,locked and 8 or 5,locked and 8 or 13)
 if locked then for i=0,7 do p(27+i,27+i,8);p(34-i,27+i,8) end end
end
local function diagnostic(phase)
 -- Elevated console with a chunky bezel, service spine, and low footplate.
 plate(23,32,19,27,3);rect(29,42,7,10,1);rect(30,43,2,8,5)
 plate(10,9,45,35,5);plate(13,12,39,25,2);smallscreen(16,14,33,20,phase~=2)
 if phase~=2 then
  line(18,25,23,25,13);line(23,25,26,19+phase,14);line(26,19+phase,29,29,13);line(29,29,32,23,13);line(32,23,37,23,13)
  rect(40,18,6,2,12);rect(40,23,4+phase,2,13);rect(40,28,6,2,11)
 else line(22,22,42,22,4);rect(31,19,2,7,5) end
 rect(15,38,19,3,1);for x=17,30,4 do px(x,39,7) end;rect(42,38,6,2,phase==2 and 4 or 13)
 bolt(12,12);bolt(50,12);scar(37,35,7);plate(17,54,31,6,3)
end
local function chair(x,y)
 ellipse(x+15,y+30,14,6,1);line(x+15,y+20,x+15,y+31,4,4)
 plate(x+1,y+9,29,22,3);plate(x+3,y+2,25,18,5);rect(x+8,y+6,15,8,3);rect(x+9,y+6,13,1,6)
 rect(x,y+16,5,14,1);rect(x+1,y+17,3,11,7);rect(x+26,y+16,5,14,1);rect(x+27,y+17,3,11,7)
 rect(x+10,y+20,12,5,4);scar(x+11,y+11,6)
end
local function console(x,y)
 plate(x,y,52,38,4);smallscreen(x+5,y+4,25,17,true);smallscreen(x+33,y+5,13,11,true)
 line(x+8,y+15,x+13,y+11,13);line(x+13,y+11,x+18,y+15,13);line(x+18,y+15,x+26,y+10,13)
 for yy=y+25,y+30,4 do for xx=x+7,x+29,4 do rect(xx,yy,2,1,7) end end
 ellipse(x+40,y+26,5,5,1);ellipse(x+40,y+25,3,3,6);px(x+40,y+24,9);bolt(x+2,y+3);scar(x+32,y+33,7)
end
local function cockpit()
 -- A single 192x128 master is cut without resampling into six grid cells.
 rect(0,20,192,24,5);rect(0,20,192,1,1);rect(0,21,192,1,7);rect(0,44,192,5,3);rect(0,48,192,1,1)
 poly({{0,20},{17,6},{175,6},{191,20},{186,58},{5,58}},1)
 poly({{4,22},{20,9},{171,9},{187,22},{181,53},{10,53}},6)
 poly({{9,24},{23,14},{168,14},{182,24},{177,49},{15,49}},10)
 for _,p in ipairs({{27,21},{48,37},{83,20},{105,39},{154,27},{169,41}}) do px(p[1],p[2],7);px(p[1]+1,p[2],5) end
 poly({{25,16},{66,16},{35,47},{18,47}},11);poly({{81,16},{91,16},{60,47},{50,47}},12)
 poly({{138,16},{144,16},{113,47},{108,47}},11)
 for _,x in ipairs({64,125}) do rect(x,12,5,40,1);rect(x+1,13,2,37,7) end
 line(26,11,166,11,8);line(17,52,176,52,7)
 line(156,17,151,25,4);line(151,25,159,32,4);line(151,25,144,27,4);px(150,25,8)
 plate(6,57,181,6,5)
 console(9,65);console(132,65)
 plate(64,62,64,29,5);smallscreen(69,66,53,20,true)
 ellipse(95,76,12,7,11);ellipse(95,76,7,4,12);line(95,68,95,83,13);line(82,76,107,76,13);rect(98,72,3,2,14)
 for _,x in ipairs({74,111}) do rect(x,94,6,18,1);rect(x+1,95,4,15,5);rect(x+2,97,2,3,13) end
 chair(81,92)
 for _,x0 in ipairs({0,188}) do for y=20,48 do rect(x0,y,4,1,(y==20 or y==43 or y==48) and 1 or (y==21 or y==42) and 7 or y>43 and 3 or 5) end end
end
local spr=Sprite(64,64,ColorMode.INDEXED);spr:setPalette(pal);spr.transparentColor=0;spr.layers[1].name='Ship tiles'
local manifest={name='Derelict ship prototype',tile_size={64,64},palette='ship-16.gpl',palette_matches='engineer-16',transparent_index=0,atlas_columns=8,tiles={},animations={},modules={}}
local images,indices={},{}
local function add(id,category,fn,properties)
 im=Image(64,64,ColorMode.INDEXED);fn();local n=#manifest.tiles
 if n>0 then spr:newEmptyFrame() end;spr.frames[n+1].duration=.1;spr:newCel(spr.layers[1],n+1,im,Point(0,0))
 im:saveAs{filename=OUT..'png/'..id..'.png',palette=pal};images[id]=im;indices[id]=n
 local tile={id=id,file='png/'..id..'.png',category=category,atlas_index=n,atlas_rect={(n%8)*64,math.floor(n/8)*64,64,64},anchor_pixels={32,32},collision={}}
 if properties then for k,v in pairs(properties) do tile[k]=v end end
 manifest.tiles[#manifest.tiles+1]=tile
end
for _,kind in ipairs({'panel_a','panel_b','panel_c','grate','conduit_h','conduit_v','hatch','reinforced','drain'}) do add('floor_'..kind,'floor',function()basefloor(kind)end,{layer='floor',walkable=true}) end
for _,edge in ipairs({'n','e','s','w','ne','se','sw','nw'}) do
 add('floor_edge_'..edge,'floor_edge',function()basefloor('panel_a');for y=0,63 do for x=0,63 do if (edge:find('n') and y<32) or (edge:find('s') and y>=32) or (edge:find('w') and x<32) or (edge:find('e') and x>=32) then px(x,y,0) end end end end,{layer='floor',walkable=true,clip_edges=edge})
end
for v=0,1 do add('space_'..v,'space',function()rect(0,0,64,64,1);for _,p in ipairs({{9,13},{44,7},{29,51},{56,36},{18,34}}) do px((p[1]+v*17)%64,(p[2]+v*23)%64,v==0 and 4 or 5) end end,{layer='background',walkable=false}) end
local wallnames={[0]='pillar','n','e','ne','s','ns','es','nes','w','nw','ew','new','sw','nsw','esw','nesw'}
for mask=0,15 do
 local collision={{20,20,24,24}};if mask&1~=0 then collision[#collision+1]={20,0,24,20} end;if mask&2~=0 then collision[#collision+1]={44,20,20,24} end;if mask&4~=0 then collision[#collision+1]={20,44,24,20} end;if mask&8~=0 then collision[#collision+1]={0,20,20,24} end
 add('wall_'..wallnames[mask],'wall',function()wall(mask)end,{layer='structure',connection_mask=mask,collision=collision})
end
for _,vertical in ipairs({false,true}) do local axis=vertical and 'ns' or 'ew'
 for _,repaired in ipairs({false,true}) do
  local c=repaired and (vertical and {{20,0,24,64}} or {{0,20,64,24}}) or (vertical and {{20,0,24,21},{20,45,24,19}} or {{0,20,21,24},{45,20,19,24}})
  add('wall_'..(repaired and 'patched_' or 'breach_')..axis,repaired and 'patched_wall' or 'breach',function()breach(vertical,repaired)end,{layer='structure',connection_mask=vertical and 5 or 10,collision=c,vacuum_hazard=not repaired})
 end
end
for _,vertical in ipairs({false,true}) do local axis=vertical and 'ns' or 'ew';local files={}
 for phase=0,3 do local id='door_'..axis..'_'..phase
  add(id,'door',function()door(vertical,phase,false)end,{layer='structure',connection_mask=vertical and 5 or 10,door_phase=phase,passable=phase==3,collision=phase==3 and (vertical and {{20,0,24,14},{20,50,24,14}} or {{0,20,14,24},{50,20,14,24}}) or (vertical and {{20,0,24,64}} or {{0,20,64,24}})})
  files[#files+1]='png/'..id..'.png'
 end
 manifest.animations[#manifest.animations+1]={id='door_'..axis..'_open',files=files,duration_ms=100,loop=false,reverse_for_close=true}
 add('door_'..axis..'_locked','door',function()door(vertical,0,true)end,{layer='structure',connection_mask=vertical and 5 or 10,passable=false,locked=true,collision=vertical and {{20,0,24,64}} or {{0,20,64,24}}})
end
im=Image(192,128,ColorMode.INDEXED);cockpit();local module=im;module:saveAs{filename=OUT..'cockpit-module.png',palette=pal}
local moduleids={}
for y=0,1 do for x=0,2 do local id='cockpit_'..x..'_'..y;moduleids[#moduleids+1]=id
 add(id,'cockpit',function()for yy=0,63 do for xx=0,63 do px(xx,yy,module:getPixel(x*64+xx,y*64+yy)) end end end,{layer='prop',module='cockpit',module_cell={x,y},collision=y==0 and {{0,8,64,50}} or (x==1 and {{0,0,64,27},{18,30,30,34}} or {{8,1,48,38}})})
end end
manifest.modules[#manifest.modules+1]={id='cockpit',size_tiles={3,2},tiles=moduleids,master='cockpit-module.png',facing='north'}
for phase=0,2 do add('diagnostic_'..phase,'diagnostic',function()diagnostic(phase)end,{layer='prop',active=phase~=2,collision={{10,16,45,43}},interaction_pixels={32,59}}) end
manifest.animations[#manifest.animations+1]={id='diagnostic_online',files={'png/diagnostic_0.png','png/diagnostic_1.png'},duration_ms=350,loop=true}
add('prop_pilot_chair','prop',function()chair(17,15)end,{layer='prop',collision={{17,20,31,31}}})
add('prop_control_console','prop',function()console(6,15)end,{layer='prop',collision={{6,17,52,36}},interaction_pixels={32,53}})
add('prop_locker','prop',function()plate(13,7,38,50,5);rect(18,13,28,36,3);rect(31,13,1,36,1);for _,x in ipairs({21,37}) do for y=17,23,3 do rect(x,y,7,1,1) end end;rect(26,31,2,8,7);rect(35,31,2,8,7);scar(38,43,6);rect(18,52,28,3,1)end,{layer='prop',collision={{13,7,38,50}}})
add('prop_crate','prop',function()plate(10,14,44,39,4);rect(15,19,34,25,3);poly({{17,19},{21,19},{47,42},{43,42}},5);poly({{43,19},{47,19},{21,42},{17,42}},5);for _,x in ipairs({13,47}) do bolt(x,17);bolt(x,46) end;rect(24,26,15,12,1);rect(26,28,11,8,4);scar(28,31,6)end,{layer='prop',collision={{10,14,44,39}}})
add('prop_oxygen_rack','prop',function()plate(10,10,45,47,3);for _,x in ipairs({18,34}) do ellipse(x+5,18,6,7,1);rect(x,18,11,28,5);rect(x+1,18,3,27,7);ellipse(x+5,45,5,4,4);rect(x+3,10,5,5,1);rect(x+4,10,3,2,8);rect(x-2,29,15,4,2);rect(x-1,29,13,1,6);rect(x+4,20,3,6,12)end;rect(14,50,37,3,5)end,{layer='prop',collision={{10,10,45,47}}})
add('prop_pipe_stack','prop',function()plate(10,9,44,47,3);for _,y in ipairs({17,29,41}) do line(14,y,48,y,1,7);line(15,y+1,46,y+1,6,4);line(15,y+1,46,y+1,8);rect(21,y-1,4,9,2);rect(39,y-1,4,9,2);rect(22,y,1,7,5)end end,{layer='prop',collision={{10,9,44,47}}})
assert(#manifest.tiles==64,'Expected 64 prototype tiles')
for _,a in ipairs(manifest.animations) do local first=a.files[1]:match('png/(.+)%.png');local last=a.files[#a.files]:match('png/(.+)%.png');local tag=spr:newTag(indices[first]+1,indices[last]+1);tag.name=a.id;for _,file in ipairs(a.files) do local id=file:match('png/(.+)%.png');spr.frames[indices[id]+1].duration=a.duration_ms/1000 end end
local slice=spr:newSlice(Rectangle(0,0,64,64));slice.name='grid_cell';slice.pivot=Point(32,32)
spr:saveAs(OUT..'ship.aseprite')
app.command.ExportSpriteSheet{ui=false,type=SpriteSheetType.ROWS,columns=8,textureFilename=OUT..'ship-atlas.png',dataFilename=OUT..'ship-atlas.aseprite.json',listTags=true,listSlices=true}
local file=assert(io.open(OUT..'manifest.json','w'));file:write(json.encode(manifest));file:close()
for _,ext in ipairs({'gpl','pal'}) do local src=assert(io.open('assets/characters/engineer/engineer-16.'..ext,'rb'));local contents=src:read('*a');src:close();local dst=assert(io.open(OUT..'ship-16.'..ext,'wb'));dst:write(contents);dst:close() end
print('Created 64 indexed grid tiles, six-cell cockpit, doors, diagnostics, atlas, native source and collision metadata.')
