# Engine geometry validation 02: handoff

RubyRaven to BlueSalmon, for bead .39. Answers Mails 2918, 3006, 3028, 3054, 3064, 3095, 3117 and 3167.

**Nothing here has been compiled, run or rendered.** The candidate is ready for root's guarded native validation: follow `BUILD.md`. Its pre-Cargo steps were rehearsed in a throwaway folder and passed. Patches were replayed from fresh sources with both `git apply` and `patch -p1 -F0` (`checks/patch-replay.out`). All maths and counts below come from reading source, not from a run. Defaults, the canonical engine, the pin and film density are untouched, and the cells4 plate is not reduced.

## Remaining blockers

1. **The real 11_7 cannot be rendered yet.** No tree holds the plate core, `complex11-counting-teaching02` and its C1 correction together. `color-native-stage01` has the plate core and no `TURN_STEPS`. `combined-repair-stage02` has the turn, a one-argument `picture_cells`, and no `PLATE`. Until root merges them, the lab's `turns` stage is the representative, and BUILD section 8 is the follow-up.
2. **The engine export needs one fetch.** Its own lock pins eight crate versions that are not on this machine. `cargo fetch --locked` reaches crates.io once and compiles nothing. The alternative, dropping `--locked` in the export, is unverified. The lab and kit builds don't use that lock.
3. **Memory and time are unmeasured.** No budget in this work bounds memory.

## The stack

Apply in this order, as in BUILD section 1.

| Target | Patch | Author | sha256 |
|---|---|---|---|
| engine | `$B/engine-plan-limits.patch` | RubyRaven, budget01 | `63c1afcab4960b7ab5ceafed7b05836ae672449b5832c073e913f3a413f268b3` |
| engine | `engine-compiled-route-test.patch` | RubyRaven | `526ece36b650e0ac3a425ef9d310a2ff62d6bda852bbc70baed2719b782808a9` |
| engine | `engine-tests-repair.patch` (new) | RubyRaven | `d9dccef8cf5898cc7d6564ac06afedffbb3d01488d5fbdfecb98ae0965c41df9` |
| kit | `$X/patches/kit-render-options-hook.patch` | LavenderPike, unchanged | `c838f47129dac745b24b6a5b6b6f2baa972416e2b9aff51d4d4c25ef686cd604` |
| kit | `kit-hook-routes-test.patch` (new) | RubyRaven | `fb8aca3a3ed8690033259775d5be3647f4c2ef007041a675e158543a1a070bb3` |
| lab | `$B/lab-plate_lab.patch` | RubyRaven, budget01 | `02bc170c2ced2dc98f095617aacca9eace7b20768bb9839be95279cedd9bdf82` |
| lab | `lab-probes.patch` | RubyRaven | `97f420b3a01df1eee821bf6f87cd8261090f0c5d8a9c55d27f8c608ca5d3106a` |
| lab | `lab-fade-turn-probes.patch` (new) | RubyRaven | `e7a730c39e41432feca6c09f27e5d222a1d1ee28a688d30d09cd93985e61137d` |
| film | `$X/patches/film-declare-plan-budget.patch` | LavenderPike, unchanged | `ba322818cace378026473e3baaafb2ea678ccc6ce50313d82f3d89790bb78e1f` |
| film, after the probes | `$X/patches/select-cells4.patch` | LavenderPike, unchanged | `9f0f44fcc35cf807d0e3de983a035637a1750e224798d521cd7401982f3d47ed` |

## What was asked, and where it is

| Ask | Mail | Done in | State |
|---|---|---|---|
| Repaint probe and lower-budget control | 2918 | `lab-probes.patch` | Unrun |
| Compiled-worker test | 2918 | `engine-compiled-route-test.patch`, plus two mutants | Unrun. Lowered limits only |
| Isolated build manifest for all 19 packages | 2918 | `packages.tsv`, `engine-budget.config.toml.in`, `instantiate_config.py`, `BUILD.md` | Rehearsed before Cargo |
| Kit delivery wiring for real film use | 3006, 3095 | Lavender's hook, plus my route test | Unrun |
| Complete-record lock checker with planted controls | 3028, 3117 | `verify_patched_lock.py` | Run: ten planted faults caught in each of two modes, on both locks. Root's retest passed (Mail 3146) |
| F1 wrong segment assertion | 3054 | `engine-tests-repair.patch` | Unrun. Confirmed by my reading of `table.rs` and `plan.rs` |
| Corrected demand and memory wording | 3054 | The sections below | Done |
| F3 mandatory cells4 fade | 3054 | `fade-probes` stage | Unrun |
| F4 facade admission above the default | 3054, 3095 | `a_declared_plan_budget_above_the_default_is_admitted_and_the_default_still_refuses` | Unrun |
| Bounded 11_7 representative, `TURN_STEPS` kept | 3064 | `turns` stage, `checks/turn_desk.py` | Unrun |
| C1 `picture_cells(&inp, &super::draw::PLATE)` | 3064 | Blocker 1, BUILD section 8 | Awaits root's merge |
| Native-facade scope stated | 3054 | "Scope" below | Done |

