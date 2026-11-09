"""Actual native light/dark material pixels, durability and failure semantics."""
from pathlib import Path
import pickle
import struct
import tempfile
import zlib

import numpy as np
import manimlib as m

# The embedded witness installs the same helper source into its globals;
# the installed-wheel invocation imports its adjacent acceptance module.
if "decode_png" not in globals():
    from textured_surfaces import decode_png, refuses


def solid_texture(path, rgba, size=(2, 2)):
    def chunk(kind, body):
        return (struct.pack('>I', len(body)) + kind + body
                + struct.pack('>I', zlib.crc32(kind + body)))
    width, height = size
    raw = (b'\0' + bytes(rgba) * width) * height
    path.write_bytes(b'\x89PNG\r\n\x1a\n'
        + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
        + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))
    return path


def verify_texture_pairs():
    with tempfile.TemporaryDirectory(prefix='fmn-texture-pairs-') as directory:
        root = Path(directory)
        light = solid_texture(root / 'light.png', (255, 0, 0, 255))
        dark = solid_texture(root / 'dark.png', (0, 0, 255, 255), (3, 1))
        source = m.Surface(u_range=(-2, 2), v_range=(-2, 2), resolution=(3, 3))
        original = m.TexturedSurface(source, light, dark, shading=(0, 0, 0))
        assert original.num_textures == 2
        assert original.image_file == str(light.resolve())
        assert original.dark_image_file == str(dark.resolve())
        np.testing.assert_array_equal(original.get_points(), source.get_points())
        copied = original.copy()
        serialized = pickle.dumps(original)
        same = m.TexturedSurface(source, light, light)
        assert same.num_textures == 1

        bad = root / 'bad.png'
        bad.write_bytes(b'not a decodable texture')
        before = source.get_points().copy()
        refuses(ValueError, lambda: m.TexturedSurface(source, light, bad))
        # Reinitialization failure must not replace the already decoded pair.
        refuses(ValueError, lambda: copied.__init__(source, light, bad))
        np.testing.assert_array_equal(source.get_points(), before)
        assert copied.dark_image_file == str(dark.resolve())

        # Neither replay nor unpickling is permitted to reopen either filename.
        light.unlink()
        dark.unlink()
        restored = pickle.loads(serialized)
        assert restored.num_textures == 2
        np.testing.assert_array_equal(restored.data, original.data)

        def still(template, name, flipped=False, opacity=1):
            class Still(m.Scene):
                # The half-opacity check below reads "no red" against a black
                # field; the default #333333 would contribute red 25.
                default_camera_config = {"background_color": "#000000"}

                def construct(self):
                    surface = template.copy().set_opacity(opacity)
                    if flipped:
                        surface.rotate(m.PI, axis=m.UP)
                    self.add(surface)
            output = root / (name + '.png')
            Still().render(output, resolution=(96, 54), fps=8, threads=1)
            return decode_png(output.read_bytes())[27, 48].astype(int)

        for index, template in enumerate((original, copied, restored)):
            lit = still(template, f'lit-{index}')
            unlit = still(template, f'dark-{index}', flipped=True)
            assert lit[0] > 240 and lit[2] < 10, lit
            assert unlit[2] > 240 and unlit[0] < 10, unlit
        faded = still(restored, 'opacity', flipped=True, opacity=.5)
        assert 100 < faded[2] < 230 and faded[0] < 10, faded

        class Turn(m.Scene):
            def construct(self):
                surface = restored.copy()
                self.add(surface)
                self.wait(1 / 8)
                self.play(m.Rotate(surface, m.PI, axis=m.UP), run_time=1 / 4,
                          rate_func=m.linear)
                self.wait(1 / 8)
        sequences = []
        for threads in (1, 4, 16):
            result = Turn().render(root / f'turn-{threads}', format='png_sequence',
                                   resolution=(96, 54), fps=8, threads=threads)
            frames = [p.read_bytes() for p in sorted(result.destination.glob('*.png'))]
            assert result.frame_count == len(frames) == 4
            sequences.append(frames)
        assert sequences[0] == sequences[1] == sequences[2]
        assert decode_png(sequences[0][0])[27, 48, 0] > 240
        assert decode_png(sequences[0][-1])[27, 48, 2] > 240
        assert sequences[0][0] != sequences[0][-1]

        class Failed(Turn):
            def construct(self):
                super().construct()
                raise RuntimeError('texture pair cancelled')
        refuses(RuntimeError, lambda: Failed().render(root / 'failed',
                format='png_sequence', resolution=(96, 54), fps=8, threads=1))
        assert not (root / 'failed').exists()
        print('light/dark textures: native pixels, copy/pickle, deleted sources, opacity, '
              '1/4/16-thread animation and cancellation passed', flush=True)


if __name__ == '__main__':
    verify_texture_pairs()
