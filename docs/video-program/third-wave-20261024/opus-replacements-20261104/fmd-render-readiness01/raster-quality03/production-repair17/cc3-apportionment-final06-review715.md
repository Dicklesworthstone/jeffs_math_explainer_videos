# CC3 review 715: apportionment final chapter 06 sheets

Reviewer: CC3 (WhiteSwan). Task 715. Written before opening root's 714 verdict.

Subject: chapter `06_not_every_time` from job `csd-apportionment-final672-06_not_every_time`, 4947 frames at 60 fps, 82.433 s. `sampling.json` records movie sha256 `4ec4c3a6dfd43aae2e3ebdede89bda2354e5be3d8f9d25affa5fd06a46a83795`. I did not hash the movie.

## Evidence and scale

I viewed 6/6 contact sheets from root's local copy of the 678-06 sheets, which is 48/48 selected pictures. By the per-frame hashes in `sampling.json`, those 48 pictures are 25 distinct frame JPEGs. The Read tool returned every sheet as an image, 1920x2344 displayed at 1736x2119, with no truncation. I viewed all six twice this session, in batches of three, and the second pass came immediately before writing. Each picture is 960 pixels wide, downscaled from the 4K frame.

Each sheet hash below was computed by me with `shasum -a 256` and equals the entry in `sampling.json`.

| Sheet | sha256 | Selected frames |
|---|---|---|
| sheet-0001.jpg | `b4cc68cc0a54f5d678aa5de672191852973311f400349e400633cb841e8fbc8c` | 0, 18, 35, 36, 66, 182, 328, 329 |
| sheet-0002.jpg | `b8685712bd8a3b0e9ae853d74d547781cad79ee76ef07c38d82f1ff929c36682` | 365, 383, 480, 485, 630, 631, 661, 984 |
| sheet-0003.jpg | `ab734e5be4dc0b1a21ff14f7ca10d808a9b9da9a2116f8bad22ad69b42a685f9` | 1336, 1337, 1355, 1372, 1373, 1403, 1894, 2414 |
| sheet-0004.jpg | `11160a76f135f87dc173485ed63d50c5a00dbef3107293a72a8f44fd50d7371f` | 2415, 2433, 2450, 2451, 2481, 3060, 3669, 3670 |
| sheet-0005.jpg | `92804e5f34436df55db1e07b7a6447c98e9d6cbc1262b13da566a5ce6be2c16e` | 3688, 3706, 3707, 3737, 4040, 4373, 4374, 4392 |
| sheet-0006.jpg | `2415adc3fb83bed72eafecb909099cbddbd717f7e4919a0da139e89d7dfc3a8d` | 4410, 4411, 4441, 4660, 4909, 4910, 4928, 4946 |

Other files read, sha256 computed by me:

| File | sha256 |
|---|---|
| `sampling.json` (reasons, cue spans, predict mark) | `76548560976e741863de2a49552db89b6d840f92cd4cfcda14b0b03a0bb29ed8` |
| `explainer_corpus/apportionment/script.tsv` (cues 05_1 to 06_6) | `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624` |
| `explainer_corpus/apportionment/src/chapters/not_every_time.rs` | `d3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94` |
| `explainer_corpus/apportionment/src/chapters/dials.rs` | `ba3db7f7ec45ed58fad570d54ef9d0d49aa394fe6b7c79517f3c4a946f0c93a0` |
| `explainer_corpus/apportionment/src/chapters/draw.rs` (dial and think helpers) | `dd385ed5f45f09ffb8c0d8563829601bb147460bc16c7d4cbbf08245a7b23ba4` |

## Result

The 48 selected pictures show no wrong number, answer leak, clipped text or overlap at this scale. Every on-screen count agrees with an independent exact-fraction recomputation. There is one Medium finding, on what the viewer has available during the prediction, and it is a teaching choice with no factual error. The remaining findings are Low or Low-Medium. Whether F1 justifies rendering chapter 06 again is root's call; I would take F1 and F2 together if the chapter is rendered again for any reason.

## Findings

### F1, Medium: the 06_1 prediction leaves its inputs off screen (frames 36 to 630)

