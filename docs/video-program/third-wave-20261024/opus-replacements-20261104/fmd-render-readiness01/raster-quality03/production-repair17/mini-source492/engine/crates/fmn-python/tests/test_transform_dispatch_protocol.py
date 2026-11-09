"""Production Transform route selection and Scene.play, with modeled storage.

The classifier and its frozen protocol table are extracted from the shipping
installer, not reimplemented by the fixture. Native geometry/render acceptance
remains in animation_semantics.py.
"""
import ast
from types import MethodType
import unittest

from test_composition_lifecycle_protocol import SOURCE, environment
from fmn_python.playback import _install_transform_dispatch


LIFECYCLE = (
    "begin", "finish", "update_mobjects", "interpolate", "interpolate_mobject",
    "interpolate_submobject", "clean_up_from_scene",
)
HELPERS = (
    "create_target", "create_starting_mobject", "init_path_func",
    "check_target_mobject_validity", "get_all_mobjects",
    "get_all_families_zipped", "get_all_mobjects_to_update",
    "get_sub_alpha", "time_spanned_alpha",
)


def install_dispatch(g):
    """Run the actual classifier/table declarations over fixture classes."""
    installer = next(node for node in ast.parse(SOURCE.read_text()).body
                     if isinstance(node, ast.FunctionDef) and node.name == "install")
    names = {"uses_python_path", "requires_python_animation"}
    assignments = {"transform_hooks", "transform_protocols"}
    nodes = [node for node in installer.body
             if (isinstance(node, ast.FunctionDef) and node.name in names)
             or (isinstance(node, ast.Assign) and any(
                 isinstance(target, ast.Name) and target.id in assignments
                 for target in node.targets))]
    assert len(nodes) == len(names) + len(assignments)
    g["original_requires_python"] = g["_requires_python_animation"]
    g["g"] = g
    exec(compile(ast.Module(nodes, []), str(SOURCE), "exec"), g)
    g["_requires_python_animation"] = g["requires_python_animation"]


def fixture(setup=None):
    g = environment()
    Transform = g["Transform"]
    def helper(self, *args):
        return None
    for name in HELPERS:
        if not hasattr(Transform, name):
            setattr(Transform, name, helper)

    class Specialized(Transform):
        # An existing shipped subclass has its own native behavior. Its
        # unmodified hooks must not be mistaken for authored overrides.
        def begin(self):
            super().begin()
        def interpolate_submobject(self, *args):
            pass
    g["Specialized"] = Specialized
    if setup is not None:
        setup(g)
    install_dispatch(g)
    _install_transform_dispatch(g)
    return g


