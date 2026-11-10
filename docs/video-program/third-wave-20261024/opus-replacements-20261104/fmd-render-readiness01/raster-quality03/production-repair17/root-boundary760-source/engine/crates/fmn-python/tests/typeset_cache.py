"""Real-extension cache acceptance; also invoked by the Rust portal test.

Run directly against an installed wheel for the additional production PNG
comparison. No layout/renderer substitute is used by this suite.
"""
from contextlib import redirect_stderr
import io
from pathlib import Path
import tempfile

import numpy as np


def run_typeset_cache(m):
    configure = m._fmn_configure_tex_cache
    info = m._fmn_tex_cache_info
    source = r"\sum_{k=1}^{3} k = 6"
    with tempfile.TemporaryDirectory(prefix="fmn-typeset-cache-") as directory:
        root = Path(directory)
        cache = root / "owned-cache"
        report = configure(cache)
        assert all(row["persistent"] for row in report["templates"].values()), report
        first = m.Tex(source)
        expected = first.get_all_points().copy()
        assert expected.size > 0
        cold = info()["templates"]["default"]
        assert cold["layout_computations"] > 0
        assert (cache / "STORE_OWNER").is_file()
        assert any((cache / "ns" / "typeset").rglob("objects/*/*"))

        # Explicit reattachment clears the resident front without replacing
        # the engine. A second construction must then prove actual disk reuse.
        configure(cache)
        warm = m.Tex(source)
        after = info()["templates"]["default"]
        np.testing.assert_array_equal(warm.get_all_points(), expected)
        assert after["layout_computations"] == cold["layout_computations"], (cold, after)
        assert after["disk_hits"] > cold["disk_hits"], (cold, after)
        m.Tex(source)
        resident = info()["templates"]["default"]
        assert resident["memory_hits"] > after["memory_hits"]

        configure(enabled=False)
        uncached = m.Tex(source)
        np.testing.assert_array_equal(uncached.get_all_points(), expected)
        assert all(not row["persistent"] for row in info()["templates"].values())
        assert (cache / "STORE_OWNER").is_file(), "disable must not delete stored data"

        sentinel = root / "foreign-data.txt"
        sentinel.write_bytes(b"not cache data")
        with redirect_stderr(io.StringIO()) as diagnostic:
            refused = configure(root)
        assert all(not row["persistent"] for row in refused["templates"].values()), refused
        assert any(row["error"] for row in refused["templates"].values())
        assert "typeset-cache-unavailable" in diagnostic.getvalue()
        np.testing.assert_array_equal(m.Tex(source).get_all_points(), expected)
        assert sentinel.read_bytes() == b"not cache data"
        configure(enabled=False)


def run_rendered_cache_acceptance(m):
    from fmn_python import render_scene
    from fmn_python.typesetting import configure_tex_cache

    class Formula(m.Scene):
        def construct(self):
            self.add(m.Tex(r"\int_0^1 x^2\,dx = \frac{1}{3}"))

    with tempfile.TemporaryDirectory(prefix="fmn-cache-render-") as directory:
        root = Path(directory)
        outputs = []
        for name, enabled in (("cold", True), ("warm", True), ("disabled", False)):
            configure_tex_cache(root / "cache", enabled=enabled)
            output = root / (name + ".png")
            result = render_scene(Formula, output, format="png", resolution=(160, 90),
                                  fps=30, threads=1)
            assert result.frame_count == 1
            outputs.append(output.read_bytes())
        assert outputs[0].startswith(b"\x89PNG\r\n\x1a\n")
        assert outputs[0] == outputs[1] == outputs[2], "cache warmth changed production PNG bytes"
        configure_tex_cache(enabled=False)


if __name__ in ("__main__", "<run_path>"):
    import manimlib as m
    run_typeset_cache(m)
    run_rendered_cache_acceptance(m)
    print("native typeset cache and cold/warm/disabled PNG acceptance passed")
