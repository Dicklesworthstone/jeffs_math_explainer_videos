# Native live interaction and edit history

This is the CPython-free Proscenium/Studio path. Native `InteractiveScene`
editing now has bounded multi-step undo/redo, and `InteractivePreview` can
own a real running `Scene` instead of recreating only its visible records.
No renderer, frame scheduler, serialization format or external dependency is
added by these changes.

## Retain executable scenes in the preview owner

`InteractivePreview::from_scene(scene)` takes ownership of the native Scene.
Its original Stage handles, updaters, event listeners, pending input, runtime
configuration, clock and RNG stay with that owner. The constructor registers
Proscenium's existing editing listeners but runs no updater and advances no
time. It uses the existing in-memory state readiness check, not a durable
encode/decode round trip.

`InteractivePreview::from_interactive(editor)` moves an already-interactive
Scene, preserving its selection, clipboard, editing history and listener IDs.
Use this method for an existing `InteractiveScene`: extracting its Scene and
wrapping it again would register a second set of editing listeners.

The owner exposes `scene()` and `scene_mut()` for the existing native
`play`/`wait`/sink APIs. `into_interactive()` returns the complete editor when
the host is done with preview ownership. The same callback-bearing runtime
can therefore continue executing after transfer in either direction.

```rust
use fmn_mobject::Mobject;
use fmn_scene::{InteractiveScene, NullSceneSink};
use fmn_studio::InteractivePreview;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = InteractiveScene::default();
    let point = editor.stage_mut().add(Mobject::from_points(&[[0.0; 3]]));
    editor.stage_mut().add_to_scene(point)?;
    editor.stage_mut().add_updater(
        point,
        |stage, target| {
            stage.shift_many(&[target], [0.01, 0.0, 0.0]);
        },
        false,
    )?;

    editor.save_undo_state();
    editor.stage_mut().shift_many(&[point], [2.0, 0.0, 0.0]);
    assert!(editor.undo());
    assert!(editor.redo());

    let mut preview = InteractivePreview::from_interactive(editor)?;
    preview.scene_mut().wait(Some(0.1), &mut NullSceneSink)?;
    let editor = preview.into_interactive();
    assert!(editor.stage().contains(point));
    Ok(())
}
```

A preview's `frame_index()` remains its adoption/source-frame anchor.
Read `scene().time()` for the actual runtime clock after subsequent playback.
This avoids treating a historical preview index as a newly sampled frame.
`is_live()` distinguishes the two ownership paths. `seek_frame` refuses a
live owner with `LiveSeekRequiresReplay` before changing time or state:
changing a clock label alone would not execute the intervening callbacks.
Use actual Scene playback or an explicitly implemented replay path. An
explicit successful snapshot `reset` switches the owner to captured mode,
replacing the prior runtime and its executable state.

The captured-frame path is deliberately separate. `from_stage` still makes
an independent preview through the durable snapshot bridge and still rejects
updater-bearing snapshots: bytes cannot recreate executable closures. A Scene
restored from durable bytes with unresolved updater identities likewise fails
the live constructor's readiness check. It must use the existing explicit
updater-rebinding protocol before playback. A refused snapshot `reset` leaves
the current preview owner in place.

## Multi-step native editing

The native editor previously retained one overwritten undo snapshot and sent
both Primary-Z and Primary-Shift-Z to that same undo operation. It now keeps
past and future branches over Marionette's in-memory CoW snapshots.

Primary-Z undoes one edit; Primary-Shift-Z redoes one. Native callers can use
`undo()` and `redo()` directly; both return false when their branch is empty.
A new recorded edit clears the old redo branch. The snapshots restore native
records, graph edges, roots, native updater registrations, selection and the
scene-owned clipboard without executing an updater.

A grab or resize saves its pre-edit state on the first actual movement and
coalesces subsequent motion until the gesture ends. Merely arming and
cancelling a motionless gesture does not consume history or erase redo.
Colour-pick history begins when a valid source is chosen, not when the picker
is merely enabled. Restoring an edit clears active grab/resize/pick/sweep state
so a following pointer move cannot immediately overwrite the restored state.

The default limit is 50 transitions total across past and future.
`set_history_limit(n)` can adjust it; shrinking removes the more distant
states while keeping nearby transitions in both directions. Zero clears both
branches and disables new capture. `clear_history()` releases saved states
without changing the live geometry. `history_depths()` returns `(undo, redo)`
counts. Hosts making their own Stage edits can call `save_undo_state()` before
a mutation; ordinary built-in editor mutations record through the same path.

## Boundaries

History is native editing history, not a replay of scene execution. It does
not rewind the Scene clock, event sequence, RNG, emitted media, external I/O
or mutable values captured inside callback closures. Updater callables are
retained by shared identity; their external state is not deep-copied. The
history count limits retained snapshots, not an exact number of bytes.
Snapshot/restore costs and live-preview performance have not been benchmarked.

The live constructors are a reusable library integration point. They do not
by themselves wire the default CLI worker to a live scene factory, implement
a Python Studio worker, or serialize executable behavior for another process.
Existing worker-source and replay capability checks remain in force. The
Python portal's separate scene-history adapter is unchanged.

## Acceptance

`crates/fmn-scene/tests/interactive_history.rs` contains 16 actual-engine
cases for branch order/limits, keyboard redo, gestures, native roots and DAG
edges, clipboard copies, callback identity and clock preservation.
`crates/fmn-studio/tests/live_interaction.rs` contains nine actual-engine
cases for ownership, queued events, frame capture, unresolved-updater refusal,
failed reset, live-seek refusal, and continued callback execution after
undo/redo and transfer.
Neither suite substitutes a fake Stage or a second dispatcher.

Run from a fully provisioned checkout using the repository's pinned toolchain:

```sh
cargo test -p fmn-scene --test interactive_history
cargo test -p fmn-studio --test live_interaction
```

These assertions require execution before native integration is considered
verified. In the authoring environment neither Cargo nor rustc was present;
local attempts could not start either test binary. Source/diff and published
blob-identity checks are not compilation or test-pass receipts.

A GitHub Actions PG-5 run at implementation checkpoint `889a8cc` compiled
both changed native crates and passed the existing one/four/sixteen-thread
scene and certified-engine corpus checks. That run did not execute these
feature-specific tests; the later trimming and live-seek changes also need
their own same-checkout validation. The full mandatory gate was not green.
