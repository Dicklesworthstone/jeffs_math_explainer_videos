# Complex chapters 02–05: picture repair (Mail 1648)

**Author:** BronzeJaguar, Bead `mx-video-program-program-87do.39`. **This is not independent.** I wrote the review (`complex-preview01-review-02-05/REVIEW.md`, `582c52ff1dd55809…`) and this repair.

**State: uncompiled and unrendered.** No Cargo, Clippy, render, ASR or synthesis ran. Nothing here has been listened to. Canonical is untouched: all 11 files still equal `baseline/` by `cmp` at delivery (`checks/canonical_unchanged.out.txt`). No Beads edit, no commit.

**Narration:** ElevenLabs Oliver stays the narrator. No Michael anywhere. All 105 cue texts are unchanged, so no take is invalidated by wording.

**Maths:** no result, cue or substantive content was changed or removed. One fact was added: the radius ½ that the voice already says (`square.circle.in_radius`, 121 facts).

## Applying it

Stack, in this order, from the repo root:

| Layer | Patch | Touches | Compiles alone? |
|---|---|---|---|
| A | `patches/chapters-02-05.patch` | turning, two_planes, reading_color, gallery, mod.rs, facts.rs, oracle.py, storyboard (picture column only) | Should: it needs no shared layer |
| P1 | `patches/shared-P1-crisp-strips.patch` | draw.rs | Should, on the baseline or on A |
| P2 | `patches/shared-P2-half-step-bands.patch` | model.rs, draw.rs (wheel), gallery.rs (tile) | Should, on the baseline or on A |
| P3 | `patches/shared-P3-optional-four-samples.patch` | draw.rs. **Optional, not in `candidate/`** | Should, on the baseline or on A |

A + P1 + P2 reproduces `candidate/` byte for byte, and each patch also applies alone to the baseline (`checks/roundtrip.out.txt`). "Should" is by reading only. If canonical has moved, compare it with the hashes below and use `git merge-file` with `baseline/` as the base, as `git apply` will refuse.

Tests: 17 new, 1 extended, all unexecuted: turning +3, two_planes +6, reading_color +3, gallery +4, draw +1 (P1), and P2 extends one model test. Please run `./x complex_analysis_visual facts-check` as well, because of the new fact.

## Calls that are yours

1. **P2, half-step bands (fixes 05-A, 05-F).** It moves every brightness edge by half a band and every phase line by half a twelfth, in every colour picture in the film. Hue and size are not touched. The wheel's wedges rotate 15° so each is centred on its named colour, and the tile seed must move too (see below). It is the only fix I found for the comb and the tan bars, because they come from outputs that sit exactly on a band edge.
2. **P3, four samples a cell (04-A2).** Softens stair-steps by a cell. It makes every repaint four times the evaluations, and render cost is unmeasured, so I left it out of `candidate/`. Your other option, a smaller `CELL`, is limited by `a_full_picture_stays_within_the_strip_budget`.
3. **04-C, the shown colouring rule.** This adds new motion in 04_1: a probe, its arm and a needle on the wheel turn to each direction as the voice names it. It was not in the storyboard, so I added it there.
4. **03-E, one window.** ±2.3 from 03_1 to 03_5 means the walk is drawn smaller: 1.17 scene units per 1, where it was 1.80. The unit holds still at the cost of a smaller ring (radius 0.59, image 0.29). The one change of scale is at 03_6, and a caption says so.
5. **03-I, rays off the axes.** The lines are now y = π/4, 3π/4, 5π/4, 7π/4, and the storyboard is updated. The voice stays true.

## Findings

Evidence below means a new in-file test (unexecuted) and its Python mirror (run). **Every "Done" means written, not seen.**

### High and Medium

