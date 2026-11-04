# Chapter 11 teaching repair

Bead `mx-video-program-program-87do.64`, Mail 1096. Author LavenderPike, for root BlueSalmon.

**Status: source only.** Not compiled, not rendered, not heard. Nothing here is applied to canonical. No Oliver take exists for it. This is not listening or AV acceptance, and no check below stands in for one.

## Bytes

Base, the exact canonical files this is cut against (`cmp` equal at 08:05:26Z, copies in `base/`):

| File | sha256 |
|---|---|
| `src/permutations.rs` | `1301e02d188cbd7552c86ae7518f65bad09923457d19d1308cd656e7759b47f8` |
| `src/main.rs` | `ba8d53ce96a2a1a61a147e4121df970ac684ad48f0866e6dfad19a382d77f704` |
| `oracle.py` | `528ca6a6bf869b465bfc338d7564be3f4720d4d29a2c7c05a93260c0b6f577ca` |
| `script.tsv` | `e5ba1c3e83692e6128aca93591d9f0b27482bd267402bd7b5b493f3428559261` |

Read but not changed: `src/axioms.rs` `d204a79b24946d79…`, `src/d4_group.rs` `1fc767e951b31ab6…`. Review read: `permutation-cross-review01.md` `6c34eeac00788bd2…`.

Staged:

| File | sha256 |
|---|---|
| `chapter11-teaching-repair.patch` (2,504 lines) | `8e2fdb7182b1a8f11003d178bfe237424a510c84f3ea2c5c6b28659173fdb547` |
| `candidate/…/src/permutations.rs` (2,278 lines) | `8b234eaf1f7f819a2416b6d3148e91b000df8deb375f3574b52cc4c3df0bf988` |
| `candidate/…/src/main.rs` (one added line) | `82ccd22e009feff3e6d2eb84a30f05b1b3eda3397e03603ca231e50389722d41` |
| `candidate/…/oracle.py` | `fbb1957b74f83cab383d2067bbc81b18d3f8516f17346ad2693dd666d6488ad8` |
| `candidate/…/script.tsv` | `021fc1053a27462b444790f80c93aa90fb0733b128817391d9aada270fedfb34` |
| `script-chapter11-original.tsv` | `cbd920e82f670231b9e31e254fe4676802dddfb8b616c651e48c8a94cf86fa07` |
| `oliver-cues-chapter11.tsv` | `fa81bca2a2d282c2319a6297c82eb51fcaa80b96b24efcb7fab1c8f841839a55` |
| `checks/mirror_rust_helpers.py` / `.out` | `ff754343…` / `e90712e8…` |
| `checks/timing_budget.py` / `.out` | `f9a75c0b…` / `949c328f…` |

The frozen 17/13/10 overlays are untouched: `patches/` still hashes `0d12e29c`, `6290c6da`, `f9c2b58a`, and nothing under `candidate/`, `patches/` or the parent `README.md` has changed since before this assignment began.

## Applying it

From the repository root: `patch -p1 --fuzz=0 -i <this folder>/chapter11-teaching-repair.patch`.

| Tree | Result |
|---|---|
| Exact canonical | Applies; all four files byte-identical to `candidate/`. `patch -R` returns canonical byte for byte. |
| Canonical + 17 + 13 + 10, then this | Applies at fuzz 0; the only offset is the oracle hunk, by 98 lines. |
| Canonical, this, then 17, 13, 10 | Applies at fuzz 0. Same four files, byte for byte, as the row above. |

On the full chain, `main.rs` is `ac8323e61d4d7722…`, `oracle.py` `36cd83364c44d37c…` and `script.tsv` `765be906bfad3f73…`. The chain oracle prints 207 lines (184 + 23); canonical + this prints 148. The `permutations.*` lines are the same on both (sha256 of those lines `22be8e5829330ef6…`).

The candidate has local copies of `line_text`, `trimmed`, `sentence_ranges`, `phrase_time`, `until_phrase`, `pulse`, `recolor` and `morph`, because canonical `axioms.rs` keeps them private and only my frozen 13 patch opens them. You can dedupe after integrating 13. Nothing requires it.

## Findings

