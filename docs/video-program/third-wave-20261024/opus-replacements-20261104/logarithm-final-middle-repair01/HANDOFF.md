# Logarithm final middle repair 01

BronzeJaguar, 2026-11-05, for BlueSalmon. Picture-only patches for O1, O2 and O3 of
`logarithm-final-review-pane168-01`, with the 13-site spacing audit and U1 evidence.

**Status: staged only.** Not applied, not compiled, not rendered, not heard. The frozen workspace,
canonical files, `script.tsv`, Beads, TODO and memory are untouched. Root has read the three patches
and asked for this seal so it can integrate and build.

## 1. What is staged

The patches are against the exact rendered source, the frozen workspace
`logarithm-stage09-workspace01/explainer_corpus/logarithm_mysteries`.

| Patch | Where | Change |
|---|---|---|
| `patches/O1-pause-line.patch` (+31 −4) | `draw.rs:372–374` | The line becomes `predict`'s probe |
| | `draw.rs`, after `predict` | `PAUSE_WORDS`, `PAUSE_SIZE` 22, `PAUSE_DROP` 0.27, `pause_line()` |
| | `draw.rs`, tests | `the_pause_line_stays_under_the_narrowest_bar` |
| | `lifting_loops.rs:374` | 06_2 bar moves under its card: `v(read_x + 1.3, read_y - 1.4)` |
| `patches/O2-rounded-value.patch` (+1 −1) | `complex_turns.rs:718` | "real part ln 2 ≈ 0.693" |
| `patches/O3-bar-off-the-axis.patch` (+1 −1) | `complex_turns.rs:636` | 05_5 bar moves under its card: `v(IN.centre[0] + 2.2, 1.85)` |

Full staged copies are in `staged/src/chapters/`. To apply, from the crate root, in any order:

```
patch -p1 -i <this folder>/patches/O1-pause-line.patch
```

Each patch also applies alone. `git apply --check` passes for each.

### Current source hashes

Recomputed at sealing. For all three files, the workspace and the stage09 candidate are byte-equal
to the rendered base below. The workspace `script.tsv` is still
`559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a`.

| File | Tree | sha256 |
|---|---|---|
| `draw.rs` | Rendered base | `2a4e84504bab636433cd419b2c9990986d7c549bf05aa3432d33bd5a9b60fe28` |
| | Staged | `060504c6b72acdc32758a417db67a87e206a6f464bfbb125d6f21c29d95a83be` |
| | Repo canonical | Equal to the rendered base |
| `complex_turns.rs` | Rendered base | `93f663d372ab79a77139287dc38b94c148909f7ea4a6ba549884c91f342eb19a` |
| | Staged | `678812f166e9ec4b8ef8125283672bf28aa4ba0add0c14a58a4a6f6077800ad8` |
| | Repo canonical | `2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead` |
| `lifting_loops.rs` | Rendered base | `01885b9f558c36bc8af425bfdf427e4b30d74d575819121e52988b5d06db7346` |
| | Staged | `daf5b18dd4d54714df652acf558b9068f48d573e88e60d205f0ff6ef45f34dfb` |
| | Repo canonical | `ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e` |

Two traps when integrating:

- **Repo canonical is not the rendered source.** `explainer_corpus/logarithm_mysteries` is clean at
  `2fb1c0c`, but 231 lines of the workspace's `complex_turns.rs` and 145 of its `lifting_loops.rs`
  are absent there, and its `script.tsv` differs. All three patches did apply to a scratch copy of
  canonical, with the `complex_turns.rs` hunks 13 lines earlier. That result is not the render's
  source, and no check below covers it. Apply to a tree built from the stage09 source.
- **Git HEAD holds my rejected first draft of `staged/`.** `fac0894` committed those three copies
  (`draw.rs` `51b98e8e…`, `complex_turns.rs` `5466ff6e…`, `lifting_loops.rs` `b78fc786…`). I didn't
  make that commit. The draft fails the binder mirror in section 6. Take this folder's working
  files, checked against `SHA256SUMS`, and not `git show HEAD:`.

## 2. Decisions for root

