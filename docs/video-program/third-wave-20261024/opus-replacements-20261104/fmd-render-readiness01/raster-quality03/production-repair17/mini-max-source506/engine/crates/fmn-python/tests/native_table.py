"""Real-native CSV/table acceptance; no parser, glyph or renderer doubles."""
from __future__ import annotations
import copy
import itertools
import pickle
import unittest
import numpy as np
import manimlib as m
from fmn_python.table import TableMobject

CSV = 'region,score,active\nwest,3.5,true\neast,,false\nnorth,7,true\n'

class NativeTableTests(unittest.TestCase):
    def test_csv_uses_native_scalar_rules_and_column_order(self):
        table = TableMobject.from_csv(CSV)
        self.assertEqual(table.headers, ('region','score','active'))
        self.assertEqual(table.values, (('west','3.5','true'),('east','','false'),('north','7','true')))
        self.assertEqual(table.shape, (3,3))
        self.assertEqual(len(table.get_rules()), 6)
        self.assertIs(table.get_columns()[1][2], table.get_cell(2,'score'))
        self.assertIs(table.get_rows()[-1][1], table.get_cell(-1,-2))

    def test_custom_separator_nulls_and_quoted_input_refusal(self):
        table = TableMobject.from_csv('b;a\nhello;2\n;3\n', separator=';')
        self.assertEqual(table.headers, ('b','a'))
        self.assertEqual(table.values, (('hello','2'),('','3')))
        # Negative control: the unguarded native convenience reader produced
        # ('"hello', 'world"'), silently dropping the real second column.
        with self.assertRaisesRegex(ValueError, 'quoted CSV'):
            TableMobject.from_csv('b;a\n"hello;world";2\n;3\n', separator=';')
        literal = TableMobject(['b','a'], [['hello;world','2'],['','3']])
        self.assertEqual(literal.values, (('hello;world','2'),('','3')))

    def test_grid_is_native_ruled_layout_not_a_second_host_layout(self):
        table = TableMobject(['α','Ω'], [['β',''],['x','42']], font_size=48)
        native = m.VMobject()
        specs = m._build_table(native,m._native_shell_factory,['α','Ω'],[['β',''],['x','42']])
        m._hang_native_children(native,specs)
        shift = -native[0].get_center()
        native.shift(shift)
        for actual,expected in zip(table._table_cells,native.submobjects[4:]):
            for a,b in zip(actual._table_content.family_members_with_points(),expected.family_members_with_points()):
                np.testing.assert_allclose(a.get_points(),b.get_points(),atol=2e-6)
        self.assertEqual(len(table.get_cell(0,1)._table_content),0)

    def test_cells_are_live_and_can_animate_on_one_scene_clock(self):
        table = TableMobject(['name','value'],[['one','1'],['two','2']])
        scene=m.Scene();scene.add(table)
        cell=table.get_cell(0,'value');before=cell.get_center().copy()
        scene.play(cell.animate.shift((1,0,0)),run_time=.125,rate_func=m.linear)
        np.testing.assert_allclose(cell.get_center(),before+(1,0,0),atol=2e-6)
        self.assertIs(table.get_cell(0,1),cell)
        self.assertGreater(scene.get_time(),0)

    def test_copy_pickle_and_checkpoint_remap_cell_families(self):
        table=TableMobject(['x'],[['1'],['2']])
        for clone in (table.copy(),copy.deepcopy(table),pickle.loads(pickle.dumps(table))):
            self.assertEqual(clone.values,table.values)
            self.assertIsNot(clone.get_cell(0,0),table.get_cell(0,0))
            clone.get_cell(0,0).shift((2,0,0))
            self.assertGreater(np.linalg.norm(clone.get_cell(0,0).get_center()-table.get_cell(0,0).get_center()),1)
        scene=m.Scene();scene.add(table);state=scene.get_state()
        table.shift((3,0,0));scene.restore_state(state)
        self.assertEqual(table.values,(('1',),('2',)))

    def test_bounded_grid_and_csv_admission(self):
        for headers,rows in [([], [['x']]),(['x'],[]),(['x'],[['a','b']]),(['a','b'],[['x']])]:
            with self.assertRaises(ValueError):TableMobject(headers,rows)
        with self.assertRaises(TypeError):TableMobject(['x'],[[2]])
        with self.assertRaises(ValueError):TableMobject(['x'],itertools.repeat(['x']))
        with self.assertRaises(ValueError):TableMobject(['x'],[['a'*262145]])
        with self.assertRaises(ValueError):TableMobject.from_csv('a'*262145)
        with self.assertRaises(ValueError):m._table_from_csv('x\n1\n', '\n')
        for rows in ([['x','y']], [[]], []):
            with self.assertRaises(ValueError):m._build_table(m.VMobject(),m._native_shell_factory,['x'],rows)

    def test_duplicate_headers_are_addressable_only_by_index(self):
        table=TableMobject(['x','x'],[['a','b']])
        self.assertIsNot(table.get_cell(0,0),table.get_cell(0,1))
        with self.assertRaises(KeyError):table.get_cell(0,'x')
        with self.assertRaises(IndexError):table.get_cell(2,0)
        with self.assertRaises(IndexError):table.get_cell(0,-3)

class NativeCsvAdmissionTests(unittest.TestCase):
    def test_native_boundary_refuses_quoted_and_ragged_rows_without_data_loss(self):
        for text, error in [('a,b\n"x,y",2\n', 'quoted CSV'),
                            ('a,b\nx,y,z\n', 'field count'),
                            ('a,b\nx\n', 'field count')]:
            with self.subTest(text=text), self.assertRaisesRegex(ValueError,error):
                m._table_from_csv(text)

if __name__=='__main__':unittest.main(verbosity=2)
