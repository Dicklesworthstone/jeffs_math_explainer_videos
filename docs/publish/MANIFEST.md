# Publishing manifest: REVIEW ONLY (2026-10-12, 18:35 EDT)

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
| bakery michael, review 1 | https://youtu.be/PpmO9Gato2w | `explainer_corpus/bakery_algorithm/renders/final_review1/bakery_algorithm_master.mp4` | `19f2e859c0ee1a0841c0115ac69ab656` | superseded by review 2 (below) |
| fish matt (original) | https://youtu.be/fUXsjbWsgNg | `explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` | `528918af361165d6f73862d5fc5b9107` | kept; michael chosen instead |
| RaptorQ michael, review 1 | https://youtu.be/LXbcbfee_WE | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `16867253aba700f7f8602ef753cb49bb` | being corrected: "phones", "once", "those", "peels" lost their s; audit items |

When a corrected review 2 of the bakery video or RaptorQ is copied here, the review-1 files above
move to `renders/final_review1/` in the same crate (never deleted), and this table says so.

## Ready, not uploaded

| Cut | Master | md5 | Length | Status |
|---|---|---|---|---|
| RaptorQ leo (the original voice, a comparison) | `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256_master.mp4` | `130493a0d35ce4e4f61eeb39047b6b73` | 21:14, −16.0 LUFS | verified; the owner chose michael, so upload only if he asks to compare |
| fish michael, v1 (superseded by v2) | `explainer_corpus/capture_recapture/variants/michael/renders/final_v1/capture_recapture_master.mp4` | `3476fd0d155af2322d8f3597a0b66cfa` | 18:04, −18.1 LUFS | verified, but three endings ("size", "times", "things") lack their s: do not upload; v2 follows |

RaptorQ leo's text: `explainer_corpus/raptorq_gf256/publish/leo/{title,description,tags}.txt`;
captions `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256.en.srt`
(`96fc2e5c…`); thumbnail `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png`.

## Ready for review upload (review 2, michael)

| Cut | Master | md5 | Length | Text | Captions (md5) | Thumbnail |
|---|---|---|---|---|---|---|
| Hoeffding michael | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` | `6898620cebc142bc28292ec8df995a9d` | 9:04, −17.8 LUFS | `explainer_corpus/hoeffding_d/publish/michael/{title,description,tags}.txt` | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding.en.srt` (`d993b6f5…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` |
| fish michael v2 (HOLD: v3 follows) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` | `2296eb1379ffccd5228a84668a112ac8` | 18:08, −18.2 LUFS | `explainer_corpus/capture_recapture/publish/michael/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture.en.srt` (`db3e5fc5…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` |
| bakery michael, review 2 | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `7df3679f4f52d3e5844425fb7b868048` | 15:43, −17.6 LUFS | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` (chapter times updated, 9f743ed) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`7048a44c…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` |

Bakery review 2 evidence: VERIFY PASS (A/V 0.000 s, both decodes clean, the quiet-peak
exception above), music QA 13/13 after a re-score, cue check PASS (4 authored warnings), longest
chapter 84.8 s; "writes", "other's" and "Green's slate" fixed; Codex's fw pass on this exact master
transcribes all three correctly. Review 1 (uploaded https://youtu.be/PpmO9Gato2w) is kept at
`explainer_corpus/bakery_algorithm/renders/final_review1/` (md5 `19f2e859…`).

**Fish michael v2 is on hold; do not upload it.** The stricter final-hiss check (92d4953, ch and
sh too) flags 12_5 "size" and 13_6 "catch", and Codex's audit heard "letters", "cancels" and two
"catch"es weakly; its director is fixing them for a v3 (v2 will be kept at
`variants/michael/renders/final_v2/`).

Fish michael v2 evidence: VERIFY PASS (A/V 0.000 s, both decodes clean), music QA 15/15, cue
check PASS (13 warnings, all authored pauses or end holds, as in matt's cut); narration gates PASS
with 0 dropped final s/z and 0 cut short; "size", "times", "things" fixed; the audit's three fish
items checked in the takes ("Marked times", "five in a one", "pull the guess down": present).
Michael reads about 3% slower than matt, so chapters 04, 05 and 14 run 86-89 s, over our ~85 s
pacing guideline (not a render limit): for the owner's ear.

Hoeffding michael evidence (Codex's audit then found two candidates, "An exact twin" and "add a
quarter each"; its narration owner is checking them, and a v2 follows if a read changes; this cut
would then move to `variants/michael/renders/final_v1/`): VERIFY PASS (A/V 0.001 s, both decodes clean), music QA 8/8, cue check
PASS (one warning: the authored 5.2 s listening pause before s9, as in robert's cut); narration
gates PASS with 0 dropped final s/z and 0 cut short; "Hoeffding" read as HERF-ding in all nine
places and the possessives (checked by formants), spelled Hoeffding in the captions.

## In progress (review 2, michael)

| Cut | Lands at | Being fixed |
|---|---|---|
| RaptorQ michael | `explainer_corpus/raptorq_gf256/renders/final/` | phones, once, those, peels; fountain codes, ten to sixteen slots, parked pieces, the letter A |

Each gets its own `publish/michael/` text (the fish and Hoeffding variants: chapter times differ),
captions in its `renders/final/`, and the video's thumbnail. Added here when verified and copied.

## Text, captions and thumbnails of the uploaded cuts

| Cut | Text files | Captions (md5) | Thumbnail, 1280×720 (md5) |
|---|---|---|---|
| Hoeffding robert | `explainer_corpus/hoeffding_d/publish/robert/{title,description,tags}.txt` | `explainer_corpus/hoeffding_d/renders/final/hoeffding.en.srt` (`d8cb9445…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` (`568b42df…`) |
| bakery michael | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`27f50dfe…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` (`4be9ed65…`) |
| fish matt | `explainer_corpus/capture_recapture/publish/matt/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/renders/final/capture_recapture.en.srt` (`1f2df8ca…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` (`bce2cbc9…`) |
| RaptorQ michael | `explainer_corpus/raptorq_gf256/publish/michael/{title,description,tags}.txt` | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256.en.srt` (`cb8b968c…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` (`ab3727ca…`) |

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
