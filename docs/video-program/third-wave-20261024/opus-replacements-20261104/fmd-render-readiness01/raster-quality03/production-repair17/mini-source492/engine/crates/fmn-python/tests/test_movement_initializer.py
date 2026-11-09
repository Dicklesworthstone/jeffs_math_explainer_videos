"""Execute the actual wheel initializer with a native-table protocol fixture."""
import ast
import importlib.util
import sys
import types
import unittest
from unittest.mock import patch

from movement_protocol_support import ROOT, environment, movement, point


class MovementInitializerTests(unittest.TestCase):
    def test_actual_initializer_installs_on_existing_class_without_export_leaks(self):
        native = environment(False)
        native.__name__ = "manimlib.manimlib"
        cls = native.Homotopy
        native.Transform = type("Transform", (native._NativeAnimation,), {})
        native.Restore = type("Restore", (native._NativeAnimation,), {})
        native._OUT = (0.,0.,1.)
        spec = importlib.util.spec_from_file_location("actual_playback",ROOT / "python/fmn_python/playback.py")
        playback = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(playback)
        path = ROOT / "python/manimlib/__init__.py"
        source = path.read_text()
        aliases = next(ast.literal_eval(node.value) for node in ast.parse(source).body
                       if isinstance(node, ast.Assign) and any(
                           isinstance(t,ast.Name) and t.id == "_REFERENCE_CLASS_BY_RUST_HELPER"
                           for t in node.targets))
        package = types.ModuleType("manimlib")
        package.__path__ = []
        package.__package__ = "manimlib"
        distribution = types.ModuleType("fmn_python")
        distribution.__path__ = []
        distribution._ensure_exclusive_manimlib_namespace = lambda:None
        shapes = types.ModuleType("manimlib.fixture_shapes")
        for name in aliases.values():
            value = type(name,(),{})
            setattr(native,name,value)
            setattr(shapes,name,value)
        for name in ("__version__","__distribution__","__franken_manim__","__abi_policy__",
                     "__engine__","__thread_policy__","__reference_commit__"):
            setattr(native,name,"native-boundary-fixture")
        native._FMN_PORTAL_RUNTIME_STATE = "ready"
        native._FMN_ANIMATION_SEMANTICS_INSTALLED = True
        authority = types.ModuleType("fmn_python.library_constructor_authority")
        authority.REFERENCE_CLASS_BY_RUST_HELPER = aliases
        authority.REFERENCE_MODULE_BY_RUST_HELPER = {key:shapes.__name__ for key in aliases}
        provenance = types.ModuleType("fmn_python.schema_provenance")
        provenance.SchemaProvenanceError = ValueError
        provenance.apply_schema_placeholder_provenance = lambda module:None
        modules = {module.__name__:module for module in
                   (native,package,distribution,shapes,authority,provenance)}
        for module_name,function in (("rendering","install_scene_rendering"),("fading","install_fading")):
            module = types.ModuleType("fmn_python."+module_name)
            setattr(module,function,lambda native:None)
            modules[module.__name__] = module
        modules["fmn_python.movement"] = movement
        modules["fmn_python.playback"] = playback
        init_mod = types.ModuleType("fmn_python.initialization")
        def mock_init(n):
            movement.install_movement(n)
            return n
        init_mod.initialize = mock_init
        modules["fmn_python.initialization"] = init_mod
        with patch.dict(sys.modules,modules):
            exec(compile(source,str(path),"exec"),vars(package))
            self.assertIs(package.Homotopy,cls)
            mob=point(native)
            animation=package.Homotopy(lambda x,y,z,t:(x+t,y,z),mob,
                                       rate_func=native._linear_rate,suspend_mobject_updating=True)
            animation.begin()
            self.assertTrue(mob.suspended)
            animation.finish()
            self.assertFalse(mob.suspended)
            self.assertTrue(native._FMN_MOVEMENT_INSTALLED)
            flow=package.PhaseFlow(lambda p:native.np.array([1.,0.,0.]),mob,virtual_time=0.)
            flow.begin()
            before=mob.points.copy()
            flow.finish()
            native.np.testing.assert_array_equal(mob.points,before)
            path=native.VMobject(points=[(0.,0.,0.),(2.,0.,0.)])
            motion=package.MoveAlongPath(mob,path,rate_func=native._linear_rate,final_alpha_value=.5)
            self.assertTrue(native._requires_python_animation(motion))
            motion.begin()
            motion.finish()
            self.assertEqual(mob.get_center()[0],1.)
            self.assertEqual({key for key in vars(package) if not key.startswith("_")},
                             {key for key in vars(native) if not key.startswith("_")})


if __name__ == "__main__":
    unittest.main()