| Finding | Cue | What the source now does |
|---|---|---|
| P0-1 | 11_1 | s3 follows card one to its new position, then the card it displaced. A pulse on card 1 at s3. |
| P0-2 | 11_2 | s4 fades the forward arrows and `same`, then draws the hero arrows reversed with real curved `links`. Caption `(1 3 4): 1↦3  (1 4 3): 1↦4`; dim note that a two-label swap is its own reverse. |
| P1-3 | 11_9 | `square_as_labels`: a labelled square. Cards 1 and 2 trade corners, the outline morphs to a bow-tie, both torn edges go red. Quarter-turn and half-turn are real card moves with `r=(1 4 3 2)=(1 2)(1 3)(1 4)` and `r^2=(1 3)(2 4)`. `rs=(2 4)` and `sr=(1 3)` run on two squares with a ring following 2→1→4 and 1→4→3. Tile thumbnails for `R.mul(S)` and `S.mul(R)` close the callback. |
| P1-4 | 11_7, 11_11 | `witness_proof`: chips on x = (1,2,4,8), six differences, tally 1008. Swapping slots 1 and 3 shows all four cases: in-place reversal (red), trade (blue), trade and both reverse (gold), untouched (dimmed). Tally flips to −1008. Caption `one swap: Δ↦−Δ, |Δ|=1008`. |
| P1-5 | 11_8 | `two_routes`: three swaps and one swap reach the same arrangement. Lamps flip the sign at each swap. Both end at −1008 and both odd; "2 swaps would give +1008: impossible". Rule `Δ_final=(-1)^NΔ_initial`. |
| P1-6 | 11_4 | `undo_and_disjoint`: cards run forward then reversed and come home. Two six-slot rows run both orders of `(1 3 5)` and `(2 6)`. Ring walks the cycle, a red arrow 5→3 shows the collision, the loop closes, "4 stays put", restated as `(2 6)(3 5 1)`. |
| P1-7 | 11_9 | s1 says all twenty-four form a group with the eight inside as a subgroup. Caption `4×3×2×1=24: the group S_4`, 24 cells, eight gold, "8 inside 24: a subgroup". |
| P1-8 | 11_10 | 11_9's picture leaves at cue start. Own board, three captions synced to s0, s2 and s3, undo run at s1. |
| P2-9 | 11_3, 11_5 | Card colour keyed to the label throughout. |
| P2-10 | 11_6, 11_7 | Second description morphs in at s0, question at s1, hold + 3.0 s as the last beat. Removed clause opens 11_7. |
| P2-11 | 11_1 | `collisions`: legal arrows, then a red bad map with a ring and "collision", gold ring and "missing". |
| P2-12 | 11_3 | Reverse uses `all_routes` (about 2.2 s, all cards at once) then hold + 1.0 s. Estimated silence after 11_3 is 1.6–2.5 s against the review's 8.9 s. Recheck on real takes. |

## Script

Only chapter 11 rows change; every other row is byte-identical (`write_cue_lists` asserted this). `script-chapter11-original.tsv` preserves the ten original rows. `oliver-cues-chapter11.tsv` lists all eleven with status, counts, both text hashes and the new text.

| Cue | Status | Words | Change |
|---|---|---|---|
| 11_1 | changed | 52 | s3 only (P0-1) |
| 11_6 | changed | 31 | Last clause removed (P2-10) |
| 11_7 | changed | 53 | Opens with the moved clause; ends at "i less than j." |
| 11_11 | **new** | 53 | The rest of the argument, with the P1-4 wording and the factors-avoiding-both sentence |
| 11_9 | changed | 91 | s1 (P1-7), s3 split in two (P1-3) |
| 11_2, 11_3, 11_4, 11_5, 11_8, 11_10 | unchanged | 64, 75, 86, 66, 70, 73 | None |

**Why 11_11 exists.** With the review's additions 11_7 reached 106 words, over the contract in `tools/coherent_narration.py:176` that stopped `15_10` earlier (`TODO.md:253`). It follows your split convention: next free number, script row in call order, so the rows read `11_7 11_11 11_8`, like `15_10 15_12 15_11`. `k.say("11_11")` is inside `witness_proof`, and playback order matches row order. No sentence was reworded by the split.

## Facts and oracle

`permutations::add_facts` registers 23 `permutations.*` keys, called from `facts()` just before `rebuild_facts`, which is the chain-safe spot. `oracle.py` gets a new section computing the same 23 from the definitions, under `perm_*` names. It shares no code with the Rust.

The oracle also asserts these edges for every always/never in the script:

- A reversed cycle equals the original only for m = 2 (m up to 7), which is what "usually" means in 11_2.
- Every trail on every permutation up to 6 labels first repeats at its start.
- m − 1 swaps build an m-cycle, up to 7.
- At sizes 2–6, every swap negates Δ, the factor between the swapped labels reverses, factors avoiding both are unchanged and pair products are kept. A pair reverses together exactly when the third label lies between.
- Every swap word up to length 6 on the witness gives 1008·(−1)^length.
- S4 is closed with inverses; the kept eight are closed with the identity.
- Every factorisation on screen.

The Rust module has 21 tests, 2 canonical and 19 new: sentence counts and the 100-word limit, anchors, maps, cycle notation, undo, disjoint rows, overlap, descriptions, witness chips, tallies, swap words, routes, square cycles, rings, 24/8, torn edges.

## Checks actually run, and what they do not prove

| Check | Result | What it does not prove |
|---|---|---|
| `rustfmt --check` on `main.rs` over a full overlay, and on the file | exit 0 | Parsing only. No name resolution, types or borrows. |
| `checks/mirror_rust_helpers.py` | 79 assertions; 15 anchors in their sentences; 36 `until_sentence` indexes in range; say order equals row order; 23 facts agree with the oracle; key set read from the Rust source matches | It runs my Python transcription of the pure helpers, not the Rust. The 19 new Rust tests have never run. |
| `oracle.py` | Runs; edge asserts pass | Same author as the Rust, so independent in code only. |
| `checks/timing_budget.py` | Only flag: 11_5 count caption 1.1–2.2 s late. Chapter about 5.5 / 4.7 / 4.1 min at 2.2 / 2.6 / 3.0 words/s | An estimate from assumed speaking rates and hand-read beat durations. |
| Patch trees above | As tabled | Text application only. |

