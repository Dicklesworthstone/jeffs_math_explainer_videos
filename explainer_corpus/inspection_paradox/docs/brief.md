# Brief — Why Your Bus Wait Is Longer Than You Expect

Values I chose without the owner are marked `(default)`.

| Field | Value |
|---|---|
| Topic (owner's words) | "Why Your Bus Is Always Late — Even When the Timetable Is Right" (third wave, Bead `mx-video-program-program-87do.13.1`). Title changed by root (Mail 6766) to "Why You Wait Longer Than the Average", with no "always late" claim, from root's packaging draft (`docs/video-program/third-wave-20261024/packaging-drafts.md`). That title was false at the film's own numbers, since 4:20 is shorter than the 6-minute average gap (WildBass's review, M4). Root then selected the working title "Why Your Bus Wait Is Longer Than You Expect" (Mail 6819), which is true against the guess of 3 minutes |
| One-sentence takeaway | A timetable can be exactly right and still make you wait longer than half its average gap, because a moment chosen at random falls in long gaps more often than in short ones: the average wait is the average of the squared gaps over twice the average gap, never less than half the average gap, and equal to it only when every gap is the same. |
| The promise | First the setting. You catch a train by a bus from this stop, and you never check the timetable. Every bus is on time and the sign's "a bus every six minutes" is true. You predict your average wait from the fair-looking guess of 3 minutes, and it is 4 min 20 s (about 40 s in; no derivation in chapter 01, per the owner's 2026-10-24 direction). By the end you can compute any timetable's average wait from its gaps. You know what the sign's number does and doesn't tell you, and you know that the answer stops applying once you plan your arrival. You also know what to allow for the train: the longest wait, not the average |
| Vocabulary (one word per idea) | you, passenger, bus, stop, the blue bus and the red bus, timetable, the sign, gap (short, long, the gap that's growing), tag, turn up and come, at random, wait, waiting, triangle, averaged over buses and averaged over passengers, the passengers' gap, spread, even gaps, completely at random (storyboard, "Vocabulary") |
| The hook (where the obvious answer fails) | Halving the average gap. It is the right answer for even gaps and wrong for every uneven timetable, without any bus being late |
| The one picture | The hour as a line with ten bus markers. Above it, one tag per bus (the gap before it): the bus company's count. Below it, one passenger dot per minute: the count of moments. Then the wait drawn as a triangle over each gap, as wide and as tall as the gap. The waiting is the triangles' area, and a random moment lands in a gap in proportion to its width |
| Continuing example | Two routes, each every 12 minutes exactly, the red bus 2 minutes after the blue: gaps 2, 10, 2, 10, … (F-T1). Ten buses an hour, mean gap 6, half of it 3. Your lucky and unlucky mornings wait 0:30 and 9:30 (F-M1). One tag per bus: half say 2 and half say 10, averaging 6. Sixty passenger dots, one a minute: 50 in long gaps, so 5/6 of the passengers against 1/2 of the tags (F-S1, F-T1.long_time_share). Stretch one gap: from even gaps (red bus 6 after the blue) slide the red bus back. The buses' share of the growing gap stays 1/2, the passengers' share runs 6, 7, 8, 9, 10 of every 12, and with the buses together all 12 of 12, while the sign's 6 stays true (F-G1). In one 12-minute stretch, twelve passengers: 2 wait ½ + 1½ = 2, 10 wait 9½ + … + ½ = 50, 52 in all, 52/12 = 13/3 = 4 min 20 s (F-W2). Triangles 2 and 50, ratio 25, tiled by 15 + 10 copies (F-A1). Over buses the gap averages 6, over passengers 26/3 = 8 min 40 s; half of it is the wait, and the time since the last bus also averages 4 min 20 s (F-S1). Even gaps (red bus 6 after blue): 3 min; sliding the red bus d minutes after the blue gives (d² + (12 − d)²)/24, from 3:00 at d = 6 to 6:00 at d = 0 (F-O1). Between whole minutes the slider's moving readout rounds to the nearest second (2.5 gives 193/48 min, shown 4:01; only 13 of the 121 tenth-minute offsets are whole seconds). The formula: mean square 52 = 36 + 16, so 3 + 16/12 = 3:00 + 1:20 (F-F1). Completely random buses, mean 6: wait 6, the passengers' gap 12 (F-R1); three buses at once every 18 minutes: mean gap 6, wait 9 (F-R2). Two classes of 2 and 10: 6 by class, 26/3 by student (F-C1). A profiler: one 10 ms call gets half the samples, as many as five 2 ms calls (F-P1). You, reading the timetable and turning up 1 minute early, wait 1 minute (F-K1). If all sixty passengers plan, each stands 1 minute before the first bus they can make with a minute to spare: 10 before each blue bus and 2 before each red, still 5/6 in long gaps, every wait 1 minute, nobody turning up before they're ready (F-K2). The transfer puzzle: red bus 4 after the blue, gaps 4 and 8, triangles 8 + 32 = 40 per 12 minutes, 10/3 min = 3 min 20 s (F-Q1). For the train, allow the longest wait, 10 minutes, just after a red bus; 4:20 is the average cost (F-W1.allowance) |
| Audience | a curious 14-year-old with school algebra; a programmer who could compute it for their own service afterwards; a probabilist who finds nothing wrong (default, plan §1.2) |
| Length | the full draft (`docs/storyboard-draft.md`): twelve chapters, 87 lines, 2,272 spoken words. The storyboard tool's checks estimate 15.7 min, with every chapter between 63.7 and 84.8 s, under the 85 s cap. Production (`docs/storyboard.md`, `script.tsv`, the registry): chapter 01 only, 7 lines, 83.7 s, the one chapter with scene source (root Mail 6790). The first draft (nine chapters, 11.3 min) is kept in `previous/draft01/`, and the twelve-chapter storyboard as it stood before the split in `previous/draft02-registry12/`. Estimated, not measured: nothing has been rendered or narrated |
| Format | 3840×2160 @ 60 fps, the program's standard (default) |
| Voice | FrankenTTS through root's established route; this lane synthesizes nothing. Pause intentions are in the storyboard and `docs/pause-overrides.json` as editorial hypotheses; voice identity, accent, pronunciation, cadence and listening acceptance are all unverified |
| Look | 3Blue1Brown: dark background, role colors (storyboard, "Color roles"), continuous motion (default) |
| Owner script supplied? | no |
| Oracle | `oracle.py`: bus times built from the two routes; every average taken second by second over the hour (3,600 midpoints, exact because the wait is linear between buses), never by the closed form; the tiling counted on a lattice; the people as explicit lists; the profiler by sampling every tenth of a millisecond. The closed form appears only in `--self-test` cross-checks and the bound sweep. The planners are built from the dots and the bus list by bisection, and the slider between whole minutes in integer half-seconds. 166 facts; 76 self-test checks, 0 failures (WORK_STATUS.md has the exact output) |
| Pronunciations | `lexicon.json` lists every risk, nothing respelled |
| Out of scope | why real buses bunch (catalog `bus_bunching`, a dynamic feedback between delays and boarding), how real passengers time their arrivals, any measured service, the general renewal–reward theorem beyond its name, transfers, crowding, and the waiting-time distribution beyond its mean |

## What the video claims, exactly

Line ids from 02_1 on are in the full draft, `docs/storyboard-draft.md`.

- **Setting.** A stop served by buses at fixed times. A person "turns up at
  random": the moment they arrive is uniform over a long stretch of time
  and doesn't depend on the bus times (no look at the timetable). The wait
  runs from arriving until the next bus of any route.
- **The example.** Gaps alternate 2 and 10 minutes. The average gap over buses
  is 6. The average wait over moments is 52/12 = 13/3 min = 4 min 20 s, not
  3 (oracle: second by second over the hour; model: closed form).
- **Why.** Within a gap of length g the wait falls linearly from g to 0, so
  the waiting a gap holds is the triangle ½g², and a random moment lands in
  a gap with probability proportional to g. The average wait over a long
  time T is (sum of ½g²)/T = E[G²]/(2E[G]) = E[G]/2 + Var(G)/(2E[G]). Here E
  and Var are averages over gaps (equivalently over buses) in the long run.
  Scope: gaps with a finite long-run mean and mean square; for a single
  repeating timetable that is automatic. The gaps don't need to be
  independent of each other (Larson & Odoni, §2.13, below).
- **Bound.** The average wait is at least half the average gap, with equality
  exactly when Var(G) = 0, i.e., all gaps are equal. Proof: Var(G) ≥ 0, on
  screen in 08_8. The sweep (F-X1) is a finite check and is labeled as one.
  It covers 2,375 choices of up to 4 gaps from 0 to 12 minutes, with
  repeats allowed, order ignored and not all zero. That counts multisets:
  true sets would be 1,091 and ordered lists 30,936 (WildBass's review).
- **Two averages.** Averaged over buses, the gap is E[G] = 6. Averaged over
  people (length-biased) it is E[G²]/E[G] = 26/3. Given their gap, a person
  arriving at random is uniform within it, so the wait is half the people's
  gap, and so is the time since the last bus.
- **Completely random buses.** The model is a Poisson process with mean gap
  6, so the gaps are exponential with E[G²] = 72. The wait is 6 (the whole
  mean gap) and the people's gap is 12. Memorylessness is the one-sentence
  reason. A gap distribution with more spread than the exponential waits
  even longer (F-R2: 0, 0, 18 waits 9). Nothing is said about where real
  services fall.
- **Elsewhere.** Class sizes 2 and 10: the student average is 26/3 (Feld and
  Grofman's class size paradox). A sampling profiler's samples fall in
  proportion to running time, so one 10 ms call draws as many samples as
  five 2 ms calls. The claim is about expected counts for samples not in
  step with the program.
- **Two ways of counting, before any average.** The tags above the line
  (one per bus) and the passenger dots below it (one per minute) are shown
  on the same timetable in 02. The stretch experiment in 03 varies only the
  red bus. The buses' share stays 1/2 at every offset from 1 to 6, while the
  passengers' share is (12 − d)/12. With even gaps the two counts agree:
  that is the negative control for the attractive idea that the sign's
  average is your experience.
- **The model's limit, in the film.** You, reading the timetable and
  turning up 1 minute before a bus, wait 1 minute (11_4, 11_5). Then every
  passenger plans (11_6, 11_7). Each dot is the moment its passenger is
  ready. Each passenger stands 1 minute before the first bus at or after
  their ready moment plus 1 minute, so nobody turns up early. That puts 10
  before each blue bus and 2 before each red. Five in six still stand in a
  long gap, the time share, but every wait is 1 minute, because nobody is
  spread through a gap any more. Planning doesn't move passengers out of
  long gaps. It removes the even spread within each gap that the triangles
  assume. The paradox is about arrival at random, not about every
  passenger.
- **Your train.** A deadline needs the longest wait. For someone who turns
  up just as a red bus leaves, that is the whole long gap, 10 minutes.
  4:20 is the average cost of a morning, not a safe margin (12_5).
- **Not claimed:** that any bus is late; that bunching causes this example;
  that most passengers arrive at random; any real city's numbers; anything
  about view counts or audience response (MOR_CASE data supports editorial
  hypotheses only).

## Sources

Read, and used:

- R. C. Larson and A. R. Odoni, *Urban Operations Research*, §2.13 "Random
  Incidence". I read the MIT-hosted online edition on 2026-10-24
  (`web.mit.edu/course/other/urban_or_book/.../chapter2/2.13.html`). The
  points used:
  - random incidence "favors gaps of longer duration in direct proportion to
    their duration";
  - the size-biased density, eq. 2.63;
  - the mean wait, eq. 2.66, which "depends only on the mean and variance of
    the inter-event time";
  - perfect headways give a mean of T0/2;
  - Example 7: clumped buses are "a 50 percent increase over the case of
    perfect headways";
  - the arrival "can in no way depend on the past history of actual arrival
    times";
  - the inter-event times "need not be mutually independent".

  K05 cites this section. The equation images weren't transcribed, so the
  video's formula is checked by the oracle, not copied from the page.
- S. L. Feld and B. Grofman, "Variation in Class Size, the Class Size
  Paradox, and Some Consequences for Students", *Research in Higher
  Education* 6 (1977) 215–222. I read pages 215 and 216 of the scan Grofman
  hosts at UC Irvine on 2026-10-24. Page 216: "We call this the class size
  paradox." Their own example: an average class of 40.5 (from 737 classes at
  Stony Brook, spring 1974) against 147 as students experience it. The video
  uses only the names, year and term.
  Root source check, 2026-10-24: the author line on page 215 of the
  [UCI-hosted scan](https://sites.socsci.uci.edu/~bgrofman/R5%20%20Feld%20and%20Grofman.%201977.%20Variation%20in%20class%20size....pdf)
  explicitly gives Scott L. Feld and Bernard Grofman, supporting the
  expanded spoken names in 10_6.
- N. Masuda and T. Hiraoka, "Waiting-time paradox in 1922", arXiv:2004.14081,
  *Northeast Journal of Complex Systems* 2(1), 1 (2020). I read only the
  abstract, on 2026-10-24, and again that day through the arXiv export API.
  It describes an essay written "in 1922" by "a Japanese physicist, Torahiko
  Terada", who "described the waiting-time paradox, also called the bus
  paradox" and "observed and analyzed data on Tokyo City trams". The 09_7
  search names come from that sentence. I haven't read Terada's essay or
  the translation itself. The abstract gives no birth or death years, so
  the 10_7 card shows none.
  Root source check, 2026-10-24: the first sentence of the
  [arXiv abstract](https://arxiv.org/abs/2004.14081) explicitly describes
  an English translation and discussion. This supports the source card's
  description in 10_9; it does not establish a reading of the translation.

Located, not read (used for no claim):

- W. Feller, *An Introduction to Probability Theory and Its Applications*,
  Vol. II. A table of contents on a secondary mirror lists a section on
  waiting-time paradoxes. I haven't seen the book, so I can't confirm the
  section or its number, and the video cites neither. The 09_7 card no
  longer names Feller.
- G. F. Newell and E. Osuna's 1972 work on bus waiting times (*Transportation
  Science* 6(1)), which a secondary source credits with the (1 + C²)/2
  form. Unverified, and not cited in the video.
- N. Masuda and M. A. Porter's "The waiting-time paradox" (Frontiers for
  Young Minds, 2020), Jake VanderPlas's blog post "The Waiting Time Paradox,
  or, Why Is My Bus Always Late?", and Allen Downey's "The Inspection Paradox
  Is Everywhere" (blog and PyData talk). I found these as treatments for
  general readers and didn't use them for any claim. Root's original title
  echoed VanderPlas's, one more reason to change it.

## Competition check (bounded; search listings only)

I watched no videos. All I saw were titles and IDs in YouTube search
results on 2026-10-24.

| Listing | ID | What I know |
|---|---|---|
| "The Inspection Paradox Explained (Waiting Time Paradox)" | `mKGmH-9K0xU` | the title and ID in search results |
| "What's the Inspection Paradox?" | `dpLVhjVQSXw` | the title and ID in search results |
| "Inspection Paradox" (a Lego-built treatment, from the listing) | `Jd1wNizPjoE` | the title and ID in search results |
| "Allen Downey: The Inspection Paradox is Everywhere" | `cXWTHfvycyM` | the title and ID in search results; a conference talk |

From the listings alone, the topic is already covered under its own name. I
can't judge whether any of them builds a continuing timetable example, the
area picture, or the bound, because I haven't watched them. This film's
planned differences are editorial hypotheses, not claims about the others:

- one honest timetable, one commuter and one set of passenger dots carried
  through all twelve chapters;
- the bus company's tags and the passengers' dots on the same picture,
  before any average;
- one gap stretched, one cause at a time, with model readouts;
- the waiting drawn as triangle area, with the tiling;
- the two averages compared side by side;
- the offset slider from 3 to 6 minutes;
- a stated bound with its equality case;
- the timetable reader and the planners as the model's limit;
- a closing timetable the viewer solves from the picture.

## Checks

Cheap, no compilation (from the repo root):

```bash
python3 -I -B explainer_corpus/inspection_paradox/oracle.py               # key<TAB>value facts
python3 -I -B explainer_corpus/inspection_paradox/oracle.py --self-test   # 76 checks
python3 -I tools/storyboard.py --check inspection_paradox                  # G2 on chapter 01; writes nothing
python3 -I tools/lint_script.py explainer_corpus/inspection_paradox/script.tsv --chapters 01_the_stop
```

Without `--check`, `tools/storyboard.py` rewrites the `script.tsv` of
every topic, not just this one (CrimsonGrove's incident, Mail 6816).

The full draft gets the same checks through the tool's own functions,
with its script written outside the repository; that command and the full
draft's lint are in WORK_STATUS.md.

Root's native batch, after integration (I have run none of these): the
crate's build, `cargo test` (the SPOKEN table, storyboard, registry and
full-draft checks, chapter 01's headless run, model and palette tests),
`facts-check` against `oracle.py`, the glyph probe, fmt and clippy.
