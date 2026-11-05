# complex02-05-repair03: cross-review

Author: RubyRaven, for BlueSalmon's Mail 2649. Bead 87do.39. Narration is ElevenLabs Oliver.

## Verdict

**No blocker in the repair's source, teaching or fades. It can go to root's build.** Nothing here has been compiled, rendered or heard, so this says only that I found no fault by reading and by small checks.

**Three concrete items stand between the repair and an installable map.** All three are patched or ready here. None changes a word of the cues.

| ID | Problem | Fix |
|---|---|---|
| B1 | The frozen binder refuses with the 8 older problems, so no map can be written | `binder/binder-8-problems.patch`: 0 problems on all four trees, and 4 of 4 controls still refuse |
| B2 | "Replace only these fields" is not enough. `narration_units.json` pins the map's hash, so a map edit alone fails with `direction map changed`. The binder's `main` also refuses to rewrite an existing version with new bytes | Install a new `oliver11` map and the units together. Built outputs are in `rebound/` |
| B3 | `mod.rs` TEX and the 13 storyboard picture cells are stale, and two of the cells quote old card text | `rebinding-tex-storyboard.patch`. Apply it **before** binding: it changes `mod.rs`, which the map's `helpers` pin |

## Root's sequence

Run from the repo root. `$O` is the `opus-replacements-20261104` folder and `$X` is this folder. Paste the real paths in.

1. `git apply $O/complex02-05-repair03/complex02-05-repair03.patch`, sha256 `49205a2a…`.
2. `git apply $X/rebinding-tex-storyboard.patch`.
3. Land anything else due in this crate now: Lavender's `draw.rs`, glyph spacing, the 11_7 wording with its `script.tsv`, the radius repair. Each one moves hashes, so bind last.
4. Copy `$X/binder/bind_oliver_plan.py`, or apply `binder-8-problems.patch` to the frozen binder (sha256 `9d6d5aee…`), then run it with `--map-version oliver11`. I ran `build()` only, never `main`.
5. Install the map and `narration_units.json` together, then build.

If only steps 1 and 2 land, compare with `rebound/`, valid only for these exact source hashes:

| Tree | `narration-direction-map-oliver11.json` | `narration_units.json` |
|---|---|---|
| canonical | `fa517b311b9f346b…` | `9d60d7c2130def88…` |
| candidate | `7bdb24f44a0c4dea…` | `85b32298b0fece92…` |

Against oliver10, the map differs only in the 5 expected hold rows (`line`, `marker`, `sha256`) and the 2 `mod.rs` helper hashes. The units differ only in `direction_contract.map.path` and `.sha256`. See `checks/outputs/rebound_diff.txt`.

## Binder patch, for root to accept or change

It re-classifies the 8 problems and turns no check off.

| Problem | Disposition |
|---|---|
| Two stale `DYNAMIC` gallery entries (`Some(p) => …`) | Replaced by the real call and its two literals, per cue: 05_1 "Three zeros inside", 05_2 "backwards twice" |
| `gallery.rs:345` | Same entry. It needed a small `by_owner` form, since the two cues pass different phrases |
| `counting.rs:215` | Sentences 3 and 4, then `line.end()` |
| `counting.rs:277` | `NEEDS_SAID`'s three phrases |
| `reading_color.rs:170` | `NAMED`'s four phrases |
| `closing.rs:278`, `:319` | **My judgment call.** `POST_VOICE_BEATS` keys the exact call on the exact cue (15_1 1.0 s, 15_2 1.2 s) and lists them in the audit. I call them picture beats and not reading windows: nothing new arrives in them. If you read them as reading windows, they need a table row, a storyboard cell and a passing rule instead |

Controls on temp copies all refuse: a changed beat, a stale `NAMED` phrase, a changed walk phrase, a changed `NEEDS_SAID` call.

Without the repair, the patched binder rebuilds oliver10 byte-equal to the map on disk in both trees (`341356cd…`, `bcb6a31b…`). With the repair, its map equals the old map plus the handoff's delta, so **the delta is right as far as it goes**.

