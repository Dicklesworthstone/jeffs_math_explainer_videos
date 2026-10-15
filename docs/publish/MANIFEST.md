# Publishing manifest: all four videos are Public (2026-10-12, 21:30 EDT)

**Update 2026-10-14: Video F (Mental Math Tricks That Actually Work (and Why), michael v2, 0720552) is Public too**, https://youtu.be/wahnqlfOC84, on the owner's word ("just publish it"); its row is in the Uploads table below. YouTube only: no GitHub release, and the channel's Advanced features (native chapters, clickable links) were still pending.

**Update 2026-10-14: Video E (How to Make a Fair Coin Out of an Unfair One, michael v3, 9ea6f85) is Public too**, https://youtu.be/f6iskjcSQCQ, on the owner's standing word for this process (BronzeRidge made it Public after its exact-master checks); its row is in the Uploads table below. Its v2's Private upload (https://youtu.be/R33HrxUyMKg) stays Private, superseded: 05_2's "Alice" was read 'Elise'. YouTube only: no GitHub release; the channel's Advanced features (native chapters, clickable links) are enabled as of this receipt.

**All four videos are Public** on the channel, Jeff's Math Explainers. RaptorQ, Hoeffding's D and
the bakery video went Public on 2026-10-12 at about 21:07 EDT (BronzeRidge, mails 1368 and 1378),
and the fish video at about 21:25 EDT (mails 1388 and 1389), as a new upload of its v4 final. Each
is the verified master under its plain title, with its description as the producer file, English
captions Published and SD/HD/4K complete with no notices; BronzeRidge confirmed the Public state
again after a fresh reload of the channel's content page. Checked from outside through YouTube's
oEmbed endpoint (21:18 for the first three, 21:27 for the fish video): the four links resolve
publicly with exactly the titles below, and the fish v3 review link answers 403. Every earlier
review upload and every original stays Private.

The owner authorized this on 2026-10-12 (relayed in mail 1361): "ok please just wrap things up and
make those fixes and then do final ultra high quality renderings to 4k 60fps for youtube, and give
them the final names and descriptions, nothing with '[Michael review]' or similar in the titles. We
want to just publish them, ok?" Shortly before, he said "the latest videos with the michael voice
sound pretty good" (mail 1358). **Only BronzeRidge uploads a video or makes it Public**; nobody else
publishes. The originals and every earlier upload are kept, and the earlier uploads stay Private.

## The release set

| Video | Final title (no suffix) | State and link | Master (md5) | Captions (md5) | Description (md5) |
|---|---|---|---|---|---|
| Hoeffding's D | Hoeffding's D: The Ring That Correlation Can't See | **Public**: https://youtu.be/C9xOzbLIARg | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` (`76f66ca94c83b647a14d5f421c3d1cb2`) | `hoeffding.en.srt` beside it (`b643a09d…`) | `explainer_corpus/hoeffding_d/publish/michael/description.txt` (`5cbe867f…`) |
| Fish | How to Count Fish You've Never Seen | **Public**: https://youtu.be/KNxRE255i-o (v4; the v3 review upload stays Private at https://youtu.be/VhWA7q6mTHw) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` (`8a17579a3ad74962464bd2523192f556`) | `capture_recapture.en.srt` beside it (`20620a55…`, the same as v3's) | `explainer_corpus/capture_recapture/publish/michael/description.txt` (`d7baab5a…`) |
| RaptorQ | The Arithmetic Where 3 × 7 = 9 | **Public**: https://youtu.be/uWDPs9JIwEk | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` (`8e0afadf33bee181d59a81447d06fcb8`) | `raptorq_gf256.en.srt` beside it (`3b8c3ebe…`) | `explainer_corpus/raptorq_gf256/publish/michael/description.txt` (`e12a9273…`) |
| Bakery | The Algorithm Lamport Is Proudest Of | **Public**: https://youtu.be/eXU6NC2fuCw | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` (`9ed06e7d12d38a87d748d8a58806d296`) | `bakery_algorithm.en.srt` beside it (`522946ba…`) | `explainer_corpus/bakery_algorithm/publish/michael/description.txt` (`96737b34…`) |