**One change goes beyond the brief: the 06_2 bar moves (O1).** The bar sat at the top of the z plane
with the axis directly under it, so a line below it would cross the axis. It now sits under the
card, with its left edge on the card's left edge. The walk, readout and voice are unchanged. If you
reject this, 06_2 needs some other placement before O1 can land.

**The line doubles as `predict`'s probe.** I rejected the first draft, which added a third mob to
the fade, because `bind_final80.py:306–315` requires `fade_out_all(stage, &[bar, probe], ` verbatim
and `hold-audit.json` records that line. Making the text the probe keeps every pinned string and
line. At the fmn pin, `add_dt_updater` (`stage.rs:1775`) fails only on a stale handle or when updater
ids run out (`stage.rs:1718–1732`), and `update_mob` (`stage.rs:2217`) runs the updaters of a mob
with children. A side effect: the comment at `unreachable.rs:1575` says `predict` uses "a bare
probe". It is now stale, and I left it because that file is not mine.

**The line appears instantly with the bar.** A fade-in would cost time the gap doesn't have.

## 3. Timing

The patches add no time. The gap is 0.6 s breath + 4.0 s think + 0.25 s fade = 4.85 s, as
`hold-audit.json` records for all 13 holds. `kit.say` starts the next cue straight after that fade,
so any played animation would move it. The line adds no played animation, and it leaves in the
existing 0.25 s fade. It shows for about 4 s at 19 characters.

## 4. Spacing audit of all 13 sites

Full data is in `site-audit.tsv`. The line is 129.9 × 13.4 cell pixels, centred 0.27 under the bar.
Every recorded picture in each hold was checked with a width allowance of 4 percent.

| Site | Source | Bar | Ink in line box | Nearest ink (cell px) |
|---|---|---|---|---|
| 00_4 | `ratio_ruler.rs:371` | Unchanged | 0 | 76.2 |
| 01_3 | `building_ruler.rs:187` | Unchanged | 0 | 207.8 |
| 02_2 | `slide_rule.rs:204` | Unchanged | 0 | 205.8 |
| 04_6 | `undoing_growth.rs:749` | Unchanged | 0 | 32.3 |
| 05_1 | `complex_turns.rs:245` | Unchanged | 0 | 34.4 |
| 05_5 | `complex_turns.rs:636` | **Moved (O3)** | 0 | 31.7 |
| 06_2 | `lifting_loops.rs:374` | **Moved (O1)** | 0 | 32.7 |
| 06_4 | `lifting_loops.rs:475` | Unchanged | 0 | **7.6**, axis top below |
| 07_3 | `branch_cut.rs:419` | Unchanged | 0 | **11.7**, "remove" row below |
| 08_1 | `product_rule.rs:221` | Unchanged | 0 | 33.0 |
| 08_4 | `product_rule.rs:346` | Unchanged | 0 | 78.2 |
| 11_3 | `unreachable.rs:1070` | Unchanged | 0 | 47.0 |
| 12_5 | `what_it_forgets.rs:1123` | Unchanged | 0 | 57.3 |

- **Tightest sites:** 06_4 and 07_3 have the smallest margins of the 13, so look at them first
  after a render.
- **Moved bars:** both new bar positions have 0 ink in every recorded hold picture. Neither crosses
  anything.
- **Chapter 04:** `undoing_growth.rs:1124` has a 0.85 row near the line's height, but it arrives in
  04_7, after the line has faded, as its own comment says about the bar. No edit is needed.
- **Files left alone:** 04, 11 and 12 are not edited. All three sites clear without a call-site
  change, though their pictures do gain the line through the shared helper. If their owners move
  anything near a site, the line box in `site-audit.tsv` is the area to keep clear.

## 5. Rebinding

Exact values are in `rebind-table.json`, and the file hashes are in section 1. No line moves in
either chapter file. In `draw.rs` lines are added only after line 390, so every pinned helper line
stays where it is.

Only two holds change marker and interval. 05_1 and 06_4 need only the file hash.

