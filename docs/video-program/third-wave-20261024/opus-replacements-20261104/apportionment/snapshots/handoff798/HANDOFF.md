# Adding a Seat — rebuild candidate, handoff (bead .30)

**From:** WhiteSwan (Claude Code, Opus 5.5) · **To:** root / BlueSalmon · **Date:** 2026-11-04 UTC
**Status:** candidate source only. Not compiled, rendered, synthesized, listened to, synced or independently reviewed.

## Narration: ElevenLabs Oliver

Every cue in this candidate is written for **ElevenLabs Oliver**. The Michael and FrankenTTS defaults in the required skills do not apply. The owner reminders in Mail 595, 698, 726 and 753 are acknowledged. I generated no audio and launched no TTS or other heavy job. I did not touch the canonical crate, its narration or any existing take. Synthesis belongs to root's guarded serial lane.

## What the film is now

The candidate has 15 chapters, 85 cues and 3,429 spoken words. `tools/storyboard.py` estimates 26.6 minutes. One running example carries the whole film: Ash, Birch and Cedar, with 4,000, 9,000 and 26,000 people. Two steps have to leave it. The quota failure under Webster needs four towns, and the impossibility proof needs five.

| # | chapter | what the viewer learns | prediction pause |
|---|---|---|---|
| 01 | one_more_seat | 15 seats give Ash, Birch and Cedar 2, 3 and 10; adding a 16th seat drops Ash to 1 | 01_3 |
| 02 | shares | the exact share; why rounding each share alone fails at 14 and 16 seats | 02_4 |
| 03 | largest_tails | Hamilton's rule: whole parts, then rank the tails; the tails always add up to the leftover seats | 03_3 |
| 04 | sixteenth | at 16 seats the tails re-rank (25, 26, 27 thirty-ninths) and Ash's lead vanishes | 04_3 |
| 05 | dials | the mechanism: each tail is a dial hand that turns p/P of a turn per seat; the small town turns slowest | 05_3 |
| 06 | not_every_time | at 17 seats Ash's seat comes back; 5 of 99 one-seat additions between 1 and 100 seats cost Ash a seat; Seaton's 1880 letter, verbatim | 06_1 |
| 07 | yardstick | Jefferson: change the people-per-seat figure, not the rounding; the window of yardsticks runs from 2,250 to 2,363.6 | 07_2 |
| 08 | signposts | one ranked list of signposts serves every house size, so no seat is ever taken back; the exact tie at 2,000 | 08_3 |
| 09 | cost | Jefferson gives Cedar 11 seats on a share of exactly 10; a dropped tail is 42.5% of Ash's count but 2.7% of Cedar's; New York after the 1830 census | 09_1 |
| 10 | rounding_rules | moving the rounding point gives Adams, Webster and Hill; Webster gives the same seats as Hamilton at 15 seats and never takes one back as the house grows; Hill at 3 seats breaks quota | 10_3 |
| 11 | webster_quota | with 3 towns Webster stays within quota (cited); a 4-town map at 14 seats where the town whose share is exactly 9 gets 8 | 11_1 |
| 12 | both_promises | Balinski and Young's quota rule in simple form: skip a signpost that would overshoot a town's ceiling; 1, 4, 10 at 15 seats, then 1, 4, 11 at 16 | 12_1 |
| 13 | population | a later census at 16 seats: Hamilton takes a seat from Cedar, which grew, and gives it to Ash, which shrank; divisor rules cannot do this (real signpost lists move) | 13_3 |
| 14 | no_rule | the Gölz, Peters and Procaccia five-town, ten-seat proof, both branches; 3 towns possible, 4 open, a lottery as a way out | 14_2 |
| 15 | recap | the full scorecard; try it yourself: under Webster, can a growing Cedar raise Ash? (a real list for a Cedar of 30,000); back to the dials | 15_3 |

## Files

All paths below are under `docs/video-program/third-wave-20261024/opus-replacements-20261104/apportionment/`. Each `candidate/explainer_corpus/apportionment/X` maps to `explainer_corpus/apportionment/X`.

