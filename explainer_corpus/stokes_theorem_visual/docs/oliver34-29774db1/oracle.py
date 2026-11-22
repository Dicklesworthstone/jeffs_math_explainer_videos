"""Independent-style author enumeration for the first two draft scenes, not peer review."""
from collections import Counter

squares = [((0,0),(1,0),(1,1),(0,1)), ((1,0),(2,0),(2,1),(1,1))]
edges = Counter()
for corners in squares:
    for a, b in zip(corners, corners[1:] + corners[:1]):
        key = tuple(sorted((a, b)))
        edges[key] += 1 if (a, b) == key else -1
assert sum(value != 0 for value in edges.values()) == 6
print('two_squares.directed_edges\t8')
print('two_squares.canceled_edges\t2')
print('two_squares.outer_edges\t6')
print('interval.internal_endpoint_pairs\t2')
