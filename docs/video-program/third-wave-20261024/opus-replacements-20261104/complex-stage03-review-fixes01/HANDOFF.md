# complex-stage03-review-fixes01: patches for F3–F6 of the stage03 cross-review

LavenderPike, for BlueSalmon. Bead `mx-video-program-program-87do.39`. Date 2026-11-05 (UTC). Narration stays ElevenLabs Oliver.

**State: frozen, not applied, uncompiled.** These are four patches against the exact stage03 bytes, plus a check script. Nothing was compiled, tested by Cargo, linted by Clippy, rendered, synthesized or heard. No take was made or selected. The candidate tree, canonical files, Beads and TODO were not touched. The only tool run on Rust was `rustfmt --check`, on scratch copies.

Root's acceptance, after reading the sealed `complex-stage03-cross-review01` (`SHA256SUMS` `33d0705b…`): "Accept F3 clearer clause though there may be none, and F4/F5 targeted guards/F6 storyboard consistency." Two findings are not in this bundle:
- **F1** (the plan and the hold map) is BronzeJaguar's binder.
- **F2** (the `.orig` backups and `.ee` hook state) is root's to preserve and exclude at promotion.

## The patches

All paths start at the stage root, `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-repair-stage03/`, under `explainer_corpus/complex_analysis_visual/`. The same paths work from the repo root once canonical holds these stage03 bytes. Each patch applies alone, or with the others in either order, with no offset and no fuzz (section C).

| Patch | sha256 | Changes |
|---|---|---|
| `patches/F3-15_2-clause.patch` | `0fa7cb79a41615885a96f7917832ac4021ce172b86c8edffa18067f1ac4b2320` | `script.tsv:104` and `docs/storyboard.md:284`: 15_2's clause |
| `patches/F4-turning-one-sample-guard.patch` | `8e90cf7e7fa0aa47813a2ac5e07ce66ce2637f07b02d98c359487da5f03328b8` | `src/chapters/counting.rs`: one new test, inserted after the takeover test (new lines 1818–1824) |
| `patches/F5-budget-painted-planes.patch` | `f3f5907519ed2234046ba82a87cb6e6bf6d24947525d5e99a3eb026c4e1dcd7a` | `src/chapters/draw.rs:1607–1613`: one test reads the planes the film paints |
| `patches/F6-storyboard-quotes.patch` | `ad92990c8217ea74656d981eebb8591c390e12eb33e205a76340a5bd89018341` | `docs/storyboard.md:134`, `:138`, `:151`: three picture cells |

| File | stage03 (receipt `3f652c2d…`) | after all four |
|---|---|---|
| `script.tsv` | `1e24aba8b520f2e61e8b2bc392a218b664cf5556ad788ceca17fbc45bb92efab` | `12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3` (F3 only) |
| `docs/storyboard.md` | `57508d96103f0a4a86ed2265a51bb65d28fc68d8e6c96b3bd4438d2d97c83b5f` | `e204cf7726dadefe46dccb419e0cc0a641acbeeb4e884c3f55ff6d39a980536b` (F3 + F6) |
| `src/chapters/counting.rs` | `c1df16ea04d4283d490598ccabe039cdc846be96458668d63544aef9ac81d36a` | `d41d414134c4ffc553df9773bc1b082fab376f1a4344d248ab7bdb3f443fe8f9` |
| `src/chapters/draw.rs` | `865599e94331038782be128189fc1acb642e1d00b2f7fe32bd9fe68c958951b9` | `3f1c581dbc028a4d1a50a46d15028a0e005e0a76cf58709c6fbbbce2d30ea2db` |

To apply, from the stage root: `patch -p1 -F0 --no-backup-if-mismatch -i <bundle>/patches/<name>`. `git apply` also works. `-F0` refuses fuzz. `--no-backup-if-mismatch` stops GNU patch writing the `.orig` files that F2 is about. Those are written only when a hunk lands at an offset, and these patches don't. If a target file no longer has the stage03 bytes above, make the patch again from `checks/review_fixes.py`'s edits in a new path. Don't force it in.

## F3. 15_2's last clause

