"""Installed-wheel input acceptance using real native controls and output.

Protocol fixtures do not establish these assertions. This script requires a
built extension and uses the actual Scene, CameraFrame, Atlas, Lumen and Reel.
"""
from contextlib import contextmanager
import importlib
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from fmn_python import render_scene
from manimlib.mobject.interactive import Button as QualifiedButton


@contextmanager
def isolated_dispatcher():
    # event_handler/__init__.py exports EventDispatcher; the top-level
    # manimlib namespace does not.
    module = importlib.import_module("manimlib.event_handler")
    previous = module.EVENT_DISPATCHER
    module.EVENT_DISPATCHER = module.EventDispatcher()
    try:
        yield module.EVENT_DISPATCHER
    finally:
        module.EVENT_DISPATCHER = previous


def world_point(scene, target):
    point = target.get_center()
    return scene.frame.from_fixed_frame_point(point) if target.is_fixed_in_frame() else point


def click(scene, target):
    return scene.on_mouse_press(world_point(scene, target), 1, 0)


def test_public_identities_and_control_bindings():
    assert m.Button is QualifiedButton
    shape = m.Square()
    button = m.Button(shape, lambda mob: None)
    motion = m.MotionMobject(m.Dot())
    checkbox = m.Checkbox()
    toggle = m.EnableDisableButton()
    slider = m.LinearNumberSlider()
    text = m.Textbox()
    panel = m.ControlPanel(checkbox)
    expected = (
        (button.mobject, m.EventType.MousePressEvent, button),
        (motion.mobject, m.EventType.MouseDragEvent, motion),
        (checkbox, m.EventType.MousePressEvent, checkbox),
        (toggle, m.EventType.MousePressEvent, toggle),
        (slider.slider, m.EventType.MouseDragEvent, slider),
        (text.box, m.EventType.MousePressEvent, text),
        (text, m.EventType.KeyPressEvent, text),
        (panel.panel, m.EventType.MouseScrollEvent, panel),
        (panel.panel_opener, m.EventType.MouseDragEvent, panel),
    )
    for target, event_type, owner in expected:
        assert any(listener.event_type == event_type and listener.callback.__self__ is owner
                   for listener in target.get_event_listners())
    assert panel.panel_opener_rect is panel.panel_opener[0]
    assert panel.panel_info_text is panel.panel_opener[1]
    assert panel.is_fixed_in_frame()


def test_button_press_changes_original_geometry_without_hover():
    shape = m.Square().shift(3 * m.RIGHT)
    seen = []
    def action(mob):
        seen.append(mob)
        mob.shift(m.UP)
    button = m.Button(shape, action)
    scene = m.Scene()
    scene.add(button)
    assert click(scene, shape) is False
    assert seen == [shape]
    np.testing.assert_allclose(shape.get_center(), [3, 1, 0], atol=1e-6)


def test_drag_capture_does_not_pan_camera():
    shape = m.Square().shift(2 * m.LEFT)
    motion = m.MotionMobject(shape)
    scene = m.Scene()
    scene.add(motion)
    camera = scene.frame.get_center().copy()
    click(scene, shape)
    scene.on_mouse_drag(2 * m.RIGHT, 4 * m.RIGHT, 1, 0)
    np.testing.assert_allclose(shape.get_center(), 2 * m.RIGHT, atol=1e-6)
    np.testing.assert_array_equal(scene.frame.get_center(), camera)


def test_fixed_slider_follows_native_camera_projection():
    slider = m.LinearNumberSlider(value=0., min_value=-2., max_value=2., step=0.5)
    slider.shift(m.LEFT)
    scene = m.Scene()
    scene.add(slider)
    scene.frame.shift(3 * m.RIGHT).scale(0.75)
    camera = scene.frame.get_center().copy()
    click(scene, slider.slider)
    end = scene.frame.from_fixed_frame_point(slider.slider_axis.get_end() + m.RIGHT)
    scene.on_mouse_drag(end, m.RIGHT, 1, 0)
    assert float(slider.get_value()) == 2.
    np.testing.assert_array_equal(scene.frame.get_center(), camera)


