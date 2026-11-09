# Review 267: updater callback fix in plate-fade candidate 264

cc3, 2026-11-08, about 05:40Z. This is an independent source review of CC2's correction 264
against the fmn pin `a399d626`. I wrote only this file. I ran no build, test, Clippy, render or
probe, and edited no candidate, canonical file, TODO, Beads entry or memory. Narration
(ElevenLabs Frederick) is unchanged. Nothing here is a picture, listening, fine-sync or
final-AV acceptance.

## Verdict

**The fix is correct and minimal. I see no reason to change `9bb07b3f…`.**
- **The cause.** build262's only error was E0369 at `draw.rs:1890`. Test 1's watcher was
  registered with `add_updater`, whose callback is `(stage, Mob)`, so `dt` was the Mob.
- **The fix.** Candidate 264 registers it with `add_dt_updater` and `move |s, _, dt|`. That
  matches the pinned signature.
- **Observation order.** At the pin, the watcher now reads the opacity lane after every
  animation, the plate ramp included, has interpolated, and before the frame is captured. So test
  1 observes exactly the captured value.
- **Exclusions.** The finish pass and the registration call both use dt = 0, so the `dt > 0.0`
  guard keeps them out, as the test intends.
- **Scope.** Nothing outside this one test changes.

## 1. Files

All paths are relative to `production-repair17/` (Q). I hashed them when the handoff appeared,
and again before writing this file; they were unchanged.

| Role | File | sha256 |
|---|---|---|
| **candidate** | `cc2-plate-fade-candidate264-draw.rs` | `9bb07b3ff030d1ac0885d1382097bde596e8cae419d86b51933e12e7eeed8218` |
| CC2 handoff | `cc2-plate-fade-correction264.md` | `bf52a1c1801dad67f126d994d92ae6727a71c98dd5342927e7632d9f47347278` |
| previous (frozen) | `cc2-plate-fade-candidate260-draw.rs` | `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064` |
| probe (unchanged) | `cc2-plate-fade-probe246.rs` | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` |
| root's failure evidence | `review-build262-failure266/tests.stderr` | read only |

- **Root's recipe268** (`review-repair-recipe268.json`) binds 264 as `draw.rs`. It binds the
  other seven files exactly as recipe262 did: boundary52, the two 238 copies, mod256,
  rigidity247, gallery249 and probe246. All eight hashes match on disk.
- **The error output.** `tests.stderr` reports one compiler error, E0369 at `draw.rs:1890:27`
  ("binary operation `>` cannot be applied to type `fmn::prelude::Mob`"). The `[package`
  TOML message above it comes from a malformed-manifest test fixture in a cargo git checkout
  of a dependency, and does not concern this crate.

## 2. The change (260 → 264)

`diff` gives exactly two changed lines, both in `a_raster_plate_fades_through_its_opacity_lane`:

```
1887:  stage.add_updater(       →  stage.add_dt_updater(
1889:  move |s, dt| {           →  move |s, _, dt| {
```

- **Non-test code.** It is byte-identical to 260, and 260's was byte-identical to 246's.
  - So the repair, the fade helpers, the frame and cue schedule, and the fade gate's view of
    draw.rs (which reads non-test code only) are unchanged.
  - My review257 conclusions on 246 and 260 carry over unchanged, including the gate mirror over
    the composed tree and the six controls.
- **Test count:** draw.rs has 23 `#[test]` in both 260 and 264.

## 3. Pinned runtime signatures

All in `crates/fmn-mobject/src/stage.rs` at `a399d626`:

- **`add_updater`** (`:1757`): `(mob, updater: impl FnMut(&mut Stage, Mob) + 'static, call:
  bool) -> Result<UpdaterId, StageError>`. It registers `UpdaterFn::NonDt`.
- **`add_dt_updater`** (`:1775`): `(mob, updater: impl FnMut(&mut Stage, Mob, f64) + 'static,
  call: bool) -> Result<UpdaterId, StageError>`. It registers `UpdaterFn::Dt`.
- **Dispatch.** `update_mob` passes dt only to the Dt kind (`UpdaterFn::Dt(f) => f(self, mob,
  dt)`; `NonDt(f) => f(self, mob)`).

**The corrected watcher fits.**
- It has three parameters, and `dt: f64` makes `dt > 0.0` well-typed.
- `s.roots()` exists on the arena (`stage.rs:1037`).
- The closure captures only an `Rc` clone and two `Copy` handles, so `'static` holds.

**Every other closure registration is consistent.** The candidate's other four updater
registrations are production19 code that has already compiled, and each uses the shape of its
kind:

| Line | Registration | Callback |
|---|---|---|
| `:1002` | `add_updater` (Painter anchor) | `|s, _|` |
| `:1151` | `add_updater` | `|s, _|` |
| `:1281` | `add_dt_updater` (clock) | `|s, _, dt|` |
| `:1684` | `add_updater` (an existing test watcher) | `|s, _|` |

`plate_fade`'s `UpdateFromFunc::new_alpha` closure `(&mut Arena, Mob, f64)` matches
`fmn-anim/src/update.rs:59`. It is non-test code and type-checked in build262.

The probe registers no updater of its own, so CC2's callback audit agrees with mine.

## 4. Opacity observation order

**Per play frame, on the native Rust path** (`crates/fmn-anim/src/frame.rs`):

1. **Prepare.** For each animation in list order, `update_mobjects(dt)` then
   `interpolate(alpha)` (`:556–558`). Each ramp is listed after its fade, so it writes the lane
   last.
2. **Complete** (`complete_play_frame`, `:618–638`):
   - scene updaters run through `stage.update_at_time(plan.dt, …)`, with dt equal to the clock's
     1/fps;
   - then `boundary`;
   - then the frame is captured (`FramePacket::freeze`).

   The `updaters_completed` skip is set only by the host-portal hook
   `mark_scene_updaters_complete` (`:435–443`), which native Rust callers do not use.

**So the watcher reads the captured value.** It is a dt updater on a rooted mob, so it runs in
step 2, after the ramp's write and before capture. On a 1 s fade at 10 fps:
- frame k (k = 1…10) reads `smooth(k/10)` going in, or `1 − smooth(k/10)` going out;
- the test indexes this correctly as `(k % 10 + 1) / 10`, with k counted from 0 over all 20
  logged frames.

**The excluded passes all have dt = 0:**
- the immediate `call = true` pass at registration;
- each play's finish epilogue, `update_scene_mobjects(stage, 0.0)` (`:298–308`, `:666–669`).

The guard keeps them out of the log. The fade-out's finish also removes the group before that
pass, so `roots().contains(&group)` is false there anyway, a second guard. Exactly 10 + 10 frames
are logged, which is what the test asserts.

## 5. Limits

- **The crate is still uncompiled after the fix.** Build262 stopped at type checking. Borrow
  checking, codegen and Clippy (`-D warnings`) have not yet run on the new test code or the probe,
  so further errors are possible there. Root's guarded build268 is the check.
- **The probe is still unrun.** Its limits, and review257's L1 on the first-frame bound under
  linear-light blending, stand.
- **No acceptance here.** A build, test or probe pass is not a picture, listening, fine-sync or
  final-AV verdict.
