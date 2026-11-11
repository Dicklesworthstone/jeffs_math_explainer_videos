# Exponential publication copy and master987 timings, plus pictures1008 review preparation (CC1 scratch, 2026-11-10 ~00:05 UTC)

**Scope and limits**
- Production files were only read.
- On CSS:
  - I read the `verify1005` inputs rows.
  - I counted framehash lines.
  - I ran a dry run of a record dump; it wrote nothing.
- Nothing was launched, uploaded, rendered or decoded. `wait-launch1008.py` is preserved and unchanged.
- For the copy check I read the script `explainer_corpus/exponential_magic/script.tsv` (d5a945b2, which equals script947) in full.
- I viewed both thumbnail images: the 1280×720 and the 320×180.
- No listening, watching or AV acceptance is claimed.

Inputs reviewed, all in `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/expanded-production808/`:

| File | sha256 (prefix) |
|---|---|
| `publication976/title.txt` | 7558da21 |
| `publication976/tags.txt` | 1de6abda |
| `publication976/description-timeline982.txt` | 4098b326 |
| `publication976/chapter-times982.json` | — |
| `publication976/draft-status.json` | c868069f |
| `thumbnail-reuse986.json` | — |
| `publication987-comparison.txt` | — |
| `publication987-master987-probe-pub.json` | — |
| `delivery-caption1004.en.{srt,vtt}` | — |
| `completed-pictures985-01..15.json` | — |
| the 30 `{root,independent}-final985-NN-review.json` files | — |

## Update: verify1005 is terminal (read from root's copy `final-review995/exponential1005-terminal-root01/`)

**Root's copy**
- `verification1005.json` has sha256 6c273f16…7909, which is root's pin.
- `resource-receipt.json` has sha256 e1d1b4d2…3c8b. It records admitted, exit 0, finished 2026-11-10T23:54:46.585874Z, and inputs equal to final inputs at 5d45eac1.

**Results recorded in it**
- movie fe33b9e4, 164,420 frames;
- `full_av_decode` and `all_decoded_video_frames_match_chapters` both true;
- 751 captions, with `fast_captions` empty;
- delivery captions SRT 127f3d1a and VTT 0d2698d0;
- chapter 15 has 5,282 frames, as expected;
- loudness −18.8 LUFS, true peak −1.5 dBFS.

**Pinned helper.** I recomputed it locally with `expected-pin1008-01.py`: the sealed 70dfce04 with only line 57 set to 6c273f16 gives **0d9d48e1…af3d**. That equals root's pinned helper.

**Loudness, for information, not a defect.** At −18.8 LUFS the true-peak ceiling binds, which the fixed-gain rule allows. Earlier uploads were −17.6 to −18.2 LUFS, so this film plays about 0.6–1.2 LU quieter than its siblings.

## Findings, most important first

**P1. Medium: the chapter 05 label is wrong.** The label is "9:31 Why the whole series converges".
- The series' own convergence is proved in 02_5: "Both converge uniformly on each bounded interval…".
- Chapter 05 proves something else: the finite product reaches the series. See 05_1 "must the whole sum arrive?", 05_5 "That proves the whole expression converges" (the expanded product) and 05_8 "So for every real x, that n-th power approaches E of x".
- A viewer who knows the maths would expect the 02_5 result under this label.
- **The film names this topic itself.** At the end of chapter 02 (1008 sheet 02/022, master frames 14846–14883), the on-screen list of open questions reads:
  - "Why adding inputs multiplies outputs", which is chapter 03's label word for word;
  - "Why the stepped approximations reach E", which is chapter 05;
  - "Why E is the only solution from height one", which is chapter 06.

  05_8 closes chapter 05 with "That pays the second debt."
- **Recommended label: "Why the stepped approximations reach E"**, which is the film's own on-screen wording.
- Alternatives in the script's words:
  - "Why the tiny steps reach the curve" (05_8);
  - "Why the whole sum arrives" (05_1).
- The fix is copy only. The labels do not appear in the film source; a search of `src/` found none of them.
- Changing it means writing new-named copies of `description-timeline982` and `chapter-times982`, because the latter binds `description_sha256`. The timestamps do not change.

