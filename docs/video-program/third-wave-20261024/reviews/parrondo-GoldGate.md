# parrondo: independent review by GoldGate

Bead `mx-video-program-program-87do.15.2` (review of WildBass's writing task
`.15.1`). Reviewer GoldGate, 2026-10-24. This review reads files and runs
arithmetic only: no Cargo, TTS, render or listening.

## Snapshot reviewed

`narration_lab/third-wave-20261024/root/intake-0550/parrondo/`, as captured
in `narration_lab/third-wave-20261024/root/intake-0550/inputs.json`
(`captured_at` 2026-10-24T05:50:51Z, `full_draft_beats` 125,
`source_stable_at_capture` true).

I re-verified it before and after the review. All 16 SHA-256 hashes match,
with no unlisted and no missing files. The live crate
`explainer_corpus/parrondo/` was not reviewed, and this review applies only
to these bytes:

| file | sha256 (prefix) | file | sha256 (prefix) |
|---|---|---|---|
| Cargo.toml | be4cc4f7a7a21e5f | src/palette.rs | 366457ba05a18ad7 |
| script.tsv | 5225e24738009606 | src/facts.rs | bb68bbac17b9cdd0 |
| oracle.py | b590609a2e1218d9 | src/score.rs | 34bac2329a425ac7 |
| docs/storyboard.md | 4bc7bf7965faf216 | src/main.rs | af175bc5f06b7451 |
| docs/WORK_STATUS.md | e47c781588b3530c | src/model.rs | 8c3f70dc26e81349 |
| docs/brief.md | 2213d7c42905891a | src/chapters/the_puzzle.rs | 237e6bfb55a86fa3 |
| docs/storyboard-draft.md | 690b8622171ee72a | src/chapters/draw.rs | f93a3ecf0aa8cb9e |
| src/chapters/mod.rs | 4e967d036b63bbb6 | src/chapters/glyphs.rs | 378ee8733b5c0287 |

I read every file in full, including the whole 18-chapter
`docs/storyboard-draft.md`.

## Verdict

**The mathematics is sound.** My own exact checks use different methods
from the author's (listed below). Of the oracle's keys I computed, 76 agree
and 0 differ, and every Ethier & Lee pin is reproduced. Both primary
sources support the claims the film relies on.

No number in `facts.rs` or `oracle.py` is wrong. What follows are:

- one source defect that blocks the corpus's clippy gate;
- four wording and claim problems in the full draft;
- three teaching items.

None needs a new number unless the optional real-crowd beat in M1 is
adopted.

## Findings: source (native opening)

**S1 — would fail `cargo clippy --all-targets -- -D warnings` (AGENTS.md
line 311). Blocking but trivial.**

- **Where:** `src/chapters/the_puzzle.rs:19` imports `FRAME_Y_RADIUS`. Its
  only uses are line 139 (the `#[cfg(test)] const LOWER_THIRD_TOP`) and line
  547 (inside `mod tests`).
- **Consequence:** the bin target's non-test build has an unused import, so
  `unused_imports` fires, which `-D warnings` turns into an error.
  `source_scan.out.txt` lists it. I confirmed it by eye. The scan's other
  hits are false positives: the `Timed` trait (used through `.rt()`) and the
  `Ratio` operator-trait methods.
- **Smallest correction:** import only `FRAME_X_RADIUS` at line 19 and add
  `#[cfg(test)] use fmn::core::constants::FRAME_Y_RADIUS;`.

**S2 — 01_10's "short stretch" outline is too small to show any luck
(teaching consequence; computed, no frame seen).**

- **Where:** `the_puzzle.rs:454-467` outlines rounds 0 to 1,000 on the
  100,000-round window `LONG` (`:119-125`; 1,400 dollars over 4 scene
  units).
- **Computed size:** in this run those rounds span −58 to +13 dollars. That
  is 0.106 × 0.203 scene units, about 29 × 55 px at 4K, inside a 0.50 ×
  0.50 outline.
- **The climb the voice recalls:** 01_5's climb (five dollars) is 3.9 px tall
  on this plot.
- **Consequence:** "Luck decides any short stretch" points at a sliver where
  no luck is legible, so the callback to 01_5 is lost.
- **Smallest correction:** at "Luck decides", show the outlined rounds in a
  small inset at their own scale (dollars −58 to +13; take the range from
  the run, as `numbers()` already does for the first rounds). Alternatively,
  bring back 01_5's step plot as a thumbnail beside the outline.

## Findings: mathematical and factual (full draft)

**M1 — chapter 09 presents the expected crowd as real tosses. Medium.**