- old: "It allows at most one extension to a given connected region, though not always everywhere, and along different paths a continuation can branch."
- new: "It allows at most one extension to a given connected region, though there may be none, and along different paths a continuation can branch."

`script.tsv` and the storyboard's words change together, because `main.rs` `the_storyboard_the_registry_and_the_script_agree` (`check_video`) requires them to be equal (section D, before and after). The rest of the cue holds:
- 15_2 is still six sentences, and sentences 0–4 are byte-equal.
- There is no digit, and no other script line changes.
- The hooks in `closing.rs` are `wait_sentence` 1–5 plus `wait_for("and it turns a real integral")`. Each still lands in the same sentence.
- No hook, hold, finish or post-voice beat moves.

What F3 still needs, outside this bundle:
- **A new 15_2 Oliver take.** Root's call when to make it. Until then, the current 15_2 take's words won't equal the script.
- **The plan.** `narration_units.json` still holds the old clause once. Its 15_2 unit text and `script_sha256` follow the script. That is the binder's job (BronzeJaguar, Mail 3580).
- **The map's `scripts.complex_analysis_visual`.** It is `1e24aba8…` today, and becomes `12442480…` once F3 is in.
- `docs/script-as-reviewed-mail622.tsv` is left alone. It records what root reviewed.

## F4. The turning field's one-sample guard

```rust
    /// `Turning` builds with `portrait`, which colours at the plate's samples per cell, and
    /// repaints with `portrait_colors`, which takes one. They agree only at one sample, so the
    /// takeover and the turn back to 11_6's colours are exact only while `PLATE` keeps one.
    #[test]
    fn the_turning_field_repaints_as_the_plate_paints() {
        assert_eq!(super::super::draw::PLATE.samples, 1);
    }
```

The film code before `#[cfg(test)]` is byte-equal. The guard holds today: `PLATE = PLATE_CELLS4`, whose samples come from `PLATE_BLEND`, `PLATE_TILE` and finally `PLATE_NOW`, where `samples: 1` (section E). `PlateStyle.samples` is `pub u32` and `PLATE` is a `pub const`. The test adds no `Some(Func::Quintic)`, which `counting.rs`'s own source-count test counts across the whole file (it stays at 3). The test sits next to `the_turning_field_takes_over_unseen_from_the_walks_own_pieces`, which is the claim it protects.

## F5. The budget test reads the planes the film paints

```rust
        // the widest pictures the chapters paint (05's sin and Γ, 8.4 wide) and the tallest
        // (04's plane and 05's exp, 6.4 high)
        for p in [
            super::super::gallery::trig_plane(),
            super::super::gallery::gamma_plane(),
            super::super::reading_color::plane(),
            super::super::gallery::exp_plane(),
        ] {
            let (cols, rows) = picture_cells(&p, &PLATE);
            assert!(rows <= 800 && cols <= 960, "{cols} × {rows}");
            assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT);
        }
```

This replaces the made-up 6.4 square with the four planes that bound the film. Section F reads the film code of all 17 compiled chapter modules. There are 21 `Painter::build` / `portrait` call sites, each traced to its plane. Under `PLATE_CELLS4` (cell `0.035 / 4`):
- The widest is 960 columns, reached only by 05's sin and Γ.
- The tallest is 731 rows, reached only by 04's plane and 05's exp.
- The four planes give (960, 480), (960, 533), (731, 731) and (366, 731).
- The old bound, `cols <= 800`, would refuse the widest. That is why F5 exists.
- `GROUP_TRANSFORM_LIMIT` is 2,000 (`crates/explainer_kit/src/kit.rs:45`).

`gallery` and `reading_color` are private sibling modules in `chapters`. `draw::tests` sits inside `chapters`, so it can reach their `pub fn`s. The film code is byte-equal, so `every_colour_picture_is_crisp`'s counts are unchanged.

## F6. Storyboard cells quote what the film draws