**P2. Low to medium: no upload sheet exists for the exponential yet.**
- Earlier uploads recorded their settings: the logarithm's `upload-draft02.json`, and the rows in `docs/publish/MANIFEST.md`.
- Those settings are:
  - English;
  - Education;
  - not made for kids;
  - paid promotion No;
  - synthetic or AI disclosure Yes;
  - automatic chapters off;
  - the exact SRT as English captions.
- `publication976` has only the title, tags and description. `draft-status.json` still records `final_master_binding: false` and `independent_copy_review: false`.
- The sheet needs to bind these exact files:

  | Item | File | sha256 (prefix) |
  |---|---|---|
  | Master | — | fe33b9e4 |
  | Captions SRT | delivery-caption1004 | 127f3d1a |
  | Captions VTT | delivery-caption1004 | 0d2698d0 |
  | Thumbnail | — | 0d54f69e |
  | Title | — | 7558da21 |
  | Description | — | 4098b326, or its P1 successor |
  | Tags | — | 1de6abda |

  - The captions must be the delivery-caption1004 files, which are the hashes verify1005 checks. They must not be `native-master987.en.srt` (d91a5d08), which still has the four fast captions #29, #135, #440 and #441.
- This is the existing convention, not a new gate.

**P3. Low: the description credits the wrong rule for one claim.** The sentence is "Its central rule, that adding inputs multiplies outputs, explains both how a slide rule works and why the exponential keeps appearing in models of change."
- In the film, the models come from the rate law:
  - 06_8: "Whether a real quantity obeys this law…";
  - 08_3: "This is the rate law we already solved";
  - 08_7: "a precise rule forces their shape";
  - 09_3.
- The addition law explains the slide rule and the half-life clock (08_6).
- One possible fix: "…explains how a slide rule works, and its rate law, change in proportion to the current amount, explains why it keeps appearing in models of change." Root or the owner decides.

**P4. Low, editorial: the reused thumbnail and the title no longer share a message.**
- The thumbnail reads "THE MAGIC OF e^x" and "GROWTH & SLIDE RULES". It was made on 2026-11-02 for the 7-chapter cut, in which the slide rule came first.
- The current title is "Why the Exponential Appears Everywhere". It shares no words with the thumbnail, and the subtitle names 2 of the 15 topics.
- The title's "Everywhere" also sits against the film's own caution: 08_1 "A curve can look exponential without having an exponential cause" and 08_7.
- The thumbnail is technically sound:
  - I measured the drawing: the axis scales are unequal, about 126.7 px per x unit against 90 px per y unit;
  - the tangent has slope 1.00 at (0, 1);
  - the curve end points match e^x (x ≈ −1.78 gives 0.168, and x ≈ 1.2 gives 3.32);
  - the file is a 1280×720 PNG of 44 KB.
- Keeping it is the owner's decision. The reuse record already says `publication_acceptance: false` and `real_mobile_device_test: false`.

**P5. Low: the companion link could be clearer.**
- The description calls the companion the logarithm "from the real number line into the complex plane".
- The public companion, lgpTDROHgQY (title confirmed by YouTube oEmbed 2026-11-10), is titled "Logarithms Made Visual: From Slide Rules to Matrices". Its script covers matrices too: 13 lines mention matrices and 11 mention complex numbers.
- The catalog intent (TODO line 9242) also linked exponential_magic to matrix_exponential. That video is public as "What Does It Mean to Raise e to a Matrix?" (KHcukYSw53E, also confirmed by oEmbed), but the description does not link it.
- Optional: name the companion by its title, and add the matrix video.

**P6. Low: the tags could be stronger.**
- "Euler's formula" is missing, although 13_4 names it.
- half-life, capacitor (or RC circuit) and logistic growth are also missing.
- The tag "e" alone is weak.
- The tags use 201 of the 500 characters allowed.

