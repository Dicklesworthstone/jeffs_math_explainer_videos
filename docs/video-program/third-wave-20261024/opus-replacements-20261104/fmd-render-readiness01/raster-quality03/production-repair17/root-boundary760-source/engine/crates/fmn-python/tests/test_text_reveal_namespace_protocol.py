"""Root export pruning must not disable a qualified animation class."""
import sys
import types
import unittest
from unittest.mock import patch

import test_text_reveal_protocol as fixtures


class QualifiedRevealTests(unittest.TestCase):
    def test_qualified_only_letter_class_is_installed_without_new_root_exports(self):
        native = fixtures.ExistingAnimationIntegrationTests().environment()
        letter = native.__dict__.pop("AddTextLetterByLetter")
        creation = types.ModuleType("manimlib.animation.creation")
        creation.AddTextLetterByLetter = letter
        with patch.dict(sys.modules, {creation.__name__: creation}):
            fixtures.reveal.install_text_reveal(native)
            before = letter.__init__
            fixtures.reveal.install_text_reveal(native)
        self.assertIs(letter.__init__, before)
        self.assertFalse(hasattr(native, "AddTextLetterByLetter"))
        self.assertIs(creation.AddTextLetterByLetter, letter)
        root = fixtures.string(fixtures.Node("a"), fixtures.Node("b"), paths=[(0,), (1,)])
        root.string = "ab"
        root._byte_span = lambda span: span
        animation = letter(root)
        animation.interpolate_mobject(.5)
        self.assertEqual(fixtures.names(root), ["a"])
        animation.interpolate_mobject(1)
        self.assertEqual(fixtures.names(root), ["a", "b"])

    def test_absent_or_foreign_qualified_class_still_refuses_initialization(self):
        for wrong in (None, object, 4):
            native = fixtures.ExistingAnimationIntegrationTests().environment()
            native.__dict__.pop("AddTextLetterByLetter")
            creation = types.SimpleNamespace(AddTextLetterByLetter=wrong)
            with patch.dict(sys.modules, {"manimlib.animation.creation": creation}):
                with self.assertRaisesRegex(ImportError, "missing authored"):
                    fixtures.reveal.install_text_reveal(native)
            self.assertFalse(vars(native).get("_FMN_TEXT_REVEAL_INSTALLED", False))


if __name__ == "__main__":
    unittest.main()
