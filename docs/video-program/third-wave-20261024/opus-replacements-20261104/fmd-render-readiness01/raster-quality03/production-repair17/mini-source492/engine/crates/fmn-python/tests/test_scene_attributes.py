"""Owned checkpoint data: production structural copier, no engine substitutes."""
from __future__ import annotations

from types import SimpleNamespace
import unittest
from unittest.mock import patch

import numpy as np
from fmn_python import scene_attributes as state


class Opaque:
    def __deepcopy__(self, memo):
        raise AssertionError("a checkpoint must not deepcopy an external object")

    def __eq__(self, other):
        raise AssertionError("a checkpoint must not execute external equality")

    __hash__ = object.__hash__


class StructuralStateTests(unittest.TestCase):
    def capture(self, *members):
        return state.capture(members, np)

    def test_nested_containers_are_detached_and_can_be_restored_repeatedly(self):
        obj = SimpleNamespace(values={"items": [1, bytearray(b"ab")]})
        snapshot = self.capture(obj)
        obj.values["items"][1][:] = b"zz"
        for _ in range(3):
            state.restore([obj], state.prepare_restore(snapshot, np, [obj]))
            self.assertEqual(obj.values, {"items": [1, bytearray(b"ab")]})
            obj.values["items"][0] = 99
        self.assertEqual(snapshot[id(obj)]["values"]["items"][0], 1)

    def test_cross_member_aliases_and_native_proxy_references_survive(self):
        a, b = SimpleNamespace(), SimpleNamespace()
        shared = {"items": [1]}
        a.values = b.values = shared
        a.peer, b.peer = b, a
        snapshot = self.capture(a, b)
        restored = state.prepare_restore(snapshot, np, [a, b])
        state.restore([a, b], restored)
        self.assertIs(a.values, b.values)
        self.assertIsNot(a.values, shared)
        self.assertIs(a.peer, b)
        self.assertIs(b.peer, a)

    def test_list_dict_and_tuple_cycles_keep_their_topology(self):
        items = []
        loop = (items,)
        items.append(loop)
        obj = SimpleNamespace(cycle=loop)
        values = self.capture(obj)[id(obj)]
        self.assertIs(values["cycle"][0][0], values["cycle"])
        d = {}; d["self"] = d
        obj.mapping = d
        values = self.capture(obj)[id(obj)]
        self.assertIs(values["mapping"]["self"], values["mapping"])
        self.assertTrue(state.same(np, values, self.capture(obj)[id(obj)]))

    def test_object_arrays_clone_containers_but_keep_proxy_identity(self):
        native = Opaque()
        array = np.empty((1, 3), dtype=object)
        values = [1, 2]
        array[0, 0], array[0, 1], array[0, 2] = native, values, values
        obj = SimpleNamespace(array=array)
        copied = self.capture(obj)[id(obj)]["array"]
        self.assertIs(copied[0, 0], native)
        self.assertIs(copied[0, 1], copied[0, 2])
        self.assertIsNot(copied[0, 1], values)
        values[0] = 99
        self.assertEqual(copied[0, 1], [1, 2])

    def test_self_referential_object_array_and_shared_numeric_array(self):
        array = np.empty(1, dtype=object); array[0] = array
        numeric = np.arange(6).reshape(2, 3)
        obj = SimpleNamespace(array=array, left=numeric, right=numeric)
        snapshot = self.capture(obj)
        copied = snapshot[id(obj)]
        self.assertIs(copied["array"][0], copied["array"])
        self.assertIs(copied["left"], copied["right"])
        self.assertTrue(state.same(np, snapshot, self.capture(obj)))

    def test_structured_object_fields_and_scalar_records_are_frozen(self):
        array = np.empty(2, dtype=[("point", "f8", 3), ("payload", "O")])
        array["point"] = [[1, 2, 3], [4, 5, 6]]
        values = [1, 2]
        array["payload"][0] = values
        array["payload"][1] = values
        obj = SimpleNamespace(array=array, scalar=array[0])
        snapshot = self.capture(obj)
        copied = snapshot[id(obj)]
        self.assertIs(copied["array"]["payload"][0], copied["scalar"]["payload"])
        self.assertTrue(state.same(np, snapshot, self.capture(obj)))
        values[0] = 99
        self.assertFalse(state.same(np, snapshot, self.capture(obj)))
        self.assertEqual(copied["scalar"]["payload"], [1, 2])

    def test_distinct_structured_fields_are_all_compared(self):
        for _ in range(30):
            a = np.zeros(4, dtype=[("a", "i8"), ("b", "i8"), ("c", "i8")])
            b = a.copy(); b["c"] = 1
            self.assertFalse(state.same(np, a, b))
            self.assertTrue(state.same(np, a, a.copy()))

    def test_readonly_shape_dtype_and_noncontiguous_values_survive(self):
        array = np.arange(24, dtype=np.int16).reshape(4, 6)[:, ::2]
        array.setflags(write=False)
        obj = SimpleNamespace(array=array)
        copied = self.capture(obj)[id(obj)]["array"]
        self.assertFalse(copied.flags.writeable)
        self.assertEqual(copied.dtype, array.dtype)
        np.testing.assert_array_equal(copied, array)
        self.assertTrue(state.same(np, array, copied))
        self.assertFalse(state.same(np, array, copied.astype(np.float64)))

    def test_opaque_objects_and_callables_are_identity_bearing(self):
        opaque = Opaque()
        callback = lambda: None
        obj = SimpleNamespace(resource=opaque, callback=callback)
        copied = self.capture(obj)[id(obj)]
        self.assertIs(copied["resource"], opaque)
        self.assertIs(copied["callback"], callback)
        self.assertTrue(state.same(np, copied, self.capture(obj)[id(obj)]))
        obj.resource = Opaque()
        self.assertFalse(state.same(np, copied, self.capture(obj)[id(obj)]))

    def test_scalar_kinds_signed_zero_and_nan_payload_are_not_conflated(self):
        for left, right in ((True, 1), (1, 1.), (0., -0.), (1j, complex(-0., 1.))):
            with self.subTest(left=left, right=right):
                self.assertFalse(state.same(np, left, right))
        values = np.array([0x7ff8000000000001, 0x7ff8000000000002], dtype="u8").view("f8")
        self.assertTrue(state.same(np, float(values[0]), float(values[0])))
        self.assertFalse(state.same(np, float(values[0]), float(values[1])))
        self.assertFalse(state.same(np, values[:1], values[1:]))

    def test_equal_values_with_different_aliases_are_different_states(self):
        shared = [1]
        self.assertFalse(state.same(np, [shared, shared], [[1], [1]]))
        self.assertFalse(state.same(np, [[1], [1]], [shared, shared]))
        # Some structures may already be identical, others not. Identity must
        # not bypass the global bidirectional alias check.
        self.assertFalse(state.same(np, [shared, shared], [shared, [1]]))

    def test_set_values_are_order_independent_and_opaque_elements_do_not_run_eq(self):
        opaque = Opaque()
        left = {opaque, (2, 3), (1, 3), "label", 4}
        right = set(reversed(list(left)))
        self.assertTrue(state.same(np, left, right))
        self.assertFalse(state.same(np, {True}, {1}))
        self.assertFalse(state.same(np, {opaque}, {Opaque()}))

    def test_set_hash_collisions_and_equal_nan_bits_preserve_multiplicity(self):
        modulus = (1 << 61) - 1
        left = {(1, 2), (1, 2 + modulus)}
        right = set()
        right.add(tuple([1, 2 + modulus])); right.add(tuple([1, 2]))
        self.assertTrue(state.same(np, left, right))
        a, b = float("nan"), float("nan")
        self.assertEqual(len({a, b}), 2)
        self.assertFalse(state.same(np, {a, b}, {a}))
        self.assertTrue(state.same(np, {a, b}, {float("nan"), float("nan")}))

    def test_restore_adds_removes_and_resets_only_owned_attributes(self):
        core, scene = Opaque(), Opaque()
        obj = SimpleNamespace(value=1, _core=core, _scene=scene, updaters=[])
        snapshot = self.capture(obj)
        del obj.value
        obj.added = 2
        callbacks = obj.updaters = [lambda: None]
        state.restore([obj], state.prepare_restore(snapshot, np, [obj]))
        self.assertEqual(obj.value, 1)
        self.assertFalse(hasattr(obj, "added"))
        self.assertIs(obj._core, core)
        self.assertIs(obj._scene, scene)
        self.assertIs(obj.updaters, callbacks)

    def test_restore_does_not_invoke_authored_setattr_or_deepcopy(self):
        class Obj:
            def __setattr__(self, name, value):
                raise AssertionError("restore must project data, not author a new edit")
            def __deepcopy__(self, memo):
                raise AssertionError("no second object graph")
        obj = Obj(); vars(obj)["value"] = [1]
        snapshot = self.capture(obj)
        vars(obj)["value"].append(2)
        state.restore([obj], state.prepare_restore(snapshot, np, [obj]))
        self.assertEqual(obj.value, [1])

    def test_array_byte_node_and_nesting_budgets_are_explicit(self):
        obj = SimpleNamespace(value=np.arange(8))
        with patch.object(state, "_MAX_BYTES", 4):
            with self.assertRaisesRegex(ValueError, "budget"):
                self.capture(obj)
        obj.value = bytearray(8)
        with patch.object(state, "_MAX_BYTES", 4):
            with self.assertRaisesRegex(ValueError, "budget"):
                self.capture(obj)
        obj.value = list(range(30))
        with patch.object(state, "_MAX_NODES", 12):
            with self.assertRaisesRegex(ValueError, "budget"):
                self.capture(obj)
        obj.value = [[[[[1]]]]]
        with patch.object(state, "_MAX_DEPTH", 4):
            with self.assertRaisesRegex(ValueError, "budget"):
                self.capture(obj)

    def test_invalid_restore_projection_is_rejected_during_preparation(self):
        obj = SimpleNamespace(value=1)
        for bad in ({}, {id(obj): []}, {id(obj): {"_core": None}}):
            with self.subTest(bad=bad):
                with self.assertRaisesRegex(ValueError, "captured family"):
                    state.prepare_restore(bad, np, [obj])
        self.assertEqual(obj.value, 1)


if __name__ == "__main__":
    unittest.main()
