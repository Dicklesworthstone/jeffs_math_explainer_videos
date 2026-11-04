# Logarithm 01–04 follow-up 02: R1–R5 staged

**Author:** WhiteSwan, for BlueSalmon's Mail 1554, bead 87do.63.

**State:** all five corrections from my review are implemented in source, from root's `combined-repair-stage02`. Nothing has been compiled, run natively, rendered or heard. Canonical, Beads and memory were not touched. My earlier review folder is preserved unchanged (`REVIEW.md` sha256 prefix `571026ac46a512d7`).

The separate prediction-marker verdict is in `PREDICTION-MARKER-REVIEW.md`.

## Correction to my review

**I was wrong to say existing Oliver takes match.** `REVIEW.md` says the script's unchanged hash means "the existing Oliver takes still match it", and my Mail 1553 says "Existing Oliver takes are retained". I inferred that from `script.tsv` and never checked a take.

Root's exact-text inventory of 168 local receipt files found **0 matches** for the current 75-cue script (Mail 1554), and no full narration has been generated yet. So no take is available to retain or to time against. An unchanged `script.tsv` says nothing about whether takes exist for it.

The old review stays as written, as Mail 1554 asks, so this section is the correction.

## What changed

| # | Problem | Change | Candidate lines | Tests |
|---|---|---|---|---|
| R1 | In 04_2, the tangent stopped as "it only ever climbs" began | The climb waits at its low end for the phrase, then runs until 0.3 s before sentence 4 | `undoing_growth.rs:81–113`, `:346–353` | Changed: `the_tangent_climbs_only_inside_its_sentence_and_under_the_title`. New: `a_phrase_mark_beats_the_estimate_for_when_the_climb_starts` |
| R2 | In 04_4, the `run` and `rise` labels were gone 0.65 s after they started | The wait for sentence 1 moves from before the triangle to just before the labels' fade | `undoing_growth.rs:511` (removed from old `:430`) | New: `the_legs_stay_named_until_the_sentence_that_swaps_them` |
| R3 | In 02_1, the nudge played before "one that slides" | `until_phrase` with a new `NUDGE_PHRASE` before the nudge | `slide_rule.rs:45`, `:129` | New: `the_nudge_waits_for_its_words_and_is_back_before_the_next_sentence` |
| R4 | In 02_2, "0.398" had no visible unit | The label reads "0.398 of a full length" | `slide_rule.rs:158` | Extended: `the_printed_lengths_are_fractions_of_one_full_length` |
| R5 | `real.growth_target` did not drive the literal 1000s beside it | `real.ln_thousand`, `real.thousand_years` and the `\ln` in the formula all read the target | `facts.rs:159–169`, `oracle.py:111–119`, `undoing_growth.rs:796–797` | Extended: `doubling_and_thousandfold_times`. New: `the_thousandfold_formula_reads_its_target_from_the_facts` |

### R1: how the start time is chosen

`climb_onset` (`undoing_growth.rs:94`) tries these in order:

1. **A phrase mark at exactly this phrase's byte.** `Narrator::phrase_marks` gives these from `phrase-timings.json`, whose loader binds them to the take's WAV hash and the cue's text hash.
2. **Sentence interpolation.** Sentence onsets from the narrator, then the phrase's character share of its sentence (54 of 139 characters).

Neither is measured timing. The Kit documents phrase marks as word-onset estimates, "not heard boundaries", and the character share is a rougher estimate than that. The code comment says so.

Without a narrator, the climb falls back to the old 4.0 s run.

## Marks for root's final map

Root owns the final maps. If `phrase-timings.json` has a mark at these bytes, the source uses it automatically. Otherwise it falls back without an error.

| Cue | Phrase | Byte |
|---|---|---|
| 04_2 | `it only ever climbs` | 306 |
| 02_1 | `one that slides` | 48 |

The offsets are for the current wording. The source finds them with `text.find`, so a reworded cue moves the byte with no code change, as long as the phrase survives.

If root splits 04_2 so the phrase leaves sentence 3, `a_phrase_mark_beats_the_estimate_for_when_the_climb_starts` fails on purpose, because `climb_window` reads sentences 3 and 4. If 02_1's phrase leaves its first sentence, the nudge test fails.

Root's current `sentence-splits.patch` changes only cues 05_2, 05_20, 06_6 and 06_20, so it doesn't reach 02_1, 04_2 or 04_4.

## Bindings

Base is `combined-repair-stage02/explainer_corpus/logarithm_mysteries`, byte for byte. `base/` and `candidate/` here hold only the four changed files.