Titles, descriptions and tags are each crate's `publish/michael/{title,description,tags}.txt`; the
upload page's other fields are in its `publish/youtube.md`. RaptorQ, Hoeffding's D and the bakery
video are the same uploads that were reviewed Private (same URLs), retitled from "[Michael review
N]" to the plain titles above; BronzeRidge re-hashed each local master against the audited md5
before publishing. The fish video is a new upload of v4 under its plain title from the start.
All 12 links in the four descriptions answered HTTP 200 on 2026-10-12 at 21:06.

Fish v4 evidence (NobleIbis's FINAL READY, mail 1383): VERIFY PASS (A/V 0.000 s, both decodes
clean, −18.1 LUFS: the quiet-peak exception below), 1092.3 s, 65,538 frames. Only chapter 12 was
re-rendered, at 8770672 with the same x264 CRF 16 preset slow, for the label fix (12_5's words
stay put while the values swap); the other 14 chapter files are byte-identical to v3's, and so are
chapter 12's timeline and soundtrack, the captions and the decoded audio of the whole master. The
15 chapter times in the description equal the final timelines. BronzeRidge's full fw of this exact
master (run cee97032-280e-4bd0-ac61-4a1c3b4819a1, mail 1381) reproduces v3's approved transcript
segment for segment, with no new candidate. The fix was read in full-resolution crops of the
master at 13:55 to 14:05 and in dense draft frames, not watched through in motion. v3 is kept
whole at `variants/michael/renders/final_v3/`.

**Encoding: the existing masters are the final quality; they are not re-rendered or re-encoded.**
Every chapter of all four is 3840×2160, 60 fps, H.264 High, x264 CRF 16 preset slow, BT.709, with
AAC 256 kb/s. Measured on the bakery's hardest chapter (09, the static) against a lossless render
of the same source: PSNR median 63.4 dB (1st percentile 57.1, worst frame 48.9), SSIM median
0.99991 (minimum 0.9995). The fish lane measured its chapter 12 the same way, independently (mail
1383): SSIM mean 0.99986, PSNR-Y mean 63.4 dB. The low bitrates (0.8 to 3.3 Mb/s) are the flat,
mostly still picture: the lossless bakery chapter is itself 3.2 Mb/s. A changed cut re-renders only its changed chapters from
source at these settings; an existing MP4 is never re-encoded or upscaled.

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
`AGENTS.md`'s literal −16 LUFS ±1 gate. The owner has since said the four latest michael cuts
"sound pretty good" and authorized publishing them (above). He made no separate statement about
the level, and no uninterrupted full watch is recorded: the cuts are published as he heard them.
In the tables below, "ready" for a review upload meant available for a Private upload only.

## Uploads (by Codex; receipt docs/publish/codex-review-uploads-20261012.md)

Every upload began Private. The four marked Public were made Public on 2026-10-12 (above); all the
others are Private.

