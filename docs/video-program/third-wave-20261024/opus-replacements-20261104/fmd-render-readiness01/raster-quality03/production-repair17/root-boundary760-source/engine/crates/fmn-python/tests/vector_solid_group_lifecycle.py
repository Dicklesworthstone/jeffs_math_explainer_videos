"""Real native compound vector solids, authored roots and pixel witnesses."""
import copy
import importlib
from pathlib import Path
import pickle
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m

three = importlib.import_module('manimlib.mobject.three_dimensions')


def native_vector(kind, source=None, **options):
    """Direct Atlas builders, independent of the new constructor adapters."""
    obj = m._native_shell_factory()
    fill = options.get('fill_color', m.BLUE_D if kind in ('cube', 'prism') else m.BLUE_E)
    opacity = options.get('fill_opacity', 1.)
    stroke = options.get('stroke_color', m.BLUE_E)
    width = options.get('stroke_width', 0. if kind in ('cube', 'prism') else 1.)
    if kind == 'cube':
        specs = obj._build_vcube(m._native_shell_factory, options.get('side_length', 2.), fill, opacity, width, 0)
    elif kind == 'prism':
        specs = obj._build_vprism(m._native_shell_factory, options.get('width', 3.), options.get('height', 2.),
            options.get('depth', 1.), fill, opacity, width, 0)
    elif kind == 'dodeca':
        specs = obj._build_dodecahedron(m._native_shell_factory, fill, opacity, stroke, width, (.2,.2,.2), 0)
    else:
        builder = obj._build_prismify_family if source.submobjects else obj._build_prismify
        specs = builder(m._native_shell_factory, source, options.get('depth', 1.), options.get('direction', m.IN))
    m._hang_native_children(obj, specs)
    if kind == 'extrusion':
        sources = source.family_members_with_points() if source.submobjects else [source]*len(obj)
        for part, src in zip(obj, sources):
            part.match_style(src)
    else:
        style = dict(fill_color=fill, fill_opacity=opacity, stroke_width=width)
        if kind == 'dodeca' or 'stroke_color' in options:
            style['stroke_color'] = stroke
        m._apply_vmobject_style_kwargs(obj, style)
    m._apply_vgroup3d_config(obj, True, (.2,.2,.2), 'no_joint')
    return obj


def render(path, obj, threads):
    # A literal common pivot keeps independent builders on the same transform
    # recipe; rounded family bounding boxes need not yield identical centers.
    obj.rotate(.3, axis=m.RIGHT, about_point=m.ORIGIN).rotate(.2, axis=m.UP, about_point=m.ORIGIN)
    scene = m.Scene()
    with scene.render_session(path, format='png_sequence', resolution=(112,72), fps=4, threads=threads):
        scene.add(obj)
        scene.play(obj.animate.shift(.5*m.RIGHT), run_time=.5, rate_func=m.linear)
    return [file.read_bytes() for file in sorted(path.glob('*.png'))]


