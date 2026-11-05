# Logarithm 11: gauge notes and footers

BronzeJaguar, for BlueSalmon, answering Mails 3142, 3144, 3172 and 3177. Narration is ElevenLabs Oliver and is unchanged.

**State: staged and frozen for root. Nothing here has been compiled, tested by Cargo, rendered or heard.** Canonical is untouched. The patch changes ten film lines in `unreachable.rs`, moves none and changes no timed call. A small helper follows the film's code.

## Your question: 1.222 beside 1.221

**It is a transient, not an endpoint mismatch. The note is right, and no value changes.**

- **The values:** the area is e^(0.2 t), so 1.221403 at t = 1 and 1.822119 at t = 3. Both notes are the correct three-place rounding.
- **Every frame around it** (`checks/outputs/readout-around-t1.png`, from your preview at 30 fps): the readout prints `1.221` from 35.467 s through 35.933 s, across the note's whole fade. It prints `1.222` from 35.967 s, because the clock has started its run to t = 3 (`:910`). Your 36.0 s sample landed just after that.
- **Why the number moves so soon:** the readout prints 1.222 as soon as t passes 1.0004.
- **Cross-check:** the 2.0 s run with the engine's smooth easing, starting at 35.9 s, reproduces all 17 readings I transcribed from those crops. The start time is the one fitted number.
- **At t = 3 there is no such moment:** the readout reaches `1.822` at 37.833 s, before the note fades in, and holds it through the sampled window (`readout-around-t3.png`).

I did not add a hold so that the two sit together at full opacity. That would add film time and move a line, and nothing is wrong with the numbers. With `≈` on the note, the small difference is honest at a glance. A new test pins that each note is the rounding of the value at its own moment.

## The repair

| Item | Line | Was | Now |
|---|---|---|---|
| Gauge notes | `:904`–`:906`, `:914`–`:916` | `t = 1: 1.221`, 18 pt, `dim` | `t = 1: ≈ 1.221`, 22 pt, `blue`; `t = 3` likewise |
| Liouville footer | `:955` | `precisely(`, `dim`, y −3.62 | `footer(`, `ink`, y −3.40 |
| Culver footer | `:1502` | same | same |
| Gallier footer | `:1653` | same | same |

`:64` drops the unused `precisely` import, which Clippy would refuse. `footer` and `FOOTER_Y` stand after the film's code, so no held line moves. `footer` builds the same `precisely: …` string as the shared helper, which I did not touch.

| Item | SHA-256 |
|---|---|
| `logarithm11-gauge-footers.patch`, on canonical or stage06 `unreachable.rs` `a676dac78580ef73…` (same bytes) | `a2764e5167d8b5d171aa79153da16009ca07f83537812e6613fcb708942a5084` |
| Result, staged in this folder at its crate path | `c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7` |

Apply with `git apply` or `patch -p1` from the repo root. Below the film's code, the patch adds two tests and moves one old bound with the footer: the block rows' `> -3.4` is now `> FOOTER_Y + 0.2`.

### Measurements

Scene units; "after" is arithmetic from the extents measured on your frames, not a render.

| Footer | Frame | Off the frame's edge, was → after | To what stands lowest above it |
|---|---|---|---|
| Liouville | 115 | 0.244 → **0.464** (the ∫ reaches lowest) | 0.614, the gauge's foot |
| Culver | 550 | 0.267 → **0.487** | 1.170, the block rows |
| Gallier | 598 | 0.278 → **0.498** | 0.671, the closing red line |

| Contrast on the ground, 720p | Nominal | Brightest tenth of inked pixels | Median inked pixel |
|---|---|---|---|
| Gauge notes as staged, `dim` at 18 pt | 6.13 | 5.18, 5.43 | **3.58, 3.45** |
| Footers as staged, `dim` at 22 pt | 6.13 | 5.84–5.97 | **3.14–3.68** |
| `signed area of the square:`, real `blue` at 22 pt on frame 115, the notes' new spec | 9.36 | 8.60 | 4.50 |
| `ink` at 22 pt on chapter 10's frame 229, the footers' new spec | 16.47 | 15.81 | 8.19 |

