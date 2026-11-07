# Chapter 08 Goursat repair: handoff (LavenderPike, pane 0)

Status: NATIVE SOURCE IMPLEMENTED, awaiting guarded build, native tests and actual picture/listening review. Codex `complete_native08` completed the delegated source repair on 2026-11-07 after the previous worker stopped before implementation. The approved `08_5-script.txt` remains byte-identical to draft02; no synthesis or narration edits were performed.

## Implemented source and verification, 2026-11-07

`loop_sums.rs` now replaces the old local-linearization shortcut with a complete native Goursat sequence:

- Four counterclockwise square boundaries, individually offset direction arrows, exact shared-edge cancellation, the parent integral identity and triangle-inequality selection bound.
- Four further subdivisions of the selected child, with retained ancestor outlines and a marked intersection point. This is an illustrative admissible branch under the contradiction hypothesis, not a fabricated numerical nonzero holomorphic integral or measured maximizer.
- Constant and linear contributions drawn as exact primitive-endpoint changes, tip to tail. The linear chain uses quadratic primitive differences, not approximate quadrature.
- The derivative remainder bound at the one limit point, an enclosing local neighborhood, magnification of the actual selected square, and a boundary walker with a live segment from c. The diagonal and perimeter remain explicit geometric objects.
- Both lengths halving, their product quartering, the separate lower and upper integral bounds, visible cancellation of the two `4^n` factors, and the arbitrary-epsilon conclusion.
- Midpoint subdivision of a triangle. All four children are counterclockwise, including the inverted middle child; opposite shared-edge directions cancel. The diameter edge is selected from the actual longest pair of vertices.

The existing `08_1` through `08_4` scene block is unchanged. The after-voice Goursat caption and existing reading-window/breath convention remain; the caption now correctly describes a remainder bound at the nested limit point rather than claiming a uniform grid error estimate.

Root additionally delegated `mod.rs` as a single-writer registry edit. It includes all new 08 fixed text/TeX and the new 09 constants supplied by `complete_native09`; chapter definitions, prediction windows and reading-window durations are unchanged. Static constant resolution found **322 unique TEXT entries and 213 unique TEX entries**, with no duplicate or empty entry reported. All new literal 08 TeX/text is covered, including the two affine-integral formulas passed through the loop variable.

Checks actually performed:

- `rustfmt --check` passes for the two assigned Rust files with repository format settings and `skip_children=true`.
- **33/33 direct phrase waits** occur exactly once and in order in the unchanged approved narration; the two affine-loop phrases also match. Narration SHA-256 remains `15e727f0bf7696a24123a6c7c8d5ac1f56a31ebccb08c0e561e164d4ad7ee8a7`.
- Independent arithmetic checks passed **12/12 child orientation/area/diameter/perimeter cases**, **3/3 directed-edge cancellation cases**, **96/96 factor-four cancellation checks**, and **12/12 exact affine primitive chains**, including c on the boundary. These are arithmetic checks, not native-render evidence.
- Six native Rust tests were added for subdivision, directed edges, scaling, affine primitives, remainder/geometry bounds and narration phrases. They have **not been executed**; root owns Cargo, clippy and the guarded build.

Source SHA-256 at handoff: `loop_sums.rs` = `fc0b49ab10fdddc292ad8ab6149ac4a78da0d12da21a64805b786ec55674b577`; `mod.rs` = `fedb9075d73901baefb03753d5b8c58417ec4dc458b0c1c55c612ddf501234ed`.

Remaining qualification: integrate the source with the exact approved 08 script, regenerate phrase marks from the selected Frederick Surrey take, run native tests/clippy and glyph probe, then inspect actual pictures and continuous synchronization. In particular inspect the long linear-primitive equation, the magnification transition, cancellation factors, central-triangle arrows and final lower third. No build, frame, audio/listening, release or publication acceptance is claimed here.

## Cue text

`08_5-script.txt` holds draft02, sha256 `15e727f0bf7696a24123a6c7c8d5ac1f56a31ebccb08c0e561e164d4ad7ee8a7`: 33 sentences, 502 words, 2,788 characters. It is one `08_5\t...` row followed by a newline, in the same shape as `09_1-script.txt`.

Draft02 is root's correction of draft01 (the pane message before synthesis):
- **Algebra.** Draft01 said "the original integral is at most four to the n times the small square's integral. And that is at most epsilon, times the original diagonal and perimeter, divided by four to the n." "And that" reads as four to the n times the small integral, which is not bounded by epsilon D0 P0 / 4^n. Draft02 states the bound on the small square's integral, then multiplies by four to the n, which cancels the fours and bounds the original by epsilon D0 P0.
- **Wording.** Comparisons of complex integrals now say "size", and so does the remainder ("the remainder's size").
- **Pigeonhole.** The sentence is split in two, so that it says each of the four is under a quarter: "Then some small square must have an integral at least a quarter of that size. Four integrals each under a quarter of that size could not add up to it."

### Superseded draft01 (preserved verbatim; do not synthesize)

sha256 `b1b6334033878f33291b96af20360a0e6987e7b0dec1505ed262f3a736576b30` (the file's bytes, including the id, the tab and the final newline):

```
08_5	Cauchy's theorem says more. If a function is holomorphic on a loop and everywhere inside it, its loop integral is zero. We will prove the pieces we need, starting with a square, and a function holomorphic on it, inside it, and a little beyond. Cut the square into four smaller squares, and walk each one counterclockwise. When we add their four loop integrals, every shared edge is walked twice, in opposite directions, so those parts cancel exactly. What remains is the loop integral around the original square. Now suppose that integral were not zero. Then at least one small square must have an integral at least a quarter as large, because four smaller ones could not add up to it. Keep that square, and cut it into four again. Each time, keep a piece whose integral is at least a quarter as large as its parent's. These nested squares shrink toward a single point, which we call c. From here on, we use the derivative at that one point only. Near c, the function is a constant, plus a linear term that stretches and turns z minus c, plus a remainder. The constant integrates to zero, because the steps around a closed loop add up to nothing. The linear term integrates to zero too, because it is the derivative of a quadratic, whose changes around a closed loop also add up to nothing. So only the remainder contributes. Pick any tiny factor, and call it epsilon. Close enough to c, the remainder is at most epsilon times the distance to c, and that is exactly what having a derivative at c means. After enough cuts, the whole nested square lies that close. There, the distance to c is at most the square's diagonal, and the steps add up to the perimeter. So the integral around that square is at most epsilon, times its diagonal, times its perimeter. Each cut halves the diagonal and halves the perimeter, so this bound shrinks to a quarter every time. But our rule kept at least a quarter of the integral at every cut. So after n cuts, the original integral is at most four to the n times the small square's integral. And that is at most epsilon, times the original diagonal and perimeter, divided by four to the n. The fours cancel, and what is left is epsilon times a fixed number. Since epsilon can be as small as we like, the integral is zero. The same shrinking argument works for a triangle. Join the midpoints of its sides, and you get four triangles at half the size. Walk all four counterclockwise, even the upside down one in the middle, and their shared edges cancel again. The longest side plays the diagonal's part, and it halves with the perimeter at every cut. So around every triangle, the integral is zero too.
```
