#!/usr/bin/env python3
"""Source census behind REVIEW.md. Reads text only; compiles and runs nothing.

    python3 -I -B route_census.py BEFORE_CRATES AFTER_CRATES KIT_SRC

BEFORE_CRATES: root's staged engine `crates` folder, the base both patches apply to (read only).
AFTER_CRATES:  a copy of that tree's `crates/fmn` and `crates/fmn-render` with
               engine-frame-table-limits.patch then engine-camera-limits.patch applied, matching
               both bundles' expected-engine.sha256 (checked here).
KIT_SRC:       root's staged explainer_kit `src` folder (read only).
Paths print relative to these roots. Every PASS is a fact about source text, not about behaviour.
A section that cannot find its text fails as a whole; route_census.negative-control.out is this
script run with BEFORE_CRATES given as AFTER_CRATES, where the patch-specific sections must fail.
"""
import hashlib
import re
import sys
from pathlib import Path

if len(sys.argv) != 4:
    sys.exit(__doc__)
BEFORE, AFTER, KIT = (Path(a).resolve() for a in sys.argv[1:])
HERE = Path(__file__).resolve().parent
BUNDLES = HERE.parents[1]
R = "fmn-render/src/"
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def text(root, rel):
    path = root / rel
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def block(source, header):
    """The brace-balanced block that starts at the first occurrence of `header`."""
    start = source.index(header)
    depth, i = 0, source.index("{", start)
    while True:
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start:i + 1]
        i += 1


def line_of(source, needle):
    return source[: source.index(needle)].count("\n") + 1


b3, a3 = text(BEFORE, R + "three_d.rs"), text(AFTER, R + "three_d.rs")
rendering = text(AFTER, "fmn/src/rendering.rs")
pipeline = text(AFTER, "fmn/src/rendering/pipeline.rs")


def stacked():
    """the AFTER copy is the stacked result both bundles expect"""
    expected = {}
    for bundle in ("engine-secondary-budget01", "engine-secondary-camera-budget01"):
        for row in (BUNDLES / bundle / "expected-engine.sha256").read_text().splitlines():
            digest, rel = row.split(maxsplit=1)
            expected[rel.strip()] = digest  # the camera bundle's rows override the frame-table bundle's
    mine = {rel: d for rel, d in expected.items() if rel.startswith(("crates/fmn/", "crates/fmn-render/"))}
    ok = []
    for rel, d in mine.items():
        path = AFTER / rel.removeprefix("crates/")
        if path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest() == d:
            ok.append(rel)
    check(len(mine) >= 8 and len(ok) == len(mine), f"{len(ok)} of {len(mine)} expected files match in AFTER")


def defaults():
    """defaults the patches must preserve"""
    for rel, header in ((R + "fill.rs", "impl Default for MonoTableLimits"),
                        (R + "bin.rs", "impl Default for BinningLimits"),
                        (R + "plan.rs", "impl Default for RenderPlanLimits")):
        check(block(text(BEFORE, rel), header) == block(text(AFTER, rel), header), f"{header}: identical text before and after")
    before_default = block(b3, "impl Default for ThreeDPreparationLimits")
    after_default = block(a3, "impl Default for ThreeDPreparationLimits")
    new_line = "            max_curve_bin_entries: 1 << 24,\n"
    check(new_line in after_default and after_default.replace(new_line, "") == before_default,
          "ThreeDPreparationLimits default: the five old values unchanged, one new line max_curve_bin_entries: 1 << 24")
    check("const CURVE_BIN_MAX_ENTRIES: u64 = 1 << 24;" in b3 and "if total > CURVE_BIN_MAX_ENTRIES ||" in b3,
          "before: bins left unbuilt when total > 1 << 24")
    check("limits.max_curve_bin_entries.min(u64::from(u32::MAX))" in a3 and "if total > curve_bin_entry_limit(budget.limits)" in a3,
          "after: the same strict comparison against min(declared, u32::MAX), so the default boundary is unchanged")
    for src, name in ((b3, "before"), (a3, "after")):
        bins = block(src, "fn build_curve_bins(")
        first_return = bins.index("return Ok(None);")
        check("curves.len() < CURVE_BIN_MIN_CURVES" in bins[:first_return],
              f"{name}: fewer than 32 curves returns before any entry limit is read")
    zero = re.findall(r"max_curve_bin_entries: 0,", a3)
    check(len(zero) == 4, f"{len(zero)} test literals set max_curve_bin_entries to 0")
    check("frame_table_limits: FrameTableLimits::default()," in rendering and "camera_limits: ThreeDPreparationLimits::default()," in rendering,
          "RenderOptions defaults both new fields to the engine defaults")


