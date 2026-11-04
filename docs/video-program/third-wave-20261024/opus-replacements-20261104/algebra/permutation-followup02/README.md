# Chapter 11 follow-up

Bead `mx-video-program-program-87do.64`, answering Mail 1253. Author LavenderPike, for root BlueSalmon.

**Status: source only.** Not built with Cargo or fmn, not rendered, not heard. Nothing is applied to canonical. No Oliver take exists for any cue here, and I synthesized nothing. `permutation-repair01` and the 17, 13 and 10 patches stay frozen, and their hashes are unchanged.

## Base and bytes

Root has not integrated repair01 (canonical `src/permutations.rs` is still `1301e02d188cbd75…`), so this is cut against the repair01 candidate. Copies are in `base/`.

| Base file | sha256 |
|---|---|
| `src/permutations.rs` | `8b234eaf1f7f819a2416b6d3148e91b000df8deb375f3574b52cc4c3df0bf988` |
| `src/main.rs` | `82ccd22e009feff3e6d2eb84a30f05b1b3eda3397e03603ca231e50389722d41` |
| `oracle.py` | `fbb1957b74f83cab383d2067bbc81b18d3f8516f17346ad2693dd666d6488ad8` |
| `script.tsv` | `021fc1053a27462b444790f80c93aa90fb0733b128817391d9aada270fedfb34` |

| Staged file | sha256 |
|---|---|
| `chapter11-followup.patch` (2,944 lines) | `cbfc848eefe6d20dcf9ecd49a295692ec040545352dcef35b7a7b74e5a4325dc` |
| `candidate/…/src/permutations.rs` (4,712 lines) | `de3c7b10d96e856ab218d07463a33aa57f97acd41188b1dc267556cfd28e3c68` |
| `candidate/…/src/main.rs` (unchanged from base) | `82ccd22e009feff3e6d2eb84a30f05b1b3eda3397e03603ca231e50389722d41` |
| `candidate/…/oracle.py` (181 lines of output) | `2832a8c1a4069da00c37f78d3a746149511ec28309227efed2390882fb72af68` |
| `candidate/…/script.tsv` | `682e06b5e21407c25429f0329b6843a98f0e8935b6600b5bbed1907a4f4e18cf` |
| `storyboard-chapter11.md` | `ba65dd00a25ba3a94947b055dee5c69180b932bd3bf18207cd60e4f160e21033` |
| `oliver-cues-chapter11.tsv` | `922f500f28076fa14211d72dbe7c89988729fc2a7d15aa010c3b901ee27c7a9e` |
| `script-chapter11-repair01.tsv` (preserved rows) | `159b0b3c467c6df5d98e20e42004a598669452c525f6fa2fb73c010549e8bb34` |

## Applying it

From the repository root, after repair01: `patch -p1 --fuzz=0 -i <this folder>/chapter11-followup.patch`.

`checks/patch_round_trip.py` ran these on fresh copies of canonical:

| Order | Result |
|---|---|
| Base copies, this, then reversed | Equals the candidate, then equals the base |
| Canonical, repair01, this | Equals the candidate in all four files, no offsets |
| Repair01, 17, 13, 10, this | Fuzz 0, one hunk offset (oracle); oracle prints 240 lines, no key twice |
| Repair01, this, 17, 13, 10 | Same bytes as the row above, every file |
| 17, 13, 10, repair01, this | Same bytes as the row above, every file |

## Changed cues

`oliver-cues-chapter11.tsv` lists all 27 cues in script order with full text and full hashes. Chapter 11 is now 1,744 words. The longest cue is 95 words.

| Cue | Status | Words | Text sha256 | Job |
|---|---|---|---|---|
| 11_12 | new | 61 | `f581f713fb981383…` | What "return" means for a whole row; predict |
| 11_13 | new | 72 | `80ef1a20e084b894…` | Run the cards six times |
| 11_14 | new | 66 | `c8587baabef66d40…` | Why six: common multiples, lcm; card 4; the sum fails |
| 11_15 | new | 34 | `9884e0c739054f6e…` | Second example; predict |
| 11_16 | new | 53 | `d20229e9da51707b…` | Four, not eight and not six |
| 11_17 | new | 64 | `4f0b5c94f480e934…` | Overlap breaks the rule, on 11_3's own pair |
| 11_8 | changed | 89 | `6e5fc793733e006a…` | Your sign-rule sentence, appended as sentence 4 |
| 11_18 | new | 56 | `e8ba201195da1eab…` | Swap counts add, on cards |
| 11_19 | new | 41 | `dace5ecc660ec5bb…` | Odd plus odd; signs multiply |
| 11_20 | new | 95 | `8769253d94d62bc8…` | The evens pass the film's own subgroup test |
| 11_21 | new | 88 | `aa8b0f6b0eac63af…` | Twelve of twenty-four, which 14_10 relies on |
| 11_10 | changed | 78 | `81d4c011d7a83c83…` | The application: one fixed shuffle; predict |
| 11_22 | new | 67 | `5d94581cb73687d8…` | Read its cycles from positions |
| 11_23 | new | 46 | `e39665918fc46977…` | Shuffle for real: six, as predicted |
| 11_24 | new | 52 | `e9381c0d7ecbefb7…` | Fifty-two cards return in eight |
| 11_25 | new | 55 | `91a23e5562fbe0c3…` | The in-shuffle takes fifty-two |
| 11_26 | new | 64 | `0e17431aa00444ed…` | The closed loop; eight wires return in three stages |
| 11_27 | new | 92 | `aa4141c6a396bf04…` | Undo, the limit of the model, one slip, next chapter |

