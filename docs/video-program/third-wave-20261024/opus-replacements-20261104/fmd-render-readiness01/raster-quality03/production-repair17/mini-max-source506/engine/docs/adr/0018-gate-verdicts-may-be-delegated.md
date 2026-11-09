# ADR-0018 — Gate verdicts may be delegated; the packet still is the pass

**Status:** Accepted
**Date:** 2026-08-20
**Bead:** fm-o3j (G1 Core 2D; the pending `gradient_fills` Look Gallery verdict)
**Amends:** policy under GOVERNANCE.md §2 (gate ownership); does not change D-01…D-24

## Context

GOVERNANCE.md §2 named Jeffrey Emanuel as the owner of record for every gate
verdict. G1's mechanical packet was complete on 2026-07-29: twenty-one
dependency beads closed, the 25-scene primitive corpus bit-locked at
`{1,4,16}`, native PNG/y4m green, and four Look Gallery panels already
settled. The only remaining action was a human side-by-side of
`gradient_fills` — FrankenManim's specified mean-value colour field versus
the Reference's triangulation seam, already Behavior-Noted in BN-06 fill.

That single visual review then blocked G2, G3, and everything downstream
for three weeks. The program owner is not a merge function. Forcing every
Gallery row and every later gate through one pair of eyes recreates the
bottleneck the swarm exists to avoid, and it is the same class of process
failure as treating an `inconclusive` performance catalog as a product
hold.

The packet discipline is still load-bearing. A chat summary is not a pass.
What has to move is *who is allowed to write the verdict into the packet*.

## Decision

1. The program owner of record remains Jeffrey Emanuel. He may **delegate**
   a named gate verdict to a reviewing agent in session (or standing). The
   delegate's identity, model, date, and the exact evidence viewed are
   recorded in the gate packet and in the bead close reason.
2. A gate still passes only against its in-repo evidence packet
   (GOVERNANCE.md §2). Mechanical rows stay mechanical. Look Gallery rows
   still use the vocabulary `at-least-as-good` / `different-but-fine
   (Behavior-Noted)` / `regression`. A `regression` still holds the gate
   that introduced it (R2).
3. Delegated visual review must inspect the actual panels (FrankenManim
   render and, when the fixture is present, the Reference capture), not
   hashes or marshal prose alone.
4. This ADR does not waive PG-1…PG-8. A performance row that is
   `inconclusive` for lack of a pinned-host baseline is an evidence gap
   owned by `fm-inr*`, not a hidden HOLD on an otherwise complete product
   gate. How G2 treats PG-1(G2) is recorded on that gate's packet, not
   here.

G1 is the first application: `gradient_fills` is **different-but-fine**,
Behavior-Noted in BN-06 fill. The Reference seam is a defect of
triangulation, not a look we owe.

## Consequences

- G2 is no longer blocked on a human who has asked not to be a gate.
- Later Gallery rows (G2 `text_sample`, G4a corpus stills) follow the same
  rule: marshal the packet, review the pixels, write the verdict.
- Forbidden: closing a gate from a status meeting, a bead count, or a
  README tense note.
- GOVERNANCE.md §2 is trued up in the same change as this ADR.
