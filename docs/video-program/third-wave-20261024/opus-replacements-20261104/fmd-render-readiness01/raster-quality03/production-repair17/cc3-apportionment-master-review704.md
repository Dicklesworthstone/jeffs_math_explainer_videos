# Independent review 704: apportionment master700, verify701 and controller702

Reviewer: CC3 (WhiteSwan). Compared against CC0's 686/687/689 and root's `root-master-preflight703.json`.

Scope: source reading and read-only checks of local files. Nothing was run on the build host, and I did not run any of these helpers. I saw no build-host output, so every statement about what happens there is reasoning from source. This review accepts no picture, motion, audio, listening, fine-sync, AV or release result.

Independence: I wrote none of these helpers. CC0 wrote the 686 family and root wrote the 700 family. I read `handoff688.md` for its stated limits and root's 703 for its claims, and tested those claims against the source where the source allows it.

| File | sha256 (computed by me) |
|---|---|
| `apportionment-csd-master700.py` | `e589f7f3302f9d97a9ad5b6b6f244ea3fccac71ffb91f5364f83fe555575743d` |
| `apportionment-csd-verify701.py` | `02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f` |
| `controller702.py` | `01ebfee47d86a5bba5df26956da7e7519eddc453aad0ee1c4329370d237ab446` |
| `root-master-preflight703.json` | `32e18f57a3b37a6c826786be38b7bc0af24c1ffc4dcdb4e686b0c974786f59e3` |
| `apportionment-csd-master686.py` | `f1dde8735fac0bcd7075030a2e74c1cccdc0e295cbc1e57265ca69ceb680857b` |
| `apportionment-csd-verify687.py` | `8920ba67279a8fa6d614c71125c3c4ebbee13e4e0a003328aedceef5fb6ea43f` |
| `controller689.py` | `15135743773db70ab0207214c9824d9f416c8638d9100ceeb063d4e64da2b479` |
| `handoff688.md` | `815fc99004ee3d30d9ca1506041166be04c96763b1ef8bdce6ebe65178b5eb39` |
| `apportionment-csd-support671.py` | `244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a` (equals 700's SUPPORT_SHA) |

Also read: `apportionment-local-finals-export682.json`, `apportionment-after-timeline673.py`, `apportionment-after-assembly667.py`, `crates/explainer_kit/src/master.rs` and `cli.rs`, `tools/narration_delivery.py`, and `crates/fmn-output/src/sound.rs` and `audio.rs` at engine pin a399d62.

## Result

- I found no execution or correctness defect in the 700/701/702 delta. A byte diff shows 686 to 700 changes only version names and `imported()` at 700:147-152. 687 to 701 and 689 to 702 change only names and pins. 702's pins equal my hashes of 700 and 701, and POLICY is unchanged in 700 and 701.
- Root's narrowing is sound. All 4005 rows are still matched to the pinned export, and no protection is lost (details under Checked).
- Three runtime assumptions are unmeasured: F1, F2 and F3. All three fail closed, so none can produce a wrong master that passes. They can cost hours or stop the run late.
- 702 launches master700 as soon as the last final672 finishes, with no gate in between. Any pretest below has to happen while it waits. 702 can keep waiting; nothing here calls for stopping it unless a pretest goes the wrong way.

## Findings, most important first

### F1. Medium, open evidence: cross-host bit-exact replay of chapters 01/02 has not been measured

Chapters 01/02 carry a soundtrack mixed on the Mac (arm64). The kit will replay them with the build-host binary (x86-64) and require byte equality at `master.rs:126` (`null_test`).

The source supports equality. The replay decoder is native PCM WAV only, so no FFmpeg build enters it (`master.rs:94` passes `None`; `audio.rs:222-223`), and all 484 audio files in the export are `.wav`. The mixer at a399d62 uses `fmn_dmath` for gain and resampler trigonometry (`sound.rs:791, 865-866, 872`), documents that parallel work splits only the output timeline, and I found no `mul_add` or fast-math in it. Differing thread counts are covered by its test at `sound.rs:1568`. That is reasoning about source. Cue logs for 01/02 hold 13 narration rows at 24 kHz mono and 2 rows at 48 kHz stereo, all at 0 dB gain, so the 24 kHz rows exercise the resampler and that arithmetic is what crosses hosts.

Local evidence is Mac-only. The three soundtrack hashes in `handoff688.md` equal my Mac preview569 `soundtrack_0.wav` hashes, so they do not show cross-host equality:

| Chapter | Mac preview569 `soundtrack_0.wav` sha256 |
|---|---|
| 08_signposts | `4e7ad48ecf33171acc3423cb9344cb48254a28a57434f231ade8af68ee6b47b3` |
| 13_population | `72eb33a9323f4b68a95431523035fda936f450d752af27b5bad02cbec0d6b509` |
| 15_recap | `ff9f83e4174c8d00e6ae593026bdda3b6609344b291682e71ed2991a14f58421` |

Root action: compare these against `result.files['soundtrack/<chapter>/soundtrack_0.wav']` in the build host's `final672-summary.json` for 08, and for 13 and 15 once they finish. This costs one file read and is the only cross-host audio evidence available before the master runs.

If the mixers do differ, the kit stops at chapter 01's null test, before join or any master file. The failure is cheap but needs new job names and a decision (for example re-rendering 01/02 on the build host).

### F2. Medium: nothing compares avcC/SPS/PPS across the 15 chapters before the stream-copy join

700's `options()` (700:121-128) compares only the text after ` - options: `. The x264 core banner is exempt by design, and chapters come from two x264/FFmpeg builds. The join is a concat with `-c:v copy`, so the output carries the first chapter's parameter sets. Equal option strings make equal decode-critical fields likely, but no step checks the bytes, and the kit's gates (frame count, audio length, timestamp grid) would not notice a difference.

If the parameter sets differ in a way that matters, verify701's frame-hash equality (701:262-263) or `-xerror` catches it. That happens only after the master job and a full decode, so a wrong file would not pass; the cost is hours. I have not observed any such difference. This is a gap in the checks, and no defect has been seen.

Root action, read-only on the build host, for all 15 chapter mp4s (the command is from memory of ffprobe options, and I have not run it on 8.0.1):

`ffprobe -v error -select_streams v:0 -show_entries stream=profile,level,extradata_size,extradata_hash -show_data_hash SHA256 -of csv=p=0 <chapter>.mp4`

Equal rows close F2. If only 01/02 differ from 03-15, root should judge which fields differ before the master launches.

### F3. Low-Medium: the framehash row form and the step time are untested on the build host's FFmpeg 8.0.1

`frame_digests` (701:112-122) requires six columns, stream 0 and `int(row[3]) == 1`. The first parse happens at 701:259, after the full master decode at 701:251-253 and the chapter 01 hash. If FFmpeg writes a different duration or row form, 701 refuses after hours of work. The master itself would stay valid and only verify would need a new name. The duration check adds little beyond the length and list equality at 701:263. My expectation that the output timebase is 1/60 with `-fps_mode passthrough` comes from memory of FFmpeg's source and has not been tested.

The master decode step hashes about 1.255 TB (100,886 × 12,441,600 bytes) in one step with a 14,400 s limit, so it needs at least about 87 MB/s through FFmpeg's sha256, sustained under whatever else the host is running. The 15 chapter steps hash the same volume again under their own limits.

Root action, optional: time a read-only chapter 01 framehash into scratch on the build host with the same arguments as 701:257-258. Rows should parse with duration 1. A run under about 686 s (4,806 frames) implies the master step fits.

### F4. Low, record only: 703's description of the 2625 external paths

703 calls them external Cargo/engine support. In the export they are 2,565 Cargo git checkout files, 59 files in the local downloads folder (the ones I listed are ElevenLabs Frederick takes), and 1 other. Execution is unaffected, because these rows are only hash-checked. The record should say so if anyone later reads "support" as code only.

### F5. Low, not established: the guard itself is unreviewed

`guard_csd02.py` has no local copy, so I could not read it. 700 records its hash and 702 calls it as `--phase native --job <name> --inputs <file> -- <python> -I -B <helper> run`. I confirmed only that this call shape and POLICY (10/8/4/56/20/10) are unchanged from 686/687/689 and that POLICY equals final672's assert. Root's statement that the guard is unchanged is root's.

## Checked, no finding

- Narrowing at 700:147-152: all 4005 rows are still matched row by row to pinned export682 at 700:142-145 (38 + 3967; 1380 inside the repository prefix, 2625 outside). Outside rows are used only for hash checks. `rebase_path` (700:163-165) still asserts the prefix, so a table path outside it cannot be rebased.
- Rebasing: only the path column changes. Timeline must equal timeline666 byte for byte. Cue-log non-stem rows must equal timeline666's including the digest column, and stem rows must match hash and start sample. `assert local not in result` then holds. The narration and stem bytes at rebased paths are proven when inputs are frozen.
- Stems in `rows` (700:332-333): final672's manifests extend timeline666's (`apportionment-after-timeline673.py:21-28`), and timeline666's inputs include every file under the relocated repo by `rglob` (`apportionment-after-assembly667.py:30`). I read the source for this; no `*-inputs.json` manifest exists locally.
- Capture key: both 01/02 captures are `soundtrack/<chapter>/soundtrack_0.wav`, so `materialize` and `got['files'][capture]` agree.
- Replay evidence (700:433-435): the kit prints `{:<14}` then a space, so `split()[0]` is the chapter key, and the assertion requires all chapters in registry order.
- `captions all` without `--score` is correct, because `cue_log` runs only for consumer `master` (`narration_delivery.py:328`). Names `apportionment.en.srt/.vtt` match `cli.rs:925-933`.
- 702 names agree with the helpers (`INPUTS`, `JOB`, `master700-summary.json`, `verification701.json`). It refuses duplicates, does not retry, waits 24 h per receipt, and the guard receipts in this directory carry the `preflight_only` key that `terminal()` requires.
- Disk: the imported mp4s are small (5,089,761 and 8,444,934 bytes). Handoff688 estimates 1.5 to 2.5 GiB of growth against the 4 GiB phase cap; I did not measure it.

## Exact steps for root

Do these while 702 waits. Each is read-only and needs no change to the sealed helpers.

1. F1: compare the 08 soundtrack hash with the table above, then 13 and 15 when they finish.
2. F2: run the ffprobe comparison over the available chapters, then over all 15 once the finals finish, if that can happen before 702 proceeds.
3. F3, optional: time and parse one chapter 01 framehash in scratch.

| Observation | Meaning | Root's move |
|---|---|---|
| 08 soundtrack equal | Positive cross-host evidence | Continue |
| 08 soundtrack unequal | Mixers or inputs differ (ambiguous) | Decide before the finals complete; stopping 702 means new job names |
| avcC rows all equal | F2 closed | Continue |
| avcC rows differ | Stream-copy join may be unsafe | Inspect fields; re-rendering 01/02 on the build host would also settle F1 |
| Framehash row is not six columns or duration is not 1 | 701 would refuse after the decode | Stop 702 and prepare a new verify name; master700 needs no change |

## Not established here

- Whether any helper runs successfully on the build host, or how long it takes.
- Cross-host audio equality, parameter-set equality and framehash row form, as above.
- `guard_csd02.py` behaviour.
- Picture, motion, audio, listening, fine-sync, independent AV or release acceptance. Passing 700/701/702 would be technical evidence only, as 701's own `pictures_viewed=False`, `audio_heard=False`, `fine_sync=False` and `independent_av_acceptance=False` fields state.
