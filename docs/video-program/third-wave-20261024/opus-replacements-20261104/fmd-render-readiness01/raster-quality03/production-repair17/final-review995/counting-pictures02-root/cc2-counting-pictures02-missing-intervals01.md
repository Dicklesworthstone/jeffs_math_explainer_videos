# counting-pictures02 sampling plan: missing intervals

CC2 (RubyRaven), 2026-11-10. Read-only source review. Nothing was launched, rendered or viewed, and no controller was written. Frederick narration is unchanged and was not assessed.

**What was checked:** root's `final-review995/counting-pictures02-root`, that is `sampling-plan02.json` and `frames-11_counting-all.txt` (831 frames), against `counting-root-repair02/counting.rs` `21c5043a`.

**The model:** `counting_frame_model01.py` (`be89a0de`) uses CC1's validated kit rules from `frame-windows01.py`. Every say() is pinned to the 29a3afd3 line frames. The model lands exactly on:
- all eight line frames;
- predict mark 5671;
- breath 11782;
- 11932 frames.

The 122 modelled events and 39 settled states are in `model01-trial.json` (`da32dd77`).

## Repair02 changes: all sampled

| Change | Frames | Plan samples |
|---|---|---|
| 11_2 first NEEDS card (new text, wider) | fade 1958-1975; on screen to 2537 | 1 in the fade, 62 on screen |
| 11_3 r 0.8 walk, fixed-gain walker | 2967-3123; stated count 3124-3293 | 16; 16 |
| 11_3 ring grows 0.8 to 1.7 | 3307-3361 | 6 |
| 11_3 r 1.7 walk, fixed-gain walker | 3388-3555; stated count and inset to 4287 | 17; 78 |
| 11_6 scale note | fade 6113-6137; on screen 6138-6991 | 5; 156 |
| 11_6 scale note removed, band fades in | 6992; 6992-7016 | 6988 before, 6994 after; 4 |

## Missing intervals (all in code that repair02 did not change)

| # | Cue | Frames | Transition | Plan samples nearest | Add |
|---|---|---|---|---|---|
| M1 | 11_7 | 7465-7490 | The painter hands over to the Turning field. Turning::build puts the field on screen at 7466, and painter.leave fades out over 7466-7490. The source says this happens "unseen". | 7454 (11_6 pieces still fading out), 7500 (claim fading in). No frame isolates the swap. | 7465, 7466, 7472, 7478, 7484, 7490 |
| M2 | 11_1 | 558/559, 884/885, 1128/1129 | Each of the three walks ends and its stated count replaces the live count. | 540 and 600; 840 and 900; 1080 and 1140. The last live frame is never sampled. | those 6 frames |
| M3 | 11_4 | 4720/4721 | The climbing \|f\| readout stops and the settled near-pole value replaces it. | 4680, 4740 | 4720, 4721 |
| M4 | 11_7 | 9322/9323 | The dog walk ends and "turns: 5" replaces the live count. | 9300, 9360 | 9322, 9323 |
| M5 | 11_8 | 10559 | The coarse walk ends and the certified count replaces the live one. | 10500; 10560 is sampled | 10559 |

The 17 frames are in `frames-11_counting-add01.txt` (`ef5b445b`). None is already in the plan; the union is 848 frames.

## Not counted as missing

- **Count swaps inside the dense windows.** These are bracketed within 2 to 9 frames: 1429/1440, 3121/3133, 3553/3565 and 6346/6352.
- **Geometry motions with at least one interior sample.** These are thin but present:
  - the 11_1 crossfades;
  - the 11_5 crossfade to blank (2 samples);
  - the 11_7 turn to 2i, 7630-7713 (1 sample, 7680);
  - the 11_7 turn back (2 samples);
  - the code panel slide (1 sample).
- **Two 11_8 coarse-dot states**, 10155-10197 and 10205-10247. Each is only a partial dot count, and every dot is in the later sampled states.
- **Extraction-time lint anchors.** The plan adds these during extraction, but they are unknown until the counting02 preview layout exists. Earlier 11_counting layouts were empty.

**Limits:** this checks the source frame model against the frame list only. It is not picture, motion, listening or fine-sync acceptance.
