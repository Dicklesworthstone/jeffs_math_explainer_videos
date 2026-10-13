# Publishing manifest: REVIEW ONLY (2026-10-12, 20:39 EDT)

**Nothing is public, and nothing may be published.** The owner reviews every cut first.

**The owner's choice: michael's voice for all four videos.** RaptorQ and the bakery video are
michael already; the fish video and Hoeffding get their michael versions (Hoeffding said
"HERF-ding"). He heard RaptorQ open with "a million phone" for "phones": michael's reads sometimes
drop a final s/z, and the ASR writes the word in full, so the gates passed it. Every michael cut is
being corrected (review 2): the chooser now detects a missing final s/z from the spectrum
(tools/narration.py, b8253f1), and each video's director replaces what remains, plus the items in
docs/publish/codex-tts-audit-20261012.md.

Channel: Jeff's Math Explainers (confirmed). Paths are relative to the repository root on the
owner's Mac; renders and captions are not in git, the text files are. Listed master candidates
were rendered at 3840×2160, 60 fps, BT.709 and have recorded `verify_video.sh` results under their
configured gates; each md5 here equals the checked master's on the build host. Originals and every
uploaded review cut are kept: a corrected cut always goes to a new path.

**Loudness qualification** (restored from 5c4975e and 64a0497, which a later rewrite of this file
dropped): every michael master here passed `VERIFY PASS` through the fixed-gain quiet-peak
exception, at −17.6 to −18.2 LUFS with the true peak at the −1.5 dBTP ceiling. That is outside
`AGENTS.md`'s literal −16 LUFS ±1 gate. The owner's disposition of the quieter level, and human
narration, score and full-watch acceptance, remain open for every cut; review-2 repairs need fresh
qualification. "Ready" here means available for a Private review upload; it does not establish
owner acceptance or permission to publish publicly.

## Uploaded for review (Private, by Codex; receipt docs/publish/codex-review-uploads-20261012.md)

| Cut | Link | Master | md5 | Status |
|---|---|---|---|---|
| Hoeffding robert (original) | https://youtu.be/savJOtXm7AQ | `explainer_corpus/hoeffding_d/renders/final/hoeffding_master.mp4` | `016f42a28ff497065dd9eb2a18a6f0a1` | kept; michael chosen instead |
| bakery michael, review 1 | https://youtu.be/PpmO9Gato2w | `explainer_corpus/bakery_algorithm/renders/final_review1/bakery_algorithm_master.mp4` | `19f2e859c0ee1a0841c0115ac69ab656` | superseded by review 3 (https://youtu.be/eXU6NC2fuCw) |
| fish matt (original) | https://youtu.be/fUXsjbWsgNg | `explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` | `528918af361165d6f73862d5fc5b9107` | kept; michael chosen instead |
| RaptorQ michael, review 1 | https://youtu.be/LXbcbfee_WE | `explainer_corpus/raptorq_gf256/renders/final_review1/raptorq_gf256_master.mp4` | `16867253aba700f7f8602ef753cb49bb` | superseded by repair 12 (https://youtu.be/uWDPs9JIwEk): "phones", "once", "those", "peels" lost their s; audit items |
| Hoeffding michael v2 | https://youtu.be/C9xOzbLIARg | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` | `76f66ca94c83b647a14d5f421c3d1cb2` | the current Hoeffding cut; evidence below |
| bakery michael, review 3 | https://youtu.be/eXU6NC2fuCw | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `9ed06e7d12d38a87d748d8a58806d296` | the current bakery cut; evidence below |
| fish michael v3 | https://youtu.be/VhWA7q6mTHw | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` | `a6a990f53e0935f5a96d0441a647f216` | the current fish cut; evidence below |
| RaptorQ michael, repair 12 | https://youtu.be/uWDPs9JIwEk | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `8e0afadf33bee181d59a81447d06fcb8` | the current RaptorQ cut; saved Private, still processing, captions not yet attached or verified; evidence below |

When a corrected review 2 of the bakery video or RaptorQ is copied here, the review-1 files above
move to `renders/final_review1/` in the same crate (never deleted), and this table says so.

