from fractions import Fraction as F
from math import exp, log, gcd
def clmul(a,b):
    r=0
    while b:
        if b&1: r^=a
        a<<=1; b>>=1
    return r
def mod(r,p):
    dp=p.bit_length()-1
    while r.bit_length()-1>=dp:
        r^=p<<(r.bit_length()-1-dp)
    return r
def mul(a,b,p=0x11D): return mod(clmul(a,b),p)
for (a,b) in [(0x57,0x83),(3,7),(2,128),(128,3),(150,3),(7,7),(2,142),(3,244),(3,5)]:
    print(a,b,"0x11D:",mul(a,b),hex(mul(a,b)),"0x11B:",mul(a,b,0x11B),hex(mul(a,b,0x11B)))
# logs
exp_t=[1]
for i in range(1,510): exp_t.append(mul(exp_t[-1],2))
print("OCT_EXP[0..12]",exp_t[:13], "distinct in 255:",len(set(exp_t[:255])))
lg={exp_t[i]:i for i in range(255)}
print("log3",lg[3],"log29",lg[29],"log128",lg[128],"log150",lg[150],"exp205",exp_t[205],"(180+25)",(lg[150]+lg[3])%255)
def order(a,p):
    x=a;o=1
    while x!=1: x=mul(x,a,p);o+=1
    return o
print("ord2 0x11B",order(2,0x11B),"ord3 0x11B",order(3,0x11B),"ord2 0x11D",order(2,0x11D))
# irreducible/primitive counts
def irreducible(p):
    for q in range(2,1<<4+1):
        if q.bit_length()-1>=1 and q.bit_length()-1<=4 and mod(p,q)==0: return False
    return True
irr=[p for p in range(256,512) if irreducible(p)]
prim=[p for p in irr if order(2,p)==255]
print("#irr",len(irr),"#prim",len(prim), 0x11D in prim, 0x11B in irr, 0x11B in prim)
# Z/256
print("nonzero nonunits",sum(1 for a in range(1,256) if gcd(a,256)>1),"row128 vals",len({128*b%256 for b in range(256)}),"row2",len({2*b%256 for b in range(256)}))
# rank products
def prod(q,start,end):
    p=1.0
    for i in range(start,end+1): p*=1-q**-i
    return p
for q in (2,256):
    print("q",q,[prod(q,1,K) for K in (1,2,3,10,60)], "fail h=0..3",[1-prod(q,h+1,400) for h in range(4)])
print("EB",sum(1/(2**i-1) for i in range(1,80)),"q256",sum(1/(256**i-1) for i in range(1,10)))
# coupon collector exact
K=10000
H=sum(F(1,i) for i in range(1,K+1))
KH=K*H
print("K*H_K =",float(KH),"ceil",-(-KH.numerator//KH.denominator),"floor",KH.numerator//KH.denominator)
print("0.99^1e6 = e^",1e6*log(0.99))
print("e^-4.8166",exp(-4.8166),"50000^3",50000**3,"K^3/3",50000**3/3)
# pentagon: E+2A cycle; holonomy
# equations: A+B, B+C, C+D, D+E, E+2A with A..E=5,3,7,2,12
A,B,C,D,E=5,3,7,2,12
ys=[A^B,B^C,C^D,D^E,E^mul(2,A)]
S=ys[0]^ys[1]^ys[2]^ys[3]
print("pentagon ys",ys,"S",S,"S^y5",S^ys[4],"1^2=",1^2,"3*5=",mul(3,5),"inv3",[x for x in range(256) if mul(3,x)==1])
# holonomy: walk A->B->C->D->E->A' with A' = (E + y5)/2: map A -> (A + c)/2 ; factor 1/2 = 142
inv2=[x for x in range(256) if mul(2,x)==1][0]; print("holonomy factor 2^-1 =",inv2)
# XOR "exactly one" for 3 operands
print("1^1^1 =",1^1^1)
