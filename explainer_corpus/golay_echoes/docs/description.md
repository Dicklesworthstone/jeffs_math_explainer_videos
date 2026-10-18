<!--
Draft delivery description for golay_echoes (2026-10-18). Every number below
is a fact of docs/fact-sheet.md, computed in src/facts.rs and recomputed by
oracle.py; the bracketed tags name them, and
`python3 -I tools/description_numbers.py golay_echoes` checks each one
(docs/description_numbers.tsv lists the one cited number that is not a fact).
The chapter list carries no timestamps: they come from the final cut
(`./x golay_echoes chapters`), which does not exist yet. The four chapters are
accepted as silent drafts only, and the narration voice is pending the
owner's listening. The release package adds the repository link and trims this
to what the upload page accepts, linking this file for the rest.
-->

# Two Bad Echoes Can Make One Perfect Measurement

Send a short pattern of up and down pulses, listen for its echo, and slide a
copy of the pattern along what comes back, scoring how well it lines up at
every shift. The score is tallest where the echo really is, but it need not be
zero away from the echo: every pattern of more than one pulse leaves ghosts,
scores where nothing is. This video shows how a second pattern, chosen as a partner
to the first, leaves ghosts of the same size with the opposite sign, so that
adding the two measurements of the same still scene, with nothing else added,
leaves only the real echoes; why the two cancel; and exactly what the trick
needs, with the cases where it fails. Such partners are complementary pairs,
also called Golay pairs after Golay.

## Chapters

1. Two bad echoes
2. What the ghosts hide
3. Why they cancel
4. When it breaks

## Precisely

What the narration says in plain words, stated exactly. The numbers are a
model's: one scene, sampled one value per pulse, no measured data.

**One pattern leaves ghosts.** Pattern A is up, up, up, down [G-F1.code_a].
Scored against its own echo it gives −1, 0, 1, 4, 1, 0, −1 at shifts −3 to 3
[G-F2.a.scores, G-F2.a.ghost_shifts]: 4 where the echo is, and ghosts a
quarter of that size at shifts −3, −1, 1 and 3 [G-F2.ghost_fraction]. No
pattern of up and down pulses longer than one pulse is free of ghosts: at the
farthest shift only the two end pulses line up, so the score there is never 0
[G-X1.far_shift_sizes]. Radar engineers call these ghosts sidelobes, and the
receiver that slides and scores is a matched filter.

