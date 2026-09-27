-- Reproducible indexed pixel art. Run from the repository root with sprite-axi.
local OUT='assets/maintenance/'
app.fs.makeAllDirectories(OUT..'png')
local pal=app.open('assets/environment/derelict-ship/ship.aseprite').palettes[1]
local im
local function px(x,y,c) x,y=math.floor(x),math.floor(y);if x>=0 and y>=0 and x<im.width and y<im.height then im:drawPixel(x,y,c) end end
local function rect(x,y,w,h,c) for yy=y,y+h-1 do for xx=x,x+w-1 do px(xx,yy,c) end end end
local function line(x,y,xx,yy,c,w)
 local n=math.max(1,math.ceil(math.max(math.abs(xx-x),math.abs(yy-y))))
 for k=0,n do rect(math.floor(x+(xx-x)*k/n+.5),math.floor(y+(yy-y)*k/n+.5),w or 1,w or 1,c) end
end
local function poly(p,c)
 local lo,hi=9999,-9999;for _,v in ipairs(p) do lo=math.min(lo,v[2]);hi=math.max(hi,v[2]) end
 for y=math.floor(lo),math.ceil(hi) do local xs={};for i,a in ipairs(p) do local b=p[i%#p+1];if (a[2]<=y+.5 and b[2]>y+.5) or (b[2]<=y+.5 and a[2]>y+.5) then xs[#xs+1]=a[1]+(y+.5-a[2])*(b[1]-a[1])/(b[2]-a[2]) end end;table.sort(xs);for k=1,#xs-1,2 do for x=math.ceil(xs[k]-.5),math.floor(xs[k+1]-.5) do px(x,y,c) end end end
end
local function ellipse(x,y,rx,ry,c)
 for yy=math.floor(y-ry),math.ceil(y+ry) do for xx=math.floor(x-rx),math.ceil(x+rx) do if (xx-x)^2/rx^2+(yy-y)^2/ry^2<=1 then px(xx,yy,c) end end end
end
local function ring(x,y,r,c) for a=0,719 do local t=a*math.pi/360;px(x+math.cos(t)*r+.5,y+math.sin(t)*r+.5,c) end end
local function rivet(x,y) rect(x,y,3,3,1);rect(x,y,2,1,8);px(x+1,y+1,4) end
local glyph={A={'010','101','111','101','101'},C={'011','100','100','100','011'},D={'110','101','101','101','110'},E={'111','100','110','100','111'},G={'011','100','101','101','011'},I={'111','010','010','010','111'},K={'101','101','110','101','101'},L={'100','100','100','100','111'},N={'101','111','111','111','101'},O={'010','101','101','101','010'},R={'110','101','110','101','101'},S={'011','100','010','001','110'},T={'111','010','010','010','010'},V={'101','101','101','101','010'},['0']={'111','101','101','101','111'},['1']={'010','110','010','010','111'},['2']={'110','001','010','100','111'},['3']={'110','001','010','001','110'},['/']={'001','001','010','100','100'},['-']={'000','000','111','000','000'}}
local function text(s,x,y,c) for ch in s:gmatch('.') do local g=glyph[ch];if g then for yy,row in ipairs(g) do for xx=1,#row do if row:sub(xx,xx)=='1' then px(x+xx-1,y+yy-1,c) end end end end;x=x+4 end end
local manifest={palette='../characters/engineer/engineer-16.pal',sampling='nearest',assets={}}
local function asset(name,w,h,frames,ms,draw)
 local s=Sprite(w,h,ColorMode.INDEXED);s:setPalette(pal);s.layers[1].name=name
 local cols=math.min(frames,32)
 local sheet=Image(w*cols,h*math.ceil(frames/cols),ColorMode.INDEXED)
 for f=0,frames-1 do
  im=Image(w,h,ColorMode.INDEXED);draw(f)
  if f>0 then s:newEmptyFrame() end
  s.frames[f+1].duration=ms/1000;s:newCel(s.layers[1],f+1,im,Point(0,0))
  im:saveAs{filename=OUT..'png/'..name..'_'..f..'.png',palette=pal};sheet:drawImage(im,Point(w*(f%cols),h*math.floor(f/cols)))
 end
 local tag=s:newTag(1,frames);tag.name=name
 s:saveAs(OUT..name..'.aseprite');sheet:saveAs{filename=OUT..name..'.png',palette=pal}
 manifest.assets[#manifest.assets+1]={id=name,frame_size={w,h},frames=frames,columns=cols,frame_ms=ms,file=name..'.png'}
end

asset('radar',128,144,1,100,function()
 -- CRT set into a deep steel instrument housing, illuminated glass and scan grid.
 poly({{9,5},{118,5},{124,11},{124,132},{117,140},{10,140},{3,132},{3,11}},1)
 rect(7,11,114,120,3);rect(9,7,109,3,7);rect(5,13,2,114,5);rect(10,132,107,5,2)
 rect(11,12,106,106,1);rect(13,14,102,102,10)
 for y=17,113,4 do rect(15,y,98,1,2) end
 for n=24,104,16 do line(n,15,n,113,11);line(15,n,113,n,11) end
 for _,r in ipairs({20,38,49}) do ring(64,64,r,12) end
 line(64,15,64,113,12);line(15,64,113,64,12)
 for n=0,31 do local a=n*math.pi/16;local r=n%4==0 and 44 or 47;line(64+math.cos(a)*r,64+math.sin(a)*r,64+math.cos(a)*49,64+math.sin(a)*49,13) end
 -- Darker circular corners let the target remain unmistakably square.
 rect(15,15,15,2,13);rect(15,18,8,1,11);rect(104,110,8,2,13)
 text('NAV / 03',15,122,7);text('LOCK',82,122,13)
 rect(15,131,94,3,1);rect(16,132,9,1,12)
 for _,p in ipairs({{8,8},{116,8},{8,132},{115,132}}) do rivet(p[1],p[2]) end
end)
asset('radar-sweep',104,104,24,100,function(f)
 local a=f*math.pi/12
 for t=0,12 do local angle=a-t*.024;line(52,52,52+math.cos(angle)*49,52+math.sin(angle)*49,t<2 and 13 or t<6 and 12 or 11) end
end)
asset('nav-ship',16,20,2,180,function(f)
 poly({{8,1},{11,7},{11,10},{15,14},{15,17},{10,15},{8,17},{6,15},{1,17},{1,14},{5,10},{5,7}},1)
 poly({{8,2},{10,8},{10,12},{14,15},{10,14},{8,16},{6,14},{2,15},{6,12},{6,8}},13)
 line(8,4,8,12,9);rect(7,8,3,4,14);rect(7,12,3,2,11)
 rect(6,16,1,2+f,12);rect(10,16,1,2+f,12);px(8,1,9)
end)
asset('helm-base',32,32,1,100,function()
 poly({{7,5},{25,5},{30,10},{30,24},{24,29},{7,29},{2,24},{2,10}},1)
 poly({{7,6},{24,6},{28,10},{28,22},{23,26},{8,26},{4,22},{4,10}},5)
 line(8,7,23,7,8);line(5,11,5,21,7);line(8,26,23,26,3)
 ellipse(16,15,8,7,1);ellipse(16,15,6,5,3);ellipse(16,15,4,3,2)
 rect(8,22,4,2,12);rect(20,22,4,2,13);rivet(6,9);rivet(24,9)
end)
asset('helm-stick',12,18,1,100,function()
 ellipse(6,14,4,2,1);rect(4,6,4,8,3);rect(5,7,1,7,7)
 ellipse(6,5,5,4,1);ellipse(6,4,4,3,12);line(4,2,7,2,14);px(4,3,9)
end)
asset('engine',224,88,4,180,function(f)
 -- Footprint is x=12..211, y=26..81. The top is elevated by 18px.
 ellipse(112,73,105,13,1)
 for _,x in ipairs({19,190}) do rect(x,24,14,57,1);rect(x+2,25,10,54,4);rect(x+3,27,3,48,7);rect(x,78,16,4,2) end
 poly({{16,18},{24,9},{198,9},{209,19},{209,64},{198,75},{26,75},{16,65}},1)
 rect(23,20,180,43,4);rect(27,64,169,9,3);line(28,65,194,65,6)
 for y=67,71,2 do line(32,y,191,y,2) end
 -- Three large access plates; pipes run between them.
 for _,x in ipairs({37,97,157}) do
  poly({{x,18},{x+7,11},{x+42,11},{x+48,18},{x+48,50},{x+42,61},{x+5,61},{x,53}},2)
  poly({{x+2,19},{x+9,13},{x+40,13},{x+45,19},{x+45,48},{x+39,56},{x+8,56},{x+2,49}},5)
  line(x+9,13,x+39,13,8);line(x+3,21,x+3,47,7);line(x+9,56,x+37,56,1)
  rect(x+11,20,25,27,1);rect(x+12,21,23,25,3)
  for yy=23,43,4 do rect(x+14,yy,19,2,1);rect(x+14,yy,19,1,6) end
  rect(x+17,50,15,3,10);rect(x+18,50,13,1,f%2==0 and 13 or 12)
  rivet(x+6,18);rivet(x+38,18);rivet(x+6,49);rivet(x+38,49)
  line(x+27,16,x+33,15,8);line(x+28,17,x+33,16,3)
 end
 for _,x in ipairs({30,90,150}) do
  rect(x,8,5,52,1);rect(x+1,8,3,52,6);rect(x+1,9,1,50,8)
  for _,y in ipairs({18,44}) do rect(x-1,y,7,4,2);rect(x,y,5,1,5) end
 end
 -- Coolant manifold, small gauge and side vents.
 rect(45,4,135,6,1);rect(46,5,132,3,6);line(48,5,176,5,8)
 rect(14,30,9,27,2);for y=32,53,4 do rect(15,y,6,2,7) end
 ellipse(199,31,6,8,1);ellipse(199,30,4,6,10);line(199,30,201,26,14)
 rect(196,47,7,8,1);rect(198,49,3,4,12+f%2)
 text('ION - 03',86,66,7)
 for _,x in ipairs({50,110,170}) do rect(x,76,5,5,1);rect(x+1,76,3,2,8) end
end)
asset('bolt',32,32,9,65,function(f)
 -- The fixed socket is at (16,22); the shank retracts while the hex turns.
 ellipse(16,23,9,5,1);ellipse(16,22,8,4,4);ellipse(16,22,5,3,2)
 local lift=math.floor((8-f)*1.25+.5);local cy=22-lift
 rect(12,cy,8,lift+2,3);rect(13,cy,2,lift+1,7)
 for y=cy+3,23,3 do line(12,y,19,y-2,1);line(13,y,19,y-2,6) end
 local a=f*math.pi/4;local vertices={}
 for k=0,5 do local t=a+k*math.pi/3;vertices[#vertices+1]={16+math.cos(t)*8,cy+math.sin(t)*5} end
 local lower={};for _,p in ipairs(vertices) do lower[#lower+1]={p[1],p[2]+3} end
 poly(lower,1);poly(vertices,7);line(11,cy-2,15,cy-4,9)
 ellipse(16,cy,3,2,2);line(16-math.cos(a)*3,cy-math.sin(a)*2,16+math.cos(a)*3,cy+math.sin(a)*2,1)
 px(20,cy+2,5)
end)
asset('tape-strip',32,12,6,70,function(f)
 local w=math.floor(32*(f+1)/6)
 for y=2,9 do for x=0,w-1 do
  local edge=(x==0 and y%3==0) or (x==w-1 and y%3==1)
  if not edge then px(x,y,y==9 and 4 or y==2 and 8 or 15) end
 end end
 for x=2,w-2,4 do px(x,4,7);px(x+1,7,6) end
 if w>9 then line(4,6,math.min(w-3,26),6,7) end
 if f<5 then rect(w-2,2,2,7,8);rect(w-2,9,2,1,5) end
end)
asset('tape-roll',24,24,4,90,function(f)
 ellipse(12,14,10,7,1);ellipse(12,13,9,6,5);ellipse(12,10,10,7,1);ellipse(12,9,9,6,15)
 ellipse(12,9,7,4,8);ellipse(12,9,4,3,1);ellipse(12,10,3,2,3)
 line(5,15,8,17,7);line(17,14,20,12,7)
 local a=f*math.pi/2;line(12+math.cos(a)*5,9+math.sin(a)*3,12+math.cos(a)*8,9+math.sin(a)*5,6)
 poly({{19,11},{23,14},{23,20},{19,18}},15);line(20,13,22,15,8)
end)
asset('wrench',40,16,1,100,function()
 poly({{1,7},{4,5},{25,5},{28,1},{35,1},{32,5},{32,9},{36,10},{39,7},{38,13},{32,15},{26,11},{4,11},{1,9}},1)
 rect(4,6,24,4,6);line(5,6,27,6,8);line(6,9,25,9,4)
 poly({{27,5},{29,2},{33,2},{30,5},{31,10},{36,12},{38,10},{36,13},{31,13},{27,10}},7)
 px(5,8,2)
end)
asset('breach',40,32,1,100,function()
 poly({{2,12},{7,7},{13,8},{17,3},{23,7},{32,5},{30,12},{38,15},{34,22},{25,25},{19,22},{11,29},{9,23},{2,24},{5,17}},1)
 line(3,11,8,6,8,2);line(10,8,14,10,5);line(18,4,22,8,7);line(30,6,32,11,8)
 line(35,17,32,22,6,2);line(25,24,21,21,4);line(7,20,10,25,7)
 line(17,19,17,25,6);line(17,25,21,27,6);px(22,27,13)
end)

-- Vertical counterpart for the current two-by-six-cell engine bays.
asset('engine-vertical',128,400,4,180,function(f)
 rect(3,17,122,380,1);rect(8,22,112,373,2)
 rect(12,16,104,361,4);rect(16,11,96,5,7);rect(19,8,90,3,8)
 rect(16,377,96,15,3);line(18,378,109,378,6)
 for _,x in ipairs({8,113}) do
  rect(x,20,7,363,1);rect(x+1,20,5,360,6);rect(x+1,21,1,358,8)
  for y=30,370,48 do rect(x-2,y,11,5,2);rect(x-1,y,9,1,7) end
 end
 for section=0,2 do local y=29+section*116
  poly({{23,y+7},{30,y},{96,y},{105,y+9},{105,y+93},{96,y+104},{30,y+104},{23,y+95}},1)
  rect(26,y+9,76,84,5);rect(31,y+3,63,6,6);line(32,y+3,93,y+3,8)
  rect(31,y+17,62,59,2);rect(33,y+19,58,55,3)
  for yy=y+21,y+70,5 do rect(36,yy,52,3,1);rect(36,yy,52,1,7) end
  rect(34,y+82,43,8,1);rect(36,y+84,39,4,10)
  for k=0,7 do rect(37+k*5,y+85,3,2,k<=f+3 and 13 or 11) end
  ellipse(88,y+86,7,7,1);ellipse(88,y+85,5,5,10);line(88,y+85,90,y+82-f%2,14)
  line(32,y+97,93,y+97,3);text('ION / 03',42,y+98,7)
  for _,x in ipairs({27,96}) do rivet(x,y+10);rivet(x,y+89) end
 end
 for x=28,96,8 do rect(x,382,4,7,1);rect(x,382,4,1,5) end
end)

asset('door-depth',64,88,5,100,function(f)
 local phase=f==4 and 0 or f
 -- Full-height lintel, narrow cap and sliding recessed leaves.
 rect(0,28,64,34,1);rect(14,31,36,31,0)
 rect(0,28,64,5,5);line(0,28,63,28,8);line(1,32,62,32,2)
 for _,x in ipairs({0,50}) do
  rect(x,33,14,27,3);rect(x+2,34,10,23,5);rect(x+3,35,2,21,7)
  rect(x+7,39,3,10,10);rect(x+8,40,1,7,13)
 end
 local width=math.floor(18*(1-phase/3))
 if width>0 then
  for _,x in ipairs({14,50-width}) do
   rect(x,34,width,27,1);rect(x+1,35,math.max(0,width-2),24,6)
   if width>5 then rect(x+3,38,width-6,14,2);rect(x+4,39,width-8,11,4) end
   line(x+1,58,x+width-2,58,8)
  end
 end
 if f==4 then line(28,37,36,55,7,2);line(36,37,28,55,7,2) end
end)

-- 16 joining masks x 16 room-outside masks. The cap is twelve pixels
-- thick, shifted 18px outboard, above a full 22px face on every side.
-- Frame pivot is (32,56): the original ground cell centre, not the roof.
asset('wall-depth',64,88,256,100,function(f)
 local mask=f%16;local outside=math.floor(f/16)
 local function bit(n,b) return (n & b)~=0 end
 local l=(bit(outside,1) and 1 or 0)+(bit(outside,4) and 1 or 0)
 local r=(bit(outside,2) and 1 or 0)+(bit(outside,8) and 1 or 0)
 local u=(bit(outside,1) and 1 or 0)+(bit(outside,2) and 1 or 0)
 local d=(bit(outside,4) and 1 or 0)+(bit(outside,8) and 1 or 0)
 local function sign(n) return n==0 and 0 or n>0 and 1 or -1 end
 local cx,cy=32+sign(r-l)*18,32+sign(d-u)*18
 local function solid(x,y)
  return (x>=cx-6 and x<cx+6 and y>=cy-6 and y<cy+6)
   or (bit(mask,1) and x>=cx-6 and x<cx+6 and y<cy)
   or (bit(mask,2) and y>=cy-6 and y<cy+6 and x>=cx)
   or (bit(mask,4) and x>=cx-6 and x<cx+6 and y>=cy)
   or (bit(mask,8) and y>=cy-6 and y<cy+6 and x<cx)
 end
 for y=0,63 do for x=0,63 do if solid(x,y) then
  for h=0,22 do px(x,y+24-h,3) end
 end end end
 for y=0,63 do for x=0,63 do if solid(x,y) then
  local edge=not solid(x,y-1);local seam=not solid(x+1,y) or not solid(x-1,y)
  px(x,y+2,edge and 8 or seam and 2 or 5)
  if x%32==12 and y%16==5 then px(x,y+2,8) end
  if not solid(x,y+1) then
   for h=3,22 do px(x,y+2+h,(x%32==0 or x%32==31) and 1 or h==3 and 4 or h==22 and 1 or 3) end
   if x%32>4 and x%32<27 then px(x,y+8,4);px(x,y+20,2) end
  end
 end end end
end)
local file=assert(io.open(OUT..'manifest.json','w'));file:write(json.encode(manifest));file:close()
print('Exported radar, ship, engine, threaded bolts, tape, breach and production depth kit.')