def test_checkbox_keeps_mark_at_resized_moved_box():
    checkbox = m.Checkbox(True).scale(2).shift(2 * m.RIGHT + m.UP)
    scene = m.Scene()
    scene.add(checkbox)
    click(scene, checkbox)
    assert not bool(checkbox.get_value())
    np.testing.assert_allclose(checkbox.box_content.get_center(), checkbox.box.get_center(), atol=1e-6)
    assert checkbox.box_content.get_width() <= checkbox.box.get_width() + 1e-6
    assert checkbox.box_content.get_height() <= checkbox.box.get_height() + 1e-6


def test_toggle_changes_color_not_native_box_points():
    toggle = m.EnableDisableButton(True).scale(1.5).shift(2 * m.RIGHT)
    before = toggle.box.get_points().copy()
    scene = m.Scene()
    scene.add(toggle)
    click(scene, toggle)
    assert not bool(toggle.get_value())
    np.testing.assert_array_equal(toggle.box.get_points(), before)
    np.testing.assert_allclose(m.color_to_rgb(toggle.box.get_fill_color()),
                               m.color_to_rgb(toggle.disable_color), atol=1e-6)


def test_textbox_consumes_selection_shortcuts_without_origin_jump():
    text = m.Textbox("x", isInitiallyActive=True).shift(2 * m.RIGHT)
    scene = m.InteractiveScene()
    scene.add(text)
    before = text.box.get_points().copy()
    assert scene.on_key_press(ord("a"), 0) is False
    assert text.get_value() == "xa"
    for symbol, modifiers in ((0xFF51, 0), (ord("c"), 2), (ord("v"), 64)):
        assert scene.on_key_press(symbol, modifiers) is False
    assert text.get_value() == "xa"
    scene.on_key_press(0xFF08, 0)
    assert text.get_value() == "x"
    np.testing.assert_array_equal(text.box.get_points(), before)
    np.testing.assert_allclose(text.text.get_center(), text.box.get_center(), atol=1e-6)


def test_textbox_keeps_real_text_and_live_selectors_across_bound_edits():
    widget = m.Textbox("seed", isInitiallyActive=True).shift(2 * m.RIGHT)
    assert type(widget.box) is m.Rectangle
    assert len(widget.box.get_vertices()) == 4
    assert type(widget.text) is m.Text
    assert widget.text.string == "seed"
    assert not widget.text.has_points()
    scene = m.Scene().add(widget)
    label = widget.text
    for value in ("native words", "x", "longer again", ""):
        widget.set_value(value)
        assert widget.text is label
        assert label.text == label.string == widget.get_value() == value
        assert not label.has_points()
        assert list(label._fmn_string_children) == list(label.submobjects)
        assert all(child.is_fixed_in_frame() for child in label.get_family())
        if value:
            selected = label.select_part(value)
            assert selected.has_points() or selected.family_members_with_points()
            selected.set_color(m.GREEN)
            assert all(child.get_fill_color() == m.GREEN for child in selected.family_members_with_points())
            np.testing.assert_allclose(label.get_center(), widget.box.get_center(), atol=2e-6)
        assert widget in scene.mobjects
    assert len(label.updaters) == 1


def test_textbox_edits_fit_live_box_and_preserve_preview_value_distinction():
    widget = m.Textbox("short", text_kwargs={"color": m.RED}, text_buff=.15)
    label = widget.text
    height = label.get_height()
    widget.box.stretch(1.5, 0).shift(m.RIGHT)
    widget.update_text("a deliberately long label that must fit")
    assert widget.get_value() == "short"
    assert label.string == "a deliberately long label that must fit"
    assert label.get_width() <= widget.box.get_width() - .3 + 3e-6
    assert label.get_height() <= height + 3e-6
    assert label.get_fill_color() == m.RED
    np.testing.assert_allclose(label.get_center(), widget.box.get_center(), atol=2e-6)
    widget.unfix_from_frame()
    widget.set_value("unfixed")
    assert not any(part.is_fixed_in_frame() for part in label.get_family())


