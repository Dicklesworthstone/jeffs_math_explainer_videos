"""Real recorded console cells with the default native camera preview enabled.

The same SceneState, mobjects, readback and Reel generation are used by normal
cells and record_to cells. Preview is observational, never an extra video frame.
"""
from pathlib import Path
import hashlib
import tempfile

import numpy as np
import manimlib as m
from fmn_python import SceneConsole


def run_recording_preview_acceptance():
    scene = m.Scene()
    scene.camera.reset_pixel_shape(96, 54)
    square = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
    square.shift(2 * m.LEFT)
    scene.add(square)
    scene.wait(0.125)
    identity, camera = scene.mobjects[0], scene.camera
    options = {"resolution": (96, 54), "threads": 1}
    with tempfile.TemporaryDirectory(prefix="fmn-recorded-preview-") as temporary:
        root = Path(temporary)
        with SceneConsole(scene, {"square": square, "RIGHT": m.RIGHT}) as editor:
            editor.run_cell("pass")
            before = scene.camera.get_pixel_array()
            assert before.shape == (54, 96, 4)
            initial_center = np.nonzero(before[:, :, 0] > 200)[1].mean()
            assert np.isfinite(initial_center) and initial_center < 48
            source = "# recorded motion\nscene.play(square.animate.shift(3 * RIGHT), run_time=0.125)"
            rendered = []
            for index in range(2):
                destination = root / f"take-{index}.y4m"
                editor.run_cell(source, record_to=destination, recording_options=options)
                receipt = editor.last_recording
                assert receipt.frame_count == 4 and not receipt.certified
                payload = destination.read_bytes()
                assert receipt.digest == hashlib.sha256(payload).hexdigest()
                assert receipt.bytes == len(payload)
                rendered.append(payload)
                assert m._portal_scene_clock(scene) == (30, 8)
                assert scene.mobjects[0] is identity and scene.camera is camera
                preview = scene.camera.get_pixel_array()
                assert preview.shape == before.shape
                center = np.nonzero(preview[:, :, 0] > 200)[1].mean()
                assert np.isfinite(center) and center > 48 > initial_center
                # Refresh after the clip cannot append frames or advance the clock.
                again = scene.camera.capture(*scene.mobjects)
                assert again is None and m._portal_scene_clock(scene) == (30, 8)
                np.testing.assert_array_equal(preview, scene.camera.get_pixel_array())
            assert rendered[0] == rendered[1], "a preview or checkpoint retry changed video bytes"
            good = editor.last_recording
            stop = KeyboardInterrupt("stop a recorded preview cell")
            editor.namespace["stop"] = stop
            failed = root / "failed.y4m"
            try:
                editor.run_cell("# recorded motion\nscene.wait(0.125)\nraise stop",
                                record_to=failed, recording_options=options)
            except KeyboardInterrupt as error:
                assert error is stop
            else:
                raise AssertionError("recorded preview swallowed cancellation")
            assert not failed.exists() and editor.last_recording is good
            assert scene.camera.get_pixel_array().shape == before.shape
            editor.run_cell(source, record_to=root / "recovered.y4m", recording_options=options)
            assert (root / "recovered.y4m").read_bytes() == rendered[0]
        assert vars(scene).get("_fmn_scene_console") is None
        assert vars(scene).get("_fmn_owned_render_session") is None
    print("native recorded console preview passed: retries, four-frame output, camera identity and cancellation")


run_recording_preview_acceptance()
