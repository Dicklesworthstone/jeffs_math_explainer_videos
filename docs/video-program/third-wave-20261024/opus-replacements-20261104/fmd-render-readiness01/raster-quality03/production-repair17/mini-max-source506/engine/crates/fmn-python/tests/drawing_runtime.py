"""Native creation semantics: live outlines, invalidation and failure ownership."""
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m


def native_outlines_use_current_renderer_state():
    constructors = (
        lambda: m.Square(fill_opacity=.8),
        lambda: m.Circle(fill_opacity=.8),
        lambda: m.Line(m.ORIGIN, m.RIGHT),
        lambda: m.VGroup(m.Square(), m.Circle().shift(m.RIGHT)),
        lambda: m.Text("Native", font_size=20),
        lambda: m.VMobject(stroke_behind=False).set_points_as_corners(
            [(0, 0, 0), (1, 0, 0), (1, 1, 0)]),
    )
    for construct in constructors:
        for behind in (True, False):
            source = construct()
            source.set_stroke(behind=not behind).set_stroke(behind=behind)
            saved = [member.data.copy() for member in source.get_family()]
            animation = m.DrawBorderThenFill(source, stroke_color=np.array([0., 0., 0.]),
                                             stroke_width=3, rate_func=m.linear)
            outline = animation.get_outline()
            assert outline is not source
            for member in outline.family_members_with_points():
                assert bool(member.uniforms["stroke_behind"]) is behind
                assert member.get_stroke_color() == m.BLACK
                assert np.isclose(member.get_stroke_width(), 3)
                assert member.get_fill_opacity() == 0
            for member, before in zip(source.get_family(), saved):
                np.testing.assert_array_equal(member.data, before)
    return len(constructors) * 2


def joint_refresh_invalidates_the_actual_family_without_geometry_writes():
    child = m.Square()
    left, right = m.VGroup(child), m.VGroup(child)
    root = m.VGroup(left, right)
    unrelated = m.Circle()
    for bound in (False, True):
        if bound:
            scene = m.Scene().add(root, unrelated)
        before = [member.data.copy() for member in root.get_family()]
        for member in (*root.get_family(), unrelated):
            member.needs_new_joint_angles = False
        assert root.refresh_joint_angles() is root
        assert all(member.needs_new_joint_angles for member in root.get_family())
        assert not unrelated.needs_new_joint_angles
        assert left[0] is right[0] is child
        for member, saved in zip(root.get_family(), before):
            np.testing.assert_array_equal(member.data, saved)
    empty = m.VMobject()
    assert empty.refresh_joint_angles() is empty and empty.needs_new_joint_angles


def drawing_failure_never_runs_an_extra_live_updater():
    count = 0
    for base in (m.ShowCreation, m.DrawBorderThenFill, m.Write):
        for phase in ("begin", "helpers", "interpolate", "finish"):
            for nested in (False, True):
                source = m.VGroup(m.Square(fill_opacity=.7), m.Circle().shift(m.RIGHT))
                source[0].suspend_updating()
                family = tuple(source.get_family())
                live_ids = {id(member) for member in family}
                before_flags = [member._is_updating_suspended() for member in family]
                ticks = []
                for member in family:
                    member.add_updater(
                        lambda obj, dt: ticks.append(dt) if id(obj) in live_ids else None,
                        call=False,
                    )
                error = LookupError("authored drawing failure")
                failed_at = []

                class Failing(base):
                    def fail(self, event):
                        if event == phase:
                            failed_at.append(len(ticks))
                            raise error

                    def begin(self):
                        super().begin()
                        self.fail("begin")

                    def update_mobjects(self, dt):
                        self.fail("helpers")
                        return super().update_mobjects(dt)

                    def interpolate_submobject(self, *args):
                        if float(args[-1]) > 0:
                            self.fail("interpolate")
                        return super().interpolate_submobject(*args)

                    def finish(self):
                        self.fail("finish")
                        return super().finish()

                scene = m.Scene().add(source)
                animation = Failing(source, run_time=.125, rate_func=m.linear,
                                    suspend_mobject_updating=True)
                try:
                    scene.play(m.AnimationGroup(animation) if nested else animation)
                except LookupError as caught:
                    assert caught is error
                else:
                    raise AssertionError("drawing failure was swallowed")
                assert len(ticks) == failed_at[-1], (base.__name__, phase, ticks)
                assert [member._is_updating_suspended() for member in family] == before_flags
                assert all(not getattr(member, "_is_animating", False) for member in family)
                assert "_fmn_scene_execution" not in vars(scene)
                source.clear_updaters()
                scene.wait(.125)
                count += 1
    return count


def authored_drawing_phases_match_independently_authored_pixels():
    class Outline(m.DrawBorderThenFill):
        def get_outline(self):
            return super().get_outline().set_stroke(width=8)

    class Image(m.Scene):
        reference = False

        def construct(self):
            source = m.Square(fill_color=m.RED, stroke_color=m.BLUE,
                              fill_opacity=.4 if self.reference else .8,
                              stroke_width=6 if self.reference else 4)
            source.set_stroke(behind=True)
            self.add(source)
            if not self.reference:
                self.play(Outline(source, final_alpha_value=.75, rate_func=m.linear),
                          run_time=.125)

    images = []
    with tempfile.TemporaryDirectory(prefix="fmn-drawing-runtime-") as directory:
        for reference, threads in ((True, 1), (False, 1), (False, 4)):
            scene = Image()
            scene.reference = reference
            path = Path(directory) / f"drawing-{reference}-{threads}.png"
            scene._begin_png(str(path), 96, 64, 30, threads, 0)
            try:
                scene.run()
                scene._finish_render(scene.frame._core, scene.camera.light_source.get_center())
            except BaseException:
                scene._abort_render()
                raise
            images.append(path.read_bytes())
    assert images[0] == images[1] == images[2], "authored border/fill phases changed native pixels"


styles = native_outlines_use_current_renderer_state()
joint_refresh_invalidates_the_actual_family_without_geometry_writes()
failures = drawing_failure_never_runs_an_extra_live_updater()
authored_drawing_phases_match_independently_authored_pixels()
print(f"drawing runtime: {styles} outline styles, {failures} failure cases, family invalidation and native 1/4-thread PNG equality passed")
