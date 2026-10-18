# Storyboard — Why P-Values Confuse Even Doctors

The hand-edited source of this video's words and pictures (first chapter
written 2026-10-17 for task mx-video-program-program-87do.7.1).
`python3 -I tools/storyboard.py p_values_medical` checks it (gate G2) and
writes `script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
research: `docs/video-program/research-20261016/evidence/pvalues-medical-opening.md`.

Words are spoken exactly as written: no colons, semicolons, dashes or
parentheses in them (voice rules, plan §5.4). `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds before the next beat answers it.

## 2 The arc

The owner asked for the doctors' survey, a medical diagnosis, and why the
reversal fools smart people. The spine is one move, made three times:
**before trusting a percent, find the group it is a share of.** A
diagnosis turns it on people (chapters 01–02); a treatment study turns it
on an imaginary world of label assignments (03–05); the end turns it back
on the survey's question (06).

| Act | Chapters | Spine |
|---|---|---|
| I. A positive result | 01 five percent of what, 02 the same test in another clinic | the survey; a test that falsely alarms five percent of people without the disease; counted out of ten thousand people, only about fifteen percent of positives have it; the same test where the disease is ten times as common gives two in three; the bridge card: a false-alarm rate is not a positive predictive value, and neither is a p-value |
| II. A treatment study | 03 the imaginary world, 04 count the whole tail, 05 what two percent does not say | 20 pairs, a coin in each pair, 15 pairs favour the treatment; if the treatment changed no one, only the labels move: all 2^20 assignments, equally likely; count every assignment at least as favourable: 21,700 of 1,048,576, about two percent; the wrong tails and the wrong direction as different numbers; what p does not measure (the size of the benefit, the chance the treatment works) |
| III. Back to the survey | 06 the question again | the keyed answer rewritten with its group and its assumptions named; what evidence a patient still needs |

One picture runs through act I: the icon array of ten thousand people, rows
of a hundred, regrouped into the block of positives (13 rows of 45, two of
them with the disease). Act II's picture is the null world as a histogram of
all 2^20 assignments that grows from a handful of shown assignments, its
tail lit; the same "which group?" highlight marks the denominator in both
acts.

### The exact continuation after chapter 01

Chapter 01 ends on the regrouped block and the two fractions with their
denominators highlighted (495 out of 9,900; 90 out of 585) and the lower-third
"neither number is a p-value". Chapter 02 opens on that picture as it rests:

- **02_the_same_test**: built (its beats are below). The clinic's mix
  changes, the test does not; both clinics side by side; the bridge card on
  a cleared stage; it ends on the study's 20 pairs, no result shown.
- **03_the_imaginary_world** (~80 s): opens on chapter 02's 20 pairs of
  patients matched before treatment, a coin in each pair giving one of them
  the treatment. The sharp null on screen as words first: "if the treatment
  changed no one's recovery"; only then the observed result, 15 pairs
  favouring the treatment (M-T1.observed), so no result is shown before its
  null model. The recovery times
  stay; the labels swap; a handful of the 2^20 assignments shown one by one,
  each with its count of favouring pairs; then the histogram of every
  assignment (M-T1.null.counts) grows. ⟨predict⟩ "How often would chance
  alone give fifteen or more?"
- **04_count_the_whole_tail** (~75 s): the bars 15 to 20 light and are
  counted: 15,504 + 4,845 + 1,140 + 190 + 20 + 1 = 21,700 of 1,048,576
  (M-T1.tail.at_least, M-T1.assignments), about two percent (M-T1.p_10dp);
  only then the name "p-value" and the formula as the count it is. The
  wrong tails, struck through, each a different number: exactly 15 (1.48%,
  M-X1.exactly_percent_2dp), more than 15 (0.59%), the other direction, and
  the two-sided question (4.14%) labelled as a different question.
