# ADR-0001 — The Rev-4 contract: semantic fidelity over pixel identity; pinned bits, free scheduler

**Status:** Accepted (retroactive record; the decision is already binding as
plan Revisions 3–4 and decisions D-01, D-18–D-24)
**Date:** 2026-07-23 (recording a decision executed in the plan's Revision 3
and Revision 4, both pre-repo-bootstrap)
**Bead:** fm-1o8
**Amends:** none — this ADR retroactively records the pivot that *produced*
the current decision log, so the ADR convention has a worked example.

## Context

Revisions 1–2 of the plan carried three goals simultaneously: exact
conformance to the Python Reference's output, cross-platform bit
reproducibility, and improved mathematics under manim's names. Rev 2's
adversarial audit proved the three mutually contradictory: the Reference's
output depends on LaTeX binaries, Pango versions, GPU drivers, float drift,
and two legacy RNG streams — pixels that cannot be reproduced *and*
corrected at once. Separately, two independent external design reviews
(GPT-5.6; Kimi K3) found that Rev 3's determinism doctrine had built nearly
every seam a high-performance engine needs, while §17 still treated
parallelism and SIMD as a posture rather than a designed system.

## Decision

Two linked rulings, executed as plan Revisions 3 and 4:

1. **The contract pivot (Rev 3).** Conformance-to-pixels is dropped as a
   goal — by decision, not by machinery. The contract is API compatibility
   and semantic fidelity: correct mathematics and a beautiful, calibrated
   look under manim's names, with every deliberate divergence documented as
   a Behavior Note. One hard sovereignty rule accompanies it: ffmpeg is the
   only external tool, ever. Roughly a third of Rev 2's compatibility
   apparatus (the shader-faithful backend, dual legacy RNGs, the
   float-drift clock, the certified Pango/llvmpipe capture environment) is
   deleted, not deferred.
2. **The performance-architecture ruling (Rev 4).** One organizing
   principle governs all optimization: **semantics and bits stay pinned;
   the scheduler gets freedom.** Every performance lever is either bit-exact
   by construction (safe in `certified`) or quarantined to `standard` and
   labeled. This lands as D-18 (the parallelism contract and its three
   permanent refusals), D-19 (FramePackets + pure-segment frame
   parallelism), D-20 (the retained render plan), D-21 (adaptive coverage),
   D-22 (the Accelerator Annex through frankentorch only), D-23 (the
   negotiated output sink), and D-24 (ExecutionPlan from HardwareTopology).

## Consequences

The decision log D-01…D-24 in plan §23 is the binding output of this
pivot; the Gauntlet judges self-goldens and the Look Gallery instead of
oracle pixels; the Reference is a design oracle and aesthetic bar, never a
pixel warden. Every subsequent amendment to the decision log or resolution
of an open question lands as an ADR in this directory per
`docs/GOVERNANCE.md` §3, with the plan trued up in the same commit.
