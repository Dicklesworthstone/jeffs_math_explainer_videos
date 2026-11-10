"""Animatable undirected networks over Atlas layouts and native scene geometry.

This enhanced front door needs no Python networkx installation. Atlas owns
layout arithmetic and its seeded RNG; existing mobjects own all drawn records.
"""
from __future__ import annotations

from collections.abc import Mapping
import itertools
import math

import numpy as np
import manimlib as m

_MAX_NODES = 1024
_MAX_EDGES = 8192
_EDITING = set()


def _bounded(values, limit, name):
    if isinstance(values, (str, bytes)):
        raise TypeError(name + ' must be an iterable, not a string')
    items = list(itertools.islice(iter(values), limit + 1))
    if len(items) > limit:
        raise ValueError(name + ' exceeds its item budget')
    return items


def _inputs(vertices, edges):
    names = _bounded(vertices, _MAX_NODES, 'network vertices')
    pairs = []
    for edge in _bounded(edges, _MAX_EDGES, 'network edges'):
        pair = _bounded(edge, 2, 'edge endpoints')
        if len(pair) != 2:
            raise ValueError('each network edge needs two endpoints')
        pairs.append(tuple(pair))
    if any(not isinstance(name, str) for name in names):
        raise TypeError('network vertex labels must be strings')
    if any(not isinstance(name, str) for pair in pairs for name in pair):
        raise TypeError('network edge labels must be strings')
    return names, pairs


def _point(value):
    values = _bounded(value, 3, 'network position')
    if len(values) not in (2, 3):
        raise ValueError('network positions need two or three components')
    array = np.asarray(values)
    if array.dtype.kind not in 'biuf':
        raise TypeError('network positions must be real numeric coordinates')
    result = np.zeros(3, dtype=float)
    result[:len(values)] = array
    if not np.isfinite(result).all() or np.any(np.abs(result) > np.finfo(np.float32).max):
        raise ValueError('network positions must be finite and f32-representable')
    return result


def _layout(names, pairs, layout, scale, center, options):
    native = getattr(m, '_network_graph_layout', None)
    if native is None:
        raise ImportError('this wheel does not contain the native network layout bridge')
    scale = float(scale)
    if not math.isfinite(scale) or scale <= 0:
        raise ValueError('layout_scale must be finite and positive')
    center = _point(center)
    options = dict(options)
    if 'shells' in options:
        groups = _bounded(options['shells'], _MAX_NODES, 'network shells')
        rings, total = [], 0
        for group in groups:
            ring = _bounded(group, _MAX_NODES-total, 'shell members')
            total += len(ring)
            rings.append(ring)
        options['shells'] = rings
    custom = isinstance(layout, Mapping)
    labels, edges, points = native(list(names), list(pairs), 'circular' if custom else layout, **options)
    if custom:
        if set(layout) != set(labels):
            raise ValueError('a custom layout must position every vertex exactly once')
        points = [_point(layout[label]) for label in labels]
    with np.errstate(over='ignore', invalid='ignore'):
        positions = [_point(center + scale * np.asarray(p, dtype=float)) for p in points]
    return tuple(labels), tuple(tuple(edge) for edge in edges), positions


def _follow_graph(graph):
    # No closure over an original owner: copied/updater-driven graphs refer
    # exclusively to their own remapped native groups.
    graph.refresh_edges()


def _discrete(method):
    def refuse(self, *args, **kwargs):
        raise m._CapabilityError(
            'network topology edits are discrete; edit between animations or '
            'in a scene updater, and use animate_layout() for vertex motion'
        )
    method._override_animate = refuse
    return method


def _edge(a, b, coordinates, style, radius):
    if a != b:
        return m.Line(coordinates[a], coordinates[b], buff=0, **style)
    result = m.Circle(radius=radius, **style)
    result.move_to(coordinates[a] + np.array([0, radius, 0]))
    result.add(m.VectorizedPoint(coordinates[a]))
    return result


