#!/usr/bin/env python3
"""Generate fmn-mobject copy-semantics fixtures (fm-ncq, §8.3, §16.4).

The Reference's copy contract (mobject.py, 3b1b/manim @ 6199a00d) is pure
Python-object structure — no OpenGL, no fonts. Rather than stand up the
Reference's full GL/config import closure just to call `copy()`, this script
transcribes the exact algorithm from mobject.py — `copy()` line for line
(including the `stash_mobject_pointers` clear list `["parents", "target",
"saved_state"]` and the `family.index(value)` attribute remap), plus
`generate_target` / `save_state` / `restore` — onto minimal mock objects,
runs it over a matrix of representative object graphs, and records the
structural facts. The Rust parity test (tests/copy_semantics.rs) rebuilds
each graph through the Stage, performs the same operations, and asserts the
same facts hold of `copy_family_mapped` / `generate_target` / `save_state` /
`restore_mobject`.

The recorded facts are structural (family sizes, family-order indices of
remapped aliases, identity sharing, cleared links) — the categories the plan
fixes as API semantics: deep-copied data, by-reference updaters, remapped
family-internal aliases, cleared parents/target/saved_state.

Output (committed): crates/fmn-mobject/fixtures/copy_semantics.txt
"""

from __future__ import annotations

import copy
import itertools as it
import os
import random

import numpy as np


class RefMob:
    """The minimal slice of the Reference Mobject that `copy()` touches."""

    def __init__(self, name: str, n_points: int):
        self.name = name
        # Structured records, as in the Reference (`data` is an ndarray
        # attribute deep-copied by copy()'s `isinstance(value, np.ndarray)`
        # branch).
        self.data = np.zeros(
            n_points, dtype=[("point", "f4", 3), ("rgba", "f4", 4)]
        )
        self.uniforms = {"vec": np.array([1.0, 2.0]), "scalar": 1.0}
        self.submobjects: list[RefMob] = []
        self.parents: list[RefMob] = []
        self.updaters: list[object] = []
        self.target: RefMob | None = None
        self.saved_state: RefMob | None = None

    def add(self, *mobs: "RefMob") -> "RefMob":
        for mob in mobs:
            self.submobjects.append(mob)
            mob.parents.append(self)
        return self

    def get_family(self) -> list["RefMob"]:
        return [self, *it.chain(*(sm.get_family() for sm in self.submobjects))]

    # ------------------------------------------------- mobject.py::copy()
    def copy(self) -> "RefMob":
        result = copy.copy(self)

        result.parents = []
        result.target = None
        result.saved_state = None

        result.uniforms = {
            key: value.copy() if isinstance(value, np.ndarray) else value
            for key, value in self.uniforms.items()
        }

        result.submobjects = [sm.copy() for sm in self.submobjects]
        for sm in result.submobjects:
            sm.parents = [result]
        result_family = [
            result,
            *it.chain(*(sm.get_family() for sm in result.submobjects)),
        ]

        result.updaters = list(self.updaters)

        family = self.get_family()
        for attr, value in list(self.__dict__.items()):
            if isinstance(value, RefMob) and value is not self:
                if value in family:
                    setattr(result, attr, result_family[family.index(value)])
            elif isinstance(value, np.ndarray):
                setattr(result, attr, value.copy())
        return result

    # -------------------------- mobject.py target / saved-state machinery
    def generate_target(self) -> "RefMob":
        self.target = self.copy()
        self.target.saved_state = self.saved_state
        return self.target

    def save_state(self) -> "RefMob":
        self.saved_state = self.copy()
        self.saved_state.target = self.target
        return self

    def restore(self) -> "RefMob":
        if self.saved_state is None:
            raise Exception("Trying to restore without having saved")
        # become() over an identical family shape (alignment is a no-op):
        # per zipped member, set_data + set_uniforms.
        for sm1, sm2 in zip(self.get_family(), self.saved_state.get_family()):
            sm1.data = sm2.data.copy()
            sm1.uniforms = {
                key: value.copy() if isinstance(value, np.ndarray) else value
                for key, value in sm2.uniforms.items()
            }
        return self


def dedup_family(family: list[RefMob]) -> list[RefMob]:
    """The engine's ratified family (G0-1/D-11): each member exactly once,
    depth-first order. For trees (every graph below) this equals the
    Reference's family."""
    out: list[RefMob] = []
    for m in family:
        if not any(m is o for o in out):
            out.append(m)
    return out


def index_of(family: list[RefMob], member: RefMob) -> int:
    for i, m in enumerate(family):
        if m is member:
            return i
    raise AssertionError("member not in family")