Unchanged text: 11_1 to 11_7, 11_11, 11_9. Script order: 11_1 to 11_4, 11_12 to 11_17, 11_5, 11_6, 11_7, 11_11, 11_8, 11_18 to 11_21, 11_9, 11_10, 11_22 to 11_27.

**Takes.** A `find narration_lab -maxdepth 5` for `11_12`–`11_27` file names found nothing, and no chapter 11 algebra take is newer than today's start. Reworded 11_8 and 11_10 need new takes; nothing should overwrite any older take of those ids. Please confirm against your ledger, which I did not read.

## What I decided, and why

**The sign rule took more than a sentence.** Your sentence is in 11_8 as written. On its own it would assert a subgroup the film had not shown. So 11_18 and 11_19 run it on cards, 11_20 uses the subgroup test from 08_4 (undo b, then do a), and 11_21 gives the count 14_10 needs by pairing each even arrangement with an odd one. If you want only the sentence, drop 11_18 to 11_21; that needs a new patch from me, because 11_9's fades chain from them.

**Order.** 11_12 states the strict meaning of return: every card in its own slot at the same moment. 11_13 runs the cards, 11_14 explains the lcm from the strips, 11_15 and 11_16 show a case where the product and the sum both fail, and 11_17 shows the rule breaking on overlapping cycles, using 11_3's pair (lengths suggest 6, the cards return in 2). The clock in 10_3 is the callback; lcm is defined plainly here, and 15_4 and 15_5 teach it later without conflict.

**Shuffles rather than a check digit.** The treatment suggests a check-digit or coding example, but chapter 17 already does ISBN-10. Shuffles use exactly this chapter's tools, and the result surprises: ten cards return in 6, a full deck in 8, and a nearly identical motion takes 52. The honest limit in the old 11_10 survives in 11_27, and now it earns its keep: a slipped neighbour is one extra swap, so eight shuffles end on an odd permutation and the deck cannot be home. That uses parity and the sign rule together.

**Three smaller changes to repair01 material.** The ring radius in 11_4 went from 0.44 to 0.37 in two places, because the bigger ring cut through slot label 3. `THE_POSITIONS`, `CYCLE_134_ON_134`, `INVERSE_134_ON_134` and their test are gone with the old 11_10. `six_row` now delegates to `painted_row`, with the same output for existing callers.

The ten cards stay one colour until 11_22 s4, because colouring them by cycle during the 11_10 prediction gave away part of the answer.

## Checks

Each output is saved next to its script in `checks/`. All were run on the final bytes above.

| Check | Result | What it cannot tell you |
|---|---|---|
| `pure_rust_arithmetic.py` | Bare `rustc`: 11 tests pass; all 33 new Rust facts equal the independent oracle | Only the pure section compiles here |
| `run_against_stub.py` | The whole of `permutations.rs` compiles with no warnings and passes `clippy-driver -D warnings` against hand-written stand-ins for fmn and the Kit; 31 tests pass; `construct` runs to the end with 0 object-ledger problems | A real signature that differs from my stand-in, any glyph, any frame |
| `stub_sensitivity.py` | 8 deliberate breakages, 8 caught, each by the expected layer | Faults outside those layers |
| `script_and_captions.py` | 159 rows outside the chapter equal the base; 27 storyboard cues rejoin to `script.tsv` byte for byte; 8 literal captions agree with the oracle; 3 prediction scenes draw no answer before the hold | Whether the picture column matches the frames |
| `patch_round_trip.py` | See "Applying it" | |

The oracle computes differently from the Rust: orders by running the map, parity by counting inversions, shuffles by dealing a real list and again by modular arithmetic. It also asserts the edge claims: order equals lcm for every permutation up to 7 labels, the evens pass the test at sizes 3 to 6, the first-two pairing at sizes 2 to 6, and every neighbouring slip in every one of the eight rounds is odd and never home.

