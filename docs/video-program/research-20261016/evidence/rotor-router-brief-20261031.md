# Can a switch calculate a probability?

**Status: candidate, not a selected slate or production authorization.** Consumer: Jeff, deciding what to develop next. This is the existing `rotor_router` idea (`mx-video-program-program-87do.19`), not a new video identity. The potential payoff is a compact sequel to the channel's [random-walk/voltage video](https://youtu.be/A0Nu7LhJYlw): use the *same kind of hitting question*, replace the random choices with alternating switches, and show precisely why the exit proportion approaches the random-walk answer. Proposed distinction: a persistent-memory failure case and a two-switch, whole-proof example. This is an editorial hypothesis, not an established coverage gap; the closest lecture already contains much of the core story.

**Inspectable research sketch:** [Two-switch interactive](rotor-router-interactive-20261031.html) compares persistent switches with resetting them, exposes the most recent path and counts, and presents the small-graph proof. Browser checks exercised two individual tokens, a 30-token batch, the proof disclosure and restart. This sketch is not a production scene, audience-comprehension test, independent mathematical review, listening review or video approval.

## The object and the surprise

Draw four points in a line, `L — A — B — R`. L and R are exits; every token starts at A. At A an arrow alternates between sending successive visits to L and B. At B another arrow alternates between sending successive visits to R and A. **Keep both arrow states between tokens.** Each token moves until it reaches an exit; arrows change after each departure. Show three tokens individually, then count a larger batch.

With next exits initially A→L and B→R, the first six exits are `L, R, L, L, R, L`. With fair independent coin flips at each visit instead, let `h(A)` and `h(B)` be the chance of reaching L before R. Then `h(A)=(1+h(B))/2` and `h(B)=h(A)/2`, so `h(A)=2/3`. The deterministic system also sends a long-run fraction 2/3 to L, despite using no random choices. One token's outcome is not a probability and deterministic paths are not distributed like independent random paths.

**A sharp failure case.** Reset both arrows before *every* token to A→L, B→R. Every token exits L, so the fraction is 1, not 2/3. The film should make the state carried between tokens visible: local alternation is essential to this argument. This reset example is a teaching proposal, not a claim that the model or comparison is new.

## An elementary proof that fits the picture

For any initial positions of these two switches and any positive number N of completed tokens, let L and R also denote the counts at the respective exits. Let x be the number of A→B departures and y the number of B→A departures. Each arrow alternates, so `|L−x| ≤ 1` and `|R−y| ≤ 1`. Flow at B gives `x=R+y`, while `N=L+R`. Therefore

`3L−2N = L−2R = (L−x)+(y−R)`,

and the triangle inequality gives **`|L/N−2/3| ≤ 2/(3N)`**. This proof is for the pictured graph and persistent switches, with no averaging or luck required. Each token does reach an exit: if it goes A→B→A, the A switch has already flipped and sends it to L on its next departure. The bound is attained for N=1 when the initial arrows are A→B and B→R; it does not assert that every prefix attains the bound.

The random counterpart is a comparison of **independent complete walks**, not a claim about each deterministic token: the left-exit count has expectation `2N/3` and variance `2N/9`. A random batch can land exactly on the expected count or do better than a particular rotor batch; the bounded worst-case error above is the specific comparison, not a claim that determinism always wins or that this is a practical fast voltage solver.

**Check performed, not independent review:** a standard-library simulation ran 1,000 tokens from each of the four initial switch configurations; all 4,000 prefixes respected `|3L−2N|≤2`. At 1,000 tokens the left counts were 667, 667, 666, 667. The proof above covers all N if correct; the finite simulation alone does not. A mathematically independent reviewer and a fresh reader still need to check the proof and whether the visual makes the distinction clear.

Reproduce the bounded check with `python3 -I`:

```python
from itertools import product

for initial in product(range(2), repeat=2):
    switches = list(initial)  # index of the next exit; A then B
    left = 0
    for n in range(1, 1001):
        vertex = 'A'
        steps = 0
        while vertex in ('A', 'B'):
            index = 0 if vertex == 'A' else 1
            exits = ('L', 'B') if index == 0 else ('R', 'A')
            vertex = exits[switches[index]]
            switches[index] ^= 1
            steps += 1
            assert steps <= 3
        left += vertex == 'L'
        assert abs(3 * left - 2 * n) <= 2
    print(initial, left, 3 * left - 2000)
```

**Generalization boundary.** Holroyd and Propp's [Rotor Walks and Markov Chains](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/rr.pdf), Theorem 1 and its discussion on PDF pages 2–4, analyzes hitting frequencies with an explicit discrepancy bound under stated hypotheses. Finite-state conditions and whether the rotor walk actually keeps hitting the target set matter; the paper discusses escape and infinite-state limitations. The two-switch proof does not by itself prove the general theorem or all random-walk statistics.

## Existing treatments and honest comparison

- **Direct competing treatment:** [James Propp, *Random walk and random aggregation, derandomized*](https://www.youtube.com/watch?v=KS7lSF5R4Qw), Microsoft Research, 54:36. YouTube's auto-generated English transcript at 1:42–8:18 presents a random hitting question, alternating arrows, hitting proportions, and an O(1/N) type comparison; at 9:42–13:00 it describes an invariant proof. At 19:09–20:33 it describes a related two-exit 2/3-versus-1/3 **chip-firing** example, explicitly distinct from rotors. The transcript can misrecognize symbols and words: the exact diagram and details require viewing. One frame near 19:33 was inspected but the graph was obscured. Only these transcript passages and that one picture were inspected; the full talk and audio were **not** reviewed. This is substantial overlap, not evidence the proposed film is novel.
- **Other leads:** a YouTube search for `"rotor-router" mathematics` on 2026-10-31 surfaced [an 18-minute research-math treatment](https://www.youtube.com/watch?v=N9bgy9ENcPo) whose listing mentions chip-firing and sandpile groups, and a [1:53 implementation visualization](https://www.youtube.com/watch?v=M9K7YO6u8p0). Their titles and snippets only were inspected, not their full content. [Canary and Wong's rotor-router applet and explanation](https://faculty.uml.edu/jpropp/rotor-router-model/) already show a persistent-arrow system with sinks and a source. Search results can be incomplete and were polluted by plumbing results; they cannot establish a lack of other treatments.

**Decision rule.** Keep as a candidate only if a complete comparison shows that the tiny continuing example, clear deterministic flow proof and reset counterexample make a material contribution for this channel's audience, and a fresh reader can explain why persistence matters. If the existing treatment already achieves that payoff clearly, park or reframe it. Still required: full competing-treatment viewing, independent math and pedagogical review, owner selection, and separate narration listening/fine-sync acceptance for any eventual cut. No synthesis, render, publication or acceptance is claimed here.
