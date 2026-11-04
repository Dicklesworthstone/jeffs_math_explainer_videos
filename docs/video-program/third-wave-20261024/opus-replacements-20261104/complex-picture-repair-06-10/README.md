# Complex chapters 06–10: native picture repair

LavenderPike for BlueSalmon, Mail 1654, bead `.39`. Date: 2026-11-04. Narration stays ElevenLabs Oliver.

**Status: a proposal. Nothing here is installed, compiled, rendered, voiced or independently accepted.** I repaired my own review's findings, so this needs a reader other than me. Cue text deltas: **none**. Script hash unchanged, `2968b0fd76540cbc`.

## What this repairs

The review (`../complex-preview01-review-06-10/REVIEW.md`, `9c2fe161174301f5`) found 5 High, 25 Medium and 19 Low.

| Severity | Found | Changed here | Left alone |
|---|---|---|---|
| High | 5 | 5 | 0 |
| Medium | 25 | 24 | 1, S2 |
| Low | 19 | 13, two of them partly | 6 |

"Not changed" below gives the reason for each one left. S1 is fixed in these five chapters only; the sites it reaches elsewhere are listed there too. No substantive explanation was shortened or removed.

The largest single fix is one line of API. `Arrow::new` trims 0.25 scene units from each end by default, which turned chapter 08's product arrows into stubs and its chains into rosettes. Every in-scope arrow now passes `.buff(0.0)`.

## How to apply

All patches apply from the repo root with `patch -p1`, against the baselines below.

1. `patches/00-draw-helpers.patch` first. Chapters 06–10 all need it.
2. Any of `06`, `07`, `08`, `09`, `10`, in any order. They touch one file each.
3. `12-mod-lists-NN.patch` only after its chapter, since it names that chapter's new constants. A chapter compiles without its list patch; a list patch does not compile without its chapter. All 120 orders of the five were tested and apply with no fuzz.
4. `11-storyboard.patch` at any point. It changes picture cells only.

**Root edits canonical between syncs**, so compare first: `shasum -a 256 -c checks/baseline-hashes.sha256`. Canonical matched every baseline at the time of this handoff; if a file has moved, `git merge-file` is safer than a patch.

### The shared helper patch and its callers

| `draw.rs` change | Kind | Callers |
|---|---|---|
| `chain_arrows` passes `.buff(0.0)` | one changed line, plus a comment | `loop_sums.rs` four sites; **`residues.rs:137`, chapter 12** |
| `axis_names(kit, stage, &plane)` | new | `one_derivative`, `one_point`, `loop_sums`, `boundary`, `rigidity` |
| `arrow_head_points(tip, from, size)` | new | `one_derivative`, `loop_sums` |

The `chain_arrows` line changes chapter 12's picture too. WhiteSwan's 11–15 repair names this patch as its dependency for that chapter and keeps a one-line fallback (Mails 1747, 1759); apply only one of the two. Nothing else in `draw.rs` changes.

## Hashes

| File | Canonical baseline | Staged |
|---|---|---|
| `src/chapters/draw.rs` | `fa1f5ddbc846dfc5` | `d691b42158063ec6` |
| `src/chapters/one_derivative.rs` | `ebeca97226687616` | `b67231d94cafe4f5` |
| `src/chapters/one_point.rs` | `6422b41859f61055` | `43b6f1c323c5f715` |
| `src/chapters/loop_sums.rs` | `bbf8e4bfbd4c8c37` | `cf383504cdc80652` |
| `src/chapters/boundary.rs` | `03880164adefa31b` | `328d2acbc247eccc` |
| `src/chapters/rigidity.rs` | `c7d53ccac33537bd` | `fbfc96dea9f9f3d6` |
| `src/chapters/mod.rs` | `13f80812713c062c` | `6c844ead2a687da2` |
| `docs/storyboard.md` | `6cc2c04286fed44b` | `c3540e43841609aa` |
| `script.tsv` | `2968b0fd76540cbc` | not staged, unchanged |

