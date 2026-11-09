"""The portal's automatic static Tex preflight and config-selected cache.

Invoked by the Rust portal test against the real extension; also runnable
directly against an installed wheel. No layout or render substitute is used.
"""
from contextlib import redirect_stderr
import gc
import importlib.util
import io
import os
from pathlib import Path
import sys
import tempfile
import threading

FORMULAS = [
    r"e^{i\pi} + 1 = 0",
    r"a^2 + b^2 = c^2",
    r"\frac{a}{b} + \frac{c}{d} = \frac{ad + bc}{bd}",
    r"x_{1,2} = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}",
    r"\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}",
    r"\int_0^1 x^2 \, dx = \frac{1}{3}",
    r"\prod_{k=1}^{n} k = n!",
    r"\lim_{h \to 0} \frac{f(x+h) - f(x)}{h}",
    r"\binom{n}{k} = \frac{n!}{k!\,(n-k)!}",
    r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
    r"f(x) = \begin{cases} x & x > 0 \\ -x & x \le 0 \end{cases}",
    r"\sqrt[3]{x + 1}",
    r"\nabla \cdot \mathbf{E} = \frac{\rho}{\varepsilon_0}",
    r"\mathbb{E}[X] = \sum_x x \, p(x)",
    r"\sigma^2 = \mathbb{E}\left[(X - \mu)^2\right]",
    r"P(A \mid B) = \frac{P(B \mid A)\, P(A)}{P(B)}",
    r"\left| \sum_i a_i b_i \right| \le \sqrt{\sum_i a_i^2} \sqrt{\sum_i b_i^2}",
    r"\hat{x} + \overline{AB}",
    r"\bar{X}_n = \frac{1}{n} \sum_{i=1}^{n} X_i",
    r"P\left(|\bar{X}_n - \mu| \ge t\right) \le 2 e^{-2 n t^2}",
]


def _scene_source():
    calls = ",\n            ".join(f"Tex({formula!r}, font_size=20)" for formula in FORMULAS)
    return f'''from manimlib import *


class TwentyStatic(Scene):
    def construct(self):
        sheet = VGroup(
            {calls},
            TexText("Hoeffding's inequality", font_size=20),
        ).arrange(DOWN, buff=0.05)
        dynamic = "x^{{" + str(7) + "}}"
        self.add(sheet, Tex(dynamic))
        self.play(sheet.animate.shift(0.1 * UP), run_time=0.2)
'''


