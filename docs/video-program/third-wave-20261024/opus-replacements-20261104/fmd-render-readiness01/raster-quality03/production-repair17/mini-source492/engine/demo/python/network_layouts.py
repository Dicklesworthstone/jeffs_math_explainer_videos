"""Native graph layouts and attached connectors, without external assets."""
from manimlib import *
from fmn_python.network_graph import NetworkGraph


class NetworkLayouts(Scene):
    def construct(self):
        graph = NetworkGraph(
            ['hub', 'a', 'b', 'c', 'd'],
            [('hub', 'a'), ('hub', 'b'), ('hub', 'c'), ('hub', 'd'), ('a', 'b')],
            layout='circular', labels=True,
            vertex_config={'radius': .24}, label_config={'font_size': 14},
        )
        self.add(graph)
        self.play(graph.animate_layout('breadth_first', layout_config={'root': 'hub'}), run_time=1)
        self.play(graph.vertices['a'].animate.shift(UP), run_time=.5)
        graph.add_edges(('c', 'new'), ('new', 'new'), positions={'new': (2, -1, 0)})
        graph.remove_edges(('a', 'hub'))
        self.play(graph.animate_layout('spring', layout_config={'seed': 7}, path_arc=PI/3), run_time=1)
