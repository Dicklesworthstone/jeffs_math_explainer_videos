"""Durable updater suspension across detached nurseries and scene adoption."""
import manimlib as m


def flags(root):
    return [member._is_updating_suspended() for member in root.get_family()]


def detached_recursive_flags_survive_scene_adoption():
    a, b = m.Square(), m.Circle()
    inner, root = m.Group(a, b), m.Group()
    root.add(inner)
    assert root.suspend_updating() is root
    assert flags(root) == [True] * 4
    assert not any(member._is_bound() for member in root.get_family())
    scene = m.Scene().add(root)
    assert flags(root) == [True] * 4
    assert root.resume_updating(call_updater=False) is root
    assert flags(root) == [False] * 4
    assert scene.mobjects == [root]


def self_only_scope_keeps_descendants_and_callbacks_untouched():
    for bound in (False, True):
        a, b = m.Square(), m.Circle()
        root, ticks = m.Group(a, b), []
        for name, member in (("root", root), ("a", a), ("b", b)):
            member.add_updater(lambda obj, dt, name=name: ticks.append((name, dt)), call=False)
        if bound:
            scene = m.Scene().add(root)
        root.suspend_updating(recurse=False)
        assert flags(root) == [True, False, False]
        root.update(.1)
        assert ticks == []
        root.resume_updating(recurse=False)
        assert ticks == [("root", 0.)], ticks
        assert flags(root) == [False] * 3


def child_resume_clears_ancestors_without_resuming_siblings():
    for bound in (False, True):
        a, b = m.Square(), m.Circle()
        inner, root = m.Group(a, b), m.Group()
        root.add(inner)
        ticks = []
        for name, member in (("root", root), ("inner", inner), ("a", a), ("b", b)):
            member.add_updater(lambda obj, dt, name=name: ticks.append((name, dt)), call=False)
        if bound:
            scene = m.Scene().add(root)
        root.suspend_updating()
        a.resume_updating(recurse=False, call_updater=False)
        assert flags(root) == [False, False, False, True]
        assert ticks == []
        root.update(.25)
        assert ticks == [("a", .25), ("inner", .25), ("root", .25)], ticks


def detached_parent_of_bound_child_is_not_an_invisible_suspension_barrier():
    a, b = m.Square(), m.Circle()
    wrapper = m.Group(a, b)
    scene = m.Scene().add(a)
    assert a._is_bound() and not wrapper._is_bound() and not b._is_bound()
    wrapper.suspend_updating()
    assert flags(wrapper) == [True] * 3
    a.resume_updating(recurse=False, call_updater=False)
    assert flags(wrapper) == [False, False, True]
    assert scene.mobjects == [a]
    assert not wrapper._is_bound() and not b._is_bound()


def shared_descendants_keep_identity_and_resume_runs_one_family_pass():
    for bound in (False, True):
        child = m.Square()
        left, right = m.Group(child), m.Group(child)
        root = m.Group(left, right)
        if bound:
            scene = m.Scene().add(root)
        root.suspend_updating()
        # Public get_family is path-wise: the shared leaf appears twice.
        assert flags(root) == [True] * 5
        root.resume_updating(call_updater=False)
        assert flags(root) == [False] * 5
        assert left[0] is right[0] is child
    # A tree witnesses the existing single child-first update(0) contract.
    a, b, ticks = m.Square(), m.Circle(), []
    root = m.Group(a, b)
    for name, member in (("root", root), ("a", a), ("b", b)):
        member.add_updater(lambda obj, dt, name=name: ticks.append((name, dt)), call=False)
    root.suspend_updating().resume_updating()
    assert ticks == [("a", 0.), ("b", 0.), ("root", 0.)], ticks


def direct_animation_restores_only_the_suspension_it_acquired():
    a, b = m.Line(m.ORIGIN, m.RIGHT), m.Line(m.UP, m.UP + m.RIGHT)
    root, ticks = m.VGroup(a, b), []
    b.suspend_updating()
    a.add_updater(lambda obj, dt: ticks.append(dt), call=False)
    animation = m.Homotopy(lambda x, y, z, t: (x + t, y, z), root,
                          suspend_mobject_updating=True, rate_func=m.linear)
    animation.begin()
    assert flags(root) == [True] * 3
    animation.finish()
    assert flags(root) == [False, False, True]
    assert ticks == [0.], ticks


