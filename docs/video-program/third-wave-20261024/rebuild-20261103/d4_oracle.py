"""Independent exact permutation oracle; no renderer or floating-point matrices."""
import itertools
import json
R = (3, 0, 1, 2)  # destinations of clockwise-listed corners after CCW turn
S = (1, 0, 3, 2)  # horizontal reflection
E = tuple(range(4))
def mul(a, b):
    return tuple(a[b[i]] for i in range(4))
def power(a, n):
    result = E
    for _ in range(n):
        result = mul(a, result)
    return result
G = [power(R, k) for k in range(4)] + [mul(power(R,k), S) for k in range(4)]
assert len(set(G)) == 8
assert all(mul(a,b) in G for a,b in itertools.product(G,repeat=2))
assert all(mul(mul(a,b),c)==mul(a,mul(b,c)) for a,b,c in itertools.product(G,repeat=3))
assert mul(S,R)[0] == 2 and mul(R,S)[0] == 0
H = frozenset(G[:4])
def coset(a, h):
    return frozenset(mul(a,b) for b in h)
normal_classes = {coset(a,H) for a in G}
assert len(normal_classes)==2
assert all(len({coset(mul(a,b),H) for a in A for b in B})==1 for A in normal_classes for B in normal_classes)
K = frozenset([E,S])
classes = {coset(a,K) for a in G}
witness = next((A,B) for A in classes for B in classes if len({coset(mul(a,b),K) for a in A for b in B})>1)
result = {'elements':G, 'table':[[G.index(mul(a,b)) for b in G] for a in G],
          'centralizer_of_flip':[G.index(a) for a in G if mul(a,S)==mul(S,a)],
          'normal_rotation_classes':sorted([sorted(G.index(a) for a in A) for A in normal_classes]),
          'nonnormal_witness_classes':[sorted(G.index(a) for a in A) for A in witness],
          'checks':{'products':64,'associativity_triples':512,'quotient_representative_pairs':64}}
print(json.dumps(result,indent=2))