06_1 asks: "Add one more seat, seventeen in all. Does Ash get its seat back?" For the whole question and the 5.03 s hold, the frame has the 16-seat dials, the whole parts 1, 3, 10, and the row "1st Birch 0.69, 2nd Cedar 0.67, 3rd Ash 0.64". Frames 66, 182, 328, 329, 383, 480 and 630 show no text for 16 or 17, no question, no seat count for any town and no per-seat hand step. The digits under the dials are whole parts, so 3 and 10 are not the seats Birch and Cedar hold at 16, which are 4 and 11.

Working out the answer takes these inputs: Ash .64 + 4/39 = .74, Birch .69 + 9/39 = .92, Cedar .67 + 26/39 = 1.33, so Cedar's whole part becomes 11, the whole parts sum to 15, two seats are left, and the top two tails are Birch and Ash.

One chapter earlier the same device did carry those inputs. During the 05_3 prediction the three step wedges and the labels "+4/39 turn", "+9/39 turn" and "+26/39 turn" stay on screen (`dials.rs:133-143`). They fade during 05_4 (`dials.rs:165`). Chapter 06 then asks a harder question that depends on the same steps, after a chapter change, with the steps gone. A viewer can still guess in 5.03 s from the shape of the argument, but cannot do the sum from what is shown, and "pause for more time" offers a pause with nothing further to work from.

Fix, picture only, no narration or cue timing change: at the start of 06_1 redraw the wedges and "+n/39 turn" labels with the existing `step_arc` and label code. Their slot under each dial is empty in the 06_1 pictures. Add a top line at y 3.3, for example "16 seats → 17: does Ash get its seat back?", using the slot "2, 4, 11" takes later. Clear the wedges, labels and line with the "?" when 06_2 starts. A dim "seats at 16: 1, 4, 11" would also separate seats from whole parts.

### F2, Low-Medium: the history opening has no on-screen anchor (06_4, frames 2451 to about 3060)

06_4 opens with "After the eighteen eighty census, the chief clerk of the Census Office, a man named Seaton". Frames 2451 and 2481 still show the Ash strip from 06_3. By frame 3060 (51.0 s) the frame holds one dim line, "Hamilton apportionments for every House size from 275 to 350", and is otherwise empty. The year does not appear in any of the 48 pictures, and Seaton's name first appears in the 06_5 source line at size 20. The move from toy councils to real Congress is carried by voice alone.

Fix: extend the range line (`not_every_time.rs:141` shows it) to "1880 census, C. W. Seaton: Hamilton apportionments for every House size from 275 to 350". The current line spans about 47% of the frame width at size 24, so the longer one fits. Trigger it and the strip's fade on "After the eighteen eighty census" instead of "worked out Hamilton".

### F3, Low: "It comes and goes as the hands turn" plays over still hands (06_3, frames 1373 to 1403)

Source shows no tracker move between the start of 06_3 and the fade on "Run Hamilton's rule", and frames 1373 and 1403 match that. A slow sweep and return, as 05_5 does at `dials.rs:184-185`, would show the sentence. The sweep must first fade the ranking row and "2, 4, 11", which describe house 17 only, as `dials.rs:183` does for chapter 05.

The first sentence, "So the paradox is not a slide", can be heard as a presentation slide, and 07_3, 08_1 and 12_6 use "slide" for moving a marker. Changing it needs a new take, so I would only change it if 06_3 is being retaken anyway; "not a one way loss" keeps the meaning.

### F4, Low: the strip tags do not say what happened (frames 2414 to 2481)

The red tags read "15→16  Ash" and so on, and nothing on screen says these additions cost a seat or that five of the ninety nine are marked. The voice says it once, then the strip holds into 06_4. A single line under the strip, such as "5 of 99 additions cost someone a seat: always Ash", would let the held picture stand alone.

### F5, Low: "2, 4, 11" sits above whole parts 1, 3, 11 with no names (frames 1336 to 1403)

The gold line is centred above the three dials, with 1, 3, 11 beneath them. For a moment the two triples look inconsistent, and the viewer has to infer the two leftover seats from the ranking row. "Ash 2, Birch 4, Cedar 11" would remove that, and matches the "House of 300:  Alabama 7" style used later.

### F6, Low: whole-part numbers pop on while the dials fade (frame 36)

