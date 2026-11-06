# Logarithm positioned-caption overlay on css: handoff

LavenderPike, 2026-11-06, for BlueSalmon (Mails 5762, 5765, 5793, 5823, 5856, 5859). One guarded job draws two caption
candidates on 40 real frames of the logarithm master, through ffmpeg's real `subtitles` filter. Root launches it. **Nothing
has run on css.**

**Proxy only.** Both candidates use libass at the first run's size. Neither shows YouTube's caption UI. No check here is an
overlap verdict: someone must view the sheets. **Neither candidate is called clear.**

## The two candidates

| | A, `candidate02.ass` | B, `candidate03.ass` |
|---|---|---|
| Rule | One top place, over the chapter title | Per-segment rectangles from the 2 fps maps, title counted as ink |
| Cues moved | 116, in 19 windows | The same 116, in 42 segments |
| Known residual | The title is crossed on every moved cue | 6 cues have no clean rectangle (25, 108, 109, 111, 245, 247). They use the least-ink place. 10 segments are rewrapped |
| Stable | Yes | No: the block moves around the frame between segments |

Both bind Bronze's sealed sidecars: SRT `b2914c6f…cb22`, VTT `a2fb0d69…4397`. Cue times and words are those files'. Candidate
B changes line breaks in the 10 rewrapped segments, and it exists only as a proxy. It has no VTT yet.

`candidate02.en.vtt` is candidate A as a positioned WebVTT, for a later private draft if root chooses it. It adds
`line:2% position:50% align:center` to 116 cues and changes nothing else. **Don't upload it on this evidence.**

## Run it

Root's staging folder from the first run is reused: `/data/projects/jeffs-math-logarithm-overlay-20261106` (S). Send this
whole folder to the same relative path under S. The master and both guard files should already be there; `check` names any
that are missing.

```sh
B=/data/projects/jeffs-math-logarithm-overlay-20261106/docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-caption-safe-overlay01
seal=<sha256 of SHA256SUMS, from the handoff mail>
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
/usr/bin/python3 -I -B "$B/overlay_css02.py" check --seal "$seal"     # read-only; prints font_sha256
tmux new-session -d -s log-overlay02 "$B/start-overlay-css02.sh 01 $seal <font_sha256>"
```

The job writes under `S/overlay-lane02`, apart from the first run. The first run took about a minute for 23 frames, so
expect a few minutes. A failed job stays in place; read its `command.log` and use run 02.

## Outputs, under `S/overlay-lane02/jobs/logarithm-safe-overlay-css-01`

| Path | What |
|---|---|
| `sheets/sheet-01.jpg` to `-14.jpg` | The sheets to view. Three frames each, A on the left and B on the right, quarter size |
| `overlay-A/`, `overlay-B/`, `clean/` | 40 full 4K PNGs each, for any close reading |
| `overlay-summary.json` | Every hash, plus each candidate's changed-pixel box |

The last log line is `POSITION OVERLAY PASS: sampled frames and sheets written; positioned libass captions are a proxy, not
YouTube UI; every sheet still needs actual viewing.`

## The 40 frames

`plan02.json` lists them in film order as F01 to F40, each with its origin.

| Origin | Frames | Why |
|---|---|---|
| `S01`–`S23` | 23 | The first run's exact frames, for a direct before and after |
| added | 13 | The 10 windows no first-run frame reached, and the worst cues 25, 111 and 245. Cues 109 and 247 are in both groups |
| control | 4 | Cues left at the bottom: 19 (pause prompt), 120 (thin axis), 338 (grid corner), 445 (panel edge) |

## Evidence

`checks/run_checks02.py`, output in `checks/outputs/run_checks02.txt`, on the final helper bytes.

| Real | Stand-in |
|---|---|
| Helper, guard, preserved guard, plan, both candidates, master, decoded frames | The `subtitles` filter: this host's ffmpeg has none, so a white box took its place |
| Exact frame seek, cue text against Bronze's sidecars | `sbh`, the font, ffmpeg's and Pillow's pins, local macOS tools |

## Not shown

- **ASS through the real filter is untried here.** The first run used an SRT. Candidate A's place is gated, so a wrong place
  stops the job. Candidate B's place has no gate.
- The header copies ffmpeg's default SRT style from memory of its source, not from a run. The bottom cues should match the
  first run's boxes, and that is worth checking in the summary.
- The maps behind both candidates are 2 fps samples of bright pixels at 960x540. A brief or dim element can be missed.
- YouTube's caption size, its top position, and its handling of `line:`, listening and independent AV acceptance all remain
  open.
