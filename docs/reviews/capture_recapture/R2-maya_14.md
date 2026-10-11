<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# Maya, 14: R2 script review of *How to Count Fish You've Never Seen*

I read my persona prompt, the whole R2 packet, my R1 review and the R1 dispositions. I read the 77 script lines one beat at a time against the storyboard's picture column. I checked that the packet's script is byte-identical to `explainer_corpus/capture_recapture/script.tsv`. I recomputed every spoken number I could, and the list is at the end. I did not edit any repository files.

## Comprehension check

| ch | What I'd tell a friend |
|---|---|
| 01 | Two catches of a thousand share a hundred fish, and somehow that says about 8,100 fish were never caught. Counting one patch and multiplying fails: 45,100, 88,000 or 2,800 depending on where you dip, because fish crowd together and a net can't tell what share of the *fish* it got. |
| 02 | The marked fish show that the second net caught one in ten of all the fish, so the lake is about ten thousand. That's marked times second catch over caught twice, "the soup guess". *(I didn't know why it was called soup until 03.)* |
| 03 | Fish stay bunched in their favourite spots, but the marks spread out so every patch is about one in ten marked. So the second catch is a fair copy of the lake, which is what "independent" means. *(I saw a straight line on the table but nobody told me what it meant.)* |
| 04 | In a 20-fish lake you can count all 4,845 netfuls of four and the chance of each number of marked fish. One marked is the biggest pile, the same one-in-four as the lake. But that goes from lake to catch, and we need catch to lake. |
| 05 | For a mystery lake (10 marked, 10 caught, 3 marked), each lake size gets a score: how likely our catch would be at that size. The top is 33, like the soup guess, and the scores add to 16⅔, so they aren't chances. *(Sixteen and two thirds percent, or what? See M2.)* |
| 06 | One mountain of chances. Cut it at one lake size and you get the netful chances, which add to one. Cut it at our three and you get the score curve, which adds to 16⅔. |
| 07 | Feller compares each lake size with the one below it. The score keeps rising while marked × second catch beats caught twice × lake size, which is the same as first-only × second-only beating caught twice × dark cell. *(I took 529 and 528 on faith.)* |
| 08 | Let the dark cell grow until 100 × dark cell = 900 × 900, which happens at 8,100. Then the table balances, the line goes straight, and the careful answer equals the soup guess. |
| 09 | Run Feller's experiment 10,000 times on a lake of exactly 10,000. Caught-twice counts bunch around 100 ± 9, and estimates land between about 8,500 and 12,000. They average a bit high because dividing by a small count pushes harder. |
| 10 | Small experiments sometimes catch zero marked fish, so you'd divide by zero. Chapman's add-one fixes that. |
| 11 | Tiny and huge lakes are basically impossible, and even the best lake gives our exact catch only 4% of the time. That's normal, like exactly 50 heads in 100 flips. |
| 12 | Keep every lake size for which our hundred would be an ordinary catch, and you get 8,400 to 12,100. Ranges built this way ring the true size 95 times in 100, and more fish caught twice makes them tighter. |
| 13 | The trick breaks when the second catch's marked share differs from the lake's. Bold fish and bait-lovers push the estimate down, while bait-avoiders and washed-off marks push it up. Newcomers mean you count the later lake, deaths don't matter, and unmixed marks make it depend on where you fish. *(The deaths part sounded wrong to me, see M1.)* |
| 14 | Laplace used births as marked fish to get France at about 28.4 million. Two code reviewers' shared bugs say about 60 in all, probably too low because easy bugs get found twice. |
| 15 | In Kosovo, lists of names gave an estimate of about 10,400 killed, and a later name-by-name count of 9,790 fell inside its range. Every example is a table with one dark cell filled in from the others. |

Every chapter has a sentence, so there are no blockers. 05, 07 and 13 are where I wobble.

## Rubric (1–5)

