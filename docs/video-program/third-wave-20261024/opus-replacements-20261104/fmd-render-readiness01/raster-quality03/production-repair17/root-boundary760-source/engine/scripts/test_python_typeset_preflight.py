"""Host argument-boundary tests; native acceptance is tests/typeset_preflight.py."""
from importlib.util import module_from_spec, spec_from_file_location
import itertools
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SOURCE = (Path(__file__).resolve().parents[1] / "crates/fmn-python/python/fmn_python/typesetting.py")
spec = spec_from_file_location("fmn_typesetting_under_test", SOURCE)
typesetting = module_from_spec(spec)
spec.loader.exec_module(typesetting)


class PreflightArguments(unittest.TestCase):
    def test_generator_is_owned_before_one_native_submission(self):
        events = []
        def sources():
            for source in (r"x^2", r"\frac{1}{2}"):
                events.append(("source", source))
                yield source
        receipt = {"count": 2}
        def submit(batch, **options):
            events.append(("native", batch, options))
            return receipt
        native = SimpleNamespace(_fmn_ensure_tex_cache=lambda: events.append("ensure"),
                                 _preflight_tex=submit)
        with patch.object(typesetting, "import_module", return_value=native):
            result = typesetting.preflight_tex(sources(), max_workers=4)
        self.assertIs(result, receipt)
        self.assertEqual(events[:3], [("source", r"x^2"), ("source", r"\frac{1}{2}"), "ensure"])
        self.assertEqual(events[3][1], [r"x^2", r"\frac{1}{2}"])
        self.assertEqual(events[3][2]["max_workers"], 4)

    def test_invalid_inputs_never_import_or_configure_the_runtime(self):
        invalid = [
            ("xyz", {}), (["x", object()], {}), (["x"], {"text_mode": 1}),
            (["x"], {"max_workers": True}), (["x"], {"max_workers": 1.5}),
            (["x"], {"max_workers": 0}), (["x"], {"max_workers": 65}),
            (["x"], {"alignment": "center"}), (["x"], {"template": b"default"}),
            (["x"], {"preamble": "x" * 262_145}),
            (["x"], {"template": "x" * 1025}), (["x\ud800"], {}),
        ]
        for sources, options in invalid:
            with self.subTest(options=options.keys()):
                with patch.object(typesetting, "import_module") as imported:
                    with self.assertRaises((TypeError, ValueError, UnicodeError)):
                        typesetting.preflight_tex(sources, **options)
                    imported.assert_not_called()

    def test_infinite_generator_is_bounded(self):
        yielded = []
        def sources():
            for index in itertools.count():
                yielded.append(index)
                yield "x"
        with self.assertRaisesRegex(ValueError, "4096 sources"):
            typesetting._preflight_arguments(sources())
        self.assertEqual(len(yielded), 4097)

    def test_utf8_and_aggregate_budgets(self):
        with self.assertRaisesRegex(ValueError, "UTF-8"):
            typesetting._preflight_arguments(["é" * 131_073])
        batch, _ = typesetting._preflight_arguments(["é" * 131_072])
        self.assertEqual(len(batch), 1)
        with self.assertRaisesRegex(ValueError, "4 MiB"):
            typesetting._preflight_arguments(["x" * 262_144] * 17)
        with self.assertRaisesRegex(ValueError, "4 MiB"):
            typesetting._preflight_arguments(["x"] * 17, preamble="p" * 262_144)

    def test_iterator_failure_precedes_any_runtime_side_effect(self):
        def sources():
            yield "x"
            raise RuntimeError("author generator failed")
        with patch.object(typesetting, "import_module") as imported:
            with self.assertRaisesRegex(RuntimeError, "author generator failed"):
                typesetting.preflight_tex(sources())
            imported.assert_not_called()

    def test_text_preamble_and_template_are_forwarded_without_rewriting(self):
        batch, options = typesetting._preflight_arguments(
            [r"$\word$"], template="basic", preamble=r"\newcommand{\word}{x}",
            text_mode=True, alignment="right", max_workers=1,
        )
        self.assertEqual(batch, [r"$\word$"])
        self.assertEqual(options, dict(template="basic", preamble=r"\newcommand{\word}{x}",
                                       text_mode=True, alignment="right", max_workers=1))

    def test_empty_batch_and_defaults(self):
        batch, options = typesetting._preflight_arguments(iter(()))
        self.assertEqual(batch, [])
        self.assertFalse(options["text_mode"])
        self.assertIsNone(options["alignment"])
        self.assertEqual(options["template"], "")


if __name__ == "__main__":
    unittest.main()
