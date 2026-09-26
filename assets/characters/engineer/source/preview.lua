-- Presentation graphics use the same indexed palette as the game sprites.
local out='assets/characters/engineer/'
local source=app.open(out..'engineer.aseprite');local palette=source.palettes[1]
local f=assert(io.open(out..'manifest.json'));local manifest=json.decode(f:read('*a'));f:close()
local lookup={};for _,v in ipairs(manifest.frames) do lookup[v.action..'_'..v.direction..'_'..math.floor(v.frame)]=math.floor(v.aseprite_frame) end
local img
local function rect(x,y,w,h,c) for yy=y,y+h-1 do for xx=x,x+w-1 do if xx>=0 and yy>=0 and xx<img.width and yy<img.height then img:drawPixel(xx,yy,c) end end end end
local font={
 A={'01110','10001','10001','11111','10001','10001','10001'},B={'11110','10001','10001','11110','10001','10001','11110'},C={'01111','10000','10000','10000','10000','10000','01111'},D={'11110','10001','10001','10001','10001','10001','11110'},E={'11111','10000','10000','11110','10000','10000','11111'},F={'11111','10000','10000','11110','10000','10000','10000'},G={'01111','10000','10000','10111','10001','10001','01111'},H={'10001','10001','10001','11111','10001','10001','10001'},I={'111','010','010','010','010','010','111'},J={'00111','00010','00010','00010','10010','10010','01100'},K={'10001','10010','10100','11000','10100','10010','10001'},L={'10000','10000','10000','10000','10000','10000','11111'},M={'10001','11011','10101','10101','10001','10001','10001'},N={'10001','11001','10101','10011','10001','10001','10001'},O={'01110','10001','10001','10001','10001','10001','01110'},P={'11110','10001','10001','11110','10000','10000','10000'},Q={'01110','10001','10001','10001','10101','10010','01101'},R={'11110','10001','10001','11110','10100','10010','10001'},S={'01111','10000','10000','01110','00001','00001','11110'},T={'11111','00100','00100','00100','00100','00100','00100'},U={'10001','10001','10001','10001','10001','10001','01110'},V={'10001','10001','10001','10001','10001','01010','00100'},W={'10001','10001','10001','10101','10101','11011','10001'},X={'10001','10001','01010','00100','01010','10001','10001'},Y={'10001','10001','01010','00100','00100','00100','00100'},Z={'11111','00001','00010','00100','01000','10000','11111'},
 ['0']={'01110','10001','10011','10101','11001','10001','01110'},['1']={'010','110','010','010','010','010','111'},['2']={'01110','10001','00001','00010','00100','01000','11111'},['3']={'11110','00001','00001','01110','00001','00001','11110'},['4']={'00010','00110','01010','10010','11111','00010','00010'},['5']={'11111','10000','10000','11110','00001','00001','11110'},['6']={'01110','10000','10000','11110','10001','10001','01110'},['7']={'11111','00001','00010','00100','01000','01000','01000'},['8']={'01110','10001','10001','01110','10001','10001','01110'},['9']={'01110','10001','10001','01111','00001','00001','01110'},['/']={'00001','00001','00010','00100','01000','10000','10000'},['.']={'0','0','0','0','0','1','1'},['-']={'000','000','000','111','000','000','000'},[':']={'0','1','1','0','1','1','0'} }
local function text(s,x,y,scale,c)
 for i=1,#s do local glyph=font[s:sub(i,i)];if glyph then for yy,row in ipairs(glyph) do for xx=1,#row do if row:sub(xx,xx)=='1' then rect(x+(xx-1)*scale,y+(yy-1)*scale,scale,scale,c) end end end;x=x+(#glyph[1]+1)*scale else x=x+3*scale end end
end
local function sprite(action,direction,frame,x,y,scale)
 local cel=source.layers[1]:cel(math.floor(lookup[action..'_'..direction..'_'..frame]));local im=cel.image
 for yy=0,im.height-1 do for xx=0,im.width-1 do local c=im:getPixel(xx,yy);if c~=0 then rect(x+(xx+cel.position.x)*scale,y+(yy+cel.position.y)*scale,scale,scale,c) end end end
end
local function floor(x,y,w,h,light)
 rect(x,y,w,h,light and 8 or 2)
 for xx=x,x+w-1,48 do rect(xx,y,1,h,light and 7 or 1);rect(xx+1,y,1,h,light and 9 or 3) end
 for yy=y,y+h-1,48 do rect(x,yy,w,1,light and 7 or 1);rect(x,yy+1,w,1,light and 9 or 3) end
end
local board=Sprite(960,708,ColorMode.INDEXED);board:setPalette(palette);img=Image(960,708,ColorMode.INDEXED)
rect(0,0,960,708,1)
text('DERELICT / ENGINEER',28,26,3,9)
text('64 X 64  /  16 COLOURS  /  128 FRAMES',30,60,1,7)
text('01 / IDLE',30,104,2,13);text('02 / WRENCH',342,104,2,13);text('03 / TAPE',654,104,2,13)
for k=0,2 do floor(24+k*312,132,288,256,k==1) end
sprite('idle','se',0,40,132,4);sprite('wrench_hold','se',0,352,132,4);sprite('tape_hold','sw',0,664,132,4)
text('NATIVE SIZE / DARK FLOOR',28,420,1,8)
local dirs={'e','se','s','sw','w','nw','n','ne'}
floor(24,444,912,88,false)
for i,d in ipairs(dirs) do sprite('idle',d,0,45+(i-1)*112,446,1);text(d:upper(),69+(i-1)*112,517,1,7) end
text('NATIVE SIZE / LIGHT FLOOR',28,552,1,8)
floor(24,576,912,88,true)
for i,d in ipairs(dirs) do sprite('idle',d,0,45+(i-1)*112,578,1);text(d:upper(),69+(i-1)*112,649,1,2) end
for i=0,15 do rect(28+i*22,681,18,12,i==0 and 2 or i) end
text('FIXED ROOT 32,38 / OVERHEAD LIGHT',420,685,1,7)
board:newCel(board.layers[1],1,img,Point(0,0));img:saveAs{filename=out..'preview/overview.png',palette=palette}
-- Animation proof: all eight movement directions and both tools on both floors.
local anim=Sprite(768,600,ColorMode.INDEXED);anim:setPalette(palette)
for f=0,47 do
 if f>0 then anim:newEmptyFrame() end;anim.frames[f+1].duration=.09
 img=Image(768,600,ColorMode.INDEXED);rect(0,0,768,600,1)
 text('MOTION PROOF / DARK AND LIGHT',20,18,2,9)
 local rows={{'walk',false},{'walk',true},{'wrench_use',false},{'wrench_use',true},{'tape_use',false},{'tape_use',true}}
 for row,v in ipairs(rows) do local y=55+(row-1)*90;floor(0,y,768,86,v[2]);local phase=v[1]=='walk' and math.floor(f*.09/.11)%4 or f%3
  for i,d in ipairs(dirs) do sprite(v[1],d,phase,(i-1)*96+16,y+2,1);text(d:upper(),(i-1)*96+43,y+74,1,v[2] and 2 or 7) end
 end
 anim:newCel(anim.layers[1],f+1,img,Point(0,0))
end
anim:saveCopyAs(out..'preview/motion.gif')
print('Created overview PNG and eight-direction animation proof GIF.')
