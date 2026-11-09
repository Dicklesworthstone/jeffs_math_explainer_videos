# Review 257: plate-fade candidate 246 (review241 M1)

cc3, 2026-11-08, about 05:25Z. This is an independent source review of CC2's investigation,
candidate and probe against production19 and the fmn pin `a399d626`.
- I wrote only this file.
- I ran no build, test, Clippy, render or probe, and made no canonical, TODO, Beads or memory
  edit.
- Narration (ElevenLabs Frederick) is unchanged.
- Nothing here is a picture, listening, fine-sync or final-AV acceptance.

## Verdict

- **Cause and repair semantics: sound.** I reproduced the cause independently from the pin,
  before reading CC2's note. The ramp's begin, per-frame and finish order is correct, and so is
  its interaction with updaters. The schedule cannot change.
- **F1 (Medium, integration blocker; confirmed):** source test 3,
  `every_engine_fade_in_the_film_carries_the_plate_ramp`, passes on production19 but fails on
  root's current build238 workspace. Root has since stopped build256. **Candidate 246 should not
  be integrated as it stands.**
- **F1 is resolved by candidate 260, final `923ad2ef…` (§8).** Its non-test code equals 246's.
  On the composed build tree, all four vector-only engine fades have vector-text targets, every
  image fade site goes through the repaired helpers, and all six regression controls hold. I find
  no reason to change the final hash.
- **L1 (Low, probe robustness):** the probe's first-fade-frame bound (≤ 0.05) would fail a
  correct fix if the renderer blends in linear light. This fails safe, as a false failure rather
  than a false pass.
- **Probe gaps:** three, none blocking (§6).

## 1. Files

All paths are relative to `production-repair17/` (Q). I hashed the files when CC2's note
appeared, and again after this review was written; they were unchanged.

