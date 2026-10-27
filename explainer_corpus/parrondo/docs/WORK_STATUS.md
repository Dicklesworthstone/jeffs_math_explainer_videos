# parrondo: work status

Owner: WildBass (Claude Code, Opus 5.5, xhigh), NTM session
`jeffs_math_explainer_videos--third-wave-20261024`, pane %16.
Bead: `mx-video-program-program-87do.15.1` (root BlueSalmon owns Beads; this
lane never writes Beads and never closes its own task).
Topic: **Two Losing Games Can Win — Here Is the Catch** (`parrondo`).

**State (2026-10-24, 17:20Z): `.15.1` and `.15.2` closed by root; `.15.3`
open.** Root has integrated and registered 01–13. 12/13 went in as native24:
115 release tests, strict all-target Clippy/build, 164/164 facts (Mail 6996).
Root's own repairs, to preserve in every later integration:

- 10/11 (native21, Mail 6942):
  - the computed 26 in 10's head;
  - 11's integer live sum rounded;
  - sequential counters;
  - 11_4's panel cleared, then revealed;
  - the illustration label leaving when the crowd returns;
  - `SHARE_X` at −2.10;
  - `Numbers.b_pct` removed.
- 12/13 (native24):
  - `the_half_point.rs` keeps the constant clearance assertion inside a
    `const` block;
  - the obsolete 13_6 callbacks in chapter 11 and in the storyboard notes
    point to 13_8.

The 14/15 handoff (Mail 6973) is frozen by root and under GoldGate's
independent review; it is not integrated. New unregistered chapters 16 and
17 (ROOT1428-P16) are written: batch_pending, uncompiled, not self-closed
(section "New unregistered chapters 16 and 17" below).

**Freeze.** Every registered file, root's integration edits and the
delivered 08–15 bytes are root's and frozen here: no edit, no format pass.
This lane holds only `when_switching_fails.rs`, `what_winning_promises.rs`,
`docs/integration-16-17.patch` and this status file (reservations 8463–8465
and 7830). The 14/15 reservations are released.

## New unregistered chapters 16 and 17 (root's assignment, ROOT1428-P16)

Not registered and not integrated. The draft, `storyboard.md`, `script.tsv`,
`mod.rs`, `main.rs`, `facts.rs`, `oracle.py`, `model.rs` and
`pause-overrides.json` are untouched; their proposed changes are the patch.
Uncompiled.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/when_switching_fails.rs` (`WhenSwitchingFails`, "16_when_switching_fails") | `ad1e66674c5d6d7f72a6d96eefda347bfcccb1b49cfdd946d663b75304814715` | 1894 | 7 |
| `src/chapters/what_winning_promises.rs` (`WhatWinningPromises`, "17_what_winning_promises") | `7e2e27de2b3cf39c728bb28f91d8ea2ee416e42d6fcf8dcab83c4a30af1134a6` | 1826 | 7 |
| `docs/integration-16-17.patch` | `c2d20e7a17b388b0148f14167ce6a72a97dbb22b5a39e43fa98d3d6a9000a6e8` | 2054 | — |

**The bytes it was built against.** These are the files live at 17:15Z:
root's native24, unchanged since 14:59Z.

- `storyboard-draft.md` `4e19b251`, `storyboard.md` `c81ef034`, `script.tsv`
  `c95d5e1b`;
- `facts.rs` `ebeeef26`, `oracle.py` `5c3acf7d`, `model.rs` `881dd3c8`;
- `mod.rs` `dba308a7`, `main.rs` `2c40269d`, `pause-overrides.json`
  `0307cf26`.

It also builds on the frozen 14/15 handoff: `integration-14-15.patch`
`f8af5d71`, `net_flow.rs` `6de828d2` and `two_products.rs` `e0578227`.

Both scenes import from 14's `net_flow`, so 14/15 must be integrated first:

- 16 uses `CHANGE_S`, `INSPECT_S`, `PANEL_END`, `Puddles`, `THINK_AT`,
  `csv`, `draw_bands`, `mix_labels`, `mix_title`, `puddles`, `stream` and
  `strike`;
- 17 uses `INSPECT_S`, `csv` and `draw_bands`.

The other imports are:

- `draw` `60024c49`;
- `looks_hopeless` `1751dde4`;
- `one_dial` `20aa25a9`;
- `one_player_to_a_crowd` `953a0a54` (16 only);
- `three_corners` `9917a027`;
- `palette` `366457ba`;
- the patched `model` and `facts`.

**Patch order:**

1. 12/13 is already integrated (native24); there is nothing to apply.
2. `git apply docs/integration-14-15.patch` (`f8af5d71`), except its
   `storyboard.md` and `pause-overrides.json` hunks. Root's reserialized
   files no longer match those two, and they are the only failures (checked).
   Apply them by hand, as the builder does:
   - `storyboard.md`: the patch's two badge/07-note edits, with 14 and 15
     appended;
   - `pause-overrides.json`: 14/15's 36 entries appended after root's, in
     root's format (JSON indent 2, `_comment` "08–15").

   The 14/15 patch's `script.tsv` then equals `storyboard.py`'s
   `script_text` of that `storyboard.md` (checked).
3. `git apply docs/integration-16-17.patch`.

The patch's `facts.rs`, `oracle.py` and `model.rs` hunks also apply to the
live files alone (checked). Its draft, `storyboard.md`, `script.tsv`,
`mod.rs`, `main.rs` and pause hunks take their context from the 14/15
step. If root integrates 14/15 differently, apply those by hand
(`git apply --include` splits them out). As a whole the patch does not
apply to the live files without 14/15 (control).

**What the patch changes** (9 files, 22 hunks, three lines of context):

- `storyboard-draft.md` (2 hunks):
  - Arc rows 16 and 17 are rewritten.
  - Sections 16 and 17 are replaced by the implemented words, pictures,
    features, holds and notes: 16_1–16_15 and 17_1–17_14.
  - The advisory estimates are ~307 s and ~267 s; there is no cap.
  - 18 stays planned in the draft only.
- `storyboard.md` (1 hunk): 16 and 17 appended.
- `script.tsv` (1 hunk): regenerated, adding 29 lines.
- `mod.rs` (2 hunks):
  - `pub mod what_winning_promises;` and `pub mod when_switching_fails;`
    in alphabetical order;
  - the two `scene!` lines after 15's, one argument a line as rustfmt lays
    them out.
- `main.rs` (2 hunks):
  - "today 01–17";
  - the comment `// 16–17: schedules played round by round, A's share, and
    what a winning average promises`, then 78 SPOKEN rows after 15_15's (33
    for 16, 45 for 17).
- `facts.rs` (2 hunks):
  - The pub consts `PLAYED_ROUNDS` (24), `EDGE_SHARES`, `WHOLE_EDGES`,
    `CROWD_START` (0) and `WINDOW` (−100, 140), after `PATTERNS`.
  - The 16/17 block, which returns `Err` in seven cases:
    - the coin-toss mix is not A on half the rounds (16_2);
    - a cycle's rounds do not average to the schedule's mean;
    - corner 0 is not refilled from corners 2 and 1 (16_7);
    - the crowd does not start on corner 0 (17_2);
    - 17's bars disagree with the horizon's shares;
    - the start does not cost the same, once, at both horizons (17_5);
    - the player's 1,000 rounds do not match the crowd's (17_10).
- `oracle.py` (8 hunks): the same 33 keys, computed by its own code (exact
  cycles, a Sturm count of the crossings, an integer programme over dollars,
  decimal square roots). Its 31 new cross-checks are listed under "Failed
  controls" below.
- `model.rs` (2 hunks):
  - `cycle`: the exact shares each round of a repeated schedule meets;
  - `played`: a schedule played round by round with `evolve_f64`;
  - `snapshots`: the exact dollar distributions of a crowd, in one pass;
  - `categories`: ahead, behind and level;
  - `moments`: mean and spread.
- `pause-overrides.json` (2 hunks): `_comment` names 08–17. It adds 48
  entries: 36 thought boundaries and 12 clause boundaries, each with its
  meaning (27 for 16, 21 for 17).

The patched oracle passes 87/87 cross-checks with 209 facts. The base (live
plus 14/15) passes 56/56 with 176. The 33 new keys are:

- `sched.a_share`;
- `pub.sched.ab.{first.b.at0.pct, cycle.a.pct2, cycle.b.pct2, cycle.b.pct,
  b.cents, refill.pct2}` and `pub.sched.aabb.{b.at0.pct, b.cents}`;
- `ordinary.sched.cents`;
- `pub.share.{points, crossings.fine, crossings.whole, edge.pct, edge.cents,
  edge.sign, whole.cents}`;
- `horizon.{rounds, rounds.ratio, start, long.mix, short.mix.tallest,
  window.pct, mix.mean, mix.spread, mix.spread2, mix.spread.ratio,
  mix.spread.ratio.fine, rounds.sqrt.fine, mix.at_rate, mix.start_cost,
  long.b.from_start}`;
- `run.short`.

**One model.**

- **16.**
  - Every schedule is played round by round: `model::played` applies A's
    coin on A rounds and B's on B rounds, through `evolve_f64`. A test
    checks ten rounds against `evolve_f64` directly. The starting crowd is
    B's settled one (38.4 % on corner 0).
  - The dots are the corner-0 shares before each B round: 30.7, 34.3, 36.3,
    37.2, 37.65, 37.82 … up to the cycle's 37.96 %, which comes from
    `model::cycle` exactly.
  - Per-corner chances are never averaged. The averaged coin appears only as
    the failed control.
  - The curve is `PUBLISHED.mix_share` at every 0.5 % (exact, drawn as
    floats). Its sign agrees with `mix_verdict_f64` everywhere.
- **17.**
  - Every histogram is `model::snapshots`, from 0 dollars on corner 0,
    negative totals allowed.
  - The bars use one pair of axes throughout: dollars won or lost from −100
    to +140, and the share of the crowd from 0 to 11 %.
  - The transformation steps through all 451 exact snapshots (every even
    round count, 100 to 1,000), with no interpolation. A tracker moves in
    whole steps of two rounds. The counter, the bars, the readouts and the
    average's line read the same snapshot; a test samples it 6,001 times.
  - The player's run is 03's seeded run of B (seed 1999, start 7). The B
    crowd from 7 dollars is `snapshots` too.

**Teaching order.**

- **16:**
  1. Does the order matter, or only how often? (16_1)
  2. Strict alternation AB, half and half (16_2).
  3. Prediction with a countdown (16_3), answered by 16_4's first words:
     AB loses 0.67 cents a round.
  4. A's rounds cost a cent wherever the crowd stands; B's verdict is one
     line in corner 0's share (12's dial, 49/130 = 37.7 %) (16_5).
  5. The actual AB evolution from B's settled crowd: the dots climb past the
     line and settle just above it (16_6).
  6. The mechanism, one cycle in exact numbers (16_7):
     49.5 % × 68.95 % + 50.5 % × 7.59 % = 37.96 %.
  7. The tempting shortcut, averaging the chances, is struck. It gives the
     coin-toss mix's coins and predicts +1.57 (16_8).
  8. AABB, the pattern Harmer and Abbott simulated. Its B rounds meet about
     36 and 33 %, both below the line (16_9).
  9. Only then the rate table: random +1.57, AB −0.67, AABB +1.47 cents a
     round (16_10).
  10. The ordinary-coins control: every schedule loses exactly 3 cents
      (16_11).
  11. The share: 1/20 and 9/10 both lose (16_12).
  12. The computed curve from 0 to 100 % (16_13).
  13. The rounded crossings (about 7 and 85 %) against the exact strict
      bounds (7.03 and 84.71 %, rounded), with the "precisely" card K07
      (16_14). At 7.0 % the mix still loses; 7.1 % wins, 84.7 % wins,
      84.8 % loses, and 85 % loses.
  14. No general switching rule (16_15).
- **17:**
  1. The player's +16 after 1,000 B rounds, with B's −0.87 cents a round
     under it, then −548 after 100,000. This is the finite-run
     counterexample (17_1).
  2. The question of a fixed number of rounds. The exact setup comes first:
     0 dollars, corner 0, negative totals allowed, −1 standing on corner 2
     (17_2).
  3. What the bars are (17_3):
     - one total each, height the share of the crowd;
     - even totals only, with the odd slots framed empty (parity);
     - together they hold the whole crowd (100.0 %);
     - added up by corner, they give the settled crowd again: 34.5, 25.4,
       40.1 %.
  4. Ahead, behind and level as colours and readouts (51.7, 39.6, 8.7 %),
     with K08 (17_4).
  5. The average, +1.29 dollars, against 100 × 1.57 cents and the one-off
     0.28-dollar cost of starting on corner 0. Almost two in five are behind
     (17_5).
  6. Prediction for 1,000 rounds (17_6), answered by 17_7's first word,
     "More" (68.9 %).
  7. The transformation, with the voice silent (17_7).
  8. Behind 28.7 %, level 2.4 % (17_8).
  9. The average grows 1.57 cents a round to +15.42; the spread grows only
     from 9.35 to 29.55 (17_8).
  10. The ratio 3.1620 is near √10 = 3.1623, and the √n growth is called a
      long-run law, not an exact finite rule, with K10 (17_9).
  11. The player's luck. From 7 dollars, B alone for 1,000 rounds: 34.5 %
      ahead, 14.6 % at least +16, "lucky, but not rare" (17_10).
  12. Behind from 0 after 1,000 rounds: A 61.2 %, B 63.9 %, the mix 28.7 %
      (17_11).
  13. The long run: with probability one, one player's average settles on
      1.57 cents, with K09 (17_12).
  14. The three promises kept apart (17_13):
      - the expected drift;
      - the finite-run chance of being ahead, never certain;
      - the almost-sure long-run average.
  15. Invented games, not advice (17_14).
- Units: cents a round for every rate; dollars for totals, averages and
  spreads; percent of the crowd for bar heights and categories.

**Failed controls.**

- **Mutants of the patched oracle** (`mutants_1617.py`, run 4): 9 of 9 are
  caught, each failing the check named:
  1. A window of −90 loses the 100-round extremes.
  2. 6.9 % is called 7.1 %, an edge share on the losing side.
  3. B's break-even line is taken as a third (3 checks fail).
  4. The fifth B round is claimed provable (`tv[9]` instead of `tv[11]`).
  5. The ordinary sweep is run with a B that reads the money.
  6. The crowd is started at 1 dollar, on corner 1 (3 checks fail).
  7. A's refill uses the coin's loss chance (2 checks fail).
  8. The square-root law is taken as exact at 100 and 1,000 rounds.
  9. The bars are added up by remainder mod 2 instead of by corner.
- **In the oracle:**
  - Averaging AB's chances gives the coin-toss mix's coins, which predict
    a win, while AB loses.
  - The fifth B round is below 49/130 and the sixth above. From the sixth
    on, every B round loses: the total-variation distance to the cycle
    (0.18 points) is below the cycle's margin (0.27 points) and never grows.
  - Coins that ignore the money make every schedule its count of each coin.
  - The rounded crossings are not the strict bounds.
  - Single corner-2 bars are not all taller than their neighbours, so only
    the corner sums are claimed.
  - Neither horizon makes being ahead certain. After one round only 29.5 %
    are ahead.
  - The spread ratio is near √10 but not equal.
  - Another seed gives another path.
- **In the tests:**
  - **16:**
    - the averaged coin equals the mix and predicts a win where AB loses;
    - AB loses and AABB wins, and ABB and AAB both win by different
      amounts;
    - the ordinary coins make every pattern its count of each coin;
    - 7 % and 85 % both lose;
    - the best share is not one half.
  - **17:**
    - started from 1 dollar, the share ahead after 100 rounds is not
      51.7 % (the float mirror gives 54.0 %);
    - after 101 rounds the even totals hold nothing and the odd totals sum
      to 1;
    - after one round only the corner-0 chance, under one half, is ahead;
    - single corner-2 bars are not ordered;
    - the spread ratio differs from √10 by more than 10⁻⁴.

**Tests in the two files** (14, all unrun):

- **16:**
  - `the_numbers_are_the_facts`;
  - `the_schedules_are_played_round_by_round`;
  - `the_curve_wins_strictly_between_its_crossings`;
  - `every_phrase_the_scene_waits_for_is_in_its_line` (65 cues);
  - `the_layout_keeps_its_rooms`;
  - `the_card_fits_three_lines` (K07);
  - `the_chapter_runs_and_leaves_room_to_think`: headless, with the 16_3
    think, the beats, the inspection holds, K07's reading time and the end
    hold.
- **17:**
  - `the_numbers_are_the_facts`;
  - `the_bars_are_whole_crowds_on_even_totals`;
  - `the_counter_and_the_bars_show_the_same_round`;
  - `every_phrase_the_scene_waits_for_is_in_its_line` (56 cues);
  - `the_layout_keeps_its_rooms`;
  - `the_cards_fit_three_lines` (K08–K10);
  - `the_chapter_runs_and_leaves_room_to_think`: headless, with the 17_6
    think, the transformation's silent time, the beats, the holds, the
    cards' reading times and the end hold.

**Checks actually run** (scratch, `python3 -I -B`; every run's output is
kept, failed runs included):

- `make_1617_patch.py` run 5 builds base = live + 14/15 as above, and new
  = base + the 16/17 edits. The patch is run 4's except for one note, in
  both the draft and `storyboard.md`. 17's pause intention had placed the
  beat after "sixteen dollars ahead", which was stale; it now matches the
  authored entry after "even though game B loses.". Run 4's patch
  (`d495cb4c`) is kept in scratch.
- `verify_1617_patch.py` r7, on run 5: 58 passed, 0 failed. It covers:
  - the 14/15 step as above;
  - the 16/17 patch applying file for file;
  - the live-only control;
  - the patched oracle: exit 0, 87/87, exactly the 33 new keys, nothing
    else changed, and the base lacks them;
  - the new keys recomputed independently: own exact matrices, an integer
    programme, integer square roots, a separate SplitMix64;
  - SPOKEN against the script and oracle, number words and banned phrases;
  - `storyboard.py`'s check and `script_text`;
  - the registry order and modules;
  - every cue of both live scenes in its line, in order, each line said
    once;
  - K07–K10 wrapping to three lines at 76 (13.0/13.6/13.1/13.9 s to read);
  - 131 pause entries valid, root's untouched and first;
  - patch hygiene, and the live files unchanged.

  Earlier failed run: r2 failed section I (55 of 56 cues) because rustfmt
  had wrapped a cue const. It is fixed by a shorter cue phrase.
- The run 5 oracles run directly: base exit 0 (176 facts), patched exit 0
  (87/87, 209 facts, byte-identical to run 4).
- `rustfmt +nightly-2026-08-31 --check`:
  - both scene files: exit 0;
  - the projected crate's `main.rs` (run 4 + these scene bytes), which
    parses every module: exit 0.

  An earlier projected crate failed. The patch's `facts.rs` had seven lines
  that were not rustfmt-shaped; the builder now writes them shaped.
- `visibility_1617.py`: 138 imports and paths found, 0 not found (text
  search, not name resolution).
- `inline_args_1213.py`: 0 candidates. Three `manual_is_multiple_of` sites
  were found by reading and fixed (two in 17, one in 16).
- `layout_17b.py`: every layout assertion of 17 holds, in floats, over all
  451 snapshots and B's crowd. The controls behave as expected.
- `textwidth.py` (CMU Serif advances, ink height of "0" = size/144): every
  bounded text in 17 fits. This is an estimate, not typesetting.

**Not run (root's gates):**

- Cargo build, test and strict Clippy; facts-check.
- The glyph probe for the new strings:
  - in the scene code: "…" (U+2026) and TeX `\times`, `\approx` in 16; "→"
    (U+2192) and TeX `\sqrt`, `\approx` in 17;
  - through the shared helpers and facts: −, ·, %, "?".
- The layout lint; renders and actual frames; TTS, listening and accent.

**Frame risks to look at:**

1. **16_6:** AB's fifth dot (37.65 %) and the line (37.69 %) are not
   separable by eye; the voice names no round. The settled dots' lower
   edges sit less than 0.05 units above the line (a test asserts it).
2. **17:** the bars are 1 dollar wide on a 2-dollar pitch, about 0.05 units
   at this scale. Look at them at 4K, along with the 0.05-unit frames round
   the empty odd slots.
3. **The transformation** redraws up to 121 rects a frame, plus live
   readouts. When the static overlays swap for live `DecimalNumber`/Text,
   their glyphs may differ.
4. **17_3:** the jagged six-dollar comb of the exact bars may read as
   noise; the corner-sum outlines carry the meaning.
5. **17_8:** the spread bracket clears the tallest bar by 0.12.
6. **Dense text:** the right column holds lines up to about 4.3 units wide,
   and 16's table and curve labels are dense.
7. **Cards** K07–K10 are three lines each and need 13–14 s to read.
8. **Widths** are estimated only; `keep_between` and `keep_apart` refuse at
   run time if real widths differ.
9. **Length:** 16 (~307 s) and 17 (~267 s) are long, advisory only. The
   conceptual seams are 16 at 16_10/16_11 or 16_11/16_12, and 17 at
   17_9/17_10. No cut is proposed.

**Pronunciation risks:**

- 16: "thirty point seven", "sixty-nine", "twenty-nine and a half", "one
  point five seven", "zero point six seven", "one point four seven", "seven
  point zero three", "eighty-four point seven one", "Harmer and Abbott".
- 17: "fifty-one point seven", "thirty-nine point six", "one fifty-seven",
  "sixty-eight point nine", "twenty-eight point seven", "three point one
  six", "thirty-four point five", "fourteen point six", "sixty-one point
  two", "sixty-three point nine", "five hundred forty-eight".

**Limits:**

- The player's run is one seeded illustration; nothing claims a typical
  outcome.
- Harmer and Abbott are credited only with simulating AABB and random
  switching.
- The strong-law and central-limit statements are cited to Ethier & Lee
  (2009) in the cards, not derived.
- All Rust here is uncompiled.

## New unregistered chapters 14 and 15 (root's assignment, ROOT1232-P14)

Not registered and not integrated. The draft, `storyboard.md`, `script.tsv`,
`mod.rs`, `main.rs`, `facts.rs`, `oracle.py`, `model.rs` and
`pause-overrides.json` are untouched; their proposed changes are the patch.
Uncompiled.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/net_flow.rs` (`NetFlow`, "14_net_flow") | `6de828d2ded1a6ee174a8329b84b052394f48df47ab048920fd8c034285d857c` | 1758 | 7 |
| `src/chapters/two_products.rs` (`TwoProducts`, "15_two_products") | `e0578227b638c9e17847f426672685fc676b6e6f8dfda783663eddd1f8b6c8d5` | 1541 | 9 |
| `docs/integration-14-15.patch` | `f8af5d71916e95c1e53d2da24ee4cdcc5549c5d766aa10a3259a5c094d5fb190` | 1073 | — |