| ID | Disposition | What changed |
|---|---|---|
| 04-A stripes | Done, P1 | `crisp` sets the strips' anti-alias band to 0.5 px, in `portrait` and `Painter::build`, as GF256 and this crate's thumbnail do. Effect predicted only |
| 02-B arrows short | Done | `reaching_arrow` uses `buff(0.0)`. Test reads the real outline |
| 05-D late result | Done | The walk starts at "and at minus one" and lands on "backwards twice", the line's last words |
| 04-D "simple pole" | Done | Both names are carded inside the loop, clear of the mark |
| 04-B, 04-G no formula | Done | Formula over the picture for each of the four functions, including during the quiz. It does not give away the poles |
| 03-H radius labels | Done | Each stands beside its dot between circles. The test checks every circle against every box; the Python mirror also checks the −45° ray |
| 03-G 1/0 card | Done | Card outside the circle, with a slanted red leader that misses every tick and stays off the axis |
| 05-J, 05-K inset, labels | Done | The same region is a gold box on the wide plot and a gold band at the strip's foot, joined at equal heights. Caption: "gold box = gold band. The strip runs on up to Im s = 32". Labels read "1/2 + 14.13i" with leaders |
| 05-A, 05-F comb, bars | Done, P2 | Predicted from the formula: tan's max/min at y = 3 goes 2.145 → 1.011; the jump across x = 0 goes 0.415 → 0.002 |
| 02-A names on axes | Done | "i" and "−1" stand off their axes |
| 02-C arrow reads as F | Done | The multiplier, its arc and its readouts are teal (the stretch-and-turn role). The F and the rule stay gold |
| 02-D grid through label | Done | "1 + i" is clear of the grid at each of the three rests; the lines still pass over it while moving. New "2i" label |
| 03-open | Done | Points are named z and f(z); the arrow fades as the planes arrive |
| 03-A "turns:" | Done | Word and value keep to one side of the axis |
| 03-D radius labels | Done | Radius 0.5 is now written, from the new fact. Both labels clear the axes and fade with the rings |
| 03-E no scale | Done | Whole-number ticks on every plane, names for 1 and i, a marked 2πi. See call 4 |
| 03-F tick colours | Done | Gold, purple, pink and ink pairs, none in the input's or output's colour. Pink is the pole colour, but no pole is marked until 04_5 |
| 03-I rays on axes | Done | See call 5 |
| 03-J fold tick | Done | Named "0". Image margin 0.04 units is tight, so please look at it |
| 04-A2 stair-steps | **Your call, P3** | Not in `candidate/` |
| 04-C rule not shown | Done | See call 3 |
| 04-E direction | Done | Three solid ink heads on the loop, one arc and head on the wheel |
| 04-J circle over poles | Done | Pole marks are re-added on top |
| 05-L table rows | Done | The column is left aligned at x = 4.15, with rows between the labels. Smallest gap 0.34 units |
| 05-M mosaic | Done | Six panels named; held until "in little tiles" |
| 05-I empty frame | Done | The picture arrives with the formula; the half-plane is then cut out of it, as Γ does. Re-judge with Oliver |

### Low

| ID | Disposition | Why, or what |
|---|---|---|
| 02-E degree sign | Done | Moved in |
| 02-F quiz card | **Not done** | `predict` takes plain text. A TeX card is a shared change for one Low |
| 02-G still 84–96.6 s | **Not done** | The hold may be right under Oliver. Re-judge with the voice |
| 03-B, 03-C counters | Done | Two decimals live; finals read 1.00 and 2.00 |
| 03-K frame height | Done | Now exactly the other frames' height |
| 03 empty planes 44–53, still spans 228–234, 294–312 | **Not done** | Re-judge with Oliver; I did not invent motion |
| 04-F, 04-I minus, repeat | Done | "at i: −1 turn, at −i: −1 turn" and a true minus at 04_5 |
| 04-H stale legends | Done | Band and phase notes clear at 04_6 and do not return |
| 04-K gold segment | Done | "gold segment: where the recipe works" |
| "positive", "negative" unspoken | **Not done** | Shared helper, one caller, harmless |
| "Here is the bump" blank | Partly | The picture still blanks, by the quiz's design; the formula now stands over it |
| 05-B, 05-C output box | Done | Named, and gone until 05_2's walk needs it |
| 05-E sin(3i) card | Done | Card drops inside the picture; arrows reach their dots |
| 05-G gold line over zeros | Done | Marks are re-added on top |
| 05-G "sin x", "Re s" upright | **Not done** | Plain-text cards; re-setting them in TeX means re-laying them out for a Low |
| sin's edge zeros | Done | Window is ±6.6, in the mosaic too |

## Shared changes and every caller

Line numbers are canonical's, from `checks/dead_code_and_callers.out.txt`. I assumed no other lane's change. `thumbnail.rs` has its own tiles and calls none of these.

| Patch | Changes | Callers affected |
|---|---|---|
| P1 | `crisp` at both `RowStrips::build` sites, draw.rs:530 and :577 | `Painter::build`, 19 sites: boundary 146; continuing 229, 540, 572; counting 139; gallery 307, 394, 460, 571, 639, 696, 784, 805; one_point 384; reading_color 62; residues 233, 285; rigidity 371, 424. `portrait`: the_bump 367 |
| P2 | `model::shade`, `BAND_OFFSET` | Reached through `model::portrait` (model.rs:519) and `portrait_colors` (draw.rs:514), so the same 20 pictures as P1. Also model tests 923–924 |
| P2 | `wheel_legend` wedges | reading_color 59 only |
| P2 | `TILE_SEED` → 0.55 + 0.95i, `tile()` corner and test | gallery only, 05_9 and its test. With half-step lines the old seed gives ratio 1.1097, which fails `hi / lo < 1.1`. The new seed gives 1.0325. The present film's is 1.0650 |
| P3 | `portrait_colors` | draw.rs:528 (`portrait`) and :572 (`Painter`), so the same 20 pictures, at 4× evaluations per repaint |
| A (not a shared file) | `gallery::minus` becomes `pub` | Used by reading_color |

