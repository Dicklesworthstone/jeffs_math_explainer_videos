# Addendum 01: CC3 reading of CC0's P1, and how the two 09 reviews compare

Written after report.md, evidence.json and SUMS were sealed (SUMS e665b1fa). Those files are unchanged. This note
was written after reading CC0's independent review (Mail 25320, evidence 2db6cd8e), which CC0 finished before CC3
sent anything.

## P1 (CC0: at MID_R the leader reads as a radius from the centre to a)

Evidence: crops/s565-leader-p1.png, the same 60x70 region of pictures 512, 565 and 704 (frames 7090, 7180 and 7920:
SMALL_R, MID_R and SHRUNK_R) enlarged 6x with nearest neighbour.

The leader and M3 are candidate03's (CC2), so this reading is independent of their author.

- At SMALL_R (512) and MID_R (565) the leader rises straight up from the top of the red hole ring and ends just past
  the green circle, with a above it. Because a hole ring sits at the centre, the line runs from the centre to the
  circle, which is the shape of a radius. I share CC0's reading at these sizes.
- Two things separate it from a radius: its foot is at the hole ring's edge, not the centre point, and it is grey
  like a, not green like the circle and like rho. Neither is strong at 540p.
- At SHRUNK_R (704) the leader runs well past the circle and reads as a pointer to the hole.

I hold P1 as a Low judgement and not a concrete defect, so `concrete_defects` stays empty. The verdict in evidence.json
does not change. Whether to act is root's call; CC3 has drafted no change.

## The two reviews compared

Both reviews viewed all 1506 selected frames and found no concrete defect.

| Item | CC3 (this author review) | CC0 (independent, 25320) |
|---|---|---|
| "x" 0.18 units from the right edge, 13392 on | L-C | O3 |
| "precisely" card over the plate bottom, 09_5 | O-2 | O4 |
| SUM's final "0" near the right edge, 6300 | measured as a check, 18 px (notes-037-056), not raised | O2 (Low), about 17 px |
| M1 closing-bar gaps | about 2 px at 540p, visible | O1: visible, narrower by eye than the opening gaps |
| sqrt 2 ring at the card | goes under the card, which stays on top | O5: tight at the card's top-left corner |
| Leader at MID_R | above | P1 |
| "inside the dashes" 3 px above the arc-and-chord card | L-A | not raised |
| PAIR_UP card on the samples box border, 9408 | L-B | not raised (anchor present) |
| "the bump" 6 px from the recipe spike, 15780 on | L-D | not raised |
| rho outside the loop by colour, before the shrink | O-1 | not raised |
| a reads pinkish over the red plate in 09_9 | not raised | O6 (note) |

Neither review reports a defect that the other contradicts.