def test_textbox_failed_native_typeset_preserves_value_text_family_and_spans():
    widget = m.Textbox("valid")
    m.Scene().add(widget)
    label, children = widget.text, tuple(widget.text.submobjects)
    points = label.get_all_points().copy()
    spans = list(label._string_sub_spans)
    try:
        widget.set_value("\U0001f980")
    except ValueError as error:
        # The portal's Textbox refusal names the schema-level class and keeps
        # Scribe's precise diagnostic (the native textbox_error spelling).
        assert str(error).startswith("unmapped glyph: "), error
        assert "U+1F980" in str(error) and "has no glyph" in str(error), error
    else:
        raise AssertionError("unmapped textbox glyph was accepted")
    try:
        widget.update_text("\U0001f980")
    except ValueError as error:
        assert str(error).startswith("unmapped glyph: "), error
    else:
        raise AssertionError("unmapped textbox preview glyph was accepted")
    assert widget.text is label and tuple(label.submobjects) == children
    assert widget.get_value() == label.string == "valid"
    assert label._string_sub_spans == spans
    np.testing.assert_array_equal(label.get_all_points(), points)
    for invalid in (None, 17, ["text"]):
        try:
            widget.update_text(invalid)
        except TypeError:
            pass
        else:
            raise AssertionError("non-string textbox preview was accepted")
    assert widget.get_value() == label.string == "valid"
    assert tuple(label.submobjects) == children


def test_textbox_copy_keeps_its_typed_label_attached_to_its_own_box():
    import copy
    source = m.Textbox("original")
    for duplicate in (source.copy(), copy.deepcopy(source)):
        assert type(duplicate.box) is m.Rectangle
        assert type(duplicate.text) is m.Text
        assert duplicate.text._fmn_textbox_box is duplicate.box
        duplicate.box.shift(3 * m.RIGHT)
        duplicate.update(0.)
        duplicate.set_value("copy")
        assert source.get_value() == source.text.string == "original"
        assert duplicate.get_value() == duplicate.text.string == "copy"
        np.testing.assert_allclose(duplicate.text.get_center(), duplicate.box.get_center(), atol=2e-6)
        assert source.text.get_center()[0] < duplicate.text.get_center()[0] - 2


def test_textbox_authored_errors_keep_identity_and_transactional_state():
    widget = m.Textbox("valid")
    m.Scene().add(widget)
    label, family = widget.text, tuple(widget.get_family())
    points = label.get_all_points().copy()
    spans = list(label._string_sub_spans)
    paths = [list(path) for path in label._string_sub_paths]
    # Matching a native diagnostic's words must not recategorize a user error.
    failure = ValueError("authored has no glyph failure")
    original = m.Text.init_points
    def authored_points(text):
        raise failure
    m.Text.init_points = authored_points
    try:
        for operation in (
            lambda: m.Textbox("construction"),
            lambda: widget.update_text("preview"),
            lambda: widget.set_value("replacement"),
        ):
            try:
                operation()
            except ValueError as error:
                assert error is failure
                assert str(error) == "authored has no glyph failure"
            else:
                raise AssertionError("authored textbox error was swallowed")
            assert widget.text is label and tuple(widget.get_family()) == family
            assert widget.get_value() == label.string == "valid"
            assert label._string_sub_spans == spans
            assert label._string_sub_paths == paths
            np.testing.assert_array_equal(label.get_all_points(), points)
    finally:
        m.Text.init_points = original


def test_textbox_primitives_run_authored_hooks_before_control_initialization():
    seen = []
    rectangle_points, text_points = m.Rectangle.init_points, m.Text.init_points
    def rectangle(self):
        rectangle_points(self)
        seen.append(("rectangle", self))
    def text(self):
        text_points(self)
        seen.append(("text", self))
    class Authored(m.Textbox):
        def init_data(self):
            assert [(name, child) for name, child in seen] == [("rectangle", self.box), ("text", self.text)]
            seen.append(("control", self))
            super().init_data()
    m.Rectangle.init_points, m.Text.init_points = rectangle, text
    try:
        widget = Authored("label")
    finally:
        m.Rectangle.init_points, m.Text.init_points = rectangle_points, text_points
    assert [name for name, _ in seen] == ["rectangle", "text", "control"]
    assert widget.text.string == "label"


