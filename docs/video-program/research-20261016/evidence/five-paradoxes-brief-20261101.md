# Five mathematical paradoxes: working video draft

Owner request, 2026-11-01: make a video explaining five counterintuitive mathematical paradoxes clearly. The selection below is an editorial judgment, not a measured ranking of every paradox. The owner has requested production; this document is a draft, not a finished video or permission to publish. Narration, when made, uses ElevenLabs. No continuous listening, fine-sync acceptance, independent mathematical or teaching review, or audience testing has occurred.

## The idea

**Working title:** Five Math Paradoxes That Make Sense Once You See Them

Each chapter poses a question, lets the viewer predict, and then shows the precise hidden assumption or comparison. One visual device recurs: the narrator pauses the picture, labels what is being counted, and changes one thing at a time. The suggested sequence moves from three doors to a crowd, a scoreboard, a race, and an infinite hotel. These are familiar problems; clarity and visual execution are the proposed value, not a claim of new mathematical discoveries.

## Draft narration and picture sequence

### Opening

Picture: a closed door, two birthday badges, a scoreboard, a runner and an endless row of hotel doors. Each appears to contradict a simple expectation.

“A game show host opens a door, and switching gives you the better chance. One machine passes inspection more often on each kind of part, yet trails overall. A runner crosses infinitely many checkpoints and still catches a tortoise. These are not tricks where math stops working. Each one asks us to look more carefully at what the picture leaves out. Here are five favorites, and the small idea that makes each one click.”

### 1. Monty Hall: what does the host know?

Picture: three doors; the viewer chooses the first. Freeze before the host reveals a goat. Display the host's rules visibly: knows the car, always opens an unchosen goat door, always offers a switch.

“There is a car behind one door and a goat behind each of the others. You choose a door. The host knows where the car is, always opens a different door with a goat, and always gives you the chance to switch. Should you?

“Imagine that you always choose the first door. The car is equally likely to be behind any of the three doors. If it is behind your first choice, switching loses. If it is behind either other door, the host must reveal the remaining goat, and switching wins. One losing case, two winning cases. The chance of winning by always switching is two out of three.

“The key is that the host's action follows a rule. If the host opens doors at random, or offers a switch only sometimes, this calculation need not apply.”

Picture: show all three equally likely car placements in parallel, then reveal the host's forced choice in the two switching-win cases. In the first case either goat can be shown; do not infer the posterior for a particular named door without specifying how the host chooses when both are available.

Author-built interactive picture experiment: [three possible worlds](five-paradoxes-monty-prototype-20261101.html). Its host chooses door 2 when both unchosen doors are goats. Desktop author review checked the initially hidden outcomes, all three reveals, and the 2-of-3 summary. This is a visual prototype, not video footage or independent accessibility/pedagogy review.

