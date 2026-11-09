"""Production initialization contract; run against either real entry point."""
import importlib
import types

import manimlib as m
from fmn_python import initialization


native = getattr(m, "_native", m)
assert vars(native).get("_FMN_PORTAL_RUNTIME_STATE") == "ready"
flags = ("FUNCTIONAL_COLOR", "DECIMAL_AUTHORING", "SCENE_RENDERING", "FADING", "SCENE_PLAYBACK", "SCENE_STATE",
         "STREAMLINE_ANIMATION", "TRACED_PATH", "INTERACTIVE_EDITING",
         "INTERACTION", "CONTROL_EVENTS", "COLOR_SLIDERS", "SCENE_EXECUTION",
         "ANIMATION_UPDATERS", "MOVEMENT", "ROTATION", "EMBEDDED_SHELL")
for flag in flags:
    assert vars(native).get("_FMN_" + flag + "_INSTALLED"), flag

identities = (m.Scene.play, m.Scene.wait, m.Scene.render, m.Mobject.remove_updater,
              m.DecimalNumber.__init__, m.DecimalNumber.set_submobjects_from_number,
              m.Mobject.set_color_by_rgb_func, m.Mobject.set_color_by_rgba_func,
              m.FadeTransform.begin, m.Animation.begin, m.Transform.begin,
              m.turn_animation_into_updater, m.InteractiveScene.on_key_press,
              m.InteractiveScene.embed, m.InteractiveScene.checkpoint_paste)
for _ in range(3):
    assert initialization.initialize(native) is native
    assert identities == (m.Scene.play, m.Scene.wait, m.Scene.render, m.Mobject.remove_updater,
                          m.DecimalNumber.__init__, m.DecimalNumber.set_submobjects_from_number,
                          m.Mobject.set_color_by_rgb_func, m.Mobject.set_color_by_rgba_func,
                          m.FadeTransform.begin, m.Animation.begin, m.Transform.begin,
                          m.turn_animation_into_updater, m.InteractiveScene.on_key_press,
                          m.InteractiveScene.embed, m.InteractiveScene.checkpoint_paste)
assert importlib.import_module("manimlib.animation.fading").FadeTransform is m.FadeTransform
assert importlib.import_module("manimlib.animation.transform").Swap is m.Swap

# Invalid installers are discovered before any installation hook can run.
# Use an empty module, never mutate production classes to inject a failure.
original_steps = initialization._STEPS
fake = types.ModuleType("_incomplete_portal")
fake._FMN_ANIMATION_SEMANTICS_INSTALLED = True
try:
    initialization._STEPS = (*original_steps, ("rendering", "_missing_installer_for_test"))
    try:
        initialization.initialize(fake)
    except ImportError as error:
        assert "missing portal runtime installer" in str(error)
    else:
        raise AssertionError("incomplete runtime package was accepted")
finally:
    initialization._STEPS = original_steps
assert fake._FMN_PORTAL_RUNTIME_STATE == "failed"
try:
    initialization.initialize(fake)
except ImportError as error:
    assert "failed" in str(error)
else:
    raise AssertionError("a partially initialized module was retried")
for state in ("initializing", "unknown-version"):
    fake._FMN_PORTAL_RUNTIME_STATE = state
    try:
        initialization.initialize(fake)
    except ImportError as error:
        assert state in str(error)
    else:
        raise AssertionError("reentrant/unknown initialization was accepted")
empty = types.ModuleType("_no_shared_protocol")
try:
    initialization.initialize(empty)
except ImportError as error:
    assert "shared animation semantics" in str(error)
else:
    raise AssertionError("uninitialized native protocol was accepted")
assert "_FMN_PORTAL_RUNTIME_STATE" not in vars(empty)
print("production portal initialization: complete, idempotent, identity-preserving and fail-closed")