def test_scene_membership_prevents_cross_scene_control_delivery():
    first, second = m.Checkbox(), m.Checkbox()
    one, two = m.Scene(), m.Scene()
    one.add(first)
    two.add(second)
    click(one, first)
    assert not bool(first.get_value()) and bool(second.get_value())
    one.remove(first)
    one.on_mouse_press(m.ORIGIN, 1, 0)
    assert bool(second.get_value())
    click(two, second)
    assert not bool(second.get_value())


def test_standalone_dispatch_contract_remains_unchanged():
    dispatcher = importlib.import_module("manimlib.event_handler").EventDispatcher()
    shape, seen = m.Square(), []
    listener = m.EventListener(shape, m.EventType.MouseDragEvent,
                               lambda mob, data: seen.append(mob))
    dispatcher.add_listner(listener)
    hover, destination = m.ORIGIN.copy(), 4 * m.RIGHT
    dispatcher(m.EventType.MouseMotionEvent, point=hover)
    dispatcher(m.EventType.MousePressEvent, point=destination)
    dispatcher(m.EventType.MouseDragEvent, point=destination)
    assert seen == [shape]
    assert dispatcher.get_mouse_point() is hover
    assert dispatcher.get_mouse_drag_point() is destination


def test_callback_failure_clears_capture_without_masking_error():
    shape = m.Square()
    failure = LookupError("native interaction witness")
    def fail(mob, data):
        raise failure
    shape.add_mouse_drag_listner(fail)
    scene = m.Scene()
    scene.add(shape)
    click(scene, shape)
    try:
        scene.on_mouse_drag(m.RIGHT, m.RIGHT, 1, 0)
    except LookupError as error:
        assert error is failure
    else:
        raise AssertionError("authored input exception disappeared")
    shape.remove_mouse_drag_listner(fail)
    seen = []
    shape.add_mouse_drag_listner(lambda mob, data: seen.append(mob))
    scene.on_mouse_drag(m.ORIGIN, m.LEFT, 1, 0)
    assert seen == []


def read_y4m(path):
    payload = Path(path).read_bytes()
    header, body = payload.split(b"\n", 1)
    assert header.startswith(b"YUV4MPEG2 ")
    fields = {token[:1]: token[1:] for token in header.split()[1:]}
    width, height = int(fields[b"W"]), int(fields[b"H"])
    frame_bytes = width * height * 3 // 2
    frames = []
    while body:
        marker, body = body.split(b"\n", 1)
        assert marker == b"FRAME"
        assert len(body) >= frame_bytes
        frames.append(np.frombuffer(body[:width * height], dtype=np.uint8).reshape(height, width))
        body = body[frame_bytes:]
    return payload, frames


def test_rendered_button_edit_changes_pixels_and_is_thread_repeatable():
    class ButtonEdit(m.Scene):
        def construct(self):
            square = m.Square(fill_opacity=1, fill_color=m.WHITE, stroke_width=0).shift(2 * m.LEFT)
            self.add(m.Button(square, lambda mob: mob.shift(4 * m.RIGHT)))
            self.wait(0.25)
            assert click(self, square) is False
            self.wait(0.25)
    with tempfile.TemporaryDirectory(prefix="fmn-input-") as directory:
        outputs = []
        for threads in (1, 4):
            destination = Path(directory) / f"input-{threads}.y4m"
            receipt = render_scene(ButtonEdit, destination, resolution=(64, 48), fps=4, threads=threads)
            assert receipt.frame_count == 2
            payload, frames = read_y4m(destination)
            assert len(frames) == 2 and not np.array_equal(frames[0], frames[1])
            centers = []
            for frame in frames:
                low, high = int(frame.min()), int(frame.max())
                assert high - low > 50
                ys, xs = np.nonzero(frame > low + (high - low) / 2)
                assert len(xs) > 8
                centers.append(float(xs.mean()))
            assert centers[1] > centers[0] + 10
            outputs.append(payload)
        assert outputs[0] == outputs[1]


