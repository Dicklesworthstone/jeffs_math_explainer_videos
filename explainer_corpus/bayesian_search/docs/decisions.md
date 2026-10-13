# Decisions — How to Find a Lost Submarine (Video H)

One row per choice a revision must start from. Status: **order** (the
owner's order fixed it), **director** (chosen here, marked `(default)` in
the brief, open to the owner), **review** (forced by a review finding; the
record is in `docs/reviews/bayesian_search/`), **pending owner** (needs his
call). A change to any row is made on purpose and written here. The words
every revision keeps are `docs/protected.tsv`.

| What | Why | Where it lives | Status |
|---|---|---|---|
| The owner's subject line opens the video, in his words | the order | 01_1; `protected.tsv` | order |
| The truth about the 99 by the third line of chapter 02; never implied savable; no cause of loss | the order's honesty rule | 02_1 to 02_3, 14_5; storyboard "The dots" rules | order |
| Voice `michael`; chapters ≤ 85 s; Maya, Dev, Ruth | the order | brief | order |
| Title "How to Find a Lost Submarine" | the order's working title, kept | brief, glyphs | director |
| The one picture: dots, each one place it could be; a search puts out the dots it would have found | the 1968 map and the Coast Guard's planner are made of exactly this | storyboard "The dots" | director |
| A made-up four-cell sea for every hand calculation, its ship "it" | numbers a viewer can count; cannot pass for the Navy's scenarios | chapters 03 to 09 | review (C4) |
| The 1968 prior map as Richardson and Stone printed it, transcribed twice | the real map, checked against its 10,000 | `model::y1968`, oracle | director |
| Five countdown quizzes, all on the made-up sea or Koopman's cell; no ring over a real person | tone (R1) and the owner's request for quizzes | `model::QUIZZES`; storyboard | review (C3) |
| The theorem shown as its proof (slices), with its three conditions spoken | a greedy rule needs its reason; the fan of 65,536 curves could not be drawn | 08_2, 08_3 | review (D5, R2, E2) |
| The ending: the first map must leave room for the truth | "never needs the first map to be right" was false | 14_3, 14_4 | review (R1, D1) |
| AF 447 told as its paper tells it: both maps near the centre, the investigators planned from the first | the paper, p. 75, Figures 7 and 8 | 12_5, 12_6, 14_2 | review (R10) |
| Palomares told where the idea began; the bomb found by the fisherman's sighting and Alvin | the official report | 04_1, 04_2, 14_2 | review (M8) |
| No Chivas Regal bets, no list of the nine scenarios, no acoustic signal counts | not in a primary source; sources disagree | fact sheet "Not said" | director |
| Palomares spoken as "Paloh-mah-ress"; Aldridge as "Alldridge"; Scorpion, Craven, Bayes, Norfolk plain and unverified | the probes in michael | `lexicon.json` | director; pending owner for the unverified four |
| Length: 16.1 min by the G2 estimate, over the order's 15 | R1 and the claim walk added about a minute | storyboard "Length" names the cuts | pending owner |
| Sound: silences after the two sentences that name the 99; no pings, alarms or ticks | tone | storyboard "Sound plan" | director |
| 08 draws the first hundred dots as bars (slices), not as dots | the rule's searches put out 13⅓, 7½, 6⅔ of a hundred: a part of a dot cannot be drawn, and 07_5 split the dots in three for the same reason; the heat and the live percents are still recounted from the lit counts | `where_next.rs`, 08_1 to 08_3 | director |
| 08_2 and 08_3 show every cell's next slice, so the columns are twelve bars (4, 3, 3, 2), not the storyboard's ten | the proof's step is that what is left is smaller than what was taken; the ninth biggest slice (A's 4.2) stands beside the eighth taken (C's 4.4), where the storyboard's ten left it out | `where_next.rs`; test `the_eight_tallest_slices_are_the_rules_in_its_order` | director |
| 09_3's band is the whole range of the thousand runs, and goes on with the formula's curve to three coverages | "a band of a thousand runs" needs an edge a viewer can state; the video's own run (66.5%, the lucky side of 63.4) lies just above the middle nine in ten, and a run that falls outside its own band would read as an outlier | `the_second_pass.rs`; test `the_band_is_the_thousand_runs_and_the_formula_runs_through_it` | director |
| 09_5's nine dots are drawn after the cell with the thirty and its lanes leave; the plot keeps the band and the curve | the quiz's frame goes round a live picture with nothing else on it that could give the answer away; the plot is where the three points of 09_6 land | `the_second_pass.rs` | director |
| 10_3 clears the hour line and the model tag before its lower-third | the two-line lower-third (sweep width, Table H-15) and the hour line's labels shared the foot of the stage | `adrift.rs` | director |
| Three citations are facts, so the screen types no number: the helicopter's 300 feet (10_3), the years of AF 447's three underwater searches (12_2), the page that prints the update (13_1) | the storyboard's strings carried numbers no fact held | `facts.rs`, `oracle.py`: `F-D.strip.altitude.feet`, `F-A.underwater_searches.years`, `F-S.update.page` | director |
| 11_2: the boat moves down its track at the speed the facts imply (F-C.aldridge.miles_south over F-C.aldridge.hours_off, 6 mph), the clock mark beside it, so the hollow point is left behind where it passes 40 miles | the storyboard says only "left behind"; one speed ties the hour line, the track and the 30 miles together | `twelve_hours.rs` | director |
| 11_3 to 11_6: each dot is placed around the point the track says he went in, spread by the model's drift error over the hours to the 08:30 re-run; the mean drift is not drawn; the point a map is run from is an open dim diamond and the white ring is the truth | what the map was told is not where he was: the two must be two marks | `twelve_hours.rs` | director |
| 11_5: the hatch and the pumps sit at 56 and 62 miles of a track drawn to 66, unlabelled; 11_4 sets the whole line as the question, not its last sentence | "far to the south" is a place, not a number; the line is the question's context | `twelve_hours.rs` | director |
| 12_4's two seconds hold with `hold` and `pause`, not `carry_pause` | `carry_pause` marks "predict" (the sting) and the storyboard asks for two seconds with nothing sounding | `the_silence.rs` | director |
| 12_2 to 12_6: the searched areas, the third map's smear and the five listening tracks are schematic and say so ("areas searched: schematic", "weights chosen by judgment; shapes schematic"); the circle stands right of centre so the labels have a column; 12_6 draws one track thick through the ring on both discs | the sources give no coordinates for the shapes; the ring is the one sourced place (6.5 nm, 12 km) | `the_silence.rs` | director |
| 12_3 and 12_5: the three method discs show a tenth of each map's dots (105, 105, 90) before they cross-fade into the 3,000; each disc of 12_5 carries every second dot (1,500), the same places on both; 12_5's lower-third is headed "the authors' better way:" | 6,000 dots in two discs is the engine's limit for a static picture, and the same places make the two maps comparable | `the_silence.rs` | director |
| 13_2 and 13_6 carry two small tallies ("94 out, 6 still lit", "65 out, 35 still lit"); 13_5's window carries the label "chance a search of F6 saw it"; 13_4 dims the map under pane 3's veil and 13_7 lifts it, and shows E5 and F6 as small ink labels | each says in words what the dots and the curve show, from the facts' counts | `how_well_they_looked.rs` | director |
| 14_4's map is 0.30 a row, not the cold open's 0.46; each of 14_2's panels is captioned with its name, the day its story ended and its label (Palomares's "found near the fisherman's mark" is the video's own wording); the Scorpion's flaw is a red band 800/6076 of a cell high, the Air France flaw three red tracks, the update plain text | the three hold lines must fit under the map; the panels must read without the narration | `four_maps.rs` | director |
