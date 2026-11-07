# Render performance source review17

## Decision

**Static domain-function caching is already implemented in production16. Do not spend time adding another scene-level static-field cache.** The concrete remaining renderer opportunity is **camera-frame reuse**: this film forces the camera route, whose exact-bound engine path rerasterizes each camera frame and does not use the vector route's persistent PixelTileCache. This is a source-grounded opportunity, not a measured speedup or an authorized engine change.

The fastest currently qualified path is disjoint frozen chapters on TRJ/CSD with the same native renderer,4K60 and final encoder contract, after their chapter review/admission gates. Existing111-frame parity supports using TRJ; it does not establish optimal thread count or narrated60fps performance. No source edits, engine changes, launches or benchmarks were performed by this review.

## Already cached: avoid a duplicate optimization

The authoritative source is continuous-quality16/draw.rs, not the older canonical source with20-step/12-step quantization. Candidate hash80bf6cf2… was formatted by build16. I ran rustfmt **to stdout only**, removed its filename banner, and obtained exactly the build16 source-copy hash `cd394e5a451bf1114bead7a086660d8f22cc282e8bb66de4b17b833a98bd9af1`. Thus the inspected logic is tied to the actual qualified binary8dc5e8c4….

- Painter::build_styled stores plain/enhanced color arrays for at most the current two functions. On each updater invocation, `(enhance, blend, from, to)` is compared to last; an unchanged tuple returns immediately. A static hold therefore does not re-evaluate the complex function, interpolate the color array or replace the image resource per frame.
- Each newly used function pays two endpoint evaluations; there is also an initial paint(f, enhance0) during construction. New tween values interpolate the full endpoint arrays and create a new RGBA8 image. Endpoint evaluation cache is per Painter and retained only for from/to; this is not a cross-chapter global cache.
- Picture::build uses one retained ImageMobject quad for PLATE_RASTER. It is **not** millions of native colored rectangles. The texture resolution is360 samples per scene unit (e.g.6.4×6.4 picture gives2304×2304 texels). Reducing this changes the approved field and is outside an exact-pixel optimization.
- Engine ImageTable::intern caches immutable textures by complete descriptor/content digest and shares Arc<Texture>. Static texture transfer decoding is already reused. New RGBA content during fades incurs new decoding; static content does not.

## Actual missing reuse in the exact-bound engine

`raster-quality03/main.rs:declare_plan_budget` sets opts.camera whenever absent, including chapters containing only vector content. main.rs SHA-256 `19a9e1051360caacba25a9276d1f32a4b7b94700bb25a4c72ca663805619f798` matches production16's source-copy receipt.

The following engine files in the local preserved geometry export match every corresponding hash in fmd-source-binding01.json; they are not assumptions based on the currently developing upstream tree:

1. fmn/src/rendering/pipeline.rs NativeCompiler::Camera calls prepare_with_camera. NativeFrameStages::rasterize takes the Camera arm, calls frame.render(team.threads()), then convert_into. The persistent VectorWorker pixel cache is used by the Vector arm instead.
2. fmn-render/src/retained.rs PreparedCameraFrame::render allocates a new raw RGBA16F framebuffer; render_camera_scene constructs ThreeDDraw references and a new ThreeDJob, then calls render_into. The RetainedFrameRenderer owns a PixelTileCache, but this camera path never passes it into rasterization. Its planar render path explicitly calls render_into_cached.
3. fmn-render/src/three_d.rs render_tile clears scratch to the camera background, determines edge sampling and traverses/rasterizes intersecting draws again. Image quads become textured triangles; vectors sharing the frame use the same3D draw sequence.

Therefore **cached model/image data does not mean cached final pixels**. Even a static backdrop still goes through frame rasterization/compositing, and even fully unchanged captures do not get whole-frame raster reuse on this path. Some geometry and texture compilation is retained, so saying “everything is rebuilt” would overstate the issue.

## What the measurements do and do not establish

- TRJ parity helper's timed render subprocess: **242.938418s for111 frames**,4K,1fps,silent,CRF0,veryfast,16 allowed CPUs/threads. All111 decoded YUV frames match the CSD reference; MP4 container hashes differ. This includes scene construction, field evaluation, rasterization, conversion, encoder and sink completion.
- CSD04 guard spans **298.604383s** and CSD05 spans **394.480881s**, both with4-CPU affinity. These are broader guard intervals including probe/image extraction and checks; they are not comparable per-stage render timing or a controlled4-versus16-core benchmark.
- No inspected receipt decomposes texture preparation, camera rasterization, conversion, encoder backpressure or initialization. **It is not established that encoding or fill is the measured bottleneck.** Source shows repeated camera raster work; its percentage remains unknown.
-242.94/111≈2.19s per output frame, so naive100000-frame extrapolation is about60.8hours. This is a warning/scheduling bound, not a forecast:1fps changes updater sampling, the expensive one-time portrait setup is amortized differently, and final narration/chapter mix differs. A60fps clip must not be substituted with a1fps render repeated60times.