**A partner cancels them.** Pattern B is up, up, down, up [G-F1.code_b]. Its
scores are 1, 0, −1, 4, −1, 0, 1 [G-F2.b.scores]: ghosts of the same size at
the same shifts, with the opposite sign. Added, the two rows give 0, 0, 0, 8,
0, 0, 0 [G-F3.sum.scores]: the echo twice as tall, and no ghost at all
[G-F3.peak_ratio, G-F3.sum.largest_ghost]. That is what makes A and B a
complementary pair: their two score rows add to 2N at the echo and to zero at
every other shift, N being the number of pulses in each, here 4 [G-F1.pulses]
(Chi, Pezeshki and Calderbank's definition 1).

**Two things at once (chapter 2).** With a loud echo and a quiet one three
steps farther [G-F5.scene], pattern A's score at the quiet echo is exactly 0:
the loud echo's ghost there (−4) and the quiet echo (4) cancel
[G-F6.a.at_quiet, G-F6.a.loud_part_at_quiet, G-F6.a.quiet_part_at_quiet]. A
zero score is not an empty place. Pattern B scores 8 there [G-F7.b.at_quiet],
and the two rows added give 32 at the loud echo, 8 at the quiet one and 0
everywhere else: the scene, 8 times over [G-F8.sum.at_loud, G-F8.sum.at_quiet,
G-F8.magnification]. Sending A twice does not help: the quiet echo stays at 0
[G-F9.a_twice.at_quiet].

**Why they cancel (chapter 3).** Start from one pulse paired with itself. To
double a pair (X, Y), send X then Y as one pattern and X then −Y as the other.
At every shift, products of pulses that sit inside one half repeat in both new
patterns, and products that reach across the middle flip sign in the second,
so those cancel; what is left is twice the old pair's rows, which already
cancel. Doubling twice gives A and B, and doubling goes on to pairs of 8, 16,
32 and 64 pulses [G-F11.lengths]. Not every complementary pair is a doubling:
there is one of 10 pulses [G-F11.length10, G-F11.length10_power_of_two].

**What "perfect" means.** The sum is exactly 2N times the scene, with no
ghosts, only in the ideal case: both sends meet the same scene with the same
echo response, the receiver adds no noise, and it records one value per pulse.
With these four-pulse patterns that is 8 times the scene [G-F16.factor,
G-F16.pulses, G-F16.per_pulse]. The pair removes its patterns' own ghosts; it
is not a promise about a real receiver.

**When it breaks (chapter 4).** If the echo moves one step between the two
sends, the sum splits it into 3 and 5 and ghosts come back at 6 shifts
[G-F14.moved.step, G-F14.moved.at_echoes, G-F14.moved.ghost_count]. If the
same echo comes back in the second send with its sign reversed, the true echo
cancels to 0 and every ghost doubles [G-F15.phase.at_echo,
G-F15.phase.ghost_ratio]: the same reflector at the same distance, not a new
object and not a negative distance. In general, with no added noise, the two
rows added are 2N times the second scene, plus pattern A's row of whatever
changed between the sends, scenes and rows taken as zero past their ends
[G-F16.factor]. A change need not erase the echo: the moved echo still scores
at both its places, and the video does not claim that motion always destroys
the measurement.

**The sign change.** In radar, half a turn of phase between the two
transmissions multiplies the second echo by −1. Chi, Pezeshki and Calderbank
write that phase factor in their eq. 8, and the video uses it for nothing
more.

**Noise.** Noise the receiver picks up is scored like any recording. A stray
blip just after the echo in the first recording gets its own little row,
pattern A read backwards from where the blip sits, and nothing in the pair's
identity makes it cancel; here the peak of 8 survives [G-F17.noise.blip,
G-F17.noise.blip_row, G-F17.noise.peak]. The pair removes the patterns' own
ghosts, not the receiver's noise.

## For programmers

Sample 0 is the moment the pattern is sent. The recording holds M samples,
recording[n] for n from 0 to M − 1, and the copy is the pattern's N pulses,
copy[j] for j from 0 to N − 1; both are taken as zero outside those samples.
The score at shift k is Σₙ recording[n]·copy[n − k] over all n: the aperiodic
cross-correlation of the recording with the pattern. Shift k lines the copy's
first pulse up with sample k, so an echo returning d steps late scores its
peak at shift d. Only shifts from −(N − 1) to M − 1 can score anything; every
other shift scores 0. A recording is the scene's echoes added, each one the
pattern delayed by its step and scaled by its strength, so a scene of L
steps gives M = L + N − 1 samples, or more if the receiver's noise runs on
past the last echo. The receiver
for a complementary pair is two correlations and one addition. The video's
numbers are exact integers, computed by its Rust code and recomputed by an
independent Python script from polynomial products
(`./x golay_echoes facts-check`).

## What the video does not claim

- Where complementary pairs are used today: no deployment is claimed.
- Anything about pulse trains that tolerate Doppler, polarimetry or polyphase
  codes: they are out of scope.
- That motion or noise always destroys the measurement.

## Sources

Read for this video:

- Chi, Pezeshki and Calderbank, *Complementary Waveforms for Sidelobe
  Suppression and Radar Polarimetry* (book chapter),
  https://users.ece.cmu.edu/~yuejiec/papers/Golaybkch_final.pdf: definition 1
  (complementary pairs); eqs. 5–6 (the matched filter); eqs. 7–9 and section
  2.2 (two transmissions added, the phase factor between them, and Doppler
  between them spoiling the sum); eq. 33 (the measurement model: delay,
  scattering, additive noise).
- Wikipedia, *Complementary sequences*,
  https://en.wikipedia.org/wiki/Complementary_sequences, read through a
  fetched summary: the length-four example and its sum, and the length-10
  pair.
- M. Bell, lecture notes, Session 37, Purdue University,
  https://engineering.purdue.edu/~mrb/resources/LecturesF/Session_37.pdf,
  read as extracted text: the matched-filter outputs of complementary codes
  added, used "where there is not significant motion between measurements".

Not read: Golay's own papers, which the Chi, Pezeshki and Calderbank chapter
cites. The pairs carry his name; this description makes no other claim about
their history.

## Every number

Every score, count and length in the video is computed by the video's own
code in exact integers and recomputed by an independent script
(`./x golay_echoes facts-check`). The two patterns and their sum are pinned to
Wikipedia's length-four example, and the sum to the definition of a
complementary pair [G-F1.code_a, G-F3.sum.scores].
