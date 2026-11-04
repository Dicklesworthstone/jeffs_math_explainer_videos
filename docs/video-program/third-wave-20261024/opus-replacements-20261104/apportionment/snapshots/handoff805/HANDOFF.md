# Adding a Seat — corrected candidate, handoff 2 (bead .30, Mail 805)

**From:** WhiteSwan (Claude Code, Opus 5.5) · **To:** root / BlueSalmon · **Date:** 2026-11-04 UTC
**Status:** candidate source only. Not compiled, rendered, synthesized, listened to, synced or independently reviewed.
**Previous handoff:** Mail 798, preserved byte for byte in `snapshots/handoff798/` (34 files, nothing in it newer than the snapshot). This round's delivered state is copied to `snapshots/handoff805/`.

## Narration: ElevenLabs Oliver

Every cue is written for **ElevenLabs Oliver**. I generated no audio and launched no TTS, render, Cargo or other heavy job. I did not touch the canonical crate (`find explainer_corpus/apportionment -newer snapshots/handoff798` lists nothing), Beads, shared files or any take. Synthesis belongs to root's guarded serial lane.

## What Mail 805 asked, and what changed

| Mail 805 item | resolution | cues | files |
|---|---|---|---|
| Exact Balinski–Young algorithm and hypotheses from the primary paper; tests to 35 cannot establish lower quota; no capped-Jefferson claim without proof or source | **Primary source read.** Balinski & Young, PNAS 71(11) 4602–4606 (1974), p. 4604, read by eye from the PMC page scans. E(h+1) = {i; a_i < p_i(h+1)/Σp_j}, and the next seat goes to the largest p_k/(a_k+1) in E(h+1). Theorem 2: "There exists a unique consistent, house-monotone method Q = Q(p) satisfying quota." This is exactly the rule the film shows; the year is 1974, not 1975. The paper gives no proof, so the film now proves the quota half itself in five new beats. Each town's seats become tiles along the house sizes. The share rounded down is the number of passed tiles, and rounded up the number of entered ones. The rule hands out the entered unheld tile that ends soonest. A counting argument on the stretch since the last later-ending award shows a passed tile is never left out. The film calls this "one way to see why", not Balinski and Young's proof, and does not claim their uniqueness result. `checks/theorem_evidence.py` stays as supporting evidence only (720,000 + 360,000 apportionments within quota) | 12_2, 12_4, 12_5–12_9, 12_10 | `both_promises.rs`, `model.rs` (`quota_rule` per the 1974 recursion, tile functions, `STRETCH`), `facts.rs`, `oracle.py` |
| 05_1 independent dial mechanism, separately addressable town phases, no flashes for narrated motion | One value tracker per dial (`house_trackers`, `town_dial`, `three_dials`). In 05_1 Ash's dial is built at 0 and its own tracker runs to 15, passing the top once at 9 3/4 seats (guarded). Birch and Cedar then appear at 15. In 05_4 each hand moves alone at the phrase that names it. 06 and 15_5 use the same per-town trackers | 05_1 (words), 05_4, 05_5, 06_1, 06_2, 15_5 (pictures) | `dials.rs`, `draw.rs`, `not_every_time.rs`, `recap.rs` |
| 13_5 / 15_4 card trajectories: stable labels, distinct lanes and ghost endpoints, value-only change language for Ash | Cards are keyed (town, seat), and each keeps its label, seat number and value together. A fixed rank column sits beside the list. `slide_cards` moves each card along its own lane with ghost outlines at both ends. 13_5 states the rule; the new 13_6 walks the census one town at a time: Birch's 4th passes Cedar's 11th; Cedar's 13th, once tied with Ash's 2nd, moves ahead of it; at Ash's step only the values change ("no card moves", guarded by identical key order). 15_4 now says Ash's and Birch's "keep exactly the same values" and are "pushed down", never up (guarded) | 13_5, 13_6 (new), 13_7 (old 13_6), 15_4 | `population.rs`, `recap.rs`, `draw.rs` |
| 11_2 / 12_4 captions: larger, simpler, brief source on screen, full citation in the description | 11_2: "3 towns: Webster keeps the quota promise" at 24 pt, over "Balinski & Young 1982, via Gölz, Peters & Procaccia 2024" at 20 pt. 12_4: "Balinski & Young, PNAS 1974, Theorem 2" at 22 pt. The old own-check caption is gone from the screen. Full citations are in the new `docs/description.md` | 11_2, 12_4 | `webster_quota.rs`, `both_promises.rs`, `docs/description.md` |
| Hill threshold √(n(n+1)) animated with neighbouring integer cases | 10_5 slides the 1-to-2 mark from Webster's 1.5 to √2 ≈ 1.41. The new 10_6 uses a wide ruler from 0 to 4: one tick per pair starts at its halfway mark and slides to its threshold as the line names it (≈ 1.41, 2.45, 3.46). Hero bars show the slides shrinking, and finally the 0-to-1 tick slides all the way to 0 ("any quotient above 0 rounds up to 1 seat"). Guard: the first threshold is 0 and the gaps decrease. 10_7 carries the consequence (every town gets a seat; Hill breaks quota with three towns) | 10_5, 10_6 (new), 10_7 (old 10_6, reworded) | `rounding_rules.rs`, `model.rs` (`hill_threshold`), `facts.rs` (`hill.threshold.0..3`) |
| Explicit bridges to the four- and five-town maps | 11_2 now proves why three towns cannot fail. One town is a whole seat short, so the yardstick is above the exact one, and each other town is less than half a seat over; the picture is a red seat box that two sub-half-seat chips cannot fill. With four towns there is half a seat of room. 11_3 shows exactly that room: each of the three small towns is lifted by a third of a seat (+1/3 chips that fill a "one whole seat" box). 14_2 explains why the proof needs more than three towns ("with three towns Webster's rule keeps both promises") before it introduces the city and four villages | 11_2, 11_3, 14_2 (new) | `webster_quota.rs`, `no_rule.rs`, `facts.rs` (`w4.lift_thirds.*`) |
| 14_1: state the exact extra assumption from the primary text | Spoken: "A town with strictly more people than another must never get fewer seats." On screen, verbatim: "if state i has strictly larger population than state j, then i must receive at least as many seats as j" (Gölz, Peters & Procaccia, arXiv 2202.11061v2, §3.1, naming Robinson and Ullman's order-preserving property; Balinski and Young 1982 itself is unread) | 14_1 | `no_rule.rs` |
| Seaton quote reading time | The screen shows only the sentence the narration speaks, verbatim, at 28 pt italic, with a 20 pt source line. 06_6 holds it at full strength until (quote + source characters) ÷ 15 s have passed since it appeared (`READ_CPS`, `wait_until`), however fast Oliver reads. The full letter passage and citation are in the description | 06_5, 06_6 (pictures) | `not_every_time.rs` |
| Duration is not a defect; no arbitrary shortening or splitting | Nothing was cut or split. The film grew to 93 cues and 4,129 spoken words; `tools/storyboard.py` estimates 31.4 min (its per-chapter "longer chapter" notes are advisory) | — | — |

### Further defects I found and fixed this round

- **12_2's walk was Jefferson's, not the quota method's.** The old animation counted down Jefferson's list to 14. But the quota method already skips once at 6 seats: Cedar's share is exactly 4, so its 5th seat (Jefferson's card 6) waits until 7. Only the state at 14 agreed, so the old guard missed it. The walk now follows `model::quota_rule`, with the ceilings at each new house size. Card 6 waits in red exactly at "If it would, skip that signpost for now". Guards: exactly one skip before 15, resolved one seat later, and the waiting card must exceed its town's ceiling. The strike of card 15 moved to 12_3's "so it is skipped". 12_7's tile walk shows the same two waits (at 6 and at 15).
- **15_4 flashed the wrong card.** At "any signpost of Ash's that was below it", the old code flashed Ash's 2nd, which was above the old cutoff. No Ash card on the shown list was ever below it. The beat now shows "Ash's next signpost: 1,600 / below the cutoff, before and after" (new fact `webster.15.ash_next_signpost`, pinned and mirrored), guarded to be below the old cutoff, which does not fall.
- **12_9 showed 5 real tiles under a caption saying 8.** The hero rings of the whole tiles really inside the stretch now appear at "But each town's row fits at most", where they are the count being bounded. At "So all those tiles fit inside the stretch" the band flashes instead.
- **Precision of words and captions.**
  - Tiles are "entered" (strictly past the start), not "reached", because a tile starting exactly at the marker is not yet counted.
  - 12_8 now covers the start case ("or to the very start if it never did") and says why ("or the rule would have picked a tile ending sooner").
  - 13_5's captions read "only get bigger / only get smaller".
  - Hill's first-seat caption says "quotient", not "share".

