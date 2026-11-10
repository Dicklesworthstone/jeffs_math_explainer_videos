# Complex864 slowdown: bounded independent source and process diagnosis

Independent Codex / BlackThrush. 2026-11-10 approximately 12:22–12:28 UTC. Read-only SSH and source inspection; no process changes, jobs, profiling or engine changes.

## Recommendation

After actual864 terminal receipt, use a new guarded successor with unchanged844 binary, source,849 narration, score21, 3840×2160/60, CRF16/veryfast and camera/image route. Raise fixed render threads to16 and explicitly allow16 distinct physical cores (for this observed topology CPUs0–15 are distinct cores on socket0). Recheck actual allowed CPU set and host load at admission. Preserve all original disk, work, target, memory and exclusive-lane safeguards and output gates. Use a longer bounded native timeout, proposed24hours: observed rates imply roughly13.6–22.9hours for16504frames at four cores if sustained; a16-core speedup is plausible but NOT measured or promised. A12-hour bound would still assume improvement or faster later scenes. No current job should be changed or duplicated.

Do not reduce plate resolution, frame rate, camera route, AA quality or scene content. Do not replace the encoder merely for speed: evidence points upstream. No performance benchmark helper was created because the frozen bounded CLI does not expose a supported successfully finalized clip.

## Measured process evidence

Native PID782368 at elapsed10575s had CPUtime11:34:19,393%CPU,RSS384768KiB and affinity0–3. Four running frame-coordinator worker threads occupied CPUs0–3; main/coordinator/emitter threads mostly waited. Encoder782421 at elapsed10572s had CPUtime00:03:18,1.8%CPU,RSS1239000KiB. Encoder rchar about44.44GB corresponds approximately3572NV12frames at12,441,600bytes/frame; this is an approximate consumed-frame proxy, not a finalized output frame count. Parent independently measured host load about8.22/8.89/10.92. Host is dual EPYC7282,32physical/64logical CPUs. Evidence supports compute-bound native frame rasterization rather than encoder saturation or a deadlock. No sampled stack establishes the exact hot raster function.

Actual852 timeline starts09_1 at36 and09_2 at8333, so current~3570frames are still inside the long first explanatory sequence. Do not extrapolate identical complexity for the remainder.

## Actual source route and caching

Frozen844 topic main.rs declare_plan_budget forces a camera for native image fields. Boundary.rs near493 builds Painter with Func::Bump. draw.rs669–676 selects PLATE_RASTER at360samples/scene-unit; Picture::build at723 creates ImageMobject::from_resource. This is the smooth colored backdrop. Camera removal is not an equivalent cheap switch: planar RetainedFrameRenderer::render rejects non-vector programs with CameraRequired.

Painter caches endpoint colors and returns immediately when its input tuple is unchanged (draw.rs~1000–1050). Therefore repeated expensive mathematical color evaluation is not established as the cause.

Important correction to an earlier intermediate message: the vector pipeline does use PixelTileCache, but this film's camera path does not use that vector cache. RetainedFrameRenderer::prepare_with_camera freezes the camera scene, PreparedCameraFrame::render calls render_camera_scene, and retained.rs632–634 invokes ThreeDJob::render_into. This camera raster path does not consult PixelTileCache. It retains geometry/image resources, not the previous composed frame's tile pixels. Repeated camera-frame raster cost is a concrete source explanation consistent with CPU evidence, but the contribution of each primitive is unmeasured.

## Thread determinism and memory

three_d.rs1535–1603 distributes disjoint row bands through a mutex queue; each band uses the same render_band operation. Tests three_d_worker_scratch_is_warmed_once_and_reused and moved_light_changes_the_lighting_frame_and_threads_do_not compare raw output at1/4/16threads, with an additional mixed-vector camera test immediately afterward. These are source observations; I did not rerun them or establish that the topic's235test receipt includes engine tests. They support trying16threads without changing image semantics, not an actual whole-film byte-equality claim. Encoder thread planning can also differ, so encoded MP4 SHA need not remain identical even if raw raster pixels do.

Scratch slots scale with worker count; output raw frames are not multiplied once per band thread. Native rendering.rs828–852 accounts retained camera frames as render_teams+1, adds estimated in-flight buffers and64MiB, and refuses a plan exceeding max_resident_bytes. Defaults:512MiB max_resident_bytes and two frames in flight (rendering.rs364–366). Keep this admission unchanged; do not automatically raise it if16threads is rejected. Current renderer RSS is below512MiB; encoder RSS is separate. This is not a peak-memory guarantee.

## Bounded clip/resume limitation

Frozen explainer CLI exposes --max-frames, but no start/end/resume frame range. CLI routes that option to a profile output path; engine rendering.rs1063–1064 returns an error when the next frame exceeds the limit. Kit propagates the error; RenderSink::drop cancels output and joins cleanup (1350–1359). Thus a successfully closed decodable bounded MP4 is not an established contract. No benchmark should claim framemd5 equality from an artifact this path may abort. It also cannot resume the already consumed864 frames through supported CLI. Do not alter engine merely to manufacture a benchmark.

## Frozen identities and remaining gates

864 runner SHA149d7d59cd00427426198f1b1e223b0034b871cc5c92ef14890ce8a9461ad9ac;844binarySHAe75258731b1786ae3182ef9eb21a7903a9e6c0bed6242413335f7dd233955384; boundarysourceSHAcd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e. Existing runner explicitly restricts affinity to four CPUs and native timeout14400seconds. Preserve it and its eventual receipt.

A successor still requires actual terminal stable completion, full schedule/PCM checks, full decode, final875-equivalent extraction and actual root/independent picture review before master/publication. This report supplies no listening, fine-sync, final visual, full-AV or release acceptance. No speedup or new render is claimed.
