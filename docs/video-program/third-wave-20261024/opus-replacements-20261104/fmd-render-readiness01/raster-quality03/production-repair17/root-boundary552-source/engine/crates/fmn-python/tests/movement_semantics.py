"""Real installed-wheel movement acceptance; never replaced by protocol doubles."""
import numpy as np
from manimlib import Animation, AnimationGroup, ComplexHomotopy, Homotopy, Line, Scene, VGroup, linear


def check_homotopy_family_lag():
    a,b=Line([0,0,0],[1,0,0]),Line([0,2,0],[1,2,0])
    initial_a,initial_b=a.get_points().copy(),b.get_points().copy()
    scene=Scene()
    observed=[]
    class Reveal(Homotopy):
        def interpolate(self,alpha):
            super().interpolate(alpha)
            if abs(alpha-.5)<1e-8:
                observed.append((a.get_points().copy(),b.get_points().copy()))
    scene.play(Reveal(lambda x,y,z,t:(x+t,y,z),VGroup(a,b),run_time=1,
                      rate_func=linear,lag_ratio=1))
    assert observed, "the native clock never sampled the middle frame"
    np.testing.assert_allclose(observed[0][0],initial_a+[.5,0,0],atol=2e-6)
    np.testing.assert_allclose(observed[0][1],initial_b,atol=2e-6)


def check_start_hook_and_complex_map():
    line=Line([1,2,3],[2,2,3])
    original=line.get_points().copy()
    calls=[]
    class Custom(ComplexHomotopy):
        def create_starting_mobject(self):
            calls.append(self)
            return super().create_starting_mobject()
    animation=Custom(lambda z,t:z*(1+t),line,rate_func=linear)
    animation.begin()
    animation.interpolate(.5)
    expected=original.copy()
    expected[:,:2]*=1.5
    np.testing.assert_allclose(line.get_points(),expected,atol=2e-6)
    assert calls==[animation]
    animation.finish()


def check_time_span_and_reverse_interpolation():
    line=Line([0,0,0],[1,0,0])
    original=line.get_points().copy()
    animation=Homotopy(lambda x,y,z,t:(x+t,y,z),line,run_time=2,
                       rate_func=linear,time_span=(.5,1.5),final_alpha_value=.5)
    animation.begin()
    for alpha,offset in ((.75,1.),(.25,0.),(.5,.5)):
        animation.interpolate(alpha)
        np.testing.assert_allclose(line.get_points(),original+[offset,0,0],atol=2e-6)
    animation.finish()
    np.testing.assert_allclose(line.get_points(),original+[.5,0,0],atol=2e-6)


def check_suspension_and_resume():
    a,b=Line([0,0,0],[1,0,0]),Line([0,2,0],[1,2,0])
    root=VGroup(a,b)
    b.suspend_updating()
    calls=[]
    a.add_updater(lambda mob,dt:calls.append(dt),call=False)
    animation=Homotopy(lambda x,y,z,t:(x+t,y,z),root,rate_func=linear,
                       suspend_mobject_updating=True)
    animation.begin()
    assert a._is_updating_suspended() and b._is_updating_suspended()
    animation.finish()
    assert not a._is_updating_suspended() and b._is_updating_suspended()
    assert calls==[0.]


def check_nested_failure_and_recovery():
    scene=Scene()
    line=Line([0,0,0],[1,0,0])
    error=ValueError("authored deformation failed")
    def deform(x,y,z,t):
        if t>.2:
            raise error
        return x+t,y,z
    animation=Homotopy(deform,line,suspend_mobject_updating=True,rate_func=linear)
    try:
        scene.play(AnimationGroup(animation))
    except ValueError as caught:
        assert caught is error
    else:
        raise AssertionError("the authored failure was swallowed")
    assert not line._is_updating_suspended()
    animation.homotopy=lambda x,y,z,t:(x,y+t,z)
    scene.play(animation,run_time=.1)
    assert not line._is_updating_suspended()


