"""Native crossfade restoration must preserve graph and record identities."""
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m


def snapshot(root):
    result, seen = [], set()
    for member in root.get_family():
        if id(member) not in seen:
            seen.add(id(member))
            result.append((member, tuple(member.submobjects), member.data.copy(),
                           tuple(member.updaters)))
    return result


def assert_restored(root, saved):
    assert {id(member) for member in root.get_family()} == {id(row[0]) for row in saved}
    for member, children, data, updaters in saved:
        assert len(member.submobjects) == len(children)
        assert all(actual is expected for actual, expected in zip(member.submobjects, children))
        assert tuple(member.updaters) == updaters
        for key in data.dtype.names:
            np.testing.assert_array_equal(member.data[key], data[key])
        for child in children:
            assert any(parent is member for parent in child.parents)


def unequal_and_nested_sources_restore_the_original_graph():
    count = 0
    for nested in (False, True):
        for nsource, ntarget in ((1, 5), (2, 3), (3, 2), (5, 1)):
            source = m.VGroup(*(m.Square(side_length=.2 + i / 10).shift(i * m.RIGHT)
                                for i in range(nsource)))
            target = m.VGroup(*(m.Circle(radius=.3).shift(i * m.UP) for i in range(ntarget)))
            if nested:
                source = m.VGroup(m.VGroup(source), m.Square())
                target = m.VGroup(m.VGroup(target), m.VGroup(m.Circle(), m.Square()))
            saved, scene = snapshot(source), m.Scene()
            scene.add(source)
            animation = m.FadeTransformPieces(source, target)
            saved_copy = snapshot(animation._fade_saved_source)
            scene.play(m.AnimationGroup(animation), run_time=.125)
            assert scene.mobjects == [target]
            assert_restored(source, saved)
            assert_restored(animation._fade_saved_source, saved_copy)
            assert not source._is_updating_suspended()
            count += 1
    return count


def shared_children_and_named_parts_survive_restoration():
    child = m.Square(fill_opacity=.7)
    source = m.VGroup(m.VGroup(child), m.VGroup(child, m.Circle()))
    source.named_part = child
    original = snapshot(source)
    target = m.VGroup(m.VGroup(m.Circle(), m.Square(), m.Circle()), m.VGroup(m.Square()))
    scene = m.Scene().add(source)
    scene.play(m.FadeTransformPieces(source, target), run_time=.125)
    assert_restored(source, original)
    assert source[0][0] is source[1][0] is source.named_part is child
    scene.remove(target).add(source)
    source.shift(m.UP)
    assert source[0][0] is source[1][0] is child


def a_later_saved_state_does_not_replace_the_fade_snapshot():
    source = m.VGroup(m.Square(), m.Circle().shift(m.RIGHT))
    initial = snapshot(source)
    target = m.VGroup(m.Circle(), m.Square(), m.Circle())
    animation = m.FadeTransformPieces(source, target, remover=True)
    source.shift(2 * m.UP).save_state()
    newer = source.saved_state
    scene = m.Scene().add(source)
    scene.play(animation, run_time=.125)
    assert source.saved_state is newer and scene.mobjects == []
    assert_restored(source, initial)


def restored_source_renders_identically_after_reuse():
    class Image(m.Scene):
        changed = False

        def construct(self):
            source = m.VGroup(m.Square(side_length=1, fill_color=m.RED, fill_opacity=1),
                               m.Circle(radius=.4, fill_color=m.BLUE, fill_opacity=1).shift(2*m.RIGHT))
            self.add(source)
            if self.changed:
                target = m.VGroup(m.Circle(), m.Square(), m.Circle().shift(m.UP))
                self.play(m.FadeTransformPieces(source, target), run_time=.125)
                self.remove(target).add(source)

    images = []
    with tempfile.TemporaryDirectory(prefix="fmn-fade-restoration-") as directory:
        for changed, threads in ((False, 1), (True, 1), (True, 4)):
            scene = Image()
            scene.changed = changed
            path = Path(directory) / f"restored-{changed}-{threads}.png"
            scene._begin_png(str(path), 96, 64, 30, threads, 0)
            try:
                scene.run()
                scene._finish_render(scene.frame._core, scene.camera.light_source.get_center())
            except BaseException:
                scene._abort_render()
                raise
            images.append(path.read_bytes())
    assert images[0] == images[1] == images[2], "crossfade reuse changed native pixels"


_count = unequal_and_nested_sources_restore_the_original_graph()
for _case in (shared_children_and_named_parts_survive_restoration,
              a_later_saved_state_does_not_replace_the_fade_snapshot,
              restored_source_renders_identically_after_reuse):
    _case()
print(f"fade restoration: {_count} unequal/nested families and 3 identity/render cases passed")
