"""Production copy-graph orchestration over explicit native shell fixtures.

The real installed-extension counterpart exercises Marionette-bound and
nursery-owned mobjects, mutation independence, and Animation.copy aliases.
"""
import ast
import copy
import types
import unittest

from creation_protocol_support import BOOTSTRAP


def environment():
    g = {"_copy": copy, "_FamilyCycleError": ValueError}
    wanted = {"_family_preorder", "_copy_mobject_graph"}
    nodes = [node for node in ast.parse(BOOTSTRAP.read_text()).body
             if isinstance(node, ast.FunctionDef) and node.name in wanted]
    exec(compile(ast.Module(nodes, []), str(BOOTSTRAP), "exec"), g)

    def install(node):
        node.submobjects = []
        node.uniforms = types.SimpleNamespace(_extras={})
        node.updaters, node.parents = [], []
        node.saved_state = node.target = None

    class Node:
        def __init__(self, name, *children, bound=False):
            install(self)
            self.name, self.bound = name, bound
            self.payload = [name]
            self.submobjects.extend(children)
        def __deepcopy__(self, memo):
            return g["_copy_mobject_graph"](self, True, memo)
        def _is_bound(self):
            return self.bound
        def _copy_family_shells(self):
            if not self.bound:
                return None
            return [(old, type(old).__new__(type(old)))
                    for old in g["_family_preorder"](self)]
        def _copy_detached_state_to(self, target):
            target.native_marker = "copied nursery"
        def _engine_state(self):
            return {"native_marker": "restored native state"}
        def _restore_engine_state(self, state):
            self.__dict__.update(state)

    g.update(_BridgeMobject=Node, _install_live_state=install)
    return Node, g["_copy_mobject_graph"]


class CopyGraphTests(unittest.TestCase):
    def setUp(self):
        self.Node, self.clone = environment()

    def test_descendant_before_root_reuses_one_authoritative_copy(self):
        for bound in (False, True):
            leaf = self.Node("leaf", bound=bound)
            root = self.Node("root", leaf, bound=bound)
            root.alias = [leaf]
            copied_leaf, copied_root = copy.deepcopy([leaf, root])
            self.assertIs(copied_root.submobjects[0], copied_leaf)
            self.assertIs(copied_root.alias[0], copied_leaf)
            self.assertIsNot(copied_leaf, leaf)
            copied_leaf.payload.append("edited")
            self.assertEqual(leaf.payload, ["leaf"])

    def test_two_root_graphs_share_the_same_copied_descendant(self):
        for bound in (False, True):
            leaf = self.Node("leaf", bound=bound)
            left = self.Node("left", leaf, bound=bound)
            right = self.Node("right", leaf, bound=bound)
            a, b = copy.deepcopy([left, right])
            self.assertIs(a.submobjects[0], b.submobjects[0])
            self.assertIsNot(a.submobjects[0], leaf)

    def test_root_before_alias_keeps_existing_behavior(self):
        leaf = self.Node("leaf")
        root = self.Node("root", leaf)
        copied_root, copied_leaf = copy.deepcopy([root, leaf])
        self.assertIs(copied_root.submobjects[0], copied_leaf)

    def test_prepopulated_memo_is_not_overwritten_or_reinitialized(self):
        for bound in (False, True):
            leaf = self.Node("leaf", bound=bound)
            child = self.Node("child", leaf, bound=bound)
            root = self.Node("root", child, bound=bound)
            copied_child = copy.deepcopy(child)
            copied_child.payload.append("retained edit")
            sentinel = object()
            copied_child.uniforms._extras["sentinel"] = sentinel
            copied_child.updaters.append(sentinel)
            original_members = tuple(copied_child.submobjects)
            memo = {id(child): copied_child, id(leaf): original_members[0]}
            copied_root = copy.deepcopy(root, memo)
            self.assertIs(copied_root.submobjects[0], copied_child)
            self.assertIs(memo[id(child)], copied_child)
            self.assertEqual(tuple(copied_child.submobjects), original_members)
            self.assertEqual(copied_child.payload, ["child", "retained edit"])
            self.assertIs(copied_child.uniforms._extras["sentinel"], sentinel)
            self.assertEqual(copied_child.updaters, [sentinel])

    def test_animation_dictionary_aliases_before_root_remain_coherent(self):
        leaf = self.Node("leaf")
        root = self.Node("root", leaf)
        animation = types.SimpleNamespace(all_submobs=[leaf], mobject=root)
        duplicate = copy.deepcopy(animation)
        self.assertIs(duplicate.all_submobs[0], duplicate.mobject.submobjects[0])

    def test_detached_snapshot_branch_respects_shared_memo(self):
        leaf = self.Node("leaf", bound=True)
        root = self.Node("root", leaf, bound=True)
        copied_leaf = self.Node("already copied")
        duplicate = self.clone(root, True, {id(leaf): copied_leaf}, detach_bound=True)
        self.assertIs(duplicate.submobjects[0], copied_leaf)
        self.assertEqual(copied_leaf.name, "already copied")

    def test_attribute_cycles_are_remapped_to_copied_root(self):
        leaf = self.Node("leaf")
        root = self.Node("root", leaf)
        root.alias = leaf
        leaf.owner = root
        copied_leaf, copied_root = copy.deepcopy([leaf, root])
        self.assertIs(copied_leaf.owner, copied_root)
        self.assertIs(copied_root.alias, copied_leaf)
        self.assertIs(copied_root.submobjects[0], copied_leaf)

    def test_shallow_copy_keeps_unrelated_aliases_and_separates_family(self):
        leaf, external = self.Node("leaf"), self.Node("external")
        root = self.Node("root", leaf)
        root.alias, root.external, root.nested = leaf, external, [leaf]
        duplicate = self.clone(root, False, {})
        self.assertIs(duplicate.alias, duplicate.submobjects[0])
        self.assertIsNot(duplicate.alias, leaf)
        self.assertIs(duplicate.external, external)
        self.assertIs(duplicate.nested, root.nested)


if __name__ == "__main__":
    unittest.main()
