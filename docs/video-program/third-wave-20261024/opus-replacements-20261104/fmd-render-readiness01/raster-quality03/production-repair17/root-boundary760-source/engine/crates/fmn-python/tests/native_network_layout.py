"""Exercise Atlas layouts through the actual native boundary, with no doubles."""
import io
import unittest
import numpy as np
import manimlib as m


class NativeNetworkTests(unittest.TestCase):
    def test_circular_positions_and_inferred_node_order(self):
        names, edges, points = m._network_graph_layout(['z'], [('a','b'),('c','a')], 'circular')
        self.assertEqual(names, ['z','a','b','c'])
        self.assertEqual(edges, [('a','b'),('c','a')])
        np.testing.assert_allclose(points, [(1,0,0),(0,1,0),(-1,0,0),(0,-1,0)], atol=1e-12)

    def test_duplicates_and_loops_keep_first_appearance(self):
        names, edges, _ = m._network_graph_layout(['b','b'], [('b','a'),('a','b'),('b','b')], 'circular')
        self.assertEqual(names,['b','a'])
        self.assertEqual(edges,[('b','a'),('b','b')])

    def test_bfs_covers_disconnected_nodes(self):
        names, _, points = m._network_graph_layout(['a','b','c','d'], [('a','b')], 'breadth_first', root='a')
        self.assertEqual(names,['a','b','c','d'])
        np.testing.assert_allclose(points,[(0,0,0),(1,0,0),(1.5,0,0),(-1.5,0,0)],atol=1e-12)

    def test_shell_membership_and_radii(self):
        _, _, points = m._network_graph_layout(['a','b','c'], [], 'shell', shells=[['b','c'],['a']])
        np.testing.assert_allclose(points,[(2/3,0,0),(1,0,0),(-1,0,0)],atol=1e-12)
        for shells in ([['a','a','c']],[['a','b']],[['a','b','unknown']]):
            with self.assertRaises(ValueError):
                m._network_graph_layout(['a','b','c'], [], 'shell', shells=shells)

    def test_seeded_spring_repeats_without_global_rng_effects(self):
        args=(['a','b','c'],[('a','b'),('b','c')])
        first=m._network_graph_layout(*args,seed=7)
        self.assertEqual(first,m._network_graph_layout(*args,seed=7))
        self.assertNotEqual(first[2],m._network_graph_layout(*args,seed=8)[2])
        self.assertTrue(np.isfinite(first[2]).all())

    def test_empty_and_invalid_raw_inputs(self):
        self.assertEqual(m._network_graph_layout([],[],'circular'),([],[],[]))
        cases=[([1],[],{}),([],[(1,'b')],{}),([], [('a',)],{}),
               ([],[],{'layout':'unknown'}),(['a'],[],{'layout':'breadth_first','root':'x'}),
               (['a'],[],{'iterations':0}),(['a'],[],{'ideal_edge':float('nan')}),
               (['a'],[],{'ideal_edge':0}),(['a'],[],{'seed':-1})]
        for nodes,edges,options in cases:
            with self.subTest(options=options), self.assertRaises((TypeError,ValueError,OverflowError)):
                m._network_graph_layout(nodes,edges,**options)

    def test_size_and_work_budgets_precede_layout(self):
        for nodes,edges,options in [(['n']*1025,[],{}),([], [('a','b')]*8193,{}),
                (['x'*1_048_577],[],{}),([str(i) for i in range(1024)],[],{'iterations':1000}),
                ([],[],{'layout':'shell','shells':[['x']*1025]})]:
            with self.assertRaises(ValueError):m._network_graph_layout(nodes,edges,**options)


def run_network_layout_acceptance():
    stream=io.StringIO()
    result=unittest.TextTestRunner(stream=stream,verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeNetworkTests))
    if not result.wasSuccessful():raise AssertionError(stream.getvalue())
    print(stream.getvalue())

if __name__=='__main__':run_network_layout_acceptance()