def check_scene_updater_failure_unwinds():
    scene=Scene()
    line=Line([0,0,0],[1,0,0])
    sibling=Line([0,2,0],[1,2,0])
    def fail(mob,dt):
        raise ValueError("scene updater")
    sibling.add_updater(fail,call=False)
    scene.add(sibling)
    try:
        scene.play(Homotopy(lambda x,y,z,t:(x+t,y,z),line,suspend_mobject_updating=True))
    except ValueError as error:
        assert str(error)=="scene updater"
    else:
        raise AssertionError("scene updater failure was swallowed")
    assert not line._is_updating_suspended()


def check_phase_flow_replay_and_zero_time():
    from manimlib import PhaseFlow
    scene=Scene()
    line=Line([0,0,0],[1,0,0])
    original=line.get_points().copy()
    flow=PhaseFlow(lambda p:np.array([1.,0.,0.]),line,virtual_time=1.,run_time=.1)
    scene.play(flow)
    np.testing.assert_allclose(line.get_points(),original+[1.,0.,0.],atol=3e-6)
    scene.play(flow)
    np.testing.assert_allclose(line.get_points(),original+[2.,0.,0.],atol=3e-6)
    scene.play(PhaseFlow(lambda p:p,line,virtual_time=0.,run_time=.1))
    np.testing.assert_allclose(line.get_points(),original+[2.,0.,0.],atol=3e-6)


def check_flow_failure_recovery():
    from manimlib import PhaseFlow
    line=Line([1,0,0],[2,0,0])
    flow=PhaseFlow(lambda p:np.array([1.,0.,0.]),line,virtual_time=1.,suspend_mobject_updating=True)
    flow.begin()
    flow.interpolate(.25)
    before=line.get_points().copy()
    def fail(point):
        raise ValueError("flow failure")
    flow.function=fail
    try:
        flow.interpolate(.5)
    except ValueError:
        pass
    else:
        raise AssertionError("flow failure was ignored")
    assert not line._is_updating_suspended()
    flow.function=lambda p:np.array([1.,0.,0.])
    flow.begin()
    np.testing.assert_allclose(line.get_points(),before,atol=2e-6)
    flow.finish()
    np.testing.assert_allclose(line.get_points(),before+[1.,0.,0.],atol=2e-6)


def check_authored_path_sampler():
    from manimlib import MoveAlongPath
    calls=[]
    class Path(Line):
        def point_from_proportion(self,t):
            calls.append(t)
            return np.array([2*t,t*t,0.])
    path=Path([0,0,0],[2,1,0])
    moving=Line([-.1,0,0],[.1,0,0])
    seen=[]
    class Motion(MoveAlongPath):
        def interpolate(self,alpha):
            super().interpolate(alpha)
            seen.append((alpha,moving.get_center().copy()))
    Scene().play(Motion(moving,path,run_time=.2,rate_func=linear))
    assert calls and seen
    for alpha,center in seen:
        np.testing.assert_allclose(center,[2*alpha,alpha*alpha,0.],atol=3e-6)


def check_late_bound_path_and_move_hooks():
    from manimlib import MoveAlongPath
    import types
    path=Line([0,0,0],[2,0,0])
    moving=Line([-.1,0,0],[.1,0,0])
    animation=MoveAlongPath(moving,path,run_time=.1,rate_func=linear)
    moves=[]
    original_move=moving.move_to
    def sampler(self,t):
        return np.array([t,2*t,0.])
    def move(self,point):
        moves.append(np.array(point))
        return original_move(point)
    path.point_from_proportion=types.MethodType(sampler,path)
    moving.move_to=types.MethodType(move,moving)
    Scene().play(animation)
    assert moves
    np.testing.assert_allclose(moving.get_center(),[1.,2.,0.],atol=2e-6)


def check_path_remover_and_fractional_endpoint():
    from manimlib import MoveAlongPath
    scene=Scene()
    moving=Line([-.1,0,0],[.1,0,0])
    path=Line([-1,0,0],[1,0,0])
    scene.play(MoveAlongPath(moving,path,rate_func=linear,run_time=.1,
                            final_alpha_value=.25,remover=True))
    np.testing.assert_allclose(moving.get_center(),[-.5,0.,0.],atol=2e-6)
    assert moving not in scene.mobjects


