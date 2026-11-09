# Scene execution hooks and failure recovery

The wheel installs `fmn_python.scene_execution` last, on the existing `Scene`
class. Direct extension embeddings may call `install_scene_execution(native)`
after installing their other playback adapters. The existing native animation
lowering, rational clock, updater release windows and output sinks remain the
execution authorities; this adapter does not implement another frame loop.

## Public play and wait hooks

A nonempty `Scene.play` prepares each builder in argument order, calls the
prepared animations' public `update_rate_info`, and invokes `self.pre_play()`.
It delegates to the existing playback path, then invokes `self.post_play()`
only after successful completion. Builder products and authored hook receivers
retain their identities. `Scene.wait` validates and resolves its duration and
condition, then uses the same pre/native-operation/post sequence.

Existing pre/post overrides, including cooperative `super()` calls, now run.
The default hooks notify the configured writer and increment `num_plays` once
per completed play or wait. A wait ended by a successful stop condition counts
as one completed operation; an empty play does not. Invalid ordinary arguments
are rejected before pre-hook work. Failed pre hooks never start the native
operation, and failed operations do not invoke post hooks. A failed post hook
is propagated once, without repeating animation finish or cleanup.

For example, the existing `end_at_animation_number=2` pre-hook check now stops
before the third play/wait, raising `EndScene`. The programmatic/console render
owners already treat that exception as normal early completion. It is not a
failure receipt or a claim that later scene code ran.

**Unchanged boundary:** Python `skip_animations`, `start_at_animation_number`
and `temp_skip` are not wired by this adapter into native sampling. Calling
their public hooks does not complete selective-render/skip integration.
Native final-state PNG still uses its existing runtime skip mode. Likewise,
this work does not make native playback call every public Scene per-frame
helper (`update_frame`, `progress_through_animations`, etc.). No optional
Studio transport, presenter loop, or partial-movie writer is silently enabled.

## Recovery of callback-owned transient state

Before the native callback table begins, the adapter captures preexisting
suspension, Python animating flags, and data/uniform lock sets for the affected
families and ancestors. Shared descendants are captured once, before either
animation can suspend them. Native-only callback slots remain untouched.

If execution fails during begin, native startup after begin, helper updates,
interpolation, finish or cleanup, begun unfinished callback owners are aborted
in reverse order where they expose an abort hook. Existing composition drivers
remain responsible for their internal child lifecycle. No artificial finish,
final endpoint interpolation, remover cleanup or scene publication is invoked
by the recovery layer.

After specialized unwinding returns, prior suspension and lock state is
restored. Original lock-set objects are retained, and previously suspended
children are not accidentally resumed. Recovery never invokes an additional
Python updater tick. Cleanup errors are attached as notes; a failing older
playback wrapper cannot replace the primary exception with its own abort error.
`KeyboardInterrupt` and `SystemExit` keep their original exception identities.

Same-scene play/wait reentry is refused before a second operation enters the
native runtime, including from pre/post hooks. Calls involving a different
Scene remain independent. Direct private `_play_animations` calls receive
failure ownership without public pre/post hooks. Temporary strong references
are released on the calling thread, preserving the unsendable proxy boundary.

## Limits of recovery

This is not transactional rollback of authored code. Geometry, scene time,
root membership, emitted bytes, callback closure state, and arbitrary user
attributes are not rewound. Successfully completed animation cleanup is not
undone. New objects introduced by arbitrary begin hooks are not part of the
pre-batch snapshot. Their specialized owner must release them. Native-only
state and callback-driver internal state remain owned by the native runtime
and the existing driver protocols.

An authored pre/post hook or writer hook may mutate and then raise; its side
effects are not rolled back. A user override that bypasses `super().play` or
`super().wait` also bypasses this adapter. Snapshot overhead is confined to
callback-driven animations, but its cost has not been benchmarked.

## Validation

`test_scene_execution_protocol.py` and `test_scene_hooks_protocol.py` test the
actual adapter against an explicit native-loop fixture. They exercise 41
orchestration cases, including a partial-begin leak control. Five hook cases
also fail against the previous adapter before passing with hook activation.
These are not native rendering or clock-validation receipts.

The installed-wheel gate additionally registers `tests/scene_execution.py`:
13 cases require real objects, suspension, callback failure windows, subsequent
scene operations, writer hooks, early termination, unpublished failed output,
and decoded Y4M frames compared across one and four threads. They must execute
against a same-checkout extension before native integration is considered
verified. No existing runtime suite or parity requirement is removed.