The overlap lint reports one box overlap: slot label 3 against the 11_4 ring, by 0.014 on an em box. The digit's ink is shorter than the box, so I left it.

**The clock is an estimate.** With no takes, the stand-in assumes words divided by a speaking rate, with sentences placed by share of characters.

| Rate | Chapter length |
|---|---|
| 2.4 words/s | 752 s |
| 2.68 words/s | 677 s (11.3 min) |
| 3.0 words/s | 608 s |

The treatment's target is 8 to 12 minutes. No new beat starts more than 1.0 s after its sentence at 2.68. The `latest_beat` figures on 11_1, 11_11 and 11_9 are larger only because their phrase anchors count from the start of the sentence.

## Decisions for you

1. **The raw cap, before synthesis.** `crates/explainer_kit/src/encode.rs:47` sets `MAX_OUTPUT_BYTES` to 256 GiB, about 368 s of 4K60. This chapter estimates near 677 s, and its voice alone is about 651 s at 2.68 words/s. By the same arithmetic, chapter 16's 1,050 words (about 392 s) are over the cap too. Either raise the constant or split the scene. A split binds cue ids to a new chapter key (`cli.rs:300,574`), so it must be settled before takes. I did not cut content to fit.
2. **Direction map.** I did not edit it. New cues need unit-plan rows. Prediction holds need full-text `after` entries: `k.hold` is at `src/permutations.rs:4082` (11_12), `:4094` (11_15), `:4160` (11_6), `:4217` (11_10), each followed by `stage.wait(3.0)`. `:4020` (11_2) and `:4070` (11_3) keep their 1.0 s.
3. **One claim to review.** 11_26 s2, "Designers of parallel computers have used this same shuffle to wire processors together." I confirmed Stone's 1971 paper exists from its bibliographic record but did not read its full text, and that paper is a proposal. "Have used" also rests on my recollection of later shuffle-exchange and Omega networks. The arithmetic after it is verified. If a reviewer cannot support the sentence, it can be softened or the wires cut.

## Strongest unresolved teaching defects

1. **Pacing is unknown, and the chapter more than doubled.** It grows from 714 words to 1,744, about 6.4 more minutes at 2.68 words/s, and four of the new cues are proof. That lands inside the treatment's target, which the 714-word version sat well below, but nobody has heard it.
2. **11_26's wires are the weakest part.** They are a true fact with a picture, not a worked result. If the chapter must shrink, cut them first.
3. **The 52-card cycle strip can mislead.** Its segments are sorted by length and sit under the bars, so a viewer may read a segment as "those bars form that cycle". The caption gives only counts.
4. **11_22 reads positions while cards are displaced.** It is deliberate: after one shuffle card 2 really sits in slot 3, so the row shows the map. But card numbers and slot numbers are both visible, and that may confuse.
5. **11_9 s1 now repeats "form a group"** after 11_20 said it. I kept 11_9's text unchanged rather than widen the rewording.
6. **52 gliding bars at 0.22 to 0.28 s per shuffle** may read as noise rather than as a staircase breaking. Only frames can settle this.

## Not verified

- Anything against real fmn or the real Kit. The stand-ins were copied by hand from reading them, so any signature I misread will pass here and fail for you. Repair01, this file's base, has never been built either.
- Glyphs and TeX: `\text` with hyphens and commas, `(+)(-)=-`, a formula starting with `=`, and `52!`. I removed `\approx` because I found no use of it in the corpus.
- Layout beyond rough text boxes. The tightest screens are the 11_19 table, the 11_21 grid, the wires and the bar captions. Arrows and moving cards are not linted.
- Render cost of 52 mobs gliding, and the raw cap above.
- Timing, listening, fine sync and independent AV review. All are open, and ElevenLabs Oliver takes do not exist yet.
- The other chapters' cross-references, beyond the ones cited above.

## Sources for the shuffle claims

- Harold S. Stone, "Parallel Processing with the Perfect Shuffle," IEEE Trans. Computers C-20(2), 153–161, 1971, DOI 10.1109/T-C.1971.223205: [dblp record](https://dblp.org/rec/journals/tc/Stone71). Full text not read.
- Eight out-shuffles and fifty-two in-shuffles, cross-checked against my own computation: [HMC Math Fun Facts](https://math.hmc.edu/funfacts/?p=339), [John D. Cook](https://www.johndcook.com/blog/2026/01/01/in-shuffle-out-shuffle/), [Susam Pal](https://susam.net/from-out-shuffles-to-multiplicative-order.html).
