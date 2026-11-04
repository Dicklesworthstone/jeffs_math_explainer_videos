#!/usr/bin/env python3
"""Author's exact corner-permutation check; not a visual or independent review."""
from pathlib import Path
import hashlib
import json


# Positions in counterclockwise order: bottom-left, bottom-right, top-right,
# top-left. A permutation maps each original corner index to its new position.
identity = (0, 1, 2, 3)
quarter_turn = (1, 2, 3, 0)
horizontal_reflection = (3, 2, 1, 0)


def compose(left, right):
    """Apply right first, then left, as stated in the video source."""
    return tuple(left[right[i]] for i in range(4))


rotations = [identity]
for _ in range(3):
    rotations.append(compose(quarter_turn, rotations[-1]))
reflections = [compose(rot, horizontal_reflection) for rot in rotations]
assert len(set(rotations + reflections)) == 8
assert compose(quarter_turn, rotations[-1]) == identity

e_r = compose(identity, quarter_turn)
s_r = compose(horizontal_reflection, quarter_turn)
assert e_r == quarter_turn
assert s_r == compose(rotations[3], horizontal_reflection)
h = (identity, horizontal_reflection)
coset_r = {compose(quarter_turn, x) for x in h}
coset_sr = {compose(s_r, x) for x in h}
assert e_r in coset_r and s_r not in coset_r
assert s_r in coset_sr and coset_r.isdisjoint(coset_sr)


def labels_at_positions(permutation):
    return [permutation.index(position) + 1 for position in range(4)]


assert labels_at_positions(e_r) == [4, 1, 2, 3]
assert labels_at_positions(s_r) == [3, 2, 1, 4]

# Check all 64 representative products in the four quotient-table cells.
classes = (set(rotations), set(reflections))
checks = 0
for row, left in enumerate(classes):
    for col, right in enumerate(classes):
        expected = classes[row ^ col]
        for a in left:
            for b in right:
                assert compose(a, b) in expected
                checks += 1
assert checks == 64

root = Path.cwd()
paths = [
    'explainer_corpus/abstract_algebra_visual/src/chapters/quotient.rs',
    'explainer_corpus/abstract_algebra_visual/src/model.rs',
    'explainer_corpus/abstract_algebra_visual/script.tsv',
    'docs/video-program/research-20261016/evidence/abstract-quotient-visual-candidate-20261102.rs',
]
for path in paths:
    print('sha256', path, hashlib.sha256((root / path).read_bytes()).hexdigest())
print('PASS 8 distinct square symmetries; e*r and s*r lie in disjoint H cosets')
print('PASS labeled corners BL,BR,TR,TL: e*r = [4,1,2,3]; s*r = [3,2,1,4]')
print('PASS 64/64 rotation/reflection quotient representative products')
print('LIMIT: mathematical author check; candidate not compiled, rendered, listened to or independently reviewed')