def check_path_play_rate_has_no_redundant_probes():
    from manimlib import MoveAlongPath
    scene=Scene()
    moving=Line([-.1,0,0],[.1,0,0])
    path=Line([0,0,0],[2,0,0])
    samples=[]
    def rate(alpha):
        samples.append(alpha)
        return alpha*alpha
    scene.play(MoveAlongPath(moving,path),run_time=.5,rate_func=rate)
    assert samples
    assert all(any(abs(alpha-k/15)<1e-8 for k in range(16)) for alpha in samples), samples
    np.testing.assert_allclose(moving.get_center(),[2.,0.,0.],atol=2e-6)


def check_mixed_native_callback_succession():
    from manimlib import MoveAlongPath, Succession
    moving=Line([-.1,0,0],[.1,0,0])
    starts=[]
    class Authored(MoveAlongPath):
        def create_starting_mobject(self):
            starts.append(self.mobject.get_center().copy())
            return super().create_starting_mobject()
    from manimlib import _native
    scene=Scene()
    stock=MoveAlongPath(moving,Line([0,0,0],[1,0,0]),run_time=.1,rate_func=linear)
    authored=Authored(moving,Line([1,0,0],[2,1,0]),run_time=.1,rate_func=linear)
    assert not _native._requires_python_animation(stock)
    assert _native._requires_python_animation(authored)
    scene.play(Succession(stock,authored))
    assert len(starts)==1
    np.testing.assert_allclose(starts[0],[1.,0.,0.],atol=2e-6)
    np.testing.assert_allclose(moving.get_center(),[2.,1.,0.],atol=2e-6)


def check_path_failure_and_scene_reuse():
    from manimlib import MoveAlongPath
    error=ValueError("authored path failure")
    enabled=[False]
    class Path(Line):
        def point_from_proportion(self,t):
            if t>.2 and not enabled[0]:
                raise error
            return super().point_from_proportion(t)
    scene=Scene()
    moving=Line([-.1,0,0],[.1,0,0])
    path=Path([0,0,0],[1,0,0])
    motion=MoveAlongPath(moving,path,run_time=.2,rate_func=linear,suspend_mobject_updating=True)
    try:
        scene.play(AnimationGroup(motion))
    except ValueError as caught:
        assert caught is error
    else:
        raise AssertionError("authored path failure was ignored")
    assert not moving._is_updating_suspended()
    enabled[0]=True
    scene.play(motion)
    np.testing.assert_allclose(moving.get_center(),[1.,0.,0.],atol=2e-6)


def _read_y4m_luma(payload):
    header,offset=payload.split(b"\n",1)[0],payload.index(b"\n")+1
    if not header.startswith(b"YUV4MPEG2 "):
        raise ValueError("not a Y4M stream")
    fields={part[:1]:part[1:] for part in header.split()[1:]}
    width,height=int(fields[b"W"]),int(fields[b"H"])
    chroma=fields.get(b"C",b"420jpeg")
    if width<=0 or height<=0 or width%2 or height%2 or chroma not in (b"420",b"420jpeg",b"420mpeg2",b"420paldv"):
        raise ValueError("acceptance reader expects even-sized 8-bit 4:2:0")
    stride=width*height*3//2
    frames=[]
    while offset<len(payload):
        end=payload.index(b"\n",offset)
        if payload[offset:end].split()[0]!=b"FRAME":
            raise ValueError("missing frame marker")
        offset=end+1
        if len(payload)-offset<stride:
            raise ValueError("truncated frame")
        frames.append(np.frombuffer(payload[offset:offset+width*height],dtype=np.uint8).reshape(height,width).copy())
        offset+=stride
    return frames


def check_deformation_reaches_renderer():
    import tempfile
    from pathlib import Path
    from manimlib import Square
    from fmn_python import render_scene
    class Deformation(Scene):
        def construct(self):
            square=Square(side_length=1.,fill_opacity=1.,stroke_width=0).shift(np.array([-1.,0.,0.]))
            self.add(square)
            self.play(Homotopy(lambda x,y,z,t:(x+2*t,y,z),square,run_time=1.,rate_func=linear))
    directory=Path(tempfile.mkdtemp(prefix="fmn-movement-acceptance-"))
    result=render_scene(Deformation,directory/"deformation.y4m",resolution=(96,54),fps=4,threads=1)
    frames=_read_y4m_luma(result.destination.read_bytes())
    assert result.frame_count==len(frames)==4
    centers=[]
    for frame in frames:
        columns=np.nonzero(frame>128)[1]
        assert len(columns)>0, "the moving square was not rendered"
        centers.append(float(np.mean(columns+.5)))
    expected=[48.+(54./8.)*x for x in (-.5,0.,.5,1.)]
    np.testing.assert_allclose(centers,expected,atol=.8)
    print(f"movement renderer artifact retained: {result.destination}")


