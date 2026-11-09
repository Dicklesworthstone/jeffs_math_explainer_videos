"""Real native SVG paint preparation and publication boundary."""
import unittest
import numpy as np
import manimlib as m

NESTED = '<svg><path fill="red" fill-rule="evenodd" d="M0 0H10V10H0Z M2 2H8V8H2Z"/></svg>'
DASHED = '<svg><path fill="none" stroke="red" stroke-dasharray="2 2" d="M0 0H10"/></svg>'


def build(source, **options):
    mob=m.VMobject()
    specs=m._build_svg_paints(mob,m._native_shell_factory,source,options)
    m._hang_native_children(mob,specs)
    return mob


class NativeSvgPaintTests(unittest.TestCase):
    def test_evenodd_fill_is_not_silently_treated_as_nonzero(self):
        evenodd=build(NESTED);nonzero=build(NESTED.replace('evenodd','nonzero'))
        self.assertFalse(evenodd.looks_identical(nonzero))
        self.assertEqual(len(evenodd),1)
        self.assertEqual(len(evenodd[0]),2)
        self.assertEqual(evenodd[0][0].get_stroke_opacity(),0)

    def test_dash_geometry_follows_native_path_distance(self):
        dashed=build(DASHED)
        self.assertEqual(len(dashed),1)
        self.assertEqual(len(dashed[0]),4)
        for piece,(start,end) in zip(dashed[0].submobjects[1:],[(0,2),(4,6),(8,10)]):
            np.testing.assert_allclose(piece.get_start(),[start,0,0],atol=1e-6)
            np.testing.assert_allclose(piece.get_end(),[end,0,0],atol=1e-6)
            self.assertEqual(piece.get_fill_opacity(),0)

    def test_overrides_do_not_fill_dash_segments(self):
        source=DASHED.replace('fill="none"','fill="blue"').replace('M0 0H10','M0 0H10V10H0Z')
        mob=build(source,fill_opacity=.4,stroke_opacity=.7)
        self.assertAlmostEqual(mob[0][0].get_fill_opacity(),.4,places=6)
        self.assertTrue(all(p.get_fill_opacity()==0 for p in mob[0].submobjects[1:]))
        self.assertTrue(all(abs(p.get_stroke_opacity()-.7)<1e-6 for p in mob[0].submobjects[1:]))

    def test_parse_budget_and_override_failures_do_not_mutate_destination(self):
        mob=m.Square();before=mob.copy();factory_calls=[]
        def factory(*args):
            factory_calls.append(args)
            return m._native_shell_factory(*args)
        for source,options in [('<svg><script/></svg>',{}),(DASHED,{'fill_opacity':float('nan')}),
                               (DASHED,{'stroke_width':-1}), (DASHED,{'unknown':1}),
                               (DASHED.replace('2 2','1e-300 1e-300'),{})]:
            with self.subTest(source=source,options=options),self.assertRaises((ValueError,TypeError)):
                m._build_svg_paints(mob,factory,source,options)
            self.assertTrue(mob.looks_identical(before));self.assertEqual(factory_calls,[])

    def test_solid_legacy_shape_is_unchanged(self):
        source='<svg><rect x="2" y="3" width="4" height="5" fill="red" stroke="blue"/></svg>'
        actual=build(source)
        old=m.VMobject()
        specs=old._build_svg_mobject(m._native_shell_factory,"",source)
        m._hang_native_children(old,specs)
        self.assertTrue(actual.looks_identical(old))

if __name__=='__main__':unittest.main(verbosity=2)