- **Where (`docs/storyboard-draft.md`):** 09_1 (line 355) says "ten players
  toss the bad coin. One wins and moves forward. Nine lose". 09_5 (line 359)
  says "Every player moved, and the crowd looks exactly the same". 09_6
  (line 360) says "thirteen, exactly half". 08_6 (line 336) says the crowd
  "settles at ten … four … twelve".
- **Where it is stated:** the idealization is stated at 08_2 (line 332,
  "split exactly by the coins' chances") and in the K04 lower-third (line
  340: "Real players scatter around these numbers"). K04 then leaves the
  screen. Chapter 09 turns the bars back into 26 dots, and the voice says
  "toss" with exact outcomes.
- **My exact numbers for 26 real random players at (10, 4, 12), one round
  of round-coin B:**

  | Event | Probability |
  |---|---|
  | The counts come back exactly 10, 4, 12 | 0.0707 |
  | All six flows are exactly 1/9, 3/1, 9/3 | 0.0422 |
  | Exactly 13 winners | 0.2016 |
  | Exactly one of the ten at corner 0 wins | 0.3874 |

  - In a settled crowd of independent players, P(exactly 10, 4, 12) = 0.0358.
  - The corner-0 count is 10 ± 2.48 (binomial 26, 5/13).
  - One seeded real crowd shows counts from (8, 8, 10) to (15, 0, 11)
    between rounds 24 and 30.
- **Consequence:** the sentences are true of expected numbers, not of tosses.
  For real tosses, 09_5's "looks exactly the same" fails about 93 % of the
  time, and 09_1's "One wins" fails 61 % of the time. Ruth will object. Maya may conclude that a
  real crowd literally stops moving.
- **Smallest correction:**
  1. Keep the frame in the words. For example, 09_1: "Check that this crowd
     really stays put, still split exactly by the chances. At corner zero,
     ten players toss the bad coin, so one wins and moves forward, and nine
     drop back to corner two."
  2. Keep a dim screen label such as "split by the chances (expected
     numbers)" beside "players: 26". It should stay from 08_2 through 09_8,
     and also on 11's bars and 14's crowd of 1,418. The phrase is screen-only,
     since the voice may not say "expected".
- **Optional, and welcome under the owner's direction:** after 09_6, add one
  beat in which a small seeded crowd of 26 real players jiggles around 10, 4,
  12. That would tie the expected crowd back to 01's lesson that luck decides
  short stretches. It needs a fact and SPOKEN rows.

**M2 — 11_5's "round by round … falls … climbs" contradicts the computed
path. Medium-low.**

- **Where:** line 409, "Round by round, corner zero's share falls from
  thirty-eight point five percent to thirty-four point six, and corner one's
  climbs from fifteen point four to twenty-five point four". The picture
  shows live shares.
- **What actually happens:** starting the mix from (10, 4, 12), a single
  round already gives corner 0 9/26 = 34.6 %. Corner 1 overshoots to 28.8 %.

  | Round | Corner 0 (%) | Corner 1 (%) |
  |---|---|---|
  | 0 | 38.5 | 15.4 |
  | 1 | 34.6 | 28.8 |
  | 2 | 33.7 | 24.1 |
  | 3 | 35.4 | 25.9 |
  | 4 | 33.9 | 25.1 |

  After that the shares settle to 34.6 and 25.4. The mix's transient
  eigenvalue is −0.685, so the crowd sloshes, as it does for B in chapter 08.
- **Consequence:** live shares computed from the model would contradict the
  words. Endpoint tweens would match them, but would not be a computed
  change.
- **Smallest correction:** animate the true rounds, as chapter 08 does. Say,
  for example: "Off corner zero. Within a round its share drops to about
  thirty-four point six percent, and after a little sloshing the crowd
  settles there, with corner one up from fifteen point four to twenty-five
  point four." The number words are unchanged.

**M3 — 10_3's wording mis-states where the good coin's wins go. Low.**

- **Where:** line 383, "So most of the good coin's wins at corner two just
  bring the player back to zero".
- **The facts:** every win at corner 2 lands on corner 0, because 2 → 0 is
  the forward move. In B's settled crowd, 75 % of the good coin's wins
  happen at corner 2: 9 of 12 with round coins, and also 75.0 % with the
  published coins.
- **Smallest correction:** "So most of the good coin's wins happen at corner
  two, and every one of them just brings the player back to zero, where the
  bad coin waits." This adds no number words.

**M4 — 13_8's history claim goes beyond the source. Low.**