| candidate file | lines | integration |
|---|---|---|
| `docs/storyboard.md` | 199 | new; this is the source of `script.tsv` |
| `script.tsv` | 86 | replaces the canonical file; sha256 `df0c6cec…f557` |
| `narration_units.json` | 611 | replaces the canonical file, which is bound to the old 23-cue script; one unit per cue, `oliver_<cue>`; sha256 `68ffe264…88f5` |
| `docs/narration-direction-map-oliver03.json` | 281 | new version; oliver01 and oliver02 stay as they are; 15 prediction overrides and holds; sha256 `71ec5904…dc37` |
| `oracle.py` | 492 | new, an independent exact `Fraction` oracle; prints 387 keys; `--check` writes cross-checks to stderr; the old `facts_oracle01.py` and `facts-oracle01.json` are left alone |
| `src/main.rs` | 51 | replaces the canonical file; same `Explainer` contract, plus `STORYBOARD` under `#[cfg(test)]` |
| `src/model.rs` | 420 | new; std-only exact integer arithmetic, with unit tests |
| `src/facts.rs` | 373 | replaces the canonical file; computes every key from `model`; history and Table 1 values are pinned with sources |
| `src/score.rs`, `Cargo.toml` | 38, 13 | byte-identical copies of canonical (`cmp` checked); `Cargo.lock` is unchanged and not copied |
| `src/chapters/mod.rs` | 179 | replaces the canonical file; registry of 15 chapters, glyph supplement, storyboard, order and voice tests |
| `src/chapters/draw.rs` | 939 | new; rows and bands, tokens, quota boxes, dials, signpost cards (now keyed and movable), scorecard, playback, sentence sync, think ring |
| `src/chapters/{one_more_seat,shares,largest_tails,sixteenth,dials,not_every_time,yardstick,signposts,cost,rounding_rules,webster_quota,both_promises,population,no_rule,recap}.rs` | 132–464 | new; one per chapter |
| `src/chapters/thumbnail.rs` | 78 | replaces the canonical file; values come from `model` |
| `checks/*.py` | — | staging tools only, not runtime; they rerun the checks below |

After integration these canonical chapter files would no longer be referenced: `alabama.rs`, `divisor.rs`, `priority.rs`, `quota.rs`, `remainders.rs`, `seat_disappears.rs`, `seventeen.rs`, `three_rows.rs` and `visual.rs`. Whether to keep them is root's call. I deleted nothing.

## Oliver cue map (for serial synthesis)

- **All 85 cues are new text.** No candidate cue matches any of the 23 canonical lines exactly.
- **Do not bind existing takes by id.** Cues `01_1`–`08_2` reuse 23 canonical ids, but every one of them now carries different words. Every existing Oliver take should be preserved and left unbound.
- **One take per cue**, in script order. `narration_units.json` lists them as `oliver_01_1` … `oliver_15_5`, and each unit is between 1 and 100 words.
- **15 prediction holds**, one after each of 01_3, 02_4, 03_3, 04_3, 05_3, 06_1, 07_2, 08_3, 09_1, 10_3, 11_1, 12_1, 13_3, 14_2 and 15_3. Each sits at the end of its cue, followed by a 4-second authored visual wait from `draw::think`. Every hold is bound to the chapter line that calls `think` and to that file's sha256. After any edit to a chapter, rerun `checks/bind_oliver_plan.py`.
- **Two cues were reworded this session, before any synthesis:**
  - 10_6 now opens "Careful, though. Hill's rule can break the quota promise even with three towns."
  - 15_4 now ends "Ash can keep its seats or lose some, but it can never gain one." The old ending, "loses one", was false: a large enough Cedar takes more than one seat.
- **Words Oliver has to get right:**
  - Gölz
  - Huntington Hill
  - Seaton
  - John Quincy Adams
  - "thirty ninths"
  - "point six seven"
  - "eighteen thirty census"
  - "nineteen forty one"

  The spoken text never uses digits, colons, semicolons, dashes or parentheses.

## Exact checks run (all from the repository root; all passed)

Here `S` is the staging directory and `C` is `$S/candidate/explainer_corpus/apportionment`.

| check | command | result |
|---|---|---|
| oracle | `(cd $C && python3 -I -B oracle.py --check)` | exit 0; 387 keys. The cross-checks pass: tails add up to the leftover count for 1 to 100 seats; divisor rules never take a seat back for 1 to 100 seats, skipping exact ties; the GPP bounds hold; the four-town map holds; the puzzle shows Webster at 15 seats going from 2, 3, 10 to 1, 3, 11 |
| Rust logic mirror | `python3 -I -B $S/checks/rust_mirror.py $C/oracle.py` | a line-by-line Python transliteration of `model.rs` and `facts.rs` gives 387 keys; differ `[]`, missing `[]` |
| storyboard | `python3 -I -B $S/checks/regen_script.py . $C` | 0 problems; `script.tsv` is current (85 cues); 26.6 min |
| cue phrases | `python3 -I -B $S/checks/phrase_scan.py $C` | all 85 cues said once, in order; 195 literal `wait_for` phrases found in their own cue, in spoken order; 15 more phrases passed through loop variables were checked by hand |
| voice rules | Python stand-in for `spoken_words_follow_the_voice_rules` | no digit, colon, semicolon, dash or parenthesis in any cue |
| Rust syntax and format | `rustfmt --edition 2024 --config-path rustfmt.toml --check $C/src/main.rs` | clean across `main.rs` and every module it declares |
| Oliver plan | `python3 -I -B $S/checks/bind_oliver_plan.py $C --check` | files current; 15 holds |
| plan against repo tooling | `python3 -I -B $S/checks/validate_oliver_plan.py . $C <empty scratch dir>` | `tools/coherent_narration.py` `direction_contract` resolves 15 directions; the `check_coherent_units` rules accept 85 units over 85 cues; the 15 hold markers sit at their lines with current hashes |
| canonical untouched | mtime and `cmp` | `explainer_corpus/apportionment/script.tsv` still dates from 2026-11-02; `score.rs` is identical |