class TransformDispatchTests(unittest.TestCase):
    def test_every_authored_lifecycle_hook_selects_callbacks(self):
        for name in LIFECYCLE:
            with self.subTest(hook=name):
                g = fixture()
                def authored(self, *args):
                    pass
                cls = type("Authored", (g["Transform"],), {name: authored})
                self.assertTrue(g["_requires_python_animation"](cls(g["Mobject"]())))

    def test_every_instance_lifecycle_hook_selects_callbacks(self):
        for name in LIFECYCLE:
            with self.subTest(hook=name):
                g = fixture()
                animation = g["Transform"](g["Mobject"]())
                setattr(animation, name, MethodType(lambda self, *args: None, animation))
                self.assertTrue(g["_requires_python_animation"](animation))

    def test_late_base_class_patch_is_compared_with_frozen_implementation(self):
        for name in LIFECYCLE:
            with self.subTest(hook=name):
                g = fixture()
                animation = g["Transform"](g["Mobject"]())
                setattr(g["Transform"], name, lambda self, *args: None)
                self.assertTrue(g["_requires_python_animation"](animation))

    def test_existing_helpers_still_select_callbacks(self):
        for name in HELPERS:
            with self.subTest(hook=name):
                g = fixture()
                cls = type("Authored", (g["Transform"],), {name: lambda self, *args: None})
                self.assertTrue(g["_requires_python_animation"](cls(g["Mobject"]())))

    def test_stock_and_inherited_shipped_protocols_remain_native(self):
        g = fixture()
        for cls in (g["Transform"], g["Specialized"],
                    type("Unchanged", (g["Transform"],), {}),
                    type("UnchangedSpecialized", (g["Specialized"],), {})):
            with self.subTest(cls=cls):
                self.assertFalse(g["_requires_python_animation"](cls(g["Mobject"]())))

    def test_overriding_specialized_hook_with_base_hook_is_still_authored(self):
        g = fixture()
        cls = type("Authored", (g["Specialized"],), {"begin": g["Transform"].begin})
        self.assertTrue(g["_requires_python_animation"](cls(g["Mobject"]())))

    def test_changed_base_reached_by_shipped_super_call_selects_callbacks(self):
        g = fixture()
        animation = g["Specialized"](g["Mobject"]())
        g["Transform"].begin = lambda self: None
        self.assertTrue(g["_requires_python_animation"](animation))

    def test_admission_does_not_evaluate_authored_descriptor(self):
        g = fixture()
        class Authored(g["Transform"]):
            @property
            def interpolate(self):
                raise AssertionError("classification must not run authored descriptors")
        self.assertTrue(g["_requires_python_animation"](Authored(g["Mobject"]())))

    def test_admission_does_not_evaluate_authored_attribute_access(self):
        g = fixture()
        class Authored(g["Transform"]):
            def __getattribute__(self, name):
                if name == "_native_kind":
                    raise AssertionError("classification must not run authored attribute hooks")
                return super().__getattribute__(name)
        self.assertTrue(g["_requires_python_animation"](Authored(g["Mobject"]())))

    def test_live_path_assignment_still_selects_callbacks(self):
        g = fixture()
        animation = g["Transform"](g["Mobject"]())
        animation.path_func = lambda start, end, alpha: start
        self.assertTrue(g["_requires_python_animation"](animation))
        animation.path_func = None
        self.assertFalse(g["_requires_python_animation"](animation))

    def test_scene_play_executes_authored_hooks_and_not_endpoint_only_kernel(self):
        g = fixture()
        parent = g["Transform"]
        class Authored(parent):
            def begin(self):
                self.authored = ["begin"]
                super().begin()
            def update_mobjects(self, dt):
                self.authored.append(("update", dt))
                super().update_mobjects(dt)
            def interpolate(self, alpha):
                # Deliberately do not move: a native endpoint lerp would
                # visibly change the scene despite this authored behavior.
                self.authored.append(("interpolate", alpha))
            def finish(self):
                self.authored.append("finish")
                super().finish()
            def clean_up_from_scene(self, scene):
                self.authored.append("cleanup")
                super().clean_up_from_scene(scene)
        mob = g["Mobject"]()
        animation = Authored(mob)
        scene = g["Scene"]()
        scene.play(animation)
        self.assertEqual(scene.calls[0][0][0][0], "python_callback")
        self.assertIs(scene.calls[0][1][0], animation)
        self.assertEqual(animation.authored[0], "begin")
        self.assertIn(("update", .25), animation.authored)
        self.assertIn(("interpolate", .5), animation.authored)
        self.assertEqual(animation.authored[-3:], ["finish", ("interpolate", 1.), "cleanup"])
        self.assertEqual(mob.x, 0.)
        self.assertFalse(mob.animating)

    def test_nested_group_keeps_authored_leaf_on_callback_route(self):
        g = fixture()
        class Authored(g["Transform"]):
            def interpolate(self, alpha):
                self.mobject.x = 10. + alpha
        mob = g["Mobject"]()
        leaf = Authored(mob)
        group = g["AnimationGroup"](g["AnimationGroup"](leaf))
        scene = g["Scene"]()
        scene.play(group)
        self.assertEqual(mob.x, 11.)
        self.assertTrue(leaf.events)
        self.assertFalse(mob.animating)


def mobject_fixture():
    def setup(g):
        Mobject, Transform = g["Mobject"], g["Transform"]
        # Model _install_live_state's instance attribute. It is not a class
        # property: class-method snapshots must not compare this list value.
        original_init = Mobject.__init__
        def init(self, *children):
            original_init(self, *children)
            self.submobjects = self.children
        Mobject.__init__ = init
        Mobject.interpolate = lambda self, start, end, alpha, path=None: self
        installer = next(node for node in ast.parse(SOURCE.read_text()).body
                         if isinstance(node, ast.FunctionDef) and node.name == "install")
        method = next(node for node in installer.body
                      if isinstance(node, ast.FunctionDef) and node.name == "transform_submobject")
        exec(compile(ast.Module([method], []), str(SOURCE), "exec"), g)
        Transform.interpolate_submobject = g["transform_submobject"]

        def interpolate(self, alpha):
            # Modeled storage, but real production dispatch into each object.
            for member in self.mobject.get_family():
                self.interpolate_submobject(member, member, self.target_mobject, alpha)

        Transform.interpolate = interpolate

    return fixture(setup)