## Cue text

**Changed: none.** `script.tsv` is byte-identical: `2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e`, 105 cues, 29 of them in chapters 02–05. In the storyboard, the words column of every row is unchanged and only the picture column differs, in 03_1, 03_6, 03_7, 04_1, 04_6, 05_4, 05_7, 05_9 (`checks/source_order.out.txt`).

On-screen text added or changed. It is not spoken:

| Chapter | Text |
|---|---|
| 02 | "2i" (arrow label) |
| 03 | z, f(z), 1, i, 2πi, 0; "radius 0.5"; "new windows: the ticks still mark whole numbers"; counters to two decimals |
| 04 | f(z) = z, z², 1/z and 1/(1 + z²) as headers; "gold segment: where the recipe works"; "at i: −1 turn, at −i: −1 turn"; true minus in "colour cycles around 0: −1" |
| 05 | "output f(z), sizes squashed"; "1/2 + 14.13i" and its three siblings; "gold box = gold band. The strip runs on up to Im s = 32", replacing "strip squashed: 0 ≤ Im s ≤ 32"; six panel names |

Numbers come from facts, except the plane's unit names (1, i, 0, 2πi), the "1/2" that `CRITICAL` already writes, and the ".00" format on the finals. Predict windows 02_5, 03_3 and 04_6 are unmoved. By reading the source, "2i" arrives only in 02_6's answer, and neither 03_3's ticks nor 04_6's formula shows an answer early.

## Mathematical checks

All independent of the Rust model, and run here in Python (`checks/geometry.py`, 0 failed):

