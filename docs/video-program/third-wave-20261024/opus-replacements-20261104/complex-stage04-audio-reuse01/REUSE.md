# complex-stage04-audio-reuse01: which Oliver recordings stage04 can reuse

LavenderPike, for BlueSalmon. Bead `mx-video-program-program-87do.39`. Date 2026-11-05 (UTC). Narration is ElevenLabs Oliver only.

**State: read-only text and hash inventory.** I generated, selected, edited, moved or wrote over nothing, and ran no heavy job. A match here means the take's submitted text is byte-equal to the stage04 cue text. That is a source match: no take was heard, transcribed, aligned or judged, so a current take is still unaccepted. Root keeps synthesis, selection, TODO and Beads.

**Snapshot.** The labs are live. This inventory is the lab state at about 02:51Z on 2026-11-05. That includes root's `narration-request-stage04-01` (prepared 02:43Z) and root's `ui-takes-stage04-01` (01_1 submitted 02:45:30Z, downloads receipt 02:46:03Z). Root work after the seal makes `verify` print FAIL with a line diff of what changed. That diff is the record of the later work. It is not a fault in the seal.

## The script it is measured against

`narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-repair-stage04/`:
- receipt `combined-repair-stage04-integration01.json`, sha256 `295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b`
- `script.tsv`, sha256 `12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3`, which is stage03 plus review-fixes01. It has 105 cues with 105 distinct texts, 4,638 words in all.

## Answer

| Status | Cues | Words | Recordings |
|---|---|---|---|
| **current** | 2: `01_1`, `11_7` | 141 | 4: two takes of each |
| **pending-download** (submitted, no audio kept) | none | — | — |
| **missing** | 103: every other cue, `15_2` included | 4,497 | none |
| **superseded** | none of the 105 cues has an older recording of its own earlier wording | — | 38: every take in `elevenlabs-complex-analysis-20261101/raw/` |

**Do not regenerate `11_7` or `01_1`.** Each already has two takes of its exact stage04 text. Choosing between them, or rejecting both, is root's work after review. The remaining synthesis is the 103 missing cues.

**Current, `11_7`.** Both takes are in `counting-oliver02`. They were submitted from a request made for the stage03 script `1e24aba8…`. Stage04 changed only 15_2, so the text is byte-equal to stage04 11_7 (`a3d4e44c…`, 96 words).

| Take | sha256 | PCM sha256 | Length | Format | Peak sample |
|---|---|---|---|---|---|
| `11_7-download01.wav` | `fecd5218…` | `700b3ad4787ad8f1d3087fc45fcb27f8727cda7e36257061a738e45031d69e2e` | 1,843,200 frames, 38.4 s | 48 kHz mono 16-bit | 29,349 |
| `11_7-download02.wav` | `07e4fac7…` | `727badce4d56a57a6440e0dab37e2ee8596575e7d44d690ce5308e876695aed1` | 1,824,000 frames, 38.0 s | 48 kHz mono 16-bit | 30,968 |

**Current, `01_1`.** Both takes are in `ui-takes-stage04-01`, from a request made for the stage04 script `12442480…`. Its text is byte-equal to stage04 01_1 (`31681cbb…`, 45 words).

| Take | sha256 | PCM sha256 | Length | Format | Peak sample |
|---|---|---|---|---|---|
| `01_1-download01.wav` | `36f8459c…` | `eb943e970a8ce3d14a84e030e67fac22b23c62f99a8769c08cf995b5c26af00c` | 810,240 frames, 16.88 s | 48 kHz mono 16-bit | 30,612 |
| `01_1-download02.wav` | `790367af…` | `d2e4075d1cbfe22d9e32a303fd15b7a61098f266079e651dffaa124b4763da4f` | 821,760 frames, 17.12 s | 48 kHz mono 16-bit | 25,536 |

For both cues the two takes differ (their PCM is not equal). Each downloads receipt says `selection: null`, `listening: false` and `fine_sync: false`. Neither request carried neighbouring text, so no neighbour wording binds either one.

**Missing, 103 cues by chapter.** `inventory/cues.tsv` has each cue's text sha256 and word count.

| Chapter | Missing cues | Words |
|---|---|---|
| 01 | 01_2–01_6 | 219 |
| 02 | 02_1–02_6 | 214 |
| 03 | 03_1–03_7 | 276 |
| 04 | 04_1–04_7 | 248 |
| 05 | 05_1–05_9 | 386 |
| 06 | 06_1–06_9 | 411 |
| 07 | 07_1–07_6 | 244 |
| 08 | 08_1–08_5 | 231 |
| 09 | 09_1–09_9 | 387 |
| 10 | 10_1–10_6 | 276 |
| 11 | 11_1–11_6, 11_8 | 370 |
| 12 | 12_1–12_4 | 166 |
| 13 | 13_1–13_11 | 442 |
| 14 | 14_1–14_8 | 472 |
| 15 | 15_1–15_4 | 155 |

