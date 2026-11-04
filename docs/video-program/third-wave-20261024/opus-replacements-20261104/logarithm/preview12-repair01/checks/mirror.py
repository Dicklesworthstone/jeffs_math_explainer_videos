"""Python mirror of chapter 12's cube geometry: turn, project, hidden, unhidden_runs.
Scratch only: settles F1's design and the Rust tests' thresholds. Not the film."""
import math
PI, TAU = math.pi, math.tau
CUBE_C = (-3.0, -0.55); CUBE_HALF = 1.05
YAW, PITCH = math.radians(35), math.radians(20)
sy, cy, sp, cp = math.sin(YAW), math.cos(YAW), math.sin(PITCH), math.cos(PITCH)
ROD = (0.0, 1.0, 0.0)
s2 = math.sqrt(2)
OTHER = (-1 / s2, 1 / s2, 0.0)
CORNER = (1.0, 1.0, 1.0); TOP = (0.0, 1.0, 0.0)
CORNERS = [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]
CORNERS = [tuple(map(float,c)) for c in CORNERS]
FACES = [((0,1,0),[3,2,6,7]),((0,-1,0),[0,1,5,4]),((0,0,1),[4,5,6,7]),((0,0,-1),[0,1,2,3]),((1,0,0),[1,2,6,5]),((-1,0,0),[0,3,7,4])]
def cross(a,b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def dot(a,b): return a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
def turn(n,ang,p):
    s,c=math.sin(ang),math.cos(ang); k=cross(n,p); al=dot(n,p)*(1-c)
    return tuple(p[i]*c+k[i]*s+n[i]*al for i in range(3))
def project(p):
    x,z=p[0]*cy+p[2]*sy,-p[0]*sy+p[2]*cy
    y=p[1]*cp-z*sp
    return (CUBE_C[0]+CUBE_HALF*x,CUBE_C[1]+CUBE_HALF*y)
W=(-sy*cp,sp,cy*cp)
def hidden(p,pose):
    a,ang=pose
    q=turn(a,-ang,p); d=turn(a,-ang,W)
    lo,hi=0.0,math.inf
    for qi,di in zip(q,d):
        if abs(di)<1e-12:
            if abs(qi)>1: return False
        else:
            x,y=(-1-qi)/di,(1-qi)/di
            lo=max(lo,min(x,y)); hi=min(hi,max(x,y))
    return hi>lo+1e-9 and hi>1e-6
def runs(pts,pose):
    out=[[]]
    for a,b in zip(pts,pts[1:]):
        mid=tuple((a[i]+b[i])/2 for i in range(3))
        if hidden(mid,pose):
            if out[-1]: out.append([])
        else:
            if not out[-1]: out[-1].append(project(a))
            out[-1].append(project(b))
    return [r for r in out if len(r)>=2]
def length(rs): return sum(math.dist(a,b) for r in rs for a,b in zip(r,r[1:]))
def path(p,axis,rate,now):
    laps=abs(rate*now)
    if laps<1e-6: return []
    n=max(math.ceil(laps*96),2)
    return [turn(axis,TAU*rate*now*k/n,p) for k in range(n+1)]
def line(a,reach,n=80):
    return [tuple(a[i]*reach*(k/(n/2)-1) for i in range(3)) for k in range(n+1)]
