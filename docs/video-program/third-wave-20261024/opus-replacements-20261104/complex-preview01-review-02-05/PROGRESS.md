# Progress: complex preview01 picture review, chapters 02–05

Working log, updated as sheets are read. `REVIEW.md` is the report. Frames are under
`narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01/<chapter>/`.

Reviewer: BronzeJaguar, who wrote these chapter scenes (see REVIEW.md on independence).

## 02_turning_numbers

Sheets 000–065 read in order, 66 of 66; frames 0–260 on them. Native re-read: frames 47, 84, 124, 190, 223.

Candidate defects, confirmed on the native frame where a frame number is marked (n):

- 02-A "−1" sits on the real axis: the minus is collinear with the axis and fuses with it. frames 83–108, 32.8–40.4 s; (n) 84. Lint caught it. "i" also sits on the imaginary axis, frames 79–108; (n) 84.
- 02-B arrows stop short at both ends (about 0.24 units each end at (n) 223). 02_1 says "an arrow from zero" and 02_2 says "tip to tail". (n) 47: blue tip to green tail gap, sum arrow tail off zero. frames 24 onward.
- 02-C the 1+i arrow and arc are the F's yellow and start at the F's foot, so they read as one glyph. frames 122–135, 45.0–50.5 s; (n) 124.
- 02-D the grid sweeps through the fixed "1+i" label during the second multiply. frames 212–218, 77.5–80.0 s. It then rests in a cell beside the 2i arrow, and the 2i arrow itself is unlabelled. (n) 223.
- 02-E "turned: 90 °": the degree sign floats well away from the number. frames 55–74. Other cards use a joined "45°".
- 02-F the quiz card uses upright i and a Unicode square, "(1 + i)² = ?"; the answer card uses math italic. frames 177–204; (n) 190.
- 02-G long still: nothing moves from about 84 s to 96.6 s at the scratch rate. Re-judge against Oliver.

Predict 02_5: hold verified on frames 156–204. No answer before the think window ends. Ring 69.5–74.0 s. "total turn" first appears at frame 220, 81.0 s, after the second multiply has finished. The diamond's top corner is at 2i but unlabelled.

Maths on the frames: 3+2i and 1−i sum to 4+i; F turns the right way (stem up to stem left at 90°); readout reaches 90 at frame 69; grid corners go (1,1) to 2i after one multiply by 1+i; final grid is ±2, stretch 2.

Extra 02 measures (native px, origin (388,382), 90 px/unit): frame 47 blue tip (638,216), green tail (674,218), gap about 36 px = 0.4 units; green tip (732,275), sum tip (726,291), true 4+i at (748,292). Frame 223: 1+i arrow tail (403,367), tip (463,308), true (478,292); 2i arrow tail y≈360, tip y≈226, true y=202. Low: names card frames 133–158 about twice as wide as its text; upright w and i in text cards. Faint dark ghost shapes on native 84 and 223 look like encoding residue, not judged as a scene defect.

## 03_two_planes

Sheets 000–078 read in order, 79 of 79; frames 0–314. Native re-read: 74, 140, 215, 255, 300.

