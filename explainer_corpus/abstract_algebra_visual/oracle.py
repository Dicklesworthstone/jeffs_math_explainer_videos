#!/usr/bin/env python3
"""Independent corner-permutation check for the square-group prototype."""
points = [(-1,-1), (1,-1), (1,1), (-1,1)]
def rotate(p): x,y=p; return -y,x
def flip(p): x,y=p; return x,-y
def compose(a,b): return tuple(a[i] for i in b)
def permutation(action): return tuple(points.index(action(p)) for p in points)
r, s = permutation(rotate), permutation(flip)
e=tuple(range(4))
rotations=[e]
for _ in range(3): rotations.append(compose(r,rotations[-1]))
group=rotations+[compose(a,s) for a in rotations]
assert len(set(group))==8
print("square.group_order\t"+str(len(group)))
print("square.centralizer_order\t"+str(sum(compose(g,s)==compose(s,g) for g in group)))
print("square.reflection_coset_order\t"+str(len({compose(s,g) for g in rotations})))
print("square.rotation_order\t"+str(len(rotations)))
h=(e,s)
cosets={frozenset(compose(g,x) for x in h) for g in group}
assert len(cosets)==4 and all(len(c)==2 for c in cosets)
print("square.coset_count\t"+str(len(cosets)))
print("square.coset_size\t"+str(len(h)))
modulus=4
values=[1+i*modulus for i in range(3)]
print("clock.modulus\t"+str(modulus))
for i,value in enumerate(values,1): print(f"clock.value.{i}\t{value}")
assert all(value%modulus==values[0]%modulus for value in values)
print("clock.position\t"+str(values[0]%modulus))
classes=[frozenset(rotations),frozenset(compose(s,x) for x in rotations)]
assert classes[0].isdisjoint(classes[1]) and len(classes[0]|classes[1])==8
for row,left in enumerate(classes):
    for column,right in enumerate(classes):
        products={compose(a,b) for a in left for b in right}
        assert products in classes
        print(f"square.quotient.{row}{column}\t"+("R" if products==classes[0] else "F"))
for name, action in [("turn_then_flip", compose(s,r)),("flip_then_turn",compose(r,s))]:
    for i,destination in enumerate(action,1):
        x,y=points[destination]
        print(f"square.{name}.{i}\t{x},{y}")
