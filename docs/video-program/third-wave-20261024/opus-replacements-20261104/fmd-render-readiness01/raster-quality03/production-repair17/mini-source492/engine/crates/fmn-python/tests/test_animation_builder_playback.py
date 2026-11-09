"""Production builder definitions and wheel normalization, without native kernels."""
import ast
import copy
import importlib.util
import pathlib
import types
import sys
from unittest.mock import patch
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("fmn_playback_under_test", ROOT / "python/fmn_python/playback.py")
playback = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(playback)


def environment():
    native = types.ModuleType("manimlib.native_playback_fixture")
    g = vars(native)
    class Mobject:
        def __init__(self, x=0):
            self.x, self.target, self.saved_state = x, None, None
        def copy(self):
            result = copy.copy(self)
            result.target = result.saved_state = None
            return result
        def generate_target(self):
            self.target = self.copy()
            return self.target
        def shift(self, value):
            self.x += value
            return self
    class Animation:
        _native_kind = None
        def __init__(self, mobject, **kwargs):
            self.mobject = mobject
            self.final_alpha_value = kwargs.pop("final_alpha_value", 1.0)
            self.remover = kwargs.pop("remover", False)
            self.run_time = kwargs.pop("run_time", None)
            self.rate_func = kwargs.pop("rate_func", None)
            self.lag_ratio = kwargs.pop("lag_ratio", None)
            self.suspend_mobject_updating = kwargs.pop("suspend_mobject_updating", False)
            self.time_span = kwargs.pop("time_span", None)
            self.name = kwargs.pop("name", "")
            if kwargs:
                raise TypeError("unknown animation options: " + ", ".join(kwargs))
    class NativeAnimation(Animation):
        _target_attr = None
    class Transform(NativeAnimation):
        _native_kind = "transform"
        _target_attr = "target_mobject"
        def __init__(self, mobject, target_mobject=None, path_func=None, path_arc=0., path_arc_axis=None, **kwargs):
            super().__init__(mobject, **kwargs)
            self.target_mobject, self.path_func = target_mobject, path_func
            self.path_arc, self.path_arc_axis = path_arc, path_arc_axis
    class Scene:
        def __init__(self):
            self.calls, self.routes = [], []
        def play(self, *animations, **kwargs):
            self.calls.append((animations, kwargs))
            self.routes.append([g["_requires_python_animation"](a) for a in animations if isinstance(a, Animation)])
            return self.calls[-1]
    def requires(animation):
        return not getattr(animation, "_native_kind", None) or bool(getattr(animation, "path_func", None))
    def refuse(where, values):
        if any(active for _, active in values):
            raise NotImplementedError(where + " unsupported path_func")
    g.update(_NativeAnimation=NativeAnimation, _OUT=(0., 0., 1.), _vec3=tuple,
             _refuse_unrouted=refuse, Mobject=Mobject, Animation=Animation, Transform=Transform, Scene=Scene,
             _requires_python_animation=requires)
    names = {"_AnimationBuilder", "MoveToTarget", "_MethodAnimation", "prepare_animation", "override_animate", "Restore"}
    path = ROOT / "python/manimlib_bootstrap.py"
    tree = ast.parse(path.read_text())
    nodes = [node for node in tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in names]
    if {node.name for node in nodes} != names:
        raise AssertionError("production builder definitions are missing")
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(path), "exec"), g)
    return native


class BuilderPlaybackTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        self.g = vars(self.native)
        self.original = self.native.Scene.play
        self.previous_requires = self.g["_requires_python_animation"]
        playback.install_scene_playback(self.native)
        self.scene, self.mob = self.native.Scene(), self.native.Mobject()
    def builder(self, **kwargs):
        return self.native._AnimationBuilder(self.mob)(**kwargs).shift(4)
    def test_production_builder_builds_method_animation(self):
        builder = self.builder(time_span=(.2, .8), suspend_mobject_updating=True, name="move")
        result = self.scene.play(builder)
        animation = result[0][0]
        self.assertIsInstance(animation, self.native._MethodAnimation)
        self.assertIs(animation.mobject, self.mob)
        self.assertIs(animation.target_mobject, self.mob.target)
        self.assertEqual(animation.target_mobject.x, 4)
        self.assertEqual(animation.time_span, (.2, .8))
        self.assertTrue(animation.suspend_mobject_updating)
        self.assertEqual(animation.name, "move")
        self.assertEqual(len(animation.methods), 1)
    def test_custom_path_and_axis_are_not_filtered(self):
        path = lambda a, b, t: a
        axis = object()
        animation = self.scene.play(self.builder(path_func=path, path_arc=.7, path_arc_axis=axis))[0][0]
        self.assertIs(animation.path_func, path)
        self.assertIs(animation.path_arc_axis, axis)
        self.assertEqual(animation.path_arc, .7)
        self.assertEqual(self.scene.routes, [[True]])
    def test_timing_and_play_overrides_stay_on_their_own_objects(self):
        rate, override = lambda t: t, lambda t: t * t
        result = self.scene.play(self.builder(run_time=2, rate_func=rate, lag_ratio=.2),
                                 run_time=3, rate_func=override, lag_ratio=0.)
        animation, = result[0]
        self.assertEqual(animation.run_time, 2)
        self.assertIs(animation.rate_func, rate)
        self.assertEqual(animation.lag_ratio, .2)
        self.assertEqual(result[1], {"run_time": 3, "rate_func": override, "lag_ratio": 0.})
    def test_default_builder_retains_native_route(self):
        self.scene.play(self.builder())
        self.assertEqual(self.scene.routes, [[False]])
    def test_nondefault_final_alpha_uses_shared_transform_callback(self):
        for alpha in (0., .5, -1., 2.):
            self.scene.play(self.builder(final_alpha_value=alpha))
            self.assertEqual(self.scene.routes[-1], [True])
            self.assertEqual(self.scene.calls[-1][0][0].final_alpha_value, alpha)
    def test_remover_uses_shared_transform_cleanup(self):
        self.scene.play(self.builder(remover=True))
        self.assertEqual(self.scene.routes, [[True]])
    def test_explicit_transform_endpoint_options_are_also_honored(self):
        animation = self.native.Transform(self.mob, final_alpha_value=.25)
        self.scene.play(animation)
        self.assertIs(self.scene.calls[0][0][0], animation)
        self.assertEqual(self.scene.routes, [[True]])
    def test_native_replacement_endpoint_options_use_python_protocol(self):
        animation = self.native.Transform(self.mob, remover=True)
        for kind in ("replacement_transform", "transform_from_copy"):
            animation._native_kind = kind
            self.assertTrue(self.g["_requires_python_animation"](animation))
    def test_specialized_native_final_alpha_defaults_are_not_reinterpreted(self):
        for kind in ("fade_out", "v_fade_out", "grow_from_center", "indicate"):
            animation = self.native.Transform(self.mob, final_alpha_value=0.)
            animation._native_kind = kind
            self.assertFalse(self.g["_requires_python_animation"](animation))
    def test_plain_python_animation_retains_original_decision(self):
        animation = self.native.Animation(self.mob)
        self.scene.play(animation)
        self.assertEqual(self.scene.routes, [[True]])
    def test_explicit_animation_is_not_prepared_again(self):
        animation = self.native.Animation(self.mob)
        def fail(proto):
            raise AssertionError("an explicit animation was rebuilt")
        self.g["prepare_animation"] = fail
        self.scene.play(animation)
        self.assertIs(self.scene.calls[0][0][0], animation)
    def test_build_override_is_invoked_once_and_product_identity_survives(self):
        product = self.native.Animation(self.mob)
        seen = []
        class Builder(self.native._AnimationBuilder):
            def build(self):
                seen.append(self)
                return product
        builder = Builder(self.mob)
        self.scene.play(builder)
        self.assertEqual(seen, [builder])
        self.assertIs(self.scene.calls[0][0][0], product)
    def test_left_to_right_build_order(self):
        seen = []
        parent, Animation, mob = self.native._AnimationBuilder, self.native.Animation, self.mob
        class Builder(parent):
            def __init__(self, name):
                super().__init__(mob)
                self.label = name
            def build(self):
                seen.append(self.label)
                return Animation(mob)
        self.scene.play(Builder("first"), Builder("second"))
        self.assertEqual(seen, ["first", "second"])
    def test_authored_prepare_hook_is_used(self):
        original = self.g["prepare_animation"]
        seen = []
        def prepare(proto):
            seen.append(proto)
            return original(proto)
        self.g["prepare_animation"] = prepare
        builder = self.builder()
        self.scene.play(builder)
        self.assertEqual(seen, [builder])
    def test_build_exception_prevents_play_and_keeps_exception_identity(self):
        error = RuntimeError("build")
        class Builder(self.native._AnimationBuilder):
            def build(self):
                raise error
        with self.assertRaises(RuntimeError) as raised:
            self.scene.play(Builder(self.mob))
        self.assertIs(raised.exception, error)
        self.assertEqual(self.scene.calls, [])
    def test_invalid_builder_product_fails_before_play(self):
        class Builder(self.native._AnimationBuilder):
            def build(self):
                return object()
        with self.assertRaisesRegex(TypeError, "must return an Animation"):
            self.scene.play(Builder(self.mob))
        self.assertEqual(self.scene.calls, [])
    def test_invalid_constructor_option_is_not_silently_discarded(self):
        with self.assertRaises((TypeError, NotImplementedError)):
            self.scene.play(self.builder(unsupported=7))
        self.assertEqual(self.scene.calls, [])
    def test_dynamic_target_lookup_matches_prepare_animation(self):
        first = self.builder()
        second = self.builder()
        current = self.mob.target
        self.scene.play(first, second)
        self.assertTrue(all(a.target_mobject is current for a in self.scene.calls[0][0]))
    def test_override_animate_product_is_not_reconstructed(self):
        product = self.native.Animation(self.mob)
        builder = self.builder()
        builder.overridden_animation = product
        self.scene.play(builder)
        self.assertIs(self.scene.calls[0][0][0], product)
    def test_arguments_can_still_only_be_set_once(self):
        builder = self.builder(run_time=2)
        with self.assertRaises(ValueError):
            builder.set_anim_args(run_time=3)
    def test_empty_play_preserves_original_return(self):
        result = self.scene.play()
        self.assertEqual(result[0], ())
        self.assertIs(result, self.scene.calls[0])
    def test_nonbuilder_unknown_input_is_left_for_existing_validation(self):
        marker = object()
        self.scene.play(marker)
        self.assertIs(self.scene.calls[0][0][0], marker)
    def test_reinstallation_preserves_later_authored_methods(self):
        replacement = lambda *args, **kwargs: "authored"
        self.native.Scene.play = replacement
        self.g["_requires_python_animation"] = replacement
        playback.install_scene_playback(self.native)
        self.assertIs(self.native.Scene.play, replacement)
        self.assertIs(self.g["_requires_python_animation"], replacement)
    def test_scene_subclass_super_dispatch_does_not_duplicate_build(self):
        parent, seen = self.native.Scene, []
        class Scene(parent):
            def play(self, *args, **kwargs):
                seen.append(args)
                return super().play(*args, **kwargs)
        scene = Scene()
        builder = self.builder()
        scene.play(builder)
        self.assertEqual(len(scene.calls), 1)
        self.assertEqual(seen, [(builder,)])
    def test_wrapped_method_preserves_public_identity(self):
        self.assertEqual(self.native.Scene.play.__name__, self.original.__name__)
        self.assertEqual(self.native.Scene.play.__qualname__, self.original.__qualname__)
        self.assertIs(self.native.Scene.play.__wrapped__, self.original)
    def test_uninstalled_route_receives_builder_as_negative_control(self):
        builder = self.builder(time_span=(0., 1.))
        self.original(self.scene, builder)
        self.assertIs(self.scene.calls[0][0][0], builder)
        self.assertEqual(self.scene.routes, [[]])

    def test_actual_wheel_initializer_installs_playback_without_public_leaks(self):
        native = environment()
        native.__name__ = "manimlib.manimlib"
        original_scene = native.Scene
        source = (ROOT / "python/manimlib/__init__.py").read_text()
        tree = ast.parse(source)
        aliases = next(ast.literal_eval(node.value) for node in tree.body
                       if isinstance(node, ast.Assign) and any(
                           isinstance(target, ast.Name) and target.id == "_REFERENCE_CLASS_BY_RUST_HELPER"
                           for target in node.targets))
        package = types.ModuleType("manimlib")
        package.__path__ = []
        distribution = types.ModuleType("fmn_python")
        distribution.__path__ = []
        distribution._ensure_exclusive_manimlib_namespace = lambda: None
        shapes = types.ModuleType("manimlib.protocol_shapes")
        for name in aliases.values():
            value = type(name, (), {})
            setattr(native, name, value)
            setattr(shapes, name, value)
        for name in ("__version__", "__distribution__", "__franken_manim__", "__abi_policy__",
                     "__engine__", "__thread_policy__", "__reference_commit__"):
            setattr(native, name, "native-boundary-fixture")
        authority = types.ModuleType("fmn_python.library_constructor_authority")
        authority.REFERENCE_CLASS_BY_RUST_HELPER = aliases
        authority.REFERENCE_MODULE_BY_RUST_HELPER = {key: shapes.__name__ for key in aliases}
        provenance = types.ModuleType("fmn_python.schema_provenance")
        provenance.SchemaProvenanceError = ValueError
        provenance.apply_schema_placeholder_provenance = lambda module: None
        rendering = types.ModuleType("fmn_python.rendering")
        rendering.install_scene_rendering = lambda module: None
        modules = {module.__name__: module for module in
                   (native, package, distribution, shapes, authority, provenance, rendering)}
        fading = types.ModuleType("fmn_python.fading")
        fading.install_fading = lambda module: None
        modules["fmn_python.fading"] = fading
        initialization = types.ModuleType("fmn_python.initialization")
        initialization.initialize = lambda module: playback.install_scene_playback(module)
        modules["fmn_python.initialization"] = initialization
        modules["fmn_python.playback"] = playback
        with patch.dict(sys.modules, modules):
            exec(compile(source, "manimlib/__init__.py", "exec"), vars(package))
            self.assertIs(package.Scene, original_scene)
            mob, scene = native.Mobject(), package.Scene()
            result = scene.play(native._AnimationBuilder(mob)(final_alpha_value=.5).shift(4))
            self.assertIsInstance(result[0][0], native._MethodAnimation)
            self.assertEqual(scene.routes, [[True]])
            self.assertEqual({name for name in vars(package) if not name.startswith("_")},
                             {name for name in vars(native) if not name.startswith("_")})


if __name__ == "__main__":
    unittest.main()
