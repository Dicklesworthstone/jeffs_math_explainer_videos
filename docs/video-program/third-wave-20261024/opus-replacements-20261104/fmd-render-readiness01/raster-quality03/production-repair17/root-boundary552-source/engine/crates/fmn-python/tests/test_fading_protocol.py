"""Production fading installer with fixture arrays, hierarchy and dispatch.

No native geometry or rendering is asserted by this protocol suite.
"""
import ast
import copy
import pathlib
import sys
import types
import unittest
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "python"))
from fmn_python.fading import install_fading


def environment():
    class Mobject:
        def __init__(self, *children, x=0., width=2., opacity=1.):
            self.submobjects = list(children)
            self.x, self.width, self.opacity = float(x), float(width), float(opacity)
            self.uniforms = {"weight": 1.}
            self.suspended = self.animating = False
            self.saved_state = None
            self.updaters = []
            self.scene = None
            self.events = []
            self.stroke_opacity = float(opacity)
        def __getitem__(self, index):
            return self.submobjects[index]
        def set_submobjects(self, children):
            self.submobjects = list(children)
            return self
        def get_family(self):
            return [self, *(member for child in self.submobjects for member in child.get_family())]
        def family_members_with_points(self):
            return [self] if not self.submobjects else [m for c in self.submobjects for m in c.family_members_with_points()]
        def copy(self):
            result = copy.deepcopy(self, {id(self.scene): self.scene})
            return result
        def save_state(self):
            self.saved_state = None
            self.saved_state = self.copy()
            return self
        def become(self, other):
            self.x, self.width, self.opacity = other.x, other.width, other.opacity
            self.uniforms = dict(other.uniforms)
            while len(self.submobjects) > len(other.submobjects):
                self.submobjects.pop()
            while len(self.submobjects) < len(other.submobjects):
                self.submobjects.append(other.submobjects[len(self.submobjects)].copy())
            for a, b in zip(self.submobjects, other.submobjects):
                a.become(b)
            return self
        def align_family(self, other):
            count = max(len(self.submobjects), len(other.submobjects))
            for obj in (self, other):
                while len(obj.submobjects) < count:
                    obj.submobjects.append(obj.submobjects[-1].copy() if obj.submobjects else Mobject())
            for a, b in zip(self.submobjects, other.submobjects):
                a.align_family(b)
            return self
        def align_data_and_family(self, other):
            return self.align_family(other)
        def replace(self, target, stretch=True, dim_to_match=1):
            self.x, self.width = target.x, target.width
            for child in self.submobjects:
                child.replace(target, stretch=stretch, dim_to_match=dim_to_match)
            self.events.append(("replace", stretch, dim_to_match))
            return self
        def get_stroke_opacity(self):
            return self.stroke_opacity
        def get_fill_opacity(self):
            return self.opacity
        def set_stroke(self, opacity=None, recurse=True):
            for obj in self.get_family() if recurse else [self]:
                if opacity is not None:
                    obj.stroke_opacity = float(opacity)
            return self
        def set_fill(self, opacity=None, recurse=True):
            for obj in self.get_family() if recurse else [self]:
                if opacity is not None:
                    obj.opacity = float(opacity)
            return self
        def unlock_data(self):
            self.events.append(("unlock",))
        def shift(self, vector):
            self.x += float(np.asarray(vector)[0])
            return self
        def scale(self, factor):
            self.width *= factor
            return self
        def get_uniforms(self):
            return dict(self.uniforms)
        def set_uniform(self, **values):
            self.uniforms.update(values)
            return self
        def set_opacity(self, value):
            for member in self.get_family():
                member.opacity = value
            return self
        def interpolate(self, start, end, alpha, path_func):
            self.x = float(np.asarray(path_func(np.array([start.x]), np.array([end.x]), alpha))[0])
            self.width = (1-alpha)*start.width + alpha*end.width
            self.opacity = (1-alpha)*start.opacity + alpha*end.opacity
            for key in start.uniforms:
                self.uniforms[key] = (1-alpha)*start.uniforms[key] + alpha*end.uniforms[key]
            return self
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self, recurse=True):
            for obj in self.get_family() if recurse else [self]:
                obj.suspended = True
            return self
        def resume_updating(self, recurse=True, call_updater=True):
            for obj in self.get_family() if recurse else [self]:
                obj.suspended = False
                if call_updater:
                    obj.update(0.)
            return self
        def set_animating_status(self, value):
            for obj in self.get_family():
                obj.animating = value
        def update(self, dt):
            if not self.suspended:
                for callback in list(self.updaters):
                    callback(self, dt)
    class VMobject(Mobject):
        pass
    class Group(Mobject):
        pass
    class CameraFrame(Mobject):
        pass
    class Animation:
        def __init__(self, mobject, run_time=1., rate_func=None, time_span=None, lag_ratio=0.,
                     final_alpha_value=1., suspend_mobject_updating=False, remover=False, **kwargs):
            self.mobject, self.run_time = mobject, run_time
            self.rate_func = rate_func or (lambda a:a)
            self.time_span, self.lag_ratio = time_span, lag_ratio
            self.final_alpha_value, self.suspend_mobject_updating = final_alpha_value, suspend_mobject_updating
            self.remover = remover
            self.__dict__.update(kwargs)
        def begin(self):
            self._ensure_runtime_defaults()
            if self.time_span is not None:
                self.run_time = max(self.run_time, self.time_span[1])
            self.mobject.set_animating_status(True)
            self.starting_mobject = self.create_starting_mobject()
            self.mobject_was_updating = False
            if self.suspend_mobject_updating:
                self.mobject_was_updating = not self.mobject._is_updating_suspended()
                self.mobject.suspend_updating()
            self.families = list(self.get_all_families_zipped())
            self.interpolate(0.)
        def finish(self):
            self.interpolate(self.final_alpha_value)
            self.mobject.set_animating_status(False)
            if self.suspend_mobject_updating and self.mobject_was_updating:
                self.mobject.resume_updating()
        def get_all_mobjects(self):
            return self.mobject, self.starting_mobject
        def clean_up_from_scene(self, scene):
            if self.remover:
                scene.remove(self.mobject)
        def interpolate_submobject(self, current, start, alpha):
            pass
        def time_spanned_alpha(self, alpha):
            if self.time_span is None:
                return alpha
            a, b = self.time_span
            return min(max(alpha*self.run_time-a, 0), b-a)/(b-a)
        def get_sub_alpha(self, alpha, index, count):
            return self.rate_func(min(max(alpha*((count-1)*self.lag_ratio+1)-index*self.lag_ratio, 0), 1))
        def _ensure_runtime_defaults(self):
            if self.run_time is None:
                self.run_time = 1.
            if self.lag_ratio is None:
                self.lag_ratio = 0.
        def create_starting_mobject(self):
            return self.mobject.copy()
        def get_all_families_zipped(self):
            return zip(*(obj.get_family() for obj in self.get_all_mobjects()))
        def get_all_mobjects_to_update(self):
            return [obj for obj in self.get_all_mobjects() if obj is not self.mobject]
        def update_mobjects(self, dt):
            for obj in self.get_all_mobjects_to_update():
                obj.update(dt)
        def interpolate(self, alpha):
            self.interpolate_mobject(alpha)
        def interpolate_mobject(self, alpha):
            alpha = self.time_spanned_alpha(alpha)
            for i, family in enumerate(self.families):
                self.interpolate_submobject(*family, self.get_sub_alpha(alpha, i, len(self.families)))
        def is_remover(self):
            return self.remover
    class Transform(Animation):
        _native_kind = "transform"
        def __init__(self, mobject, target_mobject=None, path_func=None, **kwargs):
            super().__init__(mobject, **kwargs)
            self.target_mobject, self.path_func = target_mobject, path_func
        def init_path_func(self):
            if self.path_func is None:
                self.path_func = lambda a,b,t:(1-t)*a+t*b
        def create_target(self):
            return self.target_mobject if self.target_mobject is not None else self.mobject.copy()
        def begin(self):
            self.init_path_func()
            self.target_mobject = self.create_target()
            self.target_copy = self.target_mobject.copy()
            Animation.begin(self)
        def get_all_mobjects(self):
            return self.mobject, self.starting_mobject, self.target_copy
        def interpolate_submobject(self, current, start, end, alpha):
            current.interpolate(start, end, alpha, self.path_func)
    class NativeAnimation(Animation):
        pass
    class VFadeIn(NativeAnimation):
        _native_kind = "v_fade_in"
        def __init__(self, mobject, suspend_mobject_updating=False, **kwargs):
            if not isinstance(mobject, VMobject):
                raise TypeError("requires VMobject")
            super().__init__(mobject, suspend_mobject_updating=suspend_mobject_updating, **kwargs)
    class VFadeOut(NativeAnimation):
        _native_kind = "v_fade_out"
        def __init__(self, mobject, remover=True, final_alpha_value=0., **kwargs):
            super().__init__(mobject, remover=remover, final_alpha_value=final_alpha_value, **kwargs)
    class VFadeInThenOut(VFadeIn):
        _native_kind = "v_fade_in_then_out"
        def __init__(self, mobject, rate_func=lambda t:2*t if t < .5 else 2*(1-t),
                     remover=True, final_alpha_value=.5, **kwargs):
            super().__init__(mobject, rate_func=rate_func, remover=remover,
                             final_alpha_value=final_alpha_value, **kwargs)
    class Fade(Transform):
        def __init__(self, mobject, shift=(0.,0.,0.), scale=1., **kwargs):
            super().__init__(mobject, **kwargs)
            self.shift_vect, self.scale_factor = np.array(shift), scale
    class FadeIn(Fade):
        _native_kind = "fade_in"
        def create_target(self):
            return self.mobject.copy()
        def create_starting_mobject(self):
            return self.mobject.copy().set_opacity(0).scale(1/self.scale_factor).shift(-self.shift_vect)
    class FadeOut(Fade):
        _native_kind = "fade_out"
        def __init__(self, mobject, shift=(0.,0.,0.), remover=True, final_alpha_value=0., **kwargs):
            if not remover or final_alpha_value != 0:
                raise NotImplementedError("unrouted fade options")
            super().__init__(mobject, shift=shift, **kwargs)
            self.final_alpha_value = 0.
        def create_target(self):
            return self.mobject.copy().set_opacity(0).shift(self.shift_vect).scale(self.scale_factor)
    class FadeTransform(NativeAnimation):
        _native_kind = "fade_transform"
        def ghost_to(self, source, target):
            source.replace(target, stretch=self.stretch, dim_to_match=self.dim_to_match)
            source.set_uniform(**target.get_uniforms())
            source.set_opacity(0)
    class FadeTransformPieces(FadeTransform):
        _native_kind = "fade_transform_pieces"
        def ghost_to(self, source, target):
            for a, b in zip(source.get_family(), target.get_family()):
                FadeTransform.ghost_to(self, a, b)
    class AnimationGroup(Animation):
        def __init__(self, *animations):
            self.animations = animations
    class Scene:
        def __init__(self):
            self.mobjects = []
            self.fail_updater = False
        def add(self, *objects):
            for obj in objects:
                obj.scene = self
                if obj not in self.mobjects:
                    self.mobjects.append(obj)
        def remove(self, *objects):
            self.mobjects[:] = [obj for obj in self.mobjects if obj not in objects]
        def play(self, *animations, **kwargs):
            leaves = []
            def collect(anim):
                if isinstance(anim, AnimationGroup):
                    for child in anim.animations:
                        collect(child)
                else:
                    leaves.append(anim)
            for anim in animations:
                collect(anim)
            for anim in leaves:
                self.add(anim.mobject)
                anim.begin()
            if self.fail_updater:
                raise RuntimeError("scene updater failed")
            for alpha in (.25, .5, .75, 1.):
                for anim in leaves:
                    anim.update_mobjects(.25)
                    anim.interpolate(alpha)
            for anim in leaves:
                anim.finish()
                anim.clean_up_from_scene(self)
    result = types.SimpleNamespace(**locals())
    result._ORIGIN = (0., 0., 0.)
    result._RATE_FUNC_NAMES = {}
    result._requires_python_animation = lambda animation: not getattr(animation, "_native_kind", None) or bool(getattr(animation, "path_func", None))
    return result


class FadeTransformTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.original = self.g.FadeTransform
        install_fading(self.g)
        self.source = self.g.VMobject(x=0., width=2.)
        self.target = self.g.VMobject(x=6., width=4.)
    def animation(self, **kwargs):
        return self.g.FadeTransform(self.source, self.target, **kwargs)
    def test_public_identity_and_transform_inheritance(self):
        self.assertIs(self.original, self.g.FadeTransform)
        self.assertTrue(issubclass(self.original, self.g.Transform))
        self.assertIsNone(self.animation()._native_kind)
    def test_group_contains_original_source_and_independent_target(self):
        anim = self.animation()
        self.assertIs(anim.mobject[0], self.source)
        self.assertIsNot(anim.mobject[1], self.target)
        self.assertIs(anim.to_add_on_completion, self.target)
    def test_midpoint_keeps_independent_fading_families(self):
        anim = self.animation()
        anim.begin(); anim.interpolate(.5)
        self.assertEqual([(obj.x,obj.width,obj.opacity) for obj in anim.mobject.submobjects], [(3.,3.,.5),(3.,3.,.5)])
        self.assertEqual((self.target.x,self.target.opacity), (6.,1.))
    def test_zero_state_is_prepared_before_authored_interpolation(self):
        observations = []
        class Custom(self.g.FadeTransform):
            def interpolate(self, alpha):
                super().interpolate(alpha)
                observations.append((alpha, self.mobject[0].opacity, self.mobject[1].opacity))
        anim = Custom(self.source, self.target)
        anim.begin()
        self.assertEqual(observations, [(0.,1.,0.)])
    def test_nonzero_rate_zero_is_not_overwritten(self):
        anim = self.animation(rate_func=lambda t:.25+.5*t)
        anim.begin()
        self.assertEqual((anim.mobject[0].opacity,anim.mobject[1].opacity), (.75,.25))
    def test_live_ghost_hook_and_options(self):
        calls = []
        class Custom(self.g.FadeTransform):
            def ghost_to(self, source, target):
                calls.append((self.stretch,self.dim_to_match))
                super().ghost_to(source,target)
                source.x += 2
        anim = Custom(self.source,self.target,stretch=False,dim_to_match=0)
        anim.begin(); anim.interpolate(.5)
        self.assertEqual(calls, [(False,0),(False,0)])
        self.assertEqual([m.x for m in anim.mobject.submobjects], [4.,4.])
    def test_authored_path_receives_native_geometry_arrays(self):
        calls = []
        def path(a,b,t):
            calls.append(t)
            return (1-t)*a+t*b+8*t*(1-t)
        anim = self.animation(path_func=path)
        anim.begin(); anim.interpolate(.5)
        self.assertEqual(anim.mobject[0].x, 5.)
        self.assertIn(.5,calls)
    def test_final_alpha_and_reverse_sampling(self):
        anim = self.animation(final_alpha_value=.25)
        anim.begin(); anim.interpolate(.8); anim.interpolate(.2)
        self.assertAlmostEqual(anim.mobject[0].opacity,.8)
        anim.finish()
        self.assertAlmostEqual(anim.mobject[0].opacity,.75)
        anim.finish()
    def test_time_span_uses_shared_protocol(self):
        anim = self.animation(run_time=2.,time_span=(.5,1.5))
        anim.begin(); anim.interpolate(.5)
        self.assertAlmostEqual(anim.mobject[1].opacity,.5)
    def test_cleanup_restores_source_and_installs_actual_target(self):
        anim = self.animation()
        scene = self.g.Scene(); scene.add(self.source)
        scene.play(anim)
        self.assertEqual(scene.mobjects,[self.target])
        self.assertEqual((self.source.x,self.source.width,self.source.opacity),(0.,2.,1.))
        anim.clean_up_from_scene(scene)
        self.assertEqual(scene.mobjects,[self.target])
    def test_remover_does_not_install_target(self):
        scene = self.g.Scene(); scene.play(self.animation(remover=True))
        self.assertEqual(scene.mobjects,[])
    def test_saved_target_selected_at_construction(self):
        anim = self.animation()
        self.source.x = 9; self.source.save_state()
        scene = self.g.Scene(); scene.play(anim)
        self.assertEqual(self.source.x,0.)
        self.assertEqual(self.source.saved_state.x,9.)
    def test_helpers_update_in_start_end_order(self):
        anim = self.animation(); anim.begin()
        calls = []
        anim.starting_mobject.updaters = [lambda m,dt:calls.append(("start",dt))]
        anim.ending_mobject.updaters = [lambda m,dt:calls.append(("end",dt))]
        anim.update_mobjects(.2)
        self.assertEqual(calls,[("start",.2),("end",.2)])
    def test_suspend_preserves_existing_descendant_state(self):
        self.source.suspended = True
        anim = self.animation(suspend_mobject_updating=True)
        anim.begin(); anim.finish()
        self.assertTrue(self.source.suspended)
        self.assertFalse(anim.mobject.suspended)
        self.assertFalse(anim.mobject[1].suspended)
    def test_abort_preserves_original_and_does_not_run_updaters(self):
        calls=[]; anim=self.animation(suspend_mobject_updating=True)
        anim.mobject.updaters=[lambda m,dt:calls.append(dt)]
        anim.begin(); anim.abort(); anim.abort()
        self.assertEqual(calls,[])
        self.assertFalse(anim.mobject.suspended)
        self.assertFalse(anim.mobject.animating)
        self.assertFalse(anim._fade_finished)
    def test_scene_failure_unwinds_nested_fades(self):
        anim=self.animation(suspend_mobject_updating=True)
        scene=self.g.Scene(); scene.fail_updater=True
        with self.assertRaisesRegex(RuntimeError,"scene updater"):
            scene.play(self.g.AnimationGroup(anim))
        self.assertFalse(anim.mobject.suspended)
        self.assertNotIn(self.target,scene.mobjects)
    def test_ghost_failure_propagates_exact_exception(self):
        failure=ValueError("ghost")
        class Broken(self.g.FadeTransform):
            def ghost_to(self,a,b):
                raise failure
        anim=Broken(self.source,self.target,suspend_mobject_updating=True)
        with self.assertRaises(ValueError) as raised:
            anim.begin()
        self.assertIs(raised.exception,failure)
        self.assertFalse(anim.mobject.animating)
    def test_cleanup_unbegun_is_not_a_publication(self):
        anim=self.animation(); scene=self.g.Scene()
        anim.clean_up_from_scene(scene)
        self.assertEqual(scene.mobjects,[])
        with self.assertRaises(RuntimeError): anim.finish()
    def test_malformed_start_or_alias_refused(self):
        for value in (object(),self.g.Group(),None):
            anim=self.animation()
            anim.create_starting_mobject=lambda value=value:value
            with self.assertRaises(TypeError): anim.begin()
        anim=self.animation(); anim.create_starting_mobject=lambda:anim.mobject
        with self.assertRaisesRegex(ValueError,"aliases"): anim.begin()
    def test_invalid_inputs_do_not_save_source(self):
        for kwargs in ({"dim_to_match":3},{"dim_to_match":1.5}):
            with self.assertRaises((ValueError,TypeError)): self.animation(**kwargs)
            self.assertIsNone(self.source.saved_state)
        with self.assertRaises(TypeError): self.g.FadeTransform(self.g.CameraFrame(),self.target)
    def test_reinstall_preserves_later_authored_methods(self):
        method=lambda self:"custom"
        self.g.FadeTransform.begin=method
        install_fading(self.g)
        self.assertIs(self.g.FadeTransform.begin,method)
    def test_piece_family_alignment_before_ghosting(self):
        a=self.g.VMobject(self.g.VMobject(x=1.))
        b=self.g.VMobject(self.g.VMobject(x=3.),self.g.VMobject(x=7.))
        anim=self.g.FadeTransformPieces(a,b,stretch=False,dim_to_match=0)
        anim.begin()
        self.assertEqual(len(anim.mobject[0].submobjects),2)
        self.assertEqual(len(anim.starting_mobject.get_family()),len(anim.ending_mobject.get_family()))
        scene=self.g.Scene(); anim.finish(); anim.clean_up_from_scene(scene)
        self.assertEqual(len(a.submobjects),1)
        self.assertEqual(scene.mobjects,[b])
    def test_piece_type_check(self):
        with self.assertRaises(TypeError): self.g.FadeTransformPieces(self.g.Mobject(),self.target)
    def test_instance_monkeypatch_is_executed(self):
        anim=self.animation(); calls=[]
        original=anim.ghost_to
        anim.ghost_to=lambda a,b:(calls.append((a,b)),original(a,b))
        anim.begin()
        self.assertEqual(len(calls),2)

    def test_actual_package_installs_fading_without_namespace_leaks(self):
        import fmn_python.fading as fading_module
        source = (pathlib.Path(__file__).resolve().parents[1] / "python/manimlib/__init__.py").read_text()
        tree = ast.parse(source)
        aliases = next(ast.literal_eval(node.value) for node in tree.body
                       if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name)
                       and target.id == "_REFERENCE_CLASS_BY_RUST_HELPER" for target in node.targets))
        native = types.ModuleType("manimlib.manimlib")
        vars(native).update(vars(environment()))
        package = types.ModuleType("manimlib"); package.__path__ = []
        distribution = types.ModuleType("fmn_python"); distribution.__path__ = []
        distribution._ensure_exclusive_manimlib_namespace = lambda: None
        shapes = types.ModuleType("manimlib.protocol_shapes")
        for name in aliases.values():
            value = getattr(native, name, type(name, (), {}))
            setattr(native, name, value); setattr(shapes, name, value)
        authority = types.ModuleType("fmn_python.library_constructor_authority")
        authority.REFERENCE_CLASS_BY_RUST_HELPER = aliases
        authority.REFERENCE_MODULE_BY_RUST_HELPER = {key: shapes.__name__ for key in aliases}
        provenance = types.ModuleType("fmn_python.schema_provenance")
        provenance.SchemaProvenanceError = ValueError
        provenance.apply_schema_placeholder_provenance = lambda module: None
        rendering = types.ModuleType("fmn_python.rendering")
        rendering.install_scene_rendering = lambda module: None
        playback = types.ModuleType("fmn_python.playback")
        playback.install_scene_playback = lambda module: None
        for name in ("__version__", "__distribution__", "__franken_manim__", "__abi_policy__",
                     "__engine__", "__thread_policy__", "__reference_commit__"):
            setattr(native, name, "boundary-fixture")
        modules = {m.__name__: m for m in (native, package, distribution, shapes, authority,
                                          provenance, rendering, playback)}
        initialization = types.ModuleType("fmn_python.initialization")
        initialization.initialize = lambda module: fading_module.install_fading(module)
        modules["fmn_python.initialization"] = initialization
        modules["fmn_python.fading"] = fading_module
        original = native.FadeTransform
        with patch.dict(sys.modules, modules):
            exec(compile(source, "manimlib/__init__.py", "exec"), vars(package))
            self.assertIs(package.FadeTransform, original)
            a, b, scene = package.VMobject(), package.VMobject(x=4), package.Scene()
            scene.play(package.FadeTransform(a, b))
            self.assertEqual(scene.mobjects, [b])
            self.assertEqual({k for k in vars(package) if not k.startswith("_")},
                             {k for k in vars(native) if not k.startswith("_")})


if __name__ == "__main__":
    unittest.main()
