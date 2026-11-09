"""Real-native reflow, retained ownership and data-animation witnesses."""
from __future__ import annotations
import copy
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.table import TableMobject, TableDataAnimation


def points(obj):
    return [np.asarray(member.get_points()).copy() for member in obj.family_members_with_points()]


def equal_geometry(case, actual, expected):
    left, right = points(actual), points(expected)
    case.assertEqual(len(left), len(right))
    for a, b in zip(left, right):
        np.testing.assert_allclose(a, b, atol=4e-6)


class LiveTableTests(unittest.TestCase):
    def table(self):
        return TableMobject(['name', 'value'], [['a', '1'], ['b', '2']], font_size=36)

    def test_reflow_retains_cells_rules_updaters_annotations_and_clock(self):
        table=self.table();scene=m.Scene();scene.add(table)
        cell=table.get_cell(0,1);rule=table.get_rules()[0]
        cell.set_color(m.RED);rule.set_stroke(m.GREEN,7)
        note=m.Dot(cell.get_center()+m.UP*.2,radius=.02);cell.add(note)
        seen=[];cell.add_updater(lambda obj,dt:seen.append(dt),call=False)
        anchor=cell._table_anchor.get_center().copy();relative=note.get_center()-anchor
        upper=table._table_anchors[2].get_center().copy()
        root_note=m.Dot((3,2,0));table.add(root_note)
        table.set_cell(0,'value','12345')
        self.assertIs(table.get_cell(0,1),cell);self.assertIs(table.get_rules()[0],rule)
        self.assertIs(cell.submobjects[-1],note);self.assertIs(table.submobjects[-1],root_note)
        self.assertEqual(cell._table_content[0].get_color(),m.RED)
        self.assertEqual(rule.get_stroke_color(),m.GREEN)
        np.testing.assert_allclose(table._table_anchors[2].get_center(),upper,atol=2e-6)
        np.testing.assert_allclose(note.get_center()-cell._table_anchor.get_center(),relative,atol=2e-6)
        self.assertEqual(seen,[]);self.assertEqual(scene.get_time(),0)
        scene.wait(.125);self.assertTrue(seen)

    def test_live_csv_replacement_preserves_native_scalar_rules(self):
        table=self.table();cell=table.get_cell(0,1)
        table.set_csv('name;value\na;3.5\nb;\n',separator=';')
        self.assertEqual(table.values,(('a','3.5'),('b','')))
        self.assertIs(table.get_cell(0,1),cell)

    def test_grid_shape_edits_keep_positional_survivors_and_drop_removed_cells(self):
        table=self.table();old=table.get_cell(1,0);header=table.get_headers()[1]
        table.set_data([['a','1','x'],['b','2','y'],['c','3','z']],headers=['name','value','extra'])
        self.assertEqual(table.shape,(3,3));self.assertIs(table.get_cell(1,0),old)
        self.assertIs(table.get_headers()[1],header)
        self.assertEqual(len(table.get_rules()),6)
        table.set_data([['only']],headers=['one'])
        self.assertEqual(table.shape,(1,1));self.assertNotIn(old,table.get_family())
        self.assertEqual(len(table.get_rules()),2)

    def test_empty_cells_have_native_rule_centers_and_can_gain_or_lose_glyphs(self):
        table=TableMobject(['a','b'],[['',''],['','']])
        cell=table.get_cell(0,0);center=cell.get_center().copy()
        self.assertGreater(np.linalg.norm(center),.01)
        self.assertEqual(len(cell._table_content),0)
        table.set_cell(0,0,'abc');self.assertEqual(len(cell._table_content),3)
        table.set_cell(0,0,'');self.assertEqual(len(cell._table_content),0)
        self.assertIs(cell,table.get_cell(0,0))
        np.testing.assert_allclose(cell.get_center(),center,atol=2e-6)

    def test_reflow_commutes_with_shear_reflection_rotation_and_translation(self):
        # Independent path: native reflow while axis-aligned, THEN the authored
        # affine map. Compare to reflow performed on already placed geometry.
        transform=np.array([[-1,.4,.2],[.3,1.2,-.1],[.25,.5,1]])
        shift=np.array([2,-1,.5])
        actual=self.table();expected=self.table()
        actual.apply_matrix(transform,about_point=m.ORIGIN).shift(shift)
        actual.set_data([['wide column','10'],['z','']])
        expected.set_data([['wide column','10'],['z','']])
        expected.apply_matrix(transform,about_point=m.ORIGIN).shift(shift)
        equal_geometry(self,actual,expected)

    def test_refusals_preserve_geometry_and_allow_recovery(self):
        table=self.table();before=points(table);values=table.values
        for rows in ([['a']], [['x','数'],['y','2']], [['ok',1],['b','2']]):
            with self.assertRaises((ValueError,TypeError)):
                table.set_data(rows)
            self.assertEqual(table.values,values)
            for a,b in zip(points(table),before):np.testing.assert_array_equal(a,b)
        table.set_cell(0,0,'ready');self.assertEqual(table.values[0][0],'ready')

    def test_input_callback_edits_are_not_overwritten_and_reentry_is_refused(self):
        table=self.table()
        def edited():
            table.get_cell(0,0).shift((3,0,0))
            yield ['new','4'];yield ['b','2']
        with self.assertRaisesRegex(RuntimeError,'changed'):
            table.set_data(edited())
        self.assertEqual(table.values[0][0],'a')
        edited_center=table.get_cell(0,0).get_center().copy()
        def nested():
            table.set_cell(1,0,'nested')
            yield ['new','4'];yield ['b','2']
        with self.assertRaisesRegex(RuntimeError,'reenter'):
            table.set_data(nested())
        np.testing.assert_array_equal(table.get_cell(0,0).get_center(),edited_center)
        table.set_cell(0,0,'recovered')

    def test_callback_copies_and_checkpoints_remain_editable(self):
        table=self.table();scene=m.Scene();scene.add(table);saved=[]
        def rows():
            saved.extend((table.copy(),copy.deepcopy(table),pickle.loads(pickle.dumps(table))))
            saved.append(scene.get_state())
            yield ['new','10'];yield ['b','2']
        table.set_data(rows())
        for clone in saved[:3]:
            clone.set_cell(0,0,'clone')
            self.assertEqual(clone.values[0][0],'clone')
            self.assertIsNot(clone.get_cell(0,0),table.get_cell(0,0))
        scene.restore_state(saved[3]);table.set_cell(0,0,'restored')
        self.assertEqual(table.values[0][0],'restored')

    def test_equal_data_is_a_true_noop_and_degenerate_chart_refuses(self):
        table=self.table();glyph=table.get_cell(0,0)._table_content[0]
        view=glyph.get_points();table.set_data(table.values)
        self.assertIs(glyph,table.get_cell(0,0)._table_content[0]);np.testing.assert_array_equal(view,glyph.get_points())
        table.stretch(0,0)
        with self.assertRaisesRegex(ValueError,'nondegenerate'):
            table.set_cell(0,0,'new')

    def test_data_animation_uses_transform_path_rate_lag_and_exact_endpoint(self):
        table=self.table();cell=table.get_cell(0,1)
        anim=table.animate_data([['large','42'],['b','']],rate_func=m.linear,path_arc=.6,lag_ratio=.01)
        self.assertIsInstance(anim,TableDataAnimation);self.assertTrue(m._requires_python_animation(anim))
        anim.begin();start=points(table);anim.interpolate(.5)
        self.assertTrue(any(not np.array_equal(a,b) for a,b in zip(start,points(table))))
        self.assertEqual(table.values,(('a','1'),('b','2')))
        anim.finish()
        self.assertEqual(table.values,(('large','42'),('b','')))
        self.assertIs(cell,table.get_cell(0,1))
        self.assertEqual(len(table.get_cell(1,1)._table_content),0)
        self.assertFalse(vars(table).get('_is_animating',False))
        table.set_cell(0,0,'again')

    def test_returning_rate_and_cancel_restore_committed_table(self):
        table=self.table();initial=table.copy()
        anim=table.animate_data([['abc','100'],['d','3']],rate_func=m.there_and_back)
        anim.begin();anim.interpolate(.5);anim.finish()
        self.assertEqual(table.values,initial.values);equal_geometry(self,table,initial)
        anim=table.animate_data([['abc','100'],['d','3']],rate_func=m.linear)
        anim.begin();anim.interpolate(.4);anim.abort()
        self.assertEqual(table.values,initial.values);equal_geometry(self,table,initial)
        table.set_cell(0,1,'20')

    def test_partial_final_endpoint_is_not_published_as_completed_data(self):
        table=self.table();initial=table.values
        anim=table.animate_data([['abc','100'],['d','3']],rate_func=m.linear,final_alpha_value=.5)
        anim.begin();anim.finish()
        self.assertEqual(table.values,initial)
        with self.assertRaisesRegex(RuntimeError,'unfinished'):
            table.set_cell(0,0,'other')
        anim.final_alpha_value=1;anim.finish()
        self.assertEqual(table.values,(('abc','100'),('d','3')))
        table.set_cell(0,0,'other')

    def test_succession_resolves_targets_at_begin_and_preserves_cell_handles(self):
        table=self.table();scene=m.Scene();scene.add(table);cell=table.get_cell(0,0)
        first=table.animate_data([['first','10'],['b','2']],rate_func=m.linear)
        second=table.animate_data([['last','20'],['b','2']],rate_func=m.linear)
        scene.play(m.Succession(first,second),run_time=.5)
        self.assertEqual(table.values[0][0],'last');self.assertIs(table.get_cell(0,0),cell)
        self.assertGreater(scene.get_time(),0)
        with self.assertRaises(ValueError):table.animate_data([['only']],headers=['a'])
        with self.assertRaises(m._CapabilityError):table.animate.set_cell(0,0,'x')

    def test_value_tracker_can_drive_discrete_live_table_updates(self):
        table=self.table();scene=m.Scene();tracker=m.ValueTracker(0);scene.add(table)
        table.add_updater(lambda obj:obj.set_cell(0,1,str(round(tracker.get_value()))),call=False)
        scene.play(tracker.animate.set_value(8),run_time=.25,rate_func=m.linear)
        self.assertEqual(table.values[0][1],'8')

    def test_native_table_render_matches_raw_native_grid_at_one_four_threads(self):
        class Actual(m.Scene):
            def construct(self):
                table=TableMobject(['x'],[['1']],font_size=48)
                self.add(table);self.wait(.25)
        class Expected(m.Scene):
            def construct(self):
                raw=m.VMobject();specs=m._build_table(raw,m._native_shell_factory,['x'],[['1']])
                m._hang_native_children(raw,specs)
                raw[0].set_stroke(m.GREY_B);raw[1].set_stroke(m.GREY_B)
                raw[2].set_color(m.BLUE_B);raw[3].set_color(m.WHITE)
                raw.shift(-raw[0].get_center());self.add(raw);self.wait(.25)
        with tempfile.TemporaryDirectory() as directory:
            outputs=[]
            for threads in (1,4):
                pair=[]
                for cls in (Actual,Expected):
                    path=Path(directory)/f'{cls.__name__}-{threads}.y4m'
                    receipt=cls().render(path,format='y4m',resolution=(96,54),fps=8,threads=threads)
                    self.assertEqual(receipt.frame_count,2);pair.append(path.read_bytes())
                self.assertEqual(*pair);outputs.append(pair[0])
            self.assertEqual(*outputs)

if __name__=='__main__':unittest.main(verbosity=2)
