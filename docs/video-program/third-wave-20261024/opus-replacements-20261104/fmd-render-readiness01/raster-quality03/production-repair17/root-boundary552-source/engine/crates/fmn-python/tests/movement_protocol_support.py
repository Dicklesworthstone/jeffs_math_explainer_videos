"""Production constructors/lifecycle with fixture geometry and playback.

These tests do not establish compiled-extension or renderer acceptance. The
companion movement_semantics.py suite uses the real installed wheel.
"""
import ast
import copy
import importlib.util
from pathlib import Path
from types import ModuleType

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("movement_under_test", ROOT / "python/fmn_python/movement.py")
movement = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(movement)


def environment(install=True):
    native = ModuleType("manimlib.movement_fixture")
    g = vars(native)

    class Mobject:
        def __init__(self, *children, points=()):
            self.submobjects = list(children)
            self.points = np.asarray(points, dtype=float).reshape(-1, 3).copy()
            self.suspended = self.animating = False
            self.updaters, self.calls = [], []
            self.color = "unchanged"
        def __getitem__(self, index):
            return self.submobjects[index]
        def get_family(self):
            result = [self]
            for child in self.submobjects:
                result.extend(child.get_family())
            return result
        def family_members_with_points(self):
            return [member for member in self.get_family() if member.has_points()]
        def has_points(self):
            return bool(len(self.points))
        def copy(self):
            result = copy.deepcopy(self)
            for live, new in zip(self.get_family(), result.get_family()):
                new.updaters = list(live.updaters)
            return result
        def match_points(self, other):
            self.points = other.points.copy()
            self.calls.append(("match",))
            return self
        def apply_function(self, function, **config):
            self.calls.append(("apply", dict(config)))
            for member in self.get_family():
                if member.has_points():
                    member.points[:] = np.asarray([function(point.copy()) for point in member.points])
            return self
        def move_to(self, point):
            self.shift(np.asarray(point) - self.get_center())
            return self
        def shift(self, amount):
            for member in self.get_family():
                member.points += amount
            return self
        def get_center(self):
            points = [member.points for member in self.get_family() if member.has_points()]
            if not points:
                return np.zeros(3)
            points = np.concatenate(points)
            return .5 * (points.min(axis=0) + points.max(axis=0))
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self, recurse=True):
            for member in self.get_family() if recurse else [self]:
                member.suspended = True
            return self
        def resume_updating(self, recurse=True, call_updater=True):
            for member in self.get_family() if recurse else [self]:
                member.suspended = False
            if call_updater:
                self.update(0.)
            return self
        def set_animating_status(self, value):
            for member in self.get_family():
                member.animating = value
        def update(self, dt):
            if not self.suspended:
                for child in self.submobjects:
                    child.update(dt)
                for updater in list(self.updaters):
                    updater(self, dt)
            return self
        def has_updaters(self):
            return any(member.updaters for member in self.get_family())

    class VMobject(Mobject):
        def point_from_proportion(self, alpha):
            return (1 - alpha) * self.points[0] + alpha * self.points[-1]

    class Animation:
        _native_kind = None

    class _NativeAnimation(Animation):
        pass

    class AnimationGroup(Animation):
        def __init__(self, *animations):
            self.animations = list(animations)

    class _AnimationBuilder:
        def __init__(self, product):
            self.product, self.count = product, 0
        def build(self):
            self.count += 1
            return self.product

    class Scene:
        def __init__(self):
            self.roots, self.native, self.events = [], [], []
            self.fail_after = None
        def add(self, *mobs):
            for mob in mobs:
                if mob not in self.roots:
                    self.roots.append(mob)
        def remove(self, *mobs):
            self.roots[:] = [mob for mob in self.roots if mob not in mobs]
        def play(self, *animations, **kwargs):
            leaves = []
            def collect(animation):
                if isinstance(animation, AnimationGroup):
                    for child in animation.animations:
                        collect(child)
                else:
                    leaves.append(animation)
            for animation in animations:
                collect(animation)
            for animation in leaves:
                if not g["_requires_python_animation"](animation):
                    self.native.append(animation)
                    continue
                for key, value in kwargs.items():
                    if value is not None:
                        setattr(animation, key, value)
                self.add(animation.mobject)
                animation.begin()
            for alpha in (.25, .5, .75, 1.):
                for animation in leaves:
                    if animation in self.native:
                        continue
                    animation.update_mobjects(.25)
                    animation.interpolate(alpha)
                if self.fail_after == "scene":
                    raise RuntimeError("scene updater failed")
                for root in self.roots:
                    root.update(.25)
            for animation in leaves:
                if animation not in self.native:
                    animation.finish()
                    animation.clean_up_from_scene(self)
            return "played"

    def refuse_unrouted(where, settings):
        unsupported = [key for key, active in settings if active]
        if unsupported:
            raise NotImplementedError(where + ": " + ", ".join(unsupported))

    def linear(t):
        return t
    def smooth(t):
        return t**3 * (10 - 15*t + 6*t*t)
    g.update(Mobject=Mobject, VMobject=VMobject, Animation=Animation,
             _NativeAnimation=_NativeAnimation, AnimationGroup=AnimationGroup,
             _AnimationBuilder=_AnimationBuilder, Scene=Scene, np=np, _np=np,
             _linear_rate=linear, smooth_rate=smooth, smooth=smooth,
             refuse_unrouted=refuse_unrouted,
             _RATE_FUNC_NAMES={linear:"linear", smooth:"smooth"},
             _requires_python_animation=lambda a:not bool(getattr(a, "_native_kind", None)),
             prepare_animation=lambda a:a.build() if isinstance(a, _AnimationBuilder) else a)
    g["g"] = g
    path = ROOT / "python/manimlib/_animation_semantics.py"
    installer = next(node for node in ast.parse(path.read_text()).body
                     if isinstance(node, ast.FunctionDef) and node.name == "install")
    definitions = {node.name:node for node in installer.body if isinstance(node, ast.FunctionDef)}
    methods = {
        "__init__":"animation_init", "_validate_input_type":"validate_input_type",
        "_ensure_runtime_defaults":"ensure_runtime_defaults", "begin":"animation_begin",
        "finish":"animation_finish", "create_starting_mobject":"create_starting_mobject",
        "get_all_mobjects":"get_all_mobjects", "get_all_families_zipped":"get_all_families_zipped",
        "get_all_mobjects_to_update":"get_all_mobjects_to_update", "update_mobjects":"update_mobjects",
        "interpolate":"animation_interpolate", "interpolate_mobject":"interpolate_mobject",
        "interpolate_submobject":"interpolate_submobject", "get_sub_alpha":"get_sub_alpha",
        "time_spanned_alpha":"time_spanned_alpha", "get_run_time":"get_run_time",
        "is_remover":"is_remover", "clean_up_from_scene":"clean_up_from_scene",
    }
    for name, definition in methods.items():
        exec(compile(ast.Module([definitions[definition]], []), str(path), "exec"), g)
        setattr(Animation, name, g[definition])
    exec(compile(ast.Module([definitions["native_animation_init"]], []), str(path), "exec"), g)
    _NativeAnimation.__init__ = g["native_animation_init"]
    path = ROOT / "python/manimlib_bootstrap.py"
    names = {"Homotopy", "SmoothedVectorizedHomotopy", "ComplexHomotopy", "PhaseFlow", "MoveAlongPath"}
    nodes = [node for node in ast.parse(path.read_text()).body if isinstance(node, ast.ClassDef) and node.name in names]
    assert {node.name for node in nodes} == names
    exec(compile(ast.Module(nodes, []), str(path), "exec"), g)
    native._old_homotopy_begin = native.Homotopy.begin
    native._old_homotopy_interpolate = native.Homotopy.interpolate_mobject
    if install:
        movement.install_movement(native)
    return native


def point(native, x=0., y=0., z=0.):
    return native.VMobject(points=[(x, y, z)])
