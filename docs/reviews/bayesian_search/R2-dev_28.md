# Review R2 (script), "How to Find a Lost Submarine". Reviewer: Dev, 28

Saved verbatim from the text the reviewer returned (2026-10-13); the reviewer
did not write the file itself. Finding ids D1 to D26.

I read the persona prompt and the whole packet (brief, script, storyboard, description, fact sheet) in order. I did not open any other file under `docs/reviews/bayesian_search/`. I wrote nothing in the repository; scratch files are in the session scratchpad.

**What I ran** (`python3 -I`, standard library, exact fractions where possible):

- **Four-cell sea (my own code, from the packet's words only).**
  - Prior 40/30/20/10; chance of a find 10, 20, 13⅓, 6⅔.
  - After B comes back empty: 50, 12.5, 25, 12.5. After C: 60, 15, 10, 15.
  - Rule order B C A A B D A C A A A B, with found-by 20.0, 33.3, 43.3, 50.8, 57.5, 64.2, 69.8, 74.2.
  - Brute force over all 4^8 = 65,536 orders: the rule is best at every n from 1 to 8; row by row is level after 3, 5 and 6.
  - Slices 10, 7.5, 5.6 / 20, 6⅔, 2.2 / 13⅓, 4.4 / 6⅔, 2.2.
  - Expected searches 6.34, 8.00 and 7.04.
  - Quiz sums: 8 of 9; 26 of 27.
  - Koopman: 39.3, 63.2, 86.5, 95.0, and 66.7 at 1.1 and 25.2 at 0.29.
  - The code panel run as a loop, with `i` initialised and the first-index tie-break, gives the same order.
- **1968 map.**
  - I used the oracle's transcription of Figure 2 as data only, and wrote my own search.
  - Reproduced: total 10,000; 172 printed cells; E5 1,250; B7 1,096; F6 85 at rank 24; 8,900 inside the grid, 1,100 outside.
  - Reproduced model search at 20 and 81 searches: 49.8/44.3/0.8 and 73.2/57.8/32.9; first cells E5, B7, C5, G5, E5, B7.
  - Not reproduced from the packet alone: the transcription itself against the printed figure. I could only check its internal sums.
- **Chapter 13.** F6: 0.85 × (1−0.94) ÷ (1−0.73) = 0.19%, and 0.85 × 0.35 ÷ 0.55 = 0.54%. 94% and 65% come out of coverage 2.81 and 1.05. 0.35 ÷ 0.06 = 5.8.
- **Chapter 10.** Radius 1 + 0.45h gives 1.45, 2.35, 3.70, 6.40 miles. Coverage 18 ÷ area gives 2.73, 1.04, 0.42, 0.14, so 93, 65, 34, 13. The flight-fitted 86% and 73% follow from the packet's counts.
- **Dates.** 5, 9, 63, 154, 670 days; 11 h 28 min; 5 hours.
- **Unchecked** (never "fine"):
  - All source quotations, and Chew's "Theorem 2.1".
  - Seeded counts: 1,689 / 1,707 dots inside, the 66.5% run.
  - AF 447 weights and labels, and the beacon numbers.
  - The description's "two in five" variant.
  - Colour distances, tests and clippy.
  - Whether the audio matches the text.

No blocker: I can write a correct sentence for every chapter.

## Comprehension check

1. **The question.** Someone asks how you find a missing sub in time, and a wordless demo shows that a search that finds nothing still redraws the map.
2. **Ninety-nine.** In 1968 the Navy asked it for real about the *Scorpion*, searched nine days as if the crew could be saved when they were already lost, and still had to choose where to look, because passing once over a 12-mile patch with a 200-foot strip is about 4.5 times everything the sled towed.
3. **A map of chances.** Weigh two stories 60 to 40, imagine 100 dots, count them per cell, and you have a map (40, 30, 20, 10), which makes "the likeliest cell" the natural first search.
4. **Ten thousand dots.** The Navy built its map the same way (nine weighted stories played out 10,000 times). Its top cell E5 held 1 dot in 8, and the wreck was next door in F6 (85 dots), so the map alone did not find her.
5. **A search can miss.** The chance of a find is the chance it is there times the chance of seeing it, so the canyon cell A (10%) loses to B (20%) although A is likeliest.
6. **Nothing found.** B's empty search puts out 20 of 100 dots; B drops to 12.5%, and every cell nobody searched rises by a quarter because what we know changed.
7. **Bayes' rule.** That move is Bayes' rule (multiply by the chance of an empty search, divide by what is left); after it the next search is C, not A, and the loop is search, empty, update, choose.
8. **Where next.** Greedy is optimal for a still wreck with independent equal-cost searches, because each search takes a fixed, shrinking slice of the first hundred dots; on the 1968 map the rule beats a no-map plan mostly because it has the map.
9. **The second pass.** Strips laid at random cover 63% of a cell per full coverage (about 2 in 3 at 1.1 times that); a second fresh pass catches 2 in 3 of what is left, so 8 of 9, never certain.
10. **Adrift.** A person drifts, the cloud grows with the square of time, and the same flight's chance of a find falls from 93% at hour 1 to 13% at hour 12 (model).
11. **Twelve hours.** The Coast Guard has run this loop since 1974. For Aldridge the first map was wrong because the time of the fall was guessed 5 hours early; new evidence moved the dots about 30 miles and a helicopter found him alive.
12. **The silence.** For AF 447 the analysts weighted three maps, folded in each empty search, and drew the map twice because the beacons' silence might have meant nothing; both pointed at the centre and the wreck was there.
13. **How well they looked.** On the *Scorpion* the ship's numbers called F6 94% searched because the sonar was believed to see 800 feet; measured afterwards it was 65% and the sonar saw nothing. The cell had nearly been written off and the sled went back for a scrap of metal.
14. **Four maps.** An empty search only counts if it could have found the thing. So the first map must leave room for the truth, you must judge how well you looked, and you need a new map when the evidence breaks a story. The 99 could not be saved; the method now looks for people who can be.

## Rubric (1 to 5)

| chapter | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 the question | 4 | 5 | 5 | 4 | 4 |
| 02 ninety-nine | 4 | 4 | 5 | 3 | 3 |
| 03 a map of chances | 5 | 5 | 4 | 4 | 4 |
| 04 ten thousand dots | 4 | 5 | 4 | 3 | 4 |
| 05 a search can miss | 5 | 5 | 5 | 5 | 4 |
| 06 nothing found | 5 | 4 | 5 | 4 | 5 |
| 07 Bayes' rule | 4 | 4 | 4 | 3 | 4 |
| 08 where next | 4 | 3 | 4 | 3 | 5 |
| 09 the second pass | 4 | 4 | 4 | 4 | 4 |
| 10 adrift | 3 | 3 | 5 | 3 | 4 |
| 11 twelve hours | 4 | 4 | 5 | 4 | 5 |
| 12 the silence | 3 | 4 | 4 | 3 | 3 |
| 13 how well they looked | 4 | 4 | 5 | 4 | 5 |
| 14 four maps | 4 | 4 | 5 | 4 | 4 |

## Findings

Findings that would change the video are first. Nothing below asks for a re-narrated line unless it says "words".

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| D1 | Dev | 08_4, 08_5 | major | "Twenty searches row by row have less than one chance in a hundred." My own search over the transcribed map reproduces 0.8%, but only for row by row starting at row 1, where the map is nearly empty (114 of 10,000 dots). Same plan from row 5: 22.3%; from row 12: 0.2%; column by column from A: 10.3%. A random order averages 6.6% (10th to 90th percentile 2.4 to 12.6). So the chart makes the rule look about 60 times better than a typical no-map plan, when it is about 7 times. "Most of that is the map itself" is still true (6.6, then 44.3, then 49.8), but the number I would test against is the worst start. | Narration stays. Relabel the bottom curve "row by row from the north edge (row 1 holds 1% of the map)". Add a dashed marker at 20 searches: "no map, random order: 6.6% on average, 2 to 13%". Pin it as a fact in the oracle. If one word must change, 08_4 "goes row by row" becomes "goes row by row from the top edge". |
| D2 | Dev | 08_5 chart, 08_4 | major | The 49.8, 44.3 and 0.8 are shares of all 10,000 runs, but 1,100 runs ended outside the 180 lettered cells and no plan on the chart searches them. The video never says so. If I normalise over the cells I can see, which is the natural thing to code from the picture, I get 55.9, 49.8 and 0.9. The "never updated" plan then lands exactly on the 49.8 the video prints for the rule, and I would conclude the video is wrong. | Add to the chart's axis note: "percent of all 10,000 runs; 1,100 ended outside these 180 cells and are never searched here". Say the same in the description's 1968 bullet. |
| D3 | Dev | 02_5, 02_6, 13_4 | major | Act I's hook (search everywhere fails, 4.5 times the track) uses the strip measured afterwards, and 02_5 says "It saw a strip about two hundred feet wide" with no hedge. Only a small screen label says "measured afterwards". With the 800 feet they believed, one pass was 1,148 miles, about the 1,026 they towed, which is why 13_2 can say 94% searched. The storyboard promises "chapter 13 comes back to that", but no picture or line in chapter 13 does. A viewer meets 94% searched after being told the area was unsearchable and will not know why. | Picture only. At 13_4, beside the red "believed at sea: sonar 800 ft", add "one pass over the patch at that width: 1,148 sea miles, about what the sled had towed (1,026)". Keep "measured afterwards" large at 02_5. |
| D4 | Dev | 08_3 (and 09_5) | major | "Searches that are fresh tries of equal effort" is the second of the theorem's conditions. It is spoken before it is shown, and nothing on screen at 08_3 defines it (the ledger puts "independent" only in the description). I read it as independent trials, but a non-developer has no handle. The lower-third gloss of Chew, "the least chance of still searching", is garbled: the theorem is the least chance of not having found it yet. "Equal effort" also does not cover transit, which the description says is taken as free. | Screen text at 08_3, no spoken change. Define: "fresh try: a miss now says nothing about the next try (independent, same chance each time)". Replace the Chew gloss with "after any number of searches, the least chance of not having found it yet". Add: "moving between cells costs nothing here". |
| D5 | Dev | 14_4 (with 08_5) | major | "The first map does not have to be right" is qualified by "leave room", but the practical cost is not said. I planted a cell in the 1968 map and ran the rule (2 in 3 everywhere). A truth at 85 dots (F6) is first searched at search 30; at 10 dots at search 160; at 1 dot at search 429, about 12,000 sea miles at the model's 28 per search. 40 of the 180 cells hold no dots and are never searched. 08_5 itself says the map is most of the gain. A developer reads "does not have to be right" as "any prior will do". | Keep the narration. Under the "never adds" line at 14_4 add: "a thin place is reached late: 85 dots, search 30; 1 dot, search 429 (model)". Pin the three numbers in the oracle. If one word changes: "does not have to be exactly right". |
| D6 | Dev | 10_3, 10_4 | major | I recomputed 93, 65, 34, 13 from the radii, so they reproduce, but I could not test them from what is spoken or drawn. The inputs live in the description: 180 sea miles of track, spread evenly over a circle, flight taken as instant. A 180-mile track is more than an hour of helicopter, during which the cloud moves and grows. "Twice the radius is four times the area" is a cartoon: hours 1 to 12 are radius 1.45 to 6.40 miles (4.4 times), area 6.6 to 129 square miles (19.5 times). Quartering 93% gets you nowhere. | Picture only. At 10_3, put "flight: 180 sea miles of track" next to the strip. Under each of the four markers at 10_4, print radius and area (1.45 mi, 6.6 sq mi; 2.35, 17.3; 3.70, 43; 6.40, 129). Add "flight taken as instant; cloud as a uniform disc". |
| D7 | Dev | code panel (07_2, 08_3) | major | Run as a loop it is correct: I got B C A A B D A C A A A B. As shown it cannot be pasted. `i` is read on line 1 before anything assigns it, because line 4 feeds the next round. `p` and `d` are never defined, and there is no loop or stop. The comment on line 2 says "adds up to one again", the voice at 07_2 says "a hundred" and "divide by eighty", and in code `t` is 0.8. Line 2 is not needed for the choice on line 4, because the normaliser is common, which is why a heap works. | In the description's "For programmers", give a runnable block: define `p`, `d`, `i = argmax(p*d)`, then `while True:` and the four lines. On screen, change the line-2 comment to "# 80 dots left, scaled back to 100% (1.0 here)". Note that line 2 is only for display. |
| D8 | Dev | 03_4 vs 06_2, 06_3 | minor | 03_4 defines a dot as "one place the wreck could be". At 06_2 the same dots are put out as "the ones the search would have caught", and at 05_2 the same spare dot is seen 1 time in 4. So a dot has become a place plus this search's luck, one equally likely world. As a coder this is the difference between a position and a sample with a pre-drawn coin. The lower-third at 06_3 says it, but after the dots are already out. | Show the teal ring on the 20 dots before they go out (the grammar already says so), and extend the 06_3 lower-third to "one equally likely way things could be, including whether this search would see it". |
| D9 | Dev | 02_5, 02_6 on screen; 08_4 | minor | "One cell: 26 sea miles" and "all 180 cells: 4,594": 26 × 180 = 4,680. The exact figure is 25.52 (0.84 mi × 6,076 ÷ 200 ft). 08_4's 28 miles is 25.52 × ln 3, which only chapter 09 explains. The "about 3,000" two-sensor figure has no key among the 303 (only 4,966 camera alone and 4,176 magnetometer alone). Adding the two widths gives 2,268, which is 2.2 times 1,026, not "several". | Write "25.5 (26 passes of a mile)". Replace the "together saw somewhat wider" footnote with the two single-sensor figures, or pin the combined figure and its rule. |
| D10 | Dev | 02_5 vs 13_4, 13_5 | minor | 02_5 says "the search gear ... saw a strip". At 13_4 "the sonar" arrives with a definite article and a 0 width, next to a camera and a magnetic detector I was never told were on the sled. | Picture at 02_5: the sled labelled with three sensors, "sonar, camera, magnetic detector". Show sonar as "800 ft believed, 0 measured" so 13_4 is a callback. |
| D11 | Dev | 14_3 (and 12_5, 12_6) | minor | The listening tracks light red, the role for "an estimate that turned out wrong", in a line with the 800-foot strip and the 10:30 p.m. guess. In AF 447 the analysts handled the doubt the way the video wants (two maps). The sources only support "the beacons' failure is the authors' inference". The colour says more than the narration dares. | Light the listening tracks in `dim` or ink with the label "doubted, so drawn both ways". Reserve red for the two errors that were measured. |
| D12 | Dev | 11_6 vs 10_4 | minor | 10_4 ends "twelve hours in ... thirteen in a hundred. That is why hours matter", and 11_6 says "almost twelve hours in the water" and alive. The chain "twelve hours, 13 in 100, he survived" is easy to draw. The storyboard note ("not a free drifter, not chapter 10's 13 in 100") is not on screen. | Add to the 11_6 label: "held on to buoys: not a free drifter; chapter 10's 13 in 100 is a model of the search, not of him". |
| D13 | Dev | 10_3 | minor | "Counts on spotting someone ... only inside a narrow strip" and the lower-third "the width of the strip" read as a hard edge. The description (correctly) says sweep width is an effective width: it sees some outside and misses some inside. | Lower-third: "sweep width: an effective width. Some seen outside, some missed inside. Table H-15 is for one object, height and visibility." |
| D14 | Dev | 11_1 | minor | 08_3 limits the proof to "a wreck that stays put". 11_1 says the Coast Guard "plans searches this way" for drifting dots, and nothing on screen says the guarantee is gone. The brief promised one sentence ("the simple rule is no longer guaranteed"); it ended up only in the description. | Lower-third at 11_1: "for a moving target the rule is no longer proved best (Brown 1980)". |
| D15 | Dev | 04_2 | minor | "That bomb was found near where a fisherman watched it fall, but the idea stayed." I could not tell what the idea did at Palomares. The description says it kept the books on how well each area had been searched; the narration only says the map did not find it. | Label at the 04_2 picture: "what the method did there: keep the books on which areas had been searched, and how well". |
| D16 | Dev | 11_3; storyboard 11 note | minor (unchecked) | The narration says "almost thirty miles", but the fact key is 30 and the brief says "about thirty"; "almost" implies below 30. The storyboard's source note says the narration says "a lobster buoy", while 11_6 says "two lobster buoys". | Confirm against the Tough article. If it is 30, change one word at 11_3 ("almost" to "about"); otherwise pin the real figure. Reconcile the buoy count in the note and the script. |
| D17 | Dev | 12_4, 12_3, 02_5 | minor (unchecked) | Three items I could not tie to a fact key. "For a month, ships had listened" has no key. The assignment of 35, 35, 30 to the three labelled maps ("anywhere in the circle", "near the last position", "traced back from what was found afloat") is not in the packet. "Towed just above the sea floor" has no source row. | Add a key and source for each, or soften the wording. |
| D18 | Dev | description, "For programmers" | minor | Two things. The storyboard's Dev note for ch 08 ("the rule is a merge of four sorted lists") never reached the description. "A deleted dot ... would be the wrong thing to code" is overstated. Deleting each swept dot with probability d is valid rejection sampling (same posterior in expectation, more variance, fewer survivors), and reweighting is the low-variance form. | Add: "The rule is a k-way merge: each cell yields a decreasing stream pᵢdᵢ(1−dᵢ)ᵏ. A heap on the next slice gives O(log N) per search; the code panel's normalisation makes it O(N)." Replace "wrong thing to code" with "valid but wasteful: it throws away sample size; weights keep every dot". |
| D19 | Dev | description "Where it holds"; fact sheet always-sentence 06_8 | minor | "Every unsearched cell rises by 1 ÷ (1 − pᵢdᵢ)" holds after "a search that can see (dᵢ > 0)". The factor is also 1 when the searched cell had no chance (pᵢ = 0), so "1 only for a search that cannot see" is incomplete. | Say "after a search of a cell that had some chance and that can see (pᵢ > 0 and dᵢ > 0)". Same in the fact-sheet range column. |
| D20 | Dev | description, drift paragraph | minor | "About two in five rather than 13 in 100" is called "a check run during review, not a pinned fact". It is the one unpinned number in a public description. My crude variant (radius 1 + 0.45 × √12 = 2.56 mi, same 180 miles) gives about 58%, so the figure depends on a radius definition the packet does not give. | Pin it in the oracle with its definition, or drop the parenthesis. |
| D21 | Dev | 07_2 | minor | "Then divide by what is left, eighty, so the map adds up to a hundred again." 40 ÷ 80 is 0.5, not 50. The screen shows "÷ 80: 50, 12.5 ...", so a percent factor of 100 is hidden. The chapter also stacks table, derivation and code in one hold. | Show the step as "40 ÷ 80 = ½ = 50%", or label the column "÷ 80, as percent". Keep the code lines for the hold after the table settles. |
| D22 | Dev | length note (08_4, 08_5 cut) | minor | Of the cuts the storyboard offers, 08_4 and 08_5 cost the mechanism nothing; chapter 13 depends on 07_3's formula and chapter 09's curve, not on them. It is also the chapter's most error-prone piece (D1, D2). It is the only place the video says how big the gain is, though. | If it stays, fix D1 and D2; if you must cut for length, cut it first, as listed. |
| D23 | Dev | 06_3 to 06_6 | delight | Dots go out, the row of 80 is dropped against a fixed ruler 0 to 100 and stretched, and 10 becomes 12.5. That is normalisation as one gesture, and A rising with its dots untouched is the best moment in the video. Nothing to change; keep the ruler fixed and the dots in place. | None. |
| D24 | Dev | 08_2, 08_3 | delight | The proof is real: each search of a cell takes a fixed, shrinking slice of the first hundred, so the n best searches are the n biggest slices, and the greedy rule takes them in that order. I checked it against all 65,536 orders; the rule is never behind at any n, and ties are why "ahead" is the right word. | None. |
| D25 | Dev | 13_6 | delight | Dots relight, once, "the only time in the video". Seeing 94% fall to 65% (6% left becomes 35%, almost six times as much) is the cost of a wrong d at a glance. | None. |
| D26 | Dev | 14_1 | delight | The strip that sees nothing and changes nothing (d = 0) is the missing edge case, and the line is exactly the sentence a tester would write. | None. |
