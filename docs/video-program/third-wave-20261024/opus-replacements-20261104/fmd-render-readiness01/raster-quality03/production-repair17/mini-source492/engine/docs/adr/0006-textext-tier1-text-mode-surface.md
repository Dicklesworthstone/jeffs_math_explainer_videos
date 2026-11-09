# ADR-0006 — OQ-4 resolved: the TexText tier-1 text-mode surface

**Status:** Accepted
**Date:** 2026-07-24
**Bead:** fm-7dw (Scribe II)
**Amends:** resolves OQ-4 (text-mode TeX (`TexText`) surface breadth for
tier-1; owner W6)

## Context

`TexText` runs a text mainland through TeX, with mathematics only in
explicit islands. The Reference compiled the whole string through LaTeX,
so *any* text-mode LaTeX worked incidentally. Natively, the text-mode
surface must be declared: what does tier 1 parse and lay out, and what
refuses by name? fmd-math's `parse_text` (landed over fm-wgl → fm-kg9)
implements the candidate surface; this ADR fixes it as the tier-1
contract.

## Decision

The tier-1 TexText text-mode surface is exactly:

1. **The text mainland** — literal text with TeX whitespace collapse,
   the escape set (`\$ \% \& \# \_ \{ \} \\`, `\,`-class spaces, `~`
   ties), and `%` comments.
2. **Math islands** — `$…$` (text style) and `$$…$$` (display style),
   each island the full tier-1 *math* surface (whatever the ratchet says
   math tier 1 is — islands never lag the math tier). The Reference-era
   missing-`$` recovery is kept deliberately: a self-contained math
   command or bare script in the mainland becomes an explicit implicit
   island, never a silent mangle.
3. **Text styling** — `\textbf{…}`, `\emph{…}` (and `\textit` as its
   alias surface), `\underline{…}`, nesting allowed.

Everything else in text mode — sectioning, lists, `\text`-mode font
size commands, tabular-in-text, arbitrary LaTeX — is **outside tier 1**
and refuses with the standard named, tier-tagged error (`env:center`
and friends are already T2 rows in the construct table; unknown
commands report as untiered). The G0-4 harvest supports this cut: the
corpus's TexText usage mass is mainland + islands + the three styling
forms; everything beyond is long-tail, scheduled by corpus rank under
fm-j5t like any other construct.

## Consequences

fmn-tex's `Mode::Text` is a *complete* implementation of the declared
surface (fm-7dw's tests + fmd-math's `parse_text` suites lock it);
`fmn doctor` and the public dashboard can state the TexText contract in
one sentence; text-mode gaps discovered in the wild are ratchet items
with names, not surprises. The plan's §23 OQ-4 entry is trued up in
this commit.
