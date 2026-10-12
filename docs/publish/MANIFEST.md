# Publishing manifest (2026-10-12, 15:55 EDT)

Channel: Jeff's Math Explainers, https://www.youtube.com/channel/UCOcp8GDWZvkg-xu-U2aeppA
(confirmed by the owner). Nothing has been uploaded yet.

Paths are relative to the repository root on the owner's Mac. Renders and captions
are not in git (they are .gitignored); the text files are. Every master below was rendered at
3840×2160, 60 fps, BT.709, and passed `verify_video.sh` with the delivery gates (A/V within one
AAC frame, full software and hardware decode clean); its md5 on this Mac equals the verified
master's on the build host.

Visibility: **public** (the owner's standing instruction). Category Education; audience "not made
for kids"; altered or synthetic content "No" (a generic synthetic narrator; every description
says "Narration: synthetic speech (FrankenTTS)"). Captions: English, upload the .srt "with timing".

## Publish now: three originals, ready

| # | Video (voice) | Master | md5 | Length | Loudness |
|---|---|---|---|---|---|
| 1 | Hoeffding's D (robert) | `explainer_corpus/hoeffding_d/renders/final/hoeffding_master.mp4` | `016f42a28ff497065dd9eb2a18a6f0a1` | 8:46 | −16.0 LUFS |
| 2 | The bakery algorithm (michael, its own voice) | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `19f2e859c0ee1a0841c0115ac69ab656` | 15:42 | −17.6 LUFS ¹ |
| 3 | The fish video (matt, the original) | `explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` | `528918af361165d6f73862d5fc5b9107` | 17:37 | −16.8 LUFS ¹ |

¹ Quieter than −16 because the fixed master gain stops when the true peak reaches −1.5 dBTP (the
designed behaviour for voices with sharp peaks; no compression). The bakery video's integration
lane asked for the owner's acceptance of this.

Files for each (title, description and tags are plain text, copy them exactly):

| # | Title | Description | Tags | Captions | Thumbnail (1280×720) |
|---|---|---|---|---|---|
| 1 | `explainer_corpus/hoeffding_d/publish/robert/title.txt` | `.../publish/robert/description.txt` | `.../publish/robert/tags.txt` | `explainer_corpus/hoeffding_d/renders/final/hoeffding.en.srt` (`d8cb9445…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` (`568b42df…`) |
| 2 | `explainer_corpus/bakery_algorithm/publish/michael/title.txt` | `.../publish/michael/description.txt` | `.../publish/michael/tags.txt` | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`27f50dfe…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` (`4be9ed65…`) |
| 3 | `explainer_corpus/capture_recapture/publish/matt/title.txt` | `.../publish/matt/description.txt` | `.../publish/matt/tags.txt` | `explainer_corpus/capture_recapture/renders/final/capture_recapture.en.srt` (`1f2df8ca…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` (`bce2cbc9…`) |

## RaptorQ: one decision for the owner before it goes up

"The Arithmetic Where 3 × 7 = 9" exists in two voices. Publish one of them as the RaptorQ video:

- **michael**, ready: `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4`, md5
  `16867253aba700f7f8602ef753cb49bb`, 21:01, −18.0 LUFS ¹. Text: `explainer_corpus/raptorq_gf256/publish/michael/`
  (title, description with this cut's chapter times, tags). Captions
  `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256.en.srt` (`cb8b968c…`); thumbnail
  `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` (`ab3727ca…`, voice-independent).
  The owner asked at about 11:30 to "replace the leo voice in the raptorq video with the michael voice";
  this file sits at the video's main path for that reason.
- **leo**, the original voice, rendering now from its reworked and director-approved narration:
  it lands at `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256_master.mp4`
  with its own captions and chapter times (`publish/leo/` follows). Not ready.

Until the owner chooses, upload neither. Publishing both would need a distinct title for one.

## Alternative, not for publishing unless the owner asks

- The fish video in michael's voice (an alternative; matt's is the original and is the one to
  publish): rendering now into `explainer_corpus/capture_recapture/variants/michael/renders/final/`.
  Its narration passes every gate with its own lexicon (michael says "Feller" without the r).
  Not ready; if the owner wants both versions published, it needs a distinct title.

## Preserved, never to be uploaded (last night's cuts, narration the owner rejected)

All moved aside, not deleted, on this Mac:

| Video | File | md5 |
|---|---|---|
| fish (matt, old narration) | `explainer_corpus/capture_recapture/renders/previous/capture_recapture_master.mp4.1012-154255` | `503dd043c44b3478aaf71f033f623e67` |
| RaptorQ (leo, old narration) | `explainer_corpus/raptorq_gf256/renders/previous/raptorq_gf256_master.mp4.1012-154306` | `d0f036f20f9d9201192bde2232f4dee8` |
| bakery (pre-final, old narration) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.065600` | `1defb58a302f07dcae069bcbe5402b05` |
| bakery (first rework, superseded) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.112412` | `395800dc9cce1bcb82f5cf4e006e98ea` |
| Hoeffding (first rework, superseded) | `explainer_corpus/hoeffding_d/renders/previous/hoeffding_master.mp4.112410` | `62c48a13695960bcba3c8ddde0187092` |

The build host keeps its own copies under each crate's `renders/final_before_rework/`.

## Channel page

`docs/publish/channel.md`: the channel description and keywords; link "GitHub" to
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos.

## After each upload

Record the video URL or ID and its processing state; a video is done only with both. Check on its
watch page that the chapters appear, captions list English, and the custom thumbnail shows.
