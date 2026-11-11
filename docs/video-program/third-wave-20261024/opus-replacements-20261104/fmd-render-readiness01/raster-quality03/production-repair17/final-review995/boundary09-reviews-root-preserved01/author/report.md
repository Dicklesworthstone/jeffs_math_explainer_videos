# 09 boundary: CC3 author picture review of the scope02 preview extraction

Reviewer: WhiteSwan (CC3). Kind: **author review, `authored_repair` true**. This is not the independent 09 review;
CC0 holds that one.

## Authorship

The 09 scene source is `boundary.rs` 4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932, CC2's
candidate03 applied over base cd1900aa. cd1900aa is CC3's repair752: the 09_1 inner polygon heads in hero yellow
with a ground rim (INNER_HEAD 0.16) and the name p moved to (-0.245, 0). Final969 already rendered it. CC3's reading
of the inner heads and of p is therefore an author's reading.

CC3 did not author or review candidate03 (M1 to M4, L14, the leader for a, the 09_9 card order). Its reading of those
items is independent of their author, but the review as a whole is signed as an author review.

## What was reviewed

- Movie: `preview/09_boundary.mp4`, sha256 da4df9de2a8271e8b9d27740a4943b1e789df96a5efbc5f178c8897bd9f05f74.
- Extraction: the scope02 job's 09 folder, copied read-only to `review09-cc3-actual01`. `pictures.json` sha256
  e2f1c855ce0037da9e72ddce9d8a0f59fec5e784212a4f3e606b968862408411 (16504 frames, full decode, 1506 selected
  pictures, 189 sheets). `index.tsv` sha256 f8e3e59b15fb4d3864ddce68648c0279e18741257d5e05a464832d796b840003 maps
  each picture to its sheet, frame, time and line.
- Source read beside the pictures: the candidate03 frozen folder (`boundary.rs` 4b689791 and `candidate03.patch`).

## Evidence

- All 189 contact sheets, read as images. Each sheet holds 8 pictures as 640x360 tiles of the 960x540 preview
  pictures. Every one of the 1506 selected frames was seen this way, so `viewed_frame_indices` lists all of them.
- Single 960x540 pictures opened where a tile was too small: 208, 224, 256, 340, 384, 440, 888 and 938 (frames 3660,
  4140, 4488, 5340, 5760, 6300, 11508 and 12708).
- Nearest-neighbour enlarged crops, in `crops/`: single pictures with `crop.py`, and the same region across several
  pictures side by side with `strip.py`. Twenty-three files, including three first attempts at the wrong rows
  (p791-card, p792-card, p968-x-edge), kept.
- On-screen numbers recomputed in plain Python, independently of the crate's oracle.
- Per-sheet notes: notes-001-020, notes-021-036, notes-037-056, notes-057-088, notes-089-103, notes-104-189.

## Not claimed

No 4K reading, no motion, playback or listening, no fine-sync or AV verdict, and nothing about frames outside the
1506 selected. The ElevenLabs Frederick Surrey narration was not heard. A gap of a few native pixels is about 2 px at
540p: a missing gap can be shown here, but the designed size of a clean one cannot be measured.

Root 343's four instants (frames 1661, 1914, 2131 and 2347) are not among the selected frames. `index.tsv` marks
pictures 127 and 128 (frames 2129 and 2130) against the instant at 35.512 s, and frame 1920 lies 6 frames after 1914.
These stay with the planned final stills.

## Checks