def propagation():
    """propagation on the facade (AFTER)"""
    counts = {k: pipeline.count(k + "(") for k in ("set_plan_limits", "set_frame_table_limits", "set_camera_limits")}
    check(counts == {"set_plan_limits": 4, "set_frame_table_limits": 2, "set_camera_limits": 2},
          f"pipeline.rs: {counts} (serial and compiled-worker sites for each route)")
    check(rendering.count("NativeFramePipeline::with_limits(") == 1, "rendering.rs: one with_limits construction, inside RenderSink")
    check("ThreeDJob::new_with_limits(" in text(AFTER, R + "retained.rs"), "retained.rs: the camera scene builds its job with the renderer's limits")
    kit_cli = text(KIT, "cli.rs")
    camera_branch = kit_cli[kit_cli.index("let mut opts = camera_options(&path, opts)?;"):]
    check(camera_branch.index("adjust(&mut opts);") < camera_branch.index("render_camera_chapter("),
          "kit cli.rs: the film hook runs after camera_options rebuilds the options, so declared budgets survive")


def refusal_name():
    """refusal name on a worker failure (pre-existing path, both patches)"""
    check("let NativeFrameJob { frame, mut output } = job;" in pipeline,
          f"pipeline.rs:{line_of(pipeline, 'let NativeFrameJob { frame, mut output } = job;')}: rasterize owns the reservation, so an Err via ? drops it")
    for fn in ("pub fn capture(", "pub fn capture_compiled("):
        body = block(pipeline, fn)
        check(body.index("stream.reserve()") < body.index(".reserve(sequence)") and ".map_err(RenderError::Emitter)?" in body,
              f"{fn[7:-1]}: stream permit first, then the emitter slot, whose error maps to RenderError::Emitter")
    surface = block(rendering, "fn surface_failure(")
    check("matches!(&error, RenderError::Pipeline(_))" in surface and "Emitter" not in surface.replace("emitter", ""),
          f"rendering.rs:{line_of(rendering, 'fn surface_failure(')}: the original error is recovered only for RenderError::Pipeline")
    compiled = text(AFTER, "fmn/src/rendering/compiled.rs")
    check("matches!(&error, RenderError::Pipeline(_))" in compiled and "RenderError::Emitter(" not in compiled,
          "compiled.rs: the same Pipeline-only recovery")
    check("EmitterConfig::new(layout, plan.frames_in_flight, 0)" in rendering, "the emitter has frames_in_flight slots, as many as the stream has permits")
    emitter = text(BEFORE, "fmn-output/src/emitter.rs")
    reserve = block(emitter, "pub fn reserve(&self, sequence: u64)")
    check(reserve.index("state.failure.clone()") < reserve.index("wait_for_slot"), "emitter reserve: a recorded failure is returned before and after every wait")
    guard = block(emitter, "impl Drop for ReservationGuard")
    check("EmitterError::ReservationAbandoned" in guard and "notify_all()" in guard, "a dropped reservation records ReservationAbandoned and wakes waiters")
    check("When every slot is outstanding, the next dispatch blocks" in emitter, "the module note: a slow sink blocks the next dispatch")
    runtime = text(BEFORE, "fmn-runtime/src/pipeline.rs")
    raster = runtime[runtime.index("catch_unwind(AssertUnwindSafe(|| stages.rasterize(value, team)))"):]
    err_arm = raster[raster.index("Ok(Err(source)) => {"):]
    check(err_arm.index("cancellation.cancel();") < err_arm.index("Completion::Error"),
          "runtime: cancellation is raised only after rasterize has returned, so the reservation is already dropped")
    check("ReservationAbandoned" not in pipeline and "ReservationAbandoned" not in rendering,
          "no facade code maps ReservationAbandoned back to the worker's error")