BronzeRidge (Codex) uploaded Hoeffding michael v2 and bakery review 3 (mails 1295 and 1301). Both
are saved Private, English and Education, with the synthetic-narration disclosure, the matching
thumbnail and the description equal to the producer file (Hoeffding `5cbe867f…`, bakery
`96737b34…`); each one's exact SRT (`b643a09d…`, `522946ba…`) is verified Published in the
subtitles table, and BronzeRidge later reported both 4K-complete in Studio. The older uploads above
are kept. RaptorQ's review 1 is now in `renders/final_review1/`, as its md5 shows.

Fish michael v3 (BronzeRidge, mail 1321): saved Private, title "How to Count Fish You've Never Seen
[Michael review 3]", SD/HD/4K complete with no notices, the exact SRT `20620a55…` Published as
English (video language), the description equal to the producer file `d7baab5a…` (d34de59), the
thumbnail, tags, English, Education and the synthetic-narration disclosure applied. RaptorQ michael
repair 12 (mails 1339 and 1342): saved Private with the description `e12a9273…` (5810c25); still
processing, and its captions (`3b8c3ebe…`) are **not yet attached or verified**: BronzeRidge's
receipt for those follows. Every path above was re-checked by md5 on the build host.

## Ready, not uploaded

| Cut | Master | md5 | Length | Status |
|---|---|---|---|---|
| RaptorQ leo (the original voice, a comparison) | `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256_master.mp4` | `130493a0d35ce4e4f61eeb39047b6b73` | 21:14, −16.0 LUFS | verified; the owner chose michael, so upload only if he asks to compare |
| fish michael, v1 (superseded by v2) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v1/capture_recapture_master.mp4` | `3476fd0d155af2322d8f3597a0b66cfa` | 18:04, −18.1 LUFS | verified, but three endings ("size", "times", "things") lack their s: do not upload; v2 follows |

RaptorQ leo's text: `explainer_corpus/raptorq_gf256/publish/leo/{title,description,tags}.txt`;
captions `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256.en.srt`
(`96fc2e5c…`); thumbnail `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png`.

## Corrected michael cuts (review 2 and later)

| Cut | Master | md5 | Length | Text | Captions (md5) | Thumbnail |
|---|---|---|---|---|---|---|
| Hoeffding michael v2 (uploaded, above) | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` | `76f66ca94c83b647a14d5f421c3d1cb2` | 9:04, −17.8 LUFS | `explainer_corpus/hoeffding_d/publish/michael/{title,description,tags}.txt` (v2 chapter times) | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding.en.srt` (`b643a09d…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` |
| fish michael v2 (HOLD; superseded by v3) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v2/capture_recapture_master.mp4` | `2296eb1379ffccd5228a84668a112ac8` | 18:08, −18.2 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/variants/michael/renders/final_v2/capture_recapture.en.srt` (`db3e5fc5…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| bakery michael, review 3 (uploaded, above) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `9ed06e7d12d38a87d748d8a58806d296` | 15:43, −17.6 LUFS | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` (chapter times updated, ca49d2f) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`522946ba…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` |
| fish michael v3 (uploaded, above) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` | `a6a990f53e0935f5a96d0441a647f216` | 18:12, −18.1 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` (v3 chapter times, d34de59) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture.en.srt` (`20620a55…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| RaptorQ michael, repair 12 (uploaded, processing) | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `8e0afadf33bee181d59a81447d06fcb8` | 21:04, −18.0 LUFS | `explainer_corpus/raptorq_gf256/publish/michael/{title,description,tags}.txt` (chapter times of this cut, 5810c25) | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256.en.srt` (`3b8c3ebe…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` |

Bakery review 3 evidence: its director re-read three lines and froze the set (report.json
`b142acac…`): 01_7 "staircases", 04_6 "ties" and 11_5 "car's odometer"; the other 82 lines are
review 2's. VERIFY PASS (A/V 0.000 s, both decodes clean, the quiet-peak exception above), music
QA 13/13 after a re-score (no bed trim under ed882b6: the 13 stems are byte-identical), cue check
PASS (4 authored warnings), longest chapter 84.8 s, chapter times in the text follow this cut.
Codex's independent review of this master found no material picture defect and confirms its
level, captions and chapter times. BronzeRidge's full fw of this exact master against its SRT
(run ff12a0e7-4043-4cce-bd8f-89fe3c9730e5, whisper.cpp, no dropped windows; mail 1275) clears the
repaired "staircases", "ties" and "car's odometer", and "reads and writes", "Green's slate" and
"other's"; its new candidates "tickets stay safe" and "call wakes up" clear in isolated clips of
the final, and its two "I'm" tokens fall in the music-only coda. That is ASR, not hearing: nobody
has listened to this cut or watched it through, and the three new reads are **for the owner's
ear**. The last 20.6 s (after 15:23.06) are music only. Review 2 is kept at
`explainer_corpus/bakery_algorithm/renders/final_review2/` (master `7df3679f…`, captions
`7048a44c…`); review 1 (uploaded https://youtu.be/PpmO9Gato2w) at
`explainer_corpus/bakery_algorithm/renders/final_review1/` (md5 `19f2e859…`).

**Fish michael v2 is on hold; do not upload it.** The stricter final-hiss check (92d4953, ch and
sh too) flags 12_5 "size" and 13_6 "catch", and Codex's audit heard "letters", "cancels" and two
"catch"es weakly; its director fixed them in a v3 (uploaded, above; v2 is kept at
`variants/michael/renders/final_v2/`).

Fish michael v3 evidence: FINAL READY 1304, VERIFY PASS (A/V 0.000 s, both decodes clean, −18.1
LUFS: the quiet-peak exception above), music QA 15/15 on score engine ed882b6 (fingerprint
`82eb8f25…`), narration frozen at `5ab60c7d…`. BronzeRidge's full fw of this exact master (run
ea27cf8b-d38f-4268-89a0-d71dae84400d, no dropped windows; mails 1298 and 1301) clears the
requested repairs; its director's acoustic dispositions (NobleIbis, mail 1316) clear the remaining
candidates without regeneration. For the owner's ear: the first "marked" near 3:09 (long
aspiration), and the director's own list in 1304 (abrupt vowel ends on a few lines, thin clicks
near the end of 10_2.0 and 13_2.1, the long s of 13_3.1 "shrinks", 05_1.1 at 6.0 syl/s). Open
visual finding: BlueBrook's R67 P2, a transient label distortion at 13:55 (ring_toss.rs), kept
open for the owner's review with no recut of this hash (mail 1341). Nobody has listened to this
cut or watched it through.