def _python_path_family(root, bridge, cycle_error):
    """The bootstrap's get_family loop before it moved native, as the oracle."""
    family, visiting, stack = [], set(), [(True, root)]
    while stack:
        entering, mobject = stack.pop()
        marker = id(mobject)
        if not entering:
            visiting.remove(marker)
            continue
        if not isinstance(mobject, bridge):
            raise TypeError("submobjects must be Mobject instances")
        if marker in visiting:
            raise cycle_error("submobjects would create a family cycle")
        family.append(mobject)
        children = list(mobject.submobjects)
        if children:
            visiting.add(marker)
            stack.append((False, mobject))
            stack.extend((True, child) for child in reversed(children))
    return family


def native_family_walk_matches_the_python_loop():
    # fm-5wq.31: get_family's loop runs natively. It must return the same
    # path-wise identities, read `submobjects` through ordinary attribute
    # access in the same order, and fail the same way.
    import random
    g = m.Mobject.get_family.__globals__
    bridge, cycle_error = g["_BridgeMobject"], g["_FamilyCycleError"]

    def outcome(walk):
        try:
            return [id(member) for member in walk()]
        except Exception as error:
            return (type(error), str(error))

    rng, repeated = random.Random(5), 0
    for bound in (False, True):
        for _ in range(20):
            members = [m.Square() for _ in range(6)]
            for _ in range(rng.randrange(1, 8)):
                parents = rng.sample(range(len(members)), rng.randrange(1, 3))
                members.append(m.Group(*(members[i] for i in parents)))
            root = m.Group(*rng.sample(members, 3))
            if bound:
                m.Scene().add(root)
            expected = _python_path_family(root, bridge, cycle_error)
            assert outcome(root.get_family) == [id(member) for member in expected]
            repeated += len(expected) > len(set(map(id, expected)))
    assert repeated, "no generated family reached a shared descendant twice"

    reads, mode = [], [None]

    class Authored(m.Group):
        @property
        def submobjects(self):
            live = self.__dict__["submobjects"]
            reads.append((mode[0], id(self)))
            if mode[0] == "tuple":
                return tuple(live)
            if mode[0] == "foreign":
                return [object()]
            if mode[0] == "loop":
                return [self]
            if mode[0] == "raise":
                raise LookupError("authored getter")
            return live

        @submobjects.setter
        def submobjects(self, value):
            self.__dict__["submobjects"] = value

    shared = m.Square()
    inner = Authored(shared, m.Circle())
    root = Authored(inner, m.Dot(), Authored(shared), shared)
    for mode[0] in ("read", "tuple", "foreign", "loop", "raise"):
        reads.clear()
        expected = outcome(lambda: _python_path_family(root, bridge, cycle_error))
        expected_reads = list(reads)
        reads.clear()
        assert outcome(root.get_family) == expected, mode[0]
        assert reads == expected_reads, mode[0]
    assert expected == (LookupError, "authored getter")

    # has_updaters reads `updaters` in family order and stops at the first
    # truthy list, as any() over a generator does.
    class Watched(m.Square):
        @property
        def updaters(self):
            reads.append(id(self))
            return self.__dict__["updaters"]

        @updaters.setter
        def updaters(self, value):
            self.__dict__["updaters"] = value

    leaves = [Watched() for _ in range(4)]
    group = m.Group(*leaves)
    reads.clear()
    assert group.has_updaters() is False
    assert reads == [id(leaf) for leaf in leaves]
    leaves[2].add_updater(lambda mob: None, call=False)
    reads.clear()
    assert group.has_updaters() is True
    assert reads == [id(leaf) for leaf in leaves[:3]]


_CASES = (detached_recursive_flags_survive_scene_adoption,
          self_only_scope_keeps_descendants_and_callbacks_untouched,
          child_resume_clears_ancestors_without_resuming_siblings,
          detached_parent_of_bound_child_is_not_an_invisible_suspension_barrier,
          shared_descendants_keep_identity_and_resume_runs_one_family_pass,
          direct_animation_restores_only_the_suspension_it_acquired,
          native_family_walk_matches_the_python_loop)
for _case in _CASES:
    _case()
print(f"updater family acceptance: {len(_CASES)} native cases passed")
