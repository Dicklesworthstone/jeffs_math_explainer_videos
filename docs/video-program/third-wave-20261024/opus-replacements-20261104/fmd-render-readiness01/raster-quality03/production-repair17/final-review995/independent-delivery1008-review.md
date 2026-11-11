# complex-delivery1008: independent cross-file consistency check

Reviewer: CC3 (WhiteSwan). Date: 2026-11-10. Read-only, local files only. I ran no decode, render, synthesis, upload
or Beads edit. This is a consistency check, not a new gate. It is not acceptance of final pictures, listening, AV
or publication.

Package: narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-delivery1008. The manifest
delivery-manifest1008.json is 1df646f2.

Check script: delivery1008-check.py in my scratchpad, run with `python3 -I -B`. Its first run stopped on the `#`
header line of script.tsv. I fixed the parser to skip comment lines and reran it.

## Verdict

No packaging blocker was found. There are two low record and hygiene notes, N1 and N2.

## What agrees

**Manifest against files.** All 7 rows match by sha256 and by bytes, and each row's `source` file has the same sha.
All 4 evidence rows match their files.

| File | sha256 | Bytes |
|---|---|---|
| complex_analysis_master.mp4 | 80244e88… | 219661850 |
| complex_analysis.en.srt | 0ea9277e… | 48567 |
| complex_analysis.en.vtt | 6a383cb7… | 46367 |
| title.txt | 48a242e3… | 41 |
| description.txt | c78723f3… | 2696 |
| tags.txt | 40e49939… | 367 |
| thumbnail.png | e5dfc2bd… | 679205 |

**Master against completed-verification995 (8b56250a).**
- The movie sha equals `master_sha256` 80244e88.
- Frames are 133662 in both the manifest and 995.
- The duration is 2227.7 s, which is 133662/60.
- The 995 receipt (501e6a89) records admitted True, exit 0, error and both stop fields null, and inputs equal to
  final inputs.

**Caption pair against 995.**
- The SRT and VTT equal 995's `delivery_captions_347` hashes (0ea9277e / 6a383cb7).
- There are 579 blocks, matching 995 `exact_check` and the manifest's `delivery_caption_blocks`. This is not the
  native 571 pair, 5d06423248f0 / 26e0d7e0eec2.

My own parser found:
- SRT rows equal VTT rows, and the SRT indices run 1..579;
- at most 2 lines and at most 42 characters per line;
- no caption over 20 cps;
- every caption in order and inside the video, with the last end at 2220147 ms against 2227700 ms of video.

The caption words equal, word for word, production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/script.tsv
(234f7257, 105 cues, 5395 words). They do not equal the repo-root script (see N1).

**Chapter times against the actual 993 chapters.**
- The description has 15 stamps, and 995 has 15 chapters that are contiguous from frame 0 to 133662.
- Each stamp equals floor(start_frame/60) exactly. The stamps lead the actual starts by 0.083 to 0.850 s, the largest
  at 11_counting (1516.850 s, stamped 25:16).
- The titles equal publication-chapters-checked993.json (6ef775a5), which binds the same description sha.
- YouTube rules hold: 0:00 first, ascending, smallest gap 66 s, and the last chapter is 73.0 s long.

**Copy limits.**
- The title is 40 characters.
- The description is 2695 characters (2696 bytes).
- Tags are 366 characters across 18 tags.
- There are no angle brackets and no em dashes.

**Links in the description.**
- The sister video is https://youtu.be/lgpTDROHgQY.
- The four DLMF pages are 1.9, 1.10, 5.2 and 25.2.

**Thumbnail.** A valid PNG, 3840x2160, 679205 bytes (under 2 MB).

**Original reference.**
- The manifest's `original_youtube_id` is K9r71UhRr4g. The canonical TODO identifies that id as the old complex
  upload, now "listed Public 6:30/26 views".
- The title, description and tags do not mention it.
- The manifest's `remaining` puts the original's unlisting after the uploaded 4K and English checks.

**Voice label.** The label is the same everywhere it appears:
- the manifest's `narration` is "ElevenLabs Frederick Surrey";
- the description says "Narration: ElevenLabs Frederick Surrey, a synthetic voice.";
- the preserved214 script header and the narration_units scope both say Frederick Surrey;
- prepare-assembly19.py lines 181 to 184 require the selected receipt to be "ElevenLabs UI, Frederick Surrey" with
  voice "Frederick Surrey - Smooth and Velvety" for the two replaced cues. 995 records the retained takes as byte
  identical to rebind269 and assembly19 narration.

## Notes (not blockers)

**N1 (low, record).** The repo's `explainer_corpus/complex_analysis_visual/script.tsv` (2968b0fd, last commit 2fb1c0c7)
is not the text that shipped:
- its header still reads "narrator ElevenLabs Oliver, unsynthesized";
- it has 4623 words against the captions' 5395, and a SequenceMatcher ratio of 0.910 against the caption words.

The released words are the preserved214 workspace script 234f7257. 995 bound its CSS copy
(work/review-repair268/workspace), and that matched review-repair313's. The release record should name the 268/313
script, which equals 234f7257 word for word. Any post-release caption or copy check must not use the repo-root
script. I did not compare the CSS 268 file's sha with 234f7257; only the word streams were compared.

**N2 (low, hygiene, only if committed).** delivery-manifest1008.json holds absolute local home-directory paths. The
package directory is untracked and not ignored. The mp4 and png are ignored by `*.mp4` and `*.png`, but the json,
txt, srt and vtt files would enter a commit if the directory were staged. Before any commit, make those paths
repo-relative, or keep the package out of git.

## Not established here

- **Voice of the 103 retained cues.** They come from csd-continuous-rebind16. The units carry no per-cue voice field,
  and I did not re-derive that chain. The two replaced cues are bound to Frederick by receipt.
- **The sister link.** I did not check lgpTDROHgQY's current public state online.
- **Everything 1008 itself lists as remaining:** final pictures, motion, listening, fine sync, caption playback,
  independent AV, and uploaded verification.
