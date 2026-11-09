"""Exercise the shipping family walker without needing native record storage."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import random
from types import SimpleNamespace
import unittest


_SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/copying.py"
_SPEC = importlib.util.spec_from_file_location("family_preorder_copying_adapter", _SOURCE)
assert _SPEC is not None and _SPEC.loader is not None
copying = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(copying)


class FamilyCycleError(ValueError):
    pass


class Member:
    def __init__(self, *children):
        self.submobjects = list(children)


def recursive_preorder(root):
    """The previous bootstrap algorithm, retained as a shallow-graph oracle."""
    result, seen, visiting = [], set(), set()

    def visit(member):
        marker = id(member)
        if marker in visiting:
            raise FamilyCycleError("submobjects would create a family cycle")
        if marker in seen:
            return
        if not isinstance(member, Member):
            raise TypeError("submobjects must be Mobject instances")
        visiting.add(marker)
        seen.add(marker)
        result.append(member)
        for child in member.submobjects:
            visit(child)
        visiting.remove(marker)

    visit(root)
    return result


class FamilyPreorderTests(unittest.TestCase):
    def setUp(self):
        self.g = {
            "_BridgeMobject": Member,
            "_FamilyCycleError": FamilyCycleError,
            "_family_preorder": recursive_preorder,
        }
        copying._install_family_preorder(self.g)
        self.walk = self.g["_family_preorder"]

    def test_leaf_and_preorder_preserve_original_identities(self):
        leaf = Member()
        left = Member(leaf)
        right = Member()
        root = Member(left, right)
        self.assertEqual(self.walk(leaf), [leaf])
        self.assertEqual(self.walk(root), [root, left, leaf, right])

    def test_shared_dag_and_repeated_edges_visit_members_once(self):
        tail = Member()
        shared = Member(tail)
        left, right = Member(shared, shared), Member(shared)
        root = Member(left, right, tail)
        self.assertEqual(self.walk(root), [root, left, shared, tail, right])

    def test_deep_families_do_not_use_the_python_call_stack(self):
        chain = [Member() for _ in range(20_000)]
        for parent, child in zip(chain, chain[1:]):
            parent.submobjects.append(child)
        self.assertEqual(self.walk(chain[0]), chain)
        with self.assertRaises(RecursionError):
            recursive_preorder(chain[0])

    def test_deep_back_edge_still_raises_the_native_cycle_error(self):
        chain = [Member() for _ in range(5_000)]
        for parent, child in zip(chain, chain[1:]):
            parent.submobjects.append(child)
        chain[-1].submobjects.append(chain[100])
        with self.assertRaisesRegex(FamilyCycleError, "family cycle"):
            self.walk(chain[0])

    def test_self_and_indirect_cycles_are_not_hidden_by_seen(self):
        for indirect in (False, True):
            with self.subTest(indirect=indirect):
                root = Member()
                root.submobjects.append(Member(root) if indirect else root)
                with self.assertRaises(FamilyCycleError):
                    self.walk(root)

    def test_invalid_roots_and_children_keep_the_same_type_error(self):
        for root in (None, object(), Member(object())):
            with self.subTest(root=root):
                with self.assertRaisesRegex(TypeError, "submobjects must be Mobject instances"):
                    self.walk(root)

    def test_identity_walk_never_calls_member_hash_or_equality(self):
        class Unhashable(Member):
            __hash__ = None

            def __eq__(self, other):
                raise AssertionError("family traversal uses identity")

        shared = Unhashable()
        root = Unhashable(shared, shared)
        result = self.walk(root)
        self.assertEqual(list(map(id, result)), [id(root), id(shared)])

    def test_live_child_iteration_matches_recursive_observation_order(self):
        def observe(walker):
            events = []

            class Observed(Member):
                def __init__(self, name, *children):
                    self.name, self.children = name, children

                @property
                def submobjects(self):
                    events.append((self.name, "read"))

                    def children():
                        for child in self.children:
                            events.append((self.name, "yield", child.name))
                            yield child
                        events.append((self.name, "done"))

                    return children()

            shared = Observed("shared")
            root = Observed("root", Observed("left", shared), Observed("right", shared))
            return [member.name for member in walker(root)], events

        self.assertEqual(observe(self.walk), observe(recursive_preorder))

    def test_a_child_appended_during_descent_is_not_lost_to_a_snapshot(self):
        def observe(walker):
            root, late = Member(), Member()

            class Appending(Member):
                @property
                def submobjects(self):
                    root.submobjects.append(late)
                    return ()

            first = Appending.__new__(Appending)
            root.submobjects.append(first)
            result = walker(root)
            self.assertEqual(result, [root, first, late])

        observe(recursive_preorder)
        observe(self.walk)

    def test_authored_iterator_failure_propagates_unchanged(self):
        failure = RuntimeError("authored child iterator failed")

        class Failing(Member):
            @property
            def submobjects(self):
                yield Member()
                raise failure

        root = Failing.__new__(Failing)
        with self.assertRaises(RuntimeError) as raised:
            self.walk(root)
        self.assertIs(raised.exception, failure)
        self.assertEqual(len(self.walk(Member())), 1)

    def test_shared_children_are_not_read_a_second_time(self):
        reads = []

        class Observed(Member):
            @property
            def submobjects(self):
                reads.append(self)
                return ()

        shared = Observed.__new__(Observed)
        self.walk(Member(Member(shared), Member(shared)))
        self.assertEqual(reads, [shared])

    def test_random_dags_match_the_recursive_oracle(self):
        rng = random.Random(20260925)
        for _ in range(100):
            members = [Member() for _ in range(40)]
            for index, member in enumerate(members[:-1]):
                member.submobjects = [rng.choice(members[index + 1:]) for _ in range(rng.randrange(6))]
            self.assertEqual(self.walk(members[0]), recursive_preorder(members[0]))

    def test_copier_installer_captures_the_new_walker_and_is_idempotent(self):
        class Public(Member):
            def restore(self):
                return self

        original_walk = self.g["_family_preorder"]
        native = SimpleNamespace(
            **self.g,
            Mobject=Public,
            _np=SimpleNamespace(ndarray=tuple),
            _copy_mobject_graph=lambda *args, **kwargs: None,
        )
        copying.install_mobject_copying(native)
        before = (native._family_preorder, native._copy_mobject_graph, Public.restore)
        self.assertIsNot(before[0], original_walk)
        copying.install_mobject_copying(native)
        self.assertIs(native._family_preorder, before[0])
        self.assertIs(native._copy_mobject_graph, before[1])
        self.assertIs(Public.restore, before[2])

    def test_copy_adapter_uses_the_iterative_walker_for_deep_attribute_projection(self):
        native = SimpleNamespace(
            **self.g,
            _np=SimpleNamespace(ndarray=tuple),
        )

        def copy_shells(root, deep, memo, detach_bound=False):
            for member in native._family_preorder(root):
                memo[id(member)] = Member()
            return memo[id(root)]

        native._copy_mobject_graph = copy_shells
        # Deliberately hand the installer the recursive function: this test
        # also catches capturing it before the replacement is installed.
        native._family_preorder = recursive_preorder
        copying.install_mobject_copying(native)
        root = Member()
        for _ in range(5_000):
            root = Member(root)
        memo = {}
        result = native._copy_mobject_graph(root, False, memo)
        self.assertIs(result, memo[id(root)])
        self.assertEqual(len(memo), 5_001)

    def test_reduced_namespace_keeps_its_supplied_walker(self):
        g = {"_family_preorder": recursive_preorder}
        copying._install_family_preorder(g)
        self.assertIs(g["_family_preorder"], recursive_preorder)


if __name__ == "__main__":
    unittest.main()