| Cut | Link | Master | md5 | Status |
|---|---|---|---|---|
| Hoeffding robert (original) | https://youtu.be/savJOtXm7AQ | `explainer_corpus/hoeffding_d/renders/final/hoeffding_master.mp4` | `016f42a28ff497065dd9eb2a18a6f0a1` | Private; kept; michael chosen instead |
| bakery michael, review 1 | https://youtu.be/PpmO9Gato2w | `explainer_corpus/bakery_algorithm/renders/final_review1/bakery_algorithm_master.mp4` | `19f2e859c0ee1a0841c0115ac69ab656` | Private; superseded by review 3 (https://youtu.be/eXU6NC2fuCw) |
| fish matt (original) | https://youtu.be/fUXsjbWsgNg | `explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` | `528918af361165d6f73862d5fc5b9107` | Private; kept; michael chosen instead |
| RaptorQ michael, review 1 | https://youtu.be/LXbcbfee_WE | `explainer_corpus/raptorq_gf256/renders/final_review1/raptorq_gf256_master.mp4` | `16867253aba700f7f8602ef753cb49bb` | Private; superseded by repair 12 (https://youtu.be/uWDPs9JIwEk): "phones", "once", "those", "peels" lost their s; audit items |
| Hoeffding michael v2 | https://youtu.be/C9xOzbLIARg | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` | `76f66ca94c83b647a14d5f421c3d1cb2` | **Public** since 2026-10-12 under its plain title; 4K complete, English captions Published; evidence below |
| bakery michael, review 3 | https://youtu.be/eXU6NC2fuCw | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `9ed06e7d12d38a87d748d8a58806d296` | **Public** since 2026-10-12 under its plain title; 4K complete, English captions Published; evidence below |
| fish michael v3 | https://youtu.be/VhWA7q6mTHw | `explainer_corpus/capture_recapture/variants/michael/renders/final_v3/capture_recapture_master.mp4` | `a6a990f53e0935f5a96d0441a647f216` | Private; the reviewed fish cut, superseded by v4 with the label fix (https://youtu.be/KNxRE255i-o); evidence below |
| fish michael v4 | https://youtu.be/KNxRE255i-o | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` | `8a17579a3ad74962464bd2523192f556` | **Public** since 2026-10-12 under its plain title; SD/HD/4K complete, English captions Published; evidence above |
| RaptorQ michael, repair 12 | https://youtu.be/uWDPs9JIwEk | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `8e0afadf33bee181d59a81447d06fcb8` | **Public** since 2026-10-12 under its plain title; SD/HD/4K complete, English captions Published; evidence below |
| Video F, Mental Math Tricks, michael v2 (0720552) | https://youtu.be/wahnqlfOC84 | `explainer_corpus/mental_math/renders/final/mental_math_master.mp4` | `766f27760c86c2fa698ddc830699d550` | **Public** since 2026-10-14 (BronzeRidge changed it on the owner's word, "just publish it"; fresh Studio reload: Public, SD/HD/4K complete, no notices, public watch page under the canonical title, Private badge gone; the authorization is for YouTube only: no GitHub release, Advanced features still pending). It was first saved Private for the owner's review (BronzeRidge's receipt, 2026-10-14): exact master (84,044,207 bytes), title as the producer file, description and tags as `publish/michael/`, thumbnail `5d2cbcb0…`, English captions SRT `dba90d5f…` (VTT `f67a5ada…`), Education, not made for kids, synthetic-narration disclosure Yes. BronzeRidge's final verification (2026-10-14, fresh Studio reload): Private, title and description match the producer text exactly, SD/HD/4K complete with no notices, English captions Published; watch-page playback at 2160p60 reports 3840x2160 and advances; the description's 14 timestamp links are present and match the validated package, and one representative seek, 10:33, played through 10:39 with the uploaded caption rendered (not all 14 were clicked); ebur128 of the uploaded master −18.1 LUFS, −1.5 dBTP. Open: the channel's Advanced features (native chapters and clickable links wait on it) and the GitHub Release asset (its own bead, owner approval). Review points the owner saw before publishing: −18.1 LUFS (ceiling-bound, verify's fixed-gain clause) and chapter 07 at 87.62 s. The 10:54.6 phrase "Bethe's twenty-three hundred" is written "beta is" by fw: the same spelling it gives the BEE-tuh decoy; the take's vowel is /eɪ/ (formants), not a misarticulation (EmeraldOwl; evidence in the release bead mx-e-a-f-ms4.13) |
| Video E, Fair Coin, michael v2 (29d168c) | https://youtu.be/R33HrxUyMKg | `explainer_corpus/fair_coin/renders/final_v2/fair_coin_master.mp4` | `ab9b2cfc71f4600c74a02abb74d804ef` | Private; kept; superseded by v3 (https://youtu.be/f6iskjcSQCQ): 05_2's "Alice" was read 'Elise'; not to be published |
| Video E, Fair Coin, michael v3 (9ea6f85) | https://youtu.be/f6iskjcSQCQ | `explainer_corpus/fair_coin/renders/final/fair_coin_master.mp4` | `4471f2580e022b41374d61be59c0ce11` | **Public** since 2026-10-14 (BronzeRidge, on the owner's standing word; fresh Studio reload: Public, 4K complete, copyright and guidelines no issues, canonical title and the 4,848-character description exact; the watch page has no Private badge and plays 3840x2160 at 2160p60 with the manually published English captions; the 11 native chapter entries match the description's timestamps, seeks at 342 s and 660 s checked; the full unprompted fw of this exact master found no new discrepancy against v2 and the old 'Elise' resolved; the Public source page loads with the 11:00 final chapter) |

