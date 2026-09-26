-- Rebuild with: sprite-axi run assets/prototypes/hero-directions/build.lua
local out = 'C:/Users/jbxcl/Projects/leave-it-behind/assets/prototypes/hero-directions/'
local W,H,N=160,192,12
local palettes={
 {name='01-rose-rebel',ink='#302431',hair='#65393e',hs='#402a34',hl='#9b5656',top='#df91a2',ts='#af607e',th='#ffc2c2',skirt='#343a59',sl='#616887',skin='#edb39d',ss='#c17d7c',sock='#e4e4de',shoe='#65454d',accent='#6be1be'},
 {name='02-ink-apparition',ink='#191c28',hair='#252733',hs='#191c28',hl='#656775',top='#f4efde',ts='#a3a2a3',th='#fffbee',skirt='#323442',sl='#767782',skin='#eee8d8',ss='#aca8a4',sock='#eee8d8',shoe='#292b37',accent='#ee6967'},
 {name='03-teal-sprinter',ink='#242e43',hair='#383347',hs='#242738',hl='#746071',top='#58b9b1',ts='#337784',th='#9de0c4',skirt='#32364e',sl='#5d6179',skin='#edb99d',ss='#c7837e',sock='#edeada',shoe='#d4d9d5',accent='#ffc373'}
}
local img,ox,oy
local colors={}
local function color(c)
 if not colors[c] then colors[c]=app.pixelColor.rgba(tonumber(c:sub(2,3),16),tonumber(c:sub(4,5),16),tonumber(c:sub(6,7),16),255) end
 return colors[c]
end
local function px(x,y,c) x=math.floor(x+(ox or 0)); y=math.floor(y+(oy or 0)); if x>=0 and y>=0 and x<img.width and y<img.height then img:drawPixel(x,y,color(c)) end end
local function line(x1,y1,x2,y2,c,w)
 local n=math.max(math.abs(x2-x1),math.abs(y2-y1)); for i=0,math.ceil(n) do local t=n==0 and 0 or i/math.ceil(n); for a=-math.floor((w or 1)/2),math.ceil((w or 1)/2)-1 do for b=-math.floor((w or 1)/2),math.ceil((w or 1)/2)-1 do px(x1+(x2-x1)*t+a,y1+(y2-y1)*t+b,c) end end end
end
local function poly(p,c,edge)
 local lo,hi=999,-999; for _,v in ipairs(p) do lo=math.min(lo,v[2]); hi=math.max(hi,v[2]) end
 for y=math.floor(lo),math.ceil(hi) do local xs={}; for i=1,#p do local a,b=p[i],p[i%#p+1]; if (a[2]<=y and b[2]>y) or (b[2]<=y and a[2]>y) then xs[#xs+1]=a[1]+(y-a[2])*(b[1]-a[1])/(b[2]-a[2]) end end; table.sort(xs); for k=1,#xs-1,2 do for x=math.ceil(xs[k]),math.floor(xs[k+1]) do px(x,y,c) end end end
 if edge then for i=1,#p do local a,b=p[i],p[i%#p+1]; line(a[1],a[2],b[1],b[2],edge,2) end end
end
local function ellipse(cx,cy,rx,ry,c)
 for y=-ry,ry do for x=-rx,rx do if x*x/(rx*rx)+y*y/(ry*ry)<=1 then px(cx+x,cy+y,c) end end end
end
local function limb(a,b,c,width,fill,ink)
 line(a[1],a[2],b[1],b[2],ink,width+3); line(b[1],b[2],c[1],c[2],ink,width+2)
 line(a[1],a[2],b[1],b[2],fill,width); line(b[1],b[2],c[1],c[2],fill,width-1)