## Oliver cue map (for serial synthesis)

**No take exists for any of the 93 candidate cues**, so all 93 go to synthesis, one Oliver take per cue in script order (`narration_units.json`, `oliver_<cue>`, 1–100 words each). Bind no existing take by id: every id that also exists in the canonical 23-cue script carries different words. Since handoff 798:

- **Unchanged words, same id (66):** 01_*–04_*, 05_2–05_5, 06_*, 07_*, 08_*, 09_*, 10_1–10_4, 11_1, 11_4, 11_5, 12_1, 12_3, 13_1–13_4, 15_1, 15_3, 15_5.
- **Unchanged words, new id (7):** 13_6 → 13_7, and 14_2–14_7 → 14_3–14_8.
- **New or changed words (20), verbatim from `script.tsv`:**
  - `05_1` (67 words): Picture each share as a hand on a dial. One full turn of the hand is one whole seat. Watch Ash's dial as the council grows from no seats to fifteen. Each time the hand passes the top, the whole part ticks up by one, and the tail starts again from zero. Birch's and Cedar's dials show their own shares at fifteen seats in the same way.
  - `10_5` (63 words): One more relative is worth knowing, because it became the law for the United States House in nineteen forty one. It is called the Huntington Hill method. Between one and two, it rounds up once a quotient passes the square root of one times two, about one point four one. For our towns it agrees with Webster at fifteen, sixteen and seventeen seats.
  - `10_6` (81 words, new content): Each pair of neighbouring whole numbers gets its own threshold, the square root of the two multiplied together. Between two and three, it is the square root of six, about two point four five. Between three and four, the square root of twelve, about three point four six. Each one sits a little below Webster's halfway mark, and closer to it the higher you go. But between zero and one, the threshold is the square root of zero, which is zero.
  - `10_7` (78 words, new id): So whenever there are enough seats to go round, Hill's rule gives every town at least one seat, and that can break the quota promise even with three towns. At three seats it gives each town one seat, though Cedar's share is exactly two. Every rule in this family reads one list of signposts, so none of them ever takes a seat back, and every one of them can break the quota promise for some map of towns.
  - `11_2` (100 words): Not with three towns. Balinski and Young proved it. Here is why. To break the promise, a town must end a whole seat or more below its share, so its quotient is below its share, and the yardstick sits above the exact one. Then every other quotient is below its share, and rounding to the nearest adds at most half a seat. So the other two towns end less than one seat above their shares, too little to cover the missing seat. Turned around, the same reasoning rules out a whole seat too many. With four towns, it can happen.
  - `11_3` (76 words): Take four towns of two thousand, five thousand, eight thousand and twenty seven thousand people, sharing fourteen seats. The biggest town's share is exactly nine. The three small shares are point six seven, one point six seven and two point six seven, and rounding to the nearest lifts each one by a third of a seat. Three small towns, three thirds, one whole seat. Rounding everything to the nearest hands out fifteen seats, one too many.
  - `12_2` (79 words): Birch. Michel Balinski and Peyton Young published exactly this rule in nineteen seventy four, and called it the quota method. Here it is in a simple form. Hand out seats one at a time, from the top of Jefferson's list. Before giving a town its next seat, check that the seat would not lift it above its share rounded up at the new house size. If it would, skip that signpost for now and go to the next one.
  - `12_4` (48 words): Each house is the previous house plus one seat, so nobody ever loses one. And Balinski and Young's theorem says the skipping never strands a town below its share rounded down either. Both promises, at every house size. Here is one way to see why that is true.
  - `12_5` (64 words, new content): Lay each town's seats along the house sizes as a row of tiles. Every added seat raises a town's share by its fraction of the people, so each tile is as long as the stretch of house sizes it takes to earn one whole seat. Ash's tiles are nine and three quarters long, Birch's four and a third, and Cedar's one and a half.
  - `12_6` (55 words, new id): Now slide a marker along the house sizes. A town's share, rounded down, counts the tiles the marker has passed completely. Rounded up, it counts the tiles the marker has entered. So the quota promise says that every tile the marker has passed is already handed out, and no tile it has not entered is.
  - `12_7` (55 words, new id): And Jefferson's list is these tiles in the order they end. A seat's signpost is the population divided by the seat number, so the bigger the signpost, the sooner that seat's tile ends. At every house size, the quota method takes the entered tiles nobody holds yet and hands out the one that ends soonest.
  - `12_8` (75 words, new id): Could a passed tile ever be left out? Suppose one were. Go back to the last moment the rule handed out a tile that ends later than the missing one, or to the very start if it never did. Every tile handed out since then, and the missing tile too, had not been entered at that moment, or the rule would have picked a tile ending sooner. And every one of them ends by now.
  - `12_9` (93 words, new id): So all those tiles fit inside the stretch from that moment to now, and there is one more of them than seats added in the stretch. But each town's row fits at most the stretch times its fraction of the people, and the fractions add up to one. So a stretch never holds more whole tiles than the seats added across it, and no passed tile is ever left out. The quota method never hands out a tile before the marker enters it, so it keeps the quota promise at every house size.
  - `12_10` (36 words, new id; old 12_5 with "quota method"): Notice that the quota method gives one, four, ten at fifteen seats, while Hamilton gives two, three, ten. Both keep the quota promise. That promise can allow more than one answer, and different rules choose differently.
  - `13_5` (69 words): Divisor rules cannot do this. Every signpost is a population divided by a fixed number. When a town grows, all of its signposts get bigger, so they can pass the signposts of towns that did not grow, but never fall behind them. When a town shrinks, all of its signposts get smaller, so they can fall behind the signposts of towns that did not shrink, but never pass them.
  - `13_6` (79 words, new content): Watch Jefferson's list through the census, one town at a time. Birch grows, and its fourth signpost passes Cedar's eleventh. Cedar grows a little, and its thirteenth signpost, once tied with Ash's second, moves ahead of it. Ash shrinks, and only the values on its cards change. Its first signpost falls from four thousand to three thousand nine hundred, and no card moves. At sixteen seats the list still gives one, four, eleven, before the census and after it.
  - `14_1` (63 words): Balinski and Young proved in nineteen eighty two that the answer is no, with one extra assumption. A town with strictly more people than another must never get fewer seats. A recent paper by Paul Gölz, Dominik Peters and Ariel Procaccia proves the answer is still no without that assumption, for any rule that always gives the same answer to the same question.
  - `14_2` (34 words, new content): Their proof needs more than our three towns, because with three towns Webster's rule keeps both promises. It uses a city and four villages sharing ten seats, and we can follow all of it.
  - `15_2` (51 words): Jefferson, Adams, Webster and Hill never take a seat back and never punish growth, but each can break the quota promise. The quota method of Balinski and Young keeps the quota promise and never takes a seat back, and the theorem says it must break the population promise for some map.
  - `15_4` (68 words): No. Cedar's signposts all get bigger, while Ash's and Birch's keep exactly the same values. As Cedar's climb past some of them, those are pushed down the list, but none can move up. So the cutoff for fifteen seats can only rise, and any signpost of Ash's that was below it stays below it. Ash can keep its seats or lose some, but it can never gain one.
