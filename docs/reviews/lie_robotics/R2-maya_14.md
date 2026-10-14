# R2 review, maya_14 (verbatim)

*Saved by the producer from the reviewer's returned text, with the reviewer's scratch folder written as `<scratch>` and one private path written as `<cmaes_explainer>`; nothing else was edited.*

---

# Review R2, lie_robotics: maya_14 (Maya, 14)

I read my persona prompt, then the whole packet (brief, script, storyboard, fact sheet) in order, then `explainer_corpus/lie_robotics/docs/protected.tsv`. I read nothing else under `docs/reviews/` and edited nothing.

Everything below is in my own words as a viewer. I did not review the maths: that is Ruth's job.

## Comprehension check

I have no blockers. Chapter 10's "why it mattered" is only on trust (see M-07).

| Ch | The sentence I would say to a friend |
|---|---|
| 01 | If you tilt a book and roll it, it ends up facing a different way depending on which you do first, but sliding it doesn't care about order. A robot arm has to compute with the kind of thing that does care. |
| 02 | Two quarter turns in a row are secretly one turn of a third of a turn about a slanted line. Turns form a "group" (you can combine and undo them), and you can track a turn with nine numbers in a grid. |
| 03 | Naming an orientation with three ring angles breaks at a bad spot where two rings start doing the same job (gimbal lock, Apollo had it). That is why software stores nine numbers or four. |
| 04 | Averaging two headings by their numbers can point the wrong way across the seam. So you lay a flat ruler against the circle, do the arithmetic there, and wrap the answer back on. |
| 05 | A turn can be read as an arrow whose length is a speed. Hold the arrow for a second and the move back onto the curved space is "the exponential". The smooth group of turns is a Lie group. |
| 06 | A platform that drives forward while spinning does not end up 3 m ahead. It drives a half circle and ends 1.9 m to its left, because the flat answer has to be wrapped back on. |
| 07 | A pose can change in six ways, and a velocity is a twist. Hold a twist and you get a screw. Any move of the book is one screw. |
| 08 | Each joint is a screw about its own axis. The gripper's pose is seven of them multiplied onto the pose at zero, outermost joint first. |
| 09 | By hand, with real lengths: elbow then shoulder puts the tip 42 cm forward and 88.6 cm up. Joint one then swings it 42 cm to the side. Each screw uses the axis as it was at zero. |
| 10 | To go the other way you start with speeds. Each joint gives the gripper a twist, twists add, and the Jacobian is the grid that turns joint speeds into the gripper's twist. A joint moves the tip most when the tip is far from its axis. (Why speeds is on trust until 13.) |
| 11 | Sometimes the grid loses a direction (a singularity), like gimbal lock. Near one, you need huge joint speeds to move the lost way, and this arm's extra joint covers for a straight wrist. |
| 12 | Seven joints for six directions leaves one spare. It lets the elbow swing on a circle while the gripper holds still, like my own elbow with my palm on a desk. |
| 13 | To find joint angles, treat "pose I have versus pose I want" as one screw, which is the error. Run the Jacobian backwards, move, look again and repeat. Here that takes three steps. |
| 14 | Blending two orientation grids number by number gives a squashed non-orientation. Halving the twist on the flat piece gives true poses all the way. A straight tip path differs by 7.6 cm. |
| 15 | These moves run drones, maps, physics engines and humanoids. Two tiny turns in two orders still differ by a leftover that is itself a turn, and the rule for it is the Lie bracket. The arm multiplies its motions and adds on the flat piece. |

## Rubric

Scale is 1 to 5, 5 best. "Correctness" is whether it sounds right to me, plus the numbers I recomputed. It is not a maths review.

| Ch | Clarity | Correctness | Motivation | Pacing | Delight |
|---|---|---|---|---|---|
| 01 the book | 4 | 4 | 5 | 4 | 5 |
| 02 one turn | 3 | 5 | 4 | 4 | 4 |
| 03 three angles | 3 | 4 | 4 | 3 | 5 |
| 04 halfway | 3 | 4 | 5 | 4 | 4 |
| 05 spin | 2 | 4 | 3 | 2 | 4 |
| 06 platform | 4 | 4 | 5 | 5 | 5 |
| 07 screws | 3 | 4 | 4 | 4 | 5 |
| 08 seven screws | 3 | 3 | 4 | 3 | 4 |
| 09 by hand | 5 | 5 | 4 | 4 | 4 |
| 10 velocities add | 3 | 4 | 2 | 4 | 3 |
| 11 lost directions | 3 | 4 | 4 | 3 | 4 |
| 12 spare joint | 5 | 4 | 5 | 5 | 5 |
| 13 steps | 4 | 4 | 5 | 4 | 4 |
| 14 steady path | 4 | 4 | 4 | 3 | 4 |
| 15 everywhere | 3 | 4 | 4 | 3 | 4 |