def reference_checkbox_mark(box, checked, style=None):
    """Pinned interactive.py's public construction, independent of Checkbox."""
    style = dict(stroke_color=m.GREEN if checked else m.RED, stroke_width=6) if style is None else dict(style)
    if checked:
        edges = ((m.UP / 2 + 2 * m.LEFT, m.DOWN + m.LEFT),
                 (m.DOWN + m.LEFT, m.UP + m.RIGHT))
    else:
        edges = ((m.UP + m.LEFT, m.DOWN + m.RIGHT),
                 (m.UP + m.RIGHT, m.DOWN + m.LEFT))
    result = m.VGroup(*(m.Line(start, end, **style) for start, end in edges))
    result.stretch_to_fit_width(box.get_width())
    result.stretch_to_fit_height(box.get_height())
    result.scale(.5).move_to(box)
    return result


def assert_checkbox_mark(actual, expected):
    assert type(actual) is m.VGroup
    assert not actual.has_points()
    assert len(actual) == len(expected) == 2
    for line, control in zip(actual, expected):
        assert type(line) is m.Line
        np.testing.assert_array_equal(line.get_points(), control.get_points())
        assert line.get_stroke_color() == control.get_stroke_color()
        assert line.get_stroke_width() == control.get_stroke_width()
        np.testing.assert_allclose(line.get_stroke_opacity(), control.get_stroke_opacity())


def test_checkbox_uses_public_rectangle_and_two_native_lines():
    for checked in (True, False):
        widget = m.Checkbox(checked)
        assert type(widget.box) is m.Rectangle
        assert len(widget.box.get_vertices()) == 4
        assert_checkbox_mark(widget.box_content, reference_checkbox_mark(widget.box, checked))
        assert all(member.is_fixed_in_frame() for member in widget.get_family())
        assert len(widget.get_event_listners()) == 1


def test_checkbox_factories_precede_tracker_hooks_and_retain_authored_objects():
    events = []
    class Authored(m.Checkbox):
        def get_checkmark(self):
            events.append('check')
            assert type(self.box) is m.Rectangle
            self.authored_mark = m.VGroup(m.Circle(radius=.12), m.Dot(radius=.03))
            return self.authored_mark
        def init_data(self):
            events.append('data')
            assert self.box_content is self.authored_mark
            super().init_data()
            self.decoration = m.Dot().shift(m.LEFT)
            self.add(self.decoration)
        def init_uniforms(self):
            events.append('uniforms'); super().init_uniforms()
        def init_points(self):
            events.append('points'); super().init_points()
        def init_colors(self):
            events.append('colors'); super().init_colors()
    obj = Authored()
    assert events == ['check', 'data', 'uniforms', 'points', 'colors']
    assert obj.box_content is obj.authored_mark
    assert obj.submobjects == [obj.decoration, obj.box, obj.authored_mark]
    assert type(obj.box_content[0]) is m.Circle


def test_checkbox_primitives_dispatch_authored_line_hooks():
    original, calls = m.Line.init_points, []
    def draw(self):
        original(self)
        calls.append(self)
        self.shift(.1 * m.RIGHT)
    m.Line.init_points = draw
    try:
        checkbox = m.Checkbox()
    finally:
        m.Line.init_points = original
    assert len(calls) == 2
    assert list(checkbox.box_content) == calls


def test_checkbox_full_public_line_style_options_are_live():
    checks = dict(stroke_color=m.BLUE, stroke_width=3., stroke_opacity=.35)
    crosses = dict(stroke_color=m.YELLOW, stroke_width=8., stroke_opacity=.7)
    box = dict(width=1.2, height=.7, fill_color=m.GREEN, fill_opacity=.2, stroke_width=1.)
    widget = m.Checkbox(True, rect_kwargs=box, checkmark_kwargs=checks, cross_kwargs=crosses)
    assert_checkbox_mark(widget.box_content, reference_checkbox_mark(widget.box, True, checks))
    m.Scene().add(widget)
    root, children, box_points = widget.box_content, tuple(widget.box_content), widget.box.get_points().copy()
    widget.set_value(False)
    assert widget.box_content is root and tuple(root) == children
    assert_checkbox_mark(root, reference_checkbox_mark(widget.box, False, crosses))
    np.testing.assert_array_equal(widget.box.get_points(), box_points)
    assert checks['stroke_opacity'] == .35 and crosses['stroke_width'] == 8.
    assert widget.box.get_fill_color() == m.GREEN