- **15 prediction holds:** one after each of 01_3, 02_4, 03_3, 04_3, 05_3, 06_1, 07_2, 08_3, 09_1, 10_3, 11_1, 12_1, 13_3, **14_3** (was 14_2) and 15_3. Each is bound to its chapter's `think` line and that file's sha256; rerun `checks/bind_oliver_plan.py` after any chapter edit.
- **Words Oliver has to get right:**
  - Gölz
  - Huntington Hill
  - Seaton
  - Balinski
  - Procaccia
  - "thirty ninths"
  - "point six seven"
  - "nine and three quarters"
  - "the square root of one times two"
  - "nineteen seventy four"
  - "nineteen eighty two"

  No cue contains a digit, colon, semicolon, dash or parenthesis.

## Files

Every `candidate/explainer_corpus/apportionment/X` maps to `explainer_corpus/apportionment/X`. Changed since handoff 798:

- **Narration and plan:** `docs/storyboard.md` (226 lines); `script.tsv` (93 cues, sha256 `cfcec06911c7…`); `narration_units.json` (`8b89ca40c399…`); `docs/narration-direction-map-oliver03.json` (`e1e8f58851c5…`). These hashes come from the rebind after the last `recap.rs` edit; the first copy of this handoff gave stale ones, since corrected in Mail 1012's thread.
- **Code:** `src/model.rs`, `src/facts.rs`, `src/main.rs`, and `src/chapters/{mod,draw,dials,not_every_time,rounding_rules,webster_quota,both_promises,population,no_rule,recap}.rs`.
- **Facts:** `oracle.py` (429 keys).
- **New:** `docs/description.md` and `docs/description_numbers.tsv`.
- **Staging tools:** `checks/rust_mirror.py` (mirrors the new key) and `checks/theorem_evidence.py`.
- **Do not integrate:** `candidate/explainer_corpus/apportionment/.ee/hook-state/session_start_orient.last.json`. The session-start hook wrote it into the candidate dir. It is not part of the film, and I did not delete it.