**Superseded, 38 recordings.** These are 19 submitted texts from the earlier eight-chapter complex film, two takes each, with receipts at the sibling lab's root. None of the texts equals a stage04 cue, and none contains a whole stage04 cue.

The closest any comes is one whole sentence: 12_3 shares one of its sentences with the old `04_2` text and with the long `remaining-draft` text. A shared sentence is not a take of the cue.

| Receipt (sibling root) | Takes | Words | Text begins |
|---|---|---|---|
| `01_1-distinct-takes-receipt01.json` | 2 | 76 | "Square a complex number and its distance from zero…" |
| `gallery-01_3-takes-receipt01.json` | 2 | 37 | "This rational map has a zero at minus one…" |
| `02_1-distinct-takes-receipt01.json` | 2 | 48 | "A complex derivative asks for one number…" |
| `02_2-distinct-takes-receipt01.json` | 2 | 63 | "There is a big difference between having that limit…" |
| `03_0-prediction-takes-receipt01.json` | 2 | 35 | "Imagine a holomorphic function on and around this disk…" |
| `03_1-distinct-takes-receipt01.json` | 2 | 44 | "Here is the surprise…" |
| `03_2-distinct-takes-receipt01.json` | 2 | 52 | "The result is far stronger than ordinary smoothness…" |
| `04_1-distinct-takes-receipt01.json` | 2 | 64 | "Walk once around a loop in the input plane…" |
| `04_2-distinct-takes-receipt01.json` | 2 | 52 | "The reason is a residue calculation…" |
| `05_1-distinct-takes-receipt01.json` | 2 | 48 | "Here is a real question: what is the area…" |
| `05_2-distinct-takes-receipt01.json` | 2 | 28 | "There are two simple poles inside…" |
| `05_3-distinct-takes-receipt01.json` | 2 | 41 | "Their sum is minus i times the square root of two…" |
| `06_0-prediction-takes-receipt01.json` | 2 | 38 | "Consider the growing sum one plus z plus z squared…" |
| `06_1-distinct-takes-receipt01.json` | 2 | 59 | "A power series can give us a function only inside…" |
| `06_2-distinct-takes-receipt01.json` | 2 | 47 | "Sometimes following continuations along different paths…" |
| `07_1-distinct-takes-receipt01.json` | 2 | 52 | "A complex derivative asks every direction to agree…" |
| `additional-phase-examples-takes-receipt01.json` | 2 | 35 | "A cubic such as z cubed minus one has three zeros…" |
| `opening-takes-receipt01.json` | 2 | 51 | "A real function takes a number on a line…" |
| `remaining-draft-takes-receipt01.json` | 2 | 674 | "Square a complex number…" (one long take of the old script, 281.5 s) |

**Do not match by file name.** The sibling's cue ids are the old film's numbering, so the same id names different words. The sibling's `01_1` begins "Square a complex number…", while stage04's 01_1 begins "Here is a perfectly smooth curve. One over one plus x squared…". The inventory matches by text bytes only.

## Prepared-only folders: text, not generation

Two folders in the rebuild lab hold request texts and nothing else. **They are prepared-only. They record no submission, and nothing in them was generated.** The inventory decides this from each folder's own requests file. The file's status must say text preparation with no synthesis, and no request in it may be flagged as submitted. Every file under such a folder is counted as prepared text and never as a service or generation record. That is 208 files, all 105 cues.

| Folder | Made for script | Texts | Byte-equal to stage04 | Submitted flags | Its status, verbatim |
|---|---|---|---|---|---|
| `narration-request-stage04-01` (root, 02:43Z) | `12442480…` (stage04) | 105 | 105 | all `false` | "Exact text preparation only. No synthesis. Pending sealed reuse inventory establishes already recorded11_7; never blindly regenerate. Stage04 binder remains in progress." |
| `narration-request-preparation01` | `2968b0fd…` | 105 | 103 (differ: 11_7, 15_2) | none stated | "prepared only; no synthesis or take selection" |

The stage04 folder's `matches_preparation01` claims hold against the files: each one is true exactly where the two folders' texts are byte-equal.

## What root's synthesis should take its text from

- **`narration-request-stage04-01/`**: all 105 texts are byte-equal to the stage04 script. Send the 103 missing cues from it. Skip `01_1` and `11_7`, which already have current takes.
- **The stage04 `script.tsv`**, which gives the same bytes. `cues.tsv` gives each cue's sha256, for checking a request before it is sent.
- **Not `narration-request-preparation01/`** for 15_2. Its `15_2.txt` holds the clause F3 replaced. Its `11_7.txt` holds the old six-sentence 11_7. The other 103 still match.
- **Not the stage04 plan as it stands.** `combined-repair-stage04/.../narration_units.json` (`3c4f53d7…`) still pins script `2968b0fd…`, and its unit text differs from the script at 11_7 and 15_2. A run driven from that plan would send the old 15_2 clause. This is the F1 binding gap; BronzeJaguar is binding it.

## Provenance checks