- 03-open frames 0–17 (0–7.0 s): dot, arrow and dot float on the empty background with no planes. Planes arrive 7.5–8.0 s (frames 18–19) under them; the arrow crosses the box edges. Labels 10.0 s. Static 10.5–15.0 s.
- 03-empty frames 44–53 (17.5–22.0 s): empty planes about 4.5 s while the voice states lengths and angles. Re-judge against Oliver.
- 03-A "turns:" readouts straddle the vertical axis, the value's left edge almost touching the axis. frames 54–78. Check native.
- 03-B one-decimal rounding makes doubling look wrong at moments: frame 56 0.1/0.1, frame 60 0.3/0.7, frame 62 0.5/0.9. Low.
- 03-C frame 71 (30.467 s): format jumps from "1.0"/"2.0" to "1"/"2" and shifts position. Low.
- 03-D "radius 0.25": the vertical axis runs through the word ("radiu|s"), lint caught it. Label is up only about 0.6 s, 30.6 to 31.27 s, frames 73–77. Radius ½ on the input is never labelled. Check native 74.
- Maths: radius ½ and ¼ on planes ±1.5 at the same scale (61 and 30.5 sheet px); output trace goes round twice; turns end 1 and 2.
- 03-E the planes never carry a tick or a unit, and the window changes silently between beats: ±1.5 (walk), ±2.2 (grid), ±2.5 (1/z), ±3.2 (exp output). 1 unit is 122, 83, 73, 57 sheet px. Frames 54–78 / 79–176 / 178–226 / 227–314. two_planes.rs:45–47 says this is deliberate. Teaching call for root.
- 03-F 1/z colour clash: the near dot is blue with a green image, and the four tick pairs use blue↔blue, green↔green. So the blue dot inside reads as paired with the blue tick, and the green dot outside as paired with the green tick. frames 194–226; (n) 215.
- 03-G "1/0 undefined" card sits on the circle, tangent to the green tick (card left 248, dot right edge about 246) and to the white tick (card bottom 475, dot top about 475). frames 213–226; (n) 215. Lint did not list it.
- 03-H all three radius labels are crossed by their own circles: "2.718" by radius e, "1.000" by radius 1, "0.368" by radius 1/e. Lint named only 2.718. frames 253–276; (n) 255; sheet 064–066 for 0.368.
- 03-I the four rays land exactly on the plane's own axes, so "horizontal lines become rays from zero" reads as the axes turning yellow. frames 263–276 onward. y = 0, π/2, π, 3π/2 only. Proposal: rays at other angles too.
- 03-J fold tick: a white vertical bar at the output origin with no label, frames 133–176 (51.0–69.5 s); (n) 140. Source calls it the fold (two_planes.rs:399–407). It reads as a stray mark, and the red arc straddles it from frame 160.
- 03-K exp input frame is 3–4 px taller than the output frame at each end, native y 145 vs 148, 638 vs 634. (n) 300. exp_input's comment says same top and bottom. Low.
- Not a defect: the e^{2πi} card covers the ray and circle cleanly at native 300, digits unbroken.
- Long stills, re-judge against Oliver: 03_6 opening frames 228–234 (89.5–92.0 s), empty frames while the voice says the grid turns familiar; 03_7 end frames 294–312 (113.5–120.0 s).
- Predict 03_3: card up from frame 87 (33.5 s), ring frames 100–109 (37.5–41.5 s), no bent line until frame 115 (43.0 s), after 03_4 starts at 41.733. Holds.
- Maths on frames: grid ±1 at 83 px/unit; x=±½ parabola vertex at +¼, y=±½ vertex at −¼, x=±1 vertex +1, y=±1 vertex −1, tips at ±2i; (½+½i)² at ½i; real axis to positive ray, imaginary to negative ray. Right-angle marks at ½+½i, −1+½i, ½−i map to the crossings at their squares. Red right angle at 0 maps to a straight 180° arc. 1/z: near dot computed at 1.78−1.21i vs drawn (848,414); ticks flip across the real axis; unit circle 73 px. exp: 0.368, 1, 2.718 at 21, 57, 155 px at 57 px/unit; lines climb to 2π (319 px at 50.8 px/unit); dot at 98° on both sides at frame 282.
- Added on 03: at (n) 74 the axis at x=325 runs between "turns:" (x 228-279) and its value (x 337-348), confirming 03-A; axis x=955 runs through "radius", confirming 03-D. Output grid's centre lines are deliberately absent in frames 113-126 and come in at frame 127 (two_planes.rs:387-398). Teaching read for 03: strong; right-angle marks map to their true crossings, the red right angle at 0 becomes a 180 degree arc, circles and rays are drawn in sync with the input lines.

## 04_reading_color

Sheets read in order. Running count at the foot of this section.

