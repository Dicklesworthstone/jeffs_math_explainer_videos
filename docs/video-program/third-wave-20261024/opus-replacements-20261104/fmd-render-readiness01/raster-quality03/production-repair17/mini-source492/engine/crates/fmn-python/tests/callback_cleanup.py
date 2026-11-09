"""Real-extension tests for exactly-once callback scene cleanup."""
import manimlib as m


def verify_callback_cleanup():
    assert getattr(m, "__franken_manim__", False)

    class PublishTarget(m.Animation):
        def __init__(self, source, target, **kwargs):
            self.target = target
            self.cleanups = 0
            super().__init__(source, remover=True, run_time=2 / 30, **kwargs)

        def interpolate_mobject(self, alpha):
            pass

        def clean_up_from_scene(self, scene):
            self.cleanups += 1
            scene.remove(self.mobject)
            scene.add(self.target)

    # Target and temporary source intentionally share a child. Cleanup may
    # publish that child in a new group; the placeholder must not remove it.
    for nested in (False, True):
        child = m.Square()
        source = m.VGroup(child)
        target = m.VGroup(child, m.Circle().shift(m.RIGHT))
        unrelated = m.Triangle().shift(3 * m.UP)
        animation = PublishTarget(source, target)
        scene = m.Scene()
        scene.add(source, unrelated)
        scene.play(m.AnimationGroup(animation) if nested else animation)
        assert animation.cleanups == 1
        assert sum(obj is target for obj in scene.mobjects) == 1
        assert source not in scene.mobjects
        assert unrelated in scene.mobjects
        assert target[0] is child and len(target.family_members_with_points()) == 2

    # An authored override may intentionally retain its animated object,
    # even with remover=True. Native cleanup must not override that choice.
    class Retain(m.Animation):
        def __init__(self, source):
            self.cleanups = 0
            super().__init__(source, remover=True, run_time=2 / 30)

        def interpolate_mobject(self, alpha):
            pass

        def clean_up_from_scene(self, scene):
            self.cleanups += 1

    retained = m.Circle()
    scene = m.Scene()
    scene.add(retained)
    keep = Retain(retained)
    scene.play(keep)
    assert keep.cleanups == 1 and retained in scene.mobjects

    # The inherited lifecycle still removes ordinary callback animations.
    class Remove(m.Animation):
        def interpolate_mobject(self, alpha):
            pass

    removed = m.Square()
    scene.add(removed)
    scene.play(Remove(removed, remover=True, run_time=2 / 30))
    assert removed not in scene.mobjects and retained in scene.mobjects
    return 4


if __name__ == "__main__":
    print("Callback cleanup native cases:", verify_callback_cleanup())