## Exact checks run (all passed; from the repository root unless noted)

`S` is the staging dir; `C` is `$S/candidate/explainer_corpus/apportionment`.

| check | command | result |
|---|---|---|
| storyboard → script | `python3 -I -B $S/checks/regen_script.py . $C` | 0 problems; 93 cues; 31.4 min estimate |
| cue phrases | `python3 -I -B $S/checks/phrase_scan.py $C` | 93 lines said once, in order; 237 literal `wait_for` phrases in their own line, in spoken order, 0 problems. The phrase lists passed through loop variables (05_2, 05_4, 12_5, 13_2, 13_6) were checked by hand |
| voice rules | Python stand-in over `script.tsv` | 93 cues, longest 100 words (11_2), 0 digits or forbidden marks |
| oracle | `(cd $C && python3 -I -B oracle.py --check)` | exit 0; 429 keys; cross-checks pass |
| Rust logic mirror | `(cd $S && python3 -I -B checks/rust_mirror.py candidate/explainer_corpus/apportionment/oracle.py)` | mirror 429 keys, oracle 429 keys; differ `[]` |
| theorem evidence | `(cd $S && python3 -I -B checks/theorem_evidence.py)` | quota method: 720,000 and 360,000 (tie-heavy) apportionments all within quota; Webster with 3 towns: 239,968 within quota (32 cutoff ties skipped); a 4-town failure found; Hill seats every town whenever seats ≥ towns. Evidence, not proof |
| Rust syntax and format | `rustfmt --edition 2024 --check $C/src/main.rs` | exit 0 across every module |
| Oliver plan | `python3 -I -B $S/checks/bind_oliver_plan.py $C --check` | files current; 15 holds |
| plan against repo tooling | `python3 -I -B $S/checks/validate_oliver_plan.py . $C <empty scratch dir>` | `direction_contract` resolves 15 directions; 93 units cover 93 cues in order; the hold markers have current hashes |
| description numbers | the repo's `tools/description_numbers.py` copied into an empty scratch tree with `docs/description.md` and `.tsv`; run with `--oracle` = oracle stdout and `--facts` = the same output with the thousands commas removed | PASS: 185 numbers, 181 equal an oracle fact, 4 allow-listed (1880, 6.2, 4604, 2010). **Legacy mode:** the tool's regex splits "4,000" into 4 and 000, so grouped facts need the comma-free copy, and the description names keys in prose, not in the tool's `[X-Y.key]` tag form. A number only has to equal *some* fact, which is weak |
| canonical untouched | `find explainer_corpus/apportionment -newer $S/snapshots/handoff798 -type f` | nothing |
| private paths | grep for home paths, user names and temp paths in `$S`, excluding snapshots | one hit, in root's `ORDERS.md` line 4 (not my file); none in mine |

