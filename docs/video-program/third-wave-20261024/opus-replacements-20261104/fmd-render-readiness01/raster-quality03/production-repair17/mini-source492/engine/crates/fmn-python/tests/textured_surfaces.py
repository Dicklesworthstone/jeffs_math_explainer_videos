"""Actual-extension acceptance for retained texture grids and indexed meshes."""
from pathlib import Path
from types import SimpleNamespace
import pickle
import struct
import tempfile
import zlib

import numpy as np
import manimlib as m


def texture_fixture(path):
    # Original two-by-two test art, no external or reference assets.
    pixels = bytes((255, 0, 0, 255, 0, 255, 0, 255,
                    0, 0, 255, 255, 255, 255, 255, 255))
    def chunk(kind, body):
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body))
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 2, 2, 8, 6, 0, 0, 0))
                     + chunk(b'IDAT', zlib.compress(b'\0' + pixels[:8] + b'\0' + pixels[8:]))
                     + chunk(b'IEND', b''))
    return path


def decode_png(data):
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    offset, packed = 8, bytearray()
    while offset < len(data):
        size = int.from_bytes(data[offset:offset + 4], 'big')
        kind, body = data[offset + 4:offset + 8], data[offset + 8:offset + 8 + size]
        assert zlib.crc32(kind + body) == int.from_bytes(data[offset + 8 + size:offset + 12 + size], 'big')
        offset += size + 12
        if kind == b'IHDR':
            width, height, depth, color, compression, filtering, interlace = struct.unpack('>IIBBBBB', body)
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
        elif kind == b'IDAT':
            packed.extend(body)
        elif kind == b'IEND':
            assert offset == len(data)
            break
    raw, stride = zlib.decompress(packed), width * 4
    assert len(raw) == height * (stride + 1)
    decoded = bytearray(height * stride)
    for y in range(height):
        mode = raw[y * (stride + 1)]
        assert mode in range(5)
        for x, value in enumerate(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)]):
            left = decoded[y * stride + x - 4] if x >= 4 else 0
            up = decoded[(y - 1) * stride + x] if y else 0
            corner = decoded[(y - 1) * stride + x - 4] if x >= 4 and y else 0
            prediction = left + up - corner
            paeth = min((left, up, corner), key=lambda candidate: abs(prediction - candidate))
            decoded[y * stride + x] = (value + (0, left, up, (left + up) // 2, paeth)[mode]) & 255
    return np.frombuffer(decoded, dtype=np.uint8).reshape(height, width, 4)


def mesh_fixture():
    return SimpleNamespace(
        vertices=np.array([[-2., -2., 0.], [2., -2., 0.], [2., 2., 0.], [-2., 2., 0.]]),
        faces=np.array([[3, 0, 2], [2, 0, 1]], dtype=np.uint32),
        visual=SimpleNamespace(uv=np.array([[0., 0.], [1., 0.], [1., 1.], [0., 1.]])),
    )


def refuses(error_type, callback):
    try:
        callback()
    except error_type:
        return
    raise AssertionError('expected ' + str(error_type))


def verify_textured_surfaces():
    root = Path(tempfile.mkdtemp(prefix='fmn-texture-records-'))
    path = texture_fixture(root / 'colors.png')
    other = texture_fixture(root / 'other.png')
    source = m.ParametricSurface(lambda u, v: (u, v, .2 * u * v),
                                 u_range=(-2, 2), v_range=(-1, 1), resolution=(3, 4), opacity=.6)
    source.shift(m.RIGHT).rotate(.25, axis=m.UP)
    points = source.get_points().copy()
    normals = source.data['d_normal_point'].copy()
    texture = m.TexturedSurface(source, path)
    assert texture.resolution == source.resolution and texture.num_textures == 1
    assert texture.uv_surface is source
    assert texture.data.dtype.names == ('point', 'd_normal_point', 'im_coords', 'opacity')
    np.testing.assert_array_equal(texture.get_points(), points)
    np.testing.assert_array_equal(texture.data['d_normal_point'], normals)
    np.testing.assert_allclose(texture.get_opacities(), .6)
    np.testing.assert_allclose(texture.data['im_coords'],
                               [(u, v) for u in np.linspace(0, 1, 3) for v in np.linspace(1, 0, 4)])
    assert len(texture.get_triangle_indices()) == 36
    assert texture.set_opacity([.2, .8]) is texture
    np.testing.assert_allclose(texture.get_opacities()[[0, -1]], [.2, .8])
    texture.set_color(m.RED, opacity=.4)
    np.testing.assert_allclose(texture.get_opacities(), .4)
    texture.set_image_coords_by_uv_func(lambda u, v: (u / 2, v / 2))
    np.testing.assert_allclose(texture.data['im_coords'][-1], [.5, 0])
    copied = texture.copy()
    copied.shift(m.UP)
    np.testing.assert_array_equal(texture.get_points(), points)
    np.testing.assert_allclose(copied.get_points(), points + m.UP, atol=2e-7)
    # Python closures are not pickleable; use a default source for this contract.
    pickle_source = m.TexturedSurface(m.Surface(resolution=(2, 2)), path)
    restored = pickle.loads(pickle.dumps(pickle_source))
    np.testing.assert_array_equal(restored.data, pickle_source.data)
    assert restored._image_dimensions() == (2, 2)
    # Copy live source state even after adoption, without changing source identity.
    scene = m.Scene()
    scene.add(source)
    source.shift(m.LEFT)
    from_bound = m.TexturedSurface(source, path, opacity=.25, depth_test=False, z_index=3)
    np.testing.assert_array_equal(from_bound.get_points(), source.get_points())
    assert from_bound.get_opacity() == .25 and from_bound.z_index == 3
    class DrawOrder:
        calls = 0
        def __int__(self):
            self.calls += 1
            return self.calls + 4
    order = DrawOrder()
    partial_fixed = m.TexturedSurface(source, path, is_fixed_in_frame=.25, z_index=order)
    assert partial_fixed.uniforms['is_fixed_in_frame'] == .25
    assert order.calls == 1 and partial_fixed.z_index == 5
    refuses(ValueError, lambda: m.TexturedSurface(source, path, is_fixed_in_frame=float('nan')))
    uv_before = partial_fixed.data['im_coords'].copy()
    opacity_before = partial_fixed.get_opacities().copy()
    refuses(ValueError, lambda: partial_fixed.set_image_coords_by_uv_func(lambda u, v: (1e100, v)))
    refuses(ValueError, lambda: partial_fixed.set_opacity(1e100))
    np.testing.assert_array_equal(partial_fixed.data['im_coords'], uv_before)
    np.testing.assert_array_equal(partial_fixed.get_opacities(), opacity_before)
    scene.add(texture, from_bound)
    texture.data['im_coords'][:, 0] = .75
    np.testing.assert_allclose(texture.data['im_coords'][:, 0], .75)
    assert texture._image_dimensions() == (2, 2)
    # A second constructor cannot overwrite an already adopted object's resource.
    before = texture.data.copy()
    refuses(RuntimeError, lambda: texture.__init__(source, path))
    np.testing.assert_array_equal(texture.data, before)
    mesh = mesh_fixture()
    geometry = m.TexturedGeometry(mesh, path, opacity=.5, shading=(0, 0, 0))
    indices = mesh.faces.reshape(-1)
    assert geometry.n_records() == 6
    np.testing.assert_array_equal(geometry.get_points(), mesh.vertices[indices])
    np.testing.assert_allclose(geometry.data['im_coords'], mesh.visual.uv[indices] * [1, -1] + [0, 1])
    np.testing.assert_allclose(geometry.get_unit_normals(), np.tile([0, 0, 1], (6, 1)))
    np.testing.assert_array_equal(geometry.get_triangle_indices(), np.arange(6))
    assert geometry.copy()._image_dimensions() == (2, 2)
    old = geometry.get_points().copy()
    geometry.rotate(.3, axis=m.UP)
    assert not np.array_equal(geometry.get_points(), old)
    # Validation is not a successful-looking substitute for missing capabilities.
    refuses(TypeError, lambda: m.TexturedSurface(m.Circle(), path))
    paired = m.TexturedSurface(source, path, other)
    assert paired.num_textures == 2
    assert paired.dark_image_file == str(other.resolve())
    same = m.TexturedSurface(source, path, path)
    assert same.num_textures == 1
    refuses(TypeError, lambda: m.TexturedSurface(source, path, unknown_option=True))
    refuses(ValueError, lambda: m.TexturedSurface(source, path, opacity=float('nan')))
    bad = root / 'malformed.png'
    bad.write_bytes(b'not an image')
    refuses(ValueError, lambda: m.TexturedSurface(source, bad))
    invalid = mesh_fixture()
    invalid.faces[0, 0] = 99
    refuses(ValueError, lambda: m.TexturedGeometry(invalid, path))
    invalid = mesh_fixture()
    invalid.visual.uv[0, 0] = np.nan
    refuses(ValueError, lambda: m.TexturedGeometry(invalid, path))
    invalid = mesh_fixture()
    invalid.faces = invalid.faces.astype(float)
    refuses(TypeError, lambda: m.TexturedGeometry(invalid, path))
    invalid = mesh_fixture()
    invalid.vertices = np.zeros((65537, 3))
    invalid.visual.uv = np.zeros((65537, 2))
    refuses(ValueError, lambda: m.TexturedGeometry(invalid, path))
    print('textured surfaces: native records, topology, copy/pickle, bound sources and refusals passed')


def verify_textured_rendering():
    root = Path(tempfile.mkdtemp(prefix='fmn-texture-pixels-'))
    path = texture_fixture(root / 'colors.png')
    class Textures(m.Scene):
        default_camera_config = dict(resolution=(96, 54), fps=8)
        def construct(self):
            grid = m.Surface(u_range=(-2, 2), v_range=(-2, 2), resolution=(3, 3))
            surface = m.TexturedSurface(grid, path, shading=(0, 0, 0))
            mesh = m.TexturedGeometry(mesh_fixture(), path, shading=(0, 0, 0)).scale(.5).shift(4 * m.RIGHT)
            self.add(surface, mesh)
            self.wait(1 / 8)
            self.play(m.Rotate(surface, .6, axis=m.UP), run_time=1 / 4, rate_func=m.linear)
            surface.data['im_coords'][:] = [.5, .5]
            self.wait(1 / 8)
    sequences = []
    for threads in (1, 4, 16):
        result = Textures().render(root / str(threads), format='png_sequence', threads=threads)
        frames = [p.read_bytes() for p in sorted(result.destination.glob('*.png'))]
        assert result.frame_count == len(frames) == 4
        assert frames[0] != frames[1] != frames[2] and frames[2] != frames[3]
        sequences.append(frames)
    assert sequences[0] == sequences[1] == sequences[2]
    # Independent PNG decode proves texture orientation, not merely a moving silhouette.
    first = decode_png(sequences[0][0]).astype(int)
    assert first.shape == (54, 96, 4)
    for (y, x), channel in (((17, 38), 0), ((17, 58), 1), ((37, 38), 2)):
        color = first[y, x, :3]
        assert all(color[channel] > color[other] + 40 for other in range(3) if other != channel), color
    # A resource is frozen at decode, not read from its filename each frame.
    class Frozen(m.Scene):
        default_camera_config = dict(resolution=(96, 54), fps=8)
        def construct(self):
            texture = m.TexturedSurface(m.Sphere(resolution=(5, 7)), path)
            self.add(texture)
            self.wait(1 / 8)
            path.write_bytes(b'source changed after decoding')
            self.wait(1 / 8)
    result = Frozen().render(root / 'frozen', format='png_sequence', threads=1)
    frames = [p.read_bytes() for p in sorted(result.destination.glob('*.png'))]
    assert len(frames) == 2 and frames[0] == frames[1]
    texture_fixture(path)
    class Failed(Textures):
        def construct(self):
            super().construct()
            raise RuntimeError('texture generation cancelled')
    refuses(RuntimeError, lambda: Failed().render(root / 'failed', format='png_sequence'))
    assert not (root / 'failed').exists()
    protected = root / 'protected'
    protected.mkdir()
    (protected / 'sentinel').write_bytes(b'keep')
    refuses(Exception, lambda: Textures().render(protected, format='png_sequence'))
    assert list(protected.iterdir()) == [protected / 'sentinel']
    assert (protected / 'sentinel').read_bytes() == b'keep'
    print('textured surfaces: 4-frame PNG sequences match at 1/4/16 threads; frozen images and failure paths passed')
    return sequences[0][0], sequences[0][-1]


if __name__ == '__main__':
    verify_textured_surfaces()
    verify_textured_rendering()
