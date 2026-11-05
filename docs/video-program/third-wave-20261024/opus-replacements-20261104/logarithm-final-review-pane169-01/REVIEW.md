# Logarithm final master: picture review of sheets 226-339

Reviewer: RubyRaven (pane 169), 2026-11-05. Owner of the decision: BlueSalmon.

**This is a sampled picture review, not final AV acceptance and not independent.** Stills only.
No hearing, no motion review. The author disclosure below is part of the result.

## Result

- No maths error in the 456 viewed tiles. 56 reviewer recomputations pass.
- No text overlap and no clipped text in any viewed tile.
- Both prediction holds keep the answer hidden until the countdown ends, on sampled frames.
- All four chapter joins are clean on sampled frames (08/09, 09/10, 10/11, 11/12).
- Three findings need a root decision: F1, F2 and F3.
- Nine smaller items and five unverified concerns follow. None of them is a maths error.

## Binding

| Item | Value |
|---|---|
| Master sha256 | `00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24` |
| Master | h264 3840x2160 60 fps, 116,245 frames; strict check passed per `verification.json` |
| SRT / VTT | `c9321cd7a2912d44...` / `45833988985413f3...` |
| `verification.json` | `22fb77188d6e9d86a0c9499e7a81db782d4b29830d1ddb4be40fe4dd3003fd62` |
| Sheets | `sheet-226.jpg` to `sheet-339.jpg`, 114 of 339, all hashes match |
| Frame PNGs | 456, all hashes match |

`checks/outputs/bind_sheets.out` says BIND PASS. `VIEWED.tsv` holds each sheet's hash, four frame
numbers and note. Sheets are 960x540 tiles of a 4K master, so every pixel figure here is at 960.
Replay commands are at the end of this file.

## Disclosure

I am a source author of this film, so this review is not independent of the pictures it covers:

| Mail | Work as RubyRaven that reaches sheets 226-339 |
|---|---|
| 1144 | chapter 11 and 00 picture repairs |
| 1228 | chapter 12 repairs F1-F7 |
| 1510, 1595 | chapters 11-12 repairs and follow-up, including the rule hold (H1) and cue splits |
| 1693 | cue checker and storyboard repair |
| 2011 | cross-review of the log10 interval repair |
| 2643, 4112, 4707 | stage06 preflight, stage09 render preparation and its review |

Shared notes also record chapter 09-12 label and readability repairs by sibling panes (WhiteSwan and
others). I do not count those panes as independent of me. Mail 1992 covers chapters 04-06, outside
this range. Lavender holds 001-112 and BronzeJaguar 113-225.

## Evidence

- Read as images: all 114 sheets, 226 to 339, in order. Each has one note in `VIEWED.tsv`.
- Read twice: 275, 276, 278, 279. Their notes dated from before a context reset, so I reopened the
  images and confirmed them. The bar widths in the note for 278 were corrected at that read.
- 456 frames of the 38,691 from f77554 to the last frame, 1.2%. Between tiles I saw nothing.
- From the master, two frames: see FINAL-FRAMES below.
- One enlarged crop of `frame-1295.png` (sheet 324), for P9.
- Text read, not pictures: SRT lines, `hold-audit.json`, `sources.md`, and source lines cited below.
- `checks/math_checks.py`, 56 recomputations; `checks/measure_labels.py`, 12 crops. Output in
  `checks/outputs/`.

## Observed findings

Severity is my judgement from stills. No source or production file was edited.

