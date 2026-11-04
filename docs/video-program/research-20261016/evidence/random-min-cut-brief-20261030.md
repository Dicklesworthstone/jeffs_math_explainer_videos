# Random contraction and minimum cuts: candidate brief

Status: **research draft, not a selected or approved production brief**. 2026-10-30. `random_min_cut` is an inbox idea in the catalog. The most relevant complete treatments have not been viewed; independent proof/pedagogy review, slate comparison and owner choice remain open.

**Proposed hook.** Ask where a network is easiest to split, then seemingly destroy information by choosing a connection at random and merging its endpoints. Repeat until only two supernodes remain. The remaining connections form a valid cut; repeat independently and keep the smallest cut found. The surprise is a lower bound on the chance that an arbitrarily chosen minimum cut survives, not a promise that a single run always wins. A clear potential payoff is a compact physical explanation of how degree counting turns dangerous random choices into a provable success probability.

**Model.** A connected undirected multigraph with n ≥ 2 vertices, no self-loops and positive integer edge multiplicities; choose each remaining edge uniformly, contract it, discard self-loops and keep parallel edges. Let λ be the global minimum cut size. Any current supernode has degree at least λ because it represents a nonempty proper original vertex set. While a chosen minimum cut survives at r supernodes, the graph has at least rλ/2 edges, so the next contraction hits one of its λ edges with probability at most 2/r. The survival probability is therefore at least the telescoping product from r=n down to 3, equal to 2/[n(n−1)]. Independent fresh runs miss that *fixed* cut with probability at most (1−2/[n(n−1)])^k. Taking the smallest returned cut cannot falsely beat the true minimum, but a finite set of failures to find the chosen cut is not proof of optimality.