- **Where:** line 457, "Harmer and Abbott named the effect after its
  discoverer, Juan Parrondo."
- **What the source says:** the Nature page says only that this is "a
  striking new result in game theory called Parrondo's paradox, after its
  discoverer, Juan Parrondo". It does not say who coined the name.
- **Why it matters:** the draft's own note (lines 464-466) restricts history
  to that sentence.
- **Smallest correction:** "Harmer and Abbott's paper calls it Parrondo's
  paradox, after its discoverer, Juan Parrondo."

## Findings: teaching (full draft)

**T1 — 15_4 uses a converse the derivation has not shown. Low-medium.**

- **Where:** 15_1-15_3 (lines 494-496) prove "zero net flow ⇒ the two
  products are equal". 15_4 (line 497) then concludes "Balanced, so fair",
  which is the converse. The K06 formula that justifies it arrives only at
  15_6.
- **Consequence:** a careful viewer sees a converse error at the film's
  rigorous moment.
- **Smallest correction:** add one sentence to 15_3: "And it works the other
  way: when the two products match, all three balances can hold at once, so
  the net flow is zero." The algebra behind it: weight₁ = weight₀·p₀/q₁ and
  weight₂ = weight₁·p₁/q₂, and the third balance holds exactly when p₀p₁p₂ =
  q₀q₁q₂. Alternatively, reword 15_4 in the proven direction: "Balanced, as a
  fair game's must be."

**T2 — 08_1 breaks the continuity of a persistent object. Low.**

- **Where:** the token last stands on corner 2 with 8 dollars (06_4, line
  287). 08_1 (line 331) splits "the token on B's triangle" into dots
  "gathered at corner 0".
- **Smallest correction:** in 08_1's picture, the token takes one more win
  (8 → 9) onto corner 0, then splits. That also gives "all starting at
  corner zero" its reason.

**T3 — considered; no change required.** The badge at 01_2 already shows
"9.5 · 74.5 %", about a minute and a half before 02_4 and 02_5 explain those
numbers. No prediction depends on them. If root wants a cleaner first
minute, the badge could read "published coins" alone until 02_5.

## Teaching evaluation against the owner's direction

- **Persistent objects: strong.** These recur throughout:
  - the teal token, chips and count;
  - the triangle's fixed orientation;
  - the coin badge, which flips at 07_5, 13_1, 14_1 and 15_7, each time in
    the matching handicap regime (I checked every voiced number against the
    badge);
  - the two cards;
  - the run thumbnails.
- **Controlled computed changes: good.**
  - Chapter 08's crowd evolution comes from the model.
  - 09_8's tween runs along the exact dial line.
  - Chapter 17's histograms come from an exact distribution.
  - 11_5 needs M2.
- **Prediction holds: good.** There is one ⟨predict⟩ per chapter (18 in
  all). The answers are withheld on screen where I checked: 03_3, 05_1, 06_3,
  07_2, 09_4, 12_4 and 17_4.
- **Earned explanation: good, with two exceptions.**
  - The remainder is taught on the player's own chips (02_2-02_8).
  - The triangle comes from the token's own steps (06_2-06_4).
  - The clue is measured on the player's own runs (03_5, 05_2, 07_3-07_4)
    before any weight is named. "Weight" is named only at 10_8.
  - The products are derived from the balances.
  - The exceptions are the crowd's idealization (M1) and the converse (T1).
- **Callbacks: strong.**
  - the thumbnails of 01 → 03, 04, 07;
  - 53.3 % in 07_7 → 10_6 → 12_3;
  - the ordinary coins of 04_2 → 16_2;
  - the bars of 05_3 → 13_7;
  - the cards of 01 → 18_1;
  - the trace of chapter 10 → 18_2.
- **Does the player's changing capital earn the later abstraction? Yes,
  through chapter 07.** The weak joint is the step from player to crowd
  (08_1-08_2). There the film moves from real tosses to expected numbers
  without a lasting marker, which is what M1 and T2 address. A real-crowd
  contrast would make that step earned rather than asserted.
- **Length:** no compression is requested. The draft is about 21 minutes by
  the author's estimate (unmeasured), and every chapter is ≤ 84 s.

## Independently checked, no finding

My methods are deliberately different from the author's:

- **Stationary weights:** the constant-net-flow equations around the
  triangle, solved by Cramer's rule and checked exactly (πP = π) and by float
  power iteration.
- **K06:** brute-force enumeration of spanning in-trees.
- **Periodic schedules:** the 3L-state (corner, phase) chain.
- **Real crowds:** binomial convolution.
- **Horizons:** an exact Fraction DP at 100 rounds and a float DP at 1,000.
- **Seeded runs:** my own SplitMix64.

