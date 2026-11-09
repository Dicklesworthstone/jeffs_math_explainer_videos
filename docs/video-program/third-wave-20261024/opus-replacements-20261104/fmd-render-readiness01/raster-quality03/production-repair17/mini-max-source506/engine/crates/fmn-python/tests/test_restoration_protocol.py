"""Restore dispatch tests; native geometry coverage lives in copying_semantics.py."""
from __future__ import annotations

import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest


_SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/copying.py"
_SPEC = importlib.util.spec_from_file_location("restoration_copying_adapter", _SOURCE)
assert _SPEC is not None and _SPEC.loader is not None
copying = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(copying)


def namespace():
    class Mobject:
        def restore(self):
            raise AssertionError("legacy detached restoration path was called")

        def become(self, target, match_updaters=False):
            self.calls.append((target, match_updaters))
            return self

    return SimpleNamespace(
        Mobject=Mobject,
        _copy_mobject_graph=lambda *args, **kwargs: None,
        _family_preorder=lambda root: [root],
        _np=SimpleNamespace(),
    )


class RestorationProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = namespace()
        self.cls = self.native.Mobject
        copying.install_mobject_copying(self.native)
        self.obj = self.cls()
        self.obj.calls = []

    def test_restore_uses_become_and_preserves_receiver_identity(self):
        saved = object()
        self.obj.saved_state = saved
        self.assertIs(self.obj.restore(), self.obj)
        self.assertEqual(self.obj.calls, [(saved, False)])

    def test_bound_and_detached_receivers_use_the_same_protocol(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                self.obj._is_bound = lambda: bound
                self.obj.saved_state = object()
                self.obj.restore()
                self.assertIs(self.obj.calls[-1][0], self.obj.saved_state)

    def test_missing_or_none_state_refuses_before_become(self):
        for present in (False, True):
            with self.subTest(present=present):
                if present:
                    self.obj.saved_state = None
                with self.assertRaisesRegex(Exception, "Trying to restore without having saved"):
                    self.obj.restore()
        self.assertEqual(self.obj.calls, [])

    def test_state_is_read_once_without_truthiness_coercion(self):
        class Saved:
            def __bool__(self):
                raise AssertionError("a saved mobject is not a boolean")

        saved, reads = Saved(), []

        class Authored(self.cls):
            @property
            def saved_state(self):
                reads.append("saved")
                return saved

        obj = Authored()
        obj.calls = []
        self.assertIs(obj.restore(), obj)
        self.assertEqual(reads, ["saved"])
        self.assertEqual(obj.calls, [(saved, False)])

    def test_authored_become_override_receives_exact_saved_target(self):
        seen = []

        class Authored(self.cls):
            def become(self, target):
                seen.append((self, target))
                return None  # Reference restore still returns self.

        obj = Authored()
        obj.saved_state = object()
        self.assertIs(obj.restore(), obj)
        self.assertEqual(seen, [(obj, obj.saved_state)])

    def test_become_failure_is_not_retried_or_translated(self):
        failure = RuntimeError("foreign owner or invalid record schema")

        def fail(target):
            self.obj.calls.append(target)
            raise failure

        self.obj.become = fail
        saved = self.obj.saved_state = object()
        with self.assertRaises(RuntimeError) as raised:
            self.obj.restore()
        self.assertIs(raised.exception, failure)
        self.assertEqual(self.obj.calls, [saved])
        self.assertIs(self.obj.saved_state, saved)

    def test_repeated_restore_uses_the_current_saved_state(self):
        states = [object(), object()]
        for saved in states:
            self.obj.saved_state = saved
            self.obj.restore()
        self.assertEqual(self.obj.calls, [(saved, False) for saved in states])

    def test_subclass_restore_override_and_public_class_identity_survive(self):
        class Authored(self.cls):
            def restore(self):
                return "authored"

        self.assertIs(self.native.Mobject, self.cls)
        self.assertEqual(Authored().restore(), "authored")
        self.assertEqual(self.cls.restore.__name__, "restore")

    def test_installation_is_idempotent(self):
        before = (self.cls.restore, self.native._copy_mobject_graph)
        copying.install_mobject_copying(self.native)
        self.assertIs(self.cls.restore, before[0])
        self.assertIs(self.native._copy_mobject_graph, before[1])

    def test_reduced_copier_namespace_still_installs(self):
        native = namespace()
        del native.Mobject
        copying.install_mobject_copying(native)
        self.assertTrue(native._FMN_MOBJECT_COPYING_INSTALLED)


if __name__ == "__main__":
    unittest.main()
