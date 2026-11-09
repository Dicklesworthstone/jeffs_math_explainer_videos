# Fading playback in the Python wheel

## Grouped and piecewise crossfades

`FadeTransform` now runs its complete grouped transition protocol. It retains
the original source object, an independent copy of the target, starting and
ending families, and the actual target object to install after cleanup.
`FadeTransformPieces` aligns family counts first, including unequal glyph
counts, but does not morph one source glyph outline into another target glyph.
Each branch retains its own geometry and is interpolated across time.

```python
from manimlib import *

class Crossfade(Scene):
    def construct(self):
        source = Text("a+b")
        target = Text("x+y=z").shift(RIGHT)
        self.add(source)
        self.play(FadeTransformPieces(source, target, stretch=False, dim_to_match=0))
```

Authored `ghost_to`, `create_starting_mobject`, family/lag and interpolation
hooks execute through the existing Choreo callback boundary. `path_func`,
`path_arc`, time spans, rates, final alpha and remover configuration use the
existing Transform machinery. The ghost endpoints are prepared before the
single alpha-zero interpolation; nonzero `rate_func(0)` is not overwritten.

Cleanup removes the temporary group and source, restores the source from the
saved object selected at construction, and adds the actual target unless
`remover=True`. Reassigning `source.saved_state` after construction does not
retarget this restoration. Cleanup of an unbegun or aborted transition does
not install its target. Invalid starting copies that alias the live animation
family and camera-frame crossfades are rejected.

## Vector-only fades and retained geometric fades

`VFadeIn`, `VFadeOut`, and `VFadeInThenOut` expose an executable Python lifecycle.
Only stroke and fill opacity are written: live point motion, width, color and
other state are not reset from a starting copy. Like the native VFade kernel,
opacity getters select the first starting opacity lane, and setters broadcast
that value times the rated alpha. VFadeOut uses `1 - alpha`; the then-out class
retains its there-and-back curve and default final-alpha behavior.

Per-member writes do not recurse into descendants, so each member's own lag
controls its opacity. Authored animation hooks and mobject opacity setters or
getters select callback execution. Later instance/class replacements are
recognized, including a base-class method called through a shipped subclass's
`super()`. Unmodified stock vector fades retain their native execution path.

```python
self.play(FadeOut(shape, remover=False, final_alpha_value=0.5, rate_func=linear))
```

The ordinary `FadeOut` constructor now accepts retained and partial endpoints
and records its default remover flag correctly. Nondefault ordinary FadeIn
and FadeOut endpoint/remover settings route through the shared Transform
callback lifecycle because native geometric fade lowering fixes those defaults.
Other specialized native animation kinds keep their own routing. Ordinary
native defaults and native vector final-alpha/remover parameters remain native.

Callback vector lifecycle and fade scene-failure handling release suspension
acquired by the animation without running extra updater callbacks during
unwind or reviving already-suspended descendants. Errors retain their original
exception. Aborting is not finishing: it does not promise to roll geometry
back or publish a target.

## Boundaries and validation

The wheel initializer installs `fmn_python.fading.install_fading` on the
existing public class objects before builder playback normalization. Qualified
aliases and wildcard exports remain unchanged. Consumers deliberately using a
direct ExtensionFileLoader instead can invoke the installer on their module.
The independent embedded extension initialization and native Rust API are
unchanged.

Python FadeTransform/FadeTransformPieces now use callback coordination over
native mobject operations; their interpreter overhead is not measured. This
is not a new renderer, frame clock, geometry kernel, runtime dependency, or
claim of full portal parity.

`test_fading_protocol.py` and `test_fade_effects_protocol.py` exercise production
installers with fixture object storage, interpolation and scene dispatch.
Their negative controls reproduce inert vector interpolation and the old
FadeOut constructor refusal. They do not establish native-rendering acceptance.
The installed-wheel gate also runs `fading_semantics.py`, with sixteen cases
using actual mobjects, paths, text families, compositions, native Scene.play,
callback exceptions, restoration and suspension. These require the compiled
extension; protocol tests are not substitutes for that acceptance run.