The notes grow to about 1.39 wide by the width table, so their right edge reaches about x 2.59, past the text column's start at 2.2. That room is empty on all 68 of your sampled frames from 38.5 s to 69 s. `det < 0` shares the column but sits higher, at y 2.3.

`checks/outputs/gauge-footers-NOT-A-RENDER.png` shows frame 115 as staged, then frames 115, 550 and 598 with the rows recoloured, scaled and moved by hand. The `≈` there is borrowed from frame 598 and hand-spaced, so judge the layout only.

## Decisions for you

1. **`blue` on the notes, not `ink`.** The note names the blue dot's reading, as `mirror: −1` is red beside the red mark, and it stays below the footers and rule rows in weight. If `blue` still reads faint at final size, `ink` is a one-word change on `:906` and `:916`, plus the new test's pattern.
2. **`ink` on the citations is a judgement call.** It matches chapter 10's repair. Culver and Gallier are attributions, so they now outweigh what a citation usually should. They read clearly in the mock-up and do not crowd anything, so I kept one footer style. To quiet them, change the one colour in `footer`; the rows and margin stay.
3. **The rest of the film is now different.** Six other chapter files still use `draw::precisely` (`dim`, y −3.62) once chapter 10's patch lands. Only you have the sheets for them. A film-wide change at `draw.rs:339`–`:340` would touch a shared helper map13 names, and I have not checked those chapters for room.

## Scope

- **Liouville:** "for any real linear flow, steady or not" stays byte for byte. The identity start stays where it already was: the ring at 1, `starts at 1, never touches 0: stays positive`, and the voice's "It starts at one". The footer's string does not state it, and I did not add it.
- **Culver and Gallier:** attribution, theorem scope and citations stay byte for byte. Cues are unchanged.
- **Narration:** no wording need, and no take is affected. `script.tsv`, `facts.rs` and `oracle.py` are untouched. The notes' numbers still come from the facts.
- **Timing:** no timed call changes, so the chapter's length should not change and its music stem should still fit. Only a native run can confirm that.
- **Not touched, since you did not flag them:** other small `dim` text, including the tick numbers (`:863`), `each row is one matrix …` (`:1512`) and `ln 2 ≈ 0.693` (`:1641`).

## Bindings

Both map13 holds on this file keep their line and marker, but the file's hash changes, and `tools/narration_director.py:124` raises on a stale evidence hash. New `sha256` and `evidence.sha256` for both: `c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7`.

| Hold | Lines | `interval_sha256` |
|---|---|---|
| 11_3 → 11_4, prediction | `:963`–`:1070` | **Unchanged**: `e34e65ad69a71865c956f2cf6e3d5270433370e8b6e79e5f8defc221555771db` |
| 11_20 → 11_8, reading | `:1501`–`:1590` | **Changes**, it holds the Culver call: `36386b6b24e173600d4e20b7f70e4285a825196f9000b114b109f834182d2953` → `4a4e6c72d1f92e5e88e7430c03e0f0f062e3e7972184736bfc911a4313fae5e0` |

These are values only, good only for exactly the staged file; I did not patch the map. The formula reproduces both old values on canonical. `combined-repair-stage06/bound-plan/hold-audit.json` and `inputs.sha256` also name the old hash, so your binder must take the new input.

Unaffected: the 12 other `hold-audit.json` rows keep their line and marker; `draw.rs` stays at the hash map13 names; `narration_units.json` does not name this file's hash. Chapter 10's patch is a separate file, and the two can land in either order.

## Native picture checks

`source-markers.json` has these in full. Anchor on cue and sentence: the old times are stage06's scratch voice, not Oliver. Frame numbers are labels in `logarithm-stage06-frames01/11_unreachable`, counted from 0.