- 04-A horizontal stripes across the painted picture. Frames 12 onward; (n) 20. checks/stripes.py on native 20, rows 300-339: luma dips about 15 of 166 (about 9%) roughly every 3 native px, with a slower beat of about 18 px. Likely cause (source plus measurement, not proven): RowStrips draws each row as a polyline stroke exactly one cell tall (crates/explainer_kit/src/props.rs:232), abutting the next, so both anti-aliased edges let the dark background through at each boundary. CELL = 0.035 (draw.rs:523) = 3.15 native px, which matches the period. Not verified at 1080p/4K.
- 04-A2 the same fixed cell gives stair-stepped ring edges once bands are on: frames 62 onward (26.5 s). At 4K a cell would be about 9.45 px, so the steps grow in pixels while staying the same fraction of the picture. Not measured at 4K.
- 04-B no formula on screen for the painted function. reading_color.rs has no formula title; f(z)=z, z^2, 1/z and the bump are named only by voice. No axes on the picture until 04_6. Confirm on later frames.
- 04-C the rule is told, not shown. Frames 13-31 (5.0-14.0 s) are static while 04_1 says "At each input point, find the output, and colour the point by the output's direction." No probe point, output arrow or link to the wheel. Teaching candidate; re-judge against Oliver and compare with approved films.
- Low: wheel labels "negative" and "positive" (dim, frames 2 onward) are never spoken or explained. Frames 38-60 (17.5-25.5 s) static picture, while the voice moves on to "Colour alone hides how big the output is".
- Holds: colours match voice and wheel (right red, up yellow-green, left teal, down violet); direction names arrive frames 32-37 (14.5-17.0 s), one at a time. Bands fade in frames 61-63 (26.0-27.0 s) after "brightness climbs in bands". "next band: size x 1.69" from frame 65 (28.0 s). On sheet frame 66 ring radii about 183, 108, 64 sheet px: ratios 1.69 and 1.69, as the card says.
- 04-D "simple pole" is pink on the darkest band of 1/z and is nearly unreadable; it needs a close look to read at native size. frames 197-219 (79.4-84.9 s); (n) 200, zoomed crop against "zero of order 2" at (n) 140. Lint listed the location at 79.4, did not judge contrast. "zero of order 2" (frames 135-185, white) is legible but crosses many colour edges; weak, not failing. Both sit on the picture at p(0,-0.35), source reading_color.rs:117,133.
- 04-E the loop's direction arrow is a tiny head about 8 native px, same gold as the ring, sitting on the inner edge and not reading as tangent. frames 95-221; (n) 140, 200; crop. "Same order as the wheel" and "backwards" both depend on direction, and the wheel carries no direction either. Thickness 5.0 at reading_color.rs:39.
- 04-F "colour cycles around 0: -1" uses a hyphen, not a minus, and -1 is never said aloud (voice says "the other way"). frames 196-221. Low. Colour of the card (pink) is the pole colour, consistent.
- 04-G at the predict the function is not on screen: "where are the bump's poles? how many turns at each?" frames 231-263+, while 1/(1+z^2) was only spoken. Also axes carry no tick at i or -i, and the unit circle is unlabelled. Strengthens 04-B: a viewer who missed one spoken phrase has nothing to reason from. Formula title proposed.
- 04-H stale cards: "next band: size x 1.69" and "dark lines: every twelfth of a turn" stay beside the blank plane during the question, frames 223-263+. Low. Predict ring squeezed between card and "next band", gap about 9 sheet px, frames 257+.
- 04 cut at 85.5-86.0 s: picture fades to an empty box while 04_6's first sentence "Here is the bump" is being said (04_6 starts 84.733; frames 219-224). So the voice introduces the bump over a blanking frame.
- Predict 04_6 so far: no colour on the plane from frame 223 (86.0 s); card from frame 231 (88.5 s); line ends 97.06; ring from frame 257 (98.0 s).
- Maths on frames: z^2 reds on both sides of the real axis, top teal, 45 degrees yellow-green, 135 degrees violet, so two cycles in wheel order. 1/z: right red, top violet, left teal, bottom yellow-green, so reversed. Bands of 1/z brighten outward from pole's dark? see frame 200; bright rings alternate, consistent with size falling outward.
- Chapter 04 count: sheets 000-079 read in order, 80 of 80; frames 0-316. Native re-read: 20, 140, 200, 300. Authorship: reading_color.rs is listed as mine in complex/previous/handoff01-mail750/HANDOFF.md line 60.
- Predict 04_6 holds: plane has no colour from frame 223 (86.0 s) to frame 272 (103.433 s). Card frames 231-271 (88.5-103.1 s). Ring frames 257-268 (98.0-102.5 s). First colour at frame 273 (103.5 s), poles marked at frame 277 (105.0 s), both after 04_7 starts at 102.933.
- 04-I "one backwards cycle at each: -1 and -1" frames 277-315. The sentence already says one; "-1 and -1" repeats it with hyphens. Low. (n) 300.
- 04-J gold circle is stroked across the pink pole markers, so the markers the sentence is about are struck through, frames 284-314; (n) 300. Proposal: draw the markers after the circle.
- 04-K thick gold segment from -1 to 1 is unlabelled and unexplained in 04_7 except by "That is the opening puzzle". Teaching call, low; depends on chapter 01's picture.
- Maths on frames, bump: red along the whole real axis (f real positive there). Teal on the imaginary axis beyond the poles (f negative there), red between them. Far field 1/z^2: top right violet, top left yellow-green. Poles at the crossing of unit circle and imaginary axis. Radius-1 circle touches both.