I ran no `cargo`, `rustc`, render, TTS, ASR or Beads command, and looked at no frame.

## For root's native checks

1. Build, tests and Clippy. Most likely to fail: the first corpus uses of `Arrow::path_arc` and `AnimateArgs.path_arc` (I read both in fmn at pin `a399d62`, but have never compiled them), and any borrow or type slip that a desk read misses in about 2,000 uncompiled lines.
2. Facts check against the oracle.
3. Lint and sheets. Look first at the tightest screens: 11_7/11_11 (six chips, four slots, tally, product at y 3.05) and 11_9 s1 (24 cells, eight thumbnails, three captions). No coordinate here has been seen rendered.
4. Motion to look at: the outline bow-tie morph, the `path_arc` card moves on the square (a quarter-turn should rotate about the square's centre), the tex morphs in 11_2, 11_4, 11_6.

## Narration: ElevenLabs Oliver only

Every chapter 11 cue needs a take. I found none in `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103` (no `11_N*` at depth 3), in `five-film-release-20261027` (only `expanded-algebra-13-11-*` and `06-11-*`), or in the inventory `six-machine-screened-production-20261103/expanded-algebra-current-narration-inventory02.json` (text only). Other lanes were not searched exhaustively. On that evidence the rewording invalidates no existing take; please confirm from your own ledger.

- **Text changed or new:** 11_1, 11_6, 11_7, 11_9, 11_11.
- **Text unchanged but apparently missing:** 11_2, 11_3, 11_4, 11_5, 11_8, 11_10.
- **Unit plan:** `read_units` requires every cue in script order, so 11_11 needs its own unit between 11_7 and 11_8.
- **Following context:** `request_text` caps target + next cue at 100 words. Only 11_5+11_6 (97) and 11_6+11_7 (84) fit. 11_7+11_11 is 106, the same over-limit case as most of the chapter (10_11+11_1 is 114, 11_10+12_1 is 125). If that tool governs your Oliver lane, those cues go without following context.

**Direction map.** I did not edit it. `narration-direction-map-oliver22.json` already pins stale bytes (the review says so). For the next map:

- **11_6 `after`:** "There may be many different swap descriptions of the same permutation. Can one description use an even number of swaps and another use an odd number? Predict before we prove it."
- **11_6 prediction hold:** `next_cue` 11_7 stays. `src/permutations.rs` `8b234eaf…`, line 1934, marker `k.hold(stage)?;`.
- **New visual holds, unregistered:** line 1821 (end of 11_2) and line 1871 (end of 11_3), each hold + 1.0 s. Canonical had a 1.5 s hold at its line 299, which line 1871 replaces.
- **11_11** has no map entry and needs none that I know of; it carries no hold.

Fine sync, listening and independent AV review are all open.

## Where this departs from the review's sketches

- 11_9 s3 is two sentences, so each swap gets its own anchor.
- 11_7 is split into 11_7 and 11_11, which the review did not anticipate.
- 11_10's undo uses `all_routes` (2.2 s) instead of three traces (8.7 s), with three synced captions.
- `cycle_trace` shortened from 8.0 to 5.9 s and the 11_5 traces run at pace 0.6, from my timing estimate rather than the review. If real takes run slow, restore them.
- 24-cell grid with the eight highlighted, and tile thumbnails for the callback, both from the treatment-coverage table.
- `product` moved to y 3.05 at size 36 to clear the chips. The sign caption carries `|Δ|=1008`.
- Red torn edges are both 2–3 and 4–1; the review named only 2–3, but the bow-tie has two diagonals.
- **Glyph lists not edited.** The review asked for them, but Mail 780 says the live builder captures all Kit strings. The new module typesets only through Kit (no raw `Tex::new` or `Text::new`). If that is wrong for this lane, the failure will be loud at build.

## Left out as root or owner decisions

From the review's decisions list, none implemented and no fact added for them:

1. Order as lcm on `(1 3 5)(2 6)`.
2. The sign rule in 11_8: "Swap counts add, so an even followed by an even stays even, and the even permutations form a subgroup." The review recommends it, and 14_10 later assumes it. This is the one I would add. It would need `permutations.even_on_four` 12, a picture and a word check on 11_8 (70 words now).
3. A worked application in 11_10.

## Residual risks

- About 2,000 lines of Rust have never met a compiler. Expect some repair.
- All layout is unseen. Overlap on 11_7/11_11 and 11_9 s1 is the likeliest visual defect.
- All timing is estimated. The 11_5 count caption is the known late beat; 11_3 and 11_6 silences depend on real take lengths.
- At about 4–5.5 minutes the chapter is shorter than the treatment's 8–12. Beats fit the voice, but I did not add content to reach that length.
- The 11_11 cue boundary is a new seam in the argument, between "i less than j." and "Swap the numbers". Its gap class needs choosing and hearing.
- Anchor phrases bind to current wording; any later reword needs the mirror or the Rust anchor test rerun.