end
local function hero(p,v,f,layer)
 local t=(f-1)*2*math.pi/N; local s=math.sin(t); local bob=-3*math.abs(math.cos(t)); local sway=1.5*math.sin(t)
 ox=sway; oy=bob
 if layer=='legs' then
  for _,side in ipairs({-1,1}) do
   local q=s*side; local x=80+side*10; local kneeY=133+q*9; local footY=155+q*19; local kneeX=x+side*(3+q*2); local footX=x+side*(5-q*2)
   limb({x,109},{kneeX,kneeY},{footX,footY},10,p.skin,p.ink)
   line(x+side*3,116,kneeX+side*3,kneeY,p.ss,3)
   line(kneeX,kneeY+3,footX,footY,p.ink,12); line(kneeX,kneeY+3,footX,footY,p.sock,9)
   line(kneeX+3,kneeY+5,footX+3,footY-2,p.sl,2)
   if v==3 then line(kneeX-4,kneeY+7,kneeX+4,kneeY+7,p.accent,2) end
   poly({{footX-6,footY-3},{footX+5,footY-3},{footX+8,footY+5},{footX+6,footY+10},{footX-7,footY+9},{footX-8,footY+3}},q<0 and p.ink or p.shoe,p.ink)
   line(footX-5,footY+7,footX+5,footY+7,q<0 and p.sl or p.sock,2)
   if q<0 then line(footX-3,footY,footX+3,footY,p.sl,2) end
  end
 elseif layer=='uniform' then
  for _,side in ipairs({-1,1}) do
   local q=-s*side; local shoulder={80+side*17,66}; local elbow={80+side*(25+q*3),86+q*7}; local hand={80+side*(22+q*5),93+q*12}
   limb(shoulder,elbow,hand,v==3 and 10 or 12,p.top,p.ink)
   line(elbow[1]-side*3,elbow[2],hand[1]-side*3,hand[2]-3,p.ts,3)
   ellipse(hand[1],hand[2]+3,5,6,p.ink); ellipse(hand[1],hand[2]+2,3,4,p.skin)
   line(hand[1]-4,hand[2]-3,hand[1]+4,hand[2]-3,p.th,3)
  end
  poly({{64,62},{76,59},{88,61},{97,68},{95,91},{101,108},{92,114},{67,113},{59,105},{64,86}},p.top,p.ink)
  poly({{86,65},{94,69},{90,88},{96,105},{85,106},{82,93}},p.ts)
  poly({{65,69},{70,65},{73,81},{68,93},{64,86}},p.th)
  line(70,95,75,99,p.ts,2); line(79,92,81,100,p.ts,1)
  local flare=math.cos(t+.6)*3
  poly({{64,104},{94,104},{103+flare,123},{93,125},{86,123},{77,126},{67,123},{57+flare,122}},p.skirt,p.ink)
  for k=0,4 do local x=65+k*6; line(x,109,x-3+flare,120+(k%2)*2,p.sl,2) end
  poly({{61,101},{95,101},{98,108},{90,112},{65,110},{60,107}},p.top,p.ink)
  line(65,107,91,108,p.th,2)
  if v==2 then
   poly({{63,66},{79,74},{94,66},{90,81},{78,87},{66,78}},p.skirt,p.ink)
   line(65,70,78,81,p.sock,1); line(78,81,92,70,p.sock,1)
   for y=88,100,4 do line(87,y,92,y-2,p.ink,1) end
  elseif v==3 then
   poly({{66,64},{80,71},{93,65},{90,75},{79,81},{66,73}},p.ts,p.ink)
   poly({{77,84},{87,84},{86,95},{81,99},{76,94}},p.accent,p.ink)
   line(81,88,82,94,p.ink,2); line(64,96,69,96,p.accent,2)
  end
 elseif layer=='hair-and-accessories' then
  poly({{73,52},{85,51},{87,65},{79,69},{71,64}},p.skin,p.ink)
  local flick=math.sin(t-.7)*4
  if v==2 then
   poly({{83,35},{98,46},{100,71},{111+flick,89},{104+flick,104},{93+flick,98},{94,84},{86,71}},p.hair,p.ink)
   line(96,62,100,85,p.hl,3); line(100,85,104+flick,96,p.hl,2)
  elseif v==3 then
   poly({{86,30},{101,25},{112+flick,37},{107+flick,51},{115+flick,59},{102,57},{96,43},{86,41}},p.hair,p.ink)
   line(99,31,106+flick,39,p.hl,3); line(91,32,96,36,p.accent,4)
  end
  poly({{59,42},{60,30},{67,21},{81,18},{94,23},{101,34},{99,53},{94,64},{84,67},{70,64},{60,57}},p.hair,p.ink)
  poly({{62,38},{66,27},{78,22},{88,23},{75,29},{69,40},{68,53},{63,57}},p.hl)
  poly({{90,30},{98,36},{96,54},{89,62},{77,60},{83,54},{89,46}},p.hs)
  line(68,30,76,25,p.th,1); line(65,40,64,47,p.hl,2)
  line(81,24,91,29,p.hl,2); line(93,33,96,42,p.hl,1)
  line(76,34,71,47,p.hs,2); line(81,38,78,51,p.hs,2)
  if v==1 then
   poly({{62,40},{71,43},{72,62},{67+flick,76},{59+flick,79},{61+flick,65}},p.hair,p.ink)
   line(65,49,65+flick,69,p.hl,2)
   ellipse(97,55,3,5,p.skin); line(98,59,99+flick*.3,69,p.accent,3)
   line(99,61,99,65,p.th,1)
  elseif v==2 then
   for k=0,4 do line(68+k*4,28,64+k*4,39,p.sock,1) end
   line(58,40,66,43,p.accent,3); line(58,44,65,47,p.accent,2)
  else
   poly({{60,44},{68,49},{66,61},{61,64},{58,54}},p.hair,p.ink)
   line(96,49,100,51,p.accent,3); ellipse(97,57,2,3,p.accent)
  end
  -- Crown flick and separated strands carry the manga silhouette.
  line(75,21,79,15,p.ink,2); line(79,15,87,16,p.ink,2)
 end
 ox=0; oy=0
