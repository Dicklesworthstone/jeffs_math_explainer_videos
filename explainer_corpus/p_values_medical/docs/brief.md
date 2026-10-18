# Brief — Why P-Values Confuse Even Doctors

Video `p_values_medical` (catalog revision `v1-michael-20261017`; first
chapter task mx-video-program-program-87do.7.1). Values I chose are marked
`(default)`. The storyboard (`docs/storyboard.md`) is the source of the words
and pictures; `docs/fact-sheet.md` holds every number and source.

| Field | Value |
|---|---|
| Topic (owner's direction) | The doctors' survey, a medical diagnosis, and why the reversal of a conditional fools smart people; then what a p-value calculates (slate, research-20261016). |
| Working title | **Why P-Values Confuse Even Doctors** (the slate's working title). |
| One-sentence takeaway | Before trusting a percent, find the group it is a share of: a test's false-alarm rate counts people without the disease, the answer a patient wants counts people with a positive result, and a p-value counts the assignments of an imaginary world where the treatment did nothing. |
| The hook | A survey published in 2007 (data collected February–July 2006): 88% of 277 internal-medicine residents in Connecticut felt at least fairly confident about p-values, about 59% picked the keyed answer, and a later paper argued the key itself is wrong. Then the honest failure on honest numbers: a positive result from a test that falsely alarms 5% of people without the disease feels like 95% certainty; counted, it is about 15%. |
| Cold open | The survey as dots and two bars, ≤ 20 s (01_1, 01_2), then the title. |
| The pictures | Act I: an icon array of 10,000 people (rows of 100), whose positives fly into one block of 13 rows of 45, two of them gold: the PPV as rows. Act II: the null world as a histogram of all 2^20 label assignments grown from a few shown ones, the tail lit and counted. The same ink outline marks "the group" in both. |
| Worked examples | Clinic A (1 in 100, catches 9 in 10, false alarm 1 in 20): 90 and 495 positives, 2/13 ≈ 15%. Clinic B (1 in 10): 900 and 450, 2/3. Study: 20 pairs, 15 favour treatment, 21,700 of 1,048,576 assignments at least as favourable, p ≈ 0.0207; 14 pairs give 0.0577. |
| Spoken vocabulary | storyboard §Vocabulary: the disease, positive, false alarm, people without the disease, catches, share, group; p-value defined only in act II. |
| Notation (screen) | fractions as counts (90 / 585) before any symbol; P(A \| B) only in the "precisely:" lower-third (default); no Greek letters. |
| What the script must get right | (1) The survey is a regional sample of residents; most picked the keyed answer; the dispute is about the key. (2) Every clinic number is a model and says so. (3) The false-alarm rate and the PPV are different shares; neither is a p-value (01_11 lower-third, 02 bridge card). (4) The p-value is one-sided, predeclared, counts the observation and everything more extreme, and assumes the sharp null and the random assignment; it is not the chance the treatment does nothing. (5) Optional stopping stays out of the core. |
| Colour roles | storyboard §3: disease gold, no disease blue, a negative result dark, survey teal, the group an ink outline, a wrong reading red, the title and the think ring hero gold. |
| Audience | Maya, Dev and Ruth (plan §1.2). |
| Length target | 6–8 min, six chapters ≤ 85 s (default); chapter 01 estimates 75 s (`tools/storyboard.py`). |
| Format | 3840×2160 at 60 fps, the corpus's delivery spec (default); this task stops at code: central build, drafts and review belong to the orchestrator. |
| Sound | `src/score.rs` (default, unauditioned): F major, 76 BPM, a pad bed, felt piano, vibraphone; the motif 5̂ 6̂ 5̂ 2̂ returns unchanged over moving chords and ends open. |
| Voice | FrankenTTS `michael` with his approved English accent (owner, 2026-10-17); the new sample's listening verdict is pending and not inferred. Line gaps `Thought`, as the corpus's recent videos (default). |
| Pronunciations to probe before narration | Connecticut (the silent middle c), p-values ("pee values"), Gao if ever spoken (on screen only now). No lexicon is written yet: probing needs synthesis, which this code-first wave does not run. |
| Oracle | `oracle.py`, standard library only: exact fractions, people listed one by one, the 2^20 assignments listed by `itertools.product`. |
| franken_manim | the workspace pin (root Cargo.toml). |
| Out of scope | Optional stopping (a possible sequel); clinical recommendations; S2's calibrated p-values. |