Fish michael v2 evidence: VERIFY PASS (A/V 0.000 s, both decodes clean), music QA 15/15, cue
check PASS (13 warnings, all authored pauses or end holds, as in matt's cut); narration gates PASS
with 0 dropped final s/z and 0 cut short; "size", "times", "things" fixed; the audit's three fish
items checked in the takes ("Marked times", "five in a one", "pull the guess down": present).
Michael reads about 3% slower than matt, so chapters 04, 05 and 14 run 86-89 s, over our ~85 s
pacing guideline (not a render limit): for the owner's ear.

Hoeffding michael v2 evidence: VERIFY PASS (A/V 0.000 s, both decodes clean, the quiet-peak
exception above), music QA 8/8 on score engine ed882b6, built at 7767d25 from the frozen narration
(e69d2d2, ec27646: "An exact twin", "add a quarter each" and six endings repaired: noise, values,
marks, matters, lines, coordinates). Codex's full fw on this exact master is clear, with HERF-ding in
all nine places (BronzeRidge, run 501e40e3-c53c-4fe5-bdad-bbe065a79f75, mail 1271; no owner hearing
or full watch is claimed). v1 (`6898620c…`, superseded) is kept at `variants/michael/renders/final_v1/`.

RaptorQ michael repair 12 evidence: FINAL READY 1334, VERIFY PASS (A/V 0.000 s, −18.0 LUFS: the
quiet-peak exception above). It carries review 2's repairs ("phones", "once", "those", "peels",
fountain codes, ten to sixteen slots, parked pieces, the letter A) and re-reads 12_1 "byte order",
whose t had been only a glottal catch: narration frozen at `ebc43657…` (CoralForest, mail 1309). Chapter 12's
timeline, the music of 12–17 and the 4K finals of 12–17 were rebuilt; 01–11 keep review 2's finals
byte for byte (the new score moves their music by a uniform +0.0021 dB, which the master's replay
accepted). Cue check PASS; every chapter inside 85 s (12 is 80.88 s); the description's chapter
rows come from this cut. BronzeRidge's full fw of this exact master (run
12bce32e-8433-4e4c-91a1-55cb8daea064, no dropped windows; mail 1339) reads "byte order" and "a
million phones" correctly in full and isolated clips and clears the other requested repairs; the
"byte cards" reading is CoralForest's acoustic disposition (mail 1333). Nobody has listened to this
cut or watched it through. Kept, never deleted: review 2 (`eda64ed9…`) at
`renders/final_review2d/`, the first review-2 run (`f581a5de…`) at `renders/final_review2_93da69d/`,
review 1 at `renders/final_review1/`.

