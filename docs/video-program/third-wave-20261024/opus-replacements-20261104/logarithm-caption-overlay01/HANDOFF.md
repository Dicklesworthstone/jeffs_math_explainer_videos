# Logarithm caption overlay on css: handoff

LavenderPike, 2026-11-06, for BlueSalmon (Mails 5673, 5691, 5692, 5701, 5726, 5736). One guarded job writes 23 sampled
frames of the logarithm master with the real SRT burned in by ffmpeg's `subtitles` filter, beside the same clean frames,
and 12 small sheets. Root launches it. **Nothing has run on css.**

**Proxy only.** This is default libass styling, not YouTube's caption UI. No check here is an overlap verdict: someone must
view the sheets.

## Run it

Root has made `/data/projects/jeffs-math-logarithm-overlay-20261106` (S) and is sending the four files in
`transfer-files01.json`. Send this whole folder to the same relative path under S, then on css:

```sh
B=/data/projects/jeffs-math-logarithm-overlay-20261106/docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-caption-overlay01
seal=<sha256 of SHA256SUMS, from the handoff mail>
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
/usr/bin/python3 -I -B "$B/overlay_css01.py" check --seal "$seal"     # read-only; prints font_sha256
tmux new-session -d -s log-overlay01 "$B/start-overlay-css01.sh 01 $seal <font_sha256>"
```

`check` also runs `sbh`, so a refusal uses no job name. A failed job stays in place; read its `command.log` and use run 02.

| Binding | Value |
|---|---|
| Master | `8534ec211790fb8400b809de19a6d1cb75af6150a5e90566f6a93bb90e7efd69` |
| SRT | `c9321cd7a2912d4457f2dc617735fd06f0b2fc7d684442815fa46b989bc31b7c` |
| ffmpeg, ffprobe | `/usr/bin/ffmpeg` `bdf6aabf…62f6`, `/usr/bin/ffprobe` `a6dac1e9…b2fe` (root's probe) |
| Font | `/usr/share/fonts/truetype/msttcorefonts/Arial.ttf`, which `fc-match Arial` selected. Hash passed at launch |
| Pillow, Python | 12.1.1, css `/usr/bin/python3` 3.14.4 under `-I` (root's probe) |
| Guard | `guard_css01.py`: the preserved guard, pinned by hash, new lane only. 10/8/4/56/20, as asserted and rechecked |

**Font choice:** Mail 5726 offered DejaVu or actual Arial. Arial keeps the styling default and needs no `force_style`,
which could not be tried here. libass's log line is checked at each sample, so another font stops the job at once.

## Outputs, all under `S/overlay-lane01`

| Path | What |
|---|---|
| `jobs/logarithm-overlay-css-01/sheets/sheet-01.jpg` to `-12.jpg` | The sheets to view. Lowest 760 pixels at half size, with and without the caption |
| `jobs/logarithm-overlay-css-01/overlay/`, `clean/` | 23 full 4K PNGs each |
| `jobs/logarithm-overlay-css-01/overlay-summary.json` | Every hash, plus each sample's changed-pixel box |
| `jobs/logarithm-overlay-css-01/logs/`, `command.log`, `resource-receipt.json` | Per-frame ffmpeg logs and the guard's record |
| `logarithm-overlay-css-01-postcheck.json`, `overlay-controller01.log`, `.exit` | Postcheck and controller records |

The rehearsal's stand-in job came to 27 MiB. Last log line: `OVERLAY PASS: sampled frames and sheets written; default libass
styling is a proxy, not YouTube UI; every sheet still needs actual viewing.`

## What is sampled

Measured on the delivery copy (`checks/measurements/`). Seconds; "fade" is first change to fully shown, or fade start to
last visible. Each sample sits where the footer is fully shown.

| Footer | Mail 5673 | Measured | Cues | Samples |
|---|---|---|---|---|
| F1 chapter 10, rows −2.93 and −3.40 | 1484.42–1497.62 | fade 1483.950–1484.300, cut after 1497.617 | 372–373 | S06–S07 |
| F2 Liouville, −3.40 | 1547.63–1566.10 | fade 1547.367–1547.700, fade 1565.700–1566.067 | 387–391 | S08–S12 |
| F3 Culver, −3.40 | 1720–1751 | fade 1719.567–1719.850, fade 1751.533–1751.883 | 430–433 | S13–S16 |
| F4 Gallier, −3.40 | 1766–1773 | fade 1762.267–1762.600, cut after 1773.450; red lines 1766.483–1766.800 | 435–437 | S17–S19 |
| F5 closing, −3.40 | 1930–1944 | fade 1929.667–1929.933; convention 1931.267–1931.533; family 1934.217–1934.483; last cue ends 1936.817 | 474–475 | S20–S23 |
| F6–F10 fine print, −3.62 | **in no window** | keyframes only, not frame-exact | 23, 90, 160, 287, 306 | S01–S05 |

- **Cue 433 is a new overlap.** It starts at 1751.417, while F3 is still fully shown, for 0.116 s before its fade.
- **F4 starts 3.7 s before the window.** Cue 435 overlaps it for nearly 4 s, not 0.47 s.
- **F6–F10 are added.** `draw::precisely` puts this fine print at y −3.62, lower than any row in the windows, in chapters 00,
  02, 04, 07 and 08. Each gets one sample, on the widest caption seen beside it, not one per cue. Ask if you want every cue.

F5 is split by its added lines, so cues 474 and 475 get two samples each. That is 18 samples in the windows and 5 added.

**Other low text, not sampled.** These are labels, not footnotes, from a source search at y −2.6 or lower. The ones that
state something: chapter 06 theorem at −3.55 (`lifting_loops.rs:209`), chapter 03 `KEEP_Y` at −3.55, chapter 00 "equal steps"
at −3.25 (shown by S01). Root decides whether these need samples.

## Evidence

`checks/run_checks01.py`, output in `checks/outputs/run_checks01.txt`: **32 checks, 0 failed**, on the final helper bytes.

| Real | Stand-in |
|---|---|
| Helper, `guard_css01.py`, preserved guard, plan, master, SRT, decoded frames | The `subtitles` filter: this host's ffmpeg has none, so a white box took its place |
| Exact frame seek (`pts_time` 1565.65 for frame 93939), footer ink in all 23 clean frames | `sbh`, the font, ffmpeg's and Pillow's pins, local macOS tools |

## Not shown

- **The filter has not run through this helper.** No caption was drawn by these checks. css's ffmpeg, `-copyts` with
  `subtitles`, and libass's font log line are first tried in the job.
- The footer-ink and caption gates are not a verdict, and a wrong refusal leaves the frames in place.
- The mac-mini-max ffmpeg was not used: its hash `1332dc2d…3327` matches this host's, and that build has no `subtitles`,
  `ass` or `drawtext` filter.
- Actual player seeks, YouTube's rendering, listening, fine sync and independent AV acceptance all remain open.
