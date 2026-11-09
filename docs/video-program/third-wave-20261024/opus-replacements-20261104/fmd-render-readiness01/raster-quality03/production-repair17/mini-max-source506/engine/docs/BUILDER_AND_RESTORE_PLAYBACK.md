# Animation builders and restoration

The installed wheel normalizes top-level `.animate(...)` builders into the
actual Animation returned by `build()` before entering the existing Scene
playback implementation. This removes the older builder-specific option
filter without creating another animation engine.

```python
scene.play(
    square.animate(
        run_time=2,
        rate_func=linear,
        time_span=(0.5, 1.5),
        suspend_mobject_updating=True,
    ).shift(2 * RIGHT)
)
```

Authored `build()` implementations and `override_animate` products keep their
identity. Builders are prepared in argument order, after Python has evaluated
all argument expressions, preserving dynamic target lookup. The existing
composition constructors still prepare nested builders. Custom `path_func`,
name, time span, updater suspension, final alpha and remover options now reach
the full Transform protocol rather than the short builder-specific allowlist.

Ordinary native Transform specifications do not currently apply general
final-alpha/remover overrides. Those options select the existing callback
Transform lifecycle, including for explicit Transform/ReplacementTransform
objects. Stock default transforms keep their native route. Specialized native
kinds retain their own defaults; this adapter does not claim to repair every
animation kind's endpoint configuration.

## Restore uses the same Transform protocol

`Restore` is a subclass of the existing `Transform` class, retaining the same
published class object and all qualified aliases. Construction captures the
saved mobject by reference, matching the pinned Reference. Reassigning or
clearing `mobject.saved_state` afterwards does not retarget an already-created
Restore. Mutating the captured saved object itself remains observable.

```python
square.save_state()
square.shift(3 * RIGHT)
scene.play(Restore(square, run_time=1, path_func=my_path))

scene.frame.save_state()
scene.play(scene.frame.animate.scale(0.5).shift(RIGHT))
scene.play(Restore(scene.frame))
```

Native Transform handles ordinary restoration. Authored paths and hooks use
its existing Python lifecycle over native object operations. Camera Restore
enters the existing camera choreography path, including nested and mixed
plays, without replacing the scene's camera core. Missing saved state raises
`Trying to restore without having saved`; malformed non-Mobject saved targets
are rejected before playback. Final-alpha and remover settings use the same
route as other ordinary transforms.

## Installation and validation scope

The wheel initializer installs `fmn_python.playback.install_scene_playback`
after the shared extension animation definitions exist. It changes methods
and Restore's base, not exported class identities or the wildcard namespace.
Repeated installation leaves later authored replacements intact. Consumers
that deliberately bypass the wheel initializer with ExtensionFileLoader can
call that installer on their native module explicitly. The native Rust API
and the extension's independent embedded initialization are unchanged.

This is playback integration, not a new renderer, geometry kernel, frame
clock or runtime dependency. It is not a claim of complete portal parity or
measured performance equivalence. Standard native paths remain available;
callback-selected cases can have additional interpreter overhead.

`test_animation_builder_playback.py` and `test_restore_playback.py` test the
production installers and extracted production constructors with fixture
object storage/dispatch. Those tests do not render pixels. The installed-wheel
gate includes `builder_playback.py` and `restore_playback.py`, with sixteen
real-extension cases covering actual motion, authored paths, timing,
suspension, nested playback, saved-target identity, hooks, camera state and
cleanup. They require a built extension and are not replaced by fixture tests.