def test_checkbox_getters_and_toggling_follow_live_box_geometry():
    obj = m.Checkbox().shift(2 * m.RIGHT + m.UP)
    obj.box.stretch(2., 0).stretch(1.4, 1)
    scene = m.Scene().add(obj)
    content, children = obj.box_content, tuple(obj.box_content)
    for value in (False, True, False):
        obj.set_value(value)
        assert obj.box_content is content and tuple(content) == children
        assert_checkbox_mark(content, reference_checkbox_mark(obj.box, value))
        assert obj in scene.mobjects
    assert obj.get_checkmark() is not content
    assert obj.get_cross() is not content


def test_checkbox_custom_records_and_bound_views_survive_toggling():
    class Authored(m.Checkbox):
        data_dtype = [*m.ValueTracker.data_dtype, ('custom', np.float32, (1,))]
        def init_points(self):
            super().init_points()
            self.set_points([[0., 0., 0.]])
            self.data['custom'][:] = 17.
    widget = Authored(z_index=9)
    m.Scene().add(widget)
    root_data, box_data = widget.data, widget.box.data
    for value in (False, True, False):
        widget.set_value(value)
        np.testing.assert_array_equal(root_data['custom'], [[17.]])
        np.testing.assert_array_equal(box_data['point'], widget.box.get_points())
        assert bool(widget.get_value()) is value
    assert widget.z_index == 9


def test_checkbox_mark_replacements_keep_the_live_box_camera_lock():
    for fixed in (True, False):
        widget = m.Checkbox()
        if not fixed:
            widget.unfix_from_frame()
        scene = m.Scene().add(widget)
        scene.frame.shift(3 * m.RIGHT).scale(.7)
        for value in (False, True):
            widget.set_value(value)
            assert widget.box.is_fixed_in_frame() is fixed
            assert all(part.is_fixed_in_frame() is fixed for part in widget.box_content.get_family())
            np.testing.assert_allclose(widget.box_content.get_center(), widget.box.get_center(), atol=1e-6)


def test_checkbox_copy_deepcopy_and_pickle_preserve_typed_native_families():
    import copy
    import pickle
    source = m.Checkbox()
    source_points = source.box_content.get_all_points().copy()
    for duplicate in (source.copy(), copy.deepcopy(source), pickle.loads(pickle.dumps(source))):  # ubs:ignore -- round-trip of this test's own trusted object graph
        assert type(duplicate.box) is m.Rectangle
        assert type(duplicate.box_content) is m.VGroup
        assert all(type(child) is m.Line for child in duplicate.box_content)
        assert duplicate.box is not source.box
        assert duplicate.box_content is not source.box_content
        duplicate.set_value(False)
        assert not bool(duplicate.get_value()) and bool(source.get_value())
        np.testing.assert_array_equal(source.box_content.get_all_points(), source_points)
        assert_checkbox_mark(duplicate.box_content, reference_checkbox_mark(duplicate.box, False))


def test_checkbox_failed_authored_factory_leaves_value_geometry_and_identity():
    failure = RuntimeError('custom cross failed')
    class Authored(m.Checkbox):
        def get_cross(self):
            raise failure
    widget = Authored()
    m.Scene().add(widget)
    content, children = widget.box_content, tuple(widget.box_content)
    points = content.get_all_points().copy()
    try:
        widget.set_value(False)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError('authored factory error was swallowed')
    assert bool(widget.get_value())
    assert widget.box_content is content and tuple(content) == children
    np.testing.assert_array_equal(content.get_all_points(), points)


def test_checkbox_invalid_factory_and_parameters_refuse_before_tracker_hooks():
    calls = []
    class BadMark(m.Checkbox):
        def get_checkmark(self):
            return 'not a mark'
        def init_data(self):
            calls.append('data'); super().init_data()
    for cls, options in ((BadMark, {}), (m.Checkbox, {'value': 1}),
                         (m.Checkbox, {'checkmark_kwargs': None}),
                         (m.Checkbox, {'box_content_buff': float('nan')})):
        try:
            cls(**options)
        except (TypeError, ValueError, AssertionError):
            pass
        else:
            raise AssertionError('invalid checkbox construction accepted')
    assert calls == []


