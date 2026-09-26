-- Native, palette-indexed Aseprite authoring. No resampling or antialiasing.
-- Run from repository root: sprite-axi run assets/characters/engineer/source/build.lua
local OUT = 'assets/characters/engineer/'
local W,H = 64,64
local PX,PY = 32,38
local CY,SZ = .82,.57236352085
local hex={'000000','101c29','1b2d3e','2c4255','405c70','56778b','7595a5','9ab3bd','bfd0d6','e6eff1','12354f','1c587b','3388ad','61bbd2','a0e0eb','8198a3'}
local pal=Palette(16)
local rgba={}
for i,h in ipairs(hex) do local c=Color{r=tonumber(h:sub(1,2),16),g=tonumber(h:sub(3,4),16),b=tonumber(h:sub(5,6),16),a=i==1 and 0 or 255}; pal:setColor(i-1,c); rgba[i-1]=app.pixelColor.rgba(c.red,c.green,c.blue,c.alpha) end
local dirs={'e','ese','se','sse','s','ssw','sw','wsw','w','wnw','nw','nnw','n','nne','ne','ene'}
local buf,dep,ids,surfaces,parts,fx,fy,rx,ry
local function clamp(v,a,b) return math.max(a,math.min(b,v)) end
local function idx(x,y) return y*W+x+1 end
local function world(x,y,z) return rx*x+fx*y, ry*x+fy*y,z end
local function projected(x,y,z) local a,b=world(x,y,z); return PX+a,PY+CY*b-SZ*z end
local function unproject(x,y,z) local a,b=x-PX,(y-PY+SZ*z)/CY; return a*rx+b*ry,a*fx+b*fy,z end
local function ell(x,y,z,a,b,c,mat,name)
 parts[#parts+1]={x=x,y=y,z=z,a=a,b=b,c=c,mat=mat,name=name or mat}
end
local function capsule(a,b,r,mat,name)
 local dx,dy,dz=b[1]-a[1],b[2]-a[2],b[3]-a[3]
 local n=math.ceil(math.sqrt(dx*dx+dy*dy+dz*dz)/2)
 for i=0,n do local t=i/n; ell(a[1]+t*dx,a[2]+t*dy,a[3]+t*dz,r,r,r,mat,name) end
end
local function shade(p,x,y,z,nz)
 local m=p.mat
 local c
 if m=='armor' or m=='helmet' or m=='shoulder' then
  c=nz>.90 and 8 or nz>.65 and 7 or nz>.3 and 6 or nz>-.05 and 5 or 3
 elseif m=='pack' then c=nz>.6 and 6 or nz>.1 and 4 or 2
 elseif m=='rubber' then c=nz>.65 and 4 or nz>.1 and 3 or 2
 elseif m=='boot' then c=nz>.6 and 5 or nz>.1 and 3 or 2
 elseif m=='glove' then c=nz>.6 and 6 or nz>.1 and 4 or 2
 elseif m=='patch' then c=nz>.6 and 8 or nz>.1 and 15 or 4
 end
 if m=='helmet' then
  -- An inset front visor, with a dark gasket and a restrained cyan reflection.
  if y>.34 and z<.50 and z>-.67 and math.abs(x)<.91 then
   if y<.44 or z>.37 or z<-.56 or math.abs(x)>.82 then return 1 end
   c=z>.20 and 14 or z>-.04 and 13 or z>-.19 and 12 or 11
   if math.abs(x)>.61 then c=z>0 and 12 or 10 end
   if x>-.38 and x<-.19 and z>.08 and z<.29 then c=9 end
   if x>.24 and x<.48 and z>-.27 and z<-.17 then c=10 end
   return c
  end
  -- Crown plate seam, worn paint and a pair of rivets; all suit-local.
  if y<.30 and z>.46 then
   if math.abs(x)<.10 then return z>.82 and 6 or 4 end
   if math.abs(x)>.10 and math.abs(x)<.18 then return z>.78 and 9 or 7 end
   if y>-.37 and y<-.28 and x>.32 and x<.63 then return 4 end
   if y>-.27 and y<-.19 and x>.40 and x<.61 then return 9 end
   if y<-.64 and y>-.73 and x<-.32 and x>-.57 then return 5 end
   if x>-.68 and x<-.37 and y>-.06 and y<.08 then return 3 end
   if x>-.63 and x<-.40 and y>.08 and y<.17 then return 8 end
   if x>.40 and x<.65 and y<-.44 and y>-.56 then return 4 end
   if x>.46 and x<.63 and y<-.36 and y>-.44 then return 9 end
  end
  if math.abs(x)>.85 and y>-.2 and y<.24 and z>-.23 and z<.22 then
   return math.floor((z+.23)*23)%3==0 and 7 or 2
  end
 elseif m=='armor' then
  if y>.65 then
   if z>-.4 and z<-.18 then return 2 end
   if x>-.26 and x<.26 and z>-.06 and z<.32 then
    if x<-.13 or x>.13 or z<.03 or z>.25 then return 2 end
    return z>.15 and 13 or 5
   end
   if math.abs(x)>.4 and math.abs(x)<.51 and z>-.03 and z<.47 then return 3 end
  end
  if z<-.59 then return 2 end
 elseif m=='shoulder' then
  if p.x<0 and z>.45 and y>-.5 and y<.53 then
   if x>-.55 and x<.50 then
    if math.abs(x)>.38 or y<-.35 or y>.38 then return 3 end
    return math.floor((y+.5)*12)%3==0 and 7 or 15
   end
  elseif p.x>0 and z>.6 and y>-.22 and y<.04 then return 3 end
  if z>.72 and x>.05 and x<.50 and y>.32 and y<.50 then return 9 end
 elseif m=='pack' then
  if math.abs(x)>.68 then return nz>.3 and 8 or 3 end
  if z>-.45 and z<.55 and math.floor((z+.45)*18)%4<2 then return 2 end
  if z>.64 and math.abs(x)<.52 then return 7 end
 elseif m=='boot' then
  if z<-.06 then return 1 end
  if y>.56 and z>.25 then return 7 end
  if y>-.4 and y<.25 and z>.75 then return math.floor((y+.4)*14)%3==0 and 2 or 4 end
 elseif m=='rubber' then
  if math.floor((z+1)*10)%3==0 then return 1 end
 elseif m=='glove' then
  if y>.5 and z>.2 then return 7 end
 end
 return c or 5
end
local function raster()
 for id,p in ipairs(parts) do
  local cx,cy=projected(p.x,p.y,p.z)
  local extent=math.ceil(math.max(p.a,p.b,p.c)*1.45+2)
  for sy=math.max(0,math.floor(cy-extent)),math.min(63,math.ceil(cy+extent)) do
   for sx=math.max(0,math.floor(cx-extent)),math.min(63,math.ceil(cx+extent)) do
    local u,v=sx+.5-PX,sy+.5-PY
    local ox=u*rx+v*CY*ry-p.x
    local oy=u*fx+v*CY*fy-p.y
    local oz=-v*SZ-p.z
    local dx,dy,dz=SZ*ry,SZ*fy,CY
    local aa=dx*dx/(p.a*p.a)+dy*dy/(p.b*p.b)+dz*dz/(p.c*p.c)
    local bb=2*(ox*dx/(p.a*p.a)+oy*dy/(p.b*p.b)+oz*dz/(p.c*p.c))
    local cc=ox*ox/(p.a*p.a)+oy*oy/(p.b*p.b)+oz*oz/(p.c*p.c)-1
    local disc=bb*bb-4*aa*cc
    if disc>=0 then
     local t=(-bb+math.sqrt(disc))/(2*aa); local k=idx(sx,sy)
     if t>(dep[k] or -999) then
      local x,y,z=(ox+t*dx)/p.a,(oy+t*dy)/p.b,(oz+t*dz)/p.c
      local nx,ny,nz=x/p.a,y/p.b,z/p.c
      nz=nz/math.sqrt(nx*nx+ny*ny+nz*nz)
      buf[k]=shade(p,x,y,z,nz);dep[k]=t;ids[k]=p.name;surfaces[k]={z=nz}
     end
    end
   end
  end
 end
 -- Thin ink contour and selective internal occlusion seams, no soft alpha.
 local raw={};for k,c in pairs(buf) do raw[k]=c end
 for y=0,63 do for x=0,63 do local k=idx(x,y)
  if raw[k] then
   for _,d in ipairs({{-1,0},{1,0},{0,-1},{0,1}}) do
    local xx,yy=x+d[1],y+d[2];local q=idx(xx,yy)
    if xx<0 or xx>63 or yy<0 or yy>63 or not raw[q] then buf[k]=1;break end
    if ids[k]~=ids[q] and dep[k]>dep[q]+1.0 and dep[k]<dep[q]+7 and (ids[k]=='helmet' or ids[k]=='shoulder' or ids[k]=='pack') then buf[k]=2 end
   end
  end
 end end
end
local function put(x,y,c,depth)
 x,y=math.floor(x+.5),math.floor(y+.5)
 if x>=0 and x<64 and y>=0 and y<64 then local k=idx(x,y); if not depth or depth>=(dep[k] or -999) then buf[k]=c; dep[k]=depth or 999 end end
end
local function screenline(x1,y1,x2,y2,c,width,z)
 local n=math.max(1,math.ceil(math.max(math.abs(x2-x1),math.abs(y2-y1))))
 for i=0,n do local t=i/n;local x,y=x1+(x2-x1)*t,y1+(y2-y1)*t
  for a=-math.floor(width/2),math.ceil(width/2)-1 do for b=-math.floor(width/2),math.ceil(width/2)-1 do
   local d=z and ((y+b-PY+SZ*z)/CY*SZ+CY*z) or nil;put(x+a,y+b,c,d)
  end end
 end
end
local function contact(theta,mx,my)
 local a,b=math.cos(theta),math.sin(theta); local t=999
 if a>.001 then t=math.min(t,(63-mx-PX)/a) elseif a<-.001 then t=math.min(t,(mx-PX)/a) end
 if b>.001 then t=math.min(t,(63-my-PY)/b) elseif b<-.001 then t=math.min(t,(my-PY)/b) end
 return math.floor(PX+a*t+.5),math.floor(PY+b*t+.5),t
end
local function toolRaster(kind,theta,frame,tx,ty)
 local wobble=kind=='wrench' and ({-.13,.12,0})[frame+1] or 0
 local a=theta+wobble;local vx,vy=math.cos(a),math.sin(a);local ux,uy=vy,-vx
 local function at(u,v,c)
  local x,y=tx+vx*u+ux*v,ty+vy*u+uy*v
  local depth=(y-PY+SZ*15)/CY*SZ+CY*15
  put(x,y,c,depth)
 end
 -- Inverse sampling fills every pixel even at the 22.5-degree poses.
 for y=0,63 do for x=0,63 do
  local u=(x-tx)*vx+(y-ty)*vy;local v=(x-tx)*ux+(y-ty)*uy
  local c=nil
  if kind=='wrench' then
   local av=math.abs(v)
   if u>=-20 and u<=-4 and av<=2 then c=(av>1.2 or u< -19) and 1 or (v<0 and 8 or 6) end
   if u>=-18 and u<=-11 and av<1.0 then c=3 end
   if u>=-5 and u<=3 and av<=5 and not(u>0 and av<2.1) then
    if av>4.1 or u< -4.1 or u>2.1 or (u>-.8 and av<3.0) then c=1 else c=v<0 and 8 or 6 end
    if u>1 and av>3 and av<4.1 then c=9 end
   end
  else
   local du=u+5;local r=math.sqrt(du*du+v*v)
   if r<=5.9 then
    if r>4.9 then c=1 elseif r>3.0 then c=v< -1 and 9 or v<1 and 8 or 15 elseif r>2.0 then c=4 else c=1 end
    if r>3 and r<4.8 and du>1.3 and du<2.5 then c=7 end
    if frame==0 and r>3 and v>2 and du<0 then c=6 end
   end
   -- Adhesive tab advances, presses, then tears back. Contact stays fixed.
   if frame<2 and u>=-2 and u<=0 and v>=0 and v<=5+frame*3 then c=(v>4+frame*3 or u< -1.5) and 1 or 8 end
   if frame<2 and u>=-2 and u<=-.5 and v>=4 and v<=6+frame*2 then c=9 end
   if frame==1 and r>3.0 and r<4.9 and v>0 and du<.5 then c=7 end
  end
  if c then local depth=(y-PY+SZ*15)/CY*SZ+CY*15;put(x,y,c,depth) end
 end end
end
local function draw(direction,action,frame)
 local theta=direction*math.pi/8
 local ground=math.atan(math.sin(theta)/CY,math.cos(theta))
 fx,fy=math.cos(ground),math.sin(ground);rx,ry=fy,-fx
 buf,dep,ids,surfaces,parts={},{},{},{},{}
 local walk=action=='walk'
 local strides=walk and ({-3.5,0,3.5,0})[frame+1] or 0
 local tool=action:match('wrench') and 'wrench' or action:match('tape') and 'tape' or nil
 local use=action:match('_use')~=nil
 local phase=use and frame or 2
 local mx,my=0,0
 if tool=='wrench' then
  for _,w in ipairs({-.13,0,.12}) do
   mx=math.max(mx,3*math.abs(math.cos(theta+w))+5*math.abs(math.sin(theta+w))+.5)
   my=math.max(my,3*math.abs(math.sin(theta+w))+5*math.abs(math.cos(theta+w))+.5)
  end
 else mx=6.2-5*math.abs(math.cos(theta));my=6.2-5*math.abs(math.sin(theta)) end
 local tx,ty,reach=contact(theta,mx,my)
 -- Root and torso never translate; stride is carried by the limbs.
 for _,side in ipairs({-1,1}) do
  local st=strides*side
  local lift=walk and ((frame==1 and side==1) or (frame==3 and side==-1)) and 1.5 or 0
  ell(side*7,st*.4,7.5,5.2,5,6.5,'rubber','leg'..side)
  ell(side*7,3+st,3.7+lift,5.8,7.1,3.8,'boot','boot'..side)
 end
 ell(0,-8.2,15,10.1,4.8,10,'pack','pack')
 ell(0,0,12.8,11.8,7.4,9.6,'armor','torso')
 -- Collar, shoulders and contrasting repaired shoulder panel.
 ell(0,0,20,9,7,2.2,'rubber','collar')
 for _,side in ipairs({-1,1}) do
  local swing=walk and -strides*side*.65 or 0
  local shoulder={side*12.2,-.2,15.7}
  if tool and side==1 then
   local back=tool=='wrench' and 15 or 8
   local gx,gy=tx-math.cos(theta)*back,ty-math.sin(theta)*back
   local glx,gly,glz=unproject(gx,gy,13)
   local elbow={shoulder[1]*.5+glx*.5,shoulder[2]*.5+gly*.5,12}
   capsule(shoulder,elbow,3.5,'rubber','arm'..side)
   capsule(elbow,{glx,gly,glz},3.0,'armor','forearm'..side)
   ell(glx,gly,glz,3.7,3.5,3.4,'glove','hand'..side)
  elseif tool=='tape' and use and side==-1 then
   -- The spare hand pulls and presses the tab, then returns to the suit.
   local grasp=({5,8,0})[frame+1]
   local gx,gy=tx-math.cos(theta)*4+math.sin(theta)*grasp,ty-math.sin(theta)*4-math.cos(theta)*grasp
   gx,gy=clamp(gx,5,58),clamp(gy,5,58)
   if frame==2 then gx,gy=projected(side*14,3,8.8) end
   local glx,gly,glz=unproject(gx,gy,12)
   capsule(shoulder,{glx,gly,glz},3.2,'rubber','arm'..side)
   ell(glx,gly,glz,3.8,3.8,3.6,'glove','hand'..side)
  else
   capsule(shoulder,{side*14,2+swing,9.5},3.8,'rubber','arm'..side)
   ell(side*14,3+swing,8.8,4.2,4.4,4.4,'glove','hand'..side)
  end
  ell(shoulder[1],shoulder[2],shoulder[3],5.1,5.9,4.9,'shoulder','shoulder')
 end
 ell(0,-.15,24.5,11.2,9.8,9.2,'helmet','helmet')
 raster()
 if tool then toolRaster(tool,theta,phase,tx,ty) end
 local im=Image(W,H,ColorMode.INDEXED)
 for y=0,63 do for x=0,63 do im:drawPixel(x,y,buf[idx(x,y)] or 0) end end
 return im,tool and {tx,ty} or nil
end
local spr=Sprite(W,H,ColorMode.INDEXED);spr:setPalette(pal);spr.transparentColor=0;spr.layers[1].name='Engineer'
local manifest={name='Derelict engineer',size={64,64},pivot_pixels={PX,PY},palette='engineer-16.gpl',transparent_index=0,direction_convention='0 degrees east; clockwise in image coordinates',frames={},animations={}}
local fnum=0
local function sequence(action,d,n,duration)
 local start=fnum+1;local names={}
 for f=0,n-1 do
  fnum=fnum+1;if fnum>1 then spr:newEmptyFrame() end
  spr.frames[fnum].duration=duration/1000
  local im,tip=draw(d,action,f)
  spr:newCel(spr.layers[1],fnum,im,Point(0,0))
  local name=string.format('engineer_%s_%s_%02d.png',action,dirs[d+1],f)
  im:saveAs{filename=OUT..'png/'..name,palette=pal}
  manifest.frames[#manifest.frames+1]={file='png/'..name,action=action,direction=dirs[d+1],direction_index=d,angle_degrees=d*22.5,frame=f,duration_ms=duration,aseprite_frame=fnum,atlas_index=fnum-1,atlas_rect={(fnum-1)%16*64,math.floor((fnum-1)/16)*64,64,64},contact_pixels=tip}
  names[#names+1]='png/'..name
 end
 manifest.animations[#manifest.animations+1]={name=action..'_'..dirs[d+1],files=names,loop=action=='walk',duration_ms=duration,from_frame=start,to_frame=fnum}
end
local SAMPLE=app.params['sample']=='true'
if SAMPLE then
 for d=0,14,2 do sequence('idle',d,1,120) end
 for d=0,14,2 do sequence('wrench_hold',d,1,120) end
 for d=0,14,2 do sequence('tape_hold',d,1,120) end
else
 for d=0,15 do sequence('idle',d,1,120) end
 for d=0,14,2 do sequence('walk',d,4,110) end
 for _,tool in ipairs({'wrench','tape'}) do
  for d=0,15 do sequence(tool..'_hold',d,1,120) end
  for d=0,14,2 do sequence(tool..'_use',d,3,90) end
 end
end
-- Tag only after all frames exist; Aseprite extends end tags on frame insertion.
for _,a in ipairs(manifest.animations) do local tag=spr:newTag(a.from_frame,a.to_frame);tag.name=a.name end
local slice=spr:newSlice(Rectangle(0,0,64,64));slice.name='engineer_root';slice.pivot=Point(PX,PY)
spr:saveAs(OUT..'engineer.aseprite')
app.command.ExportSpriteSheet{ui=false,type=SpriteSheetType.ROWS,columns=16,textureFilename=OUT..'engineer-atlas.png',dataFilename=OUT..'engineer-atlas.aseprite.json',listTags=true,listSlices=true}
local file=assert(io.open(OUT..'manifest.json','w'));file:write(json.encode(manifest));file:close()
local gpl=assert(io.open(OUT..'engineer-16.gpl','w'));gpl:write('GIMP Palette\nName: Derelict Engineer 16 (index 0 transparent)\nColumns: 8\n# Index 0 is transparent in PNG and Aseprite; GPL stores RGB only.\n')
local names={'Transparent','Outline','Deep seam','Joint shadow','Blue steel shadow','Blue steel','Plate midtone','Plate light','Worn edge','Specular','Visor deep','Visor blue','Visor midtone','Visor light','Visor reflection','Repair fabric'}
for i,h in ipairs(hex) do gpl:write(string.format('%3d %3d %3d\t%s\n',tonumber(h:sub(1,2),16),tonumber(h:sub(3,4),16),tonumber(h:sub(5,6),16),names[i])) end;gpl:close()
local jasc=assert(io.open(OUT..'engineer-16.pal','w'));jasc:write('JASC-PAL\n0100\n16\n');for _,h in ipairs(hex) do jasc:write(string.format('%d %d %d\n',tonumber(h:sub(1,2),16),tonumber(h:sub(3,4),16),tonumber(h:sub(5,6),16))) end;jasc:close()
print('Exported '..fnum..' indexed 64x64 sprites, shared 16-entry palette, source, atlas and contact metadata.')
