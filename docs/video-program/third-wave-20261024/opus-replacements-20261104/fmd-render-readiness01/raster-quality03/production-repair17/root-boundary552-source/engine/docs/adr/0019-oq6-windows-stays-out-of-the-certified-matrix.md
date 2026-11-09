# ADR-0019 — OQ-6: windows-x86-64 stays out of the certified matrix

**Status:** Accepted
**Date:** 2026-08-25
**Bead:** fm-u7ev (W1/OQ-6)
**Resolves:** OQ-6 — "(post-W1) Windows bit-certification."
**Amends:** nothing. The certified matrix membership question is closed; D-18's
refusals and §16.7's closure mechanism stand unchanged.

## Context

`docs/INPUT_CLOSURE.md` §5 froze the certified matrix at G0-6 (fm-zn9) and
re-measured it on the certified engine's own corpus (fm-ig3): linux-x86-64,
linux-aarch64, and macos-aarch64 produce identical digests across libc
families and execution modes, including PG-5's {1,4,16}-thread sweeps.
windows-x86-64 entered the world as *functional CI only*, with
bit-certification deferred to OQ-6, "a separate declared decision owned
post-W1". Until now no bead owned that decision, leaving G4b (fm-yp0),
the risk register (R18), and every distribution document citing a question
instead of an answer.

What a *yes* would have required is known precisely, because ADR-0010 fixed
what certification rests on: fmn-dmath owning every certified transcendental,
no FMA contraction, fixed-order reductions, IEEE-754 basic operations only —
each verified by *measurement* on the platform being added (fmn-dmath cross-
platform vectors under the target toolchain; object-code FMA audit of the
MSVC-generated scalar path; the fm-ig3 corpus hashed bit-identically on
native Windows hardware, with the toolchain identity entering the input
closure per §16.7). None of that evidence exists: this program has no
reachable native windows-x86-64 host today — the same hardware honesty
ADR-0007 recorded for CUDA (`ts2` is adequate to prove a mechanism, never to
certify bits) and INPUT_CLOSURE §5's caveat records for qemu-user aarch64.

## Decision

**windows-x86-64 is out of the certified matrix.** `--reproducible` promises
bit-identical artifacts on linux-x86-64, linux-aarch64, and macos-aarch64;
on Windows it is unavailable (or, where surfaced, explicitly standard-mode).
Functional CI — the workspace compiles warning-free and its tests pass on
windows-x86-64, failing the commit that breaks them — continues unchanged;
that lane proves portability, never bits.

This is the conservative reading of the program's own rule: adding a platform
to the certified list is an ADR backed by measurement, because "widening it
silently would mean shipping a promise nobody measured." Ruling Windows OUT
without evidence is the honest state; ruling it IN without a reachable host
would be the dishonest one. Nothing here asserts Windows *cannot* certify —
only that nobody has.

**Revisit trigger:** a persistent native windows-x86-64 host becomes
reachable (owned machine or a reproducible CI runner). The successor work is
then mechanical and bead-tracked: run the fmn-dmath vectors and the fm-ig3
certified corpus natively, audit MSVC-generated object code for FMA
contraction against ADR-0010's four properties, pin the toolchain into the
closure, and file the IN-ruling ADR citing those numbers. Note that MSVC and
MinGW-GNU toolchains are different closure identities (different codegens,
different libm story) — each would need its own evidence and its own row.

## Consequences

- G4b's closure text, the risk register, and the distribution docs cite this
  ADR instead of an open question; the dangling-reference bug dies.
- `docs/INPUT_CLOSURE.md` §5's Windows row, the plan's §23 OQ list,
  AGENTS.md's determinism contract, and README's Limitations are trued up in
  this commit.
- W11 packaging keeps shipping windows binaries as functional artifacts;
  no installer, release note, or doctor output may describe Windows output
  as certified.
- No gate waits on this: G4b enforces the frozen three-platform matrix, and
  §10.7-style annex rules were never Windows-dependent.
- The revisit path above is deliberately narrow: one class of evidence, one
  successor ADR, no standing Windows workstream.
