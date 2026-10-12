# Publishing manifest: REVIEW ONLY (2026-10-12, 16:45 EDT)

**Nothing is public, and nothing may be published.** The owner will hear every cut before
anything is posted. He thinks he wants michael's voice everywhere but must hear it first. Upload
ready cuts as **Private** only, with titles or notes that say the voice, and give him review links.

Channel: Jeff's Math Explainers, https://www.youtube.com/channel/UCOcp8GDWZvkg-xu-U2aeppA
(confirmed by the owner).

Paths are relative to the repository root on the owner's Mac. Renders and captions are not in git;
the text files are. Every master listed as ready was rendered at 3840×2160, 60 fps, BT.709 and
passed `verify_video.sh` with the delivery gates; its md5 on this Mac equals the verified master's
on the build host. Originals are never overwritten: a new cut goes to its own path.

Settings for every review upload: Private; Education; "not made for kids"; altered or synthetic
content "No" (a generic synthetic narrator; each description says the narration is synthetic);
English captions from the .srt "with timing"; the custom thumbnail.

## The review set, by video and voice

| Video | Voice | Role | Status | Master | md5 | Length |
|---|---|---|---|---|---|---|
| Hoeffding's D | robert | original | ready; uploaded Private (Codex) | `explainer_corpus/hoeffding_d/renders/final/hoeffding_master.mp4` | `016f42a28ff497065dd9eb2a18a6f0a1` | 8:46 |
| Hoeffding's D | michael | comparison | narration and director pass running | `explainer_corpus/hoeffding_d/variants/michael/renders/final/hoeffding_master.mp4` (when ready) | | |
| The bakery algorithm | michael | original (its own voice) | ready | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm_master.mp4` | `19f2e859c0ee1a0841c0115ac69ab656` | 15:42 |
| The fish video | matt | original | ready | `explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` | `528918af361165d6f73862d5fc5b9107` | 17:37 |
| The fish video | michael | alternative | rendering (4K) | `explainer_corpus/capture_recapture/variants/michael/renders/final/capture_recapture_master.mp4` (when ready) | | |
| RaptorQ | michael | the owner's 11:30 choice ("replace the leo voice ... with the michael voice") | ready | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256_master.mp4` | `16867253aba700f7f8602ef753cb49bb` | 21:01 |
| RaptorQ | leo | the original voice, for comparison | ready | `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256_master.mp4` | `130493a0d35ce4e4f61eeb39047b6b73` | 21:14 |

Loudness: the bakery (−17.6 LUFS), fish matt (−16.8) and RaptorQ michael (−18.0) masters are
quieter than −16 because the fixed master gain stops when the true peak reaches −1.5 dBTP (the
designed behaviour for voices with sharp peaks; no compression). Hoeffding robert and RaptorQ leo are −16.0.

## Text, captions and thumbnails per ready cut

Title, description (with this cut's chapter times) and tags are plain text: copy them exactly.
For a private review upload, add the voice to the title (for example "… [michael]") so the owner
can tell the cuts apart; take it out again before anything is made public.

| Cut | Text files | Captions (md5) | Thumbnail, 1280×720 (md5) |
|---|---|---|---|
| Hoeffding robert | `explainer_corpus/hoeffding_d/publish/robert/{title,description,tags}.txt` | `explainer_corpus/hoeffding_d/renders/final/hoeffding.en.srt` (`d8cb9445…`) | `explainer_corpus/hoeffding_d/renders/hoeffding_thumbnail.png` (`568b42df…`) |
| bakery michael | `explainer_corpus/bakery_algorithm/publish/michael/{title,description,tags}.txt` | `explainer_corpus/bakery_algorithm/renders/final/bakery_algorithm.en.srt` (`27f50dfe…`) | `explainer_corpus/bakery_algorithm/renders/bakery_algorithm_thumbnail.png` (`4be9ed65…`) |
| fish matt | `explainer_corpus/capture_recapture/publish/matt/{title,description,tags}.txt` | `explainer_corpus/capture_recapture/renders/final/capture_recapture.en.srt` (`1f2df8ca…`) | `explainer_corpus/capture_recapture/renders/capture_recapture_thumbnail.png` (`bce2cbc9…`) |
| RaptorQ michael | `explainer_corpus/raptorq_gf256/publish/michael/{title,description,tags}.txt` | `explainer_corpus/raptorq_gf256/renders/final/raptorq_gf256.en.srt` (`cb8b968c…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` (`ab3727ca…`) |
| RaptorQ leo | `explainer_corpus/raptorq_gf256/publish/leo/{title,description,tags}.txt` | `explainer_corpus/raptorq_gf256/variants/leo/renders/final/raptorq_gf256.en.srt` (`96fc2e5c…`) | `explainer_corpus/raptorq_gf256/renders/raptorq_gf256_thumbnail.png` (`ab3727ca…`, the same pictures) |

Each variant gets its own `publish/<voice>/` text (its chapter times differ), its captions in its
`renders/final/`, and the video's thumbnail (the pictures do not depend on the voice). They are
added here when verified and copied.

## Preserved, never to be uploaded (last night's cuts, narration the owner rejected)

Moved aside, not deleted, on this Mac:

| Video | File | md5 |
|---|---|---|
| fish (matt, old narration) | `explainer_corpus/capture_recapture/renders/previous/capture_recapture_master.mp4.1012-154255` | `503dd043c44b3478aaf71f033f623e67` |
| RaptorQ (leo, old narration) | `explainer_corpus/raptorq_gf256/renders/previous/raptorq_gf256_master.mp4.1012-154306` | `d0f036f20f9d9201192bde2232f4dee8` |
| bakery (pre-final, old narration) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.065600` | `1defb58a302f07dcae069bcbe5402b05` |
| bakery (first rework, superseded) | `explainer_corpus/bakery_algorithm/renders/previous/bakery_algorithm_master.mp4.112412` | `395800dc9cce1bcb82f5cf4e006e98ea` |
| Hoeffding (first rework, superseded) | `explainer_corpus/hoeffding_d/renders/previous/hoeffding_master.mp4.112410` | `62c48a13695960bcba3c8ddde0187092` |

The build host keeps its own copies under each crate's `renders/final_before_rework/`.

## Uploads so far

| Cut | Video ID / URL | Visibility | Processing | Reported by |
|---|---|---|---|---|
| Hoeffding robert | (Codex to fill in) | Private | | Codex |

A cut is done only with its video ID and processing state recorded.
