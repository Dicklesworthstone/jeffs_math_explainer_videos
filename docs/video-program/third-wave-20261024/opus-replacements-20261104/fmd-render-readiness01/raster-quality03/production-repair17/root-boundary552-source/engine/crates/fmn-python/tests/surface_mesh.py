"""Native sampled-surface wireframes, including independently authored output.

Run against an installed wheel with ``python -I surface_mesh.py``. The same
checks also run in the embedded extension and the permanent Gauntlet scenario.
"""
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m


def _refuses(call, exception=ValueError):
    try:
        call()
    except exception:
        return
    raise AssertionError("invalid SurfaceMesh input was accepted")


def verify_surface_mesh():
    verify_constructor_uniforms()
    verify_grid_edge_cases()
    calls = []

    def uv(u, v):
        calls.append((u, v))
        return (u, v, u * v)

    surface = m.ParametricSurface(uv, resolution=(5, 3))
    surface.stretch(2, 0).rotate(0.7, axis=m.UP).shift((2, -1, 0.5))
    before_calls = len(calls)
    points = surface.get_points().copy()
    normals = surface.get_unit_normals().copy()
    mesh = m.SurfaceMesh(surface, resolution=(4, 2), normal_nudge=0.2,
                         stroke_width=3.5, stroke_color=m.RED,
                         depth_test=False, joint_type="bevel")
    assert len(calls) == before_calls, "meshing must not execute uv_func again"
    assert len(mesh) == 6 and mesh.uv_surface is surface
    nudged = (points + 0.2 * normals).reshape(5, 3, 3)
    for line, index in zip(mesh[:4], np.linspace(0, 4, 4)):
        lo, hi = int(np.floor(index)), int(np.ceil(index))
        row = (1 - index % 1) * nudged[lo] + (index % 1) * nudged[hi]
        assert np.allclose(line.get_points()[[0, -1]], row[[0, -1]], atol=2e-6)
    for line in mesh:
        assert np.isclose(line.get_stroke_width(), 3.5)
        assert line.get_stroke_color() == m.RED
        assert line.uniforms["depth_test"] is False
        assert line.get_joint_type() == m.VMobject.joint_type_map["bevel"]
    frozen = [line.get_points().copy() for line in mesh]
    surface.shift(m.RIGHT)
    assert all(np.array_equal(line.get_points(), old) for line, old in zip(mesh, frozen))

    # Solid recipes were also wrong after rotations, stretches and grid edits.
    torus = m.Torus(resolution=(7, 5)).stretch(1.7, 0).rotate(0.6, axis=m.RIGHT)
    torus.shift((-2, 1, 0.25))
    live = torus.get_points()
    live[0] += (0.2, -0.3, 0.4)
    expected = torus.get_points().reshape(7, 5, 3).copy()
    torus_mesh = m.SurfaceMesh(torus, resolution=(7, 5), normal_nudge=0)
    assert np.allclose(torus_mesh[0].get_points()[[0, -1]], expected[0, [0, -1]])
    assert np.allclose(torus_mesh[6].get_points()[[0, -1]], expected[-1, [0, -1]])

    class AuthoredNormals(m.Surface):
        def get_unit_normals(self):
            return np.tile(m.RIGHT, (len(self.get_points()), 1))

    authored = AuthoredNormals(resolution=(2, 2))
    authored_mesh = m.SurfaceMesh(authored, resolution=(2, 2), normal_nudge=0.25)
    assert np.allclose(authored_mesh[0].get_points()[0], authored.get_points()[0] + 0.25 * m.RIGHT)
    for resolution in ((0, 0), (0, 3), (3, 0)):
        assert len(m.SurfaceMesh(m.Surface(resolution=resolution))) == 0
    singleton = m.Surface(resolution=(1, 1))
    assert len(m.SurfaceMesh(singleton, resolution=(1, 1))) == 2
    assert len(m.SurfaceMesh(authored, resolution=(0, 0))) == 0
    _refuses(lambda: m.SurfaceMesh(None), TypeError)
    _refuses(lambda: m.SurfaceMesh(m.VMobject()), TypeError)
    for kwargs in (dict(normal_nudge=float("nan")), dict(stroke_width=-1),
                   dict(stroke_width=float("inf")), dict(resolution=(65537, 1))):
        _refuses(lambda kwargs=kwargs: m.SurfaceMesh(authored, **kwargs))
    authored.resolution = (2, 3)
    _refuses(lambda: m.SurfaceMesh(authored))
    authored.resolution = (2, 2)
    authored.get_unit_normals = lambda: np.zeros((3, 3))
    _refuses(lambda: m.SurfaceMesh(authored))
    authored.get_unit_normals = lambda: np.full((4, 3), float("nan"))
    _refuses(lambda: m.SurfaceMesh(authored))
    sentinel = RuntimeError("authored normal failure")

    def broken_normals():
        raise sentinel

    authored.get_unit_normals = broken_normals
    try:
        m.SurfaceMesh(authored)
    except RuntimeError as error:
        assert error is sentinel, "the original callback exception must survive"
    else:
        raise AssertionError("normal callback failure was swallowed")
    return 12