"Correctness" is from my seat: did the numbers check out, and did anything *sound* wrong?

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 4 |
| 02 | 3 | 5 | 4 | 4 | 5 |
| 03 | 3 | 5 | 4 | 4 | 4 |
| 04 | 4 | 5 | 4 | 4 | 5 |
| 05 | 3 | 4 | 4 | 4 | 3 |
| 06 | 3 | 5 | 3 | 4 | 4 |
| 07 | 2 | 4 | 3 | 3 | 3 |
| 08 | 4 | 4 | 4 | 4 | 5 |
| 09 | 4 | 5 | 5 | 4 | 4 |
| 10 | 3 | 4 | 4 | 4 | 4 |
| 11 | 4 | 5 | 4 | 4 | 5 |
| 12 | 4 | 5 | 4 | 4 | 4 |
| 13 | 3 | 3 | 5 | 3 | 4 |
| 14 | 4 | 5 | 5 | 4 | 4 |
| 15 | 4 | 5 | 5 | 4 | 5 |

Pacing: at 150 words a minute the speech alone runs about 13.6 minutes. 04 (about 76 s of speech plus the pour) and 13 (about 77 s plus two predictions, an act card and the 1.5 s silence) are already close to 85 s, so the fixes below for 13 come with a trim.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| M1 | maya_14 | 13_6, 13_8 | major | At 13_6, 2,000 newcomers mean "you've counted the lake as it is at the second catch", 12,000. At 13_8, three fish in ten die and "the estimate doesn't move", so the needle stays at 10,000. But at the second catch only 7,000 fish are alive. So this time it counts the lake from *marking* day? "Doesn't move" sounds like good news, yet the needle is 3,000 away from the fish actually in the lake. Back to back, the two beats sound like a contradiction, and this is the "huh" beat everyone will remember. (Ruth should check whether the 13_1 lower-third, "the marked share of the lake you mean", covers this case: after the deaths, the living lake is still 10% marked, but the estimate is 10,000.) | 13_8: "The estimate doesn't move. Death doesn't care about marks, so you still count the lake as it was when you marked it." Give the gauge a second mark, "lake at the second catch: 7,000", just like 13_6's "12,000". To stay under 85 s, cut "Then it's a fair copy." from 13_1. |
| M2 | maya_14 | 05_4 → 05_7, 06_3, 06_4 | major | 05_4 gives me the heights as percents: eight, thirty-two, sixteen. Then the key line at 05_7 says they add to "sixteen and two thirds, not one". Sixteen and two thirds *percent* is less than one hundred percent, so why is that too much? The voice never tells me that "one" means a hundred percent, so the chapter's main point (scores aren't chances) depends on a unit switch I can't hear. | 05_7: "...and you get about sixteen and two thirds. Real chances would add up to one, a hundred percent." Then "The lake has one size..." as now. Next to the running sum, show "16.67 = 1,667%". 06_3 and 06_4 then read correctly. |
| M3 | maya_14 | 07_2–07_4, 07_6 | major | 07_2 says the step from 32 to 33 "comes down to two numbers, five hundred twenty-nine against five hundred twenty-eight". I don't know where they come from, so I'm just trusting it. Then 07_3 shows `(N−M)(N−n) − N(N−M−n+k)` with no words under the letters, and capital N and small n are all over one line. This is where I'd feel stupid. 07_6's "One, same as before" points at a "one" I never heard said aloud. | On screen at 07_2: 529 = 23 × 23 and 528 = 33 × 16, with word labels under the factors: "unmarked fish" (33 − 10), "fish the second net missed" (33 − 10), "lake size", "dark cell" (16). Voice: "twenty-three times twenty-three, against thirty-three times sixteen". At 07_3, keep those same word labels under the letters so the letters are just a rewrite of what I saw. At 07_4, add "In our small lake, ten times ten, minus three times thirty-three. One." I checked: 23·23 = 529, 33·16 = 528, 100 − 99 = 1, 49 − 48 = 1. |
| M4 | maya_14 | 01_3, 03_7–03_8 | major | The table turns into the mosaic at 01_3 without a word about it. At 03_7 it comes back with a straight divider while I hear "Mixing is what makes the table fair". What line? Fair how? Nobody reads the picture to me until 08_2, five chapters later. So 03_8's "that's what independent means" is pinned to a line I can't read. | Add one sentence at 03_7 after the first one: "In both columns, the second net took the same one in ten, so the line across the table runs straight." I checked: 100 of 1,000 and 900 of 9,000 are both one in ten. Then 08_2 becomes a callback, not the first explanation. |
| M5 | maya_14 | 02_3, 02_6 | minor | "Read it one way first." Read what? Nothing called "it" has been said. Then 02_6, "the same fact the other way", stops at "about one in ten in the lake is" and never reaches the answer. That second reading is exactly the one that turns into 02_7's "marked times second catch, over caught twice", so the formula still feels like a jump. | 02_3: "Read the catch one way first." End 02_6 with "We marked a thousand, so the lake holds about ten times that." |
| M6 | maya_14 | 02_7 → 03_1 | minor | "Call it the soup guess" comes before any soup is mentioned. For about ten seconds I'd think I missed a joke. | Keep 02_7, and end 03_1 with "Our second catch is the spoonful." That ties the name to the picture of the ladle-sized net. |
| M7 | maya_14 | 01_2–01_3 → 07_5 | minor | "The dark cell" is the video's main word, but it is first *spoken* at 07_5, in a sentence with three other cell names. 08_1 then relies on it. | 01_3: "This dark cell holds about eight thousand one hundred of them." |
| M8 | maya_14 | 13_3 | minor | "Bold fish get caught twice more than their share" will be heard as "caught *two times* more than their share". Then "so the dark cell shrinks" doesn't follow. | "Bold fish land in the caught-twice cell more than their share, so the dark cell shrinks." 14_6 already says "caught-twice cell". |
| M9 | maya_14 | 13_9 | minor | "The release site" is a new name. In 03_2–03_3 the place was "the dock". Two names make me think there are two places. | "Fish near the dock, and you'd say sixty-two fifty." |
| M10 | maya_14 | 08_2 | minor | "While the unmarked side's green share is bigger, the score keeps climbing." Bigger than what? Also, the column was labelled "not caught first" at 01_2, and now it's "the unmarked side". | "While the green share in the unmarked column is bigger than in the marked column, the score keeps climbing." Label the 01_2 columns "marked (first catch)" and "unmarked" so the words and the labels match. |
| M11 | maya_14 | 07_2, 07_6 | minor | "Our small lake" and "the small lake" mean 05's lake (10, 10, 3). But 04's lake was the one we "shrank" to twenty fish. I'd think of the 20-fish lake first, and 32 and 33 don't fit it. | At 05_1: "A new lake, and we don't know its size. Call it the mystery lake." At 07_2 and 07_6, say "In the mystery lake". |
| M12 | maya_14 | 08_4 | minor | "Rounded down to a whole fish" comes right after Feller's 10,000, which is already whole, so what got rounded? Also, "the careful argument" is a new name for the top score, and 12_1 then has a "careful range". | "So the top score and the soup guess give the same answer, rounded down to a whole fish, like thirty-three from thirty-three and a third." |
| M13 | maya_14 | 08_5 | minor | Why do 9,999 and 10,000 tie, and not 10,000 and 10,001? "The line is perfectly straight at ten thousand, so..." doesn't tell me. | "At Feller's numbers the line goes exactly straight at ten thousand, so the step up from nine thousand nine hundred ninety-nine changes nothing, and the two tie for the top score. Feller just says ten thousand." I checked: Mn − kN = +100 / 0 / −100 at 9,999 / 10,000 / 10,001. |
| M14 | maya_14 | 05_7 (picture) | minor | During the key line, with the music dropping out, I'm also watching a running sum and a squeezing axis, and reading Feller's quote with a bracket and "6000" in it, next to a lake of 17 to 60. I'd miss either the words or the card. | Bring the Feller card in only after the last word, during the hold, or move it to 11_1, where the lake is Feller's and 6,000 is a sensible size. |
| M15 | maya_14 | 06_4 | minor | "Same mountains, two questions." I heard two cuts but never the two questions, and they are the chapter's point. | "Same mountains, two questions. What catch will this lake give? Which lake gave our catch?" (about 3 s; 06 is about 21 s of speech against its 40 s.) |
| M16 | maya_14 | 06_1 (axis label) | minor | The axis says "marked fish caught". That's a third name for caught twice, after "caught twice" and "three were marked". | Label the axis "caught twice", as the 09_2 histogram does. |
| M17 | maya_14 | 09_6 | minor | "Two lines of code. Always check a simulation against something you can compute exactly." Did it match? The beat never says. I tune out for 6 s and get no result. | "Two lines of code. The exact chances, from the same rule as the netfuls, match the pile. Always check..." |
| M18 | maya_14 | 10_3 | minor | I hear "nearly exact on average" while the gauge shows Chapman at −6.5%, so I'd think −6.5% *is* "nearly exact". Nobody says this experiment expects only 2.5 caught twice, below the "four or more". (Re-raised from R1 m16. It's still how I'd hear it.) | Before the last clause, add "Here we only expect two or three caught twice, so Chapman still runs about six and a half percent low." |
| M19 | maya_14 | 11_2 (picture) | minor | The scrolling zeros are great, but the marker says "a float64 rounds to zero past here". I don't know what a float64 is, and it's in the main picture, not a lower-third. | Marker: "a computer's everyday numbers round to zero past here". Keep "float64" in D15's lower-third. |
| M20 | maya_14 | 12_1 | minor | "Whether a catch of a hundred would land in..." Every catch so far has been a thousand fish, so "a catch of a hundred" sounds like a net of 100 fish. | "...ask whether a hundred caught twice would land in that lake's middle ninety-five percent." |
| M21 | maya_14 | 12_4–12_5 | minor | The answer is "more fish caught twice". But I can't choose how many get caught twice, so what do I actually do? | "Mostly, more fish caught twice, which means marking more and catching more." |
| M22 | maya_14 | 14_5 | minor | "But twelve is a small number caught twice." Aloud it sounds garbled, like the number twelve got caught twice. | "But only twelve were caught twice, and that's not many." |
| M23 | maya_14 | 15_1 | minor | "The names shared between lists estimated about ten thousand four hundred killed." Names can't estimate anything. In a sentence this serious I'd rewind. | "From the names shared between lists, researchers estimated about ten thousand four hundred killed." |
| d1 | maya_14 | 01_6–01_8 | delight | "Not even close. In the weeds, eighty-eight thousand. In open water, two thousand eight hundred." Then "a hundredth of the water... what share of the fish". The R1 contradiction is gone, and 02_5 pays it off. | Keep. |
| d2 | maya_14 | 02_4–02_5 | delight | "A thousand is one tenth of how many?" I say ten thousand before the voice does. "The marked fish measured our net for us" is the sentence I'd repeat to a friend. | Keep. |
| d3 | maya_14 | 03_6 | delight | Fifty with five marked, ten with one: both one in ten, both checkable in my head, and it proves "evenly among them" with real fish. | Keep. |
| d4 | maya_14 | 04_4–04_7 | delight | "Which pile is biggest?" I reason one in four and I'm right, then 4,845 netfuls pour in. "Like shirts and pants" makes 10 × 105 obvious. | Keep. |
| d5 | maya_14 | 08_1–08_3 | delight | I can predict where the climb stops (8,100, the number from the opening). Hearing 810,000 = 810,000 while the motif resolves is the payoff of the whole video. | Keep. |
| d6 | maya_14 | 09_4 | delight | "Ten too few pushes harder than ten too many." 11,111 against 9,091 makes the lopsidedness obvious, and 14_5's lopsided 47–94 makes sense later because of it. | Keep. |
| d7 | maya_14 | 11_2–11_5 | delight | 430 zeros, then the coins answer "is 4% a problem?" with "50 heads is only 8%, yet it's the most likely". | Keep (with M19's label). |
| d8 | maya_14 | 12_3 | delight | "There the lake was fixed and the catch moved. Here the catch is fixed and the lake moves." This is the clearest sentence in the video, and it fixes my R1 "didn't we just get this range?" | Keep. |
| d9 | maya_14 | 14_4–14_6 | delight | I can do 30 × 24 ÷ 12 = 60, found 42, so 18 hiding, before the video says it. The bold-fish callback makes "probably too low" feel earned. | Keep. |
| d10 | maya_14 | 15_2–15_3 | delight | "Each is a table with one cell nobody can see, filled in by the cells we can." I'd feel like I understood the whole video in one line. | Keep. |

## How I checked

- **Recomputed in a scratch Python run with `math.comb` and exact `Fraction`. All of these match the script:**
  - **01–03:** 451 × 100 = 45,100; 88,000 and 2,800 (31.4× apart); 1,000 × 1,000 / 100 = 10,000; 10,000 − 1,900 = 8,100; 5/50 = 1/10.
  - **04:** 20·19·18·17 = 116,280; ÷ 24 = 4,845 = C(20,4). The piles are 1,365 / 2,275 / 1,050 / 150 / 5, which is 28.2 / 47.0 / 21.7 / 3.1 / 0.1%, so the spoken 28 / 47 / 22 / 3 / under 1 is right. C(5,2) = 10, C(15,2) = 105, and 10 × 105 = 1,050.
  - **05–06:** 100/3 = 33.3; the smallest lake is 17; L(20) = 7.8%, L(33) = 31.8%, L(60) = 15.9%; the best size is 33; the sum is 16.67 (partial sum to 60 is 9.89); a fixed-size slice sums to exactly 1.
  - **07–08:** 529/528 and 576/578; 100 − 99 = 1 and 100 − 102 = −2; 49 − 48 = 1; 100 × 8,100 = 900 × 900 = 810,000; L(9,999) = L(10,000) exactly; Mn − kN = +100 / 0 / −100.
  - **09–10:** the SD of k is 9.0; 10⁶/90 = 11,111 and 10⁶/110 = 9,091; P(k = 0) = 7.20%, which is 1 in 13.9; Chapman at k = 0 gives 2,600.
  - **11–12:** P(1,900) = 5.35 × 10⁻⁴³⁰; a million fish means about 1 marked fish expected; L(10,000) = 4.43%; the coins give 7.96%. The exact 95% range by test inversion is [8,400, 12,073]; the endpoints are −16.0% / +20.7%; the width ratios are 0.444 and 0.435; 447 misses out of 10,000 is 95.5% (at least 95%).
  - **13:** marks lost gives 10⁶/80 = 12,500. The two-halves model gives 160 or 40 caught twice, which is 6,250 and 25,000.
  - **14–15:** 2,037,615 ÷ 71,866⅓ = 28.35, × 1,000,000 = 28.35 million ("28.4"); 30·24/12 = 60, 42 found, 18 hiding; the exact bug range is [47, 94]; 9,790 is inside 9,002–12,122.
- **Unchecked (taken from the fact sheet only):**
  - the simulated medians 7,750, 7,140, 14,490 and 12,120 (my rough expected-value check for the bold fish gives about 7,500, the same direction);
  - the 09_5 mean of 10,080;
  - the 97 rings out of 100, and the 447;
  - chapter 03's 66% and the inset counts;
  - Chapman's −6.5% and the plain guess's +24%;
  - the recorded Laplace inputs, the Kosovo figures and the odds of 300,000 to 1.