def _load(directory):
    path = Path(directory) / "fmn_static_preflight_scene.py"
    path.write_text(_scene_source(), encoding="utf-8")
    spec = importlib.util.spec_from_file_location("fmn_static_preflight_scene", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _render(m, scene_type, output):
    from fmn_python import render_scene

    scene = scene_type()
    result = render_scene(scene, output, format="png", resolution=(160, 90), fps=10, threads=2)
    assert result.frame_count == 1, result
    return scene, Path(output).read_bytes()


def run_static_preflight(m):
    from fmn_python.typesetting import configure_static_tex_preflight, configure_tex_cache

    info = m._fmn_tex_cache_info
    with tempfile.TemporaryDirectory(prefix="fmn-static-preflight-") as directory:
        root = Path(directory)
        module = _load(root)
        try:
            # Cold: the twenty literal formulas and the TexText are found in
            # the scene's own source and typeset on the pool before setup().
            configure_tex_cache(root / "cache")
            scene, cold_png = _render(m, module.TwentyStatic, root / "cold.png")
            report = scene._fmn_static_tex_preflight
            assert report["enabled"] and report["discovered"] == 21, report
            assert report["requests"] == 21 and report["succeeded"] == 21, report
            assert report["failed"] == 0 and report["wall_ns"] > 0, report
            # Math and text are separate request groups.
            assert report["batches"] == 2, report
            before, after = report["before"]["default"], report["after"]["default"]
            preflighted = after["layout_computations"] - before["layout_computations"]
            # 21 strings plus the shared scale-calibration probe per batch;
            # the probe is laid out once.
            assert preflighted == 22, (before, after)
            workers = after["preflight_workers"]
            if (os.cpu_count() or 1) >= 2:
                assert workers >= 2 and after["preflight_active_workers"] >= 2, after
            # Construction and play laid out only the one dynamic string.
            done = info()["templates"]["default"]
            assert done["layout_computations"] - after["layout_computations"] == 1, (after, done)

            # Warm, fresh binding: every static string is a verified disk hit
            # and the rendered PNG is byte-identical to the cold one.
            configure_tex_cache(root / "cache")
            scene, warm_png = _render(m, module.TwentyStatic, root / "warm.png")
            report = scene._fmn_static_tex_preflight
            before, after = report["before"]["default"], report["after"]["default"]
            assert after["layout_computations"] == before["layout_computations"], report
            assert after["disk_hits"] - before["disk_hits"] == 22, report
            assert warm_png == cold_png, "a warm typeset cache changed the PNG"

            # Planted negative: with the preflight off, the same scene's
            # construction lays out every formula itself.
            configure_static_tex_preflight(False)
            configure_tex_cache(root / "other-cache")
            start = info()["templates"]["default"]["layout_computations"]
            scene, off_png = _render(m, module.TwentyStatic, root / "off.png")
            report = scene._fmn_static_tex_preflight
            assert not report["enabled"] and report["discovered"] == 0, report
            done = info()["templates"]["default"]["layout_computations"]
            assert done - start >= 22, (start, done)
            assert off_png == cold_png, "the preflight changed the PNG"
        finally:
            configure_static_tex_preflight(True)
            configure_tex_cache(enabled=False)
            sys.modules.pop("fmn_static_preflight_scene", None)


_MIXED_SOURCE = '''from manimlib import *


class LabelledBase(Scene):
    def label(self):
        return Tex(r"a^2 + b^2"), TexText("Pythagoras")


class Annotating:
    """A plain mixin, listed ahead of the scene base below."""

    def note(self):
        return Tex(r"\\sqrt{2}")


class Mixed(Annotating, LabelledBase):
    def construct(self):
        self.add(*self.label(), self.note(), Tex(r"c^2"))


class Trailing(LabelledBase, Annotating):
    """The mixin listed after the scene base: its MRO slot is behind Scene."""

    def construct(self):
        self.add(*self.label(), self.note())


class Upper(LabelledBase):
    def upper(self):
        return Tex(r"u^2"), Tex(r"a^2 + b^2")


class Lower(LabelledBase):
    def lower(self):
        return Tex(r"l^2")


class Diamond(Upper, Lower):
    def construct(self):
        self.add(*self.upper(), *self.lower(), *self.label())
'''

_EDITED_LATER = '''from manimlib import *


class Edited(Scene):
    def construct(self):
        self.add(Tex(r"e^x"))
'''


def _load_named(directory, name, source):
    path = Path(directory) / (name + ".py")
    path.write_text(source, encoding="utf-8")
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module, path


def run_discovery_robustness(m):
    """Mixins never hide scene bases; a source edited since import never
    fails a render (Python 3.13+ raises tokenize.TokenError there)."""
    import linecache
    from fmn_python.typesetting import _static_tex_requests

    with tempfile.TemporaryDirectory(prefix="fmn-static-discovery-") as directory:
        try:
            module, _ = _load_named(directory, "fmn_static_mixed_scene", _MIXED_SOURCE)
            sources = sorted(source for source, *_ in _static_tex_requests(module.Mixed, m))
            # The mixin's string and both base strings, not only Mixed's own.
            assert sources == sorted([r"c^2", r"\sqrt{2}", r"a^2 + b^2", "Pythagoras"]), sources
            mro = [cls.__name__ for cls in module.Trailing.__mro__]
            assert mro.index("Annotating") > mro.index("Scene"), mro
            sources = sorted(source for source, *_ in _static_tex_requests(module.Trailing, m))
            assert sources == sorted([r"\sqrt{2}", r"a^2 + b^2", "Pythagoras"]), sources
            # A diamond reads its shared base once, and a string two classes
            # construct is one request.
            sources = [source for source, *_ in _static_tex_requests(module.Diamond, m)]
            assert sorted(sources) == sorted([r"u^2", r"a^2 + b^2", r"l^2", "Pythagoras"]), sources

            module, path = _load_named(directory, "fmn_static_edited_scene", _EDITED_LATER)
            assert [source for source, *_ in _static_tex_requests(module.Edited, m)] == ["e^x"]
            # Mid-edit on disk: an unterminated string inside the class.
            path.write_text(_EDITED_LATER.replace('Tex(r"e^x")', 'Tex("""e^x'), encoding="utf-8")
            linecache.checkcache(str(path))
            assert _static_tex_requests(module.Edited, m) == []
            # The loaded class still renders; the preflight just finds nothing.
            scene, _png = _render(m, module.Edited, Path(directory) / "edited.png")
            assert scene._fmn_static_tex_preflight["discovered"] == 0
        finally:
            for name in ("fmn_static_mixed_scene", "fmn_static_edited_scene"):
                sys.modules.pop(name, None)


def run_store_errors_reported(m):
    """An attached store that can neither read nor write is reported."""
    from fmn_python.typesetting import configure_tex_cache, describe_receipt, typesetting_receipt

    with tempfile.TemporaryDirectory(prefix="fmn-static-store-errors-") as directory:
        root = Path(directory)
        module, _ = _load_named(root, "fmn_static_store_scene",
                                _EDITED_LATER.replace("Edited", "Stored"))
        try:
            configure_tex_cache(root / "cache")
            scene, cold_png = _render(m, module.Stored, root / "cold.png")
            healthy = typesetting_receipt(scene)
            assert healthy["store_errors"] == 0 and healthy["bytes_written"] > 0, healthy
            assert healthy["cache_error"] is None, healthy
            assert describe_receipt(healthy).startswith("; typeset cache: "), healthy
            # The namespace's object directory becomes a plain file: the root
            # still opens (persistent), but nothing can be read or written.
            (objects,) = (root / "cache" / "ns" / "typeset").glob("v*/objects")
            objects.rename(objects.with_name("objects.moved"))
            objects.write_bytes(b"not a directory")
            configure_tex_cache(root / "cache")
            scene, broken_png = _render(m, module.Stored, root / "broken.png")
            receipt = typesetting_receipt(scene)
            # The literal and the scale probe each failed one read and one
            # write in the preflight; construction then hit the memory front.
            assert receipt["persistent"] and receipt["store_errors"] == 4, receipt
            assert receipt["disk_hits"] == 0 and receipt["bytes_written"] == 0, receipt
            assert "cache reads or writes failed" in describe_receipt(receipt)
            assert broken_png == cold_png, "a store that cannot cache changed the PNG"

            # A root the cache refuses (an existing folder that is no store)
            # is never attached: like fmn's record and line, the receipt
            # names the refusal (`cache_error`) instead of a bare memory cache.
            foreign = root / "foreign"
            foreign.mkdir()
            (foreign / "notes.txt").write_bytes(b"not cache data")
            with redirect_stderr(io.StringIO()) as diagnostic:
                configure_tex_cache(foreign)
            assert "typeset-cache-unavailable" in diagnostic.getvalue()
            scene, refused_png = _render(m, module.Stored, root / "refused.png")
            receipt = typesetting_receipt(scene)
            assert not receipt["persistent"] and receipt["store_errors"] == 0, receipt
            assert isinstance(receipt["cache_error"], str) and receipt["cache_error"], receipt
            line = describe_receipt(receipt)
            assert line.startswith(f"; typeset cache unavailable ({receipt['cache_error']}): "), line
            assert refused_png == cold_png, "a refused root changed the PNG"
            assert (foreign / "notes.txt").read_bytes() == b"not cache data"
        finally:
            configure_tex_cache(enabled=False)
            sys.modules.pop("fmn_static_store_scene", None)


def run_refused_config_cache(m):
    """A ``directories.cache`` the cache refuses never fails a Scene or Tex:
    the thread typesets in memory and its report says why."""
    directories = m.manim_config["directories"]
    previous = directories.get("cache")
    directories["cache"] = os.path.join("..", "fmn-refused-cache")
    outcome = {}

    def build():
        try:
            m._fmn_ensure_tex_cache()
            row = m._fmn_tex_cache_info()["templates"]["default"]
            outcome["persistent"] = row["persistent"]
            outcome["error"] = row["error"]
            receipt = m._preflight_tex([r"\frac{1}{2}"])
            outcome["succeeded"] = receipt["succeeded"]
        except BaseException as error:  # surfaced below
            outcome["raised"] = repr(error)
        finally:
            m._fmn_configure_tex_cache(enabled=False)

    gc.collect()
    gc.disable()
    try:
        worker = threading.Thread(target=build)
        worker.start()
        worker.join()
    finally:
        gc.enable()
        directories["cache"] = previous
    assert "raised" not in outcome, outcome
    assert outcome["persistent"] is False and outcome["succeeded"] == 1, outcome
    assert "refused" in outcome["error"] and "parent-directory" in outcome["error"], outcome


def run_config_selected_cache(m):
    """``directories.cache`` from the config selects a fresh thread's store."""
    with tempfile.TemporaryDirectory(prefix="fmn-config-cache-") as directory:
        cache = Path(directory) / "configured"
        directories = m.manim_config["directories"]
        previous = directories.get("cache")
        directories["cache"] = str(cache)
        outcome = {}

        def build():
            # A new host thread has no binding yet: its first typesetting
            # binds the configured store. Only native typesetting runs here;
            # portal mobjects are unsendable and must stay on their thread.
            try:
                m._fmn_ensure_tex_cache()
                receipt = m._preflight_tex([r"\frac{1}{2}"])
                outcome["succeeded"] = receipt["succeeded"]
                outcome["persistent"] = receipt["after"]["persistent"]
            except BaseException as error:  # surfaced below
                outcome["error"] = repr(error)
            finally:
                m._fmn_configure_tex_cache(enabled=False)

        # Collect this thread's garbage first and keep the collector off while
        # the other thread runs, so no unsendable family is freed over there.
        gc.collect()
        gc.disable()
        try:
            worker = threading.Thread(target=build)
            worker.start()
            worker.join()
        finally:
            gc.enable()
            directories["cache"] = previous
        assert "error" not in outcome, outcome
        assert outcome["persistent"] is True and outcome["succeeded"] == 1, outcome
        assert (cache / "STORE_OWNER").is_file()
        assert any((cache / "ns" / "typeset").rglob("objects/*/*"))


if __name__ in ("__main__", "<run_path>"):
    import manimlib as m
    run_static_preflight(m)
    run_discovery_robustness(m)
    run_store_errors_reported(m)
    run_config_selected_cache(m)
    run_refused_config_cache(m)
    print("static Tex preflight and config-selected cache acceptance passed")