## Sources (full citations in `docs/description.md`)

- **Balinski & Young**, "A new method for congressional apportionment," PNAS 71(11) 4602–4606, 1974, doi:10.1073/pnas.71.11.4602 (PMC433936). Read by eye from the page scans: p. 4604 (E(h+1), Theorem 2 and the recursion). No proof is given there.
- **Gölz, Peters & Procaccia**, arXiv 2202.11061v2: §3.1 (the order-preserving quote; Balinski and Young 1982 Proposition 6.2 on three states), Theorem 1, Table 1 and the lottery result.
- **H. P. Young**, *Fairness in Apportionment* (U.S. Census Bureau PDF): PDF p. 10 (New York 1830), p. 13, printed p. 12 (Seaton), and p. 16 (Hill 1941).
- **Not read:** Balinski & Young 1975 (Monthly), and *Fair Representation* (1982) itself. The 3-town Webster result and the 1982 assumption rest on GPP's citation; the film adds its own 3-town argument.

## Remaining gaps (explicit)

1. **Rust is uncompiled and Clippy has not run.** Only rustfmt (syntax) and the Python mirror (model and facts logic) were checked; the chapter code was type-checked by reading only. New risks from this round:
   - the slice-pattern `match waits[..]` in 12_2
   - closures returning `fmn::Result` that are passed to `map`
   - the `cut_at` closure taking `&POP` as `&[u64]`
   - `facts.get(..)` assumed to return `Option<&str>`

   Earlier risks remain: `Transform` between texts with different glyph counts; fading `always_redraw` mobs after `clear_updaters`. No Rust test has run (`the_spoken_numbers`, the pinned facts including the new `webster.15.ash_next_signpost`, storyboard and registry, cue order, voice rules, layout).