## 05_gallery

Sheets read in order; running count at the foot of this section.

- 05-A comb seam on e^z: a jagged vertical seam down the column x = 0, bright one-cell teeth, frames 86-126 (32.0-47.0 s); (n) 92 and zoom. Cause from source: exp_plane is 91 columns wide (round(3.2/0.035)), so the centre column samples x exactly 0, where |e^z| is 1 give or take rounding; shade() takes ln(size)/step rem_euclid 1, so rounding flips each cell between band top (0.58) and band bottom (1.0). Numerical artefact, not mathematics. It sits on the one picture whose claim is "exact squares". To be confirmed by checks/comb_and_bars.py.
- 05-B walker's output box is unlabelled throughout, as is the "+1.4" readout until "turns:" arrives. frames 17-30. Teaching, low.
- 05-C empty output box beside the rational function for about 11 s, frames 41-69 (16.3-27.5 s). Low.
- 05-D "turns: -2 (backwards twice)" is up at full strength only about 0.2-0.3 s: not at frame 79 (30.5 s, reads -2.0), full at frame 80 (30.7 s), fading at frame 81 (30.9 s), nearly gone at frame 82 (31.0 s). The walk runs past 05_2's end (30.079) and is cut by 05_3's start at 30.7. Compare 05_1's card, 13.5-16.0 s. Scratch rate; re-judge against Oliver, but nothing protects the hold.
- 05 unit circle is a thin dark line of low contrast, frames 8-40. Low. Edge zeros of sin half off the picture at +-2pi, frames 131 onward. Low.
- Holds: cubic zeros at 120 degrees on the circle; 3 turns traced; (z-1)/(z+1)^2 labels on dark cards are legible, which is the model for repairing 04-D; exp tiles measure 18.8 sheet px each way against computed 0.5236 x 0.533 units; bracket 226 sheet px against computed 2 pi x 0.533 units; sin zeros 141.5 sheet px apart, colours alternate as sin's sign does.
- 05-D source: walk_and_count with phrase None runs (line.end - now).max(2.5), and 05_2 calls it from "where the colours run backwards", three words from the line's end. So the walk overruns the line at any plausible rate, and `shown` then gets only what is left of the hold before 05_3's fade (gallery.rs:261-290, 365-392). Not a scratch-only effect.
- 05-E sin(3i) dot and card sit beyond the white arrow's tip: dot at 3i, tip about 2.6 units; the card's top overhangs the picture's top edge by about 9 sheet px. frames 165-183 (62.5-66.4 s). Lint listed the location. Low to medium.
- 05-F "so the colours there go flat" (05_5) against full-contrast vertical bars above and below the row of zeros and poles, frames 188-227+ (67.5-80.3 s). Hue is flat as said, but |tan| near 1 and arg near 90 degrees both sit exactly on a sawtooth jump of shade(), so the tiniest ripple flips brightness across the bars. They never fade with height. A correct plot of shade(), but it reads as structure where the voice says flat. To be confirmed by checks/comb_and_bars.py.
- 05-G gold real line is drawn across the zero dots and the upright "sin x" in the card, frames 139-183. Low. tan cards and readouts legible; long hold 72.0-80.3 s. Re-judge against Oliver.
- Holds: sin(i)=1.1752i rounds to 1.18, sin(3i)=10.018i; tan zeros and poles alternate at pi/2 spacing; above yellow-green (up, i), below violet (down, -i).
- 05-H gamma: "integral works to the right" is card on the picture; legible. Left half dimmed from frame 244 (86.5 s) to 262 (92.0 s). 5 poles marked one at a time frames 271-279 (95.0-98.0 s) while voice says "at zero, at minus one, at minus two", fine. "no zeros anywhere" card 99.5 s. No maths problem on frames: poles 63 sheet px apart, at 0, -1, -2, -3, -4; none to the right. Gamma(4)=6, Gamma(5)=24.
- 05-I zeta opens with formula alone on an empty frame, frames 308-328 (106.0-112.6 s), 6.6 s, formula top left and nothing else. Picture arrives 113.0-113.5 s. Re-judge against Oliver, but it is the emptiest passage in these four chapters. "series converges: Re s > 1" has an upright s, against the formula's italic s, and sits low right of the formula, not aligned, frames 333-347.
- 05-J zeta inset grammar: gold box on the picture covers x -1 to 2, y 0 to 3, and one gold leader runs to the strip, whose border is the same gold. That is the grammar of "this box, enlarged", but the strip runs y 0 to 32, so the box holds only the strip's bottom 3/32 (9.4%), and none of the four marked zeros is in it. Only the leader's far end is true (x -1, y 0): it runs from the box's top right corner (2,3) to the strip's bottom left (-1,0), which pairs no matching corners, gallery.rs:691-695. The dim 18-point caption "strip squashed: 0 <= Im s <= 32" is the only disclosure. frames 366-~430; (n) 367. Proposal: mark the box as a base with an up arrow and "continues up to Im s = 32", or drop the leader.
- 05-K zero labels "14.13i, 21.02i, 25.01i, 30.42i" name the height only; the points are 1/2 + 14.13i etc. As written they name points on the imaginary axis, and they sit at x = 1/2. frames 368-~430. Proposal: "1/2 + 14.13i" on the first, then heights; or a small header "height". Low to medium.
- 05-L the zero labels line up with the right column and read as a table: "25.01i  a single pole: s = 1", "21.02i  zeros at -2, -4, -6, ...". Gap about 39 sheet px, baselines 4-10 px apart. frames 371 onward. Low to medium.
- Holds: zeta marks at -2, -4, -6 and pole at 1 on the picture at 43.2 sheet px per unit; dashed line at x=1; strip line at midpoint (Re = 1/2); four zero dots at sheet y 299, 214, 164, 97 against computed 299, 214, 164, 97 for 398 sheet px = 32 units. Heights are the published 14.1347, 21.0220, 25.0109, 30.4249 to two decimals.
- Running count: chapter 05 sheets 000-095 read, frames 0-383. Native re-read so far: 92, 367.
- 05-M mosaic panels carry no function names; six panels, all at full view only about 2.5 s, frames 444-450 (149.5-152.0 s); (n) 446. Marks on them 150.0-151.5 s. Sin panel's edge zeros straddle the panel border, frames 445-448. Low to medium; depends on the voice.
- 05-N last frames: the zoom holds to chapter end; "next: what one complex derivative demands" from frame 467 (158.0 s). Static from about 158.0 s to 165.2 s. Re-judge against Oliver.
- Holds: tile traced by gold outline follows the band and phase edges on (n) 464; sides nearly equal on the sheet. Chapter 05 complete: sheets 000-122, 123 of 123, frames 0-490. Native re-read: 92, 367, 446, 464.