def plan_copies():
    """plan copies (pre-existing, both routes)"""
    for root, name in ((BEFORE, "before"), (AFTER, "after")):
        vf = text(root, R + "vector_frame.rs")
        capture = block(vf, "pub fn capture(")
        # The only branch before the copy is the camera-primitive refusal; nothing tests for change.
        refusal = capture.index("return Err(RetainedFrameRendererError::CameraRequired")
        copy = capture.index("Arc::make_mut(&mut self.plan).sync")
        check(capture[:refusal].count("if ") == 1 and "if " not in capture[refusal:copy] and "revision" not in capture[refusal:copy],
              f"{name} vector_frame.rs: past the camera refusal, make_mut runs on every capture, before sync, with no change test")
        check("generation is copied only when a subsequent capture mutates it" in vf, f"{name}: the doc comment promises a copy only on mutation")
        ret = text(root, R + "retained.rs")
        check("plan: self.plan.clone()," in block(ret, "pub fn prepare_with_camera(") and "    plan: RenderPlan,\n    scene: PreparedCameraScene," in ret,
              f"{name} retained.rs: every camera capture deep-clones RenderPlan into the frame")
    check("#[derive(Debug, Clone)]\npub struct RenderPlan {" in text(AFTER, R + "plan.rs"),
          "RenderPlan's Clone is derived: every Vec and HashMap is copied with infallible allocation")


def arena_rows():
    """affine arena rows (AFTER)"""
    arena = text(AFTER, R + "arena.rs")
    check("        self.buf.push(value);\n" in block(arena, "pub(crate) fn put(&mut self, value: T)"), "Pool::put pushes: infallible, doubling growth")
    extend = block(arena, "pub(crate) fn extend(")
    check("self.put(value);" in extend and "try_reserve" not in extend, "Pool::extend calls put per row, with no reservation")
    check("self.buf.clear();" in block(arena, "pub(crate) fn begin_frame(&mut self)"), "begin_frame clears length and keeps capacity")
    engine = text(AFTER, R + "engine.rs")
    for pool in ("segments", "stroke_segments", "draws", "joins"):
        check(bool(engine) and f"arena.{pool}.try_reserve" not in engine, f"no fallible reservation for arena.{pool}")
    check("arena.draws.put(Some(Draw {" in engine and "PreparedStroke::prepare_into(\n                    &mut arena.stroke_segments," in engine,
          "a Draw row per instance and prepared stroke rows per stroked draw")
    check("mono_limits" in engine and "frame_tables.binning" not in engine and "max_working_bytes" not in engine,
          "engine.rs reads only the monotone limits; no working-byte budget is consulted where draw and stroke rows are pushed")


def camera_curves():
    """camera projected curves (pre-existing, both)"""
    for src, name in ((b3, "before"), (a3, "after")):
        check("let mut curves = Vec::new();" in src and "budget.reserve_rows(output, PreparationRows::ProjectedCurves, 1)?;" in src,
              f"{name} three_d.rs: a fresh Vec per vector, reserved one curve at a time")
    check("output.try_reserve_exact(additional)" in block(a3, "fn reserve_rows<T>("),
          "reserve_rows reserves exactly, so a full Vec grows by one row each time")


for section in (stacked, defaults, propagation, refusal_name, plan_copies, arena_rows, camera_curves):
    print(f"\n-- {section.__doc__}")
    try:
        section()
    except (ValueError, OSError) as error:
        check(False, f"section stopped: {type(error).__name__}: {error}")

print()
print("RESULT " + ("PASS" if not failures else f"FAIL {len(failures)}"))