def build_tree(
    rng: random.Random, max_depth: int, max_children: int, counter: list[int]
) -> RefMob:
    node = RefMob(f"n{counter[0]}", rng.randint(1, 6))
    counter[0] += 1
    if max_depth > 0:
        for _ in range(rng.randint(0, max_children)):
            node.add(build_tree(rng, max_depth - 1, max_children, counter))
    return node


def emit_tree(lines: list[str], root: RefMob) -> list[RefMob]:
    """NODE <index> <parent_index|-1> <n_points>, family (DFS) order."""
    family = dedup_family(root.get_family())
    for i, node in enumerate(family):
        parent = -1
        for j, cand in enumerate(family):
            if any(sm is node for sm in cand.submobjects):
                parent = j
                break
        lines.append(f"NODE {i} {parent} {len(node.data)}")
    return family


def copy_facts(
    lines: list[str],
    root: RefMob,
    aliases: list[tuple[RefMob, str]],
    updater_nodes: list[RefMob],
) -> None:
    """Run the transcribed copy() and record the §8.3 facts."""
    family = dedup_family(root.get_family())
    result = root.copy()
    result_family = dedup_family(result.get_family())

    lines.append(f"EXPECT family_size {len(result_family)}")
    order_ok = all(
        result_family[i].name == family[i].name for i in range(len(family))
    )
    lines.append(f"EXPECT order_preserved {int(order_ok)}")

    for owner, attr in aliases:
        owner_idx = index_of(family, owner)
        owner_copy = result_family[owner_idx]
        old_target = getattr(owner, attr)
        new_target = getattr(owner_copy, attr)
        old_idx = index_of(family, old_target)
        new_idx = index_of(result_family, new_target)
        lines.append(f"EXPECT alias {attr} {owner_idx} {old_idx} {new_idx}")

    for node in updater_nodes:
        idx = index_of(family, node)
        node_copy = result_family[idx]
        shared = len(node_copy.updaters) == len(node.updaters) and all(
            a is b for a, b in zip(node_copy.updaters, node.updaters)
        )
        lines.append(
            f"EXPECT updaters_shared {idx} {len(node.updaters)} {int(shared)}"
        )

    lines.append(f"EXPECT root_parents_cleared {int(len(result.parents) == 0)}")
    pointers_cleared = all(
        m.target is None and m.saved_state is None for m in result_family
    )
    lines.append(f"EXPECT pointers_cleared {int(pointers_cleared)}")

    # Data independence: mutate every original member; no copy member moves.
    for node in family:
        node.data["point"] += 99.0
        node.uniforms["vec"] += 99.0
    data_independent = all(
        float(m.data["point"].sum()) == 0.0 for m in result_family
    )
    uniforms_independent = all(
        float(m.uniforms["vec"].sum()) == 3.0 for m in result_family
    )
    lines.append(f"EXPECT data_independent {int(data_independent)}")
    lines.append(f"EXPECT uniforms_independent {int(uniforms_independent)}")


