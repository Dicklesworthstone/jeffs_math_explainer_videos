"""Full native scene witnesses for the enhanced network front door."""
from __future__ import annotations
import copy
import io
import itertools
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.network_graph import NetworkGraph, NetworkLayoutAnimation


class NetworkSceneTests(unittest.TestCase):
    def graph(self, labels=False):
        return NetworkGraph(['a','b','c'], [('a','b'),('b','c')], layout='circular',
                            layout_scale=1, labels=labels)

    def connected(self, graph):
        for (a,b), edge in graph.edges.items():
            if a != b:
                np.testing.assert_allclose(edge.get_start(),graph.vertices[a].get_center(),atol=2e-6)
                np.testing.assert_allclose(edge.get_end(),graph.vertices[b].get_center(),atol=2e-6)
            else:
                np.testing.assert_allclose(edge[0].get_center(),graph.vertices[a].get_center(),atol=2e-6)
        for name,label in graph.labels.items():
            np.testing.assert_allclose(label.get_center(),graph.vertices[name].get_center(),atol=2e-6)

    def test_native_circular_layout_and_painter_order(self):
        graph=self.graph(True)
        self.assertEqual(graph.node_names,('a','b','c'))
        self.assertEqual(graph.edge_pairs,(('a','b'),('b','c')))
        for i,node in enumerate(graph.vertices.values()):
            angle=2*np.pi*i/3
            np.testing.assert_allclose(node.get_center(),(np.cos(angle),np.sin(angle),0),atol=2e-6)
        self.assertIs(graph[0][0],graph.get_edge('a','b'))
        self.assertIs(graph.get_edge('b','a'),graph.get_edge('a','b'))
        self.connected(graph)

    def test_inferred_nodes_duplicates_and_self_loops(self):
        graph=NetworkGraph(edges=[('b','a'),('a','b'),('a','a')],layout='circular')
        self.assertEqual(graph.node_names,('b','a'))
        self.assertEqual(len(graph.edges),2)
        graph.vertices['a'].shift((2,1,3));graph.refresh_edges()
        self.connected(graph)
        self.assertGreater(graph.get_edge('a','a').get_arc_length(),.1)

    def test_layouts_are_the_native_algorithms_not_host_reimplementations(self):
        graph=self.graph()
        for kind,options in [('spring',{'seed':13}),('shell',{'shells':[['b','c'],['a']]}),
                             ('breadth_first',{'root':'a'})]:
            names,_,positions=m._network_graph_layout(list(graph.node_names),list(graph.edge_pairs),kind,**options)
            graph.change_layout(kind,layout_scale=3,center=(1,2,3),**options)
            for name,p in zip(names,positions):
                np.testing.assert_allclose(graph.vertices[name].get_center(),3*np.array(p)+(1,2,3),atol=3e-6)
            self.connected(graph)

    def test_disconnected_breadth_first_keeps_every_vertex(self):
        graph=NetworkGraph(['a','b','c'],[('a','b')],layout='breadth_first',layout_scale=1,
                           layout_config={'root':'a'})
        np.testing.assert_allclose(graph.vertices['c'].get_center(),(1.5,0,0),atol=2e-6)
        self.connected(graph)

    def test_custom_layout_2d_3d_and_detached_readback(self):
        graph=self.graph()
        graph.change_layout({'a':(0,1),'b':(1,0,2),'c':(-1,0)},layout_scale=1)
        values=graph.get_layout();values['a'][:]=50
        np.testing.assert_allclose(graph.vertices['a'].get_center(),(0,1,0),atol=2e-6)
        graph.set_vertex_positions({'a':(2,3,4)})
        self.connected(graph)

    def test_update_preserves_styles_identities_views_annotations_and_time(self):
        graph=self.graph(True);scene=m.Scene();scene.add(graph)
        edge=graph.get_edge('a','b');view=edge.get_points()
        edge.set_stroke(m.RED,7);note=m.Square(.2);edge.add(note)
        seen=[];graph.vertices['a'].add_updater(lambda obj,dt:seen.append(dt),call=False)
        before=tuple(graph.vertices.values()),tuple(graph.edges.values())
        graph.change_layout('spring',seed=7)
        self.assertEqual(before,(tuple(graph.vertices.values()),tuple(graph.edges.values())))
        self.assertIs(edge[0],note);self.assertEqual(edge.get_stroke_color(),m.RED)
        np.testing.assert_allclose(view,edge.get_points())
        self.assertEqual(seen,[]);self.assertEqual(scene.get_time(),0)
        self.connected(graph)

    def test_individual_vertex_animation_reconnects_every_frame(self):
        graph=self.graph(True);scene=m.Scene();scene.add(graph)
        seen=[]
        graph.add_updater(lambda obj,dt:(self.connected(obj),seen.append(dt)),call=False)
        scene.play(graph.vertices['b'].animate.shift((1,2,1)),run_time=.25,rate_func=m.linear)
        self.assertTrue(seen);self.connected(graph)

    def test_layout_animation_straight_midpoint_and_target_ownership(self):
        graph=self.graph(True)
        start=graph.get_layout()
        target={'a':(0,2,0),'b':(-1,-1,1),'c':(2,-1,0)}
        anim=graph.animate_layout(target,layout_scale=1,rate_func=m.linear)
        self.assertIsInstance(anim,NetworkLayoutAnimation)
        self.assertTrue(m._requires_python_animation(anim))
        anim.begin();anim.interpolate(.5)
        for name in start:
            np.testing.assert_allclose(graph.vertices[name].get_center(),(start[name]+target[name])/2,atol=2e-6)
        self.connected(graph);anim.finish();self.connected(graph)
        self.assertFalse(vars(graph).get('_is_animating',False))

    def test_curved_layout_paths_reconnect_edges_after_interpolation(self):
        graph=self.graph(True)
        anim=graph.animate_layout('shell',layout_scale=2,layout_config={'shells':[['a'],['b','c']]},
                                  path_arc=np.pi/2,rate_func=m.linear)
        anim.begin()
        for alpha in (0,.25,.7,1,.5):
            anim.interpolate(alpha);self.connected(graph)
        anim.finish()

    def test_plain_builder_and_successive_layouts(self):
        graph=self.graph();scene=m.Scene();scene.add(graph)
        scene.play(graph.animate.change_layout('circular',layout_scale=2),run_time=.125,rate_func=m.linear)
        self.connected(graph)
        scene.play(m.Succession(graph.animate_layout('spring',layout_config={'seed':2},run_time=.125),
                               graph.animate_layout('circular',layout_scale=1,run_time=.125)))
        np.testing.assert_allclose(graph.vertices['a'].get_center(),(1,0,0),atol=2e-6)
        self.connected(graph)

    def test_coincident_vertices_can_separate_again(self):
        graph=self.graph()
        graph.set_vertex_positions({'a':(0,0),'b':(0,0)})
        graph.set_vertex_positions({'a':(2,3),'b':(-2,-3)})
        self.connected(graph)
        self.assertGreater(graph.get_edge('a','b').get_arc_length(),7)

    def test_affine_transforms_and_repeated_refresh_do_not_drift(self):
        graph=NetworkGraph(edges=[('a','b'),('b','b')],layout='circular',labels=True)
        graph.apply_matrix([[1,.5,0],[0,2,.2],[0,0,1]]).rotate(.6,axis=m.RIGHT).shift((2,-1,3))
        before=graph.get_layout()
        for _ in range(5):graph.refresh_edges();self.connected(graph)
        for name,p in before.items():np.testing.assert_array_equal(graph.get_layout()[name],p)

    def test_copy_deepcopy_pickle_remap_native_groups_and_updaters(self):
        original=self.graph(True);before=original.get_layout()
        for duplicate in (original.copy(),copy.deepcopy(original),pickle.loads(pickle.dumps(original))):
            self.assertIsNot(duplicate.vertices['a'],original.vertices['a'])
            duplicate.vertices['a'].shift((3,0,0));duplicate.update(0)
            self.connected(duplicate)
            for name,p in before.items():np.testing.assert_array_equal(original.get_layout()[name],p)

    def test_scene_checkpoint_restores_layout_and_connected_edges(self):
        graph=self.graph(True);scene=m.Scene();scene.add(graph)
        before=graph.get_layout();state=scene.get_state()
        graph.change_layout('spring',seed=7);scene.restore_state(state)
        graph=scene.mobjects[0];graph.refresh_edges();self.connected(graph)
        for name,p in before.items():np.testing.assert_allclose(graph.get_layout()[name],p,atol=2e-6)

    def test_bad_inputs_do_not_move_existing_vertices(self):
        graph=self.graph();before={name:dot.data.copy() for name,dot in graph.vertices.items()}
        for positions in ({'unknown':(0,0)}, {'a':(0,0),'b':(np.nan,0)}, {'a':(1+2j,0)}):
            with self.assertRaises((TypeError,ValueError)):graph.set_vertex_positions(positions)
        for layout in ({'a':(0,0)},'unknown'):
            with self.assertRaises((TypeError,ValueError)):graph.change_layout(layout)
        for name,data in before.items():np.testing.assert_array_equal(graph.vertices[name].data,data)

    def test_host_iterables_are_bounded_and_empty_graph_works(self):
        self.assertEqual(NetworkGraph(layout='circular').node_names,())
        calls=[]
        def labels():
            for i in itertools.count():calls.append(i);yield str(i)
        with self.assertRaises(ValueError):NetworkGraph(labels())
        self.assertEqual(len(calls),1025)
        with self.assertRaises(ValueError):NetworkGraph(edges=itertools.repeat(('a','b')))
        with self.assertRaises(TypeError):NetworkGraph('abc')

    def test_callback_failure_releases_animation_for_retry(self):
        graph=self.graph();scene=m.Scene();scene.add(graph)
        error=RuntimeError('authored path failed')
        def fail(a,b,t):raise error
        anim=graph.animate_layout('spring',path_func=fail)
        with self.assertRaises(RuntimeError) as caught:scene.play(anim,run_time=.125)
        self.assertIs(caught.exception,error)
        self.assertFalse(vars(graph).get('_is_animating',False))
        scene.play(graph.animate_layout('circular'),run_time=.125)
        self.connected(graph)

    def test_live_topology_edits_preserve_survivors_and_native_views(self):
        graph=self.graph(True);scene=m.Scene();scene.add(graph)
        a=graph.vertices['a'];edge=graph.get_edge('a','b');label=graph.labels['a']
        edge.set_stroke(m.RED,5);view=edge.get_points();note=m.Square(.2);a.add(note)
        callback=lambda obj,dt:None
        a.add_updater(callback,call=False)
        self.assertIs(graph.add_edges(('b','a'),('c','d'),('d','d'),positions={'d':(2,2)}),graph)
        self.assertEqual(graph.node_names,('a','b','c','d'))
        self.assertEqual(len(graph.edges),4)
        self.assertIs(graph.vertices['a'],a);self.assertIs(graph.labels['a'],label)
        self.assertIs(graph.get_edge('b','a'),edge);self.assertIs(a[0],note)
        self.assertIs(a.updaters[0],callback);self.assertEqual(edge.get_stroke_color(),m.RED)
        graph.set_vertex_positions({'b':(0,2)})
        np.testing.assert_allclose(view,edge.get_points(),atol=2e-6)
        self.assertIs(graph.vertices['d']._scene,scene)
        self.assertEqual(tuple(scene.mobjects),(graph,));self.assertEqual(scene.get_time(),0)
        self.connected(graph)
        graph.remove_vertices('c');graph.remove_edges(('b','a'))
        self.assertEqual(graph.node_names,('a','b','d'))
        self.assertEqual(graph.edge_pairs,(('d','d'),))
        self.assertIs(graph.vertices['a'],a);self.assertGreater(edge.get_num_points(),0)
        self.connected(graph)

    def test_topology_replacement_reorders_without_replacing_surviving_objects(self):
        graph=self.graph(True);old=graph.vertices;edge=graph.get_edge('a','b')
        graph.set_graph(['c','b','a'],[('b','a')],positions={'a':(3,2,1)})
        self.assertEqual(graph.node_names,('c','b','a'))
        self.assertIs(graph.get_edge('a','b'),edge)
        for name,obj in old.items():self.assertIs(graph.vertices[name],obj)
        self.connected(graph)
        graph.set_graph();self.assertEqual(graph.node_names,());self.assertEqual(graph.edge_pairs,())
        graph.add_edges(('x','y'),positions={'x':(-1,0),'y':(1,0)})
        self.assertEqual(set(graph.labels),{'x','y'});self.connected(graph)

    def test_invalid_topology_inputs_leave_existing_graph_intact(self):
        graph=self.graph(True);before=tuple(graph.get_family())
        records=[obj.data.copy() for obj in before]
        calls=[lambda:graph.add_edges(('c','d'),positions={'d':(np.nan,0)}),
               lambda:graph.set_graph(['a'],[('a',)]),
               lambda:graph.remove_vertices('absent'),lambda:graph.remove_edges(('a','c')),
               lambda:graph.set_graph(itertools.repeat('n'))]
        for call in calls:
            with self.assertRaises((TypeError,ValueError)):call()
            self.assertEqual(tuple(graph.get_family()),before)
            for obj,data in zip(before,records):np.testing.assert_array_equal(obj.data,data)
        graph.add_edges(('c','d'));self.connected(graph)

    def test_topology_copy_and_scene_snapshot_have_independent_catalogs(self):
        graph=self.graph(True);scene=m.Scene();scene.add(graph);state=scene.get_state()
        duplicate=graph.copy();duplicate.remove_vertices('a');duplicate.add_edges(('b','d'))
        self.assertEqual(graph.node_names,('a','b','c'));self.connected(graph);self.connected(duplicate)
        graph.remove_vertices('b');graph.add_edges(('a','d'))
        scene.restore_state(state);restored=scene.mobjects[0]
        self.assertEqual(restored.node_names,('a','b','c'))
        self.assertEqual(restored.edge_pairs,(('a','b'),('b','c')))
        restored.refresh_edges();self.connected(restored)

    def test_topology_reentry_and_active_animation_refuse_and_recover(self):
        graph=self.graph()
        def vertices():
            graph.add_edges(('b','new'))
            yield 'a'
        with self.assertRaises(RuntimeError):graph.set_graph(vertices())
        self.assertEqual(graph.node_names,('a','b','c'))
        animation=graph.animate_layout('circular');animation.begin()
        with self.assertRaises(RuntimeError):graph.add_edges(('a','new'))
        animation.finish();graph.add_edges(('a','new'));self.connected(graph)
        for call in (lambda:graph.animate.set_graph(['a']),
                     lambda:graph.animate.add_edges(('a','other')),
                     lambda:graph.animate.remove_vertices('a'),
                     lambda:graph.animate.remove_edges(('a','b'))):
            with self.assertRaises(m._CapabilityError):call()
        self.assertIn('new',graph.vertices)

    def test_updater_can_publish_connectivity_without_resetting_scene_clock(self):
        graph=self.graph();scene=m.Scene();scene.add(graph)
        tracker=m.ValueTracker(0);changed=[]
        def update(obj,dt):
            if tracker.get_value()>=.5 and not changed:
                obj.add_edges(('a','d'),positions={'d':(2,2,0)})
                obj.remove_vertices('b');changed.append(dt)
            self.connected(obj)
        graph.add_updater(update,call=False)
        scene.play(tracker.animate.set_value(1),run_time=.25,rate_func=m.linear)
        self.assertEqual(len(changed),1);self.assertIn('d',graph.vertices)
        self.assertNotIn('b',graph.vertices);self.assertGreater(scene.get_time(),0)

    def test_rendered_motion_matches_independent_geometry_at_one_four_threads(self):
        start={'a':(-2,0,0),'b':(2,0,0)};end={'a':(0,1,0),'b':(1,-1,0)}
        class Actual(m.Scene):
            def construct(self):
                graph=NetworkGraph(start,[('a','b')],layout=start,layout_scale=1,
                                   vertex_config={'radius':.125})
                self.add(graph)
                self.play(graph.animate_layout(end,layout_scale=1,rate_func=m.linear),run_time=.5)
        class Expected(m.Scene):
            def construct(self):
                tracker=m.ValueTracker(0)
                line=m.Line(start['a'],start['b'],stroke_color=m.GREY_B,stroke_width=2.5)
                a=m.Dot(start['a'],radius=.125,color=m.BLUE_D)
                b=m.Dot(start['b'],radius=.125,color=m.BLUE_D)
                group=m.VGroup(line,a,b)
                def update(obj):
                    alpha=tracker.get_value()
                    pa=(1-alpha)*np.array(start['a'])+alpha*np.array(end['a'])
                    pb=(1-alpha)*np.array(start['b'])+alpha*np.array(end['b'])
                    a.move_to(pa);b.move_to(pb);line.set_points_as_corners([pa,pb])
                group.add_updater(update,call=False);self.add(group)
                self.play(tracker.animate.set_value(1),run_time=.5,rate_func=m.linear)
        with tempfile.TemporaryDirectory() as directory:
            outputs=[]
            for threads in (1,4):
                paths=[Path(directory)/f'{name}-{threads}.y4m' for name in ('actual','expected')]
                for cls,path in zip((Actual,Expected),paths):
                    receipt=cls().render(path,format='y4m',resolution=(96,54),fps=8,threads=threads)
                    self.assertEqual(receipt.frame_count,4)
                self.assertEqual(paths[0].read_bytes(),paths[1].read_bytes())
                outputs.append(paths[0].read_bytes())
            self.assertEqual(outputs[0],outputs[1])


def run_network_scene_acceptance():
    stream=io.StringIO()
    result=unittest.TextTestRunner(stream=stream,verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NetworkSceneTests))
    if not result.wasSuccessful():raise AssertionError(stream.getvalue())
    print(stream.getvalue())

if __name__=='__main__':run_network_scene_acceptance()