At frame 36 the three numbers are already almost at full strength, and everything else is still background. Peak channel value on the sheet at frames 36 and 66: Ash number 191 and 196, Birch number 235 and 243, Cedar number 209 and 215, Birch ring 22 and 255, Ash name 20 and 235, ranking row 20 and 255. Empty background peaks at 20 in the same measure. `draw.rs` adds the live whole part to the stage directly and leaves it out of the mobjects that fade in. It lasts at most the 0.6 s fade, and the shared helper means other chapters that open on a dial may do the same. Fix only if `dial` is touched for another reason: include the number in the faded group or start it at zero opacity.

### F7, Low: straight quotation marks in the Seaton quotation (frames 3737 to 4946)

The italic quotation uses straight `"` marks at both ends. Typographic opening and closing marks would suit the one quoted primary source in the chapter.

### F8, Low: the "pause for more time" device sits at the frame corner (frames 383 to 485)

On frame 383 the ring and caption together occupy 87.9% to 97.9% of picture width and 79.1% to 97.6% of picture height, measured on the sheet. The caption is size 16, dim, and the outermost ink in the chapter. Player controls may cover it once a viewer pauses, though by then it has done its job, and I did not test a player. `THINK_AT` is shared, so moving it changes other chapters. No action unless those chapters are rendered again.

### F9, Low, source only, join not viewed: chapter 05's final picture blinks off and returns

05 ends on the three dials at 16 seats with the ranking row (`dials.rs:188-191`), and `kit.finish` holds without a fade. Frames 0, 18 and 35 of chapter 06 are empty background, then frames 36 onward fade in that same picture. If the master joins the chapters with no transition, the picture disappears for 0.6 s and comes back unchanged. I did not view chapter 05's last frame or the join, so root should check this at the 05 to 06 boundary of the master.

## Checked, no finding

Prediction timing: `sampling.json` gives 06_1 ending at frame 328, a hold of frames 329 to 630 (5.03 s), the predict mark at frame 365 (6.083 s), and 06_2 starting at frame 631 (10.517 s). Source gives a 0.2 s fade, a 4.0 s drain (`THINK_S`) and a 0.2 s fade. Frames 383 (full ring), 480 and 485 (ring about half), and 630 ("?" alone, ring and caption gone) agree with that. The sheet's "+2.0 s (PREDICT_HOLD_S)" label at frame 485 names the shared kit constant, and this film uses `draw::think` instead, so frame 485 is simply mid-drain.

No answer leak: frames 630, 631 and 661 still show 16-seat hands, 1, 3, 10 and the 16-seat ranking. Frame 984 shows the new state. Source moves the hands on "The whole parts become", after "It does."

Numbers, against an independent exact-fraction recomputation from populations 4000, 9000, 26000:

| Check | Recomputed | Picture |
|---|---|---|
| House 16 | whole 1, 3, 10; tails .6410, .6923, .6667 | 1, 3, 10; Birch .69, Cedar .67, Ash .64 |
| House 17 | whole 1, 3, 11; tails .7436, .9231, .3333; 2 left; seats 2, 4, 11 | 1, 3, 11; Birch .92, Ash .74, Cedar .33; "2, 4, 11" |
| Houses 1 to 100 | 99 additions, 5 losses, all Ash: 15→16, 23→24, 54→55, 62→63, 93→94 | five red cells with those five tags, ends "adding seat 2" and "adding seat 100" |

Hand angles and gold arcs at frames 66 and 984 agree by eye with the printed tails. The five red cells fall at the positions their additions imply on the 99-cell strip, by pixel estimate on the sheet, and each tag sits over its cell.

Text against voice: "2, 4, 11", the Alabama pair and the 275 to 350 range match 06_2 and 06_4. The on-screen quotation matches the spoken words of 06_5. The note "alternative calculations, not an enacted change" matches 06_6. Frames 4660 onward dim the quotation and source while keeping the House pair, so the correction reads as the current point.

Layout: no clipping or overlap in the selected pictures. The tag rows do not collide. The five text tiers in frames 4660 to 4946 stay separate.

## Not established

This review covers 48 selected pictures of 4947 frames at contact-sheet scale, plus the named source. It makes no listening, motion, fine-sync, final AV or release claim. Fades, flashes, morphs and hand sweeps between samples were not seen. 4K detail, banding and phone legibility were not assessed. The Seaton quotation and the Alabama numbers were compared only with the script and source, not with Young or the primary record. No join was viewed. None of the proposed fixes was compiled or rendered.
