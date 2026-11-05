# Logarithm 09: matrix precision and caveat contrast

BronzeJaguar, for BlueSalmon, answering Mail 3105. Narration is ElevenLabs Oliver and is unchanged.

**State: staged and frozen for root. Nothing here has been compiled, tested by Cargo, rendered or heard.** Canonical is untouched. The patch changes three film lines in `matrix_logs.rs`, moves none and changes no timed call.

## The repair

| Line | Was | Now | Why |
|---|---|---|---|
| `:266` | `r"e^{X} ="` | `r"e^{X} \approx"` | Your finding. The card's digits are rounded, and the row sits directly above the exact `e^X = A` |
| `:179` | `model::rotation(TAU / 6.0)` | `[[0.6, -0.8], [0.8, 0.6]]` | The turned card rounded `0.866` with no sign of it. **I removed the rounding, not marked it; see the decision below** |
| `:213` | `Palette::dim()` | `Palette::ink()` | The caveat's contrast |

Exact rows stay exact: the definition at `:273` (`e^{X} = A`) and the series at `:222` keep their `=`.

| Item | SHA-256 |
|---|---|
| `logarithm09-matrix-precision.patch`, on canonical or stage06 `matrix_logs.rs` `c82b2a4d10f3c5fd…` (same bytes) | `feb5ce4b5ffad77a4a33fae126d1768aafa177a1ced57b345fe9ced0f0309663` |
| Result, staged in this folder at its crate path | `7df6f8681099c76dcd1f32494e2edcbb07b427c54178d5c557d5801052f481b0` |

Apply with `git apply` or `patch -p1` from the repo root. The patch also adds two tests and updates one test's copy of the matrix.

## Decision for you: the turned card

You asked for a clear rounding indication. **I recommend exact entries instead**, because:

- The card is on screen for only 1.25 s, and it has no left-hand side for `≈` to attach to.
- `[0.6 −0.8; 0.8 0.6]` is a true turn, orthonormal with determinant 1, and the card's three places hold it exactly.
- It suits the line above the card, "the matrix's columns: where the two unit arrows land", since the arrow tips read directly off the card.

**The cost is a picture change:** the turn is 53.13° instead of 60°. No script line, card or fact names the angle. The grid's extents grow slightly (x −6.476..0.076, y −3.676..2.876, by mirror), still inside the safe area. This is why A1 below needs your eye.

**To keep 60°**, restore `:179` and its test copy and tell me: I would stage a bare `≈` left of the card. That adds a mob and moves film lines, so it is not the smaller patch.

## The audit

| On screen in chapter 09 | Digits | After this patch |
|---|---|---|
| `e^X` card | Rounded: true values 0.90522, −0.97041, 0.77633, 0.51706; largest error 0.00041 | **`≈`** |
| Turned card | Was rounded (`0.866` for √3/2) | **Exact** |
| Stretched, sheared, flipped, `X`, `J` | Exact | Unchanged |
| `0.25`, `1.25`, `0.75` turn labels | Exact | Unchanged |
| `time t =` | A moving two-place readout that ends on 1.00 | Unchanged, makes no claim |

So `e^X` is now the only rounded card in the chapter, and the only one marked. A new test pins this. `−0.97` prints two places because the trailing zero of −0.970 is dropped; with `≈` that does no harm, so I left it.

**Outside this chapter, for your sheets, not patched:** the chapter 11 gauge labels at `unreachable.rs:904` and `:914` print `t = 1:` and `t = 3:` from 3-place facts with no sign, also in 18 pt `dim`. I did not look at their frames. `doing_nothing.rs:474` says "turns N radians" with a rounded N, followed by "about Nπ".

## The caveat

Measured on your stage06 frame 48 at 720p (contrast against the ground, 12/17/20):

| | Nominal | Brightest tenth of inked pixels | Median inked pixel |
|---|---|---|---|
| Caveat as staged, `dim` at 20 pt | 6.13 | 5.40 | **3.39** |
| Line above the card, `ink` at 22 pt | 16.47 | 15.42 | 7.72 |
| Caveat remapped to `ink`, not a render | 16.47 | 13.68 | 7.21 |

**This is a judgement call, not a hard failure.** `dim` clears WCAG's 4.5 at nominal, but the font's thin strokes bring the median pixel below it at this size. This line is also the chapter's guard against reading a matrix as a path, so I gave it `ink`.

**I kept 20 pt.** Size is not the problem: about 23 px of x-height at 4K by arithmetic, and 09_4's labels and dictionary line are also 20 pt. The voice makes the same point while the caveat is up, and its timing does not change.