`net_flow.rs` also holds what 15 imports (`pub(super)`): the panel's layout
constants and sizes, and its TeX and flow helpers (`pieces`, `cancel`,
`strike`, `stream`, `puddles`, `edge_nets`, `whole`, `whole_flows`,
`mix_title` and others).

**The bytes it was built against** (live at 14:34Z, after root's 10/11
integration): `storyboard-draft.md` `0c288f82`, `storyboard.md` `ad0b9019`,
`script.tsv` `d38394c6`, `facts.rs` `7551b0e1`, `oracle.py` `8dd54331`,
`model.rs` `881dd3c8`, `mod.rs` `ea8df654`, `main.rs` `6f4933ef`,
`pause-overrides.json` `d953317b`, and `docs/integration-12-13.patch`
`2fa03dd6`.

The sources 14 and 15 import from:

- `draw` `60024c49`;
- `looks_hopeless` `1751dde4`;
- `one_dial` `20aa25a9`;
- `one_player_to_a_crowd` `953a0a54`;
- `three_corners` `9917a027`;
- `the_trap` `51793c57`, root's 10/11 repair. 14's `shaft` is still
  `pub(super)` with the same signature.

**Patch order:**

1. 10/11 is already integrated (native21); there is nothing to apply.
2. `git apply docs/integration-12-13.patch`, then root's 12/13 registration
   exactly as that section's steps 2–4 list:
   - the two `pub mod`s, and the two scene lines after 11's;
   - 12 and 13 copied from the patched draft into `storyboard.md`, then the
     script regenerated;
   - `main.rs` "today 01–13", one comment line
     `// 12–13: one dial, then the published coins and the paper`, then the
     37 rows after 11_7's, in that section's order, rustfmt-shaped.
3. `git apply docs/integration-14-15.patch`.

The patch's `mod.rs`, `main.rs`, `storyboard.md` and `script.tsv` hunks take
their context from that simulated registration. If root registers 12/13
differently, apply those four files' hunks by hand; `git apply --include`
splits them out.

Its draft, `facts.rs`, `oracle.py`, `model.rs` and pause hunks apply to live
plus the 12/13 patch alone (checked). Without the 12/13 patch it does not
apply (control). If root adds 12/13 pause entries first, the pause hunk's
context changes; its 36 entries go after root's 08–11 entries.

**What the patch changes** (9 files, 27 hunks, three lines of context):

- `storyboard-draft.md` (4 hunks):
  - Arc rows 14 and 15 are rewritten, with the status "native source
    written, uncompiled".
  - Sections 14 and 15 are replaced by the implemented words, pictures,
    features, holds and notes: 14_1–14_15 and 15_1–15_15. The advisory
    estimates are ~282 s and ~293 s; there is no cap.
  - The badge sentence becomes "(02_5, 07_5, 13_1, 15_14) … again at 14_2".
  - 07's note becomes "the round badge stands from 07_5 to 12_12 and again
    from 14_2 to 15_13; the published coins return, filled, at 13_1 and at
    15_14".
- `storyboard.md` (3 hunks): the same badge sentence and 07 note; 14 and 15
  appended.
- `script.tsv` (1 hunk): regenerated.
- `mod.rs` (3 hunks): `pub mod net_flow;` and `pub mod two_products;` in
  alphabetical order; `scene!("14_net_flow", 0.45, net_flow::NetFlow)` and
  `scene!("15_two_products", 0.45, two_products::TwoProducts)` after 13's.
- `main.rs` (2 hunks): "today 01–15"; the comment `// 14–15: the 1,418's
  net flow, real seeded rounds, B's balances and the two products`, then 91
  SPOKEN rows after 13_10's.
- `facts.rs` (4 hunks):
  - `crowd.net` (B's 0,0,0) after `crowd.back`.
  - The real-round block after `mixcrowd.losers`. It is
    `realmix.round.1.{forward,back,net,after,gain}`, `realmix.rounds` and
    `realmix.mean.{net,gain}`, all from `model::real_crossings` on the same
    490/360/568 and seed 1999.
  - Before the products loop, `round.mix.sign.factor` (49/64),
    `round.products.mix.diff` (3/160) and `round.mix.sign.sides` (147/16).
    It returns `Err` if 12 × 49/64 ≠ 490 × 3/160.
  - The sums test's comment now names 14_8.
- `oracle.py` (7 hunks): the same 12 keys, computed by its own code. New
  cross-checks:
  - a 1331-game edge sweep, with certain coins included;
  - `line_bad` in the 729 loop;
  - the unsettled control;
  - per-round conservation of the real rounds;
  - the first-round control;
  - the mix's one line.
- `model.rs` (1 hunk): `real_crossings` after `real_crowd`. It is the same
  loop and stream, returning each round's forward and back counts per
  corner, counted before anyone moves.
- `pause-overrides.json` (2 hunks): `_comment` names 14–15. It adds 36
  entries: 32 thought boundaries and 4 clause boundaries, each with its
  meaning.

Facts go from 164 to 176. The patched oracle passes 56/56 cross-checks; the
base passes 50/50. The SPOKEN rows are in the patch's `main.rs` hunk.
14_1, 14_5 and 14_7 say no number word. 15_7's "third" is bound to
`rules.modulus` (the verifier's run 2 found it unbound).

**One model.**

- **14.**
  - The mix's crowd of 1,418 (490/360/568) and its whole forward and back
    flows come from the facts.
  - The edge table's pairs (147 − 135, 225 − 213, 355 − 343), its nets of
    12 and the gain 36 = 727 − 691 are computed from those counts, and
    `numbers()` checks every string against the facts.
  - The real round and the thousand rounds are `real_crossings` from the
    same crowd and seed. A test asserts they move exactly the players
    `real_crowd` counts.
  - B's balances (10/4/12, forward 1/3/9, back 9/1/3) cancel to
    9/160 = 9/160.
- **15.**
  - B's crowd is rebuilt from ten at corner 0 by the first two balances
    (`whole` refuses a fraction) and checked against `crowd.counts`.
  - The letters' steps follow.
  - The one line on the mix's numbers comes from the new sign facts.
  - The per-player average 3 × 12 / 1418 = 18/709 is shown as ≈ 2.54 cents
    per player per round.
  - The published table comes from the `pub.*` facts.

**Teaching order.**

- **14:**
  1. A test from the coins alone?
  2. The 1,418, with whole counts, still 34.6 % at corner 0.
  3. Paired opposing flows, one edge at a time.
  4. The viewer counts three surpluses (14_5 hold).
  5. Predict the crowd's gain (14_7 countdown), answered with 36 dollars,
     three times 12.
  6. Why a settled corner forces the adjacent nets to agree: corner 1 keeps
     its 360 (14_9).
  7. Expected split counts against one real seeded round (7, 4, −17; lost
     6 dollars) and a thousand rounds (11.9 per edge, 35.8 dollars),
     labelled as one illustration (14_10–14_11).
  8. Zero net flow means fair; B's 26 show it.
  9. Multiplying the balances, the counts cancel and leave the products
     (14_12–14_15).
- **15:**
  1. Both directions, with every chance strictly between 0 and 100 %
     (15_1).
  2. The converse built on B's 10, 4, 12, then in letters. Dividing by q is
     safe; a balanced crowd stays put, and only one spread does, because
     every corner reaches every other (15_2–15_6).
  3. The net flow kept in each balance collects into one line. Its factor
     p₂ + q₁q₂ and w₀ are positive, so the net flow has the sign of the
     forward-minus-backward product (15_7–15_8).
  4. B is fair, while averaging its forward chances misjudges it (struck,
     15_9).
  5. The mix's products, with the answer withheld for the prediction
     (15_10), revealed in 15_11.
  6. The one line in the mix's numbers (15_12), then 18/709 per player
     with the "precisely" card K06 (15_13).
  7. The published A, B and mix (15_14), then what the test never needed
     (15_15).
- Units: dollars for the crowd's gain, "cents per player per round" for the
  average.
- No spanning-tree formula is in the teaching. "Weight" is named in words
  at 09_9 ("each corner's long-run share of the rounds"). It gets its
  letter at 14_15 ("its share of the settled crowd, as w"), before 15's w₀.

**Failed controls** (in the tests, the oracle, or both):

- One real round's edges disagree (7, 4, −17) and the crowd loses 6
  dollars. Equal nets belong to the expected split, not to each round.
- An unsettled crowd: B's 10/4/12 under the mix's chances has nets 3/2, −2
  and 1/2. Each corner changes by net in minus net out.
- The mix's crowd does not balance its edges, and its products differ
  (75/640 against 63/640).
- Judging by averaged forward chances or by the forward product against 1/8:
  B's chances average 53.3 % and its forward product is under 1/8, yet B is
  fair.
- The converse fails when the products differ: the built crowd misses the
  third balance and a round moves it.
- A certain coin, [100, 200, 0]/200 (chances 50 %, 100 %, 0 %): the factor
  and w₀ are both 0, so the one line alone decides nothing there.
- The gap's size does not order the gains. (1, 8, 9) tenths has the smaller
  positive product gap but the bigger average than (4, 4, 8).

**The condition's edge (oracle sweep, 1331 games, chances 0–10 tenths).**
On the triangle, the one line and the sign rule hold for every game,
certain coins included. Whenever the products differ, p₂ + q₁q₂ > 0 and
w₀ > 0. The factor vanishes only when p₁ = 1 and p₂ = 0 (11 games), and
then both products are 0 and so is the net flow.

The strictly-positive condition is what the argument uses: dividing by q in
15_4, and reachability in 15_6. The film states it as the argument's
condition and never claims the test fails without it.

**Tests in the two files** (16, all unrun):

- **14:**
  - The words equal the numbers.
  - A settled crowd carries one net flow, on all 729 games of tenths, with
    the unsettled control.
  - The real round is the real crowd's: conservation every round, the
    thousand-round averages, the first-round control.
  - Multiplying the balances leaves the products, with the mix and
    misjudgement controls.
  - The cues, the layout rooms, and the headless run (the 14_5 count hold,
    the 14_7 think, beats, inspection holds, the end hold).
- **15:**
  - The words equal the numbers.
  - Equal products balance every edge (729 games, with the control).
  - The one line gives the sign (729 games, the certain-coin and gap-size
    controls).
  - The products decide B and the mix.
  - The published products decide A, B and the mix.
  - The cues, K06's three-line wrap, the layout rooms, and the headless
    run (the 15_10 think, the steps' and reveals' holds, card reading time,
    the end hold).

**Checks actually run** (scratch, `python3 -I -B`; every run's output is
kept, failed runs included):

- `make_1415_patch.py`, run 4: it builds base = live + 12/13 patch +
  simulated 12/13 registration, then new = base + the 14/15 edits, and
  writes the patch. Run 4's patch is byte-identical to run 3's (`f8af5d71`).
- `verify_1415_patch.py`, run 4: 54 passed, 0 failed. It covers:
  - the 12/13 patch still applies;
  - the 14/15 patch applies to the base and gives the new tree file for
    file;
  - its non-registration hunks apply to live + 12/13 alone;
  - the control that it fails on live alone;
  - the patched oracle (exit 0, 56/56, the new keys, nothing else changed;
    the base lacks them);
  - the new keys recomputed independently (exact fractions and a separate
    SplitMix64);
  - SPOKEN against the new script and the patched oracle, number words and
    banned phrases;
  - `storyboard.py`'s check, the drafts' sections, the registry order and
    the script owners;
  - every cue in its line, in order;
  - K06's wrap (3 lines at 76, 14.1 s to read);
  - pause overrides valid, with root's 08–11 entries untouched;
  - patch hygiene, and the live files unchanged.

  Earlier runs: run 1 crashed on my own NameError; run 2 failed on 15_7
  being unbound, which is now fixed.
- `layout_1415.py`: every layout assertion in both tests, in floats.
  - Run 1 found a real defect. 15's `CHAIN_YS[0] - STEP_YS[0] >= 0.5` was
    reversed (−0.6), which would be a const-eval compile failure. The
    layout was right (the products line rises to −0.75; the chain starts
    at −1.35), so the assertion now reads `STEP_YS[0] - CHAIN_YS[0] >= 0.5`
    (0.6).
  - Run 2: 0 failed. The before bytes (`03652430…`) are preserved in
    scratch.
- `visibility_1415.py` on the run 4 tree: 0 names not found (text search,
  not name resolution). I also read by eye that every model method the
  tests call exists and is `pub` (`mean`, `shares`, `products`, `chance`,
  `evolve`, `real_crowd`, `real_crossings`, `Game`'s fields, `MODULUS`,
  `DEN`, `SEED`).
- `inline_args_1213.py` on both files: 0 candidates.
- `rustfmt +nightly-2026-08-31 --check`:
  - both scene files: exit 0, empty output;
  - the run 4 base and new trees' `main.rs`, which covers every module
    there, the patched `facts.rs`, `model.rs` and `main.rs` included: exit
    0, empty output.

**Not run (root's gates):**

- Cargo build, test and strict Clippy; facts-check.
- The glyph probe for the new strings:
  - in text: ⇒/`\Rightarrow`, `\,`, →, −, ·, ?, apostrophes, "<" and ">";
  - K06's parentheses and "w0".
- The layout lint; renders and actual frames; TTS, listening and accent.

**Frame risks to look at:**

1. Tiny motions: the real round's counts change by a few players (490 →
   466). The published puddles move from 34.6 to 34.5.
2. Motion near digits:
   - counts carried down rows by `arrive_from`;
   - copies of the chances gathered into products;
   - red strikes over digits;
   - the "?" placeholders swapping for products at 15_11.
3. Dense panels:
   - the 15_7–15_8 derivation;
   - 15_12's numbers over letters at 24 pt (490's room around "w₀" is about
     0.07);
   - 15_14's table (its columns' const margins are 0.25 beyond assumed
     widths);
   - 14's real-round panel.
4. K06 is a three-line card. The triangle's title, centred at −2.3, clears
   the card's top by only 0.04 in the const assertion, which assumes a 0.08
   half-height; the real text height is unmeasured.
5. The 15 rule line at 24 pt is about 5.1 of the 5.9 available.
6. The countdown at (5.5, 2.0); the legend widths.
7. Length: 14 and 15 are long (~282 s and ~293 s, advisory). If root wants
   splits, the conceptual seams are 14 at 14_9/14_10 or 14_12/14_13, and 15
   at 15_6/15_7 or 15_13/15_14. No cut is proposed.

**Pronunciation risks:**

- 14: "one thousand four hundred eighteen", "five hundred sixty-eight",
  "thirty-seven and a half", "minus seventeen", "thirty-five point eight",
  "nine over one hundred sixty".
- 15: "nine tenths", "fifty-three point three", "seventy-five over six
  hundred forty", "sixty-three over six hundred forty", "forty-nine
  sixty-fourths", "three over one hundred sixty", "one hundred forty-seven
  over sixteen", "Harmer and Abbott".

**Follow-ups outside this lane's files (for root, not done):** 10/11's
references to "13_6's callback" become 13_8 (Mail 6925).

**Limits:** the seeded rounds are one illustration, labelled with their
length; nothing claims a typical or promised outcome. The published coins'
history stays within 13's citation. All Rust here is uncompiled.

## New unregistered chapters 12 and 13 (root's assignment, Mail 6914)

