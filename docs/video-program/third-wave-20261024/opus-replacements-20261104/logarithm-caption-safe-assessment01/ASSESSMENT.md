# Logarithm captions at YouTube's measured size and place

LavenderPike, 2026-11-06, for BlueSalmon (Mail 5893). Read-only. Nothing was decoded, rendered, uploaded or launched. Times
are Bronze's final SRT, `b2914c6f…cb22`.

## Answer

**No. At the measured size and place, the retimed captions do not clear the master.** The problem is much narrower than the
proxy suggested, though, and one unmeasured number decides which list below applies.

- **Controls visible (measured):** the −3.62 footers clear the two-line block by 56 rows. The −3.40 footers lose only the tips of
  their tallest glyphs, 3 to 8 rows. Mid-low lines between −2.46 and −3.30 are covered instead.
- **Controls hidden (not measured, and the normal playback state):** if captions drop, the footers are covered again. I sized that
  risk with an assumed 120 px drop.
- **Either state:** text or labels are under the caption for about 300 of 1,863 caption-seconds. **About 193 s, in eight spots, are
  covered in both states**, so moving between those two states does not help them.

## Minimal comparison on 23 real frames

Outlines only, on the first css run's clean 4K frames (`evidence/cmp1.jpg`). Blue is the proxy's real box, red is the measured
box and yellow is the assumed lower box.

| Box | Frames covering text or labels | What is covered |
|---|---|---|
| Proxy, libass default | 23 of 23 | Footers, as root found on the sheets |
| Measured, controls visible | 10 of 23: S01, S03, S05–S07, S18, S19, S21–S23 | Mid-low lines and labels; no footer beyond the glyph tips above |
| Assumed 120 px lower | 23 of 23 | Footers again |

The 23 frames were chosen for their low footers, so they flatter the measured state. The spot lists are the fair picture.

## Spots covered in both states

Check these first on any private stage, in this order.

| Time | Cues | Seconds | Controls visible | Assumed lower |
|---|---|---|---|---|
| 15:38.4–16:22.1 | 237–240, 243, 245, 247–248 | 32.2 | `e^L(z) = z = e^ℓ ⇒ L(z) − ℓ = 2πik, k whole` | Same formula, and from 16:15.6 "no continuous logarithm on a loop that winds around 0" |
| 22:58.7–23:09.3 | 345–347 | 10.6 | `X = (π/2 + 2πk) J ⟷ log i = (π/2 + 2πk) i` | Same formula |
| 24:43.9–24:50.8 | 372–373 | 6.9 | "turning rate of the flow of X = imaginary part of an eigenvalue of X" | Same line, and the footer |
| 01:31.4–01:46.9 | 23–25 | 14.8 | "×√2 then ×√2 = ×2" and "equal steps on the keyboard = equal ratios" | "equal steps…" and the footer |
| 06:34.9–07:10.3 | 102–103, 107–111 | 26.2 | "the 2.4 mark: one place on the dial", and its dot | Same label, and "a spiral, and its shadow" |
| 21:55.8–22:07.7 | 329–331 | 11.3 | "where each point lands, not how it travels" | Same line |
| 13:27.1–15:08.4 | 205–214, 221–229 | 83.6 | Grey label "its logarithm" / "its logarithm, close up" | Same label |
| 32:09.6–32:16.8 | 474–475 | 7.2 | Left ends of "for the number −1…" and "here: the whole family" | "here: the whole family" and the footer |

## Spots covered with controls visible only