When a corrected review 2 of the bakery video or RaptorQ is copied here, the review-1 files above
move to `renders/final_review1/` in the same crate (never deleted), and this table says so.

The receipts of the review uploads, as they stood before publication: BronzeRidge (Codex) uploaded
Hoeffding michael v2 and bakery review 3 (mails 1295 and 1301). Both were saved Private, English
and Education, with the synthetic-narration disclosure, the matching
thumbnail and the description equal to the producer file (Hoeffding `5cbe867f…`, bakery
`96737b34…`); each one's exact SRT (`b643a09d…`, `522946ba…`) is verified Published in the
subtitles table, and BronzeRidge later reported both 4K-complete in Studio. The older uploads above
are kept. RaptorQ's review 1 is now in `renders/final_review1/`, as its md5 shows.

Fish michael v3 (BronzeRidge, mail 1321): saved Private, title "How to Count Fish You've Never Seen
[Michael review 3]", SD/HD/4K complete with no notices, the exact SRT `20620a55…` Published as
English (video language), the description equal to the producer file `d7baab5a…` (d34de59), the
thumbnail, tags, English, Education and the synthetic-narration disclosure applied. RaptorQ michael
repair 12 (BronzeRidge, mail 1343, checked at 20:38): saved Private, title "The Arithmetic Where
3 × 7 = 9 [Michael review 2]", the description equal to the producer file `e12a9273…` (5810c25),
the exact SRT `3b8c3ebe…` Published as English (video language), the thumbnail, tags, English,
Education and the synthetic-narration disclosure applied; SD, HD and 4K are complete with no
notices (BronzeRidge, mail 1346, checked at 20:41). BronzeRidge's screenshot of
the channel shows all four new michael reviews Private with no notices, beside the four kept
earlier uploads. Every path above was re-checked by md5 on the build host. RaptorQ, Hoeffding and
the bakery video have since been retitled and made Public (the top of this file); the fish v3
upload has not, and v4 was uploaded separately and made Public.

## Ready, not uploaded