class VectorSolidLifecycleTests(unittest.TestCase):
    def construct(self, cls, **kwargs):
        return cls(m.Square(side_length=.6, fill_opacity=1), **kwargs) if issubclass(cls,m.Prismify) else cls(**kwargs)

    def test_each_constructor_runs_all_hooks_once_and_preserves_class_identity(self):
        for base in (m.VCube,m.VPrism,m.Dodecahedron,m.Prismify):
            calls=[]
            class Authored(base):
                def init_data(self): calls.append('data');super().init_data()
                def init_points(self): calls.append('points');super().init_points()
                def init_uniforms(self): calls.append('uniforms');super().init_uniforms()
                def init_colors(self): calls.append('colors');super().init_colors()
            with self.subTest(base=base):
                obj=self.construct(Authored)
                self.assertEqual(calls,['data','points','uniforms','colors'])
                self.assertIsInstance(obj,m.VGroup3D)
                self.assertIs(getattr(three,base.__name__),base)
        self.assertIs(m.VCube.__bases__[0],m.VGroup3D)
        self.assertIs(m.VPrism.__bases__[0],m.VCube)

    def test_custom_records_geometry_and_hook_children_survive_animation(self):
        for base,count in ((m.VCube,6),(m.VPrism,6),(m.Dodecahedron,12),(m.Prismify,6)):
            class Authored(base):
                data_dtype=m.VMobject.data_dtype+[('mass',1)]
                def init_points(self):
                    self.set_points_as_corners([m.LEFT,m.UP,m.RIGHT])
                    self.data['mass'][:]=9
                    self.decoration=m.Dot().shift(2*m.UP)
                    self.add(self.decoration)
                def init_colors(self):
                    super().init_colors()
                    self.set_fill(m.GREEN,opacity=.5,recurse=False)
            with self.subTest(base=base):
                obj=self.construct(Authored)
                self.assertIn('mass',obj.data.dtype.names)
                self.assertEqual(obj.n_records(),5)
                np.testing.assert_array_equal(obj.data['mass'],9)
                self.assertIs(obj[0],obj.decoration)
                self.assertEqual(len(obj),count+1)
                self.assertEqual(obj.get_fill_color(),m.GREEN)
                old=obj.get_points().copy()
                scene=m.Scene();scene.add(obj)
                scene.play(obj.animate.shift(m.RIGHT),run_time=1/30,rate_func=m.linear)
                np.testing.assert_allclose(obj.get_points(),old+m.RIGHT)
                np.testing.assert_array_equal(obj.data['mass'],9)
                self.assertIs(obj[0],obj.decoration)

    def test_cube_uses_public_square_and_face_helper_once(self):
        original=three.square_to_cube_faces;made=[];calls=[]
        class Face(m.Square):
            def init_points(self):
                calls.append('square')
                super().init_points()
                self.stretch(1.25,0)
        def factory(face):
            calls.append('factory')
            for part in original(face):
                made.append(part.shift(.25*m.UP).set_fill(m.RED))
                yield made[-1]
        with patch.object(three,'Square',Face,create=True),patch.object(three,'square_to_cube_faces',factory):
            obj=m.VCube(side_length=.8)
        self.assertEqual(calls,['square','factory'])
        self.assertEqual(list(obj),made)
        self.assertTrue(all(isinstance(p,Face) and p.get_fill_color()==m.RED for p in obj))
        self.assertAlmostEqual(obj[0].get_width(),1.,places=6)

    def test_dodecahedron_exposes_polygon_faces_through_copy_and_animation(self):
        obj=m.Dodecahedron()
        self.assertEqual(len(obj),12)
        for face in obj:
            self.assertIsInstance(face,m.Polygon)
            self.assertEqual(face.get_vertices().shape,(5,3))
            np.testing.assert_array_equal(face.get_vertices(),face.get_points()[::2][:-1])
        copied=obj.copy()
        self.assertTrue(all(isinstance(face,m.Polygon) for face in copied))
        self.assertTrue(all(a is not b for a,b in zip(obj,copied)))
        vertices=obj[0].get_vertices().copy()
        scene=m.Scene();scene.add(obj)
        scene.play(obj.animate.shift(m.RIGHT),run_time=1/30,rate_func=m.linear)
        np.testing.assert_allclose(obj[0].get_vertices(),vertices+m.RIGHT,atol=1e-6)
        np.testing.assert_array_equal(copied[0].get_vertices(),vertices)

    def test_dodecahedron_uses_public_polygon_factory_and_preserves_authored_faces(self):
        made=[]
        class Face(m.Polygon):
            def init_points(self):
                super().init_points()
                self.shift(.25*m.UP)
                made.append(self)
            def init_colors(self):
                super().init_colors()
                self.set_fill(m.RED)
        expected=native_vector('dodeca').shift(.25*m.UP)
        with patch.object(three,'Polygon',Face,create=True):
            obj=m.Dodecahedron()
        self.assertEqual(list(obj),made)
        self.assertEqual(len(made),12)
        for face,native in zip(obj,expected):
            self.assertIsInstance(face,Face)
            self.assertEqual(face.get_fill_color(),m.RED)
            np.testing.assert_allclose(face.get_points(),native.get_points(),atol=1e-6)
        foreign=m.Polygon(m.LEFT,m.UP,m.RIGHT)
        scene=m.Scene();scene.add(foreign)
        before=foreign.get_points().copy()
        with patch.object(three,'Polygon',lambda *args,**kwargs:foreign,create=True):
            with self.assertRaisesRegex(ValueError,'duplicate|detached'):
                m.Dodecahedron()
        np.testing.assert_array_equal(foreign.get_points(),before)

    def test_prism_rescaling_is_virtual_and_accepts_cube_recipe_options(self):
        class Authored(m.VPrism):
            def init_data(self):self.calls=[];super().init_data()
            def rescale_to_fit(self,length,dim,stretch=False,**kwargs):
                self.calls.append((length,dim,stretch))
                return super().rescale_to_fit(length,dim,stretch=stretch,**kwargs)
        obj=Authored(width=4,height=3,depth=2,side_length=.7)
        self.assertEqual(obj.calls,[(4.,0,True),(3.,1,True),(2.,2,True)])
        np.testing.assert_allclose([obj.get_width(),obj.get_height(),obj.get_depth()],[4,3,2],atol=1e-6)

    def test_stock_geometry_and_paint_agree_with_independent_builders(self):
        for cls,kind,options in ((m.VCube,'cube',dict(side_length=.8)),
                                (m.VPrism,'prism',dict(width=1.2,height=.7,depth=.5)),
                                (m.Dodecahedron,'dodeca',dict(fill_color=m.RED,fill_opacity=.4,stroke_width=2))):
            with self.subTest(kind=kind):
                obj,expected=cls(**options),native_vector(kind,**options)
                self.assertEqual(len(obj),len(expected))
                for a,b in zip(obj,expected):
                    for key in ('point','fill_rgba','stroke_rgba','stroke_width'):
                        np.testing.assert_allclose(a.data[key],b.data[key],rtol=0,atol=1e-12)

    def test_explicit_styles_and_group_uniforms_reach_all_faces(self):
        for cls in (m.VCube,m.VPrism,m.Dodecahedron):
            with self.subTest(cls=cls):
                obj=cls(fill_color=m.RED,stroke_color=m.BLUE,fill_opacity=.4,stroke_width=1.5,
                        depth_test=False,shading=(.1,.2,.3),joint_type='miter',z_index=7)
                self.assertEqual(obj.z_index,7)
                for face in obj:
                    self.assertEqual(face.get_fill_color(),m.RED)
                    self.assertEqual(face.get_stroke_color(),m.BLUE)
                    self.assertAlmostEqual(face.get_fill_opacity(),.4,places=6)
                    self.assertAlmostEqual(face.get_stroke_width(),1.5)
                    self.assertFalse(face.uniforms['depth_test'])
                    self.assertEqual(face.get_joint_type(),3)
                    np.testing.assert_allclose(face.get_shading(),(.1,.2,.3))

    def test_native_painter_order_observes_constructor_z_index(self):
        for cls in (m.VCube,m.VPrism,m.Dodecahedron):
            with self.subTest(cls=cls):
                back,front=cls(z_index=-1),cls(z_index=1)
                scene=m.Scene();scene.add(front,back)
                self.assertEqual(scene.get_mobjects(),[back,front])

    def test_explicit_channels_beat_the_color_shorthand_which_beats_defaults(self):
        # fm-qead (BN-07 C-19): explicit fill_color/stroke_color > color= >
        # constructor default. The Reference builds these faces through
        # VMobject's `fill_color or color`, so explicit channels win there
        # too; before fm-qead the portal let color= override them.
        for cls in (m.VCube,m.VPrism,m.Dodecahedron):
            obj=cls(color=m.GREEN,fill_color=m.RED,stroke_color=m.BLUE)
            self.assertTrue(all(p.get_fill_color()==m.RED and p.get_stroke_color()==m.BLUE for p in obj))
            obj=cls(color=m.GREEN)
            self.assertTrue(all(p.get_fill_color()==m.GREEN and p.get_stroke_color()==m.GREEN for p in obj))

    def test_point_less_roots_take_vmobject_defaults_under_constructor_keywords(self):
        # fm-78b6: the Reference's VGroup3D runs VMobject.__init__ before it
        # adds the faces, so an unset root channel is VMobject's default
        # (GREY_C fill at opacity 0, GREY_A stroke at width 4). Values
        # measured on the pinned Reference.
        def root(obj):
            return (obj.get_fill_color(),obj.get_fill_opacity(),obj.get_stroke_color(),
                    obj.get_stroke_width(),obj.get_stroke_opacity())
        cases=((m.VCube(),('#29ABCA',1.,'#DDDDDD',0.,1.)),
               (m.VPrism(),('#29ABCA',1.,'#DDDDDD',0.,1.)),
               (m.Dodecahedron(),('#1C758A',1.,'#1C758A',1.,1.)),
               (m.Prismify(m.Square()),('#888888',0.,'#DDDDDD',4.,1.)),
               (m.Prismify(m.Square(),fill_color=m.RED,fill_opacity=.5),('#FC6255',.5,'#DDDDDD',4.,1.)))
        for obj,expected in cases:
            with self.subTest(cls=type(obj).__name__):
                self.assertEqual(root(obj),expected)

    def test_prismify_uses_live_source_geometry_and_keeps_each_source_paint(self):
        left=m.Square(side_length=.5,fill_color=m.RED,fill_opacity=.3).shift(m.LEFT)
        right=m.Triangle(fill_color=m.BLUE,fill_opacity=.7).shift(m.RIGHT)
        sources=m.VGroup(left,right);scene=m.Scene();scene.add(sources)
        before=[p.get_points().copy() for p in (left,right)]
        class Authored(m.Prismify):
            def init_points(self):
                self.hook_called=True
                self.decoration=m.Dot().shift(3*m.UP);self.add(self.decoration)
        obj=Authored(sources,depth=.4,direction=m.OUT,fill_color=m.GREEN)
        self.assertTrue(obj.hook_called)
        self.assertIs(obj[0],obj.decoration)
        self.assertEqual(len(obj),3)
        for group,source,old in zip(list(obj)[1:],(left,right),before):
            self.assertTrue(all(p.get_fill_color()==source.get_fill_color() for p in group))
            np.testing.assert_array_equal(group[0].get_points(),old)
            np.testing.assert_array_equal(source.get_points(),old)
            np.testing.assert_allclose(group[-1].get_center()-source.get_center(),[0,0,.4],atol=1e-6)

    def test_copy_deepcopy_pickle_do_not_rerun_group_hooks(self):
        calls=[]
        class Authored(m.Dodecahedron):
            def init_points(self):calls.append('points');self.add(m.Dot())
        obj=Authored()
        for copier in (lambda x:x.copy(),copy.deepcopy):
            other=copier(obj)
            self.assertEqual(len(other),13)
            self.assertTrue(all(a is not b for a,b in zip(obj,other)))
        self.assertEqual(calls,['points'])
        for cls in (m.VCube,m.VPrism,m.Dodecahedron):
            obj=cls();other=pickle.loads(pickle.dumps(obj))
            np.testing.assert_array_equal(obj[0].get_points(),other[0].get_points())
            other.shift(m.RIGHT)
            self.assertFalse(np.array_equal(obj[0].get_points(),other[0].get_points()))

    def test_native_face_preparation_failure_does_not_run_group_hooks(self):
        calls=[];error=RuntimeError('face generation')
        class Authored(m.Dodecahedron):
            def init_data(self):calls.append('data');super().init_data()
        def fail(*args,**kwargs):raise error
        with patch.object(m._native,'_native_shell_factory',fail):
            with self.assertRaises(RuntimeError) as caught:Authored()
        self.assertIs(caught.exception,error)
        self.assertEqual(calls,[])
        self.assertEqual(len(m.Dodecahedron()),12)

    def test_hook_failure_is_not_retried_and_reentry_refuses(self):
        calls=[];error=RuntimeError('group points')
        for base in (m.VCube,m.VPrism,m.Dodecahedron,m.Prismify):
            class Authored(base):
                def init_points(self):calls.append('points');raise error
                def init_uniforms(self):calls.append('uniforms')
            with self.subTest(base=base),self.assertRaises(RuntimeError) as caught:
                self.construct(Authored)
            self.assertIs(caught.exception,error)
        self.assertEqual(calls,['points']*4)
        class Reentrant(m.VCube):
            def init_points(self):m.VCube.__init__(self)
        with self.assertRaisesRegex(RuntimeError,'progress'):Reentrant()
        self.assertEqual(len(m.VCube()),6)

    def test_foreign_face_and_duplicate_roots_refuse_before_mutation(self):
        foreign=m.Square();scene=m.Scene();scene.add(foreign);before=foreign.get_points().copy()
        with patch.object(three,'Square',lambda **kw:foreign,create=True):
            with self.assertRaisesRegex(ValueError,'detached'):m.VCube()
        np.testing.assert_array_equal(foreign.get_points(),before)
        fresh=m.Square()
        with patch.object(three,'square_to_cube_faces',lambda _: [fresh,fresh]):
            with self.assertRaisesRegex(ValueError,'duplicate'):m.VCube()

    def test_shared_descendants_are_not_duplicated_or_rejected(self):
        shared=m.Dot();a=m.Square();b=m.Square().shift(m.RIGHT);a.add(shared);b.add(shared)
        with patch.object(three,'square_to_cube_faces',lambda _: [a,b]):obj=m.VCube()
        self.assertIs(obj[0][0],obj[1][0])
        copied=obj.copy()
        self.assertIs(copied[0][0],copied[1][0])
        self.assertIsNot(copied[0][0],shared)

    def test_invalid_inputs_precede_hooks_and_scene_bound_rebuild_is_refused(self):
        calls=[]
        class Authored(m.VCube):
            def init_data(self):calls.append('data');super().init_data()
        for options in ({'side_length':float('nan')},{'fill_opacity':float('inf')},
                        {'shading':(0,0)},{'joint_type':'unknown'},{'z_index':1<<35},{'bogus':True}):
            with self.subTest(options=options),self.assertRaises((ValueError,TypeError,KeyError,NotImplementedError)):
                Authored(**options)
        self.assertEqual(calls,[])
        for cls in (m.VCube,m.VPrism,m.Dodecahedron):
            obj=cls();scene=m.Scene();scene.add(obj);previous=list(obj)
            with self.assertRaisesRegex(RuntimeError,'detached'):cls.__init__(obj)
            self.assertEqual(list(obj),previous)

    def test_factory_pixels_match_independent_native_geometry_and_threads(self):
        root=Path(tempfile.mkdtemp(prefix='fmn-vector-groups-'))
        original=three.square_to_cube_faces
        def factory(face):return [part.shift(.3*m.UP) for part in original(face)]
        def make():
            with patch.object(three,'square_to_cube_faces',factory):return m.VCube(side_length=.8,fill_opacity=.6)
        frames=render(root/'one',make(),1)
        self.assertEqual(frames,render(root/'four',make(),4))
        self.assertEqual(frames,render(root/'expected',native_vector('cube',side_length=.8,fill_opacity=.6).shift(.3*m.UP),1))
        self.assertEqual(len(frames),2);self.assertNotEqual(*frames)

    def test_dodeca_and_extrusion_hook_pixels_match_independent_geometry(self):
        root=Path(tempfile.mkdtemp(prefix='fmn-vector-compositions-'))
        source=m.Square(side_length=.7,fill_color=m.BLUE,fill_opacity=.6,stroke_width=0)
        for cls,kind in ((m.Dodecahedron,'dodeca'),(m.Prismify,'extrusion')):
            class Authored(cls):
                def init_points(self):
                    self.decoration=m.Dot(color=m.RED).shift(2*m.UP)
                    self.add(self.decoration)
                def init_colors(self):
                    super().init_colors()
                    self.decoration.set_color(m.RED).set_fill(opacity=1,border_width=0).set_stroke(width=0)
            def make():return Authored(source) if kind=='extrusion' else Authored()
            expected=native_vector(kind,source=source)
            expected.set_submobjects([m.Dot(color=m.RED).shift(2*m.UP),*expected.submobjects])
            m._apply_vgroup3d_config(expected,True,(.2,.2,.2),'no_joint')
            frames=render(root/(kind+'-one'),make(),1)
            self.assertEqual(frames,render(root/(kind+'-four'),make(),4))
            self.assertEqual(frames,render(root/(kind+'-expected'),expected,1))
            self.assertEqual(len(frames),2);self.assertNotEqual(*frames)


if __name__=='__main__':
    unittest.main()
