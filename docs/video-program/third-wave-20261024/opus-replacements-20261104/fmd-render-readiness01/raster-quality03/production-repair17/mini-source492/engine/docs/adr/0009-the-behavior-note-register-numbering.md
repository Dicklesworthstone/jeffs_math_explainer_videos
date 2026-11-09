# ADR-0009 — The Behavior Note register: numbering rules, and §16.8 trued up

**Status:** Accepted
**Date:** 2026-07-25
**Bead:** fm-nn8 (filed by fm-y69 when it hit the collision)
**Amends:** plan §16.8, whose seed list is replaced by a reference to the
register index. No decision-log entry changes; D-05 (correct by default,
documented when different) is the standing rule this operationalizes.

## Context

§16.8 seeds the Behavior Notes with ten numbered entries. Notes are cited by
bare number from the Parity Ledger's `evidence` column, from crate
documentation, and from user-facing migration guidance — so a number is an
identifier, not a label.

Three things had drifted:

1. **Two files claimed BN-08.** `BN-08-animation-contract.md` (W4) took the
   number before the de-TeX'd classes had a note; `BN-08-de-texed-natives.md`
   (W7) then wrote to the number §16.8 normatively assigns it. Nothing outside
   the animation-contract file referenced its number — every `BN-08` in
   `docs/api/ledger.tsv`, `API_OVERLAY.tsv`, and `crates/**` points at the
   de-TeX'd natives.
2. **Two files claim BN-07**, and on inspection that is *correct*: §16.8 defines
   BN-07 as "Reference bugs fixed (Appendix C rulings)", Appendix C carries
   eleven rulings owned by different workstreams, and forcing them into one
   document would mean every workstream touching Appendix C rewrites the same
   file.
3. **Two notes grew past the seed list** — BN-10 (skip-mode updater time) and
   the animation contract — with no rule saying whether that is allowed.

The absence of a numbering rule is what let all three happen quietly.

## Decision

**The register index is `docs/behavior_notes/README.md`**, and it is
authoritative. §16.8's seed list is replaced by a pointer to it, so a new note
no longer requires a plan edit.

Four rules, recorded in the index itself:

1. **One number per note.** Assigned monotonically, **never reused**, and never
   renumbered once anything outside the note cites it.
2. **A number may name a family** when the plan's own entry does — BN-07 is the
   standing example, and this ADR blesses it rather than splitting it.
3. **A reserved number stays reserved.** BN-06 belongs to the renderer and waits
   for W5. Filling a reserved slot with unrelated content is how registers rot.
4. **Status is `Draft` → `Final`**, and a note goes Final when its subsystem's
   gate passes and its migration guidance has been reviewed against real
   behaviour — not when the code lands.

**One rename, under rule 1's own escape clause:** the animation contract moves
from BN-08 to **BN-12**, because nothing outside the file cited it. The de-TeX'd
note keeps BN-08, matching §16.8. Renumbering it instead would have contradicted
a normative line of the plan to preserve a number no one referenced.

## Consequences

- `docs/behavior_notes/BN-08-animation-contract.md` →
  `BN-12-animation-contract.md`, with a dated renumbering callout in the file.
  Authorized explicitly by the program owner on 2026-07-25 under RULE 1
  (AGENTS.md), who selected the rename over leaving the ambiguity documented.
- No code, ledger, or overlay change was required — verified by grep before the
  rename, which is also the evidence for rule 1's escape clause.
- §16.8's prose is replaced with a pointer to the register index; the seed list
  survives there as the first eleven rows.
- **Future notes need no ADR.** A workstream that finds a deliberate divergence
  writes the note, takes the next free number, and adds its row. That is the
  point of moving the register out of the plan.
- BN-06 stays reserved and unwritten. It is a visible gap on purpose: the
  renderer's Behavior Note is a W5 deliverable, and an empty row states that
  more usefully than silence.