for _case in (check_homotopy_family_lag,check_start_hook_and_complex_map,
              check_time_span_and_reverse_interpolation,check_suspension_and_resume,
              check_nested_failure_and_recovery,check_scene_updater_failure_unwinds,
              check_phase_flow_replay_and_zero_time,check_flow_failure_recovery,
              check_authored_path_sampler,check_late_bound_path_and_move_hooks,
              check_path_remover_and_fractional_endpoint,check_path_play_rate_has_no_redundant_probes,
              check_mixed_native_callback_succession,check_path_failure_and_scene_reuse,
              check_deformation_reaches_renderer):
    _case()

# Moving along a native path still requires the receiver's authored lifecycle.
from pathlib import Path as _Path
import tempfile as _tempfile
import unittest as _unittest
from unittest.mock import patch as _patch
import manimlib as _m


class PathObjectProtocols(_unittest.TestCase):
    def motion(self, obj, path=None, **kwargs):
        return _m.MoveAlongPath(obj, _m.Line([0, 0, 0], [2, 0, 0]) if path is None else path,
                                run_time=.125, rate_func=_m.linear, **kwargs)

    @staticmethod
    def center(obj):
        points = obj.get_points()
        return .5 * (points.min(axis=0) + points.max(axis=0))

    def test_stock_path_motion_keeps_native_admission(self):
        for obj in (_m.Square(), _m.Dot(), _m.VGroup(_m.Dot(), _m.Square())):
            with self.subTest(obj=type(obj)):
                self.assertFalse(_m._requires_python_animation(self.motion(obj)))

    def test_authored_bounding_box_controls_placement_at_every_frame(self):
        class Offset(_m.Square):
            def get_bounding_box(self):
                return super().get_bounding_box() + _m.UP
        samples = []
        obj = Offset()
        obj.add_updater(lambda mob, dt: samples.append(self.center(mob))
                        if mob is obj and dt > 0 else None, call=False)
        animation = self.motion(obj)
        self.assertTrue(_m._requires_python_animation(animation))
        with _tempfile.TemporaryDirectory() as directory:
            scene = _m.Scene()
            with scene.render_session(_Path(directory) / 'bounds.y4m', resolution=(48, 32), fps=24):
                scene.play(animation)
        np.testing.assert_allclose(samples, [[2/3, -1, 0], [4/3, -1, 0], [2, -1, 0]], atol=3e-6)

    def test_starting_copy_updater_drives_visible_shape_each_frame(self):
        copied, elapsed, widths = [], [0.], []
        class Authored(_m.Square):
            def copy(self):
                result = super().copy()
                copied.append(result)
                def grow(snapshot, dt):
                    if dt > 0:
                        elapsed[0] += dt
                        self.set_width(1 + elapsed[0], stretch=True)
                result.add_updater(grow, call=False)
                return result
        obj = Authored()
        obj.add_updater(lambda mob, dt: widths.append(mob.get_width())
                        if mob is obj and dt > 0 else None, call=False)
        with _tempfile.TemporaryDirectory() as directory:
            scene = _m.Scene()
            with scene.render_session(_Path(directory) / 'growth.y4m', resolution=(48, 32), fps=24):
                scene.play(self.motion(obj))
        self.assertTrue(copied)
        np.testing.assert_allclose(widths, [1+1/24, 1+2/24, 1+3/24], atol=3e-6)
        np.testing.assert_allclose(self.center(obj), [2, 0, 0], atol=3e-6)

    def test_snapshot_creation_waits_for_nested_begin(self):
        seen = []
        class Authored(_m.Square):
            def copy(self):
                seen.append(self.get_width())
                return super().copy()
        obj = Authored()
        animation = self.motion(obj)
        class Prefix(_m.Animation):
            def finish(self):
                super().finish()
                obj.set_width(3, stretch=True)
        _m.Scene().play(_m.Succession(Prefix(_m.Mobject(), run_time=.125), animation))
        self.assertTrue(seen)
        self.assertAlmostEqual(seen[-1], 3, places=5)
        np.testing.assert_allclose(self.center(obj), [2, 0, 0], atol=3e-6)

    def test_copy_descriptor_is_not_called_to_choose_execution(self):
        reads = []
        class Authored(_m.Square):
            @property
            def copy(self):
                reads.append(self)
                return lambda: _m.Square.copy(self)
        obj = Authored()
        animation = self.motion(obj)
        self.assertTrue(_m._requires_python_animation(animation))
        self.assertEqual(reads, [])
        _m.Scene().play(animation)
        self.assertTrue(reads)

    def test_animation_operand_descriptors_are_not_read_at_admission(self):
        for name in ('mobject', 'path'):
            with self.subTest(name=name):
                reads = []
                class Authored(_m.MoveAlongPath):
                    pass
                animation = Authored(_m.Square(), _m.Line())
                def get(instance):
                    reads.append(name)
                    return instance.__dict__[name]
                setattr(Authored, name, property(get,
                    lambda instance, value: instance.__dict__.__setitem__(name, value)))
                self.assertTrue(_m._requires_python_animation(animation))
                self.assertEqual(reads, [])

    def test_starting_snapshot_update_override_uses_its_actual_receiver(self):
        calls = []
        class Authored(_m.Square):
            def update(self, *args, **kwargs):
                calls.append(self)
                return super().update(*args, **kwargs)
        obj = Authored()
        animation = self.motion(obj)
        self.assertTrue(_m._requires_python_animation(animation))
        self.assertEqual(calls, [])
        _m.Scene().play(animation)
        self.assertIn(animation.starting_mobject, calls)
        self.assertIsNot(animation.starting_mobject, obj)

    def test_authored_path_admission_rechecks_live_state_at_begin(self):
        healthy, calls = [True], []
        class Path(_m.Line):
            def has_points(self):
                calls.append(self)
                return healthy[0] and super().has_points()
        path = Path([0, 0, 0], [2, 0, 0])
        obj = _m.Square()
        animation = self.motion(obj, path)
        calls.clear()
        self.assertTrue(_m._requires_python_animation(animation))
        self.assertEqual(calls, [])
        healthy[0] = False
        scene = _m.Scene()
        with self.assertRaisesRegex(ValueError, 'nonempty'):
            scene.play(animation)
        healthy[0] = True
        scene.play(animation)
        np.testing.assert_allclose(self.center(obj), [2, 0, 0], atol=3e-6)

    def test_copy_failure_releases_suspension_and_allows_reuse(self):
        error, failing = RuntimeError('path snapshot failed'), [True]
        class Authored(_m.Square):
            def copy(self):
                if failing[0]:
                    raise error
                return super().copy()
        obj, scene = Authored(), _m.Scene()
        animation = self.motion(obj, suspend_mobject_updating=True)
        with self.assertRaises(RuntimeError) as caught:
            scene.play(animation)
        self.assertIs(caught.exception, error)
        self.assertFalse(obj._is_updating_suspended())
        failing[0] = False
        scene.play(animation)
        self.assertFalse(obj._is_updating_suspended())
        np.testing.assert_allclose(self.center(obj), [2, 0, 0], atol=3e-6)

    def test_late_instance_and_base_snapshot_changes_remain_observable(self):
        obj, calls = _m.Square(), []
        animation = self.motion(obj)
        self.assertFalse(_m._requires_python_animation(animation))
        def duplicate():
            calls.append(obj)
            return _m.Square.copy(obj)
        obj.copy = duplicate
        self.assertTrue(_m._requires_python_animation(animation))
        _m.Scene().play(animation)
        self.assertIn(obj, calls)
        original = _m.Mobject.copy
        def copy(self):
            calls.append(self)
            return original(self)
        with _patch.object(_m.Mobject, 'copy', copy):
            other = _m.Square()
            self.assertTrue(_m._requires_python_animation(self.motion(other)))
            _m.Scene().play(self.motion(other))
            self.assertIn(other, calls)

    def test_callback_placement_reads_a_path_animated_earlier_in_the_same_frame(self):
        class Offset(_m.Square):
            def get_bounding_box(self):
                return super().get_bounding_box() + _m.UP
        obj, path, samples = Offset(), _m.Line([0, 0, 0], [2, 0, 0]), []
        obj.add_updater(lambda mob, dt: samples.append(self.center(mob))
                        if mob is obj and dt > 0 else None, call=False)
        with _tempfile.TemporaryDirectory() as directory:
            scene = _m.Scene()
            with scene.render_session(_Path(directory) / 'live-path.y4m', resolution=(48, 32), fps=24):
                scene.add(path, obj)
                scene.play(path.animate.shift(_m.UP), self.motion(obj, path),
                           run_time=.125, rate_func=_m.linear)
        np.testing.assert_allclose(samples, [[2/3, -2/3, 0], [4/3, -1/3, 0], [2, 0, 0]], atol=3e-6)

    def test_exact_authored_path_easing_at_48fps(self):
        calls = []
        class Offset(_m.Square):
            def get_bounding_box(self):
                return super().get_bounding_box() + _m.UP
        def rate(alpha):
            calls.append(alpha)
            return alpha * alpha
        obj = Offset()
        with _tempfile.TemporaryDirectory() as directory:
            scene = _m.Scene()
            with scene.render_session(_Path(directory) / 'eased.y4m', resolution=(48, 32), fps=48):
                scene.play(_m.MoveAlongPath(obj, _m.Line([0, 0, 0], [2, 0, 0]),
                                             run_time=3/48, rate_func=rate))
        np.testing.assert_allclose(calls, [0, 1/3, 2/3, 1, 1], atol=1e-15)
        np.testing.assert_allclose(self.center(obj), [2, -1, 0], atol=3e-6)

    def test_rendered_placement_matches_independent_native_path_controls(self):
        class Offset(_m.Square):
            def get_bounding_box(self):
                return super().get_bounding_box() + _m.UP
        def render(path, kind, workers):
            scene = _m.Scene()
            with scene.render_session(path, resolution=(64, 40), fps=24, threads=workers):
                obj = (Offset() if kind == 'authored' else _m.Square()).scale(.4)
                y = -1 if kind == 'expected' else 0
                curve = _m.Line([0, y, 0], [2, y, 0])
                scene.add(obj)
                scene.play(self.motion(obj, curve))
            return path.read_bytes()
        with _tempfile.TemporaryDirectory() as directory:
            root = _Path(directory)
            expected = render(root/'expected.y4m', 'expected', 1)
            self.assertNotEqual(expected, render(root/'ignored.y4m', 'ignored', 1))
            for workers in (1, 4, 16):
                with self.subTest(workers=workers):
                    self.assertEqual(expected, render(root/f'{workers}.y4m', 'authored', workers))
            frames = expected.split(b'FRAME\n')[1:]
            self.assertEqual(len(frames), 3)
            self.assertEqual(len(set(frames)), 3)


