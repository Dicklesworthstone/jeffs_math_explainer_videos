#!/usr/bin/env python3
"""Scripted, step-by-step scenarios for animation.  Every step is validated by
replay() against the model's transition relation (an illegal step raises)."""
import os as _os, sys as _sys
_sys.path.insert(0, _os.path.dirname(_os.path.abspath(__file__)))  # works under python3 -I from any cwd
from bakery_mc import Bakery, replay, explore

def hdr(s):
    print("\n" + "=" * 100 + "\n" + s + "\n" + "=" * 100)

# (a) no choosing flag: P1 slow writer, P2 sneaks in, P1 wins tie-break, both in CS
hdr("(a) NO choosing[] : classic 2-process mutual-exclusion violation")
A = [(1, "start"), (1, "read number[2]=0"),          # P1 computed ticket 1, not yet written
     (2, "start"), (2, "read number[1]=0"), (2, "number[2]:=1"),
     (2, "L3: number[1]=0, pass"),                     # P2 enters CS
     (1, "number[1]:=1"), (1, "L3: number[2]=1, pass")]  # (1,2) < (1,1)? no -> P1 enters
replay(Bakery(2, 5, "no_choosing", any_order=False), A)

hdr("(a') SAME schedule WITH choosing[] (correct algorithm): P2 is stopped at L2")
A2 = [(1, "choosing:=1"), (1, "read number[2]=0"),
      (2, "choosing:=1"), (2, "read number[1]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),
      (2, "L2: choosing[1]=1, wait"),
      (1, "number[1]:=1"), (1, "choosing:=0"),
      (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),   # P1 enters (tie, lower id)
      (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),   # P2 waits
      (1, "number[1]:=0"), (2, "L3: number[1]=0, pass")]
replay(Bakery(2, 5, "correct", any_order=False), A2)

# (b) compare numbers only
hdr("(b1) numbers only, strict <  : tie -> both enter")
B = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
     (1, "number[1]:=1"), (2, "number[2]:=1"), (1, "choosing:=0"), (2, "choosing:=0"),
     (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),
     (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, pass")]
replay(Bakery(2, 5, "strict_nums", any_order=False), B)

hdr("(b2) numbers only, <= : tie -> both wait forever (deadlock)")
B2 = B[:8] + [(1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, wait"),
              (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),
              (1, "L3: number[2]=1, wait"), (2, "L3: number[1]=1, wait")]
st, _ = replay(Bakery(2, 5, "le_nums", any_order=False), B2)
m = Bakery(2, 5, "le_nums", any_order=False)
print("   enabled moves in final state:", [(f"P{i+1}", lab, ns == st) for i, lab, ns in m.successors(st)])
print("   -> every enabled move is a self-loop (state unchanged): no process can ever progress")

# (c) equal tickets from a non-atomic max+1 (happens in the CORRECT algorithm too)
hdr("(c) two processes take equal tickets (normal in the real algorithm; tie-break needed)")
C = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
     (1, "number[1]:=1"), (2, "number[2]:=1")]
replay(Bakery(2, 5, "correct", any_order=False), C)
hdr("(c') N=3: P3 holds ticket 1; P1 and P2 both read max=1 and both take 2")
C3 = [(3, "choosing:=1"), (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "number[3]:=1"),
      (3, "choosing:=0"),
      (1, "choosing:=1"), (2, "choosing:=1"),
      (1, "read number[2]=0"), (2, "read number[1]=0"),
      (1, "read number[3]=1"), (2, "read number[3]=1"),
      (1, "number[1]:=2"), (2, "number[2]:=2")]
replay(Bakery(3, 5, "correct", any_order=False), C3)

# (d) 4-process walkthrough
hdr("(d) correct algorithm, N=4: tie, choosing-flag save, id tie-break, FCFS, ticket growth")
D = [
    (3, "choosing:=1"), (2, "choosing:=1"),
    (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "read number[4]=0"),
    (2, "read number[1]=0"), (2, "read number[3]=0"),
    (3, "number[3]:=1"), (3, "choosing:=0"),
    (3, "L2: choosing[1]=0, pass"), (3, "L3: number[1]=0, pass"),
    (3, "L2: choosing[2]=1, wait"),                     # the choosing flag holds P3 back
    (2, "read number[4]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),   # tie: both hold 1
    (4, "choosing:=1"), (4, "read number[1]=0"), (4, "read number[2]=1"), (4, "read number[3]=1"),
    (4, "number[4]:=2"), (4, "choosing:=0"),
    (3, "L2: choosing[2]=0, pass"), (3, "L3: number[2]=1, wait"),     # (1,2) < (1,3)
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=1, pass"),     # (1,3) < (1,2)? no
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=2, pass"),     # P2 ENTERS
    (1, "choosing:=1"), (1, "read number[2]=1"), (1, "read number[3]=1"), (1, "read number[4]=2"),
    (1, "number[1]:=3"), (1, "choosing:=0"),
    (2, "number[2]:=0"),                                              # P2 exits
    (3, "L3: number[2]=0, pass"), (3, "L2: choosing[4]=0, pass"),
    (3, "L3: number[4]=2, pass"),                                     # P3 ENTERS
    (4, "L2: choosing[1]=0, pass"), (4, "L3: number[1]=3, pass"),
    (4, "L2: choosing[2]=0, pass"), (4, "L3: number[2]=0, pass"),
    (4, "L2: choosing[3]=0, pass"), (4, "L3: number[3]=1, wait"),
    (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=0, pass"),
    (1, "L2: choosing[3]=0, pass"), (1, "L3: number[3]=1, wait"),
    (3, "number[3]:=0"),                                              # P3 exits
    (4, "L3: number[3]=0, pass"),                                     # P4 ENTERS
    (1, "L3: number[3]=0, pass"), (1, "L2: choosing[4]=0, pass"), (1, "L3: number[4]=2, wait"),
    (2, "choosing:=1"), (2, "read number[1]=3"), (2, "read number[3]=0"), (2, "read number[4]=2"),
    (2, "number[2]:=4"), (2, "choosing:=0"),                          # P2 back: ticket 4
    (4, "number[4]:=0"),                                              # P4 exits
    (1, "L3: number[4]=0, pass"),                                     # P1 ENTERS
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=3, wait"),
    (1, "number[1]:=0"),
    (2, "L3: number[1]=0, pass"), (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=0, pass"),
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=0, pass"),     # P2 ENTERS again
]
replay(Bakery(4, 6, "correct", any_order=True), D)

# (e) safe registers: garbage reads, still safe
hdr("(e1) SAFE registers: P2's max-read overlaps P1's write and returns garbage 7 -> ticket 8; harmless")
E = [(1, "begin choosing:=1"), (1, "end choosing:=1"), (1, "read number[2]=0"),
     (1, "begin number[1]:=1"),
     (2, "begin choosing:=1"), (2, "end choosing:=1"),
     (2, "read number[1]=7"),                     # overlapping read: arbitrary value
     (1, "end number[1]:=1"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
     (2, "begin number[2]:=8"), (2, "end number[2]:=8"), (2, "begin choosing:=0"), (2, "end choosing:=0"),
     (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, pass"),
     (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait")]
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E)
hdr("(e2) SAFE registers: P2's L3 read overlaps P1's exit write (1 -> 0) and returns garbage")
E2 = E + [(1, "begin number[1]:=0"), (2, "L3: number[1]=9, pass"), (1, "end number[1]:=0")]
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E2)
hdr("(e3) SAFE registers: P1 re-enters; P2 (in L3) reads number[1] mid-write as garbage 0 and enters;"
    " P1's ticket is necessarily larger, so P1 waits")
E3 = E + [(1, "begin number[1]:=0"), (1, "end number[1]:=0"),          # P1 exits
          (1, "begin choosing:=1"), (1, "end choosing:=1"),
          (1, "read number[2]=8"), (1, "begin number[1]:=9"),             # P1 re-enters, writing 9
          (2, "L3: number[1]=0, pass"),                                    # garbage 0 -> P2 ENTERS
          (1, "end number[1]:=9"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
          (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, wait")]  # (8,2) < (9,1): P1 waits
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E3)