| claim (cue) | result |
|---|---|
| Two handicap regimes (round coins 07_5-12_8 and 14_1-15_6; published elsewhere) | ε = 0: A, B fair; B weights 5/13, 2/13, 6/13; mix 3/10, 5/8 → 245/709, 180/709, 284/709, mean 18/709. ε = 1/200: A −1/100, B −73443/8446300 (−0.87 ¢), mix +223/14200 (+1.57 ¢); B 38.4/15.4/46.2 %, mix 34.5/25.4/40.1 % |
| Dial and decomposition (12_2-12_8, 13_3-13_6) | B wins good − (good − bad)·share₀; fair share 5/13 and 49/130 (37.7 %); B rounds in the mix 52.54 % and 52.07 %; mix 51.27 % and 50.79 %; 76.2 % of the way to thirds; the handicap costs the mix 0.97 ¢; halfway −0.93 ¢; die-chosen 8/15 and 317/600 |
| K06, equal edge flow, sign rule (14_5, 15_6, 18_8) | exact on A, B and the mix in both regimes, and on 400 random rational games |
| Crowd of 26 (08-09) | smallest whole crowd 26; flows 1/9, 3/1, 9/3; 13 winners; total 26 every round; rounds 1-6 as the draft gives them; within half a player from round 24; transient eigenvalue −0.871 (the sloshing) |
| Crowd of 1,418 (14_2-14_4) | 490/360/568; 147/225/355 forward and 343/135/213 back; net 12 per edge; 36 = 727 − 691 |
| Schedules (16_3-16_4) | per round played, ε = 0: AB 0, AABB 4/163, ABB 2416/35601, AAB 32/1609 (exact); AB = 0 at ε = 0 for all 10 ρ tried; AB slope −1.34911 = −228/169; published AB −0.674 ¢, AABB +1.465 ¢ (a float Cesàro average over 200,000 rounds agrees) |
| Share of A (16_5) | crossings 7.031 % and 84.708 %; −0.229 ¢ at 5 %, −0.338 ¢ at 90 % |
| Horizons (17_3-17_6) | mix after 100 rounds from 0: 51.709 / 39.581 / 8.710 %; after 1,000: ahead 68.861 %; A behind 61.206 %; B behind 63.898 %. From 7 dollars the mix is 54.0 % ahead after 100 rounds; 17_3 and K08 say "from zero", so this is consistent |
| Seeded player (01, 03-05, 07, 17_1) | A −1,062, B −548, mix +1,748; at-zero rounds 33,512 / 38,371 / 34,520; A's first nine values 7, 8, 9, 8, 9, 10, 11, 12, 11; B +16 after 1,000 rounds; A's low −1,138 and high 13, inside 01_9's window |
| Closing puzzle (18_4-18_7) | −2490243/9016300 (−27.62 ¢) and +2902797/9230300 (+31.45 ¢); every coin of the reverse rule is below one half |
| Oracle agreement | 76 keys agree, 0 differ (4 of my keys have no oracle key) |

### Primary sources

**Harmer & Abbott, *Nature* 402, 864 (1999).** I read the one page as a PDF
image. It supports:

- the rules: p₁ = 1/2 − ε, p₂ = 1/10 − ε, p₃ = 3/4 − ε, M = 3, ε = 0.005;
- AABB as "the pattern in the original paper" (16_4): "two games of A
  followed by two of B";
- that random play wins;
- the single page and the flashing-ratchet analogy (13_8, 18_9);
- the figure's averaging over 50,000 trials of 100 games.

Both caption errors the brief reports are really there: "coin 3 has a poor
probability of winning" and "playing games A and B in any sequence leads to a
win". My AB result (−0.674 ¢) confirms that the second one is false. See M4
for the naming sentence.

**Ethier & Lee, *EJP* 14 (2009), 1827-1862 (arXiv:0902.2368).** I read the
arXiv abstract, and the ar5iv text through a fetch summarizer, not the PDF.
It supports:

- the SLLN and CLT (Theorem 1);
- 18/709;
- μ[2,2] = 4/163, μ[1,2] = 2416/35601, μ[2,1] = 32/1609;
- μ[1,1](ε) = −(228/169)ε + O(ε²);
- **Theorem 7: μ[1,1](0) = 0 for all ρ > 0**, which matches the draft's note
  at lines 526-528;
- that μ[r,s] is "per game played";
- π_B = (1 + ρ², ρ(1 + ρ), 1 + ρ) / (2(1 + ρ + ρ²)).