**P7. Unverified: the Smithsonian link.**
- `americanhistory.si.edu/collections/object-groups/slide-rules` returned HTTP 403, "Smithsonian site temporarily unavailable", to both WebFetch and curl at about 23:55 UTC.
- Root's record says "search result verified", which does not say the page itself opened.
- It needs a browser check before upload.
- DLMF 4.2 is confirmed: "§4.2 Definitions", with subsections on the logarithm, a general base, the exponential function and powers.

## Checked, no issue

**Chapter times.** All 15 description timestamps equal floor(start_frame / 60). This was root's comparison (`publication987-comparison.txt`, using the final983 counts).

I added an independent source: verify1005's own decoded framehash line counts, counted at 23:50 UTC.
- Chapters 01–14 equal chapter-times982 exactly, and `master-video.framehash` has 164,420 lines.
- 15_logarithm_bridge had not been decoded yet at that point. The terminal `verification1005.json` now records 5,282 frames for it, which is 164,420 − 159,138.

The YouTube chapter rules hold:
- the first chapter is at 0:00;
- there are 15 entries, in ascending order;
- each lasts at least 10 s; the shortest is 88 s.

Markers land 0.05–0.98 s before each boundary because the times are floored. The largest gap is 0.983 s, at 10_changing_rates.

**Description claims against the script.**

| Description says | Script |
|---|---|
| slope equals height | 01 |
| factorials | 02 |
| one object, three definitions | 06_9 |
| slide rule | 07 |
| chemical reaction | 08_3, first order (root already corrected this from "radioactive decay") |
| capacitor | 09 |
| changing rates | 10 |
| compound interest | 11 |
| random multipliers | 12 |
| assumptions fail | 08_7, 09_8, 12_3–5 |
| rate turned sideways | 13_2 |
| three pictures of Euler's identity | 14_1–14_4 |
| companion | 15_3 |

The credit "Frederick Surrey voice from ElevenLabs" is correct. The other 14 chapter labels match their chapters.

**Format.**
- The title is 38 characters.
- The description is about 1.9 KB, has no angle brackets, and has one chapter block.

**Master probe.**
- Video: 3840×2160 H.264 High, yuv420p, with range tv and bt709 matrix, transfer and primaries; 60/1 fps.
- Audio: AAC LC, 48 kHz stereo, 256 kb/s.
- Length: 2740.333 s.
- Both streams carry the language tag "und"; Studio sets the language at upload.
- The average video bitrate is 365 kb/s, on mostly static graphics. That is a picture-quality question for the picture review, not a metadata problem.

**Delivery captions.** The local delivery-caption1004 SRT and VTT equal verify1005's pins (127f3d1a and 0d2698d0).

## pictures1008: preparation for the independent final picture review

**Same-bytes chain, read from verify1005's inputs rows.** For all 15 chapters, `master987/final/KEY.mp4` has the same sha256 as `final983 KEY.mp4`, which is also the `movie_sha256` in each pictures985 seed.
- pictures1008 selects exactly the 985 frames: local frame = master frame − offset. This is `selection()` in 70dfce04.
- Its ffmpeg argv is 985's plus `-frames:v N`, with the same `/usr/bin/ffmpeg`.
- **So all 3,163 JPEGs should be byte-identical to the 985 JPEGs.**

**What has already been reviewed** (`review985-coverage01.log`):
- Root and an independent reviewer each viewed all 3,163 pictures on all 402 sheets.
- None of the 30 reviews recorded a concrete defect.
- All 30 reviews bind the seed record sha256 that verify1005 binds.

**Tools in scratch:**
- `dump1008-records.py` runs read-only on CSS through ssh stdin. It prints every pictures1008 chapter record, each job's receipt fields and `recheck1008.json`. A dry run today found no 1008 jobs.
- `compare1008-985.py` runs locally. It checks:
  - the receipts, the master sha and `extract_source` (path, the 985 chapter sha, offset, frames);
  - per picture: local frame = 985 frame, master frame = offset + local, the reasons and the canonical paths;
  - the sheet frames, a single verification pin, and the recheck receipt and record rows;
  - the totals: 3,163 pictures and 402 sheets.

  It counts byte-identical images and lists any that differ, with their sheet numbers.

  Its self-test passed: the positive case gave 3163/3163, and all 7 controls were caught:
  - one differing image;
  - a local frame off by one;
  - another chapter's bytes;
  - a wrong sheet label frame;
  - a failed job;
  - no recheck;
  - another verification pin.

  The dry run on today's dump exits 1 with 33 hard problems, as expected before any job exists.