| Hold | Line | New marker | New `interval_sha256` |
|---|---|---|---|
| 05_5 | 636 | `predict(&kit, stage, v(IN.centre[0] + 2.2, 1.85), 3.0)?;` | `7e85717991f92e0e27d49b641c2754fb4062db5f56c1fb730a16b99f7370d220` |
| 06_2 | 374 | `predict(&kit, stage, v(read_x + 1.3, read_y - 1.4), 2.6)?;` | `ea604094c5df34006a99f6717b1aac553864996314d0b2010ca1ee2ccd356ce5` |

The old hashes are named in oliver15 and oliver16 (workspace and stage09 copies),
`bound-plan/hold-audit.json`, `bound-plan/inputs.sha256` and `bound-plan/phrase-anchors.json`. I
searched only the workspace and stage09, so root's other manifests may also name them. The six
phrase anchors are at `complex_turns.rs` 369, 371, 373, 375 and `lifting_loops.rs` 587, 593, clear of
every changed line, so they need only the file hash.

## 6. Checks run

All were read-only and ran on the staged bytes. Scripts are in `checks/`, as run; they take their
paths as arguments.

| Check | Result |
|---|---|
| Base hashes at sealing | Workspace, stage09 candidate and my baseline agree, 3 of 3 |
| Patches rebuild the staged bytes | 6 of 6 orders |
| Each patch applies alone | 3 of 3 |
| `rustfmt --check`, 2024 edition, repo config | Exit 0 on all three files, as on the baseline |
| Binder parse mirror (`bind_final80.py:306–315`) | Passes on the staged bytes; the rejected draft fails it, as a negative control |
| Pinned `draw.rs` lines 346, 352, 354, 387, 389 | All 5 markers still in place |
| Recorded interval hashes rebuilt from the baseline | 4 of 4 match, as a positive control |
| Rendered bars against source geometry | 13 of 13 within about 1 cell pixel |
| Font against labels in the master | Ratio 1.000–1.005 on 5 labels |
| Stillness within each hold | 0 changed pixels outside the bar, 13 of 13 |
| Name collisions for the new identifiers | None in the crate |
| Other rounded values printed with `=` in 05–08 | None; line 718 was the only one |
| `≈` in plain text in this master | Renders, picture 0426 (604.58 s), which also reads "ln 2 ≈ 0.693" in 04_7 |
| New test, by hand, from font metrics | half width 0.962 < 1.2; top −0.171 < −0.135 |

## 7. Limits

- **No compile.** Nothing here was built. rustfmt proves parsing only, not types. The new test has
  never run. New calls match forms already in this crate, and I read the pinned fmn source for the
  updater and fade paths, but only your build can prove them.
- **No pixel of the new line exists yet.** The audit measures empty space in the current master's
  recorded 960×540 frames, with 5 or 6 pictures per hold. The line's size comes from CMU Serif in
  `fmd-font` 0.4.0, while `Cargo.lock` names 0.3.3. The labels above matched, but that is calibration,
  not a render.
- **`standin-13-sites-NOT-NATIVE.jpg` is a mock.** It is the recorded frames with text and a bar
  composited by Python, for judging placement only. It proves nothing about the render.
- **No motion or listening.** The line has no audio, but the holds have not been heard or watched.
- **Base is the rendered source only.** Every check here ran against those bytes. If other workers'
  patches touch these files or the hold layouts at 04, 11 or 12, recheck the patch and those sites.
- **Authorship.** I wrote these patches and reviewed the span they fix, so none of this is
  independent.

## 8. After your build

Run fmt, Clippy, the tests and `--lint`, then render.

- **Chapters whose picture changes:** 00, 01, 02, 04, 05, 06, 07, 08, 11, 12. Chapters 03, 09 and 10
  are unaffected.
- **Timelines:** these patches should change no cue, mark or `timeline_sha256`. If one changes, that
  points to a leak here, unless other edits explain it.
- **Where frames should differ:** only from `mark_frame` to `next_cue_frame` for each site in
  `site-audit.tsv`, and for O2 from about 783.4 s (first recorded at frame 47005, absent at 46975)
  through chapter 05's last full frame, 48391.
- **What to view:** each site's frame at mark plus 0.5 s, 06_4 and 07_3 first.

## 9. U1

U1 stays as uncertain authored timing, on root's instruction, with no physics or narration change.
The source and timing evidence is in `U1-evidence.md`.