_path_result = _unittest.TextTestRunner(verbosity=2).run(
    _unittest.defaultTestLoader.loadTestsFromTestCase(PathObjectProtocols))
if not _path_result.wasSuccessful():
    raise AssertionError('path object protocols failed')


class ProtocolTables(_unittest.TestCase):
    """fm-5wq.39: install-time protocol tables equal getattr_static's answer
    over the live namespace, without calling it."""

    NAMES = ("abort", "begin", "copy", "create_starting_mobject", "finish", "__getattr__",
             "__getattribute__", "get_center", "get_family", "interpolate", "interpolate_mobject",
             "interpolate_submobject", "match_points", "match_style", "move_to",
             "point_from_proportion", "rotate", "scale", "set_color", "shift", "suspend_updating",
             "update", "update_mobjects", "mro", "__call__", "no_such_protocol")

    def namespace(self):
        native = vars(getattr(_m, "_native", _m))
        meta = type(_m.Mobject)

        class Shadowing(meta):
            __dict__ = property(lambda cls: {})

        class Hidden(_m.VMobject, metaclass=Shadowing):
            def update(self, dt=0):
                return self

        class Wrapped(_m.VMobject):
            shift = staticmethod(lambda *args: None)
            scale = classmethod(lambda cls, *args: None)

        class AuthoredAnimation(_m.Animation):
            def interpolate_mobject(self, alpha):
                pass

        return dict(native, Hidden=Hidden, Wrapped=Wrapped, AuthoredAnimation=AuthoredAnimation)

    def test_tables_equal_getattr_static(self):
        from fmn_python import movement
        g = self.namespace()
        for root in (_m.Mobject, _m.VMobject, _m.Animation, _m.DecimalNumber):
            classes = {base for cls in tuple(g.values())
                       if isinstance(cls, type) and issubclass(cls, root)
                       for base in cls.__mro__ if issubclass(base, root)}
            expected = {cls: {name: movement._implementation(cls, name) for name in self.NAMES}
                        for cls in classes}
            table = movement._protocols(g, root, self.NAMES)
            self.assertEqual(set(table), set(expected), root)
            for cls, row in expected.items():
                for name, value in row.items():
                    self.assertIs(table[cls][name], value, (root.__name__, cls.__name__, name))
        # The shadowed metaclass hides Hidden's own update from getattr_static.
        own = type.__dict__["__dict__"].__get__(g["Hidden"])["update"]
        self.assertIsNot(movement._protocols(g, _m.VMobject, ("update",))[g["Hidden"]]["update"],
                         own)

    def test_static_lookup_equals_getattr_static(self):
        # fm-5wq.31: movement._getattr_static answers ordinary lookups from
        # namespaces and defers everything else to getattr_static.
        import inspect
        from fmn_python import movement

        class Data:
            def __get__(self, obj, owner=None):
                return 1

            def __set__(self, obj, value):
                pass

        class NonData:
            def __get__(self, obj, owner=None):
                return 2

        class Slotted:
            __slots__ = ("hook",)

        class Meta(type):
            meta_only = "meta"

        class WithMeta(metaclass=Meta):
            def hook(self):
                pass

        class Authored(_m.VMobject):
            prop = property(lambda self: 3)
            data_desc = Data()
            nondata = NonData()
            static = staticmethod(lambda: 0)
            klass = classmethod(lambda cls: 0)

            def shift(self, *args, **kwargs):
                return self

        g = self.namespace()
        authored = Authored()
        authored.__dict__.update(prop=5, data_desc=6, nondata=7, shift=8, extra=9)
        slotted = Slotted()
        slotted.hook = 1
        objects = [value for value in g.values() if isinstance(value, type)]
        objects += [Authored, authored, Slotted, slotted, WithMeta, WithMeta(), Meta,
                    _m.Square(), _m.VGroup(_m.Circle()), _m.Dot(), _m.DecimalNumber(1.5),
                    g["Hidden"], g["Hidden"](), g["Wrapped"](), _m.Scene]
        names = self.NAMES + ("prop", "data_desc", "nondata", "static", "klass", "extra",
                              "hook", "meta_only", "__dict__", "__class__", "__init__",
                              "submobjects", "points", "data")
        compared = 0
        for obj in objects:
            for name in names:
                self.assertIs(movement._getattr_static(obj, name),
                              inspect.getattr_static(obj, name, None), (obj, name))
                compared += 1
        self.assertGreater(compared, 5000)

    def test_tables_make_no_getattr_static_calls(self):
        import inspect
        from fmn_python import movement
        original, calls = inspect.getattr_static, []

        def counting(*args):
            calls.append(args[1])
            return original(*args)

        g = self.namespace()
        with _patch.object(inspect, "getattr_static", counting):
            movement._protocols(g, _m.Mobject, self.NAMES)
        self.assertEqual(calls, [])


_tables_result = _unittest.TextTestRunner(verbosity=2).run(
    _unittest.defaultTestLoader.loadTestsFromTestCase(ProtocolTables))
if not _tables_result.wasSuccessful():
    raise AssertionError('protocol tables failed')
