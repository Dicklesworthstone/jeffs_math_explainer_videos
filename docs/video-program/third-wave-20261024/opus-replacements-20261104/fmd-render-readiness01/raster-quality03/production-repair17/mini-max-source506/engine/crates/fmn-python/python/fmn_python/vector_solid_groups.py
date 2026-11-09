"""Vector solid groups retain authored roots while Atlas prepares their faces.

Face topology/extrusion remains native. Group lifecycle and placement stay on
VGroup3D/VCube, including later class overrides and public face factories.
"""
from __future__ import annotations

from contextlib import contextmanager
from itertools import islice
import sys

from .solid_groups import _bind, _faces, _finite, _MAX_FAMILY


def install_vector_solid_groups(native):
    g = vars(native)
    if g.get('_FMN_VECTOR_SOLID_GROUPS_INSTALLED', False):
        return
    VCube, VPrism, Dodecahedron, Prismify, VMobject = (g[name] for name in
        ('VCube', 'VPrism', 'Dodecahedron', 'Prismify', 'VMobject'))
    active = set()

    @contextmanager
    def constructing(obj):
        if obj._is_bound():
            raise RuntimeError('vector solid construction requires a detached target')
        if id(obj) in active:
            raise RuntimeError('vector solid construction is already in progress')
        active.add(id(obj))
        try:
            yield
        finally:
            active.remove(id(obj))

    def options(name, kwargs, defaults, *, default_shading=(.2, .2, .2), allow_z=True):
        values = dict(kwargs)
        z_index = int(values.pop('z_index', 0)) if allow_z else 0
        if not -(1 << 31) <= z_index < (1 << 31):
            raise ValueError('vector solid z_index must fit a signed 32-bit integer')
        shading = tuple(_finite(value, 'vector solid shading') for value in islice(iter(
            values.pop('shading', default_shading)), 4))
        if len(shading) != 3:
            raise ValueError('vector solid shading must have three components')
        style, depth_test, shading, joint_type = g['_split_native_vgroup3d_kwargs'](
            name, values, shading)
        for key, value in defaults.items():
            style.setdefault(key, value)
        for key in ('fill_opacity', 'stroke_opacity', 'opacity', 'fill_border_width'):
            if key in style and style[key] is not None:
                style[key] = _finite(style[key], 'vector solid ' + key)
        g['_preflight_vmobject_style_kwargs'](style)
        return style, depth_test, shading, joint_type, z_index

    def assemble(cls, obj, faces, config):
        style, depth_test, shading, joint_type, z_index = config
        faces = _faces(g, obj, faces, VMobject)
        # The Reference's VGroup3D runs VMobject.__init__ before it adds the
        # faces, so the point-less root takes VMobject's unset-channel defaults
        # (GREY_C fill at opacity 0, GREY_A stroke at width 4) under the
        # constructor's own keywords (fm-78b6). They apply during the normal
        # color hook, so an authored override can still replace them. Supplying
        # already-styled faces avoids repainting them after their public factory.
        root_style = dict(style)
        if root_style.get('color') is not None:
            root_style['fill_color'] = root_style['stroke_color'] = root_style['color']
        super(cls, obj).__init__(*faces, depth_test=depth_test, shading=shading,
                                joint_type=joint_type, z_index=z_index, **root_style)
        if obj._is_bound() or getattr(obj, '_scene', None) is not None:
            raise RuntimeError('vector solid ownership changed during initialization')
        # Mobject keyword storage is not native painter metadata. Publish the
        # order through its public native-backed operation, as Surface does.
        obj.set_z_index(z_index)

    def symbol(name):
        return getattr(sys.modules.get(VCube.__module__), name, g[name])

    def vcube(self, side_length=2.0, fill_color=g['_StyleDefault'](g['_BLUE_D']), fill_opacity=1,
              stroke_width=0, **kwargs):
        side_length = _finite(side_length, 'VCube side_length')
        config = options(type(self).__name__ + '()', kwargs,
                         dict(fill_color=fill_color, fill_opacity=fill_opacity, stroke_width=stroke_width))
        with constructing(self):
            self.side_length = side_length
            face = symbol('Square')(side_length=side_length, **config[0])
            _faces(g, self, [face], VMobject)
            faces = _faces(g, self, symbol('square_to_cube_faces')(face), VMobject)
            assemble(VCube, self, faces, config)

    def vprism(self, width=3.0, height=2.0, depth=1.0, **kwargs):
        dimensions = tuple(_finite(value, 'VPrism ' + name) for name, value in
                           (('width', width), ('height', height), ('depth', depth)))
        super(VPrism, self).__init__(**kwargs)
        for dim, length in enumerate(dimensions):
            self.rescale_to_fit(length, dim, stretch=True)

    def dodecahedron(self, fill_color=g['_StyleDefault'](g['_BLUE_E']), fill_opacity=1,
                     stroke_color=g['_StyleDefault'](g['_BLUE_E']), stroke_width=1, shading=(.2, .2, .2), **kwargs):
        config = options('Dodecahedron()', kwargs, dict(fill_color=fill_color,
                         fill_opacity=fill_opacity, stroke_color=stroke_color, stroke_width=stroke_width),
                         default_shading=shading)
        with constructing(self):
            scratch = g['_native_shell_factory']()
            specs = scratch._build_dodecahedron(g['_native_shell_factory'], fill_color,
                _finite(fill_opacity, 'Dodecahedron fill_opacity'), stroke_color,
                _finite(stroke_width, 'Dodecahedron stroke_width'), config[2], 0)
            g['_hang_native_children'](scratch, specs)
            # Atlas supplies the ordered topology; each public face retains
            # Polygon's vertex API and authorable construction lifecycle.
            polygon = symbol('Polygon')
            faces = [
                polygon(*face.get_points()[::2][:-1], **config[0])
                for face in scratch.submobjects
            ]
            assemble(Dodecahedron, self, faces, config)

    def prismify(self, vmobject, depth=1.0, direction=g['_IN'], **kwargs):
        if not isinstance(vmobject, VMobject):
            raise TypeError('Prismify source must be a VMobject')
        depth = _finite(depth, 'Prismify depth')
        direction = tuple(_finite(value, 'Prismify direction') for value in g['_vec3'](direction))
        config = options('Prismify()', kwargs, {}, allow_z=False)
        with constructing(self):
            scratch = g['_native_shell_factory']()
            if vmobject.submobjects:
                sources = list(islice(iter(vmobject.family_members_with_points()), _MAX_FAMILY + 1))
                if len(sources) > _MAX_FAMILY:
                    raise ValueError('Prismify exceeds the 65536-member budget')
                sources = [member for member in sources if isinstance(member, VMobject)]
                if not sources:
                    raise ValueError('Prismify family has no pointful VMobject members to extrude')
                specs = scratch._build_prismify_family(g['_native_shell_factory'], vmobject, depth, direction)
                g['_hang_native_children'](scratch, specs)
                if len(scratch.submobjects) != len(sources):
                    raise RuntimeError('Prismify source family changed during native preparation')
                for piece, source in zip(scratch.submobjects, sources):
                    piece.match_style(source)
            else:
                specs = scratch._build_prismify(g['_native_shell_factory'], vmobject, depth, direction)
                g['_hang_native_children'](scratch, specs)
                for piece in scratch.submobjects:
                    piece.match_style(vmobject)
            # Preserve per-source paints: explicit Prismify styles belong to
            # its root, not to every differently-colored extruded child.
            assemble(Prismify, self, scratch.submobjects, config)

    for cls, constructor in ((VCube, vcube), (VPrism, vprism),
                              (Dodecahedron, dodecahedron), (Prismify, prismify)):
        _bind(cls, '__init__', constructor)
    g['_FMN_VECTOR_SOLID_GROUPS_INSTALLED'] = True