def main() -> None:
    rng = random.Random(0x8_3)  # §8.3
    lines: list[str] = [
        "# copy-semantics fixtures (fm-ncq) — generated by gen_copy_fixtures.py",
        "# Reference: 3b1b/manim @ 6199a00d, mobject.py copy()/generate_target/"
        "save_state/restore, transcribed verbatim onto mock objects.",
    ]

    # --- hand-built representative graphs -------------------------------
    def case(name: str) -> None:
        lines.append(f"CASE {name}")

    case("leaf")
    leaf = RefMob("n0", 4)
    emit_tree(lines, leaf)
    copy_facts(lines, leaf, [], [])
    lines.append("END")

    case("flat_pair")
    root = RefMob("n0", 1)
    root.add(RefMob("n1", 3), RefMob("n2", 5))
    emit_tree(lines, root)
    copy_facts(lines, root, [], [])
    lines.append("END")

    case("nested_tree")
    root = RefMob("n0", 1)
    a, b = RefMob("n1", 2), RefMob("n2", 3)
    a.add(RefMob("n3", 4), RefMob("n4", 2))
    b.add(RefMob("n5", 6))
    root.add(a, b)
    emit_tree(lines, root)
    copy_facts(lines, root, [], [])
    lines.append("END")

    case("alias_direct")
    root = RefMob("n0", 1)
    tip = RefMob("n1", 2)
    body = RefMob("n2", 3)
    root.add(body, tip)
    root.tip = tip  # the classic `self.arrow is self.submobjects[i]` alias
    lines_family = emit_tree(lines, root)
    lines.append(f"ALIAS tip 0 {index_of(lines_family, tip)}")
    copy_facts(lines, root, [(root, "tip")], [])
    lines.append("END")

    case("alias_deep")
    root = RefMob("n0", 1)
    mid = RefMob("n1", 2)
    deep = RefMob("n2", 3)
    mid.add(deep)
    root.add(mid)
    root.label = deep  # root alias to a grandchild
    mid.mark = deep  # interior alias into its own subtree
    fam = emit_tree(lines, root)
    lines.append(f"ALIAS label 0 {index_of(fam, deep)}")
    lines.append(f"ALIAS mark {index_of(fam, mid)} {index_of(fam, deep)}")
    copy_facts(lines, root, [(root, "label"), (mid, "mark")], [])
    lines.append("END")

    case("alias_external")
    root = RefMob("n0", 2)
    child = RefMob("n1", 3)
    root.add(child)
    external = RefMob("x0", 1)
    root.buddy = external  # not in the family: NOT remapped, shared
    emit_tree(lines, root)
    result = root.copy()
    lines.append(f"EXPECT external_alias_shared {int(result.buddy is external)}")
    copy_facts(lines, root, [], [])
    lines.append("END")

    case("updaters")
    root = RefMob("n0", 1)
    child = RefMob("n1", 4)
    root.add(child)
    root.updaters = [object(), object()]
    child.updaters = [object()]
    fam = emit_tree(lines, root)
    lines.append("UPDATERS 0 2")
    lines.append(f"UPDATERS {index_of(fam, child)} 1")
    copy_facts(lines, root, [], [root, child])
    lines.append("END")

    # --- the target / saved-state link topology (order-sensitive) -------
    case("save_then_generate")
    m = RefMob("n0", 3)
    emit_tree(lines, m)
    m.save_state()
    m.generate_target()
    lines.append(
        f"EXPECT target_saved_state_is_saved {int(m.target.saved_state is m.saved_state)}"
    )
    lines.append(f"EXPECT saved_target_is_none {int(m.saved_state.target is None)}")
    lines.append("END")

    case("generate_then_save")
    m = RefMob("n0", 3)
    emit_tree(lines, m)
    m.generate_target()
    m.save_state()
    lines.append(f"EXPECT saved_target_is_target {int(m.saved_state.target is m.target)}")
    lines.append(
        f"EXPECT target_saved_state_is_none {int(m.target.saved_state is None)}"
    )
    lines.append("END")

    case("restore_roundtrip")
    m = RefMob("n0", 2)
    child = RefMob("n1", 3)
    m.add(child)
    emit_tree(lines, m)
    m.data["point"][0] = (1.0, 2.0, 3.0)
    child.data["point"][1] = (4.0, 5.0, 6.0)
    m.save_state()
    m.data["point"][0] = (9.0, 9.0, 9.0)
    child.data["point"][1] = (8.0, 8.0, 8.0)
    child.uniforms["scalar"] = 7.0
    m.restore()
    ok = (
        tuple(m.data["point"][0]) == (1.0, 2.0, 3.0)
        and tuple(child.data["point"][1]) == (4.0, 5.0, 6.0)
        and child.uniforms["scalar"] == 1.0
    )
    lines.append(f"EXPECT restore_roundtrip {int(ok)}")
    # Repeated restore keeps working (the saved link survives).
    m.data["point"][0] = (5.0, 5.0, 5.0)
    m.restore()
    lines.append(
        f"EXPECT restore_repeatable {int(tuple(m.data['point'][0]) == (1.0, 2.0, 3.0))}"
    )
    lines.append("END")

    # --- seeded random trees: the property corpus ------------------------
    for i in range(12):
        case(f"random_{i}")
        counter = [0]
        root = build_tree(rng, max_depth=3, max_children=3, counter=counter)
        fam = dedup_family(root.get_family())
        aliases: list[tuple[RefMob, str]] = []
        updater_nodes: list[RefMob] = []
        if len(fam) > 1:
            # A root alias to a random member and updaters on a random node.
            member = fam[rng.randrange(1, len(fam))]
            root.ref0 = member
            aliases.append((root, "ref0"))
            up_node = fam[rng.randrange(len(fam))]
            up_node.updaters = [object() for _ in range(rng.randint(1, 3))]
            updater_nodes.append(up_node)
        emit_tree(lines, root)
        for owner, attr in aliases:
            lines.append(
                f"ALIAS {attr} {index_of(fam, owner)} "
                f"{index_of(fam, getattr(owner, attr))}"
            )
        for node in updater_nodes:
            lines.append(f"UPDATERS {index_of(fam, node)} {len(node.updaters)}")
        copy_facts(lines, root, aliases, updater_nodes)
        lines.append("END")

    out_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)),
        os.pardir,
        "crates",
        "fmn-mobject",
        "fixtures",
        "copy_semantics.txt",
    )
    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"wrote {os.path.normpath(out_path)} ({len(lines)} lines)")


if __name__ == "__main__":
    main()