| ID | Severity | Sheets | Finding |
|---|---|---|---|
| F3 | Low-Medium, transition | 339 + last frames | The film stops 0.78 s after the last word, on a held 12-row card, with no fade or closing hold. f116197 to the last frame shows no change on the frames I saw. |
| F1 | Low-Medium, teaching | 264, 271, 272 | "det" is on screen at f91051 and "det" with "trace" at f92858. The narrator first says "determinant" at 1556.3 s, sheet 272. See U1. |
| F2 | Low, notation | 309-311 | `log A = diag(ln 2 + iπ, -ln 2 + iπ)`. Chapter 10 defines log A as the principal logarithm (`doing_nothing.rs:584`) with turning strictly inside (-π, π). This A has eigenvalues on the negative axis, and the footer on sheet 337 says that case has no principal log. The maths is right as "a logarithm of A"; the name is not. `unreachable.rs:1622`. |
| M1 | Low | 285-289 | The arrows use a concrete X the card never gives. The 1.17 cannot be checked from the screen, and the card just before said "here X is the sample flow from before", which this X is not. |
| P3 | Low | 240, 241, 254, 256, 284 on, 295 on | Right text reaches x≈930-940 of 960, past a 5% title-safe line at 912. Nothing is cut off. |
| P6 | Low | 276, 292, 300, 323-339 | Purple small labels are the weakest text: 3.98-4.42 by proxy, measured in the table below. Grey is 4.77-5.26; gold and white controls 13.43 and 15.20. |
| P7 | Low, film-wide rule | 278-279, 327-328 | No "pause for more time" on either card. REVIEW-AND-CONTINUITY §5 asks for it. A search of the crate's source and storyboard finds the phrase nowhere. |
| P8 | Low | 300-305 | Old -I rows stay beside the newer diag(-2, -½) rows and then beside the block tiles. Two text columns sit 21 px apart at the densest point. |
| P1 | Nit | 233-237 | Flowed grid corners are cut on vertical lines near x=35 and x=495, and read as a viewport. |
| P2 | Nit | 234+ | -0.97 beside three 3-decimal entries: -0.970 with the zero dropped. |
| P4 | Transient | 243-250 | First and last frames of clipped flows lose half their outer edge ("pinwheel"): f83946, f84114, f84444, f85187, f86311, f86551. |
| P5 | Nit | 275-279 | The green arrow to -2 runs along the middle arm of the yellow image, so the two merge. |

Checked and dropped: **P9**, the small bump at the cube's lower right corner on sheets 324 and 332. The
crop shows the far arc of the bottom circle passing just outside the vertical edge. That is true
geometry, not a defect. The notes still use the label.

### Contrast method

Ratio is WCAG, ink at the 99.5th percentile of a crop against its median. Thin strokes never reach
their drawn colour at 960, so these are conservative. They do not show that a viewer at 4K or 1080p
sees the same value.

| Frame | Sheet | Label | Ratio |
|---|---|---|---|
| `frame-1168.png` | 292 | purple: a half turn, then a stretch | 4.35 |
| `frame-1199.png` | 300 | purple: only the two axes go straight back | 3.98 |
| `frame-1295.png` | 324 | purple: spin: 1/2 lap per unit of time | 4.32 |
| `frame-1295.png` | 324 | purple: laps value 0.50 | 4.30 |
| `frame-1356.png` | 339 | purple: laps: 1/2, 1 1/2, ... | 4.42 |
| `frame-1125.png` | 282 | grey: (here X is the sample flow) | 4.88 |
| `frame-1199.png` | 300 | grey: dashes: straight back | 4.77 |
| `frame-1221.png` | 306 | grey: each row is one matrix | 5.01 |
| `frame-1289.png` | 323 | grey: each corner keeps to a circle | 5.26 |
| `frame-1295.png` | 324 | grey: time | 5.02 |
| `frame-1356.png` | 339 | control gold: so: infinitely many | 13.43 |
| `frame-1356.png` | 339 | control white: precisely footer | 15.20 |

## FINAL-FRAMES

Sheet 339's last tile stops at the cue end, f116197, so the last 48 frames are outside the
verification set. I extracted two frames from the bound master at 960x540: about f116219 and the last
frame, f116244. They are pixel-identical (sha256 `38d21b0f842f7fd5...`) and show the held 12-row card.
They are single-frame extracts and are not in this folder. Two points cannot show motion between them.

## Prediction and question timing

| Cue | Question | Silence | Bar seen | Bar starts | Answer shown | Result |
|---|---|---|---|---|---|---|
| 11_3 | Can a single steady flow end here? | 5.05 s | 4.05 s | 0.80 s after cue | f95439, after "No." begins at f95409 | Pass on sampled frames |
| 12_5 | How many steady spins end at this half turn...? | 5.07 s | 4.05 s | 0.82 s after cue | Sheet 330 on, after 12_6 starts at f112651 | Pass on sampled frames |