- **05_what_two_percent_does_not_say** (~75 s): not the chance the
  treatment does nothing, not the chance the study is wrong; 14 pairs gives
  5.77% (M-T2.p_10dp), a different amount of evidence, not a different
  truth; the sizes of the differences never entered the count, so the
  benefit in days is a separate question (the normal-model inset of the
  research brief, if it fits).
- **06_the_question_again** (~60 s): the survey's keyed answer beside a
  version that names its group and assumptions; names to search; receipts.

Optional stopping stays out of this video's core (research brief); one
closing sentence may point to a sequel.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| has the disease (a person in the icon array, a row of the block) | gold `#F0AC5F` |
| does not have the disease | blue `#58C4DD` |
| a negative result (or none shown yet): the same hue, dark (gold `#7A5A2C`, blue `#23425A`), so result is brightness and disease is hue | dark gold, dark blue |
| a person before anything about them is known | grey `#6B7280` |
| the place of a person who has left the array for the block of positives | near-background `#1C222C` |
| the survey's measured numbers (bars, answering residents) | teal `#5CD0B3` |
| the group a percent is a share of (its denominator): outlines and glows only, never a fill | `ink` |
| a wrong reading (the tempting 95%, a disputed answer key) | `warn` red `#FC6255` |
| the prediction pause, the title | `hero` gold `#FFD866` |

