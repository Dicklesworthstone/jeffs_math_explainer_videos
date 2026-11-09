# ADR-0008 — Allowlist rows are keyed by (name, version)

**Status:** Accepted
**Date:** 2026-07-25
**Bead:** fm-ekx (surfaced by the G0-8 spike's lockfile)
**Amends:** nothing in the decision log. A policy ruling under D1 (the governed
closure), of the kind GOVERNANCE §3 requires an ADR for; refines ADR-0003's
non-shipped tiers rather than reopening them.

## Context

`SUITE_ALLOWLIST.tsv` has always carried a `version` column, and
`audit_with_aux` has always deduplicated locked packages by `(name, version)`.
But the *row lookup* matched on name alone, so the file behaved as though one
version of a package could exist across the whole repository.

ADR-0003 established that non-shipped crates — the fuzz harness, non-member
spikes — carry their own committed lockfiles and resolve independently. G0-8
turned that from a design statement into a collision: the G0-5 spike's PyO3
graph pins `syn 2.0.119`; the G0-8 spike's Metal graph pins `syn 3.0.3` through
`foreign-types-macros 0.2.4`, which requires `^3`. Neither can move. Under
name-only lookup, whichever spike landed second failed the closure check with a
checksum-drift violation that no amount of correct authoring could clear.

The failure mode is worth naming precisely: **one non-shipped spike's
transitive constraint would veto another's**, for a package that never reaches
a shipped artifact. That is not the governed closure doing its job; it is a
lookup key that disagrees with its own schema.

## Decision

Allowlist rows are keyed by **`(name, version)`**, in the row lookup and in the
stale-row check alike.

Two versions of one package may coexist in the governed universe **only when
each carries its own reviewed row with its own pinned checksum**. Nothing is
loosened:

- a package at a version with no row is an `Unlisted` violation exactly as an
  unknown package is;
- checksum pinning is unchanged and now unambiguous — each version's row pins
  that version's checksum, where before two versions contended for one row;
- a consumed row matching no locked `(name, version)` is still a stale-row
  violation.

Both halves are pinned by tests in `governed_closure.rs`:
`a_listed_package_at_an_unlisted_version_is_still_caught` and
`two_versions_of_one_package_are_admitted_when_both_are_listed`.

## Consequences

- The G0-8 spike's lock lands, with `class=dev` rows for its 15 new packages
  plus a path row for `ft-kernel-metal` (consumed by path until fm-xsz performs
  the §2.9 pin bump).
- **Duplication is a signal, not a convenience.** Two rows for one name means
  two dependency graphs disagreed, and the reason column should say which. The
  `syn` rows do. If duplicate rows ever appear in the *shipped* classes
  (`runtime` / `ffi` / `build`), that is a finding to adjudicate — a shipped
  artifact carrying two majors of one crate is a closure smell this ADR
  deliberately does not bless.
- No change to `SUITE.lock`, to the shipped closure, or to any pin.