Both are inside the 4-6 s rule. On both, the card holds no answer during the hold. "A half turn, then
a stretch" on the 11_3 card is a fair lure: it shows two flows, not that one flow fails. These are the
only two `predict` calls in chapters 09-12 (source search). "Which one should a calculation return?"
(248), "Why does the argument fail there?" (296) and "Which steady motion got us here?" (320) are
rhetorical, with no hold, and I do not treat them as quizzes.

**H1, by design.** Silent interval f103972 to f105085, 18.55 s, against a planned floor of 18.33 s
(`hold-audit.json`). It is a music window with a gold mark walking the rows. Its plan says root may
shorten the row walk after listening, with 10.6 s as the floor. I helped build this hold (Mail 1595),
so a fresh listener should judge it.

## Chapter results

- **08 end, sheet 226:** puzzle chain, red false step and the Log 1 lines are correct.
- **09, 227-241:** rotation, flip, series and e^X recompute correctly (0.9052, -0.9704, 0.7763, 0.5171). Three flows reach one quarter turn.
- **10, 241-260:** identity logs, ellipses, number line, open window on "strictly between" and theorem card are correct. Sampled states agree with labels.
- **11, 260-311:** mirror, projection, gauge (1.221, 1.822, 0.670 are e^0.2, e^0.6, e^-0.4) and the commuting argument are correct. The half turn then stretch fills the target outline exactly (293). The 2x2 rule and the block rows agree with the even-count theorem, and arrows match A at measured positions (300, 301).
- **12, 312-339:** ruler and dial fit log spacing, fan and cut are right, laps = rate × time at every printed pair, and the half turn about another axis is consistent with parallel projection (334). Closing claims are correct for rigid spins.

Sheet-by-sheet notes: `VIEWED.tsv`.

## Unverified concerns

- **U1:** I cannot say whether sheets before 226 show "det". That belongs to Lavender and BronzeJaguar, and it changes F1.
- **U2:** Footer rows might meet captions if a viewer turns them on (258-260, 271-273, 303-311, 337-339). Caption placement was not rendered here.
- **U3:** Citations were not checked against the papers by me. `docs/sources.md` quotes Gallier Theorems 3.3, 3.4 and 3.11 as read. It records Culver as cited, not read, and the card says "as stated by" accordingly.
- **U4:** Motion is unseen: chapter 09's stretched and sheared states fall between tiles (229), as do fade lengths and whether P4 is noticeable. The row mark was caught only at rows 3 and 4.
- **U5:** Sound is unheard: music in H1, narration against every picture cue, the join audio, and whether the film's audio tail softens F3.

## Limits

No listening, no motion review, no independent acceptance. "Pass" above means the named check on the
named frames. Timing comes from frame numbers in `verification.json` and the timelines, which I did
not re-derive from the audio. Publication and no-listening gates are unchanged.

## Recommendations for root

None applied. All need a source change and a re-render, so they are root's call.

1. F3: decide after listening whether the last card needs a closing hold or fade.
2. F1 and F2: small on-screen wording changes if chapter 11 is rendered again.
3. M1: label the concrete X as an example, or say the gap is for one X.
4. P7: decide whether the §5 text applies to this film, since it touches every prediction card.

## Replay

From the repo root, with `R` set to this folder:

```bash
python3 -I -B "$R/checks/bind_sheets.py" "$R/checks/outputs/sheet-map.tsv"
python3 -I -B "$R/checks/math_checks.py"
python3 -I -B "$R/checks/measure_labels.py"
python3 -I -B "$R/checks/build_viewed.py" "$R/VIEWED.tsv"
```

`sheet-map.tsv` is an output of the first command and is not part of this folder as sealed.
`measure_labels.py` needs Pillow. `build_viewed.py` shows that notes and hashes line up. It cannot show
that a sheet was read.