| Anchor | When | Old frames | Expect | Fails if |
|---|---|---|---|---|
| G1 t = 1 note | 11_2 sentence 3, after `:909`, to `:993` | 81, 115, 150 | `t = 1: ≈ 1.221`, blue, level with the dot's old place | No sign; grey; the readout not at 1.221 while the clock stands at 1 |
| G2 t = 3 note | After `:919`, to `:993` | 90, 115, 150 | `t = 3: ≈ 1.822`, blue, under the tick at 2; the readout holds 1.822 until the backward run at `:925` | No sign; grey; touching the tick or `det < 0` |
| F1 Liouville | After `:960`, to `:993` | 112–154 (51.5–69.5 s) | Ink at y −3.4; the ring and `starts at 1` above it | Grey; under 0.40 off the edge; the ∫ clipped |
| F2 Culver | After `:1517`, to `:1600` | 491, 550 (223.7 s on) | Ink at y −3.4 | Grey; under 0.40; touching the rows |
| F3 Gallier | After `:1658`, to `:1676` | 573–598 (263–273.783 s) | Ink at y −3.4 | Grey; under 0.40; touching the closing red line |
| N glyphs | A final-size frame in each hold | n/a | Exact words read by eye; a real `≈`, as at `:1639`; ∫ – ½ not fallback glyphs | Any word change; a clipped or fallback glyph |

**Not a failure:** the readout leaving 1.221 just after the note has faded in. The clock has started its run.

```
python3 -B <this folder>/checks/gauge_footer_probe.py notes <frame.png>
python3 -B <this folder>/checks/gauge_footer_probe.py footer <frame.png>
```

The probe works at any frame width. Its marks are my own: 7.0 on the brightest tenth for the notes, 11.0 for the footers, 0.40 off the edge, and a width that proves the sign and size. All three footers and both notes fail on your frames, and the hand-made mock-ups pass. It measures rows; it does not read them.

## Evidence

Regenerate from the repo root with a scratch path that does not exist yet. It starts the probes with `python3 -B`, because they need PIL, runs ffmpeg for sixty small crops of your preview, and regenerates the patch and `source-markers.json`.

```
/usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>
```

| Check | Result | What it cannot show |
|---|---|---|
| Patch round trip by `git apply` | Applies, gives the staged bytes, reverses | Anything about behaviour |
| Film-line diff | Lines 64, 904–906, 914–916, 955, 1502, 1653 only; no timed call changed; the only string change is the notes' sign | That it looks right |
| Transient mirror | Closed form plus smooth easing reproduces 17 readings | The readings are my transcription of your preview, on the scratch voice |
| Geometry mirror | Margins as above; the block-row bound updated | No frame at the new rows |
| New tests' conditions | Pass on staged, fail on canonical; 10 plants caught | The Rust has never run |
| `docs/check_cues.py` on a scratch copy | Passes both; differs only in listing the two notes; refuses a removed `say` | Only cue order |
| `rustfmt --check` | Clean on both | Not a compile |

**What I read:** sheet 020, frames 115, 150, 491, 550 and 598 by eye, and the 60 dense crops. The 68 frames behind the empty-room check were measured by script, not read. This is not a chapter review; yours covers all 150 sheets.

My one failed check on the way was my own: its pattern matched `tween_all` in the import list. That is fixed in `run_checks.py`.

## Overlap and gaps

- **Not independent.** You named the defects, and I wrote the fix and every check. I flagged the gauge notes' missing sign in my log09 handoff, from source only.
- **Shared memory.** This file also carries my earlier `logarithm11-label-repair01`. WhiteSwan and RubyRaven worked on this film, and we share one auto-memory directory, one model and the same skills.
- **Still yours:** Cargo, tests and Clippy; the glyph probe; a chapter 11 render and the anchors on it; rebinding; Oliver sync and listening; the final 4K look.

I changed nothing in canonical, Beads or any sealed folder, and started no agents.