- **Windings:** z² 2; 1/z −1; the bump −1 at i and at −i; cubic 3 on r = 1.8; (z−1)/(z+1)² −2 at −1.
- **Values:** sin i = 1.1752i; sin 3i = 10.018i; tan(±4i) = ±0.9993i; e^(2πi) = 1; (1+i)² = 2i, |1+i| = √2, 45°; band ratio 1.6881.
- **P2 keeps the structure:** shading still repeats every step in size and in phase. Hue is computed separately and untouched.
- **Tile:** all 65 traced points are on band or phase lines; sides ratio 1.0325; it fits the zoom, the zoom fits the panel, and no zero is in the zoom.
- **Zeros:** four marked zeros, all on Re s = ½, all above the gold band, so the box holds none, and the caption says the strip runs on.
- **Oracle** (the crate's own, numpy and scipy): 120 → 121 facts, 0 failed cross-checks, and the only difference is the new line (`checks/oracle_diff.out.txt`).

## What ran, and what it shows

| Check | Result | What it does not show |
|---|---|---|
| `rustfmt --check` on the whole crate through `main.rs`, in a scratch overlay | Clean, across the 29 Rust files in `src/` | That it type-checks |
| `checks/roundtrip.py` | Stack equals `candidate/`, 11 of 11 | That it compiles |
| `checks/source_order.py` | No TEXT/TEX duplicate; every waited phrase literal and in order; sentence indices exist; 04_4 and 05_8 keep their windows | Timing under Oliver |
| `checks/dead_code_and_callers.py` | No unused import; no new item unused outside tests | Text matching, not rustc |
| `checks/predicted_margins.py` | 36 margins, none negative. Tightest: 0.009 (a test's own estimate), 0.05 (mosaic's lower names) | Estimates within about 8%; TeX boxes rougher |

These mirrors caught four real mistakes before delivery: a wrong bound in a new test, a test slicing at the first `#[cfg(test)]` instead of the module's, a duplicate TEX entry, and a test-only constant that would have been dead code. Expect more of that kind from the build: Rust I have not compiled has needed repair before.

## Not verified

**Compile and tests.** Not compiled, not linted, no test executed. Types were checked by reading the pin (a399d62) and the crate.

**Pictures: nothing here has been seen.** Please look at these in particular:

- **04-A:** does P1 remove the stripes, on 1080p and on 4K? Also watch the row edges of the picture and any overlay at 0.5 px.
- **05-A, 05-F:** the comb and bars, and whether the moved bands look right in the 20 pictures, especially near real and imaginary outputs.
- **03:** is the smaller walk still readable; "0" at the fold; tick names against the parabolas; the 1/0 card and leader.
- **04:** size of the solid heads on the loop and wheel; the name cards on the band; the fraction header's fit over the picture.
- **05:** the links crossing the wide plot; the band's 0.55 height at the strip's foot; the mosaic's lower names (0.05 clearance each way); the RH card at the frame's right.

**Listening: none.** I did not hear any take, and this repair touched none.

**Fine sync: unmeasured under Oliver.** New sync points are 04_1 (four phrases), 05_2 (two), 05_9 (one). `Spoken::at` uses measured phrase marks only where the narrator has them, and otherwise a share of the sentence by characters. Fixed animation time grew at some beats: about 0.9 s at 03_1 sentence 2, 0.6 s at 03_6's opening, 0.3 s each at 04_2, 04_3, 04_5 and 04_6's openings, 05_7 sentence 5 and 05_9's opening. Scratch-share estimates say these fit before the next sync, and 05_2's walk is about 3.3 s, with its result held about 1.8 s before the fade. Those are estimates at 2.6 words a second, not Oliver. A fade that overruns its sentence runs late against the voice instead of failing, so please check them on the narrated draft's cue sheets, and re-judge 02-G, 05-I and 03's still spans there too.

## Source hashes

SHA-256 of the full files. `candidate/` is A + P1 + P2. `SHA256SUMS` covers everything in this folder.

| File | Baseline | Candidate |
|---|---|---|
| `docs/storyboard.md` | `6cc2c04286fed44b22a39cac496d797d634cc442b12398278cbd7a8445cd6bff` | `5508c8db94e6b3247eaa336921f581bb08066d80bf48238c8102c0b000c68d2d` |
| `oracle.py` | `b30772102eae42fb332bc4f10f829141187fef2b382b2a8ce00946dc87e5a903` | `080d9073482f34c02cde7681fb0cd3eb54723b8d670ea3fa7040ebc419d52c94` |
| `script.tsv` | `2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e` | unchanged |
| `src/chapters/draw.rs` | `fa1f5ddbc846dfc58550274eb06aab963a45fef1a11474807cfd9a7839ebb7ca` | `29bf91b57c415cb303bbeee47659de880549eb132905f67d784025e471fa03ff` |
| `src/chapters/gallery.rs` | `67ee4db45dc59c372e28315c21fce1e17be43b2cac033a097564a5b67beed88a` | `6f310eb739ec3e5e6a085de42e31dd2be27b9c95dff3e82953530efacf1fed02` |
| `src/chapters/mod.rs` | `13f80812713c062c923811863b863d3bbb1ca577251e7c6c77b545baa376ad61` | `0ca9f5d261308934841a3ec135bf44ef5f714ef2dc5674c62536c037703acb66` |
| `src/chapters/reading_color.rs` | `69259456d3fde27dfe41f4226f35a724107fd43680c3ea3b2c0461ffef4beb35` | `dc98bcee3fc0c5b43fa7fc8601bf4a397dc508bfbd2574e4c809c0ad7d7572d0` |
| `src/chapters/turning.rs` | `a506f057ac1a54461844d7f535d93e38ebe5f02c53aca434bc0a245309a83326` | `2b4dd9a1e41e5e484e243d0fad0c5d4f3d66f92247d6a75104828989361e0e32` |
| `src/chapters/two_planes.rs` | `6867302c7f7e7dd833457f1a8f2851007b425124d54fa0d718c76e7fb02e0e28` | `d8f547c95cef1dda2d15e5512934b5412929fdf575f658ea69c8a3e29e291361` |
| `src/facts.rs` | `cf7b91af8ae17e4dcad690b41b395d7bdb93af8f1de6b6d8d46b5fe39ffab196` | `ea70c413910f9a55ceefa4b87f244cc435284504d808c9b34706168596c4b901` |
| `src/model.rs` | `e32b72591da65c134f03b8536f6d772bf518bed76849a5d2d10a6c2766b12e9c` | `9bf7e92bac77e3ce56897eaa5876c01e7fb38e5d0e8d5a1478b7aacdd32aba49` |

| Patch | SHA-256 |
|---|---|
| `chapters-02-05.patch` | `b07628de2adec09d77adde1faf39a2bf0bbc6d88b1cd240ea6a07e41ff8b5732` |
| `shared-P1-crisp-strips.patch` | `da6491c6603f756dfcaef2c184929cdaa8a2988bd41a7bc7d38a087dcd6f92eb` |
| `shared-P2-half-step-bands.patch` | `ef19cb0504065afecb67fab8369b9a38e92302c9d06f1f39545d4b6ab232a0bf` |
| `shared-P3-optional-four-samples.patch` | `96bc7bf3501e25af25be2c834167e6b82d22701f1323f198ff5a0b352045ab62` |