The original author describes contraction and the Ω(n⁻²) result in [Global Min-cuts in RNC, and Other Ramifications of a Simple Min-Cut Algorithm](https://people.csail.mit.edu/karger/Papers/mincut.pdf), §2, Theorem 2.1; [Karger and Stein](https://people.csail.mit.edu/karger/Papers/contract.pdf) discuss faster recursive methods. No claim is made that this elementary repeated algorithm is the fastest known min-cut method or that the graph models every real network. Distinguish global cuts from a specified source/sink cut.

**Exact visual check.** In a four-cycle, enumerate the four equally likely first edges and the three equally likely remaining edges. The 12 equally likely two-step paths return each of the six distinct minimum cuts exactly twice. Thus any *fixed* minimum cut occurs with probability 1/6, attaining 2/[4·3]. Every cut returned on this special graph is minimum: the probability of finding *some* minimum is 1, not 1/6. Do not conflate the two events. A second graph consists of two triangles linked by one bridge. The bridge is the unique minimum cut of size one. Contracting that bridge first has probability 1/7 and makes that cut impossible to return. A separate exact recursive enumeration of all contraction choices gives output cut size one with probability 13/35 and size two with probability 22/35. Those values are specific to this graph; they illustrate failure without weakening the general bound. An independent proof and implementation check remain open.

**Competition and kill condition.** A 2026-10-30 YouTube search produced Paul Learns Things' [procedure](https://www.youtube.com/watch?v=KqMGeNZuwfI) (13:27), [single-run analysis](https://www.youtube.com/watch?v=I-tcUxkPxgI) (27 minutes) and [multiple-run analysis](https://www.youtube.com/watch?v=HH0sl8nQoOI) (13:51), plus a [1:19 animated principle](https://www.youtube.com/watch?v=MMm9PZe1VdA). Their search snippets describe a direct overlap with procedure and probability; these are leads, not complete viewing. Park unless a complete comparison establishes a substantial distinct treatment and the two-triangle example supports the story after independent checking. No novelty, listening or production approval is claimed.

The exact six-vertex calculation can be reproduced with standard-library Python; each parallel edge is retained as a separate list entry:

```python
from fractions import Fraction as F
from collections import defaultdict

edges = [(0,1), (1,2), (2,0), (3,4), (4,5), (5,3), (2,3)]
out = defaultdict(F)
def enumerate_runs(parts, probability):
    remaining = [(a,b) for a,b in edges if next(i for i,s in enumerate(parts) if a in s) != next(i for i,s in enumerate(parts) if b in s)]
    if len(parts) == 2:
        out[len(remaining)] += probability
        return
    for a,b in remaining:
        ia = next(i for i,s in enumerate(parts) if a in s)
        ib = next(i for i,s in enumerate(parts) if b in s)
        nxt = [s for i,s in enumerate(parts) if i not in (ia,ib)] + [parts[ia] | parts[ib]]
        enumerate_runs(nxt, probability / len(remaining))
enumerate_runs([{v} for v in range(6)], F(1))
print(dict(sorted(out.items())))  # {1: Fraction(13, 35), 2: Fraction(22, 35)}
```

A second calculation uses states (a,b), the remaining supernode counts in each triangle while the bridge survives. Internal edge counts for 3, 2 and 1 supernodes are 3, 2 and 0. With S(1,1)=1, the recurrence S(a,b)=[e(a)S(a−1,b)+e(b)S(a,b−1)]/[1+e(a)+e(b)] gives S(3,3)=13/35. This cross-check uses the same modeled graph and does not replace an independent review.

For a possible on-screen worked example, the recurrence gives S(1,2)=2/3, S(1,3)=1/2, S(2,2)=8/15, S(2,3)=13/30 and S(3,3)=13/35 (interchanging the sides leaves each value unchanged). The extra 1 in each denominator is the surviving bridge: choosing it ends that path in failure. For this six-vertex graph the general fixed-cut bound is only 1/15; the exact probability 13/35 is larger. These values are conditional on reaching the named state with the bridge intact, except S(3,3), which is the initial success probability. A local exact-fraction calculation reproduced the values; it is not an independent proof or evidence of viewer understanding.

**Bounded competitor observation, 2026-10-30:** On [François Schwarzentruber’s 1:18 animation](https://www.youtube.com/watch?v=MMm9PZe1VdA), an early sampled picture states that the algorithm repeatedly chooses an edge randomly and merges its endpoints; a later sampled picture near 0:52 shows two clustered supernodes joined by parallel edges. The description identifies it as an animation of the algorithm. YouTube’s exported auto-generated Arabic transcript contained music indicators and isolated letters rather than usable speech transcription. Only those sampled pictures and the description were inspected: this is not a continuous viewing, proof of the full content, or evidence that a proposed treatment is novel. Full competitor comparison remains open.

**Competing proof overlap, 2026-10-30:** The English auto-generated transcript of [Paul Learns Things, “Karger’s Algorithm: Single-run Analysis”](https://www.youtube.com/watch?v=I-tcUxkPxgI) explicitly discusses selecting one fixed minimum cut at 0:32–1:23, the minimum-degree and handshake argument at 11:21–14:52, and the product leading to the reciprocal of n choose 2 at 16:37–26:52. This is direct overlap with the proposed proof, not evidence of novelty. The transcript occasionally renders mathematical names and symbols incorrectly and its phrasing near the conclusion may blur an upper/lower bound with equality; the recording and displayed equations have not been checked, so no error verdict is made. Targeted transcript passages were inspected, not the full video or audio; independent proof, complete competing-treatment viewing and owner selection remain open. The uncorrected exported transcript is retained as `mincut-single-run-transcript-auto-20261030.txt` (SHA-256 `d0d8f0c21f1e2684edad9a35201b2795d63bfff1742a203d3cd073231bf6b35e`).

**Primary-source overlap, 2026-10-31:** Karger's paper, immediately after Theorem 2.1 on PDF page 3, already uses an n-vertex cycle to show the fixed-cut bound is tight and states that all its minimum cuts occur with equal probability. The four-cycle calculation above specializes that published example; it is not a newly discovered construction. The distinction between a specified cut and finding any minimum cut follows from that example, but a clear presentation of that distinction alone has not been shown to distinguish a complete video. Keep this candidate provisional and look for a stronger payoff before selecting it; the two-triangle computation and all competing-video comparisons still need independent review and complete viewing.