Check: [Granberg, 1999, Monty Hall Dilemma](https://www.sciencedirect.com/science/article/pii/S0376635799000662); [Duke interactive host protocols](https://matheverywhere.pages.oit.duke.edu/course/Modules/probability/).

### 2. Birthdays: which pair are we asking about?

Picture: one person appears, then 22 others. Lines connect all pairs; show the counter growing to 253, while emphasizing that pairs are not independent trials.

“How many people must be in a room before there is more than a fifty-fifty chance that some two share a birthday? Under a simple model, only twenty-three. Not a match to your birthday: a match between any pair.

“Assume each birthday is independently and uniformly distributed among 365 days. It is easier to count the opposite: nobody matches. After the first person, the second must avoid one day, so that chance is 364 out of 365. The third must avoid two days, giving 363 out of 365, and so on. Multiply those no-match probabilities through person twenty-three. The result is about 49.27 percent. The chance of at least one match is the remainder: about 50.73 percent.

“There are 253 possible pairs, which helps explain the surprise. But those pairs are not independent, so simply multiplying a per-pair probability 253 times would not be the calculation we just did.”

Picture: reveal `1 − (364/365)(363/365)…(343/365) ≈ 50.73%`. These assumptions simplify real birthday distributions; they are part of the model, not a claim about any particular room.

Check: [Wolfram MathWorld, Birthday Problem](https://mathworld.wolfram.com/BirthdayProblem.html). Author calculation: 0.5072972343239854; 23×22/2 = 253.

### 3. Simpson's paradox: what is each average averaging?

Picture: two fictional machines, two part-difficulty bins, then a merged inspection tally. Every denominator remains on screen. No real manufacturing or causal claim is implied.

“Two machines each make easy and hard parts, which then get inspected. On easy parts, 9 out of 10 from machine A pass; for B, it is 72 out of 90. A leads, 90 percent to 80 percent. On hard parts, 18 out of 90 from A pass; for B, 1 out of 10 passes. A leads again, 20 percent to 10 percent.

“Now combine the parts. A has 27 passes out of 100; B has 73 out of 100. B leads overall. How? A made mostly hard parts and B made mostly easy parts. The overall rates weight the two categories differently.

“Both statements can be true: A has a higher pass rate at each difficulty, and B has a higher rate across all their parts. Which comparison answers your question depends on the context; the reversal alone does not prove what caused the results.”

Picture: use 100 equal tiles per machine; tint difficulty first, then count passes. Keep identical row scales so the grouping and unequal weights, rather than chart tricks, explain the reversal.

Check: [Stanford Encyclopedia, Simpson's Paradox](https://plato.stanford.edu/entries/paradox-simpson/). Author exact arithmetic: A 9/10 > B 72/90; A 18/90 > B 1/10; A 27/100 < B 73/100. Invented data, not an experiment.

### 4. Achilles and the tortoise: can infinitely many times add to a finite time?

Picture: a 100-metre head start, Achilles at 10 metres per second, tortoise at 1 metre per second. Mark where the tortoise was when each stage began. Show the running clock as well as distances.

“A tortoise starts one hundred metres ahead. Achilles runs ten metres per second; the tortoise moves at one. First Achilles reaches the tortoise's starting point, but the tortoise has moved on. When he reaches that new point, it has moved again. There are infinitely many of these stages. How can he ever catch it?

“The stages get shorter. The first takes ten seconds, the next one second, then a tenth, then a hundredth, and so on. Their sum approaches ten plus ten ninths, or one hundred ninths seconds: about 11.11 seconds.

“The usual constant-speed model gives the same answer directly: Achilles closes the hundred-metre gap at nine metres per second. Infinitely many positive stage times can have a finite limit. This answers the arithmetic challenge in that model; the deeper philosophical questions about motion and infinity have a longer history.”

Picture: animate stages 10, 1, 0.1, … under a clock approaching `100/9 s`; show the continuous trajectories meeting. Do not imply the infinite stages must be physically completed one after another by a special machine.

Check: [Stanford Encyclopedia, Zeno's Paradoxes](https://plato.stanford.edu/entries/paradox-zeno/). Author calculation: `10/(1−1/10)=100/9`; meeting position `10×100/9=1000/9` metres from Achilles's start.

### 5. Hilbert's hotel: can a full hotel have room?

Picture: rooms labelled 1, 2, 3, …; every displayed room has a guest, and an arrow rule covers every positive integer. Avoid depicting a final rightmost room.

“Imagine a hotel with rooms numbered one, two, three, and so on without end. Every room has one guest. A new guest arrives. Can everyone have a room?

“Assign the guest in room one to room two, the guest in room two to room three, and in general the guest in room n to room n plus one. Now assign the new guest room one. Every original guest has a distinct room. The new guest has one too, and every room is assigned: room one to the newcomer, and room k, for k at least two, to the former guest from room k minus one.

“That is impossible for a finite hotel. For an infinite set, a one-to-one matching can pair the whole set with a proper part of itself. This is a mathematical assignment, not a practical plan for moving infinitely many people.”

Check: [Institute for Advanced Study, Hospitality at the Hilbert Hotel](https://www.ias.edu/sites/default/files/documents/publications/ilspring16.pdf), PDF page 6 (printed page 6).

### Closing

“Five surprises, five questions: what does the host know, which pairs count, how are averages weighted, what happens to the total time, and can every guest be matched to a room? A paradox is often a sign that an ordinary intuition has wandered outside the conditions where it works.”

## Competition and differentiation

Observed 2026-11-01: [Infinity Insights, Paradoxes That Fool You](https://getinfinityinsights.com/courses/math-that-surprises-you/paradoxes-that-fool-you/) lists all five, explicitly including a Monty sample space, 23 birthdays and pair/complement counting, a difficulty-weighted Simpson example, a geometric-series race, and a hotel correspondence. It therefore overlaps not just the lineup but several proposed explanations. This observation is based on the course page; no full video viewing or listening is claimed. A repeated freeze-and-label device, transparent on-screen assumptions and coordinated picture design are proposals, not proven novel or more effective. Review the actual chapters and other competing videos before asserting a material distinction; revise examples or format based on what is found.

Additional actual-frame observation: [Marko Dimitrov's video at 45:30](https://www.youtube.com/watch?v=hFIREpnRC78&t=2730s) labels the Simpson chapter and shows two players with controllers beside easy and hard levels. The original working script also used video-game players; the present draft changes the fictional setting to manufactured parts and proposes equal-sized tiles to expose weighting. A changed setting alone does not establish a material distinction. This was one inspected frame with visible captions, muted; it was not a continuous viewing or listening review.

On 2026-11-01, inspected the chapter list and the English transcript shown on [Margin, Every Math Paradox Explained](https://www.youtube.com/watch?v=KJruW0Dz6BQ) for all five overlapping subjects. The 11:01 compilation devotes roughly 43–65 seconds each to Monty Hall, birthdays, Zeno, Hilbert and Simpson. Its Monty passage moves from the first pick's 1/3 chance to the host's goat reveal; its birthday passage points to more than 250 pairs without showing the complement calculation in the transcript; its Zeno passage illustrates a half-plus-quarter series; its Hilbert passage covers both one guest and an infinite bus; its Simpson passage uses differently sized piles without the specific numerical two-group example in this draft. Captions may contain recognition errors and their authorship was not established. A few visible comments also show viewers questioning the race limit, which motivates explicitly distinguishing finite stages from the meeting time. The transcript and chapter-list inspection is not continuous video viewing, listening, a representative audience study, or proof of novelty. The exact no-match calculation, matching inverse, and same-scale 100-part comparison are proposed ways to make our explanations inspectable, subject to independent review.

The channel already has an inspection-paradox replacement; the catalog has separate birthday and parked probability-paradox identities. This compilation is a new requested editorial work, not evidence those other ideas are complete or redundant.

## Next actions and open gaps

- Author's reproducible exact arithmetic and host-case checks: `five-paradoxes-check-20261101.py`; the separate hotel inverse argument is in the draft above. These are not independent mathematical review.
- Opening narration candidates: `narration_lab/third-wave-20261024/root/elevenlabs-five-paradoxes-20261101/opening-takes-receipt01.json` records one ElevenLabs UI request and both service-produced Oliver/Eleven v4 WAV outputs, complete source text, settings, sample counts and hashes. Both are retained locally, unlistened and unselected; submitted text is not proof of what was spoken. Remaining chapters have not been generated.
- [ ] Independently review the five mathematical explanations and source locations, especially Monty host rules, Simpson's causal limits and the infinity language.
- [ ] Inspect actual competing chapter videos and other high-reach treatments; record precisely what was viewed or listened to, and revise the treatment accordingly.
- [ ] Read this draft aloud and test it with a fresh reader; edit pacing and assumptions. This draft has no measured run time.
- [ ] Design and produce the five visual scenes and narration with ElevenLabs, preserving source and takes and observing existing guarded resource limits.
- [ ] Check actual final video, captions and picture transitions; obtain continuous listening, fine-sync and independent acceptance. Technical and ASR checks cannot substitute.
- [ ] Keep publication unclaimed and obtain any required release authorization before making the video public.