## Findings

Fix tags: **[words]** means a spoken line must be re-spoken. **[lower third]**, **[card]**, **[picture]**, **[countdown]**, **[captions]** and **[description]** mean no re-narration is needed. "Protected" means the phrase is in `protected.tsv`, and I am not asking to change it.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| M-01 | maya | 04_2, 04_3, 04_4 | major | "Halfway between 170 and −170". Going the long way round the circle, 0 is also exactly halfway. Nobody says "the short way", so when the voice calls 180 "the true halfway" I wondered why 0 is untrue. A sharp kid shouts this at the screen. | **[words, tiny]** In 04_4, "Now measure to the second, the short way. Twenty degrees." Also **[card]**: shade the 20° arc and caption the quiz card "halfway along the shorter way round". 04_3's protected "The true halfway is a hundred and eighty" stays as is. |
| M-02 | maya | 04_3 picture, 04_4 | major | 04_4 opens "The trouble is the ruler's seam", but I never heard of a ruler. It bends round the circle on screen during 04_3 while the voice is talking about averages and "the exact opposite". I rewound. | **[lower third]** during 04_3's bend-up: "the angle ruler, −180 to 180, wrapped round the circle. The seam is where its ends meet." No new words are needed. If the owner prefers words, put one short sentence at the end of 04_3. |
| M-03 | maya | 02_1 and every later "axis" | major | "Axis" is said 24 times in the spoken lines (02_1, 03_1, 05_1, 07_3, 08_1, 10_5, 11_6…) and never explained. It is not even in the spoken vocabulary list. I know "x-axis" from graphs, not "the axis of a turn". The gold line lets me guess, but the video promised every word defined at first use. | **[lower third]** at 02_1, when the gold line first appears: "axis: the line a turn spins about, like the stick through a spinning top". Add "axis" to the vocabulary list. If re-speaking 02_1 anyway: "…a single turn, about a single line, an axis." |
| M-04 | maya | 05_1 | major | Four new ideas in four sentences: a distance read as a speed (why?), "a spin", an arrow along an axis whose length is the speed, and "the flat piece is made of such spins". I lost the thread at "Read it as a speed" and was still behind at 05_2. This is the chapter I would quit in (84 s, seven beats, three definitions). | **[words]** 05_1 is not protected. Suggest: "On the circle, the step was twenty degrees. Say it takes one second. Then it is a speed, twenty degrees a second, and that is a spin. In three dimensions, a spin is one arrow along its axis, as long as its speed." **[card]** Also show the first two rows of the four-word card ("turn: how far. spin: how fast") here, not two chapters later at 07_3. |
| M-05 | maya | 08_6, and the label "held for its angle" | major | "Hold each joint's twist for its joint angle". I can hold something for seconds, not for ninety degrees. It sounds wrong to me, and the label under the formula repeats it. | **[words]** first sentence of 08_6 only: "Carry each joint's twist through its joint angle, and you get its screw, an exponential." Ruth to confirm it is still exact. Relabel the term "joint 1's screw, through its angle". The Brockett sentence is protected and stays. |
| M-06 | maya | 10_2 against 09_6 | major | 09_6 taught me "each screw is written down once, with the arm at zero… never asked where the axis is now". Then 10_2 says "For speeds, each axis counts where it is right now." That sounds like a contradiction. I don't know why speeds differ, and "counts" is not a picture. | **[words, one clause]** "For speeds the arm has already moved, so each axis counts where it is right now." **[picture]** Reuse 09_6's grey (now) against dashed gold (at zero) pair for the elbow axis. |
| M-07 | maya | 10_1 to 13_1 (chapters 10 to 12) | major | "The answer starts with speeds." Why? I am asked the real question, then it disappears for about three minutes. I enjoyed 11 and 12 on their own, but I could not say what 10 was for until 13. | **[words, one clause]** 10_1: "The answer starts with speeds, because the robot gets there in small steps." **[picture]** Keep the pink goal on screen through 10 to 12 (already in the notes). Add a small corner tag "on the way to: which angles?" in each. |
| M-08 | maya | 11_6 | major | Five sentences, and I lost it at "with two joints at the shoulder". Joint one was "at the base" (09_4) and "the base's" (10_5), but now the shoulder has "two joints… a third". The picture also jumps from the elbow-straight pose back to the L shape without telling me. The phrase "with two joints at the shoulder, would lose a direction here" is protected. | **[picture only]** Label joints 1 to 3 as one group "shoulder: three joints" in this chapter and in 08_1. Show the readout "without joint 3: directions 5. With it: 6" while the voice says "would lose" and "covers". Flash "the L shape again" at the start of the beat. |
| M-09 | maya | 07_2 and 07_3 (lower third at 07_3) | major | At 07_3 the screen holds the four-word card and a lower third: "a twist's slide arrow is the velocity of the point at the origin…". "Point at the origin?" That is the exact moment I feel stupid and stop. "Turn, spin, twist, screw" also sound like four names for one thing. | **[lower third]** Move that text into the "for programmers" tag and leave the card alone with the picture. **[card]** Give each word a tiny picture: curl, curl with speed lines, helix, helix with speed lines. Words are unchanged. |
| M-10 | maya | 01_1, 01_2 | minor | I tried the quiz with my real book and could not repeat it. Which line is "tilt", and which way do I turn? And "the cover" first appears in 01_2 without being introduced. I can only trust "facing you" and "the wall on your right". | **[lower third]** at 01_1: "start: cover up. Tilt: far edge up. Roll: right edge down." **[description]** one line, "try it with a real book". Not protected, but "about one line on the table" is and stays. |
| M-11 | maya | 01_6 against 08_6 | minor | 01_6 says "the way out is about a hundred and fifty years old, and by the end you'll read it in one line". Then 08_6 says that line was "presented in nineteen eighty-three". That is 43 years, and I felt a number didn't fit. 1873 is never connected to anything. | **[card]** Put "began in 1873" on Lie's name card at 05_7. Make 08_6's lower third "the ideas: Lie, 1870s. The formula: Brockett, 1983". Or **[words]** 01_6: "built on ideas about a hundred and fifty years old". |
| M-12 | maya | 02_6 | minor | "One turn after another is one grid times another." I don't know how to multiply two grids, and the picture shows only the result, so I feel I am meant to. | **[lower third]** "grids multiply by a rule you can look up. We never do it by hand here." (I found no later chapter that multiplies grids by hand.) |
| M-13 | maya | 03_1 | minor | "A machine that tries" to do what? I only connected "each ring's angle is one of the three numbers" when the readouts appeared. And "Three should do": why three? | **[lower third]** label the readouts "ring angle = one of the three numbers" and add "3 ways to turn, so 3 numbers should do". |
| M-14 | maya | 03_6 | minor | "The extra ones buy a naming with no bad spot." "Buy a naming" is an odd phrase. "Or four": four what? I never hear its name. The lower third "nine give one name each, four give two (q and −q)" made me feel dumb. | **[words, small]** "…and the extra numbers give a way of naming orientations with no bad spot." "Somewhere there is a bad spot" is protected and stays. **[lower third]** cut it to its first clause. |
| M-15 | maya | 04_5, 05_1, 07_1, 07_2 | minor | "The space is curved" means the circle of headings. "In space" (05_1, 07_1, 07_2) means the 3D room, and right after Apollo I heard outer space. One word, two things. | **[words, small]** 05_1 (see M-04 above), 07_1 and 07_2: say "in three dimensions". **[picture]** Label the curved thing "the curved space of all headings", later "of all orientations". |
| M-16 | maya | 05_4 against 02_3 | minor | 02_3 says "leans equally from the book's three edges", while 05_4 says "the line pointing between its three edge arrows". I took it for the same line. The picture says "the other one" (it is the roll-first diagonal), but the voice never says so. | **[picture]** Label it "the other diagonal" and use one phrase in both places. If 05_4 is re-spoken: "…the other line pointing between its three edge arrows". |
| M-17 | maya | 05_3 | minor | "The exponential": at school that means growing fast, and I wondered what growing has to do with turning. "Is the exponential" is protected. | **[lower third]** "just this move's name, not about growing". |
| M-18 | maya | 05_6 | minor | I tune out here. The screen has sin, cos and "ω × around" (a cross product I don't know, and it is not behind the "for programmers" tag). The voice sentence itself is fine. | **[picture]** Label "ω × around: the arrow at right angles to both". Put the `p' = …` line under the "for programmers" tag too. |
| M-19 | maya | 05_7 | minor | "A group of motions that is smooth like this": smooth how? And "Lie algebra" is said once and never used again, so it is a second name I carry for nothing. | **[words]** "A group of motions that change gradually like this is a Lie group, after the Norwegian mathematician Sophus Lie." Keep "Lie algebra" on the badge only. |
| M-20 | maya | 08_5 | minor | "…when its turn comes". "Turn" is the video's word for a rotation, so I tried to picture a turn. Last joint first also feels backwards. The line is protected, so only the picture. | **[picture]** Number the order on the arm, "1st, 2nd, 3rd…", as the screws light, so "its turn" reads as its place in the queue. |
| M-21 | maya | 10_4, 10_5, 08_1 | minor | The quiz says "joint one, two or seven", the answer says "the shoulder's, the base's, joint seven's". In 10_4 joint 7 is "at the wrist", but the wrist is joint 6 in 08_1 and three joints in 11_6. I kept a lookup table in my head. | **[card]** Put the names in the quiz choices: "joint 1 (base), joint 2 (shoulder), joint 7 (wrist)". Keep the names on the arm through the chapter. |
| M-22 | maya | 11_1 (and 03_3, 07_1) | minor | "Ways to turn" (03), "ways a pose can change" (07_1), "directions" (11_1). I read "direction" as left or right, not turning. It is new here and not tied back. | **[lower third]** at 11_1: "directions = the six independent ways a pose can change". |
| M-23 | maya | 11_4 lower third | minor | "…this arm also has five; straight up, three." Five what? | **[lower third]** "…still has only five directions; straight up, only three". |
| M-24 | maya | 11_5 | minor | "With the elbow bent thirty degrees, it must turn…". The sentence before is about the gripper, so I heard "the gripper must turn". "Its limit" then also floats. | **[words, one word]** "…the elbow must turn at fifty-four degrees per second." "Its limit is seventy-five" is protected and then has a clear owner. |
| M-25 | maya | 12_3 | minor | "Your arm has seven joints too." I count shoulder, elbow, wrist = 3. The 3+1+3 is only on screen briefly. The phrase is protected. | **[picture]** Label "shoulder: ball joint = 3 hinges. Wrist = 3 hinges", and hold it a beat longer. |
| M-26 | maya | 13_2 | minor | "Subtracting angles walks into both traps." I saw only the 89° trap. Where is the seam one? | **[picture]** Add the line "170° − (−170°) = 340°" to the readout, or **[words]** "walks into the traps". |
| M-27 | maya | 13_6 | minor | "Newton's method": a name, not an explanation. The sentence is protected. | **[lower third]** "Newton's method: guess, measure the miss, correct, repeat". |
| M-28 | maya | 14_2 | minor | The quiz choice "not even an orientation" is the only odd-sounding one, so I would pick it without reasoning. I also had no way to reason to it. | **[card]** Make the choices parallel: "halfway / squashed / stretched". 14_3 then still reads correctly. |
| M-29 | maya | 14_5 | minor | The chapter ends "differ by seven point six centimeters" and I don't know whether that is a lot, or which path the robot should take. It is the weakest beat in 14. | **[lower third]** "about 3 inches. Both are true paths. Only the blend is not." If length must go, this beat is the cheapest cut. |
| M-30 | maya | 15_1 | minor | Drones, maps, a physics engine and a humanoid in about 20 s. I only got "so the body stays rigid" because the swelling book shows it. | **[picture]** Hold the swelling book through that sentence. If re-narrating, split into two beats. |
| M-31 | maya | 15_3, 15_4 | minor | "That leftover" arrives before anyone named it. "Follows a rule of its own, called the Lie bracket": I never learn what the rule is from the voice. | **[words, small]** 15_3: "They do, by less than two hundredths of a degree. Call that the leftover. And it is itself a turn, almost exactly about the one axis neither turn used." Both protected phrases are intact. **[lower third]** 15_4: "the rule: two spin arrows make a third at right angles to both". |
| M-32 | maya | 03_2, 07_1 countdowns | minor | 4 s is tight for picturing three nested rings (03_2) and for counting 3+3 (07_1). I used "pause for more time". | **[countdown]** Change both to 6 s. No re-narration is needed. |
| M-33 | maya | 03_4, 07_6, 08_6, 14_5 | minor | Pacing, with no change asked to the protected lines. Chasles, Mozzi and Brockett are three names and four dates in about ten seconds, and I zone out. 03_4 gives 70 and 85, which I cannot use. If length must go: trim 03_4's last sentence (not protected), then 14_5. Do not cut Collins (03_5): it is the one human moment in act one. | **[picture]** Let the name beats ride on lower thirds while the picture keeps moving. Nothing in 10 to 12 should be cut before M-07 is fixed. |
| M-34 | maya | 01_3, 05_7, captions | minor | The script and captions say "Lie", the voice says "Lee". On screen I would read "lie". The title card's "say Lee" covers the picture but not the SRT. | **[captions]** First occurrence as "Lie (say Lee)". |
| M-35 | maya | 09_1 | minor | The storyboard picture says "wrist to flange". "Flange" is on screen only and I don't know it. | **[picture]** Label it "to the tip". |
| M-36 | maya | description | minor | `docs/description.md` does not exist. The storyboard sends these to the description, so they are recorded nowhere: the wheel-sense table (`G-P.wheels.*`) and the wheel formula's source (06 Dev note), the "gripper's tip is the flange" note (09), and the references. I would also want "where to learn more", for example the free Modern Robotics course. | **[description]** Write `docs/description.md` before publishing. No words or picture change. |
| M-37 | maya | 01_1, 01_2, 01_4 | delight | The cold open is a puzzle I can do with a real book. The answer lands at once, and "positions add, orientations don't" is the right size of claim. | Keep. |
| M-38 | maya | 02_3, 05_5 | delight | 120, 240, 360: "three of them bring the book home" is small enough to check in my head. 05_5 tying back "each edge lands where the next one was" to the first quiz made me feel clever. | Keep. |
| M-39 | maya | 04_4, 04_5 | delight | Peeling the ruler off and laying it flat against the circle is the picture of the whole video. The corner badge then makes "the same move again" easy to follow. | Keep. |
| M-40 | maya | 06_3, 06_4 | delight | The line bending at constant length into the half circle (3 m long, 1.91 m across) made "flat answer, wrapped on" physical. | Keep. |
| M-41 | maya | 07_4, 07_5 | delight | Door, drawer, bottle cap, then the wall of 264 books each with its own screw: "every time" is surprising and fun. | Keep. |
| M-42 | maya | 09_2 to 09_6 | delight | Whole chapter: small numbers I can check (36 + 40 + 12.6 = 88.6, 42 forward, 42 to the side), and 09_6 states the point in two sentences. | Keep. |
| M-43 | maya | 11_3, 11_5, 12_2 | delight | "Can't even begin to slide straight back", "four and a half turns a second", and the palm on the desk with a 6 s countdown are the three beats I would tell a friend about. | Keep. |
| M-44 | maya | 13_6, 15_3, 15_5, "for programmers" tags | delight | "From here, three" is a real surprise. The leftover being a turn about the unused axis is the best final reveal, and 15_5 pays off the title. The "for programmers" tag tells me when it is fine to skip, which keeps me from feeling stupid. | Keep. |

## What I checked, and how

**Recomputed with a script (agree with the packet):**
- Cold-open covers: tilt then roll faces (0, −1, 0), roll then tilt faces (1, 0, 0), both a 120° turn.
- 3 × 120 = 360.
- Leftover of two turns: 120° at 90°, 1.741° at 10°, 0.01745° at 1°, which is a hundredfold per tenfold.
- Platform: radius 0.955 m, 1.91 m across, half circle 3.00 m long.
- Arm heights: 36 + 42 + 40 + 12.6 = 130.6 cm.
- L-shape tip by hand at (42, 0, 88.6), then (0, 42, 88.6) after joint one.
- Lever from the shoulder axis 67.3 cm.
- Elbow rates 54.0, 160.4, 534 and 1,602 °/s, and 1,602 ÷ 360 = 4.45 turns a second.
- Blend determinant 0.5 at halfway.
- Orientations of 13_2 are 1.41° apart.
- Swelling after 1,000 steps: 1.105 in volume and 1.051 in width.
- Gimbal bar: 34% at 70° and 9% at 85°.
- 2026 − 1873 = 153 years.

**Checked by reading:** the spoken column has no colons, semicolons, parentheses or dashes (hyphens only inside compound numbers). I read every phrase in `protected.tsv` against my findings. None of my findings is against a protected phrase, and where one sits next to a fix I said so.

**Unchecked (I did not recompute):**
- Newton's run (505, 94, 4.2, 0.03 mm), the 21.5° crossing, the 29 cm circle, the 7.6 cm bow.
- All sources, quotations and dates (Euler, Mozzi, Chasles, Brockett, Apollo thresholds, KUKA limits).
- The 192, 264, 66, 45 and 12 edge tests, and Rodrigues' formula.

**Persona notes:** the storyboard as inlined carries no notes tagged for Maya (only Ruth, Dev and "misconception" notes). I did not read R1's findings or dispositions, as instructed, so I cannot tell which earlier Maya notes were accepted, and I did not look for them in the storyboard.
