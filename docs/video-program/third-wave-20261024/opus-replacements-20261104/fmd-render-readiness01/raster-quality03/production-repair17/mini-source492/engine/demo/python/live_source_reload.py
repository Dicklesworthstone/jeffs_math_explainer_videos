"""Edit this file while a live native scene stays in its host IPython session.

    python demo/python/live_source_reload.py

At the prompt, call auto_reload(), then place(square). Edit the offset in
place below, save the file, and call place(square) again. The same square moves
using the new definition; the scene, clock and checkpoints are not recreated.
reload_source() explicitly refreshes definitions even without autoreload.
"""
from manimlib import RIGHT, Scene, Square
from fmn_python import embed_scene
from fmn_python.scene_loading import SceneSource


def place(square):
    square.move_to(1.5 * RIGHT)


if __name__ == "__main__":
    scene = Scene()
    square = Square()
    scene.add(square)
    with SceneSource(__file__, Scene) as loaded:
        embed_scene(scene, dict(vars(loaded.module), scene=scene, square=square))