Assignment marker GERMAN1133-P12. Delivered as Mail 6924 and frozen by root
(SHA-256, `tend-1232/parrondo-12-13-intake`). Not registered and not integrated:
`mod.rs`, `main.rs`, `storyboard.md`, `script.tsv`, the draft, `facts.rs`
and `oracle.py` are untouched; their proposed changes are the patch.
Uncompiled.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/one_dial.rs` (`OneDial`, "12_one_dial") | `20aa25a9e8991d7a91eaac44c37d933fa4ac5a237beecc101a87bb34fc90c630` | 1270 | 8 |
| `src/chapters/the_half_point.rs` (`TheHalfPoint`, "13_the_half_point") | `c5b2f080c3c4d83d17c28e856261ae6207ebd869cede47f9a0beb1251fede3d1` | 1114 | 7 |
| `docs/integration-12-13.patch` | `2fa03dd6003afdacd7fa68d4eee909f75feeebc3ea0a2096f9deebc07c0af4a1` | 182 | — |

`one_dial.rs` also holds the dial's helpers that 13 imports (`pub(super)`:
the axes, marks, points, line, uprights, `line_tex`, `average_tex`,
`pct_value` and the dial's layout constants).

**The bytes it was built against** (live at 12:45Z, root's 08/09
integration): `storyboard-draft.md` `b763d867`, `storyboard.md`
`8b96ab60`, `facts.rs` `5a630ac7`, `oracle.py` `8695632d`, `main.rs`
`5e94e9f3`, `mod.rs` `6623609a`, `script.tsv` `0bd182a0`. The sources 12
and 13 import from: `draw` `60024c49`, `looks_hopeless` `1751dde4`,
`one_player_to_a_crowd` `953a0a54` (root's test-only edit; the imported
items are unchanged), `three_corners` `9917a027`, `model` `881dd3c8`.

**What the patch changes** (4 files, 11 hunks, one line of context):

- `storyboard-draft.md`:
  - 12 and 13 are replaced by the implemented words and pictures, with new
    notes: 12 runs 12_1–12_12 and 13 runs 13_1–13_10.
  - 07's note on the badge now says the round badge stands "from 07_5 to
    12_12" (it said 12_8).
  - Advisory estimates are ~140 s for 12 and ~114 s for 13, both over the
    review mark; there is no cap.
- `storyboard.md`: the same 07 note.
- `facts.rs`:
  - `source.volume` 402, `source.page` 864 and `source.year` 1999 are
    pinned to `HARMER_ABBOTT`, after `coins.published.tails.pct`, taking the
    facts from 161 to 164. They feed 13_10's card and its spoken "nineteen
    ninety-nine". 13 refuses to draw without them.
  - The sums test's comment now names 13_7 and 12_9.
- `oracle.py`: the same three facts, emitted after
  `coins.published.tails.pct`.

**Integration steps** (root's; after 10/11):

1. Apply the patches in order: `git apply` `docs/integration-10-11.patch`
   (it still applies to `b763d867`), then `docs/integration-12-13.patch`.
   Applying 12/13 first and then 10/11 gives the same files (verified).
2. `src/chapters/mod.rs`:
   - Add `pub mod one_dial;` and `pub mod the_half_point;` in alphabetical
     order.
   - After 11's entry, add `scene!("12_one_dial", 0.45, one_dial::OneDial),`
     and `scene!("13_the_half_point", 0.45, the_half_point::TheHalfPoint),`.
3. `docs/storyboard.md` and `script.tsv`: copy 12 and 13 from the patched
   draft, and regenerate the script as for 08/09.
4. `main.rs`: the module comment's "today 01–09" (01–11 after 10/11)
   becomes "today 01–13", and the SPOKEN rows below are added.
5. Facts-check should then see 164 facts. The 15 new tests need the script
   lines (cue test, headless runs) and the source facts (13's `numbers()`).

**Follow-ups outside this lane's files (not done, for root):**

- `mix_the_games.rs:32` and `:1088` call the player's 34.5 % "13_6's
  callback"; in this 13 that callback is 13_8.
- 11's notes, as the 10/11 patch adds them ("it is 13_6's callback"), need
  the same change.

**One model.**

- 12: one tracker turns the dial. It moves the hero marker on corner 0's
  bar and the gold needle on the dial from B's weight 5/13 to the mix's
  245/709.
  - A second tracker slides a point along B's line from a third to 5/13.
  - Every point and mark is placed by `on_line(top, slope, share)` from the
    coins' facts, and the tests assert the points lie on the line.
- 13: one tracker `shave` (0 → 1) moves both crowd marks, through float
  stationary shares of the coins less h·ε. The test checks its ends
  against the exact 5/13, 245/709, 32401/84463 and 49/142 (1e-12) and that
  it falls strictly.
  - A second tracker `lower` drops the line by h·ε and slides its crossing
    of 50 % from 5/13 to 49/130. After `shave` stops, the moving marks
    become fixed ticks, because a redrawn mark would redraw over a flash.
  - The runs' chips use `model::player(Play::B/Mix).share_at_zero()`.
  - The bars are `looks_hopeless::bars(averages(&f))`, exactly 05_3's.

**What 12 does** (cue → picture; holds in bold):

- **12_1.** 11's panel and the outlined badge; "Stay with the round coins"
  frames the badge.
  - "each break even": "game A alone: breaks even" and "game B alone: breaks
    even" appear where the dial will be.
  - "a fair coin splits the rounds": each teal bar becomes equal blue and
    violet halves, labelled "A rounds" and "B rounds".
- **12_2.** "never looks at the money" frames all six halves, which then
  flash, blue then violet.
  - "A's coin wins exactly half": "A rounds: 50 % at every corner".
  - "can't be where": the blue halves dim. **`BEAT`.**
- **12_3.** The violet halves flash; "just one number" frames corner 0's
  violet half; "swaps a seventy-five percent coin" brings "B rounds: 10 %
  at corner 0, 75 % elsewhere".
- **12_4.** "a dial": the alone lines give way to the share axis (32–40 %).
  "on its own sets it": a violet mark at 38.5 %, and corner 0's ghost
  flashes.
- **12_5.** "sit wherever the crowd sits" frames the halves. "moved the
  crowd off corner zero": a red outline of the lost part of corner 0's bar
  appears and flashes. "turns the dial down": the tracker turns the dial
  and the mark reads "the mix: 34.6 %". **`BEAT`.**
- **12_6.** "What is that turn worth": the win axis.
  - "With a third": the dim mark and its upright; "fifty-three point
    three": the point at (33.3, 53.3).
  - "At game B's own": the point slides along the line.
  - "exactly half": the 50 % line and B's point. **`BEAT`.**
- **12_7.** "a straight line": the full line.
  - "seventy-five percent, minus": the TeX line; "five thirteenths": the
    even TeX, and B's point flashes.
  - "the weight we found": the ghost flashes; "That's why": a frame on
    "game B alone: 38.5 %" points at B's point.
- **12_8.** The gold upright at 34.6 %, the question, and the countdown
  (**`THINK_S`**).
- **12_9.** "Fifty-two point five": the gold point and "52.5 %".
  "Averaged": "½ · 50 % + ½ · 52.5 % ≈ 51.3 %"; "of all its rounds" frames
  it.
- **12_10.** "+2.54 cents a round" and the K05 card, held for `read_s`.
- **12_11.** Frames on the A legend, the red outline and the B legend
  (pointing at the corner 1 and 2 halves).
- **12_12.** "keeps a third": dashed thirds on the bars. "most of the way":
  the arrow 38.5 → 34.6 %, dashed to 33.3 %, with "76 % of the way to a
  third". **Stood.**

**What 13 does:**

- **13_1.** 12's dial is rebuilt, with the coins row (50/10/75 %) and the
  ledger (0.00, 0.00, +2.54).
  - "shave the half point": a "−0.5" falls toward each chance as it becomes
    49.5/9.5/74.5 %.
  - "published": the filled published badge; "both games really lose"
    frames game A and game B.
- **13_2.** "losing a cent": "→ −1.00".
- **13_3.** "barely move": the round points dim; `shave` moves both marks
  (1.5 s), which then settle.
  - "to thirty-eight point four": "game B alone: 38.4 %" and its tick
    flash; "to thirty-four and a half": "the mix: 34.5 %". **`BEAT`.**
- **13_4.** "drops by half a point": `lower` (1.4 s) drops the line; the
  old line goes faint, the round points go, and the hero crossing slides.
  - "falls to": "break even: 37.7 %", its upright, and both TeX lines.
  - "sits just past that": B's point under 50 %, flashed; "B loses": "→
    −0.87". **`BEAT`.**
- **13_5.** "an edge of" frames +2.54; "Does it survive": "→ ?" and the
  countdown (**`THINK_S`**).
- **13_6.** "It does": the gold point at (34.5, 52.1) with "52.1 %",
  flashed at "still win".
  - "now lose a little" frames "49.5 %" and points at the A coin.
  - "fifty point eight": "½ · 49.5 % + ½ · 52.1 % ≈ 50.8 %".
- **13_7.** "costs the mix": "+2.54 − 0.97 ≈ +1.57 cents", framed at
  "bigger than that". "still wins": "→ +1.57".
- **13_8.** "own runs": the model's and the runs' lines, plus teal chips on
  the marks, flashed at their words. **`BEAT`.**
- **13_9.** "every number": 05_3's bars, the strike and faint halfway.
  Frames on the A/B labels, then on the mix label and its bar. **hold +
  `INSPECT_S` 1.5 s.**
- **13_10.** "paper": the citation card; "appeared in Nature" frames its
  last line. **Stood**, then fade.

Semantic pause intentions (narration, in the draft's notes): a held beat
after 12_2's "can't be where the winning comes from"; the answers 12_9 and
13_6 begin promptly after their countdowns; "It does." gets a short beat in
the reading.

**SPOKEN rows for `main.rs`** (37, each value checked against the patched
oracle):

```
("12_2", "wins exactly half the time", "round.mix.a_rounds.win", "1/2"),
("12_3", "a seventy-five percent coin for a ten percent one", "coins.round.heads.pct", "50,10,75"),
("12_4", "thirty-eight point five percent", "crowd.share0.pct", "38.5"),
("12_5", "thirty-four point six percent", "round.mix.weights.pct", "34.6,25.4,40.1"),
("12_6", "With a third of B's rounds", "thirds.pct", "33.3"),
("12_6", "fifty-three point three percent", "round.b.thirds.win.pct", "53.3"),
("12_6", "own thirty-eight point five", "crowd.share0.pct", "38.5"),
("12_6", "exactly half", "round.b.win", "1/2"),
("12_7", "seventy-five percent, minus sixty-five percent", "round.b.line.pct", "75,65"),
("12_7", "exactly five thirteenths", "round.b.fair_share", "5/13"),
("12_8", "thirty-four point six percent", "round.mix.weights.pct", "34.6,25.4,40.1"),
("12_9", "Fifty-two point five percent", "round.mix.b_rounds.win.pct", "52.5"),
("12_9", "the A rounds' fifty", "round.mix.a_rounds.win", "1/2"),
("12_9", "fifty-one point three percent", "round.mix.win.pct", "51.3"),
("12_10", "just over two and a half cents a round", "round.mix.mean.cents", "+2.54"),
("12_12", "keeps a third of the rounds", "thirds.pct", "33.3"),
("12_12", "most of the way", "round.mix.toward_thirds.pct", "76"),
("13_1", "the half point off every coin", "rules.handicap", "1/200"),
("13_2", "losing a cent a round", "pub.a.mean.cents", "-1.00"),
("13_3", "from thirty-eight point five", "crowd.share0.pct", "38.5"),
("13_3", "to thirty-eight point four percent", "pub.b.weights.pct", "38.4,15.4,46.2"),
("13_3", "from thirty-four point six", "round.mix.weights.pct", "34.6,25.4,40.1"),
("13_3", "to thirty-four and a half", "pub.mix.weights.pct", "34.5,25.4,40.1"),
("13_4", "drops by half a point", "rules.handicap", "1/200"),
("13_4", "thirty-seven point seven percent", "pub.b.fair_share.pct", "37.7"),
("13_4", "a little under a cent a round", "pub.b.mean.cents", "-0.87"),
("13_5", "just over two and a half cents a round", "round.mix.mean.cents", "+2.54"),
("13_6", "At thirty-four and a half percent", "pub.mix.weights.pct", "34.5,25.4,40.1"),
("13_6", "fifty-two point one percent", "pub.mix.b_rounds.win.pct", "52.1"),
("13_6", "fifty point eight percent", "pub.mix.win.pct", "50.8"),
("13_7", "about one cent a round", "handicap.mix_cost.cents", "0.97"),
("13_7", "just over one and a half cents a round", "pub.mix.mean.cents", "+1.57"),
("13_8", "thirty-eight point four percent in game B", "run.b.at0.pct", "38.4"),
("13_8", "thirty-four and a half in the mix", "run.mix.at0.pct", "34.5"),
("13_9", "A loses a cent a round", "pub.a.mean.cents", "-1.00"),
("13_9", "B a little under a cent", "pub.b.mean.cents", "-0.87"),
("13_10", "nineteen ninety-nine", "source.year", "1999"),
```

12_1 and 12_11 say no number word. 12_5's "half of every corner's rounds"
is the fair game-picking coin; no fact key holds it, as with 11_2, and the
line's other row meets `main.rs`'s rule.

**Tests in the two files** (15, all unrun):

- **12:**
  - The words equal the numbers.
  - One number decides B and the mix, with the controls: B's rounds spread
    as B alone's break even; the A rounds win exactly 1/2; a corner-reading
    switch.
  - The sums print ≈ when they round, with an exact "=" and a refused
    mismatch as controls.
  - The line is even where its coins say (5/13 and 49/130, 1/130 apart).
  - The cues, the K05 wrap and the layout rooms.
  - The headless run, with its beats, think, card read and end hold.
- **13:**
  - The words equal the numbers.
  - The shaved coins end on the exact model.
  - The edge survives, with the controls: the naive cost is 1.00, not 0.97,
    and shaving only the good coin gives 49/129.
  - The runs agree at one decimal and differ at two.
  - The cues, the layout rooms, and the headless run.

**Checks actually run** (scratch, `python3 -I -B`; outputs kept beside
each script):

- `checks_1213.py`: 50 checks, 0 failed. It covers the dial line for both
  coin sets and any split, the chain of corner and game (a corner-preferring
  coin breaks it: 0.6073 against 0.3401), the round and published mixes, and
  the wrong answers.
- `checks_1213b.py`: 102 checks, 0 failed.
  - Every cue is in its proposed line, once and in order.
  - The scenes say their lines in order.
  - The 37 SPOKEN rows match the patched oracle.
  - Number words and banned phrases are as `main.rs` parses them.
  - The K05 wrap and the citation agree with the brief.
  - Hygiene passes. It also lists the `flash_all` targets, which I read by
    eye: all are shapes.
- `layout_1213.py`: 51 passed, 0 failed. Every layout assertion of both
  tests is re-evaluated in floats, and the run found two defects, now
  fixed:
  - 12's `THINK_AT` [5.2, 1.6] failed its own countdown-above-the-line
    assertion (0.5300 > 0.5327 false); it is now [5.3, 1.75].
  - The "−0.5" ended overlapping the chance's digits; it now stops 0.08
    above them, and a const assertion holds it there.
- `verify_1213_patch.py`, run 2: 0 failed.
  - The patch applies alone, after 10/11, and before 10/11 (same files).
  - The patched oracle passes 50/50 cross-checks with 164 facts and emits
    the citation; the live oracle lacks it (control).
  - The live files are unchanged.
  - Run 1 failed only the 08/09 orders, because root had already integrated
    08/09. Its output is preserved.
- `visibility_1213.py`: every imported name is declared visible (0 not
  found).
- `inline_args_1213.py`: 0 `uninlined_format_args` candidates. A probe
  confirmed it catches a multi-line positive.
- A float check of the `shave` shares: they fall strictly over 21 steps and
  their ends equal the exact fractions.
- `rustfmt +nightly-2026-08-31 --check` passes on both files and on the
  patched `facts.rs` copy. Diffs were applied by hand; earlier runs'
  outputs are preserved.

**Not run (root's gates):** Cargo build, test and strict Clippy;
facts-check; the glyph probe for the new strings (→, − and · in text;
TeX's `\approx`, `\tfrac` and `\times`; the apostrophe and parentheses in
the citation); the layout lint; renders and actual frames; TTS, listening
and accent.

**Frame risks to look at:**

1. Text widths are unmeasured, but `keep_between` and `keep_apart` refuse
   loudly in the headless test. The tightest are:
   - the panel TeX at 26 pt ("50 % at share = 24.5/65 = 49/130 ≈ 37.7 %");
   - 13_8's 70-character caption at 20 pt;
   - the citation title at 26 pt in a 9.6-unit card;
   - row 1's two mark words side by side (as in 12).
2. 13_3's motion is small on purpose: 0.07 and 0.03 scene units (about 19
   and 9 px at 4K).
3. The falling "−0.5" and 13_4's crossing dot both move. Mail 6922's
   lesson is that motion past digits can pass the lint and still show in
   frames.
4. 13_1 rebuilds the whole dial at once, with the new coins and ledger, in
   0.6 s. If it reads as a wall, stage the coins and ledger after the dial.
5. The faint old line beside the live one in 13_4 must read as "before".
6. 12_12's "76 %" words sit between the arrow and the 50 % line
   (const-asserted).

**Limits:** history stays within the brief's quoted sentence plus the date
and length (M4). The runs are one seeded illustration, labelled with their
length. All Rust here is uncompiled.

## New unregistered chapters 10 and 11 (root's assignment, Mail 6904)

Not registered and not integrated. `mod.rs`, `main.rs`, `storyboard.md`,
`script.tsv` and the draft are untouched. Uncompiled.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/the_trap.rs` (`TheTrap`, "10_the_trap") | `0d4daf7873830c265ea10a13e7aa6eb44d26ec3686dac7b692d7dc399bd37162` | 1312 | 7 |
| `src/chapters/mix_the_games.rs` (`MixTheGames`, "11_mix_the_games") | `f3b247707762242f07e1bd559306208031bbd4a1f08eed5eb06948a96a577b3e` | 1206 | 7 |
| `docs/integration-10-11.patch` (against `storyboard-draft.md` `52d093cf…`) | `9d3b68e713ba387df15080dfb0c94277f61cc178cc7f87e1d151991f8cd314f3` | 117 | — |

The delivered 08/09 bytes, unchanged by this lane:
`one_player_to_a_crowd` `cf3e54fa`, `ten_four_twelve` `c2ddfc6f`,
`integration-08-09.patch` `a0e24eb8`. The registered files are unchanged
too, rechecked at 11:27Z: `does_b_win` `1e22ff62`, `draw` `60024c49`,
`glyphs` `9891c7e1`, `looks_hopeless` `1751dde4`, `mod` `22ad9c91`,
`the_clue` `b413c943`, `the_puzzle` `24f0ef14`, `the_rule` `ad182b3d`,
`three_corners` `9917a027`, `where_you_stand` `fce6cc38`, `facts`
`5a630ac7`, `main` `671cee30`, `model` `881dd3c8`, `palette` `366457ba`,
`score` `34bac232`, `Cargo.toml` `be4cc4f7`, `script.tsv` `63573112`,
`oracle.py` `8695632d`, `brief.md` `2213d7c4`, `storyboard.md` `aa30898a`,
`storyboard-draft.md` `52d093cf`.

**What the patch changes in the draft** (both chapters only):

- 10 gets one new line, so 10 runs 10_1–10_8.
  - New 10_3 separates the illustration from the claim: "That was one
    player's likeliest two rounds, not the only way they can go. To see how
    common it is, go back to the whole crowd, settled at ten, four and
    twelve."
  - The old 10_3 becomes 10_4, and now counts the wins: "Each round, the
    good coin wins for three players at corner one and for nine at corner
    two. So of its twelve wins a round, nine happen at corner two, and every
    one of them just brings a player back to zero, where the bad coin
    waits."
  - The old 10_4–10_7 become 10_5–10_8, words unchanged.
- 11 gets one new line, so 11 runs 11_1–11_7.
  - 11_4 adds "still split exactly by the chances".
  - The old 11_5 splits in two. 11_5 ends at "then sloshes a little above
    and below that." New 11_6 is "Round after round the swings shrink, and
    the crowd settles. Corner zero keeps about thirty-four point six
    percent, and corner one ends up at twenty-five point four percent, up
    from fifteen point four."
  - The old 11_6 becomes 11_7.
- Every picture cell for 10 and 11 is rewritten to what the scenes draw,
  with new notes for 10 and 11.
- Advisory estimates from `tools/storyboard.py`: 10 rises from ~77 s to
  ~102 s, over the 85 s review mark (advisory; no cap), and 11 from ~67 s
  to ~78 s.

**One model.**

- 10 has two parts:
  - The player's path: the dollars 9 → 8 → 9 from 06's walk, with the
    corners by `corner_of`. The token moves on the arrows' own drawn
    shafts, through one tracker per move.
  - The crowd's claim: 10, 4, 12 and the flows 1, 3, 9 from
    `crowd.counts` / `crowd.forward`.