**Suggested review once the recheck has exited 0** (use new output names each time):
1. `ssh css /usr/bin/python3 -I -B - < dump1008-records.py > records1008-01.json`
2. `python3 -I -B compare1008-985.py <P808> records1008-01.json 6c273f16974dd3ac0f0363722be9ef308ed4f58a3d5809837acb3aaaf47f7909`. Expect exit 0: no hard problems and 3163/3163 identical.
3. View the new 1008 sheets. Only the label strip changes: "masterframe N | seconds" replaces 985's chapter-local "frame N".
   - At minimum, view the first and last sheet of each chapter (30 sheets).
   - The first label of each chapter should read its chapter-times982 start frame: 0, 5920, 14884, 25190, 34296, 47284, 60525, 74876, 86526, 98939, 110749, 119463, 132277, 146356 and 159138.
   - If step 2 exits 2, view every listed sheet first.
   - Whether to re-view all 402 sheets is root's decision.
4. State the scope as pictures only: no listening, AV, fine sync or publication acceptance.

**Authorship.** I wrote pictures1008, pin1008, the serial script, the launcher, both new tools and this note. So a check I run on the 1008 output is not independent. The comparator and the sheet viewing should be run by CC2 or another reviewer.

## pictures1008 actual output: chapters 01 and 02 (records dumped about 00:00 UTC on 2026-11-11)

**Records.** `records1008-01.json` was dumped from CSS read-only. `compare1008-985-run01.log` reports the following.

| Check | 01_rate_law | 02_factorials |
|---|---|---|
| Hard problems | 0 | 0 |
| Pictures byte-identical to 985 | 64/64 | 172/172 |
| Receipt | admitted, exit 0, stable inputs | admitted, exit 0, stable inputs |
| Master | fe33b9e4 | fe33b9e4 |
| `extract_source` = final983 chapter sha and offset | yes | yes |
| Local = 985 frame, global = offset + local | yes | yes |
| Sheet frames | match | match |
| Verification pin | 6c273f16 | 6c273f16 |
| `stream` matched | true | true |

- The stream check shows the expected tags: transfer and primaries unset in the chapter, bt709 in the master.
- The other HARD lines in the log are only for chapters 03–15, the recheck and the totals, which had not finished yet.

**Sheets.** All 30 sheets for chapters 01 and 02 were copied read-only to `sheets1008-review01/`. `check-sheets1008-01.log` shows each one's sha256 equals its record: 30/30.

**Viewed so far: 10 of the 30 sheets.**
- 01: all 8 sheets.
- 02: sheets 001 and 022, the first and last.

What I saw:
- **Labels.** They render legibly and match the records. 01 runs from masterframe 0 at 0.000 s to masterframe 5919 at 98.650 s. 02 starts at masterframe 5920 at 98.667 s, which is the 1:38 chapter start, and ends at 14883 at 248.050 s, just before 03's 4:08 start.
- **On-screen numbers I checked against the arithmetic:**
  - 1.25, 1.56, 1.95 and 2.44 are the successive powers of 1.25;
  - "4, 8, 16 steps: 2.441, 2.566, 2.638" equals (1+1/n)^n;
  - the constant-rate side reads height 2.00 at slope 1.00.
- **No concrete defect seen.**

Expected transients, not defects:
- The first frames of each chapter are black or fading in.
- 01 masterframe 3453 catches "4 steps: 2.441" in the middle of a cross-fade, so the text appears doubled for that frame.
- 02 masterframe 6820 catches a derivative connector half drawn.
- The short thick slope segment overhangs the time-1 axis end by a design-sized amount (a slope preview).

These are the same JPEG bytes that root and the independent reviewers already viewed in final985.

**Scope.** Pictures only. No listening, AV, fine sync or publication acceptance. I authored pictures1008, so this is not fully independent of the helper; the images themselves are byte-identical to the 985 pictures that were reviewed independently.