## The knot budget does not cover moving geometry

Root's verified numbers stand: a cells4 repaint needs 2,150,602 knots, the explicit budget is `1<<22`, and point motion separately needs 1,073,108 segments.

`max_retained_profile_knots` raises one ceiling. How much motion costs depends on how the plate moves (LavenderPike's table; I checked the fixed limits by reading):

| z² cells4, 731 rows | New segments |
|---|---|
| Translation through a placement | 0 |
| Translation or uniform scale, baked into points | 734 |
| Rotation or warp | 536,554 a frame, with 731 distinct outlines |
| The second frame of a rotation or warp | Asks 1,073,108; a named refusal under `1 << 20` |

Rotation or warp also runs into these limits. Raising `max_retained_segments` alone doesn't clear them:

- **Rebuilds:** 536,554 unreachable segments exceed `max_unreachable_segments` (65,536), so every later frame rebuilds.
- **Monotone pieces:** at four pieces a segment the bound is 4,292,432, over the fixed `max_pieces` of `1<<22`. The real count per straight segment was not worked out.

The film's plates fade and recolour only, as far as a text search shows, so this does not bind cells4 today. It does bind any future idea of moving a plate.

| Ceiling | Value | Can `RenderOptions` change it? |
|---|---|---|
| `max_retained_profile_knots` | `1<<20`, proposed `1<<22` | Yes |
| `max_input_curves`, `max_retained_segments`, `max_instances`, `max_retained_shapes`, `max_retained_styles` | `max_input_curves` and `max_retained_segments` are `1<<20` | Yes |
| `max_unreachable_segments` | `1<<16` | Yes |
| `MonoTableLimits`: `max_pieces`, `max_table_bytes` | `1<<22`, `1<<28` | No: `retained.rs:242`, `vector_frame.rs:159` use the default |
| `BinningLimits`: `max_instances`, `max_tile_references`, `max_working_bytes` | `1<<20`, `1<<24`, 256 MiB | No |
| `MAX_STROKE_PROFILE_KNOTS`, per mobject | `1<<20` | No |
| u32 indexing | Root's observation, `plan.rs` 285-316 and 1027-1039 | Separate from admission |

Cells8's real limit is `max_input_curves`: 2,147,684 curves, crossing the limit at row 715 asking 1,049,620 (LavenderPike).

## Corrected demand

Budget01's segment column was wrong, and its `demand.py` control was circular, so it could not catch that. Rows differ only in position, and a shape's digest leaves position out, so a plate's rows share outlines.

| | z² cells4 | sin(1/z) cells4 | Six panels |
|---|---|---|---|
| Knots, still | 1,075,301 | 824,960 | 1,726,758 |
| Knots, full repaint | 2,150,602 | 1,649,920 | 3,453,516 (safe bound; the film's six fade one at a time: 2,014,551) |
| Input curves | 538,016 | 412,800 | 864,270 |
| Segments, budget01 (**wrong**) | 536,554 | 411,520 | 860,706 |
| **Segments, by reading** | **734** | **643** | **1,449** |

11_7's plate is separate: 640 rows of 1,289 knots, 824,960 live knots and 409,600 cells.

Memory, all unmeasured (LavenderPike's F6 table):

- **z² still geometry state:** about 152 MiB; segments about 63 KiB; monotone table at most 2,936 pieces, about 137 KiB.
- **Stage records in a fade:** 209 MiB, because `Transform` takes two family copies.
- **Knots and keys in a rebuild:** 377 MiB.
- **Upper bound in a fade:** about 882 MiB before frame buffers.

So "approaching 1 GiB" is a rough figure, and it applies to fades rather than to stills. Root's controller has to measure it.

## Kit wiring

**Decision: I adopted LavenderPike's hook unchanged and moved my own trial to `superseded-kit-slot/`.** My per-chapter slot named `plan_limits` inside the kit, so the kit could no longer build on the unpatched pin other films share. The hook's `adjust: fn(&mut RenderOptions)` names nothing the pin lacks. A film-wide budget costs nothing extra, since non-plate chapters retain about zero knots.

Reading the patch, I found the hook last at every site:

| Route | Site, after the patch | Position |
|---|---|---|
| Retained draft and `--crf` | `cli.rs:690` | After the delivery rebuild, immediately before `render`, inside the retry loop |
| Camera draft and `--crf` | `cli.rs:695` | After `camera_options`, before `render_camera_chapter` |
| Thumbnail | `cli.rs:1055` | Before the render |
| Glyph capture, retained and camera | `glyphs.rs:91` | Before the `match` on the camera |
| Glyph sheet | `glyphs.rs:146` | Not hooked, rightly: the sheet draws text, no chapter |

Two gaps, both handled:

- **The patch's own test has no camera chapter and only counts hook calls.** `kit-hook-routes-test.patch` adds `the_hook_governs_every_render_route`, which exercises seven render routes, including camera, with a behavioural check. It sets `max_output_bytes`, which `delivery_options` raises, so a hook that ran before the rebuild would fail it. It names no field the pin lacks.
- **LavenderPike's proof command uses `--max-frames N`.** A chapter longer than that returns `scene exceeded max_frames`, publishes nothing and exits non-zero, so an admitted plate would look like a failure. BUILD renders whole chapters at `64x36` instead.

No stage01 chapter calls `request_camera`, so no film route tests the camera path above the default. This is the only review of the patch beyond LavenderPike's own. We share a model and memory, so it is not independent.

**`1<<22` is a first setting, not a ceiling (Mail 3167).** It stays configurable: a film changes the one constant in `declare_plan_budget`. But the setting applies to the whole film, not to one scene, because the hook receives only the options and no chapter key. A per-scene budget would need the hook to receive the chapter. Admission stays separate from the checked addition and index-capacity checks, and no budget replaces the memory guard.

## Scope

**Native `fmn` facade only.** This makes no claim for the CLI, wasm or Python. These routes stay at the default with no option (LavenderPike's F5):

- `fmn-wasm/src/lib.rs:482`, `camera_player.rs:24`
- `fmn-studio/src/camera_bundle.rs:208`, `native/raster.rs:45`
- `fmn-python/src/portal_readback.rs:216`, `portal_studio.rs:116`
- `fmn-cli/src/lib.rs:3163` and `:4488`, `offline_cpu.rs:242`, `studio_live.rs:912`

## The tests, and what they can't show

- **Facade above the default (new).** Eight rows of 131,209 records come to 1,049,672 knots, 1,096 over the default. The default must refuse by name and publish nothing; `1<<21` must render. A clamp to the default fails this, which every earlier facade test would have missed. The knot count relies on my reading of `from_records`: one knot per record on a connected path.
- **Compiled workers.** Lowered limits only. Nothing tests the compiled route above the default, which would take a bundle of over a million knots. The film uses the live route, and the `fmn` CLI has no way to declare a budget.
- **`a_budget_moves_compaction_and_never_the_pixels`.** It runs on `RetainedFrameRenderer`, not the film's route with `VectorFrameCompiler` and earlier frames in flight. The lab probes exercise compaction on the facade, but they do not compare pixels across budgets on that route.

Not added, because no concrete failure called for them:

- a snapshot-form refusal check
- a thread sweep over {1, 4, 16}
- an e2e scenario (fm-fjq is not registered)

## 11_7 representative

`Kind::Turn` follows the film's `Turning`: the same plane, the quintic, `TURN_STEPS` 48, leading coefficient `2i`, the same quantised updater and `recolor`, on top of the painter as it fades.

- **What it leaves out:** the output curve's `set_points`, which holds no knots, and all but 9 of the film's up to 96 repaints. The film's `TURN_STEPS` is not lowered. The lab's short length only sets how many steps it samples.
- **A safeguard:** the film ignores a false `recolor`, which is how C1 hides. The lab compares row colours before and after, and its receipt says `TURN WROTE NOTHING` if no colour changed.
- **Expected by the desk model (`checks/turn_desk.py`, an upper bound):** the takeover is refused under both the default and `1<<21`, before any repaint. Under `1<<22` the plates kept per capture are `1, 3, 4, 5, 3, 4, 5, 2, 3, 4, 5, 2, 3`, with compactions at captures 4, 7 and 11.
- **The thin margin:** the peak is 4,124,800 knots, only 69,504 (1.7%) under `1<<22`. If root wants more headroom for this chapter, that is the number to weigh.
- **Past those 9 repaints:** the model repeats 2, 3, 4, 5 plates up to 96. That is reasoning, not evidence.

## Corrections to my earlier work

- Budget01 said the lab example was "already unformatted". That was false, an artifact of checking without the repo's `rustfmt.toml`. It is clean.
- Budget01's step 6, a three-frame repaint, is superseded: three frames never compact at `1<<22`.
- Budget01's segment, monotone, memory and cells8 wording is replaced by the sections above. Budget01 itself stays sealed and unchanged.
- The first `verify_patched_lock.py` said every other package was untouched, but it compared only name, version and source. It is corrected in place, since this folder was not yet sealed.

## Files

- **Read first:** `HANDOFF.md`, `BUILD.md`.
- **New patches:** `engine-tests-repair.patch`, `lab-fade-turn-probes.patch`, `kit-hook-routes-test.patch`.
- **Earlier patches, unchanged:** `engine-compiled-route-test.patch`, `lab-probes.patch`, `controls/mutant-*.patch`.
- **Hash lists:** `expected-engine.sha256`, `expected-kit.sha256`, `expected-lab.sha256`, `expected-cells4-draw.sha256`, `bundle.sha256`.
- **Build inputs:** `packages.tsv`, `engine-budget.config.toml.in`, `instantiate_config.py`, `repoint_lab.py`, `verify_patched_lock.py`.
- **Evidence:** `checks/`.
- **Not for applying:** `superseded-kit-slot/`, my kept trial.

Code edits were made by hand on scratch copies. ElevenLabs Oliver narration is unaffected, and no audio or wording changed. ASR is not listening, and sampled pictures are not continuous AV: this bundle offers neither, and no picture has been produced to review.