| Cut | Master | md5 | Length | Status |
|---|---|---|---|---|
| RaptorQ leo (the original voice, a comparison) | `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256_master.mp4` | `130493a0d35ce4e4f61eeb39047b6b73` | 21:14, −16.0 LUFS | verified; the owner chose michael, so upload only if he asks to compare |
| fish michael, v1 (superseded by v2) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v1/capture_recapture_master.mp4` | `3476fd0d155af2322d8f3597a0b66cfa` | 18:04, −18.1 LUFS | verified, but three endings ("size", "times", "things") lack their s: do not upload; v2 follows |
| Video F, Mental Math Tricks, michael v2 (0720552) | `explainer_corpus/mental_math/renders/final/mental_math_master.mp4` | `766f27760c86c2fa698ddc830699d550` | 15:34, −18.1 LUFS (true peak at the −1.5 dBTP ceiling) | verified on the build host (finals2); `tools/release.py check mental_math` PASS on this laptop copy, 2026-10-14; uploaded Private on 2026-10-14 and made **Public** the same day on the owner's word (https://youtu.be/wahnqlfOC84, row in Uploads above). F v1 (`renders/final_v1/`, `ccc684b23c200d6ed09094509d02d73b`) has orphan captions and v1's chapter times: do not upload |
| Video E, Fair Coin, michael v3 (9ea6f85) | `explainer_corpus/fair_coin/renders/final/fair_coin_master.mp4` | `4471f2580e022b41374d61be59c0ce11` | 11:56 (716.4 s, 89,244,021 bytes), −17.7 LUFS (true peak at the −1.5 dBTP ceiling: verify's own note) | verified on the build host (finals2, 22:50; v2 with 9ea6f85's pick of 05_2's "Alice wins four games in five", which v2 read as 'Elise': chapter 05 is 4,169 frames against 4,205, every later start 0.6 s earlier, the other ten chapters' video packets byte-identical, their audio re-scored on the new timeline: chapters 01-04 differ from v2's by about −38 dB re the signal, 06-11 by about −21 dB, QA PASS in all 11); delivery check (final_check, blank_stretches, band_dwell) on mx-e-a-e-e-k23.11: as v2's, 183 cues, 0 blank stretches, band_dwell's one flag the 06_7 band that grows in place; fetched md5-checked; `./x fair_coin release-check` PASS on this laptop copy, 2026-10-14; uploaded and made **Public** on 2026-10-14 as https://youtu.be/f6iskjcSQCQ (row in Uploads above). The v2 cut (`ab9b2cfc…`, 05_2 heard 'Elise'; its Private upload https://youtu.be/R33HrxUyMKg is superseded and stays Private) is at `renders/final_v2/` and the fbe959c cut (`a5805f30…`, its listen windows soundless) at `renders/final_v1/`: do not upload either |
| Video G, Lie Robotics, michael v2 (8086fe4) | `explainer_corpus/lie_robotics/renders/final/lie_robotics_master.mp4` | `b158f58a24f43878cea07c2c782b6c24` | 16:38, −17.8 LUFS (true peak at the −1.5 dBTP ceiling) | verified on the build host (finals2, 20:22; with 5b31523's plot notes, chapter 11 the only change, PeachPuma's comparison on mx-e-a-suh.13); fetched md5-checked; `./x lie_robotics release-check` PASS on this laptop copy, 2026-10-14; not uploaded: Private for the owner's review first, by BronzeRidge only. The fbe959c cut (`c4cd3fca…`) is at `renders/final_v1/` and again at `renders/final_v2/`: do not upload |
| Video H, How to Find a Lost Submarine, michael v2 (ab6c734) | `explainer_corpus/bayesian_search/renders/final/bayesian_search_master.mp4` | `797dac61555d252bd544e595149d85af` | 15:54, −18.1 LUFS (true peak at the −1.5 dBTP ceiling) | verified on the build host (finals2, 21:24; with 1261dd3's 10_3 band dwell and d7e904b's theme chord); fetched md5-checked; `./x bayesian_search release-check` PASS on this laptop copy, 2026-10-14; not uploaded: Private for the owner's review first, by BronzeRidge only. The fbe959c cut (`bdcc3dcbf92a6d39ff94ced91d540ada`) is at `renders/final_v1_fbe959c/`: do not upload |
| Video I, Proofs You Can See, michael v3 (a305d08) | `explainer_corpus/visual_proofs/renders/final/visual_proofs_master.mp4` | `f3c7a03a821b86b9deb4c680670a0cd5` | 18:11, −17.9 LUFS (true peak at the −1.5 dBTP ceiling) | verified on the build host (clone 1, 21:54; with 7ca56f3's 07_3 aside, which now stands about 6.5 s, seen on the final's frames); delivery check (final_check, blank_stretches, band_dwell) as v2's: the same four notes for the owner on mx-e-a-dza.11 and band_dwell's one 13_7 flag, a picture zooming in, not a band; fetched md5-checked; `./x visual_proofs release-check` PASS on this laptop copy, 2026-10-14; not uploaded: Private for the owner's review first, by BronzeRidge only. The v2 cut (`5a08f714…`, 07_3's aside 1.47 s) is at `renders/final_v1/`: do not upload |
| Video J, What Does It Mean to Raise e to a Matrix?, michael v2 (a425038) | `explainer_corpus/matrix_exponential/renders/final/matrix_exponential_master.mp4` | `0d08270c475f772de2c70e793c8228c9` | 18:08, −18.0 LUFS (true peak at the −1.5 dBTP ceiling) | verified on the build host (clone 1, 21:01; with a425038's four bands held to 2.6 s); fetched md5-checked; `./x matrix_exponential release-check` PASS on this laptop copy, 2026-10-14; not uploaded: Private for the owner's review first, by BronzeRidge only. The v1 final (`ab6c734` finals) is at `renders/final_v1_ab6c734/`: do not upload |

**E, G, H and J are their v2 now (2026-10-14, mx-vu2t).** Their earlier cuts predated fixes made for them:
46678ef (E's three listen windows, 02_8, 03_2 and 05_2, were soundless), 5b31523 (G 11_5/11_6's plot
notes), 1261dd3 (H 10_3's band dwell), and a425038 (J's four bands held to 2.6 s); the rows above are the v2 cuts that carry them, and only those are to be uploaded. I is its v3 (a305d08, 7ca56f3's 07_3 aside). K's v2 (c777b7e) had two
bands under 2.6 s (04_7 2.53 s, 07_5 2.30 s), fixed in 08b72fd: K v3 renders in clone 1 at 6c2af21, and only it is to be uploaded.

RaptorQ leo's text: `explainer_corpus/raptorq_gf256/publish/leo/{title,description,tags}.txt`;
captions `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256.en.srt`
(`96fc2e5c…`); thumbnail `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png`.

Video F's text: `explainer_corpus/mental_math/publish/michael/{title,description,tags}.txt` (v2's
chapter times, e4338e1); captions `explainer_corpus/mental_math/renders/final/mental_math.en.srt`
(`dba90d5f…`); thumbnail `explainer_corpus/mental_math/renders/mental_math_thumbnail.png`
(`5d2cbcb0…`); the upload sheet is `python3 -I tools/release.py check mental_math` (the description,
tags, category, audience, Private and the synthetic-content line of `publish/youtube.md`).

Video E's text: `explainer_corpus/fair_coin/publish/michael/{title,description,tags}.txt` (chapter
times filled from v3's final timelines, 1e2e379); captions `explainer_corpus/fair_coin/renders/final/fair_coin.en.srt`
(`4b3f2895…`, VTT `a9dc8f8d…`: v3's own, made by the kit at 9ea6f85, after its captions fix 1f44182,
final_check PASS, 183 cues as v2's `92a2d8cb…`/`f1f89fa6…`, whose cues after chapter 05 sit 0.6 s later; v1's
fbe959c captions, `446ee8d4…`, with nine orphan tails, are at `renders/final_v1/orphan_captions_fbe959c/`:
do not upload them); thumbnail `explainer_corpus/fair_coin/renders/fair_coin_thumbnail.png`
(`728c23b9…`); the upload sheet is `python3 -I tools/release.py check fair_coin` (the description,
tags, category, audience, Private and the synthetic-content line of `publish/youtube.md`).

Video G's text: `explainer_corpus/lie_robotics/publish/michael/{title,description,tags}.txt` (chapter
times filled from final timelines, c2b4a1b); captions `explainer_corpus/lie_robotics/renders/final/lie_robotics.en.srt`
(`52127452…`, VTT `56c9ed9c…`: rebuilt from the same narration and timelines by the kit's captions fix
1f44182, 0 orphan tails, final_check PASS); thumbnail `explainer_corpus/lie_robotics/renders/lie_robotics_thumbnail.png`
(`7fc39fc8…`); the upload sheet is `python3 -I tools/release.py check lie_robotics` (the description,
tags, category, audience, Private and the synthetic-content line of `publish/youtube.md`).

Video H's text: `explainer_corpus/bayesian_search/publish/michael/{title,description,tags}.txt` (v2's
chapter times); captions `explainer_corpus/bayesian_search/renders/final/bayesian_search.en.srt`
(`644c9c29…`, VTT `44cf8763…`); thumbnail `explainer_corpus/bayesian_search/renders/bayesian_search_thumbnail.png`
(`35bb4628…`); the upload sheet is `./x bayesian_search release-check`.

Video I's text: `explainer_corpus/visual_proofs/publish/michael/{title,description,tags}.txt` (v2's
chapter times, 0381fba); captions `explainer_corpus/visual_proofs/renders/final/visual_proofs.en.srt`
(`02862100…`, VTT `601a14f4…`: v3's, the same bytes as v2's, for no line moved); thumbnail `explainer_corpus/visual_proofs/renders/visual_proofs_thumbnail.png`
(`a78eb233…`); the upload sheet is `./x visual_proofs release-check`.

Video J's text: `explainer_corpus/matrix_exponential/publish/michael/{title,description,tags}.txt` (v2's
chapter times); captions `explainer_corpus/matrix_exponential/renders/final/matrix_exponential.en.srt`
(`7ca2d3cf…`, VTT `a2c8f666…`); thumbnail `explainer_corpus/matrix_exponential/renders/matrix_exponential_thumbnail.png`
(`634f318f…`); the upload sheet is `./x matrix_exponential release-check`.

## Corrected michael cuts (review 2 and later)

| Cut | Master | md5 | Length | Text | Captions (md5) | Thumbnail |
|---|---|---|---|---|---|---|
| Hoeffding michael v2 (uploaded, above) | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` | `76f66ca94c83b647a14d5f421c3d1cb2` | 9:04, −17.8 LUFS | `explainer_corpus/hoeffding_d/publish/michael/{title,description,tags}.txt` (v2 chapter times) | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding.en.srt` (`b643a09d…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` |
| fish michael v2 (HOLD; superseded by v3) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v2/capture_recapture_master.mp4` | `2296eb1379ffccd5228a84668a112ac8` | 18:08, −18.2 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/variants/michael/renders/final_v2/capture_recapture.en.srt` (`db3e5fc5…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| bakery michael, review 3 (uploaded, above) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `9ed06e7d12d38a87d748d8a58806d296` | 15:43, −17.6 LUFS | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` (chapter times updated, ca49d2f) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`522946ba…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` |
| fish michael v3 (uploaded, above; kept at `final_v3/`) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v3/capture_recapture_master.mp4` | `a6a990f53e0935f5a96d0441a647f216` | 18:12, −18.1 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` (v3 chapter times, d34de59) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v3/capture_recapture.en.srt` (`20620a55…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| fish michael v4 (Public, above) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` | `8a17579a3ad74962464bd2523192f556` | 18:12, −18.1 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` (the same chapter times as v3, d34de59) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture.en.srt` (`20620a55…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| RaptorQ michael, repair 12 (uploaded, above) | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `8e0afadf33bee181d59a81447d06fcb8` | 21:04, −18.0 LUFS | `explainer_corpus/raptorq_gf256/publish/michael/{title,description,tags}.txt` (chapter times of this cut, 5810c25) | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256.en.srt` (`3b8c3ebe…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` |

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
the final, and its two "I'm" tokens fall in the music-only coda. That is ASR, not hearing; the
owner's later "sound pretty good" covers this cut (above), and no full watch is recorded. The last 20.6 s (after 15:23.06) are music only. Review 2 is kept at
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
open for the owner's review with no recut of this hash (mail 1341); its fix landed as 8770672 and
is in v4, the published cut (the release set, above). The owner's later "sound pretty good" covers this
cut; no full watch is recorded.

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
all nine places (BronzeRidge, run 501e40e3-c53c-4fe5-bdad-bbe065a79f75, mail 1271). The owner's
later "sound pretty good" covers this cut; no full watch is recorded. v1 (`6898620c…`, superseded) is kept at `variants/michael/renders/final_v1/`.

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
"byte cards" reading is CoralForest's acoustic disposition (mail 1333). The owner's later "sound
pretty good" covers this cut; no full watch is recorded. Kept, never deleted: review 2 (`eda64ed9…`) at
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

The corrected michael cuts' text, captions and thumbnails (Hoeffding v2, bakery review 3, fish v3
and v4, RaptorQ repair 12) are in the corrected-cuts table above.

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
content "Yes" as applied on the michael review uploads (the narration is a synthetic voice, and
each description says so); English captions from the .srt "with timing"; the custom thumbnail; the
voice in the review title. A cut is done only with its video ID and processing state recorded.

For the public release the settings are the same, with Visibility Public and the plain title from
the release set above (no voice, no review number). A release is done only with its video ID, its
Public state, its captions and its 4K processing recorded here by the upload lane's receipt: done
for all four (RaptorQ, Hoeffding and the bakery video in mails 1368 and 1378; the fish video in
mails 1388 and 1389).