2. **Nothing has been rendered.** Layout, text sizes and lane clearance are designed numbers, not seen frames. The glyph probe for `− × → – ö √ ≈ ÷ ≤ §` has not run.
3. **No Oliver takes exist.** Synthesis, listening, take choice, fine sync, captions, score and independent audiovisual review are all open. I listened to nothing and watched nothing.
4. **Phrase timing is interpolated** until measured marks exist. Known overrun: after "and go to the next one", 12_2's walk from 7 to 14 seats and the ceiling reset take about 3.6 s, probably 2–3 s past the end of the line, before 12_3 starts.
5. **Secondary attribution.** The 1982 assumption and the 3-town Webster result are cited through GPP, not read in Balinski and Young.
6. **No fresh-viewer test** and no comparison with the other replacement films. Competing videos were not watched (only the earlier brief was read).

## Strongest remaining teaching defects

1. **Chapter 12 is the densest stretch:** 10 beats and about 257 s, with a five-beat proof. The tile picture, the passed/entered convention and the stretch argument are exact, but no viewer has tried to follow them. This is the first place to test with a fresh reader.
2. **The supposition in 12_8–12_9 cannot be drawn literally.** The run never fails, so the missing tile is shown by emptying a held tile, and "8 tiles" exists only as a caption beside the 5 that really fit. It is honest, but some viewers may read it as a mismatch rather than a contradiction.
3. **The 11_2 argument is schematic.** The seat box and half-seat chips are generic, not our towns' numbers, and the line is 100 words of reasoning over a mostly static picture.
4. **Text density in 14_1.** Seven lines: the result, the assumption heading, the two-line verbatim quote, its source, then three GPP lines. No reading-time guard there, unlike Seaton's quote.
5. **Silent motion after 12_2** (gap 4) may read as a stall until fine sync.
6. **Long-form length (31.4 min estimate).** Root has ruled that duration is not a defect, but the pacing of chapters 10, 12 and 14 has had no listening review.