class TransformMobjectDispatchTests(unittest.TestCase):
    def test_source_and_nested_source_interpolators_select_callbacks(self):
        for nested in (False, True):
            with self.subTest(nested=nested):
                g = mobject_fixture()
                class Authored(g["Mobject"]):
                    def interpolate(self, *args):
                        raise AssertionError("admission must not interpolate")
                child = Authored()
                root = g["Mobject"](child) if nested else child
                self.assertTrue(g["_requires_python_animation"](g["Transform"](root)))

    def test_target_family_overrides_select_callbacks(self):
        g = mobject_fixture()
        class Authored(g["Mobject"]):
            def get_family(self):
                raise AssertionError("admission must not evaluate authored family traversal")
        animation = g["Transform"](g["Mobject"](), g["Mobject"](Authored()))
        self.assertTrue(g["_requires_python_animation"](animation))

    def test_instance_and_later_base_patches_are_live(self):
        for instance in (True, False):
            with self.subTest(instance=instance):
                g = mobject_fixture()
                child = g["Mobject"]()
                animation = g["Transform"](g["Mobject"](child))
                self.assertFalse(g["_requires_python_animation"](animation))
                hook = lambda self, *args: None
                if instance:
                    child.interpolate = MethodType(hook, child)
                else:
                    g["Mobject"].interpolate = hook
                self.assertTrue(g["_requires_python_animation"](animation))

    def test_family_mutations_are_rechecked_between_plays(self):
        g = mobject_fixture()
        class Authored(g["Mobject"]):
            def interpolate(self, *args):
                return self
        root = g["Mobject"]()
        animation = g["Transform"](root)
        self.assertFalse(g["_requires_python_animation"](animation))
        root.children.append(Authored())
        self.assertTrue(g["_requires_python_animation"](animation))
        root.children.clear()
        self.assertFalse(g["_requires_python_animation"](animation))

    def test_authored_descriptors_are_not_evaluated_during_admission(self):
        for name in ("interpolate", "get_family", "submobjects"):
            with self.subTest(attribute=name):
                g = mobject_fixture()
                def fail(self):
                    raise AssertionError("admission evaluated an authored descriptor")
                descriptor = property(fail, lambda self, value: None)
                cls = type("Authored", (g["Mobject"],), {name: descriptor})
                self.assertTrue(g["_requires_python_animation"](g["Transform"](cls())))

    def test_authored_attribute_lookup_is_not_evaluated_during_admission(self):
        g = mobject_fixture()
        class Authored(g["Mobject"]):
            def __getattribute__(self, name):
                if name == "submobjects":
                    raise AssertionError("admission evaluated authored attribute lookup")
                return super().__getattribute__(name)
        self.assertTrue(g["_requires_python_animation"](g["Transform"](Authored())))

    def test_stock_families_and_unmodified_subclasses_remain_native(self):
        g = mobject_fixture()
        class Unchanged(g["Mobject"]):
            pass
        shared = Unchanged()
        root = g["Mobject"](g["Mobject"](shared), Unchanged(shared))
        self.assertFalse(g["_requires_python_animation"](g["Transform"](root)))

    def test_authored_child_iterators_are_not_consumed_during_admission(self):
        g = mobject_fixture()
        class AuthoredChildren(list):
            def __iter__(self):
                raise AssertionError("admission consumed an authored child iterator")
        root = g["Mobject"]()
        root.submobjects = AuthoredChildren()
        self.assertTrue(g["_requires_python_animation"](g["Transform"](root)))

    def test_deep_and_shared_families_do_not_recurse_or_expand_every_path(self):
        g = mobject_fixture()
        root = g["Mobject"]()
        # Only 1101 identities, but 2**1100 paths. Admission inspects behavior
        # once per identity; interpolation retains its own family semantics.
        for _ in range(1100):
            root = g["Mobject"](root, root)
        self.assertFalse(g["_requires_python_animation"](g["Transform"](root)))

    def test_scene_play_preserves_custom_object_inside_stock_transform(self):
        for grouped in (False, True):
            with self.subTest(composition=grouped):
                g = mobject_fixture()
                class Authored(g["Mobject"]):
                    def interpolate(self, start, end, alpha, path=None):
                        self.x = 100. + alpha
                        return self
                child = Authored()
                animation = g["Transform"](g["Mobject"](child))
                animation.path_func = None
                entry = g["AnimationGroup"](animation) if grouped else animation
                scene = g["Scene"]()
                scene.play(entry)
                self.assertEqual(child.x, 101.)
                self.assertTrue(animation.events)
                self.assertFalse(animation.mobject.animating)


if __name__ == "__main__":
    unittest.main()