| Role | File | sha256 |
|---|---|---|
| base `draw.rs` (production19; unchanged in build238) | `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/draw.rs` | `cd394e5a451bf111…` |
| **candidate** | `cc2-plate-fade-candidate246-draw.rs` | `aab61199e7a0dea9c5482ee2aa127f94c0c5bb06e2edbb5202350db964a1ea56` |
| **probe** | `cc2-plate-fade-probe246.rs` | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` |
| CC2 handoff | `cc2-plate-fade-investigation246.md` | `1792f4de82f14ca42b976bbdbf04b8310ab4eba73f1dcbab0b107cd5578972ed` |

My own diff, base → candidate, has three hunks:
- `image_records`, `plate_fade`, and the ramp in `fade_in_all`;
- the ramp in `fade_out_all`;
- three tests and the probe module.

It removes no production line, which matches the note.

## 2. Cause (agrees with CC2; also corrects review241's attribution)

- **No rgba column, so the fade can't reach the plate.**
  - Production plates are raster: `PLATE = PLATE_RASTER` (base `draw.rs:636`).
  - `Picture::build` adds one `ImageMobject` quad (`:683–704`).
  - Its record schema is `point`, `im_coords` and `opacity`, with no `*rgba` field
    (fmn-library `image.rs:296–320`).
  - Both fades prepare their invisible copy with `set_family_opacity_zero`
    (fmn-anim `transform.rs:291`, `:358`). That calls `set_family_opacity`, which writes only
    the alpha of fields whose names end in `rgba` (fmn-mobject `align.rs:192–215`).
  - So start and target carry the same `opacity` lane, and the plate is at full strength on
    every fade frame.
  - `fade_out`'s remover then takes it off after the play (fmn-anim `fading.rs:94–95`;
    `frame.rs:298–308`).
- **The lane does reach pixels.** The renderer reads `opacity` for every image record (fmn-render
  `retained.rs:828–836`), so a downstream write to it shows on screen.
- **Review241's attribution.** Review241 guessed at RowStrips paint. RowStrips and Blend use
  `stroke_rgba` and would fade, but the film does not use them, so CC2's correction stands.

## 3. Repair semantics, checked at the pin

The ramp is `UpdateFromFunc::new_alpha(owner = m, …)`. In each play it is listed directly after
`fade_in(m)` or `fade_out(m)`, with the same `.rt(rt)`.

- **Closure.** `new_alpha` passes `rate(time_spanned(α))` to its closure, whose first argument is
  the arena (fmn-anim `update.rs:56–108`). The rate is `AnimConfig::default().rate_func`, which is
  `RateFunc::default()` (`animation.rs:519–533`), the default the fades also use.
- **Config.** `with_config` replaces the whole config. The candidate sets `name` and
  `final_alpha_value` and keeps the defaults otherwise: `remover` false,
  `suspend_mobject_updating` false and `run_time` 1. The `.rt(rt)` after it then sets the run
  time.
- **Begin, in list order** (`frame.rs:195–216`).
  - The fade begins first, so its start and target copies are taken while the lane is still at
    its resting 1.
  - The ramp's begin then runs its zero interpolation (`animation.rs`, the `after_begin` contract
    at `:770–779`). That sets the lane to 0 for a fade in, or 1 for a fade out. No frame is
    captured at begin.
- **Each frame.** Animations interpolate in list order (`frame.rs:520–562`), so the ramp writes
  last. It does not matter whether the Transform rewrites the lane with its 1 → 1 lerp or skips
  it as matching data; I did not trace which. Either way the captured value is the ramp's.
- **Finish** (`frame.rs:298–308`). This runs, per animation in list order, `finish` and then
  `clean_up_from_scene`; then one zero-dt updater pass. `finish` lands on `final_alpha_value`
  (`animation.rs:783–788`).
  - **Fade in:** the lane ends at 1.
  - **Fade out:** the remover takes the group off first. The ramp's finish then puts the lane back
    at rate(0) → 1 − 0 = 1.
  - **No flash:** no frame is captured between those steps, and a non-remover's cleanup does
    nothing. So the group stays removed with its lane at 1, ready for any later show.
- **Accessors:** they exist with the shapes the candidate calls.
  - `set_scalar_lanes(mob, &str, f64, recurse)` (`align.rs:239`);
  - `Entry::image_resource` (`stage.rs:443`);
  - `Stage::family` (`stage.rs:1139`).
- **Updaters:**
  - **The Painter's updater.** It sits on `anchor` and recolours through `set_image_resource`,
    which replaces only the resource. So it cannot reset the ramp, and that holds in the final
    zero-dt pass too.
  - **Clearing:** `leave` clears the updater and removes the anchor before fading, and so does
    the gallery's inline `leaving` loop (base `gallery.rs:928–931`).
  - **The ramp's owner:** it is the plate group, which has no updaters. The ramp neither suspends
    nor resumes updating.

## 4. Timing

- **No duration can change.** A play lasts as long as its longest member (`plan_play`). Each ramp
  has its fade's exact run time, and no play is added. So segment lengths, cue waits and stage
  time cannot change.
- **No new call sites.** The 17 registered chapters' call sites are untouched, because the ramp
  rides inside the two helpers.
- **Tests:** unit test 2 checks the frame counts at 60 fps for nine run times. The probe's phases
  A and B must match phase C's [10, 5, 10] frames, and C runs production19's bodies.
- **Possible cost:** `UpdateFromFunc` keeps the Unclassified signature, so these segments never
  run frame-parallel (`update.rs:1–5`). That may slow renders, but cannot change frames. This is
  unmeasured.

## 5. Coverage of fade paths

- **Chapters.** CC2 found that in the 17 registered files, the engine's `fade_in(` and
  `fade_out(` occur only in the two draw.rs helpers, and that `fade_all(` never occurs. My mirror
  confirms this on production19.
- **Kit (my addition).** No kit fade reaches a plate:
  - `Kit::finish` (`kit.rs:201`) only holds and lays the music stem; it has no fade.
  - The kit's own `fade_all` (`play.rs:61`) is called only by the animatic (`animatic.rs:132`).
  - The kit's other fades act on vector marks only: `predict_pause`'s "?" text (`props.rs:219`)
    and the telemetry triangles.
- **Plates outside Painter:**
  - **Go through the helpers:** gallery tiles (`gallery.rs:930`), the_bump's preview `portrait`
    (`the_bump.rs:368`) and counting's Turning-field exit all pass through `fade_in_all` /
    `fade_out_all`.
  - **Nested plates:** `image_records` takes the whole family, so a plate inside a larger faded
    group is covered.
  - **Unchanged on purpose:** plates added by `add_to_scene` with no fade (counting's Turning
    field entering) still appear in one step, as designed.

## 6. Tests and probe

### F1 (Medium): test 3 rejects the already-accepted 06 and 08 repairs

`every_engine_fade_in_the_film_carries_the_plate_ramp` reads every registered chapter from
disk. It requires `fade_in(` and `fade_out(` to appear exactly once in draw.rs and nowhere
else. I mirrored it (read-only Python, same module list and split marker) over three
compositions:

| Composition | Result |
|---|---|
| production19 + candidate 246 | passes (17 files) |
| build238 (production19 + `boundary-label-repair52.rs`, both review-repair 238 copies, `mod-review-repair236.rs`) + candidate 246 | **fails:** `loop_sums` `fade_out(` ×2; `one_derivative` `fade_in(` ×1 and `fade_out(` ×1 |
| the same + `rigidity-review-repair247.rs` | **fails** the same way (rigidity adds no engine fade) |

- **The sources:**
  - one_derivative 238's `hand_over` (`:196–208`) has one `fade_out(` and one `fade_in(`;
  - loop_sums 238 has two `fade_out(` calls for the c and D_n labels (`:646`, `:648`).
- **Why the guard is too strict:** both chapters draw no plate, so the guard's invariant ("no
  plate fades without the ramp") still holds there. Only the test is too strict.
- **What a fix needs:** it must allow those exact vector-only sites and keep the guard's power
  for plate chapters. The fix CC2 is preparing for root (precise vector-only exceptions in 06 and
  08, no image constructors there, plate bypass checks kept) is the right shape. I will review it
  as candidate 260.

### Tests 1 and 2

- **Test 1** (`a_raster_plate_fades_through_its_opacity_lane`). It reads the lane the renderer is
  given on every frame, through a frame-only updater. That updater runs after the animations, so
  it sees the ramp's value. It also checks:
  - that all six records agree;
  - the in and out curves against `smooth`, within 1e-6;
  - that the lane goes back to 1;
  - that the group is removed.

  This is a valid unit-level proof. It relies on the existing `scene`, `grid`, `lane` and `CELLS`
  test helpers (base `draw.rs:1502–1566`), which exist.
- **Test 2** (`a_plate_fade_takes_the_frames_a_vector_fade_takes`). It is valid, but relative: it
  compares a plate fade with a vector fade under the candidate's helpers. The anchor to
  production19 is the probe's phase C.

### The probe

The design is good:
- **Defect control:** phase C must still pop, or the probe declares itself blind.
- **Fade ends:** frame equality on both sides of each fade end. The last fade-in frame must equal
  the first hold frame, and the frame after a fade out must be bare background, which catches a
  flash.
- **Full strength:** it must match production19's pixel for pixel.
- **Frame counts:** clock, Y4M and artifact counts are all asserted.
- **Indexing:** frame k of a fade is Y4M frame `start + k − 1`, and `ground` is the frame before
  `a_in`; both are correct.

**L1 (Low).** The bounds assume blending in display (gamma-encoded) space. I computed the
expected shares for both cases, with the probe's colours (ground `#0E1116`, plate 0.85 grey,
smooth rate, 10 frames, BT.709 luma). The light-to-dark gap is about 172 limited-range luma
levels, so quantisation is about ±0.006.

| Frame | Opacity | Display-space share | Linear-light share | Probe bound |
|---|---|---|---|---|
| first fade-in (k = 1) | 0.0086 | 0.009 | **0.055** | ≤ 0.05 |
| mid (k = 5) | 0.5 | 0.500 | 0.714 | 0.25–0.75 |
| last (k = 10) | 1 | 1.000 | 1.000 | ≥ 0.99 |

If the camera path blends in linear light, the vector control square fails the first-frame
bound, and so does a correctly fading plate. The mid band then holds with only 0.04 to spare.
Because the vector control fails first, the error is visible and points at the bound. Root
should read the printed share rows before treating a probe failure as a fix failure. Raising the
first-frame bound to 0.08 would cover both cases.

**Probe gaps (none blocking):**
- It runs at 640×360 and 10 fps only. Unit test 2 covers the 60 fps frame counts, but no pixel
  check runs at the film's route size.
- It never re-shows the same plate after a leave. The lane goes back to 1, and a re-show starts
  again from 0; this holds by semantics but is not exercised.
- It has no nested-family plate and no multi-plate play like the gallery tiles' eight-plate
  `leaving` list. `image_records` and the per-mob ramp cover those by construction.

## 7. Not established

- **Compilation is unproven.** I could not settle four of the API facts by reading:
  - the `fmn::animation::UpdateFromFunc` facade path;
  - whether draw.rs's `Arena` name is the same type as the closure's arena parameter;
  - `fmn::core::rate::smooth`;
  - the probe's `render_with_fs`, `VirtualFs` and `RenderOptions` field paths.

  Clippy is not run. CC2's own list of these risks stands.
- **The film is not inspected.** I measured no pixel and judged no picture or motion. Whether the
  now-real fades read well in each chapter is root's picture decision after re-render. That
  includes the gallery's 0.25 s tile fades, and the 10 → 11 join, which now ends on an empty
  frame.
- **Performance is unmeasured.**
- **Untouched by this candidate:** listening, fine sync, captions, final AV, and Frederick
  narration and timing.

## 8. Correction 260: final `923ad2ef…` (added about 05:35Z)

**Files.**

| File | sha256 |
|---|---|
| `cc2-plate-fade-candidate260-draw.rs` (final; CC2 calls it the final hash) | `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064` |
| `cc2-plate-fade-correction260.md` | `af489fe7c12a342c5451132c077ab86fa8ccf4290f35263ffee6810bc0db517e` |
| `cc2-plate-fade-probe246.rs` (unchanged) | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` |

- **Earlier drafts.** I first saw the draft `c182e336…`. CC2 reports that the final differs from
  it only by the test-only `ReviewedFades` type alias, and I reviewed the final.
- **The composed tree checked.** `review-repair-recipe262.json` was not on disk yet, although
  `review-repair-build262.py` reads it. I used recipe256's seven non-draw files instead, and all
  seven hashes match on disk:
  - `boundary-label-repair52.rs` `dc80c541…`;
  - `loop-sums-review-repair238.rs` `8a13e8a1…`;
  - `one-derivative-review-repair238.rs` `7a4cc01a…`;
  - `mod-review-repair256.rs` `e1f4d1b8…`;
  - `rigidity-review-repair247.rs` `0bcfbc7d…`;
  - `gallery-review-repair249.rs` `d57e003a…`;
  - the probe `c0c56084…`.

  To these I added the final 260 as `draw.rs`, with preserved214 for everything else.
- **Recipe262 matches.** It appeared after the checks ran. It binds exactly this composition: the
  same seven files and hashes as recipe256, with `draw.rs` = `923ad2ef…`. All eight hashes
  match on disk.

**What changed from 246.**
- The non-test code (1,439 lines) is byte-identical to 246's. So §§2–5 above (cause, ramp
  semantics, updaters, timing and coverage) carry over unchanged.
- Compared with production19's draw.rs, the file still only adds lines.
- Every change from 246 is inside `mod tests`. The old gate is replaced by `ReviewedFades`,
  `VECTOR_ONLY_FADES`, `PLATE_WORDS` and `fade_gate`, plus the updated registry test and a new
  control test.

**The acceptance questions**, checked with a read-only Python mirror of `fade_gate`. The mirror
decodes the gate's pinned Rust string literals from the candidate itself rather than retyping
them, and it reads the registry from `mod-review-repair256.rs`.

1. **The four vector-only engine fades have vector targets.**

   | File | Fade | Target and its only binding | Constructor |
   |---|---|---|---|
   | loop_sums 238 | `:646` `fade_out(…, centre_name, …)` | `:607` `kit.tex(stage, "c", …)` | `Tex::new(…).build` (`kit.rs:480–495`) |
   | loop_sums 238 | `:648` `fade_out(…, diagonal_name, …)` | `:618` `kit.tex(stage, "D_n", …)` | the same |
   | one_derivative 238 | `:200` `fade_out(…, m, …)` in `hand_over` | its one call `:543` with `&[scale_note]`; `:272` `kit.text(…)` | `Text::new(…).build` (`kit.rs:403–405`) |
   | one_derivative 238 | `:204` `fade_in(…, m, …)` in `hand_over` | `zoomed` `:537–540`: two `kit.text_left(…)` | `text_left` calls `text` (`kit.rs:449–451`) |

   - Each target is bound once, and `hand_over(` occurs twice: its definition and its one call.
   - Neither file contains any of the 14 plate words outside comments.
   - I read `kit.rs` locally. It was last changed in `f9293808` (2026-10-27), with a clean tree,
     but it is not among root's binding254 files.
2. **Every actual image fade site goes through the repair.** On the composed tree, all 17
   registered files pass `fade_gate`:
   - the only other engine fade lines are draw.rs's two helper lines;
   - each ramp appears exactly once;
   - `gallery-review-repair249.rs`, a plate chapter, has no engine fade of its own, so its plate
     exits go through the helpers.

   §5's kit check still applies, since no kit fade reaches a plate.
3. **The six regression controls hold.**
   - On the real composed files, the gate accepts loop_sums, one_derivative and draw.
   - It refuses all six cases:
     1. an engine fade of `painter.group` in an unlisted file;
     2. the same fade added to a listed file;
     3. a `Painter::build` in a listed file;
     4. `scale_note` no longer built by `kit.text(`;
     5. a second binding of `centre_name`;
     6. draw without its falling ramp.
   - The 246 rule fails on exactly loop_sums and one_derivative, which reproduces F1.

**Limits of the source scan** (listed, not blocking):
- **Plate words.** A listed file stays safe only while every plate constructor needs one of the 14
  words. Today it does: the only public helpers that build plates are reached as
  `Painter::build` / `Picture::build` / `portrait…`, and `group` and `portrait_colors` cannot
  make a plate on their own. A future helper with a neutral name could slip past the word scan,
  but it would still fail if it added a fade to a listed file, or changed a listed fade line or
  target pin.
- **Comments.** `//` stripping is per line, so a `//` inside a string literal hides the rest of
  that line from the word scan. Fade lines inside comments still count.
- **Text, not types.** The gate reads source text, not types. Renaming, re-binding or
  re-formatting a listed line fails the gate and needs a fresh review, which is the intended
  behaviour.

**Probe and compile.**
- The probe is unchanged, so L1 stands: read the printed share rows before treating a failed
  first-frame bound as a failed fix.
- 260 is uncompiled and Clippy is not run. The alias addresses a likely `type_complexity` lint.
- CC2 expects build262 to pass 233 tests, plus the ignored probe run on purpose. I have not
  verified that count.
- Root's compiler run and native probe are the next evidence, not this review.

**Conclusion.** 260 resolves F1 without touching the repair. I see no reason to change the final
hash `923ad2ef…`, and the probe stays `c0c56084…`.