def test_checkbox_animation_keeps_typed_line_endpoints_and_bool_state():
    widget = m.Checkbox()
    scene = m.Scene().add(widget)
    children = tuple(widget.box_content)
    scene.play(widget.animate.set_value(False), run_time=.125)
    assert not bool(widget.get_value())
    assert tuple(widget.box_content) == children
    assert_checkbox_mark(widget.box_content, reference_checkbox_mark(widget.box, False))


def test_checkbox_render_matches_independent_native_line_composition():
    def render(path, actual, workers):
        scene = m.Scene()
        with scene.render_session(path, format='png_sequence', resolution=(96,54), fps=8, threads=workers):
            if actual:
                widget = m.Checkbox()
                widget.scale(3).shift(m.RIGHT)
                scene.add(widget)
            else:
                box = m.Rectangle(width=.5, height=.5, fill_opacity=0.)
                box.scale(3).shift(m.RIGHT)
                mark = reference_checkbox_mark(box, True)
                root = m.Group(box, mark).fix_in_frame()
                scene.add(root)
            scene.frame.shift(2 * m.LEFT).scale(.7)
            for checked in (True, False, True):
                if actual:
                    widget.set_value(checked)
                else:
                    mark.become(reference_checkbox_mark(box, checked))
                    mark.fix_in_frame()
                scene.wait(.125)
        return [p.read_bytes() for p in sorted(path.glob('*.png'))]
    root = Path(tempfile.mkdtemp(prefix='fmn-typed-checkbox-'))
    control = render(root/'control', False, 1)
    assert len(control) == 3 and control[0] != control[1] and control[0] == control[2]
    for workers in (1,4,16):
        assert render(root/str(workers), True, workers) == control


for case in (
    test_textbox_keeps_real_text_and_live_selectors_across_bound_edits,
    test_textbox_edits_fit_live_box_and_preserve_preview_value_distinction,
    test_textbox_failed_native_typeset_preserves_value_text_family_and_spans,
    test_textbox_copy_keeps_its_typed_label_attached_to_its_own_box,
    test_textbox_authored_errors_keep_identity_and_transactional_state,
    test_textbox_primitives_run_authored_hooks_before_control_initialization,
    test_checkbox_mark_replacements_keep_the_live_box_camera_lock,
    test_checkbox_uses_public_rectangle_and_two_native_lines,
    test_checkbox_factories_precede_tracker_hooks_and_retain_authored_objects,
    test_checkbox_primitives_dispatch_authored_line_hooks,
    test_checkbox_full_public_line_style_options_are_live,
    test_checkbox_getters_and_toggling_follow_live_box_geometry,
    test_checkbox_custom_records_and_bound_views_survive_toggling,
    test_checkbox_copy_deepcopy_and_pickle_preserve_typed_native_families,
    test_checkbox_failed_authored_factory_leaves_value_geometry_and_identity,
    test_checkbox_invalid_factory_and_parameters_refuse_before_tracker_hooks,
    test_checkbox_animation_keeps_typed_line_endpoints_and_bool_state,
    test_checkbox_render_matches_independent_native_line_composition,
    test_public_identities_and_control_bindings,
    test_button_press_changes_original_geometry_without_hover,
    test_drag_capture_does_not_pan_camera,
    test_fixed_slider_follows_native_camera_projection,
    test_checkbox_keeps_mark_at_resized_moved_box,
    test_toggle_changes_color_not_native_box_points,
    test_textbox_consumes_selection_shortcuts_without_origin_jump,
    test_scene_membership_prevents_cross_scene_control_delivery,
    test_standalone_dispatch_contract_remains_unchanged,
    test_callback_failure_clears_capture_without_masking_error,
    test_rendered_button_edit_changes_pixels_and_is_thread_repeatable,
):
    with isolated_dispatcher():
        case()
    print("native control interaction passed:", case.__name__)