Full hashes of everything in this folder are in `HASHES.sha256`. Any applied patch stales the direction map's source hashes, so root rebinds them.

## Changes by finding

### Chapter 06 (`one_derivative.rs`)

| Finding | Severity | Change |
|---|---|---|
| 06-A | High | 06_3 no longer waits 17 s for the square. At "the function moves", four blue steps and their green images appear. `× f′(z₀)` comes at sentence 2. At sentence 3 the green fan stretches for 0.9 s, then turns for 0.9 s. It clears for the square at sentence 4. A test proves the fan ends at exactly f′(z₀)·h |
| 06-B | High | 06_5 shows `z = x + iy` at its start and `f = u + iv` on "Call the output's parts" |
| 06-C | High | 06_8 draws the plane and square at its start. On "Reflect", a green copy flips across a gold axis. `x + iy ↦ x − iy` arrives on "It sends". The prediction card is unchanged, and nothing reveals the answer |
| 06-D | Medium | 06_7 has the symbolic grid and caption at its start, the numbers on sentence 1, and `= × (2 + 2i)`, built from `deriv.value`, on sentence 2 |
| 06-E | Medium | 06_1 names `f(z) = z²`, `h` and "change in the output". As h shrinks, both arrows shrink to a quarter and `→ f′(z₀)` joins the ratio |
| 06-F | Medium | 06_2 titles the inset "the ratio", runs one live `re + im i` reading per approach, says which approach, and draws the target above the trails |
| 06-G | Medium | `z₀` moved clear of the square and spiral. `z₀²` moves aside as the fan arrives. The zoom caption moved off the axis |
| 06-H | Medium | Rows lowered under the coordinates. `u_x` glossed. A bar and "must agree" join the two rows on sentence 4. The two equations arrive on their own sentences, their rates coloured as in the rows |
| 06-I | Medium | The orientation mark is larger, with a solid head, and on the green square it reverses as it flips |
| 06-J | Low | `x + 2iy` shown; "no complex derivative" under the first cross; the square grows tall over 1.2 s; caption brighter |
| 06-K | Low | Partly: the grid and `= × (2 + 2i)` drop trailing zeros through `whole()`, still printed from pinned facts. The `2.000` readout in 06_2 stays |

### Chapter 07 (`one_point.rs`)

| Finding | Severity | Change |
|---|---|---|
| 07-A | Medium | Each line and its ray share a colour (real blue, imaginary green), each named on both planes. **This bends the colour roles** (blue input, green output) for one cue, for root to judge |
| 07-B | Medium | "180°" moved off the vertical axis |
| 07-C | Medium | Carded "0" and "1"; Re and Im on all three planes |
| 07-D | Medium | S1, three arrows |
| 07-E | Medium | Caption now reads `\|g\| ≤ 1 = M`, which defines the formula's M. The string the test checks is still present |
| 07-F | Low | "\|z\|²: one point, no disk" sits beside its dot, brighter |

### Chapter 08 (`loop_sums.rs`, plus the helper)

| Finding | Severity | Change |
|---|---|---|
| 08-A | High | S1: product, step, walk and chain arrows at full length. The circulation marks get a solid head pointing the way they turn, with a test |
| 08-B | Medium | The second lap is laid over the first in its own colour, so the picture keeps moving, and each lap is captioned. A test proves the second lap lies on the first. The lap captions' fades come out of the chain's own time |
| 08-C | Medium | "distance from the start:" clears when the total arrives |
| 08-D | Medium | Re, Im, "unit circle" and a direction head on the ring, cleared before the circle grows in 08_3. Under "As the steps shrink", the drawn step and its product shrink to the finer cut's size |
| 08-G | Low | Partly: the card no longer has a double colon. The upright f in the hypothesis caption stays |

### Chapter 09 (`boundary.rs`)

