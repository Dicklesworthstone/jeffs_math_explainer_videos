"""Production Python protocols with fixture record storage and native drivers.

These tests deliberately do not claim native geometry or rendering acceptance.
The companion creation_semantics.py suite exercises the compiled extension.
"""
import abc
import ast
import copy
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "python" / "manimlib" / "_animation_semantics.py"
BOOTSTRAP = ROOT / "python" / "manimlib_bootstrap.py"
spec = importlib.util.spec_from_file_location("creation_semantics_under_test", SOURCE)
semantics = importlib.util.module_from_spec(spec)
spec.loader.exec_module(semantics)

# Values install() captures before any authored code runs, which the functions
# it defines close over (fm-5wq.31's shipped_straight_path). A harness that lifts
# those functions out of install() must lift these captures with them.
INSTALL_CAPTURES = frozenset({"shipped_straight_path"})


def is_install_capture(node):
    return isinstance(node, ast.Assign) and any(
        isinstance(target, ast.Name) and target.id in INSTALL_CAPTURES for target in node.targets)


def environment():
    class Mobject:
        def __init__(self, *children, points=None):
            self.submobjects = list(children)
            points = [] if points is None else points
            self.data = np.zeros(len(points), dtype=[("point", "f8", (3,)), ("rgba", "f8", (4,)), ("stroke_width", "f8", (1,))])
            self.data["point"] = np.asarray(points).reshape(-1, 3)
            self.data["rgba"][:] = 1
            self.uniforms = {}
            self.locked_data_keys, self.locked_uniform_keys = set(), set()
            self.const_data_keys, self.pointlike_data_keys = set(), ("point",)
            self.animating = self.suspended = False
            self._scene = None
            self.calls, self.updaters = [], []
            self.fill_opacity, self.stroke_width, self.stroke_color = .7, 5., "#FFFFFF"
            self.stroke_behind = False
        @property
        def fill_opacity(self):
            return float(self.data["rgba"][0, 3]) if len(self.data) else self._empty_fill
        @fill_opacity.setter
        def fill_opacity(self, value):
            self._empty_fill = value
            self.data["rgba"][:, 3] = value
        @property
        def stroke_width(self):
            return float(self.data["stroke_width"][0, 0]) if len(self.data) else self._empty_width
        @stroke_width.setter
        def stroke_width(self, value):
            self._empty_width = value
            self.data["stroke_width"][:] = value
        def get_uniforms(self):
            return dict(self.uniforms)
        def set_uniform(self, **kwargs):
            self.uniforms.update(kwargs)
            return self
        def align_data_and_family(self, other):
            if len(self.get_family()) != len(other.get_family()):
                raise ValueError("fixture requires equal family sizes")
            for left, right in zip(self.get_family(), other.get_family()):
                count = max(len(left.data), len(right.data))
                left.data = np.resize(left.data, count)
                right.data = np.resize(right.data, count)
            return self
        def get_family(self):
            result = [self]
            for child in self.submobjects:
                for member in child.get_family():
                    if all(member is not old for old in result):
                        result.append(member)
            return result
        def family_members_with_points(self):
            return [mob for mob in self.get_family() if len(mob.data)]
        def copy(self):
            result = copy.deepcopy(self, {id(self._scene): self._scene})
            return result
        def _is_bound(self):
            return self._scene is not None
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self):
            self.suspended = True
        def resume_updating(self):
            self.suspended = False
            self.calls.append(("resume",))
        def set_animating_status(self, value):
            self.animating = value
        def note_changed_data(self):
            self.calls.append(("dirty",))
        def update(self, dt):
            if not self.suspended:
                for updater in self.updaters:
                    updater(self, dt)
        def get_points(self):
            return self.data["point"]
        def get_color(self):
            return self.stroke_color
        def get_stroke_color(self):
            return self.stroke_color
        def set_fill(self, opacity):
            for mob in self.get_family():
                mob.fill_opacity = opacity
            return self
        def set_stroke(self, color=None, width=None, behind=None):
            if color is not None:
                self.stroke_color = color
            if width is not None:
                self.stroke_width = width
            return self
        def match_style(self, other):
            for mob, source in zip(self.get_family(), other.get_family()):
                mob.fill_opacity, mob.stroke_width = source.fill_opacity, source.stroke_width
                mob.stroke_color = source.stroke_color
            return self
        def set_data(self, data):
            self.data = data.copy()
            return self
        def refresh_joint_angles(self):
            self.calls.append(("joints",))
            return self
    class VMobject(Mobject):
        def pointwise_become_partial(self, source, lower, upper):
            self.calls.append(("partial", lower, upper, type(source)))
            self.data = source.data.copy()
            if len(self.data):
                a, b = source.data["point"][[0, -1]]
                self.data["point"][:] = [a + t * (b - a) for t in np.linspace(lower, upper, len(self.data))]
            return self
    class Surface(Mobject):
        resolution = (2, 2)
        preferred_creation_axis = 1
        def pointwise_become_partial(self, source, lower, upper):
            self.calls.append(("surface_partial", lower, upper, self.preferred_creation_axis))
            self.data = source.data.copy()
            return self
    class VGroup(VMobject):
        pass
    class Animation:
        pass
    class Transform(Animation):
        _target_attr = "target_mobject"
    class AnimationGroup(Animation):
        _native_kind, _default_lag_ratio = "animation_group", 0.
        def __init__(self, *animations, lag_ratio=0., run_time=None, **kwargs):
            self.animations = list(animations)
            Animation.__init__(self, VGroup(*(a.mobject for a in animations)),
                               lag_ratio=lag_ratio, run_time=1. if run_time is None else run_time, **kwargs)
            if run_time is None:
                _, self.run_time = g["_composition_timings"](self, self.animations)
            self.rate_func = g["_linear_rate"]
    class Succession(AnimationGroup):
        _native_kind, _default_lag_ratio = "succession", 1.
        def __init__(self, *animations, **kwargs):
            kwargs.setdefault("lag_ratio", 1.)
            super().__init__(*animations, **kwargs)
    class _AnimationBuilder:
        def __init__(self, animation):
            self.overridden_animation = animation
        def build(self):
            return self.overridden_animation
    class Scene:
        def __init__(self):
            self.roots, self.native = [], []
            self.samples = (.25, .5, .75, 1.)
        def add(self, mob):
            for member in mob.get_family():
                member._scene = self
            if all(mob is not existing for existing in self.roots):
                self.roots.append(mob)
        def remove(self, mob):
            self.roots[:] = [old for old in self.roots if old is not mob]
        def play(self, *animations, run_time=None, rate_func=None, lag_ratio=None):
            def lower(animation):
                if isinstance(animation, AnimationGroup):
                    return g["_CompositionCallbackDriver"](animation, [lower(child) for child in animation.animations])
                if not g["_requires_python_animation"](animation):
                    self.native.append(animation)
                    return None
                animation._ensure_runtime_defaults()
                for name, value in (("run_time", run_time), ("rate_func", rate_func), ("lag_ratio", lag_ratio)):
                    if value is not None:
                        setattr(animation, name, value)
                return animation
            drivers = []
            for animation in animations:
                self.add(animation.mobject)
                driver = lower(animation)
                drivers.append(driver)
                if driver is not None:
                    driver.begin()
            if getattr(self, "fail_prologue", False):
                raise ValueError("native prologue failure")
            for alpha in self.samples:
                for driver in drivers:
                    if driver is not None:
                        driver.update_mobjects(.25)
                        driver.interpolate(alpha)
                if getattr(self, "fail_updater", False):
                    raise ValueError("scene updater failure")
            for driver in drivers:
                if driver is not None:
                    driver.finish()
                    driver.clean_up_from_scene(self)
    def refuse(name, items):
        if any(value for _, value in items):
            raise NotImplementedError(name)
    def intervals(durations, lag):
        result, start = [], 0.
        for duration in durations:
            result.append((start, start + duration))
            start += lag * duration
        return result
    def linear(t):
        return t
    def smooth(t):
        return t**3 * (10 - 15*t + 6*t*t)
    def double_smooth(t):
        return .5 * smooth(2*t) if t < .5 else .5 * (1 + smooth(2*t - 1))
    g = dict(locals())
    g.update(_abc=abc, _np=np, np=np, _copy=copy, copy_module=copy, _OUT=(0., 0., 1.),
             _linear_rate=linear, _smooth_rate=smooth, smooth_rate=smooth,
             _interpolate=lambda a,b,t:(1-t)*a+t*b, straight_path=lambda a,b,t:(1-t)*a+t*b,
             interpolate_value=lambda a,b,t:(1-t)*a+t*b,
             _refuse_unrouted=refuse, refuse_unrouted=refuse,
             _RATE_FUNC_NAMES={linear:"linear", smooth:"smooth", double_smooth:"double_smooth"},
             _FMN_ROOT=SimpleNamespace(_composition_intervals=intervals),
             prepare_animation=lambda animation: animation.build() if isinstance(animation, _AnimationBuilder) else animation)
    g["g"] = g
    names = ("_NativeAnimation", "ShowPartial", "ShowCreation", "Uncreate", "DrawBorderThenFill", "Write",
             "ShowPassingFlash", "ShowCreationThenDestruction", "_requires_python_animation",
             "_composition_member_run_time", "_composition_timings", "_composition_timeline_position",
             "_CompositionCallbackDriver")
    tree = ast.parse(BOOTSTRAP.read_text())
    nodes = {node.name: node for node in tree.body if isinstance(node, (ast.FunctionDef, ast.ClassDef))}
    for name in names:
        exec(compile(ast.Module([nodes[name]], []), str(BOOTSTRAP), "exec"), g)
    install = next(node for node in ast.parse(SOURCE.read_text()).body if isinstance(node, ast.FunctionDef) and node.name == "install")
    for node in install.body:
        # install()'s captures run first: the lifted functions close over them.
        if isinstance(node, ast.FunctionDef) or is_install_capture(node):
            exec(compile(ast.Module([node], []), str(SOURCE), "exec"), g)
    mappings = {
        "__init__":"animation_init", "_validate_input_type":"validate_input_type",
        "_ensure_runtime_defaults":"ensure_runtime_defaults", "begin":"animation_begin",
        "finish":"animation_finish", "create_starting_mobject":"create_starting_mobject",
        "get_all_mobjects":"get_all_mobjects", "get_all_families_zipped":"get_all_families_zipped",
        "get_all_mobjects_to_update":"get_all_mobjects_to_update", "update_mobjects":"update_mobjects",
        "interpolate":"animation_interpolate", "interpolate_mobject":"interpolate_mobject",
        "interpolate_submobject":"interpolate_submobject", "get_sub_alpha":"get_sub_alpha",
        "time_spanned_alpha":"time_spanned_alpha", "get_run_time":"get_run_time",
        "clean_up_from_scene":"clean_up_from_scene", "is_remover":"is_remover",
    }
    for name, function in mappings.items():
        setattr(Animation, name, g[function])
    g["_NativeAnimation"].__init__ = g["native_animation_init"]
    Mobject.interpolate = g["mobject_interpolate"]
    g["DrawBorderThenFill"].get_outline = g["draw_border_get_outline"]
    return g


def line(g):
    return g["VMobject"](points=[[0.,0.,0.],[.5,0.,0.],[1.,0.,0.]])