Colour is never the only cue: the disease is also position (the array's top
row, the block's top two rows) and every group carries its words.

## Vocabulary (one word per idea)

**the disease** (never "illness", "condition" or "sick"); **positive**,
**test positive** and **a positive result** (the result, never "a
diagnosis"); **false alarm** (a positive result for someone without the
disease; never "false positive" in the narration, which names it once on
screen); **people without the disease** (never "healthy"); **catches** (the
test finds someone with the disease); **share** (a part of a group, the
word for what a percent is); **group** (the people a share is taken of; the
word for the denominator, which the screen calls "the bottom of the
fraction" once); **model** on screen for every synthetic clinic number
(plan rule 25); **p-value** (named in the survey, defined only in act II).
The survey's people are **residents** (doctors in training); "doctors" only in
the title.

Numbers: rates are said "five percent" or "one in a hundred" as the screen
writes them; counts in full ("four hundred ninety-five"); the survey's 58.8
is said "about fifty-nine percent" and shown as printed.

## Misconceptions this chapter defuses

| Wrong belief at this point | Beat that defuses it |
|---|---|
| "a five percent false-alarm rate means a positive is ninety-five percent reliable" | 01_5 to 01_10: count the positives |
| "the survey shows most doctors picked a wrong answer" | 01_2: most picked the keyed answer; the dispute is about the key |
| "the survey sampled all doctors" | 01_1 and its tag: 277 of 367 internal-medicine residents, 11 programs, Connecticut; collected in 2006, published in 2007 |
| "fifteen percent is a fact about the test" | 01_11 and the continuation: it is a fact about this test in this clinic (02 changes only the clinic) |

## 01_five_percent_of_what: Five percent of what? (~82 s at the kit's scratch rate)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A survey published in two thousand seven tested two hundred seventy-seven internal medicine residents in Connecticut. Eighty-eight percent felt at least fairly confident about p-values. *(M-C1.survey.published_year, .respondents, .confident_percent; the data were collected February to July 2006, so the year said is the publication's)* | Cold open. Left: one dot for each of the 367 residents of the surveyed programs, all faint; the 277 who answered light teal, one by one. Tag above, two lines: "published in JAMA, 2007 · data collected February to July 2006" and "11 internal-medicine residency programs, Connecticut"; under the dots "277 of 367 residents answered (75.5%)". Right: a teal bar grows to 88%, "fair to complete confidence about p-values (as reported)" | lagged reveal; proportion bar | |
| 01_2 | Only about fifty-nine percent picked the answer marked correct. A later paper argued even that answer is wrong. *(M-C1.survey.keyed_percent_spoken; Gao 2020)* | A second teal bar grows to 58.8%, "picked the answer marked correct". Under it a quote card, verbatim: the keyed choice c for p > 0.05, "The probability is greater than 1 in 20 that a difference this large could occur by chance alone." with "as reprinted by Gao 2020". At "even that answer" a red tag lands on the card: "Gao 2020: none of the four choices is correct" | proportion bar; quote card; highlight | |
| 01_3 | *(title)* | The survey crossfades into the title card: **Why P-Values Confuse Even Doctors** | title card | listen 2 s (the motif: left open) |
| 01_4 | Your test comes back positive. It gives a false alarm to five percent of people without the disease. *(M-F1.false_alarm_percent)* | A result card "POSITIVE"; beside it the lab's rule as a card: "false alarm: 5% of people without the disease" | card | |
| 01_5 | ⟨predict⟩ So is there a ninety-five percent chance you have it? *(M-F1.tempting_percent)* | "95%?" in large type by the cards; the think ring drains with "pause for more time" | predict pause | |
| 01_6 | Count people instead. Ten thousand of them, and one in a hundred has the disease. *(M-F1.people, .prevalence.one_in)* | The cards leave and the "95%?" shrinks to the top right corner. An icon array of 10,000 grey people fades in (100 rows of 100, "10,000 people (model)"); the top row turns dark gold as the ledger beside it reads "100 · have the disease, 1 in 100" | grid of row strips; tracker recolour | |
| 01_7 | That's one row of a hundred. Say the test catches ninety. *(M-F1.with, .true_pos)* | An outline marks the top row. Along it 90 cells brighten one after another (each a tile of its own) as the ledger's count runs to 90, "of them test positive"; the last 10 stay dark gold | live count; tracker | |
| 01_8 | The other nine thousand nine hundred don't have it. Five percent of them still test positive. Four hundred ninety-five false alarms. *(M-F1.without, .false_alarm_percent, .false_pos)* | The other 99 rows turn dark blue ("9,900 · don't have the disease"); then a stripe five cells wide lights bright blue down all 99 rows while a count runs to 495 ("test positive anyway: false alarms") | live count; highlight stripe | |
| 01_9 | Now gather everyone who tested positive. | The 585 bright cells lift off the array and fly right, growing, into a block of 13 rows of 45: the 90 gold on top (2 rows), the 495 blue below (11 rows). Their places in the array go faint | morph that keeps identity; lagged | |
| 01_10 | Ninety have the disease, and four hundred ninety-five don't. So only about fifteen percent of the positives have it. *(M-F1.true_pos, .false_pos, .ppv_percent)* | Words on the block: "90 with the disease" by the gold rows, "495 without" by the blue; the fraction 90 / 585 ≈ 15% beneath ("2 rows of 13"); the small "95%?" is struck through in red | fraction; highlight | |
| 01_11 | Five percent was a share of people without the disease. Your question is a share of people who tested positive. *(M-F1.false_alarm_percent; M-F1.ppv.exact)* | Two fractions side by side, each with its group: "495 / 9,900 = 5%: positive, among people without the disease" (the array's 99 blue rows and the block's 11 blue rows outlined in ink) and "90 / 585 ≈ 15%: the disease, among people who tested positive" (the block outlined). Lower-third, precisely: "in this model clinic, P(positive | no disease) = 1/20 and P(disease | positive) = 2/13 ≈ 15.4%. Neither number is a p-value." | fractions; glow outlines; lower-third | hold |

## 02_the_same_test: The same test in another clinic (~81 s at the kit's scratch rate)

Opens on chapter 01's last picture as it rests (the array with its vacated
positives, the block of 13 rows of 45, the two fractions, the struck "95%?").
Assumption on screen throughout: the test's two rates are the same in both
clinics (a model; real tests can perform differently in different
populations).

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Take the same test to a clinic where one person in ten has the disease. *(M-F2.prevalence.one_in)* | Chapter 01's picture gathers into the left half as clinic A: its array redrawn small, its block of positives scaled into place beside it, both fractions moved under it; the struck "95%?" and the word labels leave. Headers: "clinic A · 1 in 100 has the disease" and "clinic B · 1 in 10 has the disease". Clinic B's array of 10,000 grey people fades in on the right; at "one person in ten" its top ten rows turn dark gold | morph that keeps identity (the block scales, the fractions slide); grid of row strips; tracker recolour | |
| 02_2 | The test hasn't changed. It still catches nine in ten, and still gives five percent of the rest a false alarm. *(M-F2.sensitivity.exact, .false_alarm_percent)* | A card across the middle: "the same test in both clinics (assumed): catches 9 in 10 with the disease · a false alarm for 5% of people without it". Clinic A's ledger is already under its array: 100, 90, 9,900, 495 | card | |
| 02_3 | ⟨predict⟩ So will a positive result mean more here, or less? | A "?" where clinic B's block will be; the think ring drains with "pause for more time" | predict pause | |
| 02_4 | Count again. A thousand have the disease, and nine hundred of them test positive. *(M-F2.with, .true_pos)* | Clinic B's ledger begins: "1,000 · have the disease"; along its nine full gold rows 900 tiles brighten one after another as the count runs to 900, "of them test positive"; the tenth gold row stays dark | live count; lagged reveal | |
| 02_5 | Of the other nine thousand, five percent is four hundred fifty false alarms. *(M-F2.without, .false_alarm_percent, .false_pos)* | Clinic B's other 90 rows turn dark blue ("9,000 · don't have the disease"); a stripe five cells wide lights down them as the count runs to 450 ("false alarms"); then clinic B's false-alarm fraction 450 / 9,000 = 5% appears beside clinic A's 495 / 9,900 = 5% | live count; fraction | |
| 02_6 | Gather the positives. Nine hundred of the thirteen hundred fifty have the disease. That's two in three. *(M-F2.true_pos, .positives, .ppv.exact)* | Clinic B's 1,350 tiles fly into its block, 45 wide like clinic A's: 30 rows, the 20 gold rows on top; "20 rows of 30" beneath; at "two in three" the fraction 900 / 1,350 ≈ 67% appears under clinic B | morph that keeps identity; fraction | |
| 02_7 | So here a positive result means about sixty-seven percent, not fifteen. *(M-F2.ppv_percent; M-F1.ppv_percent)* | Both blocks outlined in ink, the groups the two answers are shares of; the two fractions 90 / 585 ≈ 15% and 900 / 1,350 ≈ 67% glow | glow outlines | |
| 02_8 | Why? The false alarms barely moved, but the test caught ten times as many people with the disease. *(M-F2.caught_vs_a)* | At "false alarms" both ledgers' false-alarm lines are outlined (495 and 450) with both 5% fractions; at "ten times" both "test positive" lines are outlined (90 and 900) and a chip "× 10" lands between them | highlight; chip | |
| 02_9 | So keep the two apart. A false alarm rate is not the chance a positive result is right. And neither one is a p-value. | The stage clears. Two cards side by side: "false-alarm rate · P(positive \| no disease) = 1/20 · the same in both clinics" in blue, and "positive predictive value · P(disease \| positive) = 2/13 in clinic A, 2/3 in clinic B · moves with the clinic" in gold, a "≠" between them; at "neither one" a third line beneath: "Neither is a p-value." | cards; TeX | |
| 02_10 | A p-value answers a question about a study instead. To find it, we first build an imaginary world where the treatment does nothing. *(M-T1.pairs on screen; no result is shown)* | The cards leave. Twenty pairs of patients appear one pair after another, two plain discs in each pair's box, with "a synthetic study: 20 pairs of patients, a coin in each pair picks who is treated (model)"; at "imaginary world" a caption: "imagine the treatment changed no one's recovery". No pair is marked as favouring anything | lagged reveal; caption | hold |