| Finding | Severity | Change |
|---|---|---|
| 09-A | Medium | The quotient alone on "divide it by"; the split with its first term gold on "Near a"; the second term lit, with its caption in the same colour, on "The rest" |
| 09-B | Medium | Poles ringed on "swallowed"; each share on a dark card on "Each pole pulls" |
| 09-C | Medium | `c_n` reads "average of" with θ defined; panel title and "1"; "the slope at the centre" and "the next coefficient" beside their reads; the ring lights on the last sentence |
| 09-D | Medium | The circle grows from the centre at "the largest disk" and stops at the poles. Graph labels: −1, 1, x, the bump, the recipe. ±i named; "radius 1" with its segment; the pole at i ringed on "no further" |
| 09-E | Medium | The segment from 1 to i and "√2 ≈ 1.414" (from the fact) show as the voice says it. The check rows now say " = the bump" and " (outside: it blows up)" |
| 09-F | Medium | "bump = 1/(1 + z²)", "r = 1/2", "a = 0", and "opposite samples share a value", which explains 8 samples as 4 dots, with a test. The panel is still empty for the cue's first sentence, now with labels beside it |
| 09-G | Low | One ring at a time; "r = 2" |
| 09-I | Low | Plate lifted 0.15 |
| 09-J | Low | The old circle fades instead of collapsing; "1" labels the new centre. Storyboard row changed to match |
| 09-K | Low | `CURVE_STEPS = 920`, so 0 and ±1 are on the grid and the two spikes draw alike, with a test |
| 09-L | Low | "a little beyond" on a card |

### Chapter 10 (`rigidity.rs`)

| Finding | Severity | Change |
|---|---|---|
| 10-A | Medium | Plate named `sin(1/z)`, with axes, Re and Im. Zeros arrive a pair at a time on "one over pi", "one over two pi", "one over three pi", the rest on "and so on". "1/π" marks the first. The question still has no answer on screen |
| 10-B | Medium | Plate named `sin z`, with axes, Re and Im, and 0, ±π marked |
| 10-C | Medium | On "must grow huge", a dot climbs from 0 to 3i over 1.2 s with a live `\|sin z\|`, ending on the existing reading, named 3i. A test ties the climb's end to the fact |
| 10-D | Medium | Re, Im, and "agree here" on the real line. The band growth is unchanged |
| 10-E | Low | Cross arms shortened to 0.12, inside the innermost zero at 0.177, with a test; "0" on a card |
| 10-F | Low | x named; "zero here", and "not zero here" on "yet it is not zero" |
| 10-G | Low | All of U fills after the disks; each disk is 0.04 s shorter to pay for it |
| 10-H | Low | Plate lifted 0.3 |

### On-screen words changed

None of these are spoken.

| Constant | Was | Now |
|---|---|---|
| `one_derivative::ZOOMED` | "zoomed in about 2i to keep the image's size" | "zoomed in about 2i", with `ZOOMED_WHY` on a second line |
| `one_point::PRODUCT` | "…\|g\| ≤ 1: derivative 0 at 0" | "…\|g\| ≤ 1 = M: derivative 0 at 0" |
| `loop_sums::GOURSAT` | "Goursat's lemma: each…" | "by Goursat's lemma each…", the rest the same, 2 characters longer; the 5.0 s window still clears the rule of thumb (7.2 s clean, 6.6 s needed) |
| `boundary::COEF_TEX` | `avg_k`, `θ_k` | "average of", `θ`, as the storyboard already says |

`one_derivative::CR_TEX` no longer appears on screen but stays on the TEX list, so root may drop it. New strings are on the supplement lists through the `12-mod-lists` patches.

### Storyboard

Picture cells changed: 06_3, 06_7, 06_8, 08_2, 09_7, 09_8, 09_9, 10_4, 10_6. Every words column is identical across all 105 rows.

`tools/storyboard.py --check` fails on **canonical and staged alike**, with the same four problems, none from this repair: no prediction pause in chapters 05, 08 and 12, and a stale-script report even though every row's words match `script.tsv`. Root should look at that report separately. The 06_7 row keeps its "morph" feature tag, and nothing there morphs; that was already so before this repair.