| Row | Was | Now | The film's string |
|---|---|---|---|
| `storyboard.md:134` 04_2 | "band: size × 1.69" | "next band: size × 1.69" | `reading_color::BAND` plus `fact("color.band_ratio")`, which is "1.69" |
| `storyboard.md:138` 04_6 | "bump: where are the poles? how many turns?" | "where are the bump's poles?" / "how many turns at each?" | `reading_color::ASK` |
| `storyboard.md:151` 05_7 | "series converges to the right" | "series converges: Re s > 1" | `gallery::CONVERGES` |

Only the picture cell of each row changes. Each row keeps five cells, no line is added, and the words column that `check_video` compares is untouched (section G).

## What the hold rows need

Section H builds the map from teaching02 `holds/narration-direction-map-oliver11.json` plus `complex-stage03-cross-review01/proposed/hold-rebind-stage03.json`, and checks that all 28 rows bind stage03 as it stands. Six rows sit on patched files. In each, the `line` and `marker` stay and only the file sha moves:
- the three `counting.rs` visual_holds rows: 11_5:440, 11_6:503 and 11_8:604, moving `c1df16ea…` → `d41d4141…`
- the three `draw.rs` helper rows: 1112, 1166 and 1198, moving `865599e9…` → `3f1c581d…`

F3 moves the map's `scripts` hash, and F6 moves nothing a row binds.

## Checks

`checks/review_fixes.py` holds the edits as exact old and new strings. Each old string must occur exactly once, and the patches are made from those edits. `make` wrote `patches/` once. `check <fresh-scratch>` writes only into its scratch folder and runs these sections:

- **A.** The receipt is `3f652c2d…`. The four target files equal the receipt and the disk. Every other stage03 source file the checks read, and the plan, still equal the receipt.
- **B.** Each patch regenerates byte-equal from the edits. Each names only relative `a/` and `b/` stage paths.
- **C.** GNU patch applies each patch to byte-equal scratch copies of stage03: alone, all forward and all reversed. None has an offset, fuzz, `.orig` or `.rej`, and each gives exactly the edits.
- **D–G.** The per-finding checks described above, against stage03's own source.
- **H.** The hold rows.
- **I.** `rustfmt --check` with the repo's `rustfmt.toml` and pinned toolchain, on scratch copies:
  - both files are clean before and after
  - a control copy, with one space added to the new F5 assert, is refused, so the check is live

Rerun from the repo root:

```sh
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage03-review-fixes01/checks/review_fixes.py check <fresh-scratch> | diff - docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage03-review-fixes01/checks/outputs/check.txt
```

The output contains no paths or times, so a clean rerun is byte-equal to `checks/outputs/check.txt`. That file ends with `FAILURES: none`.

## Limits

- The two new tests were never compiled or run, and Clippy was not run. rustfmt parses and formats; it does not resolve names or types.
  - The names F5 uses already exist: `trig_plane`, `gamma_plane`, `exp_plane`, `reading_color::plane`, `picture_cells`, `PLATE` and `explainer_kit::GROUP_TRANSFORM_LIMIT`. Section F checks they are `pub` and declared.
  - The `assert_eq!` on a constant follows the file's own `assert_eq!(PLATE, PLATE_CELLS4)` and counting's `assert_eq!(ROOT_WALK_SQUASH, 2.0)`.
  - Root's native build is still the evidence that these compile.
- The plane mirror in section F reads `Plane::new` literals and six hand-read plane sizes, each checked against its own source words. It doesn't execute the film.
- **Independence:** none.
  - F4 guards my `PlateStyle` where it meets WhiteSwan's `Turning`.
  - F5 rewrites my own test.
  - F3 and F6 are my wording.
  - No one else has reviewed these patches.
- **Noticed, not patched** (outside what root accepted): `draw.rs:615–616`'s doc comment on `PLATE_CELLS4` still says "A bracket for what finer cells cost, not a proposal", though `PLATE` is now `PLATE_CELLS4`. The comment is mine, and the fix is one line if root wants it.

## Files

- `HANDOFF.md`: this file.
- `patches/`: the four patches above.
- `checks/review_fixes.py`: the edits, `make` and `check`.
- `checks/outputs/check.txt`: the sealed `check` output.
- `inputs.sha256`: every file read, repo-relative.
- `SHA256SUMS`: this bundle.
