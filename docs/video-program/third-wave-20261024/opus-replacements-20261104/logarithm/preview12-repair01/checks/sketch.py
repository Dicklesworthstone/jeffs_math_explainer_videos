import sys; sys.path.insert(0,'.')
from mirror import *
from PIL import Image, ImageDraw
SS=2; U=90*SS
def px(p): return (1280*SS/2+p[0]*U, 720*SS/2-p[1]*U)
COL={0:(150,160,175),1:(70,75,85),2:(90,150,210),3:(90,208,179),4:(155,115,175),5:(120,190,110)}
GOLD=(255,214,90); RED=(235,90,90); INK=(235,235,235)
def faces(d,pose):
    pts=[px(project(turn(*pose,c))) for c in CORNERS]
    for k,(n,idx) in enumerate(FACES):
        n=tuple(map(float,n))
        if dot(turn(*pose,n),W)>1e-9:
            d.polygon([pts[i] for i in idx],fill=COL[k])
            d.line([pts[i] for i in idx]+[pts[idx[0]]],fill=INK,width=2*SS)
def stroke(d,rs,c,w):
    for r in rs: d.line([px(p) for p in r],fill=c,width=int(w*SS),joint='curve')
def mark(d,p,pose,r,c,ring=False):
    if hidden(p,pose): return
    x,y=px(project(p)); R=r*U
    if ring: d.ellipse([x-R,y-R,x+R,y+R],outline=c,width=2*SS)
    else: d.ellipse([x-R,y-R,x+R,y+R],fill=c)
def picture(t,mode):
    im=Image.new('RGB',(620*SS,520*SS),(12,17,20)); 
    full=Image.new('RGB',(1280*SS,720*SS),(12,17,20)); d=ImageDraw.Draw(full)
    pose=(OTHER,TAU*.5*t)
    faces(d,pose)
    TIP=(0.0,2.0,0.0)
    a,b=turn(*pose,TOP),turn(*pose,TIP)
    stem=[tuple(a[i]+(b[i]-a[i])*k/24 for i in range(3)) for k in range(25)]
    if mode=='pin': stroke(d,runs(stem,pose),RED,3.5)
    stroke(d,runs(line(ROD,2.0),pose),GOLD,5)
    stroke(d,runs(line(OTHER,3.0),pose),RED,4)
    if mode=='pin': stroke(d,runs(path(TIP,OTHER,.5,t),pose),RED,3)
    else: stroke(d,[[project(p) for p in path(TOP,OTHER,.5,t)]],(190,90,95),3)
    mark(d,CORNER,pose,.13,GOLD,True); mark(d,turn(ROD,PI,CORNER),pose,.2,GOLD,True)
    stroke(d,runs(path(CORNER,OTHER,.5,t),pose),RED,3)
    mark(d,turn(*pose,CORNER),pose,.09,RED)
    mark(d,a,pose,.07,GOLD)
    if mode=='pin': mark(d,b,pose,.07,RED)
    return full.crop((int((px((-6.0,0))[0])),int(px((0,2.1))[1]),int(px((0.2,0))[0]),int(px((0,-3.0))[1]))).resize((int(6.2*90),int(5.1*90)))
ts=[.25,.5,.75,1.0]
sheet=Image.new('RGB',(4*558,2*459),(40,40,40))
for j,m in enumerate(['pin','xray']):
    for i,t in enumerate(ts):
        sheet.paste(picture(t,m),(i*558,j*459))
sheet.save('sketch_pin_vs_xray.png'); print(sheet.size)
