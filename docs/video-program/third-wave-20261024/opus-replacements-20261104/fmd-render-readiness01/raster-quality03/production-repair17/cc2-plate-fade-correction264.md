# cc2 plate-fade correction 264 (bead .39)

This fixes build262's compile error in candidate260's test module. Nothing was compiled, built,
rendered, run, committed or deleted. The frozen 260 and 246 files are untouched. Paths are
relative to this folder.

## Files

| File | sha256 |
|---|---|
| `cc2-plate-fade-candidate264-draw.rs` (new; install as `src/chapters/draw.rs`) | `9bb07b3ff030d1ac0885d1382097bde596e8cae419d86b51933e12e7eeed8218` |
| `cc2-plate-fade-candidate260-draw.rs` (frozen, preserved) | `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064` |
| `cc2-plate-fade-probe246.rs` (unchanged; install as `src/chapters/plate_fade_probe.rs`) | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` |

## Cause

Build262 reported `draw.rs:1890` E0369: in `a_raster_plate_fades_through_its_opacity_lane`,
`if dt > 0.0` compares a `Mob`. At pin `a399d626`:

- `crates/fmn-mobject/src/stage.rs:1757` `add_updater` takes `FnMut(&mut Stage, Mob)`, so the
  second closure argument is the updater's mobject, not dt.
- The dt kind is `add_dt_updater` (`:1775`), which takes `FnMut(&mut Stage, Mob, f64)`.
  `update_mob` (`:2247`) passes dt only to that kind.
- The facade `Stage` reaches both through `DerefMut` (`crates/fmn/src/lib.rs:1520`).
- On a frame, `complete_play_frame` calls `update_at_time(1/fps)`. The finish pass
  (`update_scene_mobjects(…, 0.0)`) and the `call = true` registration both pass dt = 0. So the
  test's original intent holds with the dt kind: log frame passes only.

## Exact diff (260 → 264)

```diff
@@ -1884,9 +1884,9 @@
             assert_eq!(image_records(stage, group), vec![image], "the plate's one image record");
             let watch = stage.arena_mut().add(VMobject::new());
             let seen = std::rc::Rc::clone(&log);
-            stage.add_updater(
+            stage.add_dt_updater(
                 watch,
-                move |s, dt| {
+                move |s, _, dt| {
                     if dt > 0.0 && s.roots().contains(&group) {
                         seen.borrow_mut()
                             .push(s.get(image).unwrap().buffer.read_column("opacity").unwrap());
```

Nothing else changed:

- **Non-test code:** lines 1–1439 are byte-identical to 260 and 246 (`1ad7b87a…`). The runtime
  fix, the fade helpers and every frame and cue schedule are unchanged.
- **Formatting:** rustfmt is clean with the probe under the install names.
- **Fade gate:** draw.rs non-test code still has one `fade_in(` and one `fade_out(`, so the gate's
  view of draw.rs is unchanged.

## Test count

Unchanged. draw.rs has 23 `#[test]` in both 260 and 264; no test was added or removed. Build
expectations stay as root set them for build262: 233 passed, plus the one ignored pixel probe
run explicitly.

## Callback audit

- **Probe246:** it registers no updater and calls no update-closure API. It uses only Painter's
  own production updater, reached through `Painter::show`/`leave`. No correction is needed, and
  the original is preserved unchanged.
- **Candidate, every closure added beyond production19:**
  - `plate_fade`'s `UpdateFromFunc::new_alpha(owner, move |s: &mut Arena, _: Mob, a: f64| …)`
    matches `crates/fmn-anim/src/update.rs:59` (`FnMut(&mut Stage, Mob, f64)`). It is non-test
    code and type-checked in build262.
  - The test watcher above is the only wrong one.

## Untested limits

- **Uncompiled after the fix.** Build262 stopped at type checking, so later compiler passes and
  Clippy may not yet have run on all of the new test and probe code. Further errors are possible
  there, and root's next guarded build is the check.
- **The probe is still unrun.** Its limits stand as in investigation246. A build pass is not a
  picture verdict.
