-- A bounded visual study; the production prototype tiles are not modified.
local out='assets/environment/derelict-ship-depth-study/'
local src=app.open('assets/environment/derelict-ship/ship.aseprite');local pal=src.palettes[1]
local f=assert(io.open('assets/environment/derelict-ship/manifest.json'));local manifest=json.decode(f:read('*a'));f:close()
local ids={};for _,t in ipairs(manifest.tiles) do ids[t.id]=math.floor(t.atlas_index)+1 end
local engineer=app.open('assets/characters/engineer/engineer.aseprite')
local W,H,O=384,352,24
local im
local function px(x,y,c) x,y=math.floor(x),math.floor(y);if x>=0 and y>=0 and x<W and y<H then im:drawPixel(x,y,c) end end
local function rect(x,y,w,h,c) for yy=y,y+h-1 do for xx=x,x+w-1 do px(xx,yy,c) end end end
local function line(x1,y1,x2,y2,c,w) local n=math.max(1,math.ceil(math.max(math.abs(x2-x1),math.abs(y2-y1))));for i=0,n do local t=i/n;rect(math.floor(x1+(x2-x1)*t+.5),math.floor(y1+(y2-y1)*t+.5),w or 1,w or 1,c) end end
local function poly(p,c)
 local lo,hi=999,-999;for _,v in ipairs(p) do lo=math.min(lo,v[2]);hi=math.max(hi,v[2]) end
 for y=lo,hi do local xs={};for i,a in ipairs(p) do local b=p[i%#p+1];if (a[2]<=y+.5 and b[2]>y+.5) or (b[2]<=y+.5 and a[2]>y+.5) then xs[#xs+1]=a[1]+(y+.5-a[2])*(b[1]-a[1])/(b[2]-a[2]) end end;table.sort(xs);for k=1,#xs-1,2 do for x=math.ceil(xs[k]-.5),math.floor(xs[k+1]-.5) do px(x,y,c) end end end
end
local function tile(id,x,y) local cel=src.layers[1]:cel(ids[id]);im:drawImage(cel.image,Point(x+cel.position.x,y+cel.position.y)) end
local function hero()local cel=engineer.layers[1]:cel(3);im:drawImage(cel.image,Point(144-32+cel.position.x,292-38+cel.position.y))end
local function floor()
 im=Image(W,H,ColorMode.INDEXED);rect(0,0,W,H,1)
 for y=0,5 do for x=0,5 do tile('space_'..((x+y)%2),x*64,y*64) end end
 for y=0,4 do for x=0,5 do
  local edge=(y==0 and 'n' or y==4 and 's' or '')..(x==0 and 'w' or x==5 and 'e' or '')
  local id=edge~='' and 'floor_edge_'..edge or ('floor_panel_'..((x+y)%3==0 and 'b' or 'a'))
  tile(id,x*64,y*64+O)
 end end
 tile('floor_grate',256,152);tile('floor_drain',192,216)
end
local names={[0]='pillar','n','e','ne','s','ns','es','nes','w','nw','ew','new','sw','nsw','esw','nesw'}
local function occupied(x,y)return x>=0 and x<=5 and y>=0 and y<=4 and (x==0 or x==5 or y==0 or y==4)end
local function old()
 floor()
 for y=0,4 do for x=0,5 do if occupied(x,y) then
  local mask=(occupied(x,y-1) and 1 or 0)+(occupied(x+1,y) and 2 or 0)+(occupied(x,y+1) and 4 or 0)+(occupied(x-1,y) and 8 or 0)
  tile(x==3 and y==0 and 'door_ew_0' or x==5 and y==2 and 'wall_breach_ns' or 'wall_'..names[mask],x*64,y*64+O)
 end end end
 tile('diagnostic_0',64,88);hero();return im
end
local function mask(x,y)
 local yy=y-O
 local base=(x>=20 and x<=363 and ((yy>=20 and yy<=43) or (yy>=276 and yy<=299))) or (yy>=20 and yy<=299 and ((x>=20 and x<=43) or (x>=340 and x<=363)))
 if not base then return false end
 -- Open the frame for a recessed door, then add the raised header separately.
 if x>=206 and x<=241 and yy>=20 and yy<=43 then return false end
 -- Wider physical gap keeps a visible aperture behind the elevated near stump.
 if x>=340 and x<=363 then
  local j=math.floor((x-340)/4)+1;local a=({0,2,-2,3,0,-1})[j];local b=({2,-1,1,-2,3,0})[j]
  if yy>=139+a and yy<=182+b then return false end
 end
 return true
end
local function depthWall(cut)
 local function height(x,y) return cut and y-O>=276 and x>=44 and x<=339 and 6 or 22 end
 -- One view and height field for every wall orientation. Vertical side faces
 -- are visible only where the fixed camera actually exposes them.
 for y=0,H-1 do for x=0,W-1 do
  local hit,h,gy=-1,0,0
  for z=0,22 do local yy=y+z;if mask(x,yy) and z<=height(x,yy) then hit,h,gy=z,height(x,yy),yy end end
  if hit>=0 then
   local c=3
   if hit==h then
    local edge,rim=false,false
    for _,d in ipairs({{-1,0},{1,0},{0,-1},{0,1}}) do
     if not mask(x+d[1],gy+d[2]) then edge=true end
     if not mask(x+2*d[1],gy+2*d[2]) then rim=true end
    end
    c=edge and 1 or rim and 8 or 5
    local lx,ly=x%64,(gy-O)%64
    if not edge and not rim then
     if lx>=28 and lx<=35 and ly>=28 and ly<=35 then c=4 end
     if (lx==9 or lx==50) and ly>=25 and ly<=38 then c=3 end
     if (ly==9 or ly==50) and lx>=25 and lx<=38 then c=3 end
     if lx>=30 and lx<=36 and ly==30 then c=7 end
     if lx>=29 and lx<=35 and ly==31 then c=2 end
    end
   else
    c=hit<2 and 1 or hit<5 and 2 or hit>h-4 and 4 or 3
    if x%64==8 or x%64==51 then c=2 end
    if x%64==9 or x%64==52 then c=4 end
    if hit==h-5 and (x%64==16 or x%64==47) then c=7 end
    if x>=340 and x<=363 and gy-O<200 and gy-O>120 and hit>h-6 then c=6 end
   end
   px(x,y,c)
  end
 end end
 -- The door sits behind its raised lintel and two substantial jambs.
 rect(206,46,36,22,1);rect(208,48,14,17,4);rect(226,48,14,17,4)
 rect(210,49,10,2,6);rect(228,49,10,2,6);rect(221,49,2,16,7);rect(225,49,2,16,7)
 rect(213,57,6,3,2);rect(230,57,6,3,2);rect(207,65,34,3,2)
 rect(202,27,4,39,1);rect(202,28,2,35,7);rect(242,27,4,39,1);rect(244,28,2,35,7)
 rect(201,22,46,24,5);rect(201,22,46,1,1);rect(202,23,44,1,8);rect(201,43,46,3,2)
 rect(208,28,32,10,3);rect(210,29,28,2,7);rect(218,33,12,2,13)
 rect(197,48,3,9,1);rect(198,49,1,4,13)
 -- Torn metal and hanging cables follow the actual exposed cut ends.
 line(341,146,344,141,8,2);line(349,146,353,142,7,2);line(358,145,362,140,8,2)
 line(342,162,346,173,1,2);line(346,173,351,175,1,2);line(342,161,347,172,6);line(347,172,351,174,6)
 line(362,185,368,179,6,2);line(357,184,362,180,8)
end
local function diagnostic()
 -- A raised, sloped housing and visible cabinet front, with a low footprint.
 rect(80,145,34,7,1);rect(84,143,26,5,3);rect(89,149,5,3,4);rect(104,149,5,3,4)
 poly({{73,111},{118,111},{122,128},{116,145},{77,145},{71,128}},1)
 poly({{75,115},{116,115},{120,127},{73,127}},6)
 poly({{73,129},{120,129},{115,143},{78,143}},3)
 line(77,131,115,131,5);rect(83,134,26,6,2);rect(86,135,2,4,4);rect(103,135,2,4,4)
 poly({{73,84},{117,84},{122,90},{120,112},{71,112},{68,90}},1)
 poly({{74,86},{116,86},{120,90},{118,109},{73,109},{70,90}},5)
 rect(75,89,39,18,1);rect(77,91,35,14,10);rect(78,92,33,1,12)
 line(79,100,85,100,13);line(85,100,89,94,14);line(89,94,93,103,13);line(93,103,97,98,13);line(97,98,102,98,13)
 rect(106,95,4,2,13);rect(106,100,3,2,12);line(77,87,112,87,8)
 for y=116,122,4 do for x=79,99,4 do rect(x,y,2,1,8) end end
 rect(108,117,6,3,1);px(110,118,13)
 line(80,111,89,110,2);line(82,112,88,111,7)
end
local function raised(cut)
 floor();diagnostic();hero();depthWall(cut);return im
end
local variants={{name='current',image=old()},{name='raised-solid',image=raised(false)},{name='raised-cutaway',image=raised(true)}}
local spr=Sprite(W,H,ColorMode.INDEXED);spr:setPalette(pal);spr.layers[1].name='Depth study'
for i,v in ipairs(variants) do if i>1 then spr:newEmptyFrame() end;spr.frames[i].duration=1;spr:newCel(spr.layers[1],i,v.image,Point(0,0));v.image:saveAs{filename=out..v.name..'.png',palette=pal} end
for i,v in ipairs(variants) do local tag=spr:newTag(i,i);tag.name=v.name end
spr:saveAs(out..'depth-study.aseprite')
local board=Image(784,352,ColorMode.INDEXED)
for y=0,351 do for x=0,783 do board:drawPixel(x,y,1) end end
board:drawImage(variants[1].image,Point(0,0));board:drawImage(variants[3].image,Point(400,0))
board:saveAs{filename=out..'comparison.png',palette=pal}
print('Exported current/raised/full-height/cutaway study, comparison and editable Aseprite. Original tiles unchanged.')