## Ranked exact-pixel opportunities and limits

**1. Immediate scheduling, no source change:** retain production17's native quality route, partition whole qualified chapters across hosts, and reuse only chapters meeting unchanged-final17-reuse-review.md's source/audio/timeline/stem contract. Keep per-host guard coverage and stable final encoder/stream parameters; do not let a second render compete with an admitted build on that host. More threads is not proven faster;16 was parity-tested only for the diagnostic. This is the only currently evidenced production-path acceleration without new qualification work.

**2. Best targeted engineering proposal: reuse fully identical camera frames.** A bounded worker-local cache of the exact raw/converted pixels can skip rasterization/conversion when the entire frozen scene, camera, render/output policy and image content are identical. Preserve every original output sequence number/frame and audio clock; feed identical pixels to the same encoder. Key must include painter order, geometry/style/uniforms, image descriptor+content, camera/light/clip/depth, sample policy, background and output conversion settings. Never key only the domain image or tracker values. This should benefit static teaching holds; no speedup is claimed until a paired benchmark and raw-frame/PCM identity test pass. It requires engine-owner authorization and implementation, outside this review.

**3. Finer camera-tile cache, if whole-frame hit rate is low:** reuse unchanged tiles when a small dot/text region moves. Must invalidate by every overlapping draw and preserve order/AA/depth/compositing. The existing vector PixelTileCache is a reference, not a switch already usable by the camera path. More invasive than whole-frame caching; defer until stage/hit-rate data justify it.

**4. Scene-level allocation savings during actual fades:** endpoint interpolation currently allocates Vec<Vec<Srgb>>, then picture_resource converts to RGBA8 and hashes/interns it. A fused same-order loop/reused buffers could remove temporary color allocation while retaining exactly the existing floating arithmetic and u8 rounding. This affects changing fields only; static holds already return. Reassociating floating mixes, replacing the cached endpoint formula with a mathematically equivalent direct portrait, or interpolating quantized u8 textures can change pixels and requires exact evidence. No basis to rank this ahead of camera reuse without profiling.

Do not change AA, field density, resolution/fps, quantize fades, switch lossy encoder/preset, or switch to the planar route simply to obtain its cache. Camera versus planar rendering can differ in sampling/compositing; vector-only chapters may be candidates for a separately proved route change, but that is not currently qualified. Likewise precompositing just a background layer can change rounding/order/depth interactions; whole identical-frame reuse has a much cleaner equality contract.

## Small next diagnostic once the active build permits it

Use the existing exact production binary to profile one representative short4K60 interval containing both an unchanging hold and a moving overlay/field transition, with stage timing/encoder wait accounting if available. Compare render input/output frame and PCM hashes, then an alternating same-host candidate/base run if a cache is implemented. Keep initialization timing separate. This is a proposed bounded test, not launched or measured here. Do not hold all useful already-qualified chapter renders waiting for speculative optimization.

## Evidence hashes

- `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/main.rs`: `19a9e1051360caacba25a9276d1f32a4b7b94700bb25a4c72ca663805619f798`
- `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/continuous-quality16/draw.rs`: `80bf6cf27757662868f4ddcd6df02c3488bdb2c45dc25194f558a36815dfabeb`
- `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/continuous-quality16/build16.py`: `9ce0f48dcccc65fe394ecb38da4c042c1e1e5c82708fc4e6e0e92095207390af`
- `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/trj-picture-parity16cores01-result.json`: `68e2d17141e16a9fac7d7f6919af97172ae3b1a7e250c45e102e84f6ab5aea92`
- `/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/engine/crates/fmn/src/rendering/pipeline.rs`: `634c22b57bd59d225e793a2201330729136ba162e42bdc9a146863c2d758573a`
- `/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/engine/crates/fmn-render/src/retained.rs`: `c2124bcc4514303dfd38a748a7bc0b0c93f68a559b21a447c9ec803d5725ffae`
- `/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/engine/crates/fmn-render/src/table.rs`: `14de8d56e3357a1d714c72fb5f121abeb9185fe334a118f7105a2a857078177d`
- `/Users/jemanuel/scratch/jmev-geometry-native-20261104-01/engine/crates/fmn-render/src/three_d.rs`: `61ae25888f2f38602ad8a244f706600e11c520c96e0b56780bf5203200214451`

Local preserved engine AGENTS was read; its referenced /data/projects/AGENTS.md is absent on this workstation. No engine work was attempted. Memory was not used as performance evidence.
