# Animated flow lines and temporal traces

The wheel installs the adapters on the existing public classes. Direct
extension embeddings may call `install_streamline_animation(native)` after
`install_scene_playback(native)`, and `install_traced_path(native)` after the
native module has initialized. These adapters do not provide another renderer,
ODE integrator or frame clock.

## AnimatedStreamLines

Each original flow line now exposes its real `line.anim` (`VShowPassingFlash`),
`line.virtual_time` and signed `line.time`. `line_anim_config` forwards the
existing flash options, including custom easing, taper, family lag and time
windows. The default is linear easing and a flash width of one. A per-line
`run_time` conflicts with the duration derived from virtual time/rate multiple
and is rejected. The native geometry and named-stream lag draws are unchanged.

The displayed group contains the original lines. Normal Mobject updater order,
suspension, recursive updates and additional updaters work. Helpers update
before each flash interpolation. Changes to an existing flash object's runtime
or a line's phase counter are observed on the next tick. Line membership is
captured when the effect is constructed; create a new effect for a new family.
Zero-virtual-time lines hold alpha zero rather than dividing by zero.

Removing the owned updater, including through a parent's recursive
`clear_updaters`, aborts begun flashes, restores their stroke widths and releases
only their owned suspension. It does not perform scene removal. Errors in an
owned flash unwind the other owned flashes and preserve the first exception.
Copies of the updater cannot drive or cancel the original effect. A copied
visual group is not automatically a second independent flow controller.

## Behavior note: timestamped TracedPath / TracingTail

The old portal followed the Reference's frame-count approximation: it ignored
`time_per_anchor` and estimated history length from the most recent `dt`.
A long frame could therefore remove nearly the entire trail, while a tiny frame
could bring back old samples. This implementation deliberately replaces that
behavior with elapsed-time windows and the declared anchor spacing.

`time_per_anchor` now controls the temporal grid. The source callable is read
once per positive updater step. Grid points are linear interpolations between
observed positions; the adapter never calls authored code at invented times or
claims to reconstruct unobserved curved motion. The current endpoint remains
visible between grid points. The left edge of a finite trail is interpolated at
`current time - time_traced`; short windows do not retain expired history.
`None` or positive infinity means an unbounded trace; zero keeps one point.

Curve smoothing and style interpolation still use the existing VMobject
methods. `TracingTail` accepts a Mobject or callable and initializes a finite,
stationary prehistory with its usual tapered width/opacity defaults. All public
class identities and virtual geometry hooks are preserved.

This changes trace geometry relative to the former per-frame implementation.
Choose a spacing near the intended sample interval for similar density, or a
smaller spacing for denser interpolation (not additional source observations).
The trace is sample-dependent for curved motion; this is not cross-FPS output
identity or certified rendering. Spacing changes apply prospectively and do not
resample already retained history. `traced_points` is the displayed anchor
mirror, not a writable source history. Trace counters/history are not rewound
by arbitrary external edits to `time`; construct a new trace to reset it.

Finite windows retain only the required grid history plus boundary samples.
There is a 100,000-anchor resource ceiling: oversized windows or long unbounded
traces fail explicitly rather than silently downsampling. A large update skips
already-expired grid points before allocating them. Temporal state is published
after native curve/style calls succeed, but arbitrary authored geometry hooks
are not transactionally rolled back.

## Verification boundary

The protocol suites test real adapter code against explicit storage/flash
fixtures. `temporal_visualization.py` additionally requires native RK45 lines,
real flash/trace geometry, scene-updater execution and decoded rendered output.
Only running that installed-wheel suite proves those integration assertions;
protocol tests and syntax checks are not native rendering receipts.
