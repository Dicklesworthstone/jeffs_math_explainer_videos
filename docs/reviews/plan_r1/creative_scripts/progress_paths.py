# Enumerate the 6 interleavings of two "read x; write x+1" programs and count
# those that lose an update; check that the losing ones are exactly the
# staircase paths through the progress-square point (1,1) (both read, neither written).
from itertools import permutations

steps = [("P1", 0), ("P1", 1), ("P2", 0), ("P2", 1)]   # 0 = read, 1 = write
orders = set()
for p in permutations(steps):
    if p.index(("P1", 0)) < p.index(("P1", 1)) and p.index(("P2", 0)) < p.index(("P2", 1)):
        orders.add(p)

lost = through_center = both = 0
for o in sorted(orders):
    x, local, pos, visited = 41, {}, {"P1": 0, "P2": 0}, []
    for who, op in o:
        if op == 0:
            local[who] = x
        else:
            x = local[who] + 1
        pos[who] += 1
        visited.append((pos["P1"], pos["P2"]))
    is_lost = (x == 42)
    is_center = (1, 1) in visited
    lost += is_lost
    through_center += is_center
    both += is_lost and is_center
    print(" ".join(f"{w}{'RW'[op]}" for w, op in o), "-> x =", x, "| through (1,1):", is_center)
print("schedules:", len(orders), "lost updates:", lost, "through (1,1):", through_center, "both:", both)