end
local sprites={}
for v,p in ipairs(palettes) do
 local spr=Sprite(W,H,ColorMode.RGB); sprites[v]=spr
 spr.layers[1].name='legs'; local ls={spr.layers[1]}; for _,name in ipairs({'uniform','hair-and-accessories'}) do local l=spr:newLayer(); l.name=name; ls[#ls+1]=l end
 for f=1,N do if f>1 then spr:newEmptyFrame() end; spr.frames[f].duration=.08
  for _,l in ipairs(ls) do img=Image(W,H,ColorMode.RGB); hero(p,v,f,l.name); spr:newCel(l,f,img,Point(0,0)) end
 end
 local tag=spr:newTag(1,N); tag.name='run-away'; spr:saveAs(out..p.name..'.aseprite')
 app.command.ExportSpriteSheet{ui=false,type=SpriteSheetType.HORIZONTAL,textureFilename=out..p.name..'-sheet.png',dataFilename=out..p.name..'-sheet.json',listLayers=true,listTags=true}
 spr:saveCopyAs(out..p.name..'.gif')
end
local function hall(f,v)
 local bg=v==2 and '#ede9df' or '#dde4e2'; local wall=v==2 and '#d2d0c9' or '#9caeb6'; local edge='#697a8b'
 poly({{0,0},{279,0},{279,359},{0,359}},bg)
 poly({{0,0},{111,100},{111,155},{0,300}},wall)
 poly({{279,0},{169,100},{169,155},{279,300}},wall)
 poly({{0,300},{111,155},{169,155},{279,300},{279,359},{0,359}},v==2 and '#c4c3be' or '#aeb7c1')
 poly({{111,100},{169,100},{169,155},{111,155}},'#7d8e9d',edge)
 poly({{124,113},{155,113},{155,155},{124,155}},'#cbdcdb',edge)
 line(140,115,140,153,edge,1)
 for _,x in ipairs({-110,40,240,390}) do line(140,155,x,359,edge,1) end
 for k=0,8 do local d=((k+(f-1)/N)/9)^2; local y=155+d*240; if y<360 then line(111-d*190,y,169+d*190,y,'#8796a4',1) end end
 for _,d in ipairs({.20,.43,.76,1.13}) do
  local x=111-125*d; local top=100-85*d; local bottom=155+143*d
  poly({{x,top},{x+20*d,top+15*d},{x+20*d,bottom-27*d},{x,bottom}},'#c5d9da',edge)
  line(x+4*d,top+15*d,x+16*d,top+24*d,'#f4eee1',2)
  local r=169+125*d
  poly({{r-23*d,top+16*d},{r,top},{r,bottom},{r-23*d,bottom-28*d}},'#748998',edge)
  line(r-17*d,top+39*d,r-5*d,top+33*d,'#b4c9cc',2)
  line(r-17*d,top+46*d,r-5*d,top+40*d,'#b4c9cc',2)
 end
 for _,y in ipairs({15,61,87}) do local size=(103-y)*.45; poly({{140-size,y},{140+size,y},{140+size*.78,y+5},{140-size*.78,y+5}},'#fff6d9') end
 ellipse(140,325,33,7,'#788393')
end
local preview=Sprite(840,360,ColorMode.RGB); preview.layers[1].name='school-camera-comparison'
for f=1,N do
 if f>1 then preview:newEmptyFrame() end; preview.frames[f].duration=.08
 local canvas=Image(840,360,ColorMode.RGB)
 for v,p in ipairs(palettes) do
  img=Image(280,360,ColorMode.RGB); ox=0;oy=0;hall(f,v)
  for _,l in ipairs(sprites[v].layers) do img:drawImage(l:cel(f).image,Point(60,146)) end
  canvas:drawImage(img,Point((v-1)*280,0))
 end
 preview:newCel(preview.layers[1],f,canvas,Point(0,0))
end
preview:saveAs(out..'school-run-comparison.aseprite'); preview:saveCopyAs(out..'school-run-comparison.gif')
preview.cels[1].image:saveAs(out..'school-run-comparison.png')
print('Created 3 layered 12-frame runners, sheets, metadata and corridor comparison.')
