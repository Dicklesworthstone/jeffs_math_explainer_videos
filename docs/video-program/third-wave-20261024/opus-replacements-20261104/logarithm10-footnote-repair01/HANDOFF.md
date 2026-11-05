# Logarithm 10: closing footnote readability

BronzeJaguar, for BlueSalmon, answering Mail 3130. Narration is ElevenLabs Oliver and is unchanged.

**State: staged and frozen for root. Nothing here has been compiled, tested by Cargo, rendered or heard.** Canonical is untouched. The patch changes six film lines in `doing_nothing.rs`, moves none and changes no timed call. The strings on screen stay the same, byte for byte.

## The repair

| | Upper row (turning rate) | Lower row (`precisely:` condition) |
|---|---|---|
| Colour | `dim` → **`ink`** | `dim` → **`ink`** |
| Size | 18 → **22 pt** | 22 pt, kept |
| Centre | y −3.15 → **−2.93** | y −3.62 → **−3.40** |

| Measure, in scene units | As staged, frame 229 | After |
|---|---|---|
| Lower row to the frame's edge | 0.267 (24 px at 720p) | **0.484** (about 44 px; 5% of the height is 0.40) |
| Upper row to the grid | 0.489 | 0.268 |
| Between the rows | 0.267 | 0.251 |
| Lower row's half-width | 6.333 | 6.333, unchanged |

The "after" column is arithmetic from the widths and heights measured on frame 229. `checks/outputs/footnotes-before-after-NOT-A-RENDER.png` shows the layout: your frame above, and below it the same frame with the rows recoloured, scaled and moved by hand. The upper row is blurred there by the hand scaling, so judge the layout only.

| Contrast on the ground, frame 229 at 720p | Nominal | Brightest tenth of inked pixels | Median inked pixel |
|---|---|---|---|
| Upper row as staged, `dim` at 18 pt | 6.13 | 5.29 | **3.30** |
| Lower row as staged, `dim` at 22 pt | 6.13 | 5.93 | **3.69** |
| `A sends no direction ...`, `ink` at 22 pt, for comparison | 16.47 | 15.81 | 8.19 |

| Item | SHA-256 |
|---|---|
| `logarithm10-footnote.patch`, on canonical or stage06 `doing_nothing.rs` `eeee764f0fe5fd3c…` (same bytes) | `14a3dca5c561b13012c346bc448dc02f6a18069e89fa002ccece67196d6bec0d` |
| Result, staged in this folder at its crate path | `7f445be212c586093d2092113b086d809dcc189619985baf1c8f0c407c709b47` |

Apply with `git apply` or `patch -p1` from the repo root.

## The film lines

| Line | Change | Why |
|---|---|---|
| `:41` | Drops `precisely` from the import | Not called here any more; Clippy would refuse the unused import |
| `:66`–`:67` | A stale two-line comment becomes `FINE_SIZE` and `FINE_ROWS` | Keeps the line count and gives the new test real values |
| `:69` | `SOURCE` now starts with `precisely: ` | So the call fits one line. The screen string is what the helper built |
| `:594` | Upper row: `FINE_SIZE`, `ink`, `FINE_ROWS[0]` | The repair |
| `:595` | Lower row: `kit.text` directly, same three | The repair, without touching the shared helper |

Below the film code, the existing fine-print test drops its separate prefix count and now also requires the prefix. A new test, `the_fine_print_is_in_ink_and_clear_of_the_edge_and_the_grid`, pins both rows' calls and all the margins.

## Decisions for you

1. **`ink` on both rows is a judgement call.** `dim` clears WCAG's 4.5 at nominal, but its median pixel does not at this size. The fine print now has the same weight as the rule rows above it. I chose this because 10_4 sends the viewer there ("On screen is the textbook version") and nothing else is competing during the hold. To keep them quieter, set `:594`–`:595` back to `Palette::dim()` and keep the rows and size; the new test and the probe's contrast mark then need to change too.
2. **This chapter's fine print now differs from the rest of the film.** Seven other chapter files still call `draw::precisely`, which is `dim` at y −3.62. A film-wide change would be `draw.rs:339`–`:340`, but that is a shared helper named in map13, and I have not checked whether those chapters have room higher up, so I did not propose it. Mail 3144's Liouville line gets a local patch in my next folder.

## Scope

- **Words:** the turning-rate explanation and the full condition are unchanged, including `(−∞, 0]`, `−π < Im < π`, `real if A is` and the citation. Still 197 characters, as `docs/storyboard.md:99` says, so the reading window is 14.92 chars/s over 13.2 s.
- **Narration:** no wording need, and no take is affected. `script.tsv` is untouched.
- **Timing:** no timed call changes, so the chapter's length should not change and its music stem should still fit. Only a native run can confirm that.
- **Not split into two lines:** the lower row is long at 134 characters, but three rows do not fit between the grid and the new margin without moving the grid.
- **Not touched:** the 18 pt `dim` tick labels under the turning line. You did not flag them.