- **Receipts against files.** All 42 recordings are named by exactly one receipt, and every receipt record has its file on disk. That is 21 receipts in all:
  - 19 take receipts in the sibling lab
  - the `11_7` downloads receipt
  - the `01_1` downloads receipt

  The two downloads receipts each name their request by hash. All 206 fields the receipts state agree with the files: sha256, frames, rate, width, channels and PCM sha256.
- **The PCM hash method.** The PCM sha256 here is the sha256 of the WAV data frames, read with Python's `wave`. It reproduces the `pcm_sha256` of all four takes in the rebuild lab's two downloads receipts.
- **Duplicates.** No take pair is byte-identical PCM.
- **Source texts.** Where a receipt names a source text file, the file equals the submitted text. The opening receipt names none. For 11_7 and 01_1, the cue's `.txt`, the request's `text` and its `text_sha256` all agree.
- **Submissions.** The rebuild lab has one submission record: `ui-takes-stage04-01/01_1-submission01.json`. It names the 01_1 request by hash, and that request has two retained recordings, so no stage04 cue is pending download.
- **Generations with no audio on disk.** Every other text record in both labs was scanned for an exact stage04 cue text. The records are classed as follows:

  | Class | Files | Cues | What it means |
  |---|---|---|---|
  | Recording records | 4 | 2 | The requests and `.txt` beside the 01_1 and 11_7 takes |
  | Prepared-only | 208 | 105 | The two folders above |
  | Service records with no retained recording | 0 | — | — |
  | Plans, maps and reviews | 68 | 103 | Hold cue texts but are not service records |

  So no record shows a stage04 cue generated but not kept.
- **Download-name tokens.** These are recorded, not interpreted. Every receipt says stability 0.5 and similarity 0.75. The download names carry different tokens:
  - 32 sibling takes: `pvc_sp100_s100_sb100_v4`
  - 6 sibling takes (opening, 03_0, remaining-draft): `s50_v4`
  - the four 11_7 and 01_1 takes: `pvc_s50_sim75_v4`, which agrees with their requests
- **Derived audio.** `five-film-release-20261027` holds 174 complex audio files (730.1 MB): decodes, assemblies, soundtracks, music stems, a master and ASR cuts of the old film. 126 are 24 kHz WAV, 34 are 48 kHz and 4 are 16 kHz. Python's `wave` can't read the other 9 as PCM: one master WAV and eight `.f32` music stems, which are hashed by file only. None of the 174 equals a recording by file, and none of the 165 readable ones equals one by PCM. `inventory/derived-audio.tsv` lists each with its sha256. They are products of the old takes, not recordings, so none can be a stage04 take.
- **Audio no receipt names.** The build hashes any audio file no receipt names and lists it as `no-receipt`, never current. At the snapshot there is none.
- **Preservation.** The run read 1,119 files, opening each `rb`. It stat-ed each before and after, and none changed. It writes only into its output folder and refuses a path inside `narration_lab`.

## Rerun

From the repo root:

```sh
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage04-audio-reuse01/checks/audio_reuse.py verify <fresh-scratch>
```

It rebuilds the four inventory files in the scratch folder and reports whether each is byte-equal to `inventory/`. For a changed file other than `recordings.json` it prints a line diff, so later root takes show up as new `current` rows. The build asserts the stage04 receipt and script hashes, so a moved script stops the rerun instead of being inventoried silently. `inputs.sha256` checks the key inputs with `shasum -a 256 -c` from the repo root.

## Limits

- **Text, not speech.** Byte-equal submitted text is not proof of what a take says. Each 01_1 and 11_7 take still needs root's audio review (listening and alignment), selection and fine sync. The peak sample is a number, not a judgement of loudness or clipping.
- **Local records only.** I didn't open the ElevenLabs history or any download folder outside the repo. A generation never recorded in the labs would not show here. The 01_1 request carries root's own `history_check` note, which I report but did not repeat.
- **Exact only.** Cues whose old wording was close to the new are still missing. Whether to cut a sentence out of an old take is a synthesis decision for root, and this inventory doesn't propose one.
- **A snapshot.** Counts hold at the snapshot above. A later rerun is the way to see root's later takes.

## Files

- `REUSE.md`: this file.
- `checks/audio_reuse.py`: `build` and `verify`.
- `inventory/summary.txt`: the build's own report, which every number above comes from.
- `inventory/cues.tsv`: the 105 cues. Each row has:
  - status, words and text sha256
  - current recordings, with a PCM sha256 prefix
  - the cue's status in `narration-request-preparation01`, in `narration-request-stage04-01` and in the stage04 plan
  - notes
- `inventory/recordings.json`: the 42 recordings. Each has:
  - its receipt and settings
  - file sha256, PCM sha256, format, length and peak
  - the check of each receipt field
  - its submitted text and text sha256
  - its stage04 match and shared sentences
- `inventory/derived-audio.tsv`: the 174 derived files.
- `inputs.sha256`: the key inputs, repo-relative.
- `SHA256SUMS`: this bundle.