## Sources

- **Gölz, Peters and Procaccia**, *In This Apportionment Lottery, the House Always Wins*, arXiv 2202.11061v2, accepted by *Operations Research* in June 2024.
  - p. 1: status of the paper
  - p. 7: Balinski and Young 1975, the quota method
  - p. 9: Balinski and Young 1982, which needs the extra assumption
  - Table 1 and Theorem 1: profiles A, B and C at 10 seats; every share is pinned to Table 1 in `facts.rs`
  - the randomized result used in 14_7
- **H. Peyton Young**, *Fairness in Apportionment*, U.S. Census Bureau PDF.
  - PDF p. 10, Table 4: New York after 1830, quota 38.593, 40 seats
  - PDF p. 13 (printed p. 12): Seaton's letter, used verbatim in 06_5, with the 275–350 range and the 8 → 7 change from 299 to 300 seats
  - PDF p. 16: Huntington Hill adopted in 1941
- **Competing treatments.** These come from the existing brief, `docs/video-program/research-20261016/evidence/apportionment-brief-20261031.md`, under "Competition and disposition". Earlier researchers read those transcripts and sampled single frames. I read only the brief: no competing video was watched or listened to, and I read no transcript myself.
  - Mathispower4u's *Alabama Paradox* already explains larger towns' remainders overtaking a smaller town's (4:33–5:07, transcript). So this film's mechanism is not new. What it adds is the dial picture and one set of towns carried through every rule.
  - My one web search for videos returned only text treatments (Wikipedia, Cut the Knot, Tufts notes, LibreTexts), which I saw as titles and snippets.

## Remaining gaps (explicit)

1. **Rust is uncompiled.** By rule, no cargo or rustc runs on this machine. Only syntax (rustfmt) and the model/facts logic (the Python mirror) were checked. The chapter code was type-checked by reading only. The API assumptions most likely to break a build:
   - `Transform` between text mobs with different glyph counts (the counters, cutoff labels and card values)
   - fading `always_redraw` mobs after `clear_updaters`
   - the closure pattern bindings in the card filters
   - `stage.move_to` used as a statement

   None of the tests have run: `the_spoken_numbers`, the storyboard and registry test, the cue-order test, the voice rules and the draw layout tests.
2. **Nothing has been rendered, and the glyphs have not been probed.** `√ ≈ ÷ ö` were added to the glyph supplement. Other crates use them, but this crate's glyph probe has not run.
3. **No Oliver takes exist for these cues.** Synthesis, listening, take selection, fine sync, captions, score and independent audiovisual review are all open.
4. **Phrase cues are drafts.** `Spoken::wait_for` interpolates inside a sentence until measured marks exist. It never waits backwards, so an animation that runs longer than its phrase drifts. The likeliest case is the 14-step walk in 12_2, about 3.4 s.
5. **Balinski and Young 1975 is unread in the original.** The IIASA copy sat behind bot protection and the AMS Feature Column returned 403. Two claims rest on the citation at GPP p. 7 plus the film's own check for 1 to 35 seats (a tie at 36): the "simple form" in 12_2 and the lower-quota guarantee in 12_4. Wikipedia's *House monotonicity* notes that general capped divisor methods need an extra lower-quota safeguard. Root or a reviewer should confirm that the Jefferson-priority version needs none.
6. **No fresh-reader test** of the script has been done, and no comparison against the other three replacement films.

## Strongest unresolved teaching defects

1. **Length.** The estimate is 26.6 min. Chapter 14 runs 174 s, chapter 02 126 s and chapter 10 118 s. If chapter 14 is too dense, the natural split is "the proof" (14_2–14_5) and "what is known" (14_6–14_7).
2. **The 05_1 dial demonstration is weaker than written.** Moving one hand alone would move every hand, so the scene flashes the whole-part ticks instead. The idea of one hand per town is carried by narration more than by motion.
3. **The card moves in 13_5 and 15_4 are exact but may read as noisy.** Cards that swap ranks pass through each other, and their rank numbers morph mid-move. Ash's own change in 13_5 (4,000 → 3,900) moves no rank, so "moves down" shows only as a value change.
4. **Text density.** The citation and own-check captions in 11_2 and 12_4 are 15–16 px and up to about 107 characters. That is likely too small and too long to read at the pace of their lines.
5. **Leaving the running example.** The four-town map (11) and the five-town proof (14) depart from Ash, Birch and Cedar, and the bridge back is verbal only.
6. **Partial Seaton quote.** All five lines are shown, but only the last two are highlighted as spoken; the first three are left to the viewer to read.
7. **Hill's rounding point is shown only between 1 and 2** (√2 against 1.5). The general rule is narrated, not drawn.
8. **Vague assumption in 14_1.** "One extra assumption about how rules treat bigger towns" is accurate but leaves the assumption unnamed. A reviewer may want it stated precisely, from the primary text.