- The die game is `ROUND.die_chosen_win()` (= `facts::thirds_win` = 8/15).
- 11's crowd is three value trackers, one per corner, plus a round
  tracker. Each round glides to the next exact round of the mix from 10, 4,
  12. The bars, live shares, total, round number, puddles, streams and the
  trace all read those trackers, so the total stays 26. B's ghost heights
  are B's weights, fixed.

**What 10 does** (cue → picture; holds in bold):

- **10_1.** 09's picture is rebuilt: B's triangle with the round-number
  chances, the badge, "players: 26" and its label, and the 26 dots at 10,
  4, 12.
  - "collect extra players" rings corner 0 (hollow) as its dots pulse.
  - "Follow one": the crowd dims and the same teal token stands on corner 0,
    with "9" and "9 → remainder 0".
  - "one round later: which corner?" brings the countdown (**`THINK_S`**).
- **10_2.**
  - "At corner two": the token takes the back arrow 0 → 2 (1.6 s), with a
    red trace, and the counter becomes "8" and "8 → remainder 2".
  - "nine times in ten" outlines the "90 %" label.
  - "usually wins" outlines the "75 %".
  - "around to zero again": the token goes forward 2 → 0 with a green
    trace, back to "9".
  - **`BEAT` 1 s.**
- **10_3.** "illustration: one player's likeliest two rounds" (teal)
  appears. At "go back to the whole crowd" the token and counter give way to
  "players: 26", the dots return and the traces dim. "settled at ten, four
  and twelve" brings teal 10, 4, 12 beside the corners.
- **10_4.** "the good coin's wins each round:" heads the panel.
  - "corner 1: 75 % of 4 = 3", then "corner 2: 75 % of 12 = 9". Each corner
    is ringed as its winners pulse.
  - At "nine happen at corner two": the green TeX 9/(3 + 9) = 3/4.
  - "brings a player back": a teal stream, width ∝ 9 of 26, draws along
    2 → 0.
  - "where the bad coin waits" rings corner 0.
- **10_5.** "the trap: 0 → 2 → 0" (red). The traces return and pulse, and
  corner 0 is ringed again. **`TRAP_HOLD` 1.5 s.**
- **10_6.** The traces, stream, counts and wins leave, and "game B: your
  dollars pick the coin" appears. At "Let a die choose" a dark veil dims the
  triangle and "die game: a die picks the coin" appears. "instead of your
  money" outlines B's rule. Die faces 1 and 2 lead to the bad coin's
  "10 %", and faces 3–6 to the good coin's "75 %".
- **10_7.** "1/3 of rounds" and "2/3 of rounds" appear. "wherever you
  stand" rings all three corners through the veil. "this game wins" brings
  the TeX 1/3 · 10 % + 2/3 · 75 % ≈ 53.3 % and "the die game wins".
  **`VERDICT_HOLD` 1.5 s.**
- **10_8.** Rings go round both coins and both rules are outlined. At
  "Reading your money" the die, coins and veil leave and B's rule is
  outlined. "down to exactly half" brings "reading your money: 53.3 % →
  50 %", outlined. Stand, then the fade.

**What 11 does:**

- **11_1–11_3.**
  - 04's gold coin "A" appears under "the mix", with "heads → game A" and
    "tails → game B".
  - "every arrow becomes an average" pulses all six arrows. A's "50 %" is
    shown past corner 0's label, and B's "10 %" is outlined.
  - "forward thirty percent in all" swaps in "30 %" / "70 %" and the sum
    1/2 · 50 % + 1/2 · 10 % = 30 %.
  - 11_3 does the same for corners 1 and 2 (62.5 % / 37.5 %), and the
    title becomes "the mix".
- **11_4.** The coin, rules and sums give way to the panel: "round 0", the
  live "players: 26", the dim persistent **"split by the chances (expected
  numbers)"**, rows with live bars and one-decimal shares (38.5, 15.4,
  46.2), and the note "shares shown to one decimal; the total is exactly
  26".
  - "ten, four and twelve" merges the dots into puddles, and each bar gets
    a dim ghost outline at B's height, kept to the end.
  - "still split exactly by the chances" outlines the label.
  - "which way will the crowd shift?" brings the countdown (**`THINK_S`**).
    No settled tick, line or value is drawn during the prediction.
- **11_5.**
  - "Off corner zero" brings a small trace with no numbers: "share at
    corner 0, round by round". Corner 0 is ringed.
  - "Its share drops at once": round 1 plays slowly (2.2 s), with streams on
    all six arrows, and the shares read 34.6, 28.8, 36.5. Corner 1's bar
    shoots past where it will end.
  - "to about thirty-four point six" outlines corner 0's share.
  - "then sloshes": rounds 2–6 run at 1 s each, with streams, while the
    trace zigzags.
  - **Rounds 7–12 then play in silence, 0.5 s each, plus `BEAT` 1 s.**
- **11_6.** Rounds 13–18 play at 0.3 s each.
  - At "the crowd settles", dim ticks mark the settled heights, and a
    dashed "34.6 %" line crosses the trace.
  - At each "keeps" / "ends up", that row becomes fixed: "38.5 → 34.6 %",
    "15.4 → 25.4 %", "46.2 → 40.1 %".
  - "up from fifteen point four" pulses corner 1's ghost.
- **11_7.** "ignores the corners" rings all three corners. "toward equal
  thirds" brings dashed thirds ticks and a dashed "33.3 %" on the trace.
  "That shift of weight" adds a red outline, pulsing, filling the gap
  between corner 0's bar and its ghost. "where the winning comes from"
  outlines "38.5 → 34.6 %". Stand, then the fade.

**Emphasis.** No typeset text is flashed, scaled or recoloured.

- Words get `frame_around` + `point_at` outlines, and changed labels are
  `swap`ped.
- Corners get `ring_corners`' hollow rings; the dots, token, traces,
  streams, ghosts and the red outline pulse. A disc is never flashed.
- The veil is a stage rectangle, not emphasis.

**Model and fact bindings** (`numbers()` refuses to draw if any fails):

- **10:**
  - `crowd.counts` 10,4,12 and `crowd.forward` 1,3,9. The dollars are 9, 8,
    9 (`walk()[3]` + 1, then back and forward), with their printed
    remainders, and the path is 0 → 2 → 0.
  - The bad coin's 9/10 back, and the tails fact `coins.round.tails.pct`
    (A, bad, good: index 1).
  - The likeliest two-round path is 0 → 2 → 0, by `max_by_key` over all
    four.
  - The 3 + 9 = 12 wins, and `crowd.good_wins_at2` = 3/4 =
    `good_wins_at2(B)`.
  - The die: faces 1–2 and 3–6, 1/3 = `thirds.pct`. The die game is
    `ROUND.die_chosen_win()` = `facts::thirds_win` = 8/15, printing
    `round.b.thirds.win.pct`. B's own win is `round.b.win` 1/2.
  - At runtime: `under_veil` refuses if the veil reaches the badge or any
    part of B's picture sticks out of it; `keep_between`/`keep_apart` guard
    the rest.
- **11:**
  - The mix's six chances equal `a.mixed(&b, 1, 2)`'s and match
    `round.mix.coins.pct` 30,62.5 and `round.mix.tails.pct` 70,37.5. The
    coins' pcts and the two sums come from the same facts.
  - 26 players start at `crowd.counts`.
  - The exact rounds from 10, 4, 12 equal `mixshift.round.1..6`, and the
    float rounds' readouts equal them too. Conservation holds in every
    round.
  - `round.b.weights.pct` and `round.mix.weights.pct` are pinned.
  - Corner 0's round 1 reads 34.6, within 0.001 of the limit. There are at
    least two crossings, the swing shrinks from round 2, and corner 1
    rises.
  - The settling round is computed: `last` = 18, after `QUICK_TO` 12, by
    `settled_from` over `SETTLE_LOOK`.
  - A third < the mix's corner 0 < B's corner 0.

**Tests written (not run; Cargo is forbidden here):**

- `the_trap.rs`:
  - `the_trap_numbers_are_the_words` pins every built string.
  - `the_path_is_one_player_and_the_share_is_the_crowd`:
    - the four two-round paths are 27/40, 9/40, 3/40 and 1/40, so one
      player's path is likeliest, not certain;
    - controls: with the bad coin's win counted the share is 9/13, not 3/4,
      and a 70 % good coin gives 97/134;
    - the published coins print 75.0 % but are not exactly 3/4.
  - `the_die_changes_only_the_rule`: 8/15; the die game's crowd sits at
    thirds, B's does not; 317/600 with the published coins, where B's mean
    is below 0.
  - `the_token_walks_on_its_arrows`: headless. The trace's endpoints are
    the arrows' ends, and the drawn shaft contains every point of ours
    within 1e-6.
  - `every_phrase_the_scene_waits_for_is_in_its_line` (30 cues).
  - `the_layout_keeps_its_rooms`: constant asserts in `const { }`; corners
    inside the veil; counts beside their corners.
  - `the_chapter_runs_and_leaves_room_to_think`: 10_1–10_8 in order;
    `THINK_S` after 10_1, `BEAT` after 10_2, `TRAP_HOLD` after 10_5 and
    `VERDICT_HOLD` after 10_7.
- `mix_the_games.rs`:
  - `mix_the_games_numbers_are_the_words`, with `last == 18` (19 rounds).
  - `every_round_comes_from_ten_four_twelve`: exact against the facts,
    conservation, and float against exact. Controls: 26 on corner 0, and
    thirds, give other first rounds.
  - `the_crowd_sloshes_before_it_settles`: 34.6 in round 1, 5 crossings,
    corner 1's 28.8 overshoot, and `settled_from` = 18, not 17. Control: a
    monotone path has 0 crossings.
  - `this_mix_is_the_round_model_not_the_published_run`: the facts' 34.5 is
    the published run, and no drawn string contains "34.5". Controls:
    halfway between thirds and B is 35.9 ≠ 34.6; A alone keeps thirds.
  - `every_phrase_the_scene_waits_for_is_in_its_line` (26 cues).
  - `the_layout_keeps_its_rooms`: constant asserts, plus runtime checks on
    the longest bar and the trace's range.
  - `the_chapter_runs_and_leaves_room_to_think`: 11_1–11_7; `THINK_S` after
    11_4; the silent rounds plus `BEAT` (≥ 4 s) after 11_5; ≥ 1 s standing
    after 11_7.
- The cue, SPOKEN and headless tests read `crate::SCRIPT`, so they cannot
  pass until the patch is applied and the script regenerated.

**Checks actually run** (cheap only; repo root, 2026-10-24, scratch scripts
outside the repo):

- `rustfmt +nightly-2026-08-31 --edition 2024 --config-path rustfmt.toml
  --check` on the two new files, rerun on the final bytes: exit 0, no
  output. Only they were ever formatted. This is a parse check, not a
  compile.
- `checks_1011.py`: 63/63, written from the rules, not from `model.rs`.
  - It covers the two-round paths, 3/4 against 9/13 and 97/134, the die
    game's 8/15 and its thirds, and the mix from 10, 4, 12: round 1 34.615 %
    against the limit 34.556 %, five crossings in rounds 1–6, the 28.8
    overshoot, the shrinking swing, settling for good from 18 (not 17), and
    the 35.9 halfway control.
  - Run 1 was 62/63. I had expected the published coins to give exactly
    3/4 too. They print 75.0 % but are not exactly 3/4. The check and the
    10 notes now say so. Run 1's script and output are kept.
- `layout_1011.py`: 31/31. A width emulation (fmn's calibration with
  Computer Modern advances), not the render.
  - Tightest rows: 11's fixed settled rows end at −0.834 against the
    panel's −0.7, and 10's "10 %" chance ends at −2.478 against −2.35.
  - Run 1 also passed, but one row ("100.0") was unreachable and passed
    through a special case. It is replaced by the real longest bar, and
    run 1 is kept.
- `checks_1011b.py`: 70/70.
  - The 56 cue constants parsed from both sources are each in their
    proposed line exactly once, in order, and every line has a cue.
    Control: the live draft lacks the new cues, so the patch is needed.
  - 28 proposed SPOKEN rows match the oracle's emitted values (oracle exit
    0, "50/50 cross-checks passed", 161 facts).
  - Every number-word line has a row (10_6 and 11_1 have no number word).
  - No banned phrase. Control: the substring rule catches "drifts".
  - Hygiene: no `todo!`, `unimplemented!`, `dbg!`, prints, `#[ignore]`,
    home paths or lines over 120; the patch touches only the draft.
- `visibility_1011.py`: each of the 110 names the two files import, or
  reach through `facts::` / `model::` / `palette::`, has a `pub` or
  `pub(super)` declaration in its module. This is a text search, not name
  resolution. Control: 08's private `glide`, `arrow_curve`, `Crowd` and
  `streams` read as not visible, which is why 11 keeps local copies.
- The repo's `tools/storyboard.py`, imported read-only (only `parse()`,
  `check()` and `spoken()` are called; nothing is written): no problems on
  the patched copy. Estimates are 101.7 s and 78.0 s, total 22.6 min over
  18 chapters.
- The patch was checked by `verify_1011_patch.py` in both orders, each
  with `git apply` on scratch copies against an independent string splice.
  - Order A: the live draft plus 10/11.
  - Order B: the live draft plus the delivered 08/09 patch plus 10/11.
  - Both equal the splice, 08/09's sections are untouched, and the live
    draft is unchanged.
  - Two earlier attempts failed and are kept:
    - A 3-line-context version did not apply after 08/09, because 08/09
      adds a note just above "## 10_the_trap". The patch is now `-U1`.
    - Verifier run 1 failed order B on its own bug: it passed the 08/09
      patch path relative to the repo while `git apply` ran in the scratch
      copy. The paths are now resolved.

**Integration list for root's coordinated step** (not done here; after
08/09):

1. `src/chapters/mod.rs`: add `pub mod mix_the_games;` and `pub mod
   the_trap;`, plus
   `scene!("10_the_trap", 0.45, the_trap::TheTrap)` and
   `scene!("11_mix_the_games", 0.45, mix_the_games::MixTheGames)` after
   09's. The intensity is root's call.
   - Both import from `one_player_to_a_crowd` (08), so 08 must be
     registered first.
   - 11 imports `the_trap::shaft`.
   - Neither imports from 09.
2. Apply `docs/integration-08-09.patch`, then `docs/integration-10-11.patch`,
   to `storyboard-draft.md`. Copy the patched 10 and 11 sections into
   `docs/storyboard.md`, then run `python3 -I -B tools/storyboard.py
   parrondo`.
3. `src/main.rs`: change the doc line to "today 01–11" and add these
   SPOKEN rows:
   ```rust
   ("10_1", "corner zero", "rules.modulus", "3"),
   ("10_2", "nine times in ten", "coins.round.tails.pct", "50,90,25"),
   ("10_2", "around to zero again", "rules.modulus", "3"),
   ("10_3", "ten, four and twelve", "crowd.counts", "10,4,12"),
   ("10_4", "three players at corner one", "crowd.forward", "1,3,9"),
   ("10_4", "nine at corner two", "crowd.forward", "1,3,9"),
   ("10_4", "of its twelve wins a round, nine", "crowd.good_wins_at2", "3/4"),
   ("10_4", "back to zero", "rules.modulus", "3"),
   ("10_5", "a multiple of three", "rules.modulus", "3"),
   ("10_7", "one round in three", "thirds.pct", "33.3"),
   ("10_7", "fifty-three point three percent", "round.b.thirds.win.pct", "53.3"),
   ("10_8", "fifty-three point three percent", "round.b.thirds.win.pct", "53.3"),
   ("10_8", "exactly half", "round.b.win", "1/2"),
   ("11_2", "At corner zero", "rules.modulus", "3"),
   ("11_2", "A's fifty percent", "coins.round.heads.pct", "50,10,75"),
   ("11_2", "B's ten percent", "coins.round.heads.pct", "50,10,75"),
   ("11_2", "forward thirty percent in all", "round.mix.coins.pct", "30,62.5"),
   ("11_3", "between fifty and seventy-five", "coins.round.heads.pct", "50,10,75"),
   ("11_3", "sixty-two and a half percent", "round.mix.coins.pct", "30,62.5"),
   ("11_4", "ten, four and twelve", "crowd.counts", "10,4,12"),
   ("11_5", "Off corner zero", "rules.modulus", "3"),
   ("11_5", "thirty-eight point five percent", "crowd.share0.pct", "38.5"),
   ("11_5", "about thirty-four point six", "mixshift.round.1", "34.6,28.8,36.5"),
   ("11_6", "about thirty-four point six percent", "round.mix.weights.pct", "34.6,25.4,40.1"),
   ("11_6", "twenty-five point four percent", "round.mix.weights.pct", "34.6,25.4,40.1"),
   ("11_6", "up from fifteen point four", "round.b.weights.pct", "38.5,15.4,46.2"),
   ("11_7", "toward equal thirds", "thirds.pct", "33.3"),
   ("11_7", "off corner zero", "rules.modulus", "3"),
   ```
   11_2's "half the time" is the fair coin, which has no fact key. The
   line's other rows satisfy the per-line rule. A `rules.mix.fair` fact
   would be root's call.
4. Comments that cite the old id: `facts.rs:292`, `oracle.py:482` and the
   check message at `oracle.py:698` say "10_3" for what is now 10_4. Their
   "11_5" references stay correct. Nothing in the draft outside 10 and 11
   cites a renumbered id.
5. No `Cargo.toml`, `facts.rs`, `model.rs`, `glyphs.rs` or oracle change is
   needed. Every key above is already emitted (161 facts). The glyph
   probe collects every string the scenes typeset. ≈ appears only inside
   TeX (`\approx`), and the plain-text signs (→, ·, %, ?, parentheses) are
   already in `glyphs::DIGITS` or typeset by 06–09.

**Shared-helper proposals** (optional; not needed for correctness, and
no shared file was edited):