See `checks/outputs/caveat-dim-vs-ink-NOT-A-RENDER.png`: your frame with the caveat's pixels recoloured by hand, which shows the hierarchy but is not what the engine will draw. The measurements are at 720p only, since no 4K frame of this chapter exists yet.

## Validation anchors

`source-markers.json` has these in full. Anchor on cue and sentence: the old times are stage06's scratch voice, not Oliver. Frame numbers are labels in `logarithm-stage06-frames01/09_matrix_logs`, counted from 0.

| Anchor | When | Old frame | Expect | Fails if |
|---|---|---|---|---|
| A1 turned | 09_1 sentence 3, first example, 0.9 s hold at `:206` | 29 (12.4 s) | `[0.6 −0.8; 0.8 0.6]`, arrow tips matching it | Any `0.866`; the grid reaches the title or right column |
| A2 caveat | 09_1 sentence 4 until 09_2's fade | 46, 48 (18 s+) | Ink, same place and size; `caveat_contrast.py probe` at or above 11.0 | Still grey; probe below 11.0 |
| A3 word | 09_2 sentence 4 until 09_4's fade | 110, 113, 128 (44.4 s+) | `e^X ≈`, green, clear of the unchanged card's bracket | `=`; touching the bracket; a fallback glyph |
| A4 exact | 09_3, and the series row before it | 128 | `e^X = A` and the series keep `=` | Either shows `≈` |

```
python3 -B <this folder>/checks/caveat_contrast.py probe <frame.png>
```

The probe's 11.0 is my own mark, halfway between the two nominal contrasts. Its controls are your frame 48 (5.40 fails) and the ink sibling line (15.42 passes). It measures a box; it does not read the picture.

A3's glyph has a real precedent: `ln 2 ≈ 0.693` and `e ≈ 2.718` on your `04_undoing_growth` frame 162. The new string reaches the glyph probe by itself, because the crate captures strings built through the kit, so there is no registry row to add. In standard TeX metrics `≈` and `=` have the same advance, so the word should not move; that is not measured in this engine.

## Narration and bindings

- **No wording need, no take affected.** "Where everything ends up is the matrix e to the X" is exact about the abstract matrix, and the voice never says the digits. `script.tsv` is untouched.
- **No hold moves.** Map13 has no visual hold on this file, and neither map13 nor `narration_units.json` names its hash. All 6 `hold-audit.json` rows on this file keep their line and marker.
- **Rebind one input.** `combined-repair-stage06/bound-plan/hold-audit.json` and `inputs.sha256` name the old hash, so your binder must take the new input.

## Evidence

Regenerate from the repo root with a scratch path that does not exist yet:

```
/usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>
```

It starts `caveat_contrast.py` with `python3 -B`, because that needs PIL, and regenerates the patch from the staged file.

| Check | Result | What it cannot show |
|---|---|---|
| Patch round trip by `git apply` | Applies, gives the staged bytes, reverses | Anything about behaviour |
| Film-line diff | Lines 179, 213, 266 only; line count kept; no timed call changed; the only string change is the `e^X` word | That it looks right |
| Closed form and series for `e^X`, independent of `model.rs` | Agree within 1e-12; reproduce your four digits | That the Rust computes the same |
| Exactness and geometry mirror | All cards as in the audit; landings, clip and safe area pass on the new turn | No frame; the mirror covers grid corners, not every stroke |
| New tests' source conditions | Pass on staged, fail on canonical, 4 plants caught | The numeric half of the tests is mirrored, not run |
| `docs/check_cues.py` on a scratch copy | Same on canonical and staged; refuses a removed `say` | Only cue order |
| `rustfmt --check` | Clean on both | Not a compile |

**Frames I read, and only these:** 09 frames 29, 48, 113, 128, and 04 frame 162. This is not a chapter review; your sheets continue.

## Overlap and gaps

- **Not independent.** You named the `e^X` and caveat defects, and I wrote the fix and every check.
- **I missed this earlier.** My independent source review of the 08–10 repairs (Mail 1533, `logarithm-repair08-10-independent-source-review01/REVIEW.md:184`) measured this card and word and did not flag the `=`. A source reading did not catch what your picture review did.
- **Shared memory.** WhiteSwan repaired this chapter, and we share one auto-memory directory, one model and the same skills. RubyRaven's rebuild is the base.
- **Still yours:** Cargo, tests and Clippy; the glyph probe; a chapter 09 render and A1–A4 on it; Oliver sync and listening; the final 4K look of the caveat.

I changed nothing in canonical, Beads or any sealed folder, and started no agents.
