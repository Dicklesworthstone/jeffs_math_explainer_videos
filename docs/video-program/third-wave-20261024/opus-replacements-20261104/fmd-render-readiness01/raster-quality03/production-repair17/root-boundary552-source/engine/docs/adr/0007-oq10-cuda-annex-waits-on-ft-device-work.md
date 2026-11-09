# ADR-0007 — OQ-10: the CUDA annex waits on upstream ft device work

**Status:** Accepted
**Date:** 2026-07-25
**Bead:** fm-ekx (G0-8, the accelerator proof spike)
**Resolves:** OQ-10 — "Whether CUDA-via-ft reaches annex quality now or waits on
upstream ft device work — the ledger item's outcome decides."
**Amends:** nothing. D-22 stands unchanged; this decides the timing of its
second backend.

## Context

§10.7 gives the Accelerator Annex two backends, both through frankentorch:
Metal now, CUDA "via ft as an upstream-ledger spike (§2.9)". G0-8's brief was to
prove the Metal half end-to-end *and* open the CUDA feasibility question on the
ledger, with OQ-10 owned by this gate.

The Metal half is proven — `docs/g0/G0-8-accelerator-ratification.md`. Proving
it also established what a *generic* GPU path through ft actually requires,
which is what makes the CUDA question answerable rather than speculative:
frankentorch had no route to a GPU for a consumer's own kernel at all, and G0-8
closed that gap upstream as `ft-kernel-metal::compute` (UPSTREAM_LEDGER row 8).

## Decision

**CUDA-via-ft does not reach annex quality now. It waits on upstream ft device
work**, tracked as ledger row 7, with two named preconditions that must be
settled *before* any CUDA code is written:

1. **The gateway's buffer API must generalize; its dispatch API need not.**
   The Metal gateway's `SharedBuffer` is a unified-memory allocation whose
   contents the host reads directly — correct on Apple silicon, where
   `has_unified_memory()` is true and §10.7's "the handoff is nearly free" is a
   fact about the hardware. §17.6's CUDA playbook wants the opposite shape: a
   scene **resident** on the device across frames, with only deltas uploaded.
   That is a different contract (device-resident handles, explicit upload and
   readback, lifetime across dispatches), and it is the piece of the gateway
   that has to change. The dispatch API, the flat single-typed buffer layout,
   the host-side deterministic binning, and the kernel itself port by
   construction — the spike's kernel uses no Metal-specific intrinsic beyond a
   two-line `cbrt` stand-in.

2. **The D1 closure must rule on shipping precompiled PTX.** Metal compiles
   shader source at run time from a string, which is why the spike ships its
   kernel as `include_str!`. CUDA has no equivalent without either shipping a
   toolchain or precompiling to PTX at build time. Precompiled PTX in a
   foundation crate is a new artifact class in the governed closure — a
   build-time toolchain dependency and a binary blob — and D1 has no standing
   ruling on either. That ruling is a prerequisite, not an implementation
   detail to settle in a pull request.

**On hardware.** The only reachable NVIDIA part is `ts2` (GTX 1070, Pascal,
8 GiB). It is adequate to prove the gateway shape functionally. It is **not**
PG-A's declared RTX-4090-class profile, so any measurement taken there is a
feasibility observation and must never be reported as a PG-A number.

## Consequences

- Ledger row 7 stays **proposed**, now carrying the two preconditions and the
  hardware caveat rather than an open question.
- **fm-ktj** (the CUDA annex production backend) remains blocked on fm-ekx's
  successor work, not on fm-ekx. Its first task is precondition 1, upstream in
  frankentorch; its second is precondition 2, a D1 ruling that will be its own
  ADR.
- Nothing in W5 waits for this. §10.7 already excludes annex engines from the
  core gates, and G3's preview duty is served by Metal, so a deferred CUDA
  backend costs the program no gate.
- The annex's constraints are unaffected and remain permanent: standard-mode
  only, never `certified` (D-18), PG-A-gated only, frankentorch the only
  gateway, no wgpu.