## Bindings

**Map13's one hold on this file must be rebound.** Its line and marker survive (`:602`, `kit.music_breath(stage, FINE_PRINT_S - read_for)?;`), but its interval runs from `:537` to `:602`, which contains the two changed rows, so both hashes change. `tools/narration_director.py:124` raises on a stale evidence hash.

| 10_4 → 11_1 reading hold | Old | New |
|---|---|---|
| `sha256`, `evidence.sha256` | `eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143` | `7f445be212c586093d2092113b086d809dcc189619985baf1c8f0c407c709b47` |
| `interval_sha256` | `eb79bd6b505c3dc00074e026184177378cbb3dd599501f35ea47b890a497bf6c` | `9fe875942e620939da7839720e40e029c830cfe152247ae59de1db3c4c2d8927` |

These are values only, good only for exactly the staged file; I did not patch the map. The formula reproduces the old interval hash on canonical. `combined-repair-stage06/bound-plan/hold-audit.json` and `inputs.sha256` also name the old hash, so your binder must take the new input.

Unaffected: the 7 other `hold-audit.json` rows on this file keep their line and marker; `draw.rs` stays at the hash map13 names; `narration_units.json` does not name this file's hash.

## Native picture checks

`source-markers.json` has these in full, plus the rebinding values. Anchor on cue and sentence: the old times are stage06's scratch voice, not Oliver. Frame numbers are labels in `logarithm-stage06-frames01/10_doing_nothing`, counted from 0.

| Anchor | When | Old frames | Expect | Fails if |
|---|---|---|---|---|
| N1 both rows | 10_4 sentence 4, after the 0.5 s fade at `:596`, to the chapter's end | 200–229 (90–103.183 s) | Both rows in ink at 22 pt, centred; the probe passes | Grey; lower row under 0.40 from the edge; under 0.15 to the grid or between rows; a side past 6.86 |
| N2 exact words | Same hold, a final-size frame | 229 | Exactly the strings in `source-markers.json`, read by eye | Any change in wording, signs or citation; a clipped or fallback `⇒ − ∞ π` |
| N3 reading window | `:597` to the chapter's end, on Oliver | n/a | At least 13.2 s | Shorter |

```
python3 -B <this folder>/checks/footnote_probe.py probe <frame.png>
```

The probe works at any frame width. Its marks are my own: contrast 11.0 on the brightest tenth, halfway between the two nominal contrasts, and the distances in N1. Your frame 229 fails it on four counts, and the hand-made mock-up passes. It measures rows; it does not read them.

## Evidence

Regenerate from the repo root with a scratch path that does not exist yet. It starts the probe with `python3 -B`, because that needs PIL, and regenerates the patch and `source-markers.json`.

```
/usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>
```

| Check | Result | What it cannot show |
|---|---|---|
| Patch round trip by `git apply` | Applies, gives the staged bytes, reverses | Anything about behaviour |
| Film-line diff | Lines 41, 66, 67, 69, 594, 595 only; line count kept; no timed call changed | That it looks right |
| Strings | Upper row identical; lower row equals the helper's format of the old string | That the engine draws them |
| Geometry mirror | Margins as above, from frame 229's measured extents | No frame at the new rows; assumes height and width scale with size |
| New test's conditions | Pass on staged; source half fails on canonical; 10 plants caught | The Rust has never run |
| `docs/check_cues.py` on a scratch copy | Passes both; differs only in listing the lower row's literal; refuses a removed `say` | Only cue order |
| `rustfmt --check` | Clean on both | Not a compile |

**Frames I read, and only these:** sheets 049, 050 and 057, and frames 200 and 229 (229 by eye; both measured). Your review covers the rest of the chapter.

My checks failed twice on the way, both my own errors: a plant hit a comment instead of the constant, and I had not expected the cue checker to list the changed literal. Both are fixed in `run_checks.py`.

## Overlap and gaps

- **Not independent.** You named the defects, and I wrote the fix and every check. Earlier I reviewed WhiteSwan's repair of this chapter (Mail 1533) without raising this. We share one auto-memory directory, one model and the same skills.
- **Still yours:** Cargo, tests and Clippy; the glyph probe; a chapter 10 render and N1–N3 on it; rebinding; Oliver sync and listening; the final 4K look.

I changed nothing in canonical, Beads or any sealed folder, and started no agents.
