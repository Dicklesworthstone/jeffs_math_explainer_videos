# cc0 thumbnail route 205: complex analysis publication thumbnail

Dispatch-0 (cc0), 2026-11-08 UTC. **Inspection only.** I rendered nothing, built nothing, and changed no source, helper or packaging file. This file is the only file I wrote in the repository. For viewing only, I made two downscaled copies outside the repository.

Root has since selected `publication19-thumbnail208/thumbnail.png`, an exact copy of the reviewed thumbnail24 PNG. This report therefore checks that selection's provenance; it does not propose a new render.

## Result

**My read-only provenance checks agree. No new render is needed.** The exact production19 binary cannot produce an acceptable thumbnail (blocker 1). The selected PNG is the correct route, but its provenance has to be recorded as "production19 source plus `thumbnail24.rs`", not as the production19 binary (concern 2).

## The native thumbnail route

- **Command.** The kit's `thumbnail` command renders the video's thumbnail scene as one PNG: `crates/explainer_kit/src/cli.rs:1015-1037`. The output is `OUT/complex_analysis_visual_thumbnail.png`, and an earlier file is moved aside rather than overwritten.
- **Scene.** The complex crate registers its scene as `thumbnail: Some(chapters::thumbnail::make)` in `src/main.rs`; the scene is `src/chapters/thumbnail.rs`.
- **Helper.** `thumbnail24.py`, sha `64e493a6…`, was launched by `thumbnail-after-timeline24.py`, sha `37d43c9d…`, through `guard_csd02.py` as job `csd-production-thumbnail24`.

## Provenance checks (all agree)

| Check | Evidence |
|---|---|
| The selected PNG is the reviewed image | `publication19-thumbnail208/thumbnail.png` and `thumbnail24-reviewed/complex_analysis_visual_thumbnail.png` both hash to `e5dfc2bd04aa3af148ee66ca8107594acf3015a237f5f36a08f1c3c72c4c2eab`. The image is 3840×2160 RGBA, 679,205 bytes, and its alpha is 255 everywhere. |
| The terminal record matches | `thumbnail24-reviewed/terminal.json` has sha `19d957c3…`, equal to `terminal_sha256` in `publication19-thumbnail208/review.json`. The job ran from 17:22:49Z to 17:33:57Z on Nov 7, exited 0, and its `inputs_sha256` equals `final_inputs_sha256` (`3496ae1c…`). It hit no resource stop. |
| The source is production19's, with one file changed | `thumbnail24.py:13-14` copies the `production-repair19` workspace and asserts its `thumbnail.rs` equals `before-thumbnail24.rs` (`5d90834b…`). It then swaps in `thumbnail24.rs` (`7e7f7a09…`). `:19` asserts that this is the only changed file, and `:31` asserts the tree is unchanged after the build. |
| The build matches build19 | Both builds use the same cargo/rustc pins, `--release --locked --offline` and engine-budget config (`thumbnail24.py:15-21`, `build19.py:19-25`). Both pass 223 tests and clippy. The `facts` sha is `7d528509…` in both (`thumbnail24.py:25` and build19's `qualification.json`), and `chapters` output is byte-equal to build19's (`:26`). |
| The image is the selected one | The command was `thumbnail24 thumbnail --res 3840x2160 --out …/csd-production-thumbnail24/output`, run in `work/production-repo07` (`thumbnail24.py:27`). Binary sha is `9b2a01d6…`, the image sha is `e5dfc2bd…`, and the file decodes as a 3840×2160 PNG (`:30`). |
| The picture matches the film's convention | `thumbnail24.rs` draws the film's own `Painter` with `Func::Square`, not a separate tile grid. I viewed a 1280×720 downscale: the hue field is smooth and makes two full turns around 0. The real axis is red and the imaginary axis is cyan, as arg(z²) requires. The title, subtitle and legend are readable, with clear margins and no overlap. |
| Upload limits | The PNG is 16:9 and 679,205 bytes, under YouTube's 2 MB custom-thumbnail limit. It is fully opaque, so there is no transparency to flatten. |

## Concrete concerns

1. **Blocker for the "exact production19 binary" route.** Production19 (`f4836eee…`) compiles `thumbnail.rs` sha `5d90834b…` (`thumbnail24.py:14`). That is the coarse 51×51 tile grid with 12 flat colours. The owner rejected exactly that picture in `fmd-render-readiness01/thumbnail-rejection-repair01.json` (2026-11-06T18:40:50Z): "Owner rejects coarse flat 51x51 tiles and twelve-colour quantization." So `production19 thumbnail --res 1280x720 …` is cheap, but it would produce a rejected image. Do not use it.
2. **Record the provenance precisely.** The selected PNG was made by binary `9b2a01d6…`, not the movie's binary `f4836eee…`. The source is identical except for one file, which the table above proves. The release record should say "production19 source + `thumbnail24.rs` (`7e7f7a09…`), binary `9b2a01d6…`". Root's `publication19-thumbnail208/review.json` already says "separate native thumbnail source", which is accurate.
3. **The thumbnail cannot be reproduced from the repository.** The canonical `explainer_corpus/complex_analysis_visual/src/chapters/thumbnail.rs` is still the rejected coarse version (sha `5d90834b…`, header "provisional, awaiting editorial/device review"). `thumbnail24.rs` and `thumbnail24-reviewed/` are untracked: `git ls-files` lists neither. AGENTS.md requires a published cut to stay reproducible from its tagged commit, and anyone running the canonical `thumbnail` command today gets the rejected picture. This does not block the upload. Root owns the integration, and `thumbnail24.rs` is the file to carry over.
4. **Low: the legend is not legible at phone size.** On a 320×180 downscale, which I viewed, the title and subtitle read, but "z²: HUE = OUTPUT ANGLE" is a few pixels tall. The earlier smooth03 review saw the same thing ("secondary caption tiny"). Both thumbnail24 reviews viewed only 2048×1152 and 3840×2160. This is cosmetic; the legend is secondary and the title carries the thumbnail.

The thumbnail text ("THE BEAUTY OF COMPLEX ANALYSIS", "A PLANE IN COLOR") does not conflict with the draft03 title ("Complex Analysis Made Visual: From Domain Coloring to Residues"). The picture is a domain colouring.

## Limits

- I read only local copies. There was no SSH, and the `csd` job folder and `work/bin/thumbnail24` were not read.
- The hashes on `csd` are taken from the copied `terminal.json`.
- This is not publication, picture, audio or AV acceptance.
