# Two processes that never let the bakery empty: tickets climb 1,2,3,4,5,6,...
import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
from bakery_mc import Bakery, replay
S = [(1,"choosing:=1"),(1,"read number[2]=0"),(1,"number[1]:=1"),(1,"choosing:=0"),
     (1,"L2: choosing[2]=0, pass"),(1,"L3: number[2]=0, pass")]           # P1 in CS with 1
t_other = {1: 1, 2: 0}
for r in range(5):
    a, b = (2, 1) if r % 2 == 0 else (1, 2)     # a = process taking the next ticket while b is in CS
    nb = 1 + r                                  # b's ticket
    na = nb + 1
    S += [(a,"choosing:=1"),(a,f"read number[{b}]={nb}"),(a,f"number[{a}]:={na}"),(a,"choosing:=0"),
          (a,f"L2: choosing[{b}]=0, pass"),(a,f"L3: number[{b}]={nb}, wait"),
          (b,f"number[{b}]:=0"),(a,f"L3: number[{b}]=0, pass")]
st, rows = replay(Bakery(2, 7, "correct", any_order=False), S, verbose=False)
print("final state:", rows[-1][1]); print("tickets taken in order: 1," + ",".join(str(2+r) for r in range(5)))