Each gets its own `publish/michael/` text (the fish and Hoeffding variants: chapter times differ),
captions in its `renders/final/`, and the video's thumbnail. Added here when verified and copied.

## Text, captions and thumbnails of the uploaded cuts

| Cut | Text files | Captions (md5) | Thumbnail, 1280×720 (md5) |
|---|---|---|---|
| Hoeffding robert | `explainer_corpus/hoeffding_d/publish/robert/{title,description,tags}.txt` | `explainer_corpus/hoeffding_d/renders/final/hoeffding.en.srt` (`d8cb9445…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` (`568b42df…`) |
| bakery michael, review 1 | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` (as uploaded: 7736fdd; the files now carry review 3's chapter times, ca49d2f) | `explainer_corpus/bakery_algorithm/renders/final_review1/bakery_algorithm.en.srt` (`27f50dfe…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` (`4be9ed65…`) |
| fish matt | `explainer_corpus/capture_recapture/publish/matt/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/renders/final/capture_recapture.en.srt` (`1f2df8ca…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` (`bce2cbc9…`) |
| RaptorQ michael, review 1 | `explainer_corpus/raptorq_gf256/publish/michael/{title,description,tags}.txt` (the files now carry repair 12's chapter times, 5810c25) | `explainer_corpus/raptorq_gf256/renders/final_review1/raptorq_gf256.en.srt` (`cb8b968c…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` (`ab3727ca…`) |

The corrected michael cuts' text, captions and thumbnails (Hoeffding v2, bakery review 3, fish v3,
RaptorQ repair 12) are in the corrected-cuts table above.

## Preserved, never to be uploaded (last night's cuts, narration the owner rejected)

| Video | File | md5 |
|---|---|---|
| fish (matt, old narration) | `explainer_corpus/capture_recapture/renders/previous/capture_recapture_master.mp4.1012-154255` | `503dd043c44b3478aaf71f033f623e67` |
| RaptorQ (leo, old narration) | `explainer_corpus/raptorq_gf256/renders/previous/raptorq_gf256_master.mp4.1012-154306` | `d0f036f20f9d9201192bde2232f4dee8` |
| bakery (pre-final, old narration) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.065600` | `1defb58a302f07dcae069bcbe5402b05` |
| bakery (first rework, superseded) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.112412` | `395800dc9cce1bcb82f5cf4e006e98ea` |
| Hoeffding (first rework, superseded) | `explainer_corpus/hoeffding_d/renders/previous/hoeffding_master.mp4.112410` | `62c48a13695960bcba3c8ddde0187092` |

The build host keeps its own copies under each crate's `renders/final_before_rework/`.

Settings for every review upload: Private; Education; "not made for kids"; altered or synthetic
content "No" (each description says the narration is synthetic); English captions from the .srt
"with timing"; the custom thumbnail; the voice in the review title. A cut is done only with its
video ID and processing state recorded.