| Item | Author | Result at 540p | Evidence |
|---|---|---|---|
| 752 inner heads | CC3 (author reading) | Yellow triangles on the green inner polygon, distinct from the polygon and from the small circle. The ground rim is too small to resolve. | picture 256 (4488) |
| 752 p | CC3 (author reading) | Left of the inner polygon, clear of it and of the dashes; does not read as a name for the outer polygon. | picture 256 |
| M1, ARC and SUM | candidate03 | About 2 px between the z of dz and the closing bar in both bounds. | p384-arc-bar, p440-sum-bar |
| M1, SHRINK | candidate03 | About 2 px between the z and the bar. | p608-shrink-bar |
| M2 | candidate03 | rho leaves with the heads (7067 to 7076), returns at the small circle's right in the bar fade (7077 to 7090) and rides the circle down; never on the dashes. About 5 px clear at MID_R and 4 px at SHRUNK_R. | p496, p512, p608, p704 |
| M3 and the leader | candidate03 | a stays above the small circle's line: about 2.5 px at SMALL_R, 3.7 px at MID_R, 3 px at SHRUNK_R. The leader's foot stays at the hole ring. | same crops |
| L14 | candidate03 | About 1 px of dark gap between the green circle and the red hole ring at SHRUNK_R; the lines never merge. The designed 7.6 px at 4K cannot be measured here. | p704-shrunk |
| M4, 09_7 | candidate03 | a is full while the ring is a dot (13662 to 13667), fades while the ring is well short of it (13668 to 13675) and is gone by 13677. It returns at 13734 inside the ring, about 5 px clear. The ring is never drawn through a visible a. | s1000-1015-a, s1000-1015-b, s1070-1080 |
| M4, 09_9 | candidate03 | a is hidden 15618 to 15653 and returns at 15655 inside the ring, about 3 px clear. | s1362-1368-one |
| 09_9 card order | candidate03 | The ring's line goes under the "1" card and under the sqrt 2 card; both cards stay on top. | s1362-1368-one, s1408-1432-root2 |
| Layout lint anchors | (base scene) | 9408 is Low L-B and 13392 is Low L-C, below. | p791-card02, p792-card02, p968-x-edge02 |
| Numbers on screen | (base scene) | Means of 1/(1+z^2) over 8 and 16 samples of abs(z) = 1/2: 1.0039216 and 1.0000153; over 16 samples of abs(z) = 2: -0.0000153. The 60-term recipe centred at 1: 0.2000000002 at 2 and 212.18 at 2.6. 1 - sqrt 2 = -0.414 and 1 + sqrt 2 = 2.414. c_1 = 0 and c_2 = -1 match 1 - z^2 + z^4. All agree with the pictures. | plain Python |

## Findings

**Concrete defects: none.**

Four Lows, each older than candidate03 and repair752 (neither patch touches the code involved):

- **L-A** (09_1, from 5280 until the arc-and-chord card leaves). "inside the dashes" stands about 3 px above the
  card's top border (picture 340: text bottom near y 250, card top near y 253). They do not touch, but the caption
  reads as sitting on the card. Source: the card's top is at 0.25; the caption is centred at `GAP_TOP - 0.4` = 0.35 at
  size 17.
- **L-B** (09_3, layout lint anchor 9408). The card "opposite samples share a value" sits on the bottom border of the
  samples box and hides the box's green line under its middle (p792-card02, frame 9420). While it fades in, the box's
  vertical axis shows through it across the "o" of "opposite" (p791-card02, frame 9408). Once opaque (9420 on) it
  hides the axis. Source: the card at `v(right, -0.75)`; the box ends near y -1.01.
- **L-C** (09_7, layout lint anchor 13392, to the graph's fade). The axis name "x" ends at x 948 of 960 in picture
  968, 0.18 scene units from the right edge, inside the 0.25 margin the lint uses. Not clipped. Source:
  `axis_end[0] + 0.18`.
- **L-D** (09_9, from 15780 to the end fade). The graph's name "the bump" ends about 6 px left of the recipe's right
  spike at 2.41 (s1490-bump-spike: text end near x 879, spike near x 885). Crowded but readable: the label is white
  like the bump and the spike is yellow. Source: `TITLE` at `g.p(2.0, 0.45)`.

Two observations, not defects:

- **O-1** (09_1, from 2580 until rho leaves at 7067). rho names the small green circle but stands at RHO_AT, outside
  the yellow loop and about 0.67 from a, so it is tied to the small circle by colour rather than by position. Neither
  752 nor candidate03 moved it, and moving it into the annulus would put it under the triangle walks.
- **O-2** (09_5, 11460 to 12069). The shared "precisely" footnote card covers about 15 px of the plate's bottom,
  including its edge (p885-plate-bottom against p888-precisely). The r = 2 circle and its lowest sample stay about
  23 px clear. The card comes from the shared `precisely_now` helper.

## Verdict

`safe_to_advance_to_final4k` true on this evidence: no concrete defect in the 1506 selected pictures, and every
candidate03 item reads as designed at 540p. Because this is an author review, root weighs it beside CC0's
independent review. At final 4K, check the items 540p cannot settle: the 752 ground rim, the 2 px M1 gaps, the
7.6 px L14 gap, and root 343's four instants.
