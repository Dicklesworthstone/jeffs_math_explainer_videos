"""Apply the two native registration hooks and their production installer.

This guarded integration exists because the connector writes whole files,
while lib.rs and bridge.py are large and concurrently edited. No unrelated
source is regenerated, no file is removed, and missing anchors fail closed.
"""
from pathlib import Path


def replace_once(path, old, new):
    path = Path(path)
    source = path.read_text()
    if (new and new in source) or (not new and old not in source):
        return
    if source.count(old) != 1:
        raise RuntimeError(f'{path}: expected exactly one integration anchor')
    path.write_text(source.replace(old, new, 1))


replace_once('crates/fmn-python/src/lib.rs',
             'mod portal_studio;\n',
             'mod portal_readback;\nmod portal_studio;\n')
replace_once('crates/fmn-python/src/lib.rs',
             '    module.add_class::<PyCameraCore>()?;\n    module.add_class::<PyFieldProbe>()?;',
             '    module.add_class::<PyCameraCore>()?;\n    module.add_class::<portal_readback::CameraCapture>()?;\n    module.add_class::<PyFieldProbe>()?;')
replace_once('crates/fmn-python/python/fmn_python/initialization.py',
             '    ("rendering", "install_scene_rendering"),\n',
             '    ("rendering", "install_scene_rendering"),\n    ("camera_capture", "install_camera_capture"),\n')
replace_once('crates/fmn-python/tests/bridge.py',
             '    "clear": lambda: _cam.clear(),\n', '')
replace_once('crates/fmn-python/tests/bridge.py',
'''_capture_refused = False
try:
    _cam.capture()
except Exception as error:
    _capture_refused = "Lumen" in str(error) and "run" in str(error)
_img_refused = False
try:
    _cam.get_image()
except Exception as error:
    _img_refused = "Lumen" in str(error)
_check("capture and get image refuse toward native png",
       _capture_refused and _img_refused)
_px_refused = False
try:
    _cam.get_pixel_array()
except Exception as error:
    _px_refused = "Lumen" in str(error)
_check("pixel array access refuses toward native png", _px_refused)''',
'''_cam.clear()
_cam.capture()
_pixels = _cam.get_pixel_array()
_check("camera readback is real RGBA", _pixels.shape == (720, 1280, 4)
       and _pixels.dtype == np.uint8)
_img_refused = False
try:
    _cam.get_image()
except Exception as error:
    _img_refused = "Lumen" in str(error)
_check("Pillow image surface remains separate", _img_refused)''')