def verify_constructor_uniforms():
    """Inherited VMobject options must reach real native-built descendants."""
    constructors = (m.Line, m.Circle, m.Square, m.Arrow, m.DashedLine,
                    m.Elbow, m.DecimalNumber, m.Integer)
    for constructor in constructors:
        obj = constructor(joint_type="bevel", anti_alias_width=0.75,
                          scale_stroke_with_zoom=True, depth_test=True,
                          is_fixed_in_frame=True)
        family = [part for part in obj.get_family() if part.has_points()]
        assert family, constructor.__name__
        for part in family:
            assert part.get_joint_type() == m.VMobject.joint_type_map["bevel"]
            assert np.isclose(part.get_anti_alias_width(), 0.75)
            assert part.get_scale_stroke_with_zoom()
            assert part.uniforms["depth_test"] is True
            assert part.is_fixed_in_frame()
        defaults = constructor()
        assert defaults.get_joint_type() == m.VMobject.joint_type_map["auto"]
    for kwargs in (dict(joint_type="not-a-joint"),
                   dict(anti_alias_width=-1),
                   dict(anti_alias_width=float("nan")),
                   dict(anti_alias_width=float("inf"))):
        _refuses(lambda kwargs=kwargs: m.Line(**kwargs))
    _refuses(lambda: m.Line(unsupported_render_option=True), TypeError)


def verify_grid_edge_cases():
    """Compare both iso-curve directions against independent sampled rows."""
    for nu in (1, 2, 3, 7, 11):
        for nv in (1, 2, 4, 9):
            surface = m.ParametricSurface(
                lambda u, v: (u, v, u * v), resolution=(nu, nv))
            surface.stretch(1.3, 0).rotate(0.4, axis=m.UP).shift((0.2, -0.4, 0.7))
            nudged = (surface.get_points() + 0.025 * surface.get_unit_normals()).reshape(nu, nv, 3)
            for rows, columns in ((0, 0), (1, 1), (3, 5), (7, 2)):
                mesh = m.SurfaceMesh(surface, resolution=(rows, columns), normal_nudge=0.025)
                assert len(mesh) == rows + columns
                for line, index in zip(mesh[:rows], np.linspace(0, nu - 1, rows)):
                    lo, hi = int(np.floor(index)), int(np.ceil(index))
                    row = (1 - index % 1) * nudged[lo] + (index % 1) * nudged[hi]
                    assert np.allclose(line.get_points()[[0, -1]], row[[0, -1]], atol=2e-6)
                for line, index in zip(mesh[rows:], np.linspace(0, nv - 1, columns)):
                    lo, hi = int(np.floor(index)), int(np.ceil(index))
                    column = (1 - index % 1) * nudged[:, lo] + (index % 1) * nudged[:, hi]
                    assert np.allclose(line.get_points()[[0, -1]], column[[0, -1]], atol=2e-6)
    copied = mesh.copy()
    before = mesh[0].get_points().copy()
    assert np.array_equal(copied[0].get_points(), before)
    copied.shift(m.RIGHT)
    assert np.array_equal(mesh[0].get_points(), before)


def render_surface_mesh(destination, seed=0):
    """Compare the real mesh renderer to six independently authored Lines."""
    checks = verify_surface_mesh()

    class WireframeScene(m.Scene):
        reference = False
        random_seed = seed & 0xFFFF_FFFF

        def construct(self):
            style = dict(stroke_color=m.RED, stroke_width=3, joint_type="no_joint")
            if self.reference:
                lines = [m.Line((1, y, 0), (0, y, 0), **style)
                         for y in (-1.75, 0.25, 2.25)]
                lines += [m.Line((x, -1.75, 0), (x, 2.25, 0), **style)
                          for x in (1, 0.5, 0)]
                self.add(m.VGroup(*lines))
            else:
                surface = m.ParametricSurface(lambda u, v: (u, v, 0),
                    u_range=(-1, 1), v_range=(-1, 1), resolution=(2, 2))
                surface.stretch(2, 0).stretch(0.5, 1).rotate(np.pi / 2)
                surface.shift((0.5, 0.25, 0))
                self.add(m.SurfaceMesh(surface, resolution=(3, 3),
                    normal_nudge=0, depth_test=False, **style))

    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    images = []
    for name, reference, threads in (("mesh-1.png", False, 1),
                                      ("mesh-4.png", False, 4),
                                      ("reference.png", True, 1)):
        scene = WireframeScene()
        scene.reference = reference
        path = destination / name
        scene._begin_png(str(path), 128, 128, 30, threads, seed)
        try:
            scene.run()
            scene._finish_render(scene.frame._core, scene.camera.light_source.get_center())
        except BaseException:
            scene._abort_render()
            raise
        images.append(path.read_bytes())
    assert images[0] == images[1], "thread count changed wireframe output"
    assert images[0] == images[2], "wireframe differs from analytic grid Lines"
    assert images[0][:8] == b"\x89PNG\r\n\x1a\n"
    return images[0], checks


if __name__ == "__main__":
    with tempfile.TemporaryDirectory(prefix="fmn-surface-mesh-") as directory:
        png, checks = render_surface_mesh(directory)
    print(f"SurfaceMesh: {checks} native semantic checks; identical 1/4-thread and analytic PNGs ({len(png)} bytes)")