class NetworkGraph(m.VGroup):
    """String-labeled undirected graph with live edges and native layouts.

    Vertices supplied explicitly come first; edge-only endpoints follow in
    first-appearance order. Duplicate undirected edges coalesce. Self-loops
    draw as native circles. ``vertices``, ``edges`` and ``labels`` are fresh
    mappings to owned scene objects, not a separate mutable graph database.
    """
    def __init__(self, vertices=(), edges=(), *, layout='spring', layout_scale=2.0,
                 center=(0, 0, 0), layout_config=None, labels=False,
                 vertex_config=None, edge_config=None, label_config=None, **kwargs):
        names, pairs = _inputs(vertices, edges)
        names, pairs, positions = _layout(names, pairs, layout, layout_scale, center, layout_config or {})
        vertex_style = dict(radius=.12, color=m.BLUE_D)
        vertex_style.update(vertex_config or {})
        radius = float(vertex_style['radius'])
        if not math.isfinite(radius) or radius <= 0:
            raise ValueError('network vertex radius must be finite and positive')
        vertex_style['radius'] = radius
        edge_style = dict(stroke_color=m.GREY_B, stroke_width=2.5)
        edge_style.update(edge_config or {})
        if edge_style.get('buff', 0) != 0 or edge_style.get('path_arc', 0) != 0:
            raise ValueError('network edges are straight center-to-center connectors')
        edge_style.pop('buff', None)
        edge_style.pop('path_arc', None)
        label_style = dict(font_size=16, color=m.WHITE)
        label_style.update(label_config or {})
        if not isinstance(labels, (bool, Mapping)):
            raise TypeError('labels must be a boolean or a vertex-to-text mapping')
        if isinstance(labels, Mapping) and not set(labels).issubset(names):
            raise ValueError('a label names an unknown vertex')
        dots = [m.Dot(point, **vertex_style) for point in positions]
        coordinates = dict(zip(names, positions))
        lines = [_edge(a, b, coordinates, edge_style, 2 * radius) for a, b in pairs]
        texts, label_names = [], []
        for name, point in zip(names, positions):
            text = name if labels is True else labels.get(name) if isinstance(labels, Mapping) else None
            if text is not None:
                if not isinstance(text, str):
                    raise TypeError('vertex labels must be plain strings')
                texts.append(m.Text(text, **label_style).move_to(point))
                label_names.append(name)
        self._node_names, self._edge_pairs = names, pairs
        self._label_names = tuple(label_names)
        self._vertex_config, self._edge_config, self._label_config = vertex_style, edge_style, label_style
        self._label_policy = dict(labels) if isinstance(labels, Mapping) else labels
        self._edge_group, self._vertex_group, self._label_group = m.VGroup(*lines), m.VGroup(*dots), m.VGroup(*texts)
        super().__init__(self._edge_group, self._vertex_group, self._label_group, **kwargs)
        self.add_updater(_follow_graph, call=False)

    @property
    def node_names(self):
        return self._node_names

    @property
    def edge_pairs(self):
        return self._edge_pairs

    @property
    def vertices(self):
        return dict(zip(self._node_names, self._vertex_group.submobjects))

    @property
    def edges(self):
        return dict(zip(self._edge_pairs, self._edge_group.submobjects))

    @property
    def labels(self):
        return dict(zip(self._label_names, self._label_group.submobjects))

    def get_edge(self, a, b):
        edges = self.edges
        return edges[(a, b)] if (a, b) in edges else edges[(b, a)]

    def _check_structure(self):
        if (len(self._vertex_group) != len(self._node_names)
                or len(self._edge_group) != len(self._edge_pairs)
                or len(self._label_group) != len(self._label_names)):
            raise RuntimeError('network-owned groups were structurally changed')

    def refresh_edges(self):
        """Follow current vertex positions without changing topology or time."""
        self._check_structure()
        positions = {name: _point(dot.get_center()) for name, dot in self.vertices.items()}
        for (a, b), edge in self.edges.items():
            if a == b:
                edge.shift(positions[a] - edge.submobjects[0].get_center())
            else:
                edge.set_points_as_corners([positions[a], positions[b]])
        for name, label in self.labels.items():
            label.move_to(positions[name])
        return self

    def get_layout(self):
        """Return detached world-coordinate positions from the live vertices."""
        return {name: np.array(dot.get_center(), copy=True) for name, dot in self.vertices.items()}

    @_discrete
    def set_graph(self, vertices=(), edges=(), *, positions=None):
        """Replace connectivity while preserving all surviving visual objects.

        New vertices default to the origin unless positioned explicitly. Old
        vertices retain their live geometry and styles. The constructor's label
        policy and style dictionaries apply only to newly constructed members.
        This is a discrete edit, refused during an animation of this family.
        """
        marker = id(self)
        if marker in _EDITING:
            raise RuntimeError('network topology editing cannot reenter itself')
        if vars(self).get('_is_animating', False):
            raise RuntimeError('finish the graph animation before editing its topology')
        self._check_structure()
        before = (self._node_names, self._edge_pairs, tuple(self._vertex_group),
                  tuple(self._edge_group), tuple(self._label_group))
        _EDITING.add(marker)
        try:
            names, pairs = _inputs(vertices, edges)
            names, pairs, _ = _layout(names, pairs, 'circular', 1, (0, 0, 0), {})
            supplied = {} if positions is None else positions
            if not isinstance(supplied, Mapping) or not set(supplied).issubset(names):
                raise ValueError('positions must map vertices in the replacement graph')
            supplied = {name: _point(point) for name, point in supplied.items()}
            old_vertices, old_edges, old_labels = self.vertices, self.edges, self.labels
            coordinates = {name: supplied.get(name, _point(old_vertices[name].get_center())
                           if name in old_vertices else np.zeros(3)) for name in names}
            dots = [old_vertices[name] if name in old_vertices else
                    m.Dot(coordinates[name], **self._vertex_config) for name in names]
            lines = []
            for a, b in pairs:
                old = old_edges.get((a, b), old_edges.get((b, a)))
                lines.append(old if old is not None else
                             _edge(a, b, coordinates, self._edge_config, 2*self._vertex_config['radius']))
            texts, label_names = [], []
            for name in names:
                text = name if self._label_policy is True else self._label_policy.get(name) \
                    if isinstance(self._label_policy, Mapping) else None
                if text is not None:
                    texts.append(old_labels[name] if name in old_labels else
                                 m.Text(text, **self._label_config).move_to(coordinates[name]))
                    label_names.append(name)
            after = (self._node_names, self._edge_pairs, tuple(self._vertex_group),
                     tuple(self._edge_group), tuple(self._label_group))
            if before != after or vars(self).get('_is_animating', False):
                raise RuntimeError('network changed during topology preparation')
            # All user conversion, validation, layout and native construction
            # precede any change to the graph's existing family.
            self._edge_group.set_submobjects(lines)
            self._vertex_group.set_submobjects(dots)
            self._label_group.set_submobjects(texts)
            self._node_names, self._edge_pairs, self._label_names = names, pairs, tuple(label_names)
            current_vertices = self.vertices
            for name, point in supplied.items():
                current_vertices[name].move_to(point)
            self.refresh_edges()
        finally:
            _EDITING.discard(marker)
        return self

    @_discrete
    def add_edges(self, *edges, positions=None):
        """Add edges and infer any new endpoints, retaining existing members."""
        added = _bounded(edges, _MAX_EDGES, 'network edges')
        return self.set_graph(self._node_names, itertools.chain(self._edge_pairs, added), positions=positions)

    @_discrete
    def remove_vertices(self, *vertices):
        """Remove the named vertices, their labels and every incident edge."""
        removed = set(_bounded(vertices, _MAX_NODES, 'network vertices'))
        if not removed.issubset(self._node_names):
            raise ValueError('cannot remove an unknown vertex')
        return self.set_graph([name for name in self._node_names if name not in removed],
                              [pair for pair in self._edge_pairs if not removed.intersection(pair)])

    @_discrete
    def remove_edges(self, *edges):
        """Remove undirected edges in either orientation without dropping vertices."""
        _, pairs = _inputs((), edges)
        removed = {frozenset(pair) for pair in pairs}
        if not removed.issubset(frozenset(pair) for pair in self._edge_pairs):
            raise ValueError('cannot remove an unknown edge')
        return self.set_graph(self._node_names,
                              [pair for pair in self._edge_pairs if frozenset(pair) not in removed])

    def set_vertex_positions(self, positions):
        """Move a subset of vertices, validating every position before writing."""
        self._check_structure()
        if not isinstance(positions, Mapping):
            raise TypeError('vertex positions must be a mapping')
        vertices = self.vertices
        if not set(positions).issubset(vertices):
            raise ValueError('a position names an unknown vertex')
        prepared = [(vertices[name], _point(point)) for name, point in positions.items()]
        for dot, point in prepared:
            dot.move_to(point)
        return self.refresh_edges()

    def change_layout(self, layout='spring', *, layout_scale=2.0, center=(0, 0, 0), **layout_config):
        """Apply a named native layout or a full custom coordinate mapping.

        Targets are world coordinates. Scale/center apply to the new layout,
        not to earlier geometric transforms of the displayed graph.
        """
        self._check_structure()
        names, pairs, positions = _layout(self._node_names, self._edge_pairs, layout,
                                          layout_scale, center, layout_config)
        if names != self._node_names or pairs != self._edge_pairs:
            raise RuntimeError('native layout changed the graph topology')
        return self.set_vertex_positions(dict(zip(names, positions)))

    def animate_layout(self, layout='spring', *, layout_scale=2.0, center=(0, 0, 0),
                       layout_config=None, **animation_config):
        """Animate vertices and reconnect edges after each shared-clock sample."""
        target = self.copy()
        target.change_layout(layout, layout_scale=layout_scale, center=center, **(layout_config or {}))
        return NetworkLayoutAnimation(self, target, **animation_config)


class NetworkLayoutAnimation(m.Transform):
    """The normal Transform lifecycle, plus post-interpolation edge geometry."""
    def __init__(self, graph, target, **kwargs):
        if not isinstance(graph, NetworkGraph) or not isinstance(target, NetworkGraph):
            raise TypeError('network layout animation requires NetworkGraph endpoints')
        if (graph.node_names != target.node_names or graph.edge_pairs != target.edge_pairs
                or graph._label_names != target._label_names):
            raise ValueError('layout animation requires the same ordered graph topology')
        super().__init__(graph, target, **kwargs)

    def interpolate_mobject(self, alpha):
        super().interpolate_mobject(alpha)
        self.mobject.refresh_edges()
