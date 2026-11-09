# ADR-0002 — D-11 ratified with amendments: the G0-1 object model

**Status:** Accepted (retroactive record of the 2026-07-22 ratification;
the plan's D-11 is already trued up)
**Date:** 2026-07-23
**Bead:** fm-dzv (G0-1 spike); recorded under fm-1o8
**Amends:** D-11 — from "arena + handles pending G0 ratification" to
"ratified as amended".

## Context

D-11 held the §8.1 ownership model (Stage arena, generational handles, CoW
snapshots) pending the G0-1 object-model and buffer-lifetime spike. The
spike (`spikes/g0-1-object-model`: the ten lifetime scenarios, the live
NumPy-view protocol across resize/align/become/glyph-rebuild, and the
compiling fluent-API prototype) ran to green and ratified the model with
amendments.

## Decision

D-11 is ratified as amended. The amendments, normative for W3
(fmn-mobject), are recorded in full in
[`docs/g0/G0-1-object-model-ratification.md`](../g0/G0-1-object-model-ratification.md);
in brief: stage-scoped generational `Copy` handles with the two-scene
policy; root-set scene membership (`remove()` unroots, never destroys);
explicit delete as the only destructor, deferring under pins (the
Python-proxy identity story); first-class multiple parents (diamond-safe
DAG traversal); the snapshot/view exclusion rule for §8.2 — which retires
R12's copy-based fallback; and the scoped-stage + deferred-command fluent
surface.

## Consequences

W3 implements fmn-mobject from the ratification note without consulting
the spike's internals. R12's pivot path (copy-based export) is not needed.
The plan's §23 D-11 entry was trued up at ratification time — this ADR
exists so the amendment is discoverable through the ADR index, giving the
convention (GOVERNANCE.md §3) its worked amendment example.
