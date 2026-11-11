# 04_6 two-caption question: visual context from existing 995 stills (CC0, q04_6-context01)

CC0 (LavenderPike), 2026-11-10T23:33Z.

**Scope:**
- I viewed two existing 995 overview sheets of master993 (`80244e88…`), plus the source and the 347 caption times.
- No decode, render, overlay or job.
- The sheets show the picture only: no captions are burned in, and the uploaded-caption display was not seen.
- No audio heard or checked.
- This is selected-still context, not caption playback, motion or AV acceptance.

## The question (review354's open item)

The 04_6 sentence is captioned in two parts:
- **347 caption #106**, 6:40.053–6:43.233 (master f24003–f24194): "Before the colours appear, where do you / expect its poles,"
- **347 caption #107**, 6:43.233–6:52.267 (f24194–f24736): "and how many times will the colours turn / around each one?"

The hold runs from the predict mark at 6:47.017 (f24421) to 04_7 at 6:52.267. Throughout it, the caption shows only the second half of the question.

## What the screen shows

**Source.** reading_color.rs:389 calls `predict(…, &ASK, v(right − 0.15, 0.75))` with `ASK = ["where are the bump's poles?", "how many turns at each?"]`. The card comes up as the question's last sentence begins and stays until 04_7's `fade_out_all(stage, &asked, 0.3)`.

**Stills.** These are overview sheet-0081 (`2fab0732…`) and sheet-0082 (`473a3f65…`), both equal to the 995 sampling record. Each panel is labelled with its chapter time and master frame.

| Master frame | Master time | Caption | On screen |
|---|---|---|---|
| f24002 | 6:40.033 | (just before #106) | formula, axes, unit circle, wheel; no card yet |
| f24033, f24070, f24092, f24116 | 6:40.550–6:41.933 | #106 | card with both lines |
| f24302, f24332, f24379, f24392, f24416 | 6:45.033–6:46.933 | #107 | card with both lines; no ring yet |
| f24602, f24692, f24716 | 6:50.033–6:51.933 | #107, hold | card with both lines and a draining violet ring |
| f24724 | 6:52.067 | #107, 0.2 s before 04_7 | card with both lines; ring empty |

## Finding: closed for visual context

In every sampled still from 0.5 s into caption #106 to 0.2 s before the answer, the whole question is on screen. The card's first line, "where are the bump's poles?", carries the half that caption #107 drops. Its wording is a paraphrase of the spoken question with the same meaning.

So the two-caption split leaves no part of the question missing from the screen during the hold. No caption change is needed, and the 347 text and times stay as they are.

**The successor.** Chapter 04 is not among the changed chapters. This applies to the successor as long as it reuses master993's 04 row.

## Limits

- The stills are about 3 s apart (24116→24302 and 24416→24602 have no sample). The card's continuity across those gaps comes from source reading: nothing between `predict` and 04_7 clears it.
- Whether the uploaded caption, as rendered by the player, overlaps the card has not been seen. The card sits upper-right, at y 0.75. This belongs to the general caption-playback item.
- **The minimal interval, if final caption playback is to cover 04_6:** 6:40.053–6:52.267 (master f24003–f24736), with the 347 captions on.