With all sibling patches stacked, one problem remains: `storyboard:225 words of 11_7 differ from script.tsv`. That is expected from applying patch 02 without root's new 11_7 script, as that repair's own handoff says. It clears once the script lands. The 23 hold rows still build.

## Timing hooks

All 14 new hooks are exact, each once in its cue and on a word start. The independent read (`checks/outputs/replay_and_hooks.txt`) found 85 literal hooks, in voice order within all 27 cues, and kept 71, added 14, removed 3, matching the handoff's TSV.

| Cue | Source | Hook | Byte |
|---|---|---|---|
| 02_5 | `turning.rs:432` | an eighth of a turn | 37 |
| 02_6 | `:478` | and angles add | 139 |
| 02_6 | `:481` | Every complex derivative | 172 |
| 03_2 | `two_planes.rs:438` | squares each length | 66 |
| 03_2 | `:468` | doubles each angle | 90 |
| 03_5 | `:725` | each point flips | 140 |
| 04_3 | `reading_color.rs:308` | sentence 2 | 89 |
| 04_4 | `:328` | Once for a simple zero (`until`) | 81 |
| 04_4 | `:330` | twice for a zero of order two | 105 |
| 04_4 | `:332` | sentence 2 | 147 |
| 05_3 | `gallery.rs:509` | exact squares | 151 |
| 05_3 | `:512` | repeating every two pi | 166 |
| 05_8 | `:856` | is the Riemann hypothesis | 71 |
| 05_9 | `:955` | nearly square | 134 |

**Phrase marks, correcting the handoff.** It says the 12 new phrases need new marks. In fact complex has no phrase-timings at all, so **every** hook runs on sentence share until one exists, and a missing mark falls back silently. The producer regex in `elevenlabs-golay-20261028/phrase_marks.py` marks every word start, so one run covers all of them, including the variable hooks now listed in the audit.

## Teaching

Read in source and checked by hand. No wrong statement found.

- **03_2:** 1.4 at 30° squares to 1.96 at 60°, so the length squares and the angle doubles. The new labels clear the arrow tips and each other.
- **03_5:** `1/e^{iθ} = e^{−iθ}`. On existing frame 200, all four computed rings and arrows end on the output dots already drawn.
- **04_3 to 04_5:** the needle follows `arg f`, twice with the wheel for `z²` and once against it for `1/z`. The live count ends on the stated 2 and −1.
- **04_4 card:** naming `z = a` removes the old ambiguity. 115 chars, 7.667 s against 7.7 s clean, and it wraps to two lines at 92.
- **05_3:** for `e^z`, `ln|w| = x` and `arg w = y`, so tiles are exact squares. Measured on existing frame 114, the portrait's steps lie within 1 px of the computed lines vertically and 1.7 px horizontally, under the outline's width.
- **05_8 card:** the strip runs to height 32, so it holds exactly the four marked zeros. The next is near 32.94. "this strip" is true.

## Lifecycle and fades

By reading: each new mob reaches one fade and none fades twice.

| Mobs | Leave at |
|---|---|
| `swept`, underlines | Inside `sweep_arc` and `underline_pulse`, which fade their own |
| `shown_rule` + `angles` | 03_2 sentence 2 |
| `second_lap`, `fold`, `flips` | 03_3, 03_5, 03_6 |
| `rescaled` | 03_6 sentence 2 |
| Walk's dot, needle, count | 04_4 sentence 2; 04_6 through `clearing` |
| `double_pole`, `pole_named`, `one_square` | 05_3, 05_3, 05_4 |
| `converges`, `pole_says`, `zedge`, `squashed` | 05_7 sentence 5; `squashed` at 05_8 sentence 2 |
| Painter groups | 05_9, anchors stopped first |

Checks on the engine and the crate, source only:

- **Pre-rooting holds:** `begin_animations` roots only mobs outside the family, and `add_many_to_scene` re-appends, so a re-add promotes.
- **`stage.bring_to_front` does not exist on `Stage`:** the handoff's correction is right.
- **A stopped live value that fades has precedent elsewhere:** for example `visual_proofs/staircases.rs:227` and `logarithm_mysteries/branch_cut.rs:557`. Handoff worry 1 is low risk, though still a first in this crate.
- **Removed public items:** the patch removes none.
- **Existing tests:** no test literal names source text that the patch removed. This scan is narrow; it sees only `contains`-style string matches.