## Not changed

| Finding | Why |
|---|---|
| S2 (`CELL`) | Medium, no patch: the review left it as root's call, as it changes the painted plates' grid across the film |
| 06-K, the `2.000` in 06_2 | Judgement call; the facts stay pinned |
| 07-G | Judgement call; both moments follow the words |
| 07-H | Depends on Oliver's timing. Splitting the names card would collide with the precisely card, so recheck under Oliver |
| 08-E | The adding happens in 08_2 |
| 08-F | Recheck once the arrows are visible |
| 08-G, upright f | Cosmetic |
| 09-H | The review proposed nothing; a fair clue, root's call |
| 10-I | Root's call |

**Other `Arrow::new` sites still carry the default buff.** Not in scope and not changed: `closing.rs:95,117`, `two_planes.rs:250`, `continuing.rs:602`, `gallery.rs:402,410,474,497`, `reading_color.rs:39`, `residues.rs:122`, `real_integral.rs:203,207,529,530,536`, `the_bump.rs:371`, `turning.rs:167`. Some are WhiteSwan's to handle; chapters 01–05 have no repair lane that I know of.

## Actual coverage and its limits

**Run, all passing** (`checks/run_checks.sh`):
- Canonical matches the baselines.
- `checks/source_checks.py`: all 106 voice waits name a phrase or sentence that exists, in script order; new constants are used and listed; new picture objects are cleared; TEXT and TEX stay free of duplicates (258 and 147 strings, plus the ten closing captions, which I compared by eye).
- `checks/geometry_checks.py`: Python arithmetic mirroring the new tests, estimated label boxes on 19 scenes, and cards against rings.
- `rustfmt --check` over every module, staged files overlaid on a copy of the crate. This proves the Rust parses, not that it type-checks.
- The patches rebuild the staged tree from canonical byte for byte, with no fuzz.
- `checks/storyboard_compare.py`, in isolated roots.

**Not done, and still open:**
- **Uncompiled.** No `cargo check`, Clippy or test run. I read the signatures of the helpers I call, but a first build may still need small fixes.
- **The 8 new Rust tests have never run**, and neither have the existing ones against these sources. The new tests' arithmetic is mirrored in Python only.
- **Unrendered.** I have seen no frame of this code. Label positions come from estimated text widths, and the estimate has already been wrong once in this work: `carded`'s padding moved nine card placements before delivery.
- **No listening, no final AV, no fine sync.** Every "on the word" above is a source wait, not an observed moment.
- **Not independently accepted**, and these are my own findings repaired by me.

**What to check first in a preview, most likely to be wrong first:**
1. **Arrow updaters: 06_1, 06_3, 08_1.** These call `set_points` with a built `Arrow`'s outline on a filled arrow. Nothing else in this crate does that; the existing updaters use plain polylines. If the fill misbehaves, this is where it shows.
2. **The chain's thin arrows: 08_3 and 08_4.** Computed at 5.4 and 3.2 px of shaft at 4K. The 08_4 row cannot grow, because z⁻¹'s column sets the common scale.
3. **The lit terms: 09_1.** The colour needles are whole `\frac{…}` terms. If `select` returns nothing, the split shows unlit rather than failing.
4. **Colour pairing: 07_1**, as flagged at 07-A.
5. **Overruns:** a cue that runs long pushes the next one. The tightest fits by word count are 06_3 sentence 3 (2.1 s of animation in about 4 s of voice) and 10_6's climb (1.7 s in about 1.9 s), which ends the chapter and would only delay the music breath. Recheck under Oliver.

## Author and reviewer overlap

LavenderPike wrote no source in this film before this repair; the chapters are from BronzeJaguar's lane and integration is root's. **I wrote the review these repairs answer**, so the same judgement chose both the defects and the fixes. A reader should test the fixes against the frames rather than against my findings. The lanes also share a model family and a memory directory.