Not checked: the paper number "62". *Statistical Science* 14 (1999) was not
read (nor by the author).

### Native opening, checked by reading

- **Kit API:** every kit call matches the working tree's signatures.
  - `Kit::say`, `hold`, `mark`, `sentence_starts`, `text`, `text_left`,
    `tex`, `live_number` (its 8 arguments and `edge`) and `finish`;
  - `tween`, `fade_all`, `rect`, `Frame2`, `precisely`, `thousands` and
    `signed_thousands`;
  - the `Explainer` and `Chapter` fields.

  `Narrator::say` waits for the gap before recording a line's start, so the
  `Spoken.start` taken after `kit.say` is the line's true start.
- **Helpers:** the draw helpers mirror `explainer_corpus/freivalds/src/chapters/draw.rs`
  (the `indicate` scale, the empty-anchor updaters in `think`, the line
  timing).
- **Phrases and sentences:** all 11 waited-for phrases are in their lines.
  The kit's sentence splitter and `draw.rs`'s splitter agree on every
  chapter-01 line.
- **Layout:** every assertion in `the_layout_keeps_its_rooms` holds by
  arithmetic. K01 (181 characters) wraps to two lines at the kit's
  92-character width, within the three-line bound the layout assumes. Its
  12.1 s reading time is kept by `wait_until`.
- **Badge: an estimate only, so no finding.** The kit's lower-third starts
  at 92 characters per line at 26 pt, in a band about 11 units wide. That
  gives about 0.12 units per character, so the 37-character 18-pt badge would
  be about 3.4 units wide and end near x = 6.3, inside the frame's 7.11.
  The scene guards only the badge's left edge (`the_puzzle.rs:288`). A
  right-edge `keep_between` would be cheap hardening for the longer
  round-coins badge in later chapters.
- **01_9 timing (scratch-voice estimate only):** "ending about" lands at
  about 6.7 s, when about 54 % of the run is drawn and the token reads about
  −881. The readout fades in after about 9.8 s, near the line's end. That is
  acceptable; recheck it in the narrated render. It is not a finding.

## Commands actually run (from the repository root)

- `python3 -I -B narration_lab/third-wave-20261024/root/intake-0550/parrondo/oracle.py`
  - exit 0, stderr "# 45/45 cross-checks passed";
  - stdout saved outside the repo for the comparison;
  - its sha256 is 2d53d1bfc99aded1….
- `python3 -I -B docs/video-program/third-wave-20261024/reviews/parrondo-GoldGate/check.py <oracle stdout>`
  - exit 0: 53 ok, 0 FAIL, "76 keys agree, 0 differ";
  - output in `parrondo-GoldGate/check.out.txt`.
- `python3 -I -B docs/video-program/third-wave-20261024/reviews/parrondo-GoldGate/source_scan.py narration_lab/third-wave-20261024/root/intake-0550/parrondo`
  - exit 0;
  - output in `parrondo-GoldGate/source_scan.out.txt`.
- A hash re-verification of the snapshot against `inputs.json` (inline
  Python): 16/16 match.
- Reads of the kit (`crates/explainer_kit/src/{kit,narration,play,plot,props,cli}.rs`,
  working tree) and of freivalds' `draw.rs` for API comparison.
- Python 3.15.0; repository HEAD 6033720 (the kit has uncommitted
  working-tree changes, `narration.rs` among them).
- Web:
  - the Nature PDF hosted at `sites.oxy.edu`, read as an image;
  - the arXiv abstract page and the ar5iv HTML of arXiv:0902.2368.

  Nothing downloaded is stored in the repository.

Evidence files (sha256): `check.py` 5a2228cc…, `check.out.txt` 9d3551bb…,
`source_scan.py` 5c772785…, `source_scan.out.txt` e951e826….

## Not run and not claimed

- **Not run:** Cargo build, clippy, the Rust tests (including the headless
  `the_chapter_runs_and_leaves_room_to_think`), `parrondo facts`,
  facts-check, the glyph probe, `tools/storyboard.py --check` and
  `tools/lint_script.py` on this snapshot. The author reports the last two.
  S1 is a reading of the source, not compiler output.
- **No frames:** no render was made or seen, and no frame finding is
  claimed. S2 is arithmetic from the layout constants.
- **Not done:** no TTS, narration, listening, pacing or English-accent
  check. Written plans and passing arithmetic do not show rendered clarity,
  natural pacing or accent acceptance.
- **Beads:** no Beads, shared-file or other-writer edits, no commit and no
  push.