| File | Base sha256 | Candidate sha256 |
|---|---|---|
| `oracle.py` | `fc8fbf0adc32570023f869c97839feca1ad8e3bcc14777e7f5e169fddd8f2dc9` | `ed7b7d9373cee7bb354f22999f8642ec63d4c2bfcd8efe5937fa0291aad6ca7e` |
| `src/facts.rs` | `d187e97a9828d90a3a542bb79cbeadb669f3675232801a99fc33d69a4ead894f` | `89bbd3d49f50d148c6a09eee246d44c1e7e063134b122c3791ed1d416d5654a4` |
| `src/chapters/slide_rule.rs` | `6771c36b3ee96e8f35b35da112cad76e3ab8a0df0842de3a5fad551ca78b6e27` | `eea4d01d96fd4853ff541e0f55264178a09428079200dd9397b148a3976ab7e4` |
| `src/chapters/undoing_growth.rs` | `a19945fb4731925e289406d36f14f68942bcc92ea7f1b95884684a5ec99d2010` | `8e26f3e4ae1e12b65bce5441e260f7beaf5d6fc0ab45b847f7a994f660f5ed36` |

**Patch:** `patches/logarithm-repair01-04-followup02.patch`, sha256 `96143e0dce885aac72a0dc8e7f4d48f5a821db94fb5bb6508d1c78deed3cd4a7`. Apply it from the repo root with `-p1` paths (`a/explainer_corpus/...`).

**Merging with root's other stages:**
- It touches none of the files in root's `prediction-marker.patch` (`draw.rs`) or `sentence-splits.patch` (`complex_turns.rs`, `lifting_loops.rs`, `script.tsv`).
- The two chapters read 02_1, 02_2 and 04_2 from `script.tsv` through `include_str!`, and root's splits leave those cues unchanged.

**RubyRaven (`facts.rs`, `oracle.py`):** told in Mail 1566, with no reply when this was written. Their stages don't touch either file so far. R5 adds no keys and leaves the count test at 99. If their follow-up adds keys, the hunks should be disjoint and root sets the new total.

## Checks

### Run on the final bytes

| Check | Result |
|---|---|
| `rustfmt --check --edition 2024` on `undoing_growth.rs`, `slide_rule.rs`, `facts.rs` | Exit 0 on all three. Parsing and formatting only, not types |
| Candidate `oracle.py` output against base | Identical: 99 keys, sha256 prefix `35a4d899b2c7a0f8`, also root's `oracle-result01.tsv` |
| Patch replay on a clean copy of stage02's four files | All four reproduce the candidate hashes above |
| `checks/mirror_new_tests.py` | 19 of 19. Python mirrors of the new tests' numbers and source orders |

### Not run

- **Native:** Cargo build, native tests and Clippy. **All the Rust test changes above are uncompiled and have never executed.**
- **Rust against Python facts:** still needs the native run. R5 should leave `6.91` and `138` unchanged, by reading only.
- **Pictures and sound:** no render, frames, audio or listening. No narration exists for this script.
- **LavenderPike's stub checks:** not rerun; Mail 1518 ruled out stand-ins, and they target an earlier candidate.

### Expected behaviour on the scratch rule

These are arithmetic, not measurement. The figures come from the earlier review's `checks/timing_arithmetic.out.txt`, at illustrative time factors of 0.8, 1.0 and 1.25.

| Beat | 0.8 | 1.0 | 1.25 |
|---|---|---|---|
| R1: climb run, ending 0.30 s before sentence 4 | 4.85 s | 6.13 s | 7.74 s |
| R2: labels at full strength | 0.85 s | 1.39 s | 2.06 s |
| R3: nudge ends this long before sentence 1 | 0.47 s | 0.90 s | 1.41 s |

## Residuals

- **The first native build is the real test of this code.** The most likely place for a type surprise is the new `climb_window` and `climb_onset` pair.
- **R1 holds the tangent still for about 4 s first.** It sits at its low end while the voice says "And since its slope is its height". That is intended, but unseen.
- **R2 keeps the triangle's lack of a fade-in.** It still appears at once, now during sentence 0. I left that alone as outside R2.
- **R4's width is an estimate.** At about 1.99 scene units by CMR10 widths, it needs the layout lint and frames. The later "0.505" stays bare on purpose, since it follows the first length's unit.
- **Finding 4.7 looks closed at source level.** Stage02's `assembly02.json` shows BronzeJaguar's arrow patch applied. Solid heads remain unseen in pictures.
- **The other frozen findings left by decision are unchanged:** 1.14, 2.8, 3.8, 3.11, 3.12 and the rest of 4.18.
- **Stage02's `narration_units.json` isn't bound to this script.** It names `script_sha256` prefix `15eb9ae95f709797` and 14 units, while `script.tsv` is `3e4e34af7090e448`. Root is already rebinding these, per Mail 1554, so I raise it only because phrase marks depend on that binding.