| Time | Cues | Seconds | Covered |
|---|---|---|---|
| 00:50.4–00:57.4 | 13–14 | 7.0 | "note interval depends on frequency ratio" |
| 01:16.6–01:31.4 | 20–22 | 14.9 | "×√2 then ×√2 =", with "×2" outside |
| 02:23.6–02:30.0 | 35–36 | 5.8 | "every doubling adds the same length" |
| 04:15.7–04:19.4 | 65 | 3.7 | Part of "every base: the same ruler, stretched" |
| 05:07.4–05:10.3 | 78 | 2.8 | Middle of "sliding adds the lengths; the marks turn the sum back into a product" |
| 06:44.1–06:52.8 | 104–106 | 8.7 | "the 2.4 mark…" |
| 09:24.5–09:34.5 | 147–149 | 9.5 | The denominator of `log₂ x = ln x / ln 2` |
| 11:28.4–11:40.2 | 177–178 | 11.8 | "y sets the angle" |
| 14:56.6–14:59.9 | 227 | 3.3 | "change in the logarithm = 2πi × winding number"; also 15:03.9–15:08.4 as it fades |
| 15:56.5–16:15.6 | 241–242, 244, 246 | 10.8 | The 15:38 formula, under one-line captions |
| 17:50.0–18:01.7 | 269–271 | 11.7 | Left part of "angle just above the cut: near π…" |
| 18:17.5–18:32.5 | 276–279 | 15.0 | Left ends of "any curve from 0…" and "branch point" |
| 29:26.5–29:32.9 | 436–437 | 6.4 | Top edge of "flattening matrices, like zero, have no logarithm" |

## Spots covered with the assumed lower box only

All are footers.

| Time | Cues | Seconds | Footer |
|---|---|---|---|
| 05:44.9–05:52.4 | 89–90 | 7.4 | "a working slide rule reads about three significant figures…" |
| 10:04.1–10:25.7 | 157–162 | 21.6 | "100 × ln 2 ≈ 69.3…" |
| 18:59.2–19:11.8 | 286–288 | 12.6 | "Log is holomorphic off (−∞, 0]…" |
| 20:20.3–20:41.8 | 305–310 | 20.9 | "for nonzero z, w on the principal branch…" |
| 25:47.3–26:08.1 | 387–391 | 20.1 | "for any real linear flow…" |
| 28:39.5–29:16.1 | 430–433 | 18.2 | "Culver, Proc. AMS 17…" |
| 29:22.2–29:32.9 | 435–437 | 10.6 | "Gallier, arXiv 0805.0245…" |

## Not listed as text

With controls visible, the box also crosses diagram parts. They are in `evidence/measured-0.json`:

- 03:19.7–03:51.9: the lower ruler of the 19-doublings and 12-threes strip. Its ends and labels stay outside.
- 09:34.5–10:25.7: axis ticks 50, 100, 150 and "years".
- 19:34.0–20:41.8: axis tick labels.
- 22:10.3–22:43.7: corners of the tilted squares.
- 29:36.1–30:22.3: panel edges, and the left end of "many flows…".

Another 57 cues touch only an edge, a thin line or the left end of a right-column line. Cues 36, 147–148 and 269–271 also touch the
top edge of the assumed lower box.

## Suggested order

1. **Measure hidden controls first, on the Robot film.** Repeat Mail 5882's read during playback with the controls gone. That needs
   no upload and picks the list.
2. **Stage the exact master once, with Bronze's unpositioned sidecar.** Check the eight shared spots in both control states, then
   the list that step 1 picked.
3. **Decide then.** Accept the spots as a captions-on residual, position only those spots if the platform honours it at this size,
   or repair them natively. Nothing here justifies a rerender or a shrink yet.

Don't upload `candidate02` or `candidate03`. Both are sized for the proxy, `candidate02` crosses the title on every moved cue, and
they stay parked and unsealed in `logarithm-caption-safe-overlay01`.

## Limits

- Root's geometry is one paused state with controls visible, on another film, with one line's width. Viewer font settings are not
  recorded.
- The 120 px drop is my assumption, not a measurement. A larger real drop would change the lower lists.
- One-line captions are assumed to stay anchored at the block's bottom.
- Each box uses the cue's widest line plus 5%, so a shorter second line is overstated.
- The cue maps are brightest pixels over 2 fps samples at 960x540, threshold 34. They are not frames. Brief or dim elements can be
  missed, and a fading element counts at its brightest.
- The spot lists are my reading of `evidence/c1.jpg` to `c5.jpg`, so seconds are approximate where a cue was judged by eye.
- None of this is visual acceptance, sync evidence or proof of YouTube's behaviour.

## Evidence

`evidence/` holds the four small scripts, `measured-0.json`, `measured-120.json`, `cmp.json`, and the sheets. The red box in the sheets is
measured and the yellow box is assumed. `compare1.py` read the 23 existing PNGs at nice 10; there was no extraction job.
