"""Real imperative Scene -> frozen Lumen jobs -> native ordered publications.

No renderer, native output method or scheduler is mocked. Synchronous Camera
readback supplies independent capture-time expectations for the delivered PNGs.
"""
from pathlib import Path
import hashlib
import struct
import sys
import tempfile
import threading
import zlib

import numpy as np
import manimlib as m


def png_pixels(path):
    payload = path.read_bytes()
    assert payload[:8] == b'\x89PNG\r\n\x1a\n'
    width, height, depth, color = struct.unpack_from('>IIBB', payload, 16)
    assert (depth, color) == (8, 6)
    at, packed = 8, bytearray()
    while at < len(payload):
        size = int.from_bytes(payload[at:at + 4], 'big')
        kind, body = payload[at + 4:at + 8], payload[at + 8:at + 8 + size]
        assert zlib.crc32(kind + body) == int.from_bytes(payload[at + 8 + size:at + 12 + size], 'big')
        if kind == b'IDAT':
            packed.extend(body)
        at += size + 12
    raw = zlib.decompress(packed)
    stride = width * 4
    assert len(raw) == height * (stride + 1)
    previous, rows = bytearray(stride), []
    for y in range(height):
        start = y * (stride + 1)
        mode = raw[start]
        row = bytearray(raw[start + 1:start + 1 + stride])
        assert mode in range(5)
        for x in range(stride):
            left = row[x - 4] if x >= 4 else 0
            up = previous[x]
            corner = previous[x - 4] if x >= 4 else 0
            if mode == 0:
                prediction = 0
            elif mode == 1:
                prediction = left
            elif mode == 2:
                prediction = up
            elif mode == 3:
                prediction = (left + up) // 2
            else:
                base = left + up - corner
                prediction = min((left, up, corner), key=lambda value: abs(base - value))
            row[x] = (row[x] + prediction) & 255
        rows.append(bytes(row))
        previous = row
    return np.frombuffer(b''.join(rows), dtype=np.uint8).reshape(height, width, 4)


def capture_time_pixels(root):
    outputs = []
    for threads in (1, 4, 16):
        scene = m.Scene()
        expected = []
        directory = root / f'frames-{threads}'
        with scene.render_session(directory, format='png_sequence', resolution=(96, 54),
                                  fps=8, threads=threads) as session:
            shape = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
            label = m.Text('Frozen').scale(.3).shift(2 * m.UP)
            scene.add(shape, label)
            points = shape.get_points()
            initial = points.copy()
            for index in range(12):
                points[:] = initial
                points[:, 0] += -2 + index * .35
                scene.frame.move_to((index * .03, index * -.015, 0))
                scene.camera.background_rgba[:] = ((index % 3) * .04, 0, .03, 1)
                scene.camera.capture(*scene.mobjects)
                expected.append(scene.camera.get_pixel_array().copy())
                scene.wait(1 / 8)
            points[:, 1] += 30
            scene.set_background_color(m.RED)
        assert scene.get_time() == 12 / 8
        assert session.result.frame_count == 12
        frames = sorted(directory.glob('*.png'))
        assert len(frames) == len(expected)
        for frame, reference in zip(frames, expected):
            np.testing.assert_array_equal(png_pixels(frame), reference,
                err_msg=f'capture-time camera/view state was lost: {frame}')
        outputs.append([hashlib.sha256(frame.read_bytes()).digest() for frame in frames])
    assert outputs[0] == outputs[1] == outputs[2], 'thread scheduling changed canonical PNGs'


def real_render_releases_the_gil(root):
    scene = m.Scene()
    shape = m.Square(fill_opacity=1)
    ready, start, progressed = threading.Event(), threading.Event(), threading.Event()
    def observer():
        ready.set()
        start.wait()
        progressed.set()
    worker = threading.Thread(target=observer)
    worker.start()
    assert ready.wait(5)
    previous = sys.getswitchinterval()
    try:
        # Disallow incidental bytecode-timeslice yielding. Only a real native
        # wait in this render may let the waiting Python observer make progress.
        sys.setswitchinterval(60)
        with scene.render_session(root / 'gil', format='png_sequence',
                                  resolution=(96, 54), fps=8, threads=2):
            scene.add(shape)
            start.set()
            assert not progressed.is_set(), 'observer ran before native render entry'
            scene.wait(2 / 8)
            assert progressed.is_set(), 'real render retained the GIL across native waits'
    finally:
        sys.setswitchinterval(previous)
        start.set()
        worker.join(5)
    assert not worker.is_alive()


def failure_cancels_all_workers_without_partial_publication(root):
    for exception in (RuntimeError('authored pipeline failure'), KeyboardInterrupt('pipeline interrupted')):
        destination = root / type(exception).__name__
        scene = m.Scene()
        try:
            with scene.render_session(destination, format='png_sequence',
                                      resolution=(96, 54), fps=8, threads=4):
                shape = m.Circle(fill_opacity=1)
                scene.add(shape)
                scene.play(shape.animate.shift(m.RIGHT), run_time=1)
                raise exception
        except BaseException as caught:
            assert caught is exception, 'cancellation replaced the authored failure'
        else:
            raise AssertionError('failed scene published')
        assert not destination.exists()
        fresh = m.Scene()
        with fresh.render_session(destination, format='png_sequence',
                                  resolution=(96, 54), fps=8, threads=1) as session:
            fresh.add(m.Square(fill_opacity=1))
            fresh.wait(1 / 8)
        assert session.result.frame_count == 1, 'failed generation leaked into its successor'


def late_publication_conflict_keeps_existing_artifact(root):
    destination = root / 'competing.y4m'
    scene = m.Scene()
    marker = b'existing unrelated artifact'
    try:
        with scene.render_session(destination, format='y4m', resolution=(96, 54), fps=8, threads=4):
            scene.add(m.Circle(fill_opacity=1))
            scene.wait(4 / 8)
            destination.write_bytes(marker)
    except (RuntimeError, OSError, ValueError):
        pass
    else:
        raise AssertionError('publication overwrote a competing artifact')
    assert destination.read_bytes() == marker


def independent_generations_do_not_share_worker_state(root):
    a, b = m.Scene(), m.Scene()
    with a.render_session(root / 'red', format='png_sequence', resolution=(96, 54), fps=8, threads=2) as sa:
        with b.render_session(root / 'blue', format='png_sequence', resolution=(96, 54), fps=8, threads=2) as sb:
            a.add(m.Square(fill_color=m.RED, fill_opacity=1, stroke_width=0))
            b.add(m.Square(fill_color=m.BLUE, fill_opacity=1, stroke_width=0))
            for _ in range(6):
                a.wait(1 / 8)
                b.wait(1 / 8)
    assert sa.result.frame_count == sb.result.frame_count == 6
    red = png_pixels(sorted(sa.result.destination.glob('*.png'))[-1])[27, 48].astype(int)
    blue = png_pixels(sorted(sb.result.destination.glob('*.png'))[-1])[27, 48].astype(int)
    assert red[0] > red[2] and blue[2] > blue[0]


def run_pipeline_acceptance():
    cases = (capture_time_pixels, real_render_releases_the_gil,
             failure_cancels_all_workers_without_partial_publication,
             late_publication_conflict_keeps_existing_artifact,
             independent_generations_do_not_share_worker_state)
    with tempfile.TemporaryDirectory(prefix='fmn-frame-pipeline-') as directory:
        for case in cases:
            root = Path(directory) / case.__name__
            root.mkdir()
            case(root)
            print('native frame pipeline passed:', case.__name__, flush=True)


run_pipeline_acceptance()