## Low findings, no patch

| ID | Finding | What to do |
|---|---|---|
| L1 | 02_6 "and angles add" is reached at +1.05 s (rule .4 + pulse .65), but two words take about .7–.8 s. So the second pulse starts about .3 s late, still inside the phrase. The handoff's "0.00 s late" comes from character share | Judge on Oliver |
| L2 | 03_2 arcs and `arg` labels stand at full opacity only about .6–.8 s, plus Oliver's sentence pause, before the fade at "Walk a point" | Judge on Oliver. If short, a hold to "once around" would add a hook, so rebind |
| L3 | 05_3's tile is at the height where gold reads worst: contrast to the pixels under its outline is mean 3.90, min 1.90, the lowest mean of 17 tile heights. Band 9 gives mean 5.91, min 3.26 and stays inside the bracket | Look first at the render, as handoff item 4 asks. `EXP_TILE_BAND = (1, 9)` is the one-constant fix, and it needs a rebind |
| L4 | `gallery.rs:861` says the note goes before the lower third "comes up over the place it stands". On existing frame 411 the note ends near px 625 and the card starts near px 643, so they do not overlap. The fade is harmless | Comment only. Leave it |

The handoff's 05-crowd row is also wrong to say `squashed` fades "before the strip comes in". It arrives with the strip and leaves at 05_8 sentence 2, before the lower third. The code is right; the row is not.

## Confirmed from the handoff

- Patch hash; replay with full context on canonical and candidate equals the staged copies; a control refuses.
- `SHA256SUMS` 19 of 19. Canonical still equals `checks/baseline.sha256`, 9 of 9.
- `rustfmt --check` exit 0 on all four staged files, without following modules.
- All 8 binder problems as listed, and the 5 hold rows.
- Candidate and canonical differ as stated.
- 4 proposed TEX entries are new among 163, so `no_supplement_is_empty_or_duplicated` should hold.
- FOUR is 55 chars, 3.667 s against 4.1 s clean.

Left alone on purpose: the 04_4 window stays at 7.5. It passes, and raising it needs no change here.

## Pictures

Ten existing frames read, from `complex-combined-picture-frames02`. **These are the old film, not the repair**, in scratch voice at 720p. Five carry my computed overlay, labelled on the image, and label sizes there are estimates:

- 02: frame 247
- 03: frames 50, 90, 200
- 04: frames 110, 168
- 05: frames 68, 114 (with a zoom), 411, 467

That is a sample chosen for geometry. I did not read every frame the picture review cites. Images are in `checks/outputs/overlays/`.

## Not run

- `cargo` of any kind. **All 19 new tests and the 1 changed test are still unrun**, and my patches have never compiled. The TEX edit is four lines naming constants the staged source defines.
- The binder's `main`. The Rust storyboard parser: my check parses cells in Python, and the binder's own reader passes.
- Any render of the repair, so no picture of it exists. Fine sync and overlap were not measured: timings above are hand estimates at 2.2–3.0 words a second.
- No TTS, ASR, listening or final AV. ASR is not listening. Sampled pictures are not continuous AV. No exact Oliver take was checked.

## Authorship: this is not independent

BronzeJaguar wrote these scenes, the review and the repair. My memory directory holds that lane's notes alongside mine, and it records earlier complex work under this lineage: the glyphs02 review, Clippy repair05 and the glyph spacing repair. So treat this as a second reading and a mechanical gate, not as independent acceptance. The binder patch and the TEX and storyboard patch are my own and have no reviewer yet.

## Files

- `REVIEW.md`
- `binder/binder-8-problems.patch`, `binder/bind_oliver_plan.py`
- `rebinding-tex-storyboard.patch`
- `rebound/{canonical,candidate}/`
- `checks/`, with outputs in `checks/outputs/`
- `bundle.sha256`