- 11 keeps local copies of 08's private `glide`, `Crowd` and `streams`,
  and `the_trap::shaft` repeats 08's private `arrow_curve`.
  - Making those `pub(super)` in 08 and deleting the copies would remove
    about 90 lines (11's lines 372–438 and `the_trap.rs` 429–451).
  - That edit touches the delivered 08, so it is root's.
- `rings_at` (10, 11) repeats 08's `ring_corners` for arbitrary centres. A
  `draw.rs` home would serve both.

**Scene risks for root's frame review** (none can be closed here):

- 10_4's teal stream runs over the dimmed green 2 → 0 trace.
- 11_5: corner 1's overshoot to 28.8 is seen only as bar motion. Its
  settled tick waits for 11_6, by design (no answer during the
  prediction).
- Live values and bars appear at full opacity when created, as in 08.
- The veil's coverage is guarded at runtime, but no frame of it has been
  seen.
- 11's trace spans 31.3–39.5 % over 1.9 units of height. Round 0's 38.5
  sits near its top.
  - Rounds 7–12 still swing 34.39–34.80, about 0.1 unit, which is
    visible.
  - Rounds 13–18 stay within 0.03 points of the limit (34.54–34.58),
    about 0.01 unit, so the trace looks flat there. By design: they are
    the settling rounds.
- 10 is estimated at about 102 s by words, plus the holds. Long by design
  (owner direction); no cap.

**Limits:**

- Nothing is compiled or run in Rust. APIs were read at the kit and fmn
  pin `a399d626`, not built against. Expect compile fixes at
  qualification.
- No frame rendered or looked at, no narration made, no listening.
  Pacing, clarity and English accent are unqualified.
- 34.6 % is the round-number coins' split-crowd model. The published
  run's 34.5 % (05_2) is a different measurement, and 11 never draws it.
- Pronunciation risks (in the patch's notes): "likeliest", "die" (the
  cube), "fifty-three point three", "sloshes", "thirty-four point six",
  "twenty-five point four".

## New unregistered chapters 08 and 09 (root's assignment, Mail 6891)

Not registered and not integrated: `mod.rs`, `main.rs`, `storyboard.md`,
`script.tsv` and the draft are untouched. Uncompiled. 09's words are the
draft's exactly. 08's are the draft's plus two sentences in 08_3 and the
old 08_7 split in two at the real crowd's silent rounds (08_7, 08_8),
proposed in the patch.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/one_player_to_a_crowd.rs` (`OnePlayerToACrowd`, "08_one_player_to_a_crowd") | `cf3e54fa82fdf28a09ada721a1773077b0ecc616291d524cfe6a51497d39d5ad` | 1653 | 9 |
| `src/chapters/ten_four_twelve.rs` (`TenFourTwelve`, "09_ten_four_twelve") | `c2ddfc6fea72cd9ac686d898a781d7e662e7112b599a932cbd8894fd23e00a4f` | 940 | 5 |
| `docs/integration-08-09.patch` (against `storyboard-draft.md` `52d093cf…`) | `a0e24eb88a2c4574fab832c7390311ee196e24da4c88585a62c1ade6ac29cdb5` | 93 | — |

Registered files at handoff, unchanged by this lane (prefixes):
`does_b_win` `1e22ff62`, `draw` `60024c49`, `glyphs` `9891c7e1`,
`looks_hopeless` `1751dde4`, `mod` `22ad9c91`, `the_clue` `b413c943`,
`the_puzzle` `24f0ef14`, `the_rule` `ad182b3d`, `three_corners`
`9917a027`, `where_you_stand` `fce6cc38`, `facts` `5a630ac7`, `main`
`671cee30`, `model` `881dd3c8`, `palette` `366457ba`, `score` `34bac232`,
`Cargo.toml` `be4cc4f7`, `script.tsv` `63573112`, `oracle.py` `8695632d`,
`brief.md` `2213d7c4`, `storyboard.md` `aa30898a`, `storyboard-draft.md`
`52d093cf`.

**One model.** The split crowd is four value trackers: the round
(`progress`) and one per corner. Each round glides all four in one play to
the next round of `model::evolve_f64`, from 26 on corner 0. The puddles,
the panel's `live_bar`s and live numbers, the live total, the round
counter, the share at corner 0 and its trace all read those trackers, so
the total reads 26 in every frame. Rounds 1–6 are checked against the
facts' `crowd.round.1..6` (exact `Ratio`s) before anything is drawn. The
real crowd is a local per-player replay of `model::real_crowd`, refused
unless its counts equal the model's in all 1,000 rounds and the facts' in
the eight shown. Its triangle, dots, counts, counter and tally read one
tracker of rounds. 09's share bar, its percentage and B's wins read one
tracker.

**What 08 does** (cue → picture; holds in bold):

- **08_1.** 07's end is rebuilt: both triangles, round-number chances,
  badge, counter "8", "8 → remainder 2", the token on B's corner 2, and
  07_7's hypothetical. "really a third each" outlines the hypothetical.
  "full of luck" pulses the token. At "To see past it", A and the
  hypothetical leave and the **same token** wins once more, 2 → 0, onto
  corner 0: "9", "9 → remainder 0". At "twenty-six copies" it becomes 26
  dots on corner 0, "26" beside them. The counter gives way to "round 0"
  and "players: 26". "all starting at corner zero" gets a hollow ring at
  corner 0. "all playing game B" outlines B's title.
- **08_2.** At "split exactly by the coins' chances" the dots merge into
  puddles, area ∝ players. The panel gets the rows "corner 0/1/2" with
  live bars and one-decimal numbers, the persistent dim **"split by the
  chances (expected numbers)"** and the note "shown to one decimal; the
  total is exactly 26". At "After one round" comes the countdown
  (**`THINK_S`**). Nothing moves before 08_3.
- **08_3.** At "At corner two", round 1 plays slowly (2.4 s), with
  streams on 0 → 1 and 0 → 2. "2.6" (green) appears at "A tenth" and
  "23.4" (dim) at "nine tenths", each past its chance label, which is
  outlined. At "not fractions of people" comes K04, "expected numbers: 26
  times one player's chance of being at that corner after that round; real
  players come whole and scatter around them". "are expected numbers"
  outlines the persistent label. A hold follows. The 9/10 back flow has
  2.4 s of motion plus the rest of the line before anything quickens.
- **08_4.** Round 2 (1.6 s) runs with streams. "the total stays
  twenty-six" outlines "players: 26" and its live total. Rounds 3–6 follow
  at 0.7 s each. Hold.
- **08_5.** K04 leaves after its reading time (15 chars/s from its
  fade-in). "Watch the share" brings the share row, the trace axes (rounds
  0–60) and the dashed "33.3 %". "where the bad coin waits" gets a hollow
  ring at corner 0. "swings wildly at first" outlines the trace, whose tip
  follows the trackers between whole rounds. At "steadies above a third",
  rounds 7–12 play at 0.5 s each. Hold.
- **08_6.** Rounds 13–60 play at 0.1 s each, linear. At each count phrase
  that row's live number becomes a fixed teal "10", "4", "12", outlined. At
  "Not a third each" the share becomes a fixed "38.5" and "within half a
  player of 10, 4, 12 from round 24" appears. The share row, then
  "33.3 %", are outlined. **Hold + `SETTLE_HOLD` 2 s.**
- **08_7.** The share row, trace, note and counter leave. "Real players
  don't split" outlines the persistent label. At "Toss real coins" the real
  triangle comes in at the lower left, labelled "real tosses, seeded
  illustration", with its own "round 0", 26 dots at 10, 4, 12 and a live
  count per corner. The split bars and puddles stay. After the line,
  **eight rounds hop in silence**, one a second (0.7 s move + 0.3 s
  rest).
- **08_8.** "exactly 10, 4, 12 so far:" appears with its live tally (0).
  At "In this run of a thousand rounds" the counter runs to 1,000 in 3 s,
  the dots snapping to each round's places. At "only forty times" comes
  "this run: exactly 10, 4, 12 in 40 of 1,000 rounds"; at "on average",
  "average over the 1,000 rounds: 10.0, 4.0, 12.0". **Hold +
  `RESULT_HOLD` 3 s.** The real crowd leaves, then a 1 s beat and the fade.

**What 09 does:**

- **09_1.** 08's end is rebuilt statically: triangle, chances, badge,
  "players: 26", label, bars and puddles at 10, 4, 12. At "really stays
  put" they become 26 dots, 10, 4, 12, at the corners. "still split
  exactly by the chances" outlines the label.
- **09_1–09_3, one corner per line** (a hold between lines). The corner
  gets a hollow ring while its dots pulse. At the forward phrase its
  forward group waits past the start of its arrow, on the bow side (outside
  the triangle forward, inside back), with its green count. At the back
  phrase the back group waits with its dim count: 1 and 9, then 3 and 1,
  then 9 and 3.
- **09_4.** At "How many players land" the question "how many land on each
  corner?" appears with the countdown (**`THINK_S`**). The dots wait and
  nothing is summed.
- **09_5.** All 26 dots move at once (2 s), straight to their arrows'
  corners, while the counts, the question and the countdown fade. At each
  "Corner … gets" its sum appears beside the corner, listed by origin with
  summands in stream colours: "1 + 9 = 10", "1 + 3 = 4", "9 + 3 = 12".
  These **incoming sums** are separate texts from the departing counts,
  which have gone. "exactly as they were" outlines each total. **Hold +
  `AS_WERE_HOLD` 2 s.**
- **09_6.** "Count the winners" pulses the 13 forward movers. "make
  thirteen" brings "winners: 1 + 3 + 9 = 13"; "exactly half of twenty-six"
  brings "13 of 26 = 1/2" (green). "perfectly fair" outlines the badge
  ("no handicap"). **Hold + `BEAT` 1 s.**
- **09_7.** At "where the crowd sits" a red ring goes round corner 0's dots
  and pulses. At "Ten of twenty-six" come "share at corner 0: 10 of 26" and
  a red bar growing to a live 38.5 %. At "more than a third" a dim mark
  "33.3 %" appears and pulses.
- **09_8.** The bar falls back to 33.3 % and "B wins 53.3 %" appears (live
  from the same share, 07_7's number). At "with the bad coin" the bar grows
  to 38.5 % while B's wins slide to 50.0 %. "cancel … exactly" outlines the
  row. Hold.
- **09_9.** At "is called its weight" comes "weight: a corner's long-run
  share of the rounds". At each "… thirteenths" that corner's sum gives
  way to "10/26 = 5/13", "4/26 = 2/13", "12/26 = 6/13". After a hold the
  names card "stationary distribution · Markov chain" stays for its
  reading time, then a stand and the fade. The words "stationary" and
  "weight" are not used before 09_9.

**Emphasis.** No typeset text is indicated, scaled or recoloured. Words
get `point_at` outlines and changed numbers are `swap`ped. `flash_all`
pulses only the token, dots, the red ring and the mark. Following root's
07 frame finding (Mail 6896: a filled pulse under the white thirds),
corners are pointed at with `ring_corners`: separate hollow rings whose
strokes pulse, the dots on them pulsing in the same play. A disc is never
flashed.

Model and fact bindings (`numbers()` refuses to draw if any fails):

- **08:**
  - 8 dollars (06's walk end, `walk()[3]`) is corner 2, and 9 is corner
    0, by `corner_of`.
  - 07_7's hypothetical equals `round.b.thirds.win.pct`.
  - `crowd.size` is 26 = `smallest_whole_crowd`.
  - Round B's chances are exactly 1/10, 3/4 and 3/4 (a tenth; three
    quarters); the badge equals `coins.round.heads.pct`.
  - `crowd.round.1..6` equal `crowd_rounds`, exactly and as printed.
  - The float rounds match the exact ones through round 6. Every round
    keeps 26 within 1e-9.
  - The counts settle at `crowd.counts` 10, 4, 12 with
    `crowd.share0.pct` 38.5, and the share is over a third.
  - `crowd.within_half_from` 24 equals `settles_from`.
  - `last` is the first round from which every readout shows its settled
    value through round `SETTLE_LOOK` (200): 60, pinned in a test.
  - The real crowd: `realcrowd.round.1..8`, `realcrowd.rounds` 1000,
    `realcrowd.exact` 40, `realcrowd.mean` 10.0, 4.0, 12.0.
  - At runtime: `keep_between`/`keep_apart` guards; K04 is refused above
    B's title.
- **09:**
  - `crowd.size`, `crowd.counts`, `crowd.forward` 1,3,9, `crowd.back`
    9,1,3 and `crowd.arrivals` 1+9,1+3,9+3 equal `model::flows` and
    `facts::arrivals`. They must be whole and come back to the counts.
  - The bad coin is at corner 0 and the good coin at 1 and 2.
  - `crowd.winners` 13 = half of 26 = `round.b.win`. Round B's mean is 0;
    published B's is below 0.
  - 10/26 is 38.5 %, over a third, and `thirds.pct` is 33.3.
  - B's wins: at a third they are `thirds_win(ROUND)` = 53.3 %
    (`round.b.thirds.win.pct`); at its share, `win()` = ½.
  - `round.b.weights` 5/13, 2/13, 6/13.
  - At runtime: `keep_between`/`keep_apart`; the names card is refused
    above the title. The landing places are refused unless each corner
    gets back exactly its count.

Tests written (not run; Cargo is forbidden here):

- `one_player_to_a_crowd.rs`:
  - `one_player_to_a_crowd_numbers_are_the_words` pins every string, the
    chances, 8 → 9 and `last == 60`.
  - `the_readouts_round_as_the_facts_print`. Negative control: Rust prints
    the float 5.85 as "5.8"; the facts and `shown` print 5.9. Rounds 1–6
    equal the facts.
  - `the_split_crowd_keeps_every_player`: 26 in every round to 60.
    Controls: forward flows alone leave 2.6 of 26; a third each does not
    stay put; 10, 4, 12 stays exactly; the published coins do not keep
    10, 4, 12.
  - `a_stream_lies_on_its_arrow`: headless; the stream's points equal
    `arrow_along`'s shaft within 1e-6.
  - `the_real_crowd_is_one_seeds_illustration`: counts equal
    `model::real_crowd`, every player steps ±1, the crowd leaves 10, 4, 12
    in round 1, the tally is 0 at 8 and 40 at 1,000. Control: seed 2000
    gives 51.
  - `the_card_fits_two_lines`.
  - `every_phrase_the_scene_waits_for_is_in_its_line` (30 cues).
  - `the_layout_keeps_its_rooms`: constant asserts in `const { }`, plus
    runtime name, real-corner and slot spacing.
  - `the_chapter_runs_and_leaves_room_to_think`: 08_1–08_8 once, in
    order; `THINK_S` after 08_2; `SETTLE_HOLD` after 08_6; at least 8 s of
    silent hops between 08_7 and 08_8; `RESULT_HOLD` at the end.
- `ten_four_twelve.rs`:
  - `ten_four_twelve_numbers_are_the_words`, including 53.3 → 50.0
    through `shown`.
  - `the_crowd_stays_put_and_the_controls_do_not`. Controls: a third each
    moves; forward-only gives 9, 1, 3; the published coins move. One real
    round returns exactly with probability 0.0707 (three binomials).
  - `every_phrase_the_scene_waits_for_is_in_its_line` (29 cues).
  - `the_layout_keeps_its_rooms`: groups on the correct side of their
    edge and clear of the disc; sums against names and the frame; constant
    asserts in `const { }`.
  - `the_chapter_runs_and_leaves_room_to_think`: 09_1–09_9; `THINK_S`
    after 09_4; `AS_WERE_HOLD` after 09_5; `BEAT` after 09_6; the card's
    reading time at the end.
- The cue, SPOKEN and headless tests read `crate::SCRIPT`, so they cannot
  pass until the patch is applied and the script regenerated.

Checks actually run (cheap only; repo root, 2026-10-24, scratch scripts
outside the repo):

- `rustfmt +nightly-2026-08-31 --edition 2024 --config-path rustfmt.toml
  --check` on the two new files: exit 0. Only they were formatted. This is
  a parse and layout check, not a compile.
- `checks_0809.py`: 33/33. It is written from the rules, not from
  `model.rs`, and covers:
  - the flows and rounds 1–6; conservation for 200 rounds;
  - settling and "within half a player from round 24";
  - flows, arrivals, 13 winners, fairness, 38.5 %, 53.3 % and 5/13;
  - the seed-1999 real crowd: the first eight rounds, 40, the averages;
  - the binomial 0.0707.
- `checks_0809b.py`: 88/88. The exact rounds 0–200 are Fractions,
  rounded half away; a float replay through the scenes' `shown` agrees in
  every round. It finds:
  - the counts settle for good from round 41, the share from round 60;
  - the closest readout to a rounding boundary in rounds 7–60 is 0.002
    tenths;
  - shown sums are 26.1 or 25.9 in 12 of rounds 1–60;
  - every real player moves one corner each round;
  - the tally is 0 at 8 and 40 at 1,000;
  - other seeds give 7→44, 42→27, 1998→43, 2000→51, 2001→27;
  - no real count reaches three digits (maxima 18, 11, 21);
  - slot spacing is 0.109 ≥ 0.10;
  - the 59 cue constants parsed from both sources are in their lines, in
    order, against the live draft plus the proposed words;
  - 41 proposed SPOKEN rows match the oracle's emitted values (oracle exit
    0, "50/50 cross-checks passed", 161 facts);
  - every number-word line has a row; no banned scope word.

  Run 1 was 87/88. I had expected plain float formatting to misprint some
  round, and it misprints none in rounds 0–200. The nudge in `shown` is a
  guard (the literal 5.85 control shows why), not load-bearing for these
  floats. The check now records that observation; run 1's script and
  output are kept beside it.
- `layout_0809.py`: 30/30. It is a width emulation, not the render:
  fmn's calibration (a "0" inks font_size/144 units, from
  `fmn-library/src/text.rs`) with Computer Modern advance widths. The
  tightest rows are corner 1's "4/26 = 2/13", 0.25 inside the frame edge,
  and 09's "38.5" against its "%", gap 0.14.
- The repo's own `tools/storyboard.py` parser and checker, imported
  read-only (no file written), on the live draft and the patched copy: no
  problems. 08 is estimated at 94.9 s (advisory; no cap) and 09 at 84.5 s.
  The headings become ~95 s and ~85 s.
- The patch: `git apply --check` and `git apply` on a scratch copy of the
  live draft give the proposed file byte for byte; the live draft is
  unchanged.
- Scans: no `todo!`, `unimplemented!`, `dbg!`, `println!` or absolute
  home path; no line over 120 characters; `unwrap` only in tests.
- Root's compile lessons (Mails 6892/6893) are applied to the new files:
  a local `fn words` instead of a closure returning borrows, constant
  asserts in `const { }`, and stage-geometry tolerances at 1e-6.

Integration list for root's coordinated step (not done here):

1. `src/chapters/mod.rs`: add `pub mod one_player_to_a_crowd;` and `pub mod
   ten_four_twelve;`, plus
   `scene!("08_one_player_to_a_crowd", 0.45, one_player_to_a_crowd::OnePlayerToACrowd)`
   and `scene!("09_ten_four_twelve", 0.45, ten_four_twelve::TenFourTwelve)`.
   The intensity is root's call. 08/09 import from `looks_hopeless`,
   `three_corners` and `draw` (all registered); 09 imports from 08.
2. Apply `docs/integration-08-09.patch` to `storyboard-draft.md`. Copy the
   patched 08 and 09 sections into `docs/storyboard.md`, then run
   `python3 -I -B tools/storyboard.py parrondo` (08_3 grows; 08_7 is split
   into 08_7 and 08_8).
3. `src/main.rs`: change the doc line's "today 01–07" to "today 01–09" and
   add these SPOKEN rows:
   ```rust
   ("08_1", "really a third each", "thirds.pct", "33.3"),
   ("08_1", "twenty-six copies", "crowd.size", "26"),
   ("08_1", "corner zero", "rules.modulus", "3"),
   ("08_3", "At corner two", "rules.modulus", "3"),
   ("08_3", "A tenth of the crowd", "coins.round.heads.pct", "50,10,75"),
   ("08_3", "nine tenths fall back", "coins.round.tails.pct", "50,90,25"),
   ("08_3", "Two point six and twenty-three point four", "crowd.round.1", "0.0,2.6,23.4"),
   ("08_3", "twenty-six times each chance", "crowd.size", "26"),
   ("08_4", "three quarters of corner two", "coins.round.heads.pct", "50,10,75"),
   ("08_4", "the total stays twenty-six", "crowd.size", "26"),
   ("08_5", "corner zero", "rules.modulus", "3"),
   ("08_5", "steadies above a third", "thirds.pct", "33.3"),
   ("08_6", "ten players on corner zero", "crowd.counts", "10,4,12"),
   ("08_6", "four on corner one", "crowd.counts", "10,4,12"),
   ("08_6", "twelve on corner two", "crowd.counts", "10,4,12"),
   ("08_6", "Not a third each", "thirds.pct", "33.3"),
   ("08_7", "these twenty-six", "crowd.size", "26"),
   ("08_8", "a thousand rounds", "realcrowd.rounds", "1000"),
   ("08_8", "exactly ten, four and twelve", "crowd.counts", "10,4,12"),
   ("08_8", "only forty times", "realcrowd.exact", "40"),
   ("08_8", "on average", "realcrowd.mean", "10.0,4.0,12.0"),
   ("09_1", "ten players toss the bad coin", "crowd.counts", "10,4,12"),
   ("09_1", "one wins and moves forward", "crowd.forward", "1,3,9"),
   ("09_1", "nine drop back to corner two", "crowd.back", "9,1,3"),
   ("09_2", "the good coin splits four players", "crowd.counts", "10,4,12"),
   ("09_2", "Three move on to corner two", "crowd.forward", "1,3,9"),
   ("09_2", "one falls back to zero", "crowd.back", "9,1,3"),
   ("09_3", "it splits twelve", "crowd.counts", "10,4,12"),
   ("09_3", "Nine wrap around to zero", "crowd.forward", "1,3,9"),
   ("09_3", "three fall back to one", "crowd.back", "9,1,3"),
   ("09_5", "one plus nine, ten", "crowd.arrivals", "1+9,1+3,9+3"),
   ("09_5", "one plus three, four", "crowd.arrivals", "1+9,1+3,9+3"),
   ("09_5", "nine plus three, twelve", "crowd.arrivals", "1+9,1+3,9+3"),
   ("09_6", "One, three and nine make thirteen", "crowd.winners", "13"),
   ("09_6", "exactly half of twenty-six", "round.b.win", "1/2"),
   ("09_6", "Without the half point", "rules.handicap", "1/200"),
   ("09_7", "Ten of twenty-six", "crowd.counts", "10,4,12"),
   ("09_7", "thirty-eight and a half percent", "crowd.share0.pct", "38.5"),
   ("09_7", "more than a third", "thirds.pct", "33.3"),
   ("09_8", "corner zero", "rules.modulus", "3"),
   ("09_9", "five thirteenths, two thirteenths and six thirteenths", "round.b.weights", "5/13,2/13,6/13"),
   ```
4. `glyphs.rs`: no change. The probe collects every string the scenes
   typeset; the new signs (`; ' ( ) ?` and letters) are plain Computer
   Modern text.
5. No `Cargo.toml`, `facts.rs`, `model.rs` or oracle change. Every key
   above is already emitted (161 facts).

Shared-helper proposals (optional, not needed for correctness):

- `ring_corners` (08) and root's `point_at_corners` (07, private) are the
  same hollow-ring pattern; one could move to `draw.rs`.
- 08's local `real_players` replays `model::real_crowd` per player. A
  `model::real_players` returning identities would remove the replay. The
  scene already refuses if they ever differ.
- 08 copies 07's private `MID_X`, `IF_Y`, `SUM_Y`, `WOULD_Y`, the round
  badge and the sum's TeX to rebuild 07's end; `pub(super)` in
  `where_you_stand.rs` would remove the copy.

Scene risks for root's frame review (none can be closed here):

- 09_5's straight moves. A nine-dot cluster (spread 0.26) passes within
  reach of the chance label 0.1 past each arrow's bow for a moment.
  Following the arrow would not avoid that. If frames show it, dim the six
  chance labels during the 2 s move and restore them.
- The live readouts (`always_redraw`) appear at full opacity when created.
  They cannot fade in, as in 03–07.
- Rounds 13–60 at 0.1 s are exact at 10 and 60 fps. Every play is ceiled
  to whole frames.
- The trace's x scale is 0–60 rounds, so rounds 0–6 use the left 10 %.
  The swings are clearest at 08_5, but early detail is small.
- During the 3 s run to 1,000 rounds the dots snap about 333 rounds a
  second. That reads as churn, by intent, not as motion to follow.
- Updaters may lag one frame behind the tracker they read (the trace tip
  and the real dots).
- `fade_out` with shift remains unqualified in frames. The width model is
  an emulation. The real guards are the headless run's `keep_between` /
  `keep_apart`, plus the kit's layout lint at holds.
- 08 is estimated at about 95 s by words alone, plus the silent hops and
  holds. Long by design (owner direction); no cap.

Limits:

- Nothing is compiled or run in Rust. Expect compile fixes at root's
  qualification. APIs were read at the kit and fmn pin `a399d626`, not
  built against.
- No frame rendered or looked at, no narration made, no listening.
  Pacing, clarity and English accent are unqualified.
- The 40 of 1,000 is one seed's count over successive (dependent) rounds.
  The scene and draft label it an illustration, not a probability or a
  typical result.
- Pronunciation risks (in the patch's notes): "a tenth", "nine tenths",
  "twenty-three point four", "sloshes", "thirteenths" three times in
  09_9, "thirty-eight and a half percent".

## Text emphasis repair in 04 and 05 (root's assignment, Mail 6879)

Root has seen doubled glyphs after indications on typeset text in native
scenes. `flash_all` is an fmn `indicate`, a Transform to a scaled,
recoloured copy. Eight calls aimed it at text. Each one now points with a
separate frame, or pulses a shape. No animation targets the words.
Spoken words, cue phrases, `CUES`, `numbers()`, computed values, every
other call and every prediction hold are unchanged. The diffs against the
before-copies show only the hunks listed here, plus the module docs, two
helpers, two constants and two tests. Still unregistered and uncompiled.

| File | before (as Mail 6868; root's `tend-0743/parrondo-04-05-intake` holds the same bytes) | after | lines | tests |
|---|---|---|---|---|
| `src/chapters/looks_hopeless.rs` | `1452978e53555a077a2ce78ced211520a955e64990834ab7008447c2775692d7` | `c919f8871ca838f14c5d29de90e96fa737f20ac11a9ba14399aee2fd18957d4e` | 924 → 1042 | 7 → 8 |
| `src/chapters/the_clue.rs` | `2352fc89ec6062d6b0b18a3a5eabed5b96b3df028a87c3034e17d1ed24c05139` | `b413c9436180947b6892ead49130378a9c2d18245085df550363498c2dd666d7` | 842 → 917 | 8 → 9 |

New helpers, in `looks_hopeless.rs` next to the other `pub(super)` pieces
05 shares. Shared `draw.rs` is not touched.

- `frame_around(stage, mobs, pad, color)` builds a rectangle on the box of
  `mobs` as typeset, `pad` outside it. Its stroke (`POINT_W` 2.0) starts at
  opacity 0. It refuses an empty list.
- `point_at(stage, frame, shapes, rt)` adds the frame and brings its stroke
  opacity 0 → 1 → 0 (`there_and_back`). In the same play it `indicate`s
  `shapes` (marks, never text), then removes the frame from the draw list.
  It is **one** play of `rt`. fmn rounds every play up to whole frames on
  the exact value of the f64, so two half-plays of 0.4 s run one frame
  longer than one 0.8 s play: 10 vs 9 frames at 10 fps, 50 vs 49 at 60 fps.
  A fade-in/fade-out pair would have shifted later timing by a frame.
- `POINT_PAD` is 0.07, the pad used by 07's outlines.

The eight sites (cue → old → new; same run time):

1. **04_3 "Two losers mix into a loser"** (`looks_hopeless.rs:705`). Old:
   `flash_all(&[merged_loses], 0.8)` on red "loses". New: a hero frame
   around "loses", `point_at(.., &[], 0.8)`.
2. **04_6 "well over a thousand dollars"** (`:769`). Old: `flash_all(&[end],
   0.8)` on "after 100,000 rounds: 1,748 dollars up". New: a hero frame
   around the readout, 0.8.
3. **05_1 "Nothing about the coins changed"** (`the_clue.rs:436`). Old:
   `flash_all(&tag, 0.8)` on the badge's back and words. New: an **ink**
   frame with pad 0 on the badge back's own edge, 0.8. It is ink rather
   than hero because a hero-outlined badge is 07's sign of the round-number
   coins, and here the narration says nothing changed.
4. **05_1 "In game B"** (`:441`). Old: `flash_all(&[b_mark[1]], 0.6)` on
   "game B 38.4 %". New: a hero frame around the label, with B's violet
   tick pulsing in the same play, 0.6.
5. **05_2 "thirty-four and a half percent"** (`:527`). Old:
   `flash_all(&[mix_words, percent, mix_mark], 0.8)` on two texts and the
   gold tick. New: one hero frame around the readout row. That frame is
   the box of "the mix:" and "%", and the live number stands between them.
   The gold tick pulses with it, 0.8. The live readout itself was never
   flashed and still isn't.
6. **05_2 "much closer to a third"** (`:530`). Old: `flash_all(&third, 0.8)`
   on the dim tick and "a third 33.3 %". New: a hero frame around the
   label, with the dim tick pulsing, 0.8.
7. **05_3 "Two games that each lose"** (`:559`). Old:
   `flash_all(&[bars.labels[0], bars.labels[1]], 0.8)`. New: one hero frame
   around both labels, "game A −1.00" and "game B −0.87", 0.8. The bars
   are not pulsed: an indicate would scale them away from their to-scale
   lengths and off the zero line.
8. **05_4 "the part that reads your money"** (`:632`). Old:
   `flash_all(&[count_line], 0.8)` after the existing hero outline came in.
   New: that outline itself pulses. Its stroke width goes 2.5 →
   `OUTLINE_PULSE_W` 5.0 → 2.5 (`there_and_back`, 0.8 s) in one play.
   There is no scale change, which would brush card B's title.

The `flash_all` calls left in 04 are all on discs: the mix's gold coin, the
two ordinary coins and the chooser. Each disc grows behind its face text,
which is never transformed. Root did not list these four.

New tests (written, **not run**):

- `looks_hopeless::tests::pointing_leaves_the_words_as_typeset` is a probe
  scene with real typesetting. It checks four things. The frame is the
  words' box plus the pad each side, with the same centre. `point_at` takes
  exactly as long as a `flash_all` of the same `rt` (both 0.8). Afterwards
  the words' box is bit-identical and the pulsed mark is back within 1e-9.
  The draw list holds the same mobjects as before, so the frame is gone and
  no copy is left behind. An empty frame request is an error.
- `the_clue::tests::the_pointing_frames_clear_their_ticks`, on real
  typesetting: B's label frame stays above B's tick, and a third's stays
  below its tick. These are the tightest clearances in the emulation, about
  0.05.
- The existing headless runs of 04 and 05 (`the_chapter_runs_and_leaves_room_to_think`)
  now pass through every new call. They still need root's script rows,
  because 04/05 are unregistered.

Checks actually run (2026-10-24, cheap only; commands from the repo root):

- `rustfmt --edition 2024 --config-path rustfmt.toml` on these two files
  only, then `--check` on them: exit 0. These files are not reachable from
  `main.rs` (unregistered), so this is a per-file parse, not a type check.
- A sha256 of every file under `explainer_corpus/parrondo` before and
  after: only these two files changed. All the frozen hashes match
  (`draw.rs` `60024c49…`, `mod.rs` `55debdca…`, `main.rs` `ad3306f3…`,
  `facts.rs` `5a630ac7…`, `model.rs` `881dd3c8…`, `storyboard*.md`,
  `script.tsv` `d5019cd0…`, 02/03 and 06/07 sources).
- `diff -u` against the before-copies (scratch copies; the same bytes as
  root's 04/05 intake). No change in the `AT_*` phrases, the `CUES` lists,
  `numbers()`, any computed string or any timing call except the eight
  sites.
- `grep` for `todo!|unimplemented!|dbg!|println!` and home paths: none.
  `grep flash_all(|indicate(`: only the four disc flashes, the
  `point_at` shape indicate, and the probe test's tick.
- A scratch Python emulation of every frame's clearance, using the
  estimated text boxes from the 04/05 checks: **30 checks, 0 failed**.
  - Every frame is inside the lint margins. The badge edge's top is 3.79
    against the 3.85 limit.
  - "loses" is 0.118 clear of the merged disc and 0.342 clear of the sum.
  - The end readout is 0.268 below the badge and above the run plus the
    token.
  - B's label frame is 0.049 above its tick. It is clear of "40 %", the
    think ring and the readout row.
  - The readout row spans its number and is 0.247 below the caption.
  - The third's frame is 0.049 below its tick. It is clear of "32 %", the
    window, "dollars" and the gold mark.
  - The losers' frame ends at 0.370, left of A's bar at 0.500, and is
    0.436 clear of "halfway".
  - A mutant with pad 0.15 fails the two tick clearances, exit 1, as it
    should.
  - The rerun of the earlier 04/05 numbers/words check: ALL PASS.

Not done, and left to root:

- The Rust is not compiled or type-checked. Strict Clippy and the probe
  tests have not been run.
- No frame has been rendered or looked at. The emulation estimates text
  boxes; it does not measure typeset glyphs.
- Root has not checked whether a hero frame reads as "pointing" at 4K, or
  whether there_and_back's single peak holds the frame long enough to see.
  `there_and_back_with_pause` takes two arguments, so it cannot be passed
  as a rate function here.
- Whether the badge's ink edge reads clearly enough on the faint fill is
  unjudged.
- `docs/storyboard-draft.md` (frozen) still says the badge, the readout
  and the verdict "flash". The wording can follow at root's integration;
  the WORK_STATUS descriptions of 04/05 below keep the delivered wording
  for history.
- `explainer_corpus/parrondo/.ee/hook-state/session_start_orient.last.json`
  (mtime 03:36 local, before this work) is a session-hook file in the
  topic. This lane did not write it. It is not in any handoff list and was
  left untouched.

## New unregistered chapters 04 and 05 (root's assignment, Mail 6853)

Not registered and not integrated: `mod.rs`, `main.rs`, `storyboard.md` and
`script.tsv` do not mention them. Uncompiled. The words are the full draft's
04 and 05 lines exactly. The pictures follow the draft, with the deltas
listed below.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/looks_hopeless.rs` (`LooksHopeless`, "04_looks_hopeless") | `1452978e53555a077a2ce78ced211520a955e64990834ab7008447c2775692d7` | 924 | 7 |
| `src/chapters/the_clue.rs` (`TheClue`, "05_the_clue") | `2352fc89ec6062d6b0b18a3a5eabed5b96b3df028a87c3034e17d1ed24c05139` | 842 | 8 |

What the scenes do, on the same player and the same seeded runs:

- **04_1.** The player's token, "player", the count "7" and "dollars" stand
  where 01 to 03 left them; the full badge sits top right. At "flips a fair
  coin" a gold coin appears above the token with face "A". "heads → game A"
  (blue) comes in, then the face turns to "B" with "tails → game B"
  (violet). At "Call this the mix" the name "the mix" appears in gold.
- **04_2 and 04_3.** These are the controlled counterexample, built before
  any game number. Two ink coins, "49 %" and "48 %", each carry a red
  "loses", and each flashes at its number. At "Choosing between them" a gold
  chooser disc appears with an arrow and "1/2" to each coin. At "like one
  coin" the faces, verdicts and chooser fade out first, so no text is ever
  under a moving disc. The two discs then fly together and become one coin
  "48.5 %", red "loses", with
  `\tfrac{1}{2} \cdot 49\% + \tfrac{1}{2} \cdot 48\% = 48.5\%` beneath it.
  At "Two losers mix into a loser" the verdict flashes. Every figure comes
  from `Game::mixed` and the facts. The scene refuses to draw if the merged
  coin is not below a half.
- **04_4 and 04_5.** Bars are drawn to scale about a zero line, under the
  caption "long-run average per round, cents". "game A −1.00" grows at its
  cue, then "game B −0.87", both growing from the zero line leftward. A
  dashed gold bar "halfway: −0.93 ?" follows. Then the prediction
  countdown; the dashed bar waits.
- **04_6.** The bars, rules, count, "player" and "dollars" leave. The token
  and the gold coin stay. The long window has ticks −1,000, 0 and +1,000,
  with A's and B's whole runs faint and labelled "A" and "B". The token
  flies to round 0 at 7 dollars and rides the tip of the mix's gold run for
  6.5 s. The run's end gets "mix", and above the window
  "after 100,000 rounds: 1,748 dollars up" appears and flashes at "well over
  a thousand dollars". Then the scene holds.
- **05_1.** The same three runs come back lower, in a shorter window (centre
  y −1.6) under a share scale from 32 % to 40 %. A dim mark "a third"
  stands on the scale. The full gold mix run is shown, with the token at its
  end. The badge flashes at "Nothing about the coins changed". B's mark
  "game B 38.4 %" (violet) comes in at "In game B". At "In the mix" the gold
  label "the mix:" appears with **no number**, then the countdown.
- **05_2.** At "the player stood" the mix's run dims (stroke only) and the
  token flies back to its start. The run replays, and one tracker drives the
  path, the readout and a gold mark on the scale. The share is always of
  exactly the rounds drawn so far, the same rule as 03. There is no number at
  round 0. The number comes in at the first drawn point, 250 rounds, and
  ends at 34.5 % (34,520 of 100,000). The share flashes at "thirty-four
  and a half percent", and the third's mark at "a third".
- **05_3.** Everything but the badge leaves. The bars return with the
  halfway guess. A and B flash at "each lose". At "mixed by a fair coin" a
  red strike crosses "halfway: −0.93 ?", and the label and dashes dim. At
  "a game that wins" the mix's bar grows rightward from zero to "the mix
  +1.57". Then the K02 lower third holds for its reading time, everything
  but the badge clears, and the title stands for at least 2.5 s: "Two
  Losing Games Can Win" (ink, 56 px) / "Here is the catch" (hero, 34 px).
- **05_4.** Card B returns at the centre, with its bad and good coins and
  both remainder rules. At "the part that reads your money" its line "count
  your dollars" is outlined in hero and flashes. At "where the player
  stands" the token and "player" return to their usual place beside it.
  Then the scene holds.

Model and fact bindings (each scene's `numbers()` refuses to draw if one
fails):

- **04:**
  - The ordinary coins are 49/100 and 48/100, mixed through
    `Game::mixed`, so 48.5 % and losing; they match
    `ordinary.coins.pct`/`ordinary.mixed.pct`.
  - `averages()` requires A = −1/100 and A < B < 0, with the halfway
    average below 0 and the mix's above. The cents labels must match
    `pub.{a,b,mix}.mean.cents`, and the badge text must match
    `coins.published.heads.pct`.
  - The run's net is 1,748, from `run.mix.net`, and must exceed 1,500 for
    "well over a thousand".
- **05:**
  - The mix's coins are the per-corner average of A's and B's.
  - The shares are from the seeded runs: B 38.4 % (more than 38) and the
    mix 34.5 %, each equal to its `run.*.at0.pct` fact. Then
    3·(mix − ⅓) < B − mix, the "much closer to a third" claim.
  - The mix's exact mean is between 1.5 and 1.6 cents.
  - K02's three fractions are checked against the `pub.*.mean` facts.
  - Every share is inside the scale.

Tests written (not run; Cargo is forbidden here):

- `looks_hopeless.rs`:
  - `looks_hopeless_numbers_are_the_words`.
  - `two_losing_coins_always_mix_into_a_loser` (all 49 × 49 ordinary pairs,
    plus the facts' 7,203-case sweep with 0 winners).
  - `the_average_of_the_averages_is_not_the_mix` (the negative control:
    −0.93 < 0 < +1.57).
  - `the_bars_are_drawn_to_scale`.
  - `every_phrase_the_scene_waits_for_is_in_its_line`.
  - `the_layout_keeps_its_rooms`: constant asserts in a `const { }` block;
    signed distances keep each "1/2" clear of its arrow; the window, ticks
    and end labels clear the token at both ends of the run; the readout
    floor clears the run's highest point.
  - The headless run: 04_1 to 04_6 once, in order, with at least `THINK_S`
    after 04_5.
- `the_clue.rs`:
  - `the_clue_numbers_are_the_words`.
  - `the_readout_follows_the_drawn_rounds`: none at round 0; 4,001 tracker
    values against an independent running hand count; ends at 34.5 %.
  - `the_mark_stays_on_the_scale_and_ends_near_a_third`.
  - `the_share_not_the_coins_decides`, the negative control. The mix's own
    coins, weighted by B's occupancy, win 0.49529 < ½. Weighted by the
    mix's own occupancy they win 0.50781 > ½. Weighting by the exact
    stationary crowd gives the exact win chance.
  - `the_card_fits_three_lines` (K02 wrapped at 76 characters, at most three
    lines).
  - `every_phrase_the_scene_waits_for_is_in_its_line`, and `the_layout_keeps_its_rooms`: constant asserts
    in `const { }`; run, scale, end-label and window checks.
  - The headless run: 05_1 to 05_4, with the think time after 05_1 and at
    least `TITLE_HOLD` for the title.

Checks actually run (cheap only; from the repo root):

- `rustfmt --check --edition 2024 --config-path rustfmt.toml` on the two new
  files: exit 0. This is a parse and layout check, not a compile. Only these
  two files were formatted. The hashes of every existing parrondo file are
  unchanged by it, among them `does_b_win.rs` `1e22ff62…` and `the_rule.rs`
  `ad182b3d…` as root left them.
- `python3 -I -B <scratch>/checks_0405.py explainer_corpus/parrondo`:
  ALL PASS. It is an independent Python emulation that imports `oracle.py`
  read-only and parses the draft's 04/05 words. It covers:
  - the numbers and both negative controls;
  - bars to scale;
  - the run bounds inside WIDE and CLUE;
  - the readout floor;
  - the end labels and ticks against the token radius;
  - the "1/2" signed distances;
  - the 05 replay prefix logic (none at round 0);
  - K02's wrap;
  - estimated text widths;
  - all 27 cue phrases;
  - the proposed SPOKEN phrases;
  - number-word coverage of every 04/05 line;
  - no banned scope words.
- The 13 proposed SPOKEN values below equal `oracle.py`'s output (run
  2026-10-24), and each key is registered in `facts.rs`.
- Scans: no `todo!`, `unimplemented!`, `dbg!`, `println!` or absolute home
  path in either file.

Draft picture deltas, for root to copy into the picture cells when 04/05
enter `docs/storyboard.md` (the draft is frozen here):

- **04_1:** the coin's faces are background-coloured letters on gold. Blue
  and violet are carried by the two rule lines "heads → game A" and "tails →
  game B". The token with "player", "7" and "dollars" and the full badge
  are on screen.
- **04_3:** a gold chooser disc with an arrow and "1/2" to each coin; the
  sum `½·49 % + ½·48 % = 48.5 %` under the merged coin.
- **04_4:** the caption "long-run average per round, cents".
- **04_6:** the readout reads "after 100,000 rounds: 1,748 dollars up",
  not "+1,748 dollars". The window has ticks −1,000/0/+1,000 and the end
  labels "A", "B" and "mix". The gold coin stays.
- **05_1:** the runs in a lower window under a 32–40 % scale; the
  dim mark "a third 33.3 %"; B's mark "game B 38.4 %"; "the mix:" without a
  number.
- **05_2:** the mix's run replays, the token riding its tip. A gold mark
  moves along the scale. There is no share at round 0; the number comes in
  with the first 250 rounds.
- **05_3:** a red strike and dimming on the halfway bar; the mix's bar grows
  rightward from zero. The badge stays through the card and title. K02 is
  shortened to fit three lines: "exact long-run averages, dollars a round:
  game A −1/100, game B −73443/8446300, the mix +223/14200; a run's average
  settles on these with probability one (Ethier & Lee, 2009)". The coin
  attribution moves to the brief and the badge, so "Notes for 05" needs that
  K02 text.
- **05_4:** card B at (0.5, 0.2); the player returns at its usual place.

Integration list for root's coordinated step (not done here):

1. `src/chapters/mod.rs`: `pub mod looks_hopeless;` and `pub mod the_clue;`,
   and `scene!("04_looks_hopeless", 0.45, looks_hopeless::LooksHopeless)`
   and `scene!("05_the_clue", 0.45, the_clue::TheClue)`. The intensity is
   root's call; 01 to 03 use 0.45.
2. Copy the draft's 04 and 05 sections, with the picture deltas above, into
   `docs/storyboard.md` (and the same cells in `storyboard-draft.md`, which
   `the_produced_chapters_are_the_drafts` requires). Then run
   `python3 -I -B tools/storyboard.py parrondo`. The cue and headless tests
   read `crate::SCRIPT`, so they fail until then.
3. `src/main.rs` SPOKEN rows (values checked against the oracle):
   ```rust
   ("04_2", "forty-nine percent", "ordinary.coins.pct", "49,48"),
   ("04_2", "one forty-eight", "ordinary.coins.pct", "49,48"),
   ("04_3", "forty-eight and a half percent", "ordinary.mixed.pct", "48.5"),
   ("04_4", "Game A loses a cent a round", "pub.a.mean.cents", "-1.00"),
   ("04_4", "a little under a cent", "pub.b.mean.cents", "-0.87"),
   ("04_6", "a hundred thousand rounds", "run.rounds", "100000"),
   ("04_6", "well over a thousand dollars", "run.mix.net", "1748"),
   ("05_1", "a multiple of three", "rules.modulus", "3"),
   ("05_1", "more than thirty-eight percent", "run.b.at0.pct", "38.4"),
   ("05_2", "a multiple of three", "rules.modulus", "3"),
   ("05_2", "thirty-four and a half percent", "run.mix.at0.pct", "34.5"),
   ("05_2", "closer to a third", "thirds.pct", "33.3"),
   ("05_3", "one and a half cents a round", "pub.mix.mean.cents", "+1.57"),
   ```
   Then change the doc line "today 01–03" to "today 01–05".
4. `src/chapters/glyphs.rs`: no new strings are needed, since TITLE and
   SUBTITLE are already probed. Its comment could say chapter 05 now
   typesets them.
5. No `Cargo.toml` or workspace change.

Shared-helper proposal (a proposal only; draw.rs is frozen):

- 03 and 05 each carry private copies of `at_zero_before`, `drawn_index`,
  `shown_share` and `first_drawn`; one `pub(super)` copy in `draw.rs` would
  do.
- Other candidates:
  - a card-B builder (03 and 05);
  - a stroke-only dim, `set_stroke(None, None, Some(o))` wrapped with its
    run time, now repeated in 03 and 05;
  - the player's place and the badge text (01 to 05).
- None is needed for correctness.

Limits:

- Nothing is compiled or run in Rust. Expect compile fixes at root's
  qualification. APIs were checked by reading the kit and fmn pin
  `a399d626`, not by building.
- The cue, SPOKEN and headless tests need the 04/05 script lines, so they
  cannot pass until step 2.
- Text widths are estimates from the glyph calibration. The real guard is
  `keep_between` in the headless run.
- No frame has been rendered or looked at, no narration made, no listening.
  Pacing, clarity and English accent are unqualified.
- Pronunciation risk: "forty-eight and a half" (04_3), as the draft notes.

## New unregistered chapters 06 and 07 (root's assignment, Mail 6870)

Not registered and not integrated: `mod.rs`, `main.rs`, `storyboard.md` and
`script.tsv` do not mention them. Uncompiled. The words are the full
draft's 06 and 07 lines exactly; the pictures follow the draft with the
deltas listed below.

| File | sha256 | lines | tests |
|---|---|---|---|
| `src/chapters/three_corners.rs` (`ThreeCorners`, "06_three_corners") | `2979146f4d5f8d477428aa037135929d8e359f5cd812c0e6f295719a231d5a1c` | 1028 | 6 |
| `src/chapters/where_you_stand.rs` (`WhereYouStand`, "07_where_you_stand") | `50294b7be53ccbc5dbc2cdf48bb23c2b071e2db05bbe2954a60a51285a58c01e` | 778 | 8 |

`three_corners.rs` also holds the triangle geometry and builders 07 reuses
(`pub(super)`): corners, arrows and their label places, names, titles,
chances, the counter and remainder line, `swap` (a text replaced by
cross-fade, never transformed) and `keep_apart` (a runtime refusal when two
typeset texts come within a given room). It imports the player's place
and the badge from `looks_hopeless.rs`, so 06/07 need 04's module
registered too.

What the scenes do, on the same player:

- **06_1.** The player as 02 left them: the teal token, "player", "7
  dollars", the seven chips in 02's rows of three with the leftover row
  outlined in hero, and the published badge. At "only ever looks at the
  remainder" the two full rows dim and the outline flashes (a shape). At
  "track just that" every chip and the big count slide and fade into a small
  counter "7 dollars" at the top left. At "Draw three corners" come three
  faint edges and the corner discs: 0 at the top tinted red, 1 lower right,
  2 lower left. The names "0", "1", "2" come in at "zero, one and two". The
  token is brought in front of the discs.
- **06_2.** At "leave one over", "7 → remainder 1" appears under the
  counter. At "stands at corner one" the token itself steps onto corner 1
  ("player" fades) and the disc flashes. At "A win moves them" the green
  forward arrow 1 → 2 is drawn. At "to corner two" the token steps, and the
  counter and its line are replaced: "8", "8 → remainder 2".
- **06_3.** The prediction. The token waits on corner 2 and the countdown
  stands on the left, where A's triangle will be. No arrow leaves corner 2.
- **06_4.** At "Around to corner zero" the green arrow 2 → 0 is drawn and
  the token steps; the counter turns "9" and the old remainder line goes.
  At "nine leaves nothing over", "9 → remainder 0" comes in red and corner
  0 flashes. At "A loss always moves" the dim back arrow 0 → 2 inside the
  triangle is drawn and the token steps back: "8", "8 → remainder 2". The
  other three arrows follow and the faint edges go. Then the line's breath
  and a 1 s beat.
- **06_5.** "game B" (violet) appears under the triangle. Each label sits
  beyond its arrow's bow, placed by its typeset size. Corner 0 flashes at
  "the bad coin"; its two labels (red) come in at "forward nine and a half":
  0→1 "9.5 %" and 0→2 "90.5 %". Corners 1 and 2 flash at "the good coin";
  at "forward seventy-four" come 1→2 and 2→0 "74.5 %" (green), and 1→0 and
  2→1 "25.5 %" (dim).
- **06_6.** At "Game A is the same", game A's triangle appears on the
  left, at the same size, as card A was: blue forward arrows, dim back
  arrows, "game A". The forward labels "49.5 %" (blue) come in at "forward
  forty-nine", the back labels "50.5 %" (dim) at "back fifty and a half".
  Both triangles then stand for at least 4 s from the last label, and the
  scene fades.
- **07_1.** The scene rebuilds 06's end: both triangles labelled, the
  token on B's corner 2, the counter "8", "8 → remainder 2", the
  published badge. At "six dollars" and "six hundred", "6 → remainder 0"
  and "600 → remainder 0" (red) appear between the triangles' top corners.
  At "plays the same way" both slide and shrink into B's corner 0, which
  flashes. Then the K03 "precisely" card comes in. At "stand at each corner"
  B's corners flash in turn. The card stays for its reading time (133
  characters at 15 a second, 8.9 s from its fade-in) before 07_2.
- **07_2.** A's corners flash at "play only game A". The countdown stands
  between the triangles. No share is shown.
- **07_3.** At "One third" the countdown goes and "1/3" appears in each of
  A's corners, from the model's shares of game A. They flash at "share the
  rounds equally". At "game A run spent", under A's title, comes "the
  player's 100,000 rounds" (dim) over "33.5 % at corner 0" (blue), and A's
  corner 0 flashes.
- **07_4.** At "does not treat its corners alike", hero outlines (separate
  rectangles) are drawn round B's two corner-0 labels. At "more than
  thirty-eight percent", under B's title, comes "the player's 100,000
  rounds" over "38.4 % at corner 0" (violet), and B's corner 0 flashes. The
  two readouts then stand side by side for the breath and 1.5 s.
- **07_5.** The readouts and outlines leave. At "make the numbers rounder"
  the three coins appear at the right: A's (blue, "A"), the bad coin (red)
  and the good coin (green), each with its chance, "49.5 %", "9.5 %",
  "74.5 %". At "half a point shaved off" an outline is drawn round the three
  chances. At "Put the half point back" a hero "+0.5" (from the handicap
  1/200) rises and fades over each chance while the chances are replaced by
  "50 %", "10 %", "75 %". In the same moment the filled published badge is
  replaced by the outlined "round-number coins · 50 · 10 · 75 % · no
  handicap". Then the breath and a 1 s turn.
- **07_6.** The arrows relabel with the round coins, each label replaced,
  never transformed: A's six labels become "50 %" (A's coin flashes); B's
  corner-0 pair becomes "10 %" and "90 %" (the bad coin flashes); B's other
  four become "75 %" and "25 %" (the good coin flashes).
- **07_7.** The badge's outline flashes at "our worked example". At "With
  equal thirds", between the top corners, comes "with equal thirds at every
  corner:" (dim) over `\tfrac{1}{3} \cdot 10\% + \tfrac{2}{3} \cdot 75\%
  \approx 53.3\%`; at "would now win", "B would win" (green). Then the
  breath and a 3 s thinking hold, and the fade. 08 starts from both
  triangles, the token on B's corner 2 and 8 dollars.

Nothing in 06/07 indicates or recolours text. `flash_all` targets only
discs, coin discs, the leftover outline and the badge's rectangle. A changed
number or label is a new text cross-faded in by `swap`, and a pointed-at
label gets a separate outline.

Model and fact bindings (each scene's `numbers()` refuses to draw if one
fails):

- **06:**
  - `rules.modulus` = 3 = `MODULUS`.
  - The walk is `walk()` = START, +1, +2, +1, so 7, 8, 9, 8. Each count's
    remainder must equal the `remainder.examples` fact. The corners must be
    1, 2, 0, 2, and each step must be exactly one corner forward or back.
  - The arrows' chances come from `PUBLISHED.game_a()`/`game_b()`. They must
    read 9.5/74.5/74.5 and 49.5/50.5 and equal `coins.published.heads.pct`
    and `coins.published.tails.pct`.
  - The badge text comes from `badge_text`.
  - At runtime the scene refuses if a counter, line or label leaves its band
    (`keep_between`), if corner 0's name reaches the badge, or if two texts
    come within 0.08 (`keep_apart`, at 06_5 and 06_6).
- **07:**
  - 6 and 600 must leave 0 by the facts. K03's −1 → 2 and −3 → 0 are read
    from `remainder.examples`, with the typographic minus.
  - Game A's shares are exactly ⅓ for the published and the round coins;
    `thirds.pct` = 33.3.
  - Each readout is rebuilt from `run.{a,b}.at0.rounds` over `run.rounds`
    (100,000) and must equal `run.{a,b}.at0.pct`. A's must be "33.5"; B's
    must be over 38 % and under 39 %.
  - Every round coin must be its published coin plus `rules.handicap`
    (1/200, "0.5"), and every round percentage a whole number. The legend
    must equal `coins.{published,round}.heads.pct`.
  - Round A must have chance ½ and mean 0. Round B must read 10/75/75.
  - `thirds_win(&ROUND)` must be "53.3" = `round.b.thirds.win.pct` and more
    than ½. The sum uses `\approx`, chosen like 03's because 8/15 is not a
    whole tenth.
  - At runtime the scene refuses if a middle-column text reaches either
    corner-0 disc, if "1/3" overflows its disc, if K03 reaches the titles,
    or if two texts come within 0.08 (at 07_1, 07_4 and 07_7).
- No weight of B's corners is shown. The negative control
  `equal_thirds_is_not_where_b_stands` uses them only in a test.

Tests written (not run; Cargo is forbidden here):

- `three_corners.rs`:
  - `three_corners_numbers_are_the_words`.
  - `the_corners_are_the_remainders`: −30..30 one corner per win and loss,
    both wraps; the walk uses arrows 1→2, 2→0 and back 0→2; every arrow
    runs between its corners; 0 is at the top, and 0 → 1 → 2 runs clockwise.
  - `a_count_below_zero_still_stands_on_a_corner`, the negative control:
    Rust's `%` gives −1 for −1 dollars, which is no corner, while the
    remainder and the facts give 2, and −3 gives 0.
  - `every_phrase_the_scene_waits_for_is_in_its_line` (19 cues).
  - `the_layout_keeps_its_rooms`:
    - forward labels outside and back labels inside, beyond the bow;
    - no two labels, names or titles within 0.08, at a label width of 0.7
      (30 % over the calibrated "90.5 %");
    - everything inside the frame, under the badge and above a two-line
      lower third;
    - the counter left of A's corner 0;
    - 02's rows and both parts of the countdown clear of B;
    - constant asserts in `const { }`.
  - Headless: 06_1 to 06_6 once, in order; at least `THINK_S` after 06_3;
    the beat before 06_5; at least 1 s standing after 06_6.
- `where_you_stand.rs`:
  - `where_you_stand_numbers_are_the_words`.
  - `the_half_point_is_the_only_change`: each coin and each corner's chance
    in both games moves by exactly 1/200; round A is fair.
  - `equal_thirds_is_not_where_b_stands`, the negative control. With the
    published coins, thirds give 52.8 %, yet B's mean is below 0 and its
    run's share is over ⅓ + 4 points. With the round coins, thirds give
    8/15, yet B's real win is exactly ½, its mean 0, and its corner-0 share
    above ⅓.
  - `game_a_shares_its_corners_equally_for_any_coin` (a = 1..199 of 200).
  - `the_card_fits_two_lines` (K03 at 72 characters).
  - `every_phrase_the_scene_waits_for_is_in_its_line` (19 cues).
  - `the_layout_keeps_its_rooms`: the middle column, legend, "+0.5" rise
    and readouts; constant asserts in `const { }`.
  - Headless: 07_1 to 07_7 once, in order. At least `THINK_S` after 07_2,
    K03's reading time before 07_2, `COMPARE_S` after 07_4, `TURN` after
    07_5, and at least `THINK_HOLD` after 07_7.

Checks actually run (cheap only; from the repo root, 2026-10-24):

- `rustfmt --check --edition 2024 --config-path rustfmt.toml` on the two new
  files: exit 0. This is a parse and layout check, not a compile. Only
  these two files were formatted; every other parrondo file's hash is
  unchanged (`looks_hopeless.rs` `1452978e…`, `the_clue.rs` `2352fc89…`,
  `does_b_win.rs` `1e22ff62…`, `the_rule.rs` `ad182b3d…`, `draw.rs`
  `60024c49…`, `mod.rs` `55debdca…`, `main.rs` `ad3306f3…`, `model.rs`
  `881dd3c8…`, `facts.rs` `5a630ac7…`, `storyboard-draft.md` `25611bb6…`,
  `storyboard.md` `c5b5e89b…`, `script.tsv` `d5019cd0…`).
- `python3 -I -B <scratch>/checks_0607.py explainer_corpus/parrondo`: ALL
  PASS, 133 checks. It is an independent Python emulation that imports
  `oracle.py` read-only, runs it (exit 0, "50/50 cross-checks passed",
  no FAIL lines on its stderr) and parses the draft's 06/07 words. It
  covers:
  - the walk and wraps, and the truncating-`%` negative control;
  - every published and round chance; the 1/200 change; A fair for every
    coin;
  - both equal-thirds controls; the two seeded runs (33,512 and 38,371
    rounds at corner 0);
  - arrow ends, bow sides and orientation;
  - estimated text boxes pairwise apart at 06_5, 06_6, 07_1, 07_4, 07_5
    and 07_7; the frame;
  - arrows sampled along `curved_arrow`'s curve and head against every
    text box; texts against discs; arrows against the legend coins;
  - the badge, the middle column and the TeX width estimate (3.7 of 4.96);
  - K03's wrap through an emulation of the lower third's own narrowing
    loop (two lines; top −2.78 against the titles' −2.39);
  - both countdowns;
  - the 38 cues present and in order;
  - the 19 proposed SPOKEN rows against the oracle's emitted values;
  - number-word coverage of every 06/07 line; no banned scope word.
- Two deliberately broken copies of that script fail as they should. With
  B's triangle moved to x 1.2, the names land on discs. With the bow gap
  made −0.12, arrows run through 6+ labels. So the checks are not
  vacuous.
- Scans: no `todo!`, `unimplemented!`, `dbg!`, `println!` or absolute home
  path in either file; no line over 120 characters.

Draft picture deltas, for root to copy into the picture cells when 06/07
enter `docs/storyboard.md` and the draft
(`the_produced_chapters_are_the_drafts` requires both). Words, ids and hold
cells are unchanged. Proposed cells:

- **06_1** picture: The player as 02 left them: token, "player", "7 dollars", the seven chips in rows of three with the leftover row outlined, the published badge. The full rows dim; every chip and the big count flow into a small "7 dollars" counter at the top left. Three corner discs: 0 at the top, tinted red, 1 lower right, 2 lower left; then the names "0", "1", "2"
- **06_2** picture: "7 → remainder 1" under the counter; the teal token itself steps onto corner 1 (identity kept); a green forward arrow 1 → 2; the token steps, the counter "8" and "8 → remainder 2"
- **06_3** picture: The token waits on corner 2; countdown where game A's triangle will stand; no arrow out of corner 2 yet
- **06_4** picture: Green arrow 2 → 0, the token steps, the counter "9"; "9 → remainder 0" in red and corner 0 flashes; then a dim back arrow 0 → 2, the token steps back, "8". All six arrows drawn: forward clockwise outside in green, back inside in dim; a beat
- **06_5** picture: "game B" under the triangle. Arrow labels from facts, beyond each arrow's bow: corner 0's in red, 0→1 "9.5 %", 0→2 "90.5 %"; 1→2 and 2→0 "74.5 %" in green, 1→0 and 2→1 "25.5 %" dim
- **06_6** picture: Game A's triangle on the left, the same size, as card A was: forward "49.5 %" in blue, back "50.5 %" dim, "game A" under it; features: "second triangle"
- **07_1** picture: The picture 06 left. "6 → remainder 0" and "600 → remainder 0" appear between the top corners and both slide into B's corner 0. A "precisely" card (K03), held for its reading time
- **07_2** picture: A's corners flash; countdown between the triangles; no shares shown
- **07_3** picture: "1/3" in each of A's corners; under A, "the player's 100,000 rounds" / "33.5 % at corner 0"; features: "readout from facts"
- **07_4** picture: Outlines round B's corner-0 labels; under B, "the player's 100,000 rounds" / "38.4 % at corner 0"; B's corner 0 flashes; both readouts held side by side
- **07_5** picture: The coins at the right, A's, the bad and the good, with "49.5 %", "9.5 %", "74.5 %"; an outline round the chances; a "+0.5" rises and fades over each as they become "50 %", "10 %", "75 %", and the badge becomes the outlined "round-number coins · 50 · 10 · 75 % · no handicap"; features: "badge, replaced from facts"
- **07_6** picture: Both triangles relabel, each label replaced: A's arrows "50 %"; B's 0→1 "10 %", 0→2 "90 %", the others "75 %" and "25 %"; each coin flashes with its words
- **07_7** picture: "with equal thirds at every corner:" over TeX ⅓ · 10 % + ⅔ · 75 % ≈ 53.3 % (callback to 03_2, ≈ as there, 8/15 not a whole tenth), then "B would win", between the top corners under the badge
- "Notes for 07": "the bars grow with them" becomes "the thirds appear with them". The K03 text is unchanged and matches the scene.

Integration list for root's coordinated step (not done here):

1. `src/chapters/mod.rs` (after 04/05's `pub mod looks_hopeless;` and
   `pub mod the_clue;`, which 06/07 import from): `pub mod three_corners;`
   and `pub mod where_you_stand;`, and
   `scene!("06_three_corners", 0.45, three_corners::ThreeCorners)` and
   `scene!("07_where_you_stand", 0.45, where_you_stand::WhereYouStand)`.
   The intensity is root's call.
2. Copy the draft's 06 and 07 sections, with the cells above, into
   `docs/storyboard.md` and the same cells in `storyboard-draft.md`. Then
   run `python3 -I -B tools/storyboard.py parrondo`. The cue, SPOKEN and
   headless tests read `crate::SCRIPT`, so they fail until then.
3. `src/main.rs` SPOKEN rows (values checked against the oracle's output
   by the script above; every key is registered in `facts.rs`):
   ```rust
   ("06_1", "three corners", "rules.modulus", "3"),
   ("06_1", "zero, one and two", "rules.modulus", "3"),
   ("06_2", "seven dollars", "run.start", "7"),
   ("06_4", "Eight dollars plus one is nine", "remainder.examples", "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0"),
   ("06_4", "nine leaves nothing over", "remainder.examples", "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0"),
   ("06_5", "forward nine and a half percent", "coins.published.heads.pct", "49.5,9.5,74.5"),
   ("06_5", "forward seventy-four and a half percent", "coins.published.heads.pct", "49.5,9.5,74.5"),
   ("06_6", "forward forty-nine and a half percent", "coins.published.heads.pct", "49.5,9.5,74.5"),
   ("06_6", "back fifty and a half", "coins.published.tails.pct", "50.5,90.5,25.5"),
   ("07_1", "six dollars or six hundred", "remainder.examples", "6:0,7:1,8:2,9:0,10:1,600:0,-1:2,-3:0"),
   ("07_2", "corner zero", "rules.modulus", "3"),
   ("07_3", "One third", "thirds.pct", "33.3"),
   ("07_3", "thirty-three and a half percent", "run.a.at0.pct", "33.5"),
   ("07_4", "more than thirty-eight percent", "run.b.at0.pct", "38.4"),
   ("07_5", "half a point", "rules.handicap", "1/200"),
   ("07_6", "a ten percent coin", "coins.round.heads.pct", "50,10,75"),
   ("07_6", "seventy-five percent coin", "coins.round.heads.pct", "50,10,75"),
   ("07_7", "equal thirds", "thirds.pct", "33.3"),
   ("07_7", "fifty-three point three percent", "round.b.thirds.win.pct", "53.3"),
   ```
   Then change the doc line's "today 01–03" (01–05 once 04/05 are in) to
   "today 01–07".
4. `src/chapters/glyphs.rs`: no change needed. Every new string is typeset
   by the scenes themselves, and their signs (→ · − + % / :) are already in
   DIGITS.
5. No `Cargo.toml` or workspace change.

Existing-helper proposals (proposals only; draw.rs, 02 and 04/05 are frozen):

- 04 and 05 indicate text through `flash_all` (`indicate` is a target
  transform with a recolour). That is the pattern root has seen duplicate
  typeset labels elsewhere. The places:
  - `looks_hopeless.rs:654` (the merged coin's "loses");
  - `looks_hopeless.rs:717` (the end readout);
  - `the_clue.rs:427` (the badge's words, with its back);
  - `the_clue.rs:431` (B's mark);
  - `the_clue.rs:514` (the mix's words and percentage);
  - `the_clue.rs:516` (the third's mark);
  - `the_clue.rs:543` (the bars' labels);
  - `the_clue.rs:607` (the count line).

  The smallest fix is to flash only the shapes beside them, or to draw a
  hero outline as 07 does. Root decides whether to unfreeze them.
- `swap`, `keep_apart` and an outlined-badge variant of `full_badge` (07
  keeps a local `round_badge` copy) could move to `draw.rs`.
- 06 repeats 02's private grid constants (`COUNT_AT`, `DOLLARS_X`,
  `GRID_X0`, `GRID_Y0`, `PITCH`, `LABEL_GAP`) and leftover-outline builder.
  Making them `pub(super)` in `the_rule.rs` would remove the copy.
- None is needed for correctness.

Limits:

- Nothing is compiled or run in Rust. Expect compile fixes at root's
  qualification. APIs were read at the kit and fmn pin `a399d626`, not
  built against.
- One API judgment is unverified: whether a played animation re-promotes
  its mobject's draw order. The "1/3" texts and the token sit over discs
  that later flash; the token is brought to the front explicitly.
- `fade_out` with a shift (06's fold, 07's slide into corner 0 and the
  "+0.5" rise) is the same self-targeted transform as `fade_out_all`, used
  on text. It is unqualified in frames.
- The cue, SPOKEN and headless tests need the 06/07 script lines, so they
  cannot pass until step 2.
- Text and TeX widths are estimates from the glyph calibration; the real
  guards are `keep_between`/`keep_apart` in the headless run. If the lower
  third narrows K03 below 67 characters a line, it takes three lines and
  the scene refuses (by design) rather than overlapping the titles.
- No frame rendered or looked at, no narration made, no listening. Pacing,
  clarity and English accent are unqualified.
- Pronunciation risks, as the draft notes: "remainder", "thirty-three and a
  half", "fifty-three point three"; "ninety and a half" is screen-only.

## Files

| Path | What | Status |
|---|---|---|
| `docs/brief.md` | sources, model, continuing example, bounded competition check (search results vs watched videos kept apart), arc, vocabulary, risks incl. pronunciation | written; arc section updated to the second draft |
| `docs/storyboard-draft.md` | the full planned film: 18 chapters, 127 lines, words and pictures, predictions, callbacks, pause intentions | written; review repairs applied; chapter 01 identical to production |
| `docs/storyboard.md` | production storyboard: chapters 01–11 (02–11 copied in by root) | 01 written here; 02–11 integrated by root |
| `script.tsv` | generated from `docs/storyboard.md` (01–11, last line 11_7) | generated by root |
| `docs/history/*-20261024-draft{1,2}.*` | earlier drafts, unchanged copies | preserved |
| `oracle.py` | independent exact-fraction oracle and facts ledger: Gaussian elimination, brute-force crowds, exact horizon DP, 729-game and 7,203-case sweeps, negative controls, narration-truth checks on the seeded runs | written, runs |
| `src/model.rs` | native model: exact ratios, ring and periodic chains, crowds, SplitMix64 runs, horizons, crossings; unit tests | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen |
| `src/facts.rs` | every number the film shows or says (all 18 draft chapters), source pins; tests | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen |
| `src/chapters/the_puzzle.rs` | chapter 01 native scene and its tests | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen |
| `src/chapters/draw.rs` | shared look: token, chips, coin, cards, badge, arrows, runs, think countdown, line timing, layout guard | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen |
| `src/chapters/mod.rs` | registry: 01–11 (02–11 added by root) | root's (native21 for 10/11) |
| `src/chapters/the_rule.rs`, `src/chapters/does_b_win.rs` | chapters 02 and 03 native scenes and their tests | registered by root, with root's own edits (`ad182b3d…`, `1e22ff62…`); frozen |
| `src/chapters/looks_hopeless.rs`, `src/chapters/the_clue.rs` | chapters 04 and 05 native scenes and their tests | delivered (Mail 6868); text-emphasis repair (Mail 6879; `c919f887…`, `b413c943…`); registered by root with two test fixes (native09, Mail 6892); now root's |
| `src/chapters/three_corners.rs`, `src/chapters/where_you_stand.rs` | chapters 06 and 07 native scenes and their tests | delivered (Mail 6878); registered by root with its own fixes (native13, Mails 6893/6896); now root's |
| `src/chapters/one_player_to_a_crowd.rs`, `src/chapters/ten_four_twelve.rs` | chapters 08 and 09 native scenes and their tests | delivered (Mail 6902); registered by root with its own repairs (native16, Mail 6922); now root's |
| `docs/integration-08-09.patch` | proposed 08/09 words (08_3, the 08_7/08_8 split), picture cells and notes, against `storyboard-draft.md` `52d093cf…` | root integrated 08/09 (native16); whether every hunk landed as written is not re-checked here (a reverse `git apply --check` fails against later draft edits) |
| `src/chapters/the_trap.rs`, `src/chapters/mix_the_games.rs` | chapters 10 and 11 native scenes and their tests | delivered (Mail 6911); registered by root with its own repairs (native21, Mail 6942); now root's |
| `src/chapters/one_dial.rs`, `src/chapters/the_half_point.rs`, `docs/integration-12-13.patch` | chapters 12 and 13 and their proposed integration | delivered (Mail 6924); frozen by root (`tend-1232`); **unregistered** |
| `src/chapters/net_flow.rs`, `src/chapters/two_products.rs`, `docs/integration-14-15.patch` | chapters 14 and 15 and their proposed integration (after 12/13) | written (ROOT1232-P14), uncompiled, **unregistered** |
| `src/chapters/glyphs.rs`, `src/palette.rs`, `src/score.rs` | glyph probe strings, color roles, musical identity (unlistened) | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen; score unlistened |
| `src/main.rs` | CLI registration; spoken-number, vocabulary/scope, storyboard/registry/script and draft-equality tests | root-qualified (qualify03: 29 release tests, strict Clippy, build, 161 facts); frozen; root extends SPOKEN per chapter |
| `Cargo.toml` | crate manifest (`parrondo_explainer`, bin `parrondo`); excluded from the workspace by root | written |

## Root's qualify02 and the one unfrozen edit (2026-10-24)

Root's `parrondo-qualify02` passed fmt and 29 release tests, then strict
Clippy stopped at `draw.rs:524` (`manual_is_multiple_of`). Root unfroze
`draw.rs` for that expression only:
`(last % step.max(1) != 0)` became `(!last.is_multiple_of(step.max(1)))`,
keeping `step.max(1)`. `rustfmt --check` exits 0; draw.rs is frozen again
(Mail 6844). The only other `% … == 0` in `src` is `model.rs:374`, on
`i64`, which the lint does not cover (`is_multiple_of` is unsigned only),
so it was left unchanged.

Checks for the round-0 repair (cheap only): `rustfmt --check` on
`does_b_win.rs` exits 0. A Python emulation of the repaired test, run on
the oracle's own seeded B run (the oracle imported read-only), passes:

- 401 drawn points, with nothing shown at round 0;
- the first share is 36.4 % at 250 rounds;
- every share past round 0 is at least 36.40 %, against 36.33;
- the end is 38.4 % (38,371 of 100,000);
- 4,001 tracker values checked.

A first run of that emulation failed on my own Python format string (`:.1`
is one significant digit in Python), not on the data; it was fixed and
rerun. The Rust test itself is unrun.

## Checks actually run for the freeze handoff (2026-10-24, cheap only)

- `rustfmt --check --edition 2024 --config-path ../../rustfmt.toml` on
  `model.rs`, `facts.rs`, `main.rs`, `draw.rs`, `the_puzzle.rs`,
  `the_rule.rs` and `does_b_win.rs`: exit 0 each. This is a parse and layout
  check only, not a compile.
- No `gen` identifier is left in `src`. No `todo!`/`unimplemented!` and no
  absolute home path in `src`, the docs or `oracle.py`.
- The token comparison against root's `parrondo-inputs02.json` bytes
  described above.
- The full draft after the 02/03 picture edits, through the storyboard
  tool's `parse`/`check` (scratch script):
  - no problems, 02 at 74.3 s, 03 at 51.4 s, every chapter at most 84.4 s;
  - chapter 01 and the colour roles are identical in both storyboards;
  - the generated script is byte-identical to the one before the edits.
- `tools/lint_script.py` on that script: "127 lines, 0 errors, 1 warnings
  -> PASS" (09_5, as before).
- `oracle.py` and the production storyboard/script did not change since the
  runs below, so those runs were not repeated.

## Checks actually run after the review repairs (2026-10-24, cheap only)

- `python3 -I -B explainer_corpus/parrondo/oracle.py`: exit 0, "50/50
  cross-checks passed", no FAIL; new facts `crowd.good_wins_at2` 3/4,
  `realcrowd.round.1` to `.8` (9,6,11 … 10,5,11), `realcrowd.exact` 40,
  `realcrowd.mean` 10.0,4.0,12.0, `mixshift.round.1` to `.6`; new checks
  10_3, 08_7, the split-crowd control, 11_5 (5 crossings in 6 rounds) and
  15_1 (the converse on 729 games).
- `python3 -I -B tools/storyboard.py --check parrondo`: exit 0, 01 at
  84.4 s, 10 beats (words unchanged, so `script.tsv` is unchanged).
- `python3 -I -B tools/lint_script.py explainer_corpus/parrondo/script.tsv`:
  "10 lines, 0 errors, 0 warnings -> PASS".
- The full draft through the tool's own `parse`/`check`/`script_text`
  (imported read-only from a scratch script; the CLI reads only
  `docs/storyboard.md`): no problems, every chapter at most 84.4 s, "total
  22.0 min over 18 chapters" (an estimate from word counts); lint of its
  script "127 lines, 0 errors, 1 warnings" (09_5, kept on purpose). The
  same script found chapter 01 and the colour roles identical in both
  storyboards.
- A Python emulation of the new 01_10 layout assertions on the seeded run
  (not the Rust test): the drawn run's lowest point over the inset's band
  is y −0.099 against a ceiling of −0.25; label room −4.22 > −4.4; the
  inset's run spans 0.90 of its height.
- Scans: no `todo!`/`unimplemented!`, no absolute home path; no banned
  scope word in the draft's voice.

Earlier, before the review (first handoff):

- `python3 -I -B explainer_corpus/parrondo/oracle.py`: exit 0,
  "45/45 cross-checks passed" (includes the new `run.a.expected.net` = −1000).
- `python3 -I tools/storyboard.py --check parrondo`: exit 0, "total 1.4 min
  over 1 chapters" (01 at 84.4 s, 10 beats).
- `python3 -I tools/lint_script.py explainer_corpus/parrondo/script.tsv`:
  "10 lines, 0 errors, 0 warnings -> PASS".
- The full draft, checked by the same tool on a scratch copy (the tool reads
  only `docs/storyboard.md`): `--check` exit 0, "total 21.2 min over 18
  chapters" (an estimate from word counts, not a measurement); lint "125
  lines, 0 errors, 1 warnings" (09_5, three sentences opening with
  "Corner", kept on purpose and noted in the draft).
- Python simulations of the new Rust text tests (not the tests themselves):
  chapter 01's section is identical in both storyboards; every phrase the
  scene waits for and every SPOKEN phrase is in its line; no line has an
  unguarded number word or a banned scope word.
- Scans: no `todo!`/`unimplemented!`, no animatic left in `src` or the
  production storyboard, no absolute home path in the topic.

## Tests written for root's native check (not run here)

`src/chapters/the_puzzle.rs`: the chapter's numbers are its words; every
waited-for phrase is in its line; the layout keeps its rooms (chips, cards,
long-run window, lower third, thumbnail); **the chapter runs headless to its
end** with the scratch voice and real typesetting, says 01_1 to 01_10 once in
order, and leaves at least the think time between the end of 01_7 and 01_8.
The scene also refuses to draw when a typeset label leaves its band
(`keep_between`), so that headless run checks the real text widths.
`src/main.rs`: spoken numbers equal their facts; vocabulary and scope;
storyboard, registry and script agree; produced chapters equal the draft's.
`src/model.rs`, `src/facts.rs`, `src/palette.rs`, `src/score.rs`: as listed
in their files.

Added with the repairs (not run here): `model.rs`
`a_real_crowd_jumps_while_the_split_one_stays`; `facts.rs`
`the_crowds_do_what_the_words_say` and new headline rows; `the_puzzle.rs`
inset, badge and readout assertions in its existing tests. The headless
run also meets the new `keep_between` guards (badge both sides, inset
title, inset labels) and the longer 01_9 readout with real text widths;
those widths are unmeasured until that run.

## Limitations (standing)

- Nothing here has been compiled: Cargo builds are forbidden in this phase.
  Every Rust file is uncompiled source until root's serialized native check;
  expect compile fixes there. APIs were checked by reading the kit and the
  fmn pin `a399d626` source, not by building.
- No narration, no render, no frame has been looked at, no listening. A
  written storyboard and passing arithmetic do not show rendered clarity,
  natural pacing or English-accent acceptance.
- Chapters 08 to 18 have words and pictures only, in the full draft; root
  assigns their native scenes. Their facts are already computed and tested.
  Chapters 04 to 07 have native source that is unregistered and
  uncompiled. Chapters 02 and 03 are integrated by root.
- Pronunciation risks are recorded in `docs/brief.md` ("Risks") and the
  chapter notes, not resolved.
