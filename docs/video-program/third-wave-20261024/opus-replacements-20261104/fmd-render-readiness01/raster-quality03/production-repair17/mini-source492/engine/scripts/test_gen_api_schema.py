from __future__ import annotations

import ast
import unittest

import gen_api_schema as schema


class ClassNamespaceExtractionTests(unittest.TestCase):
    def rows(self, source: str) -> tuple[list[tuple[str, ...]], list[tuple[str, ...]]]:
        tree = ast.parse(source)
        cls = next(node for node in tree.body if isinstance(node, ast.ClassDef))
        return schema.class_schema_rows("m", cls.name, cls)

    def test_later_method_definition_replaces_symbol_and_signature(self) -> None:
        symbols, params = self.rows(
            """
class Example:
    def rotate(self, obsolete, /):
        return obsolete

    def rotate(self, angle, axis=None, *, about_point=None, **kwargs):
        return angle
"""
        )
        self.assertEqual(
            symbols,
            [("m", "Example.rotate", "method", "defined", "0", "-")],
        )
        self.assertEqual(
            [(row[2], row[3], row[5]) for row in params],
            [
                ("self", "positional_or_keyword", "-"),
                ("angle", "positional_or_keyword", "-"),
                ("axis", "positional_or_keyword", "None"),
                ("about_point", "keyword_only", "None"),
                ("kwargs", "var_keyword", "-"),
            ],
        )

    def test_methods_attributes_and_deletions_follow_final_namespace(self) -> None:
        symbols, params = self.rows(
            """
class Example:
    helper = 1
    def helper(self, value):
        return value

    def mode(self):
        return "method"
    mode = "attribute"

    removed = 3
    del removed
"""
        )
        self.assertEqual(
            symbols,
            [
                ("m", "Example.helper", "method", "defined", "0", "-"),
                ("m", "Example.mode", "attribute", "defined", "0", "'attribute'"),
            ],
        )
        self.assertEqual([row[2] for row in params], ["self", "value"])

    def test_property_setter_rebinding_remains_a_property(self) -> None:
        symbols, params = self.rows(
            """
class Example:
    @property
    def value(self):
        return 1

    @value.setter
    def value(self, new_value):
        pass
"""
        )
        self.assertEqual(
            symbols,
            [("m", "Example.value", "property", "defined", "0", "-")],
        )
        self.assertEqual([row[2] for row in params], ["self", "new_value"])

    def test_multi_target_assignments_keep_each_final_binding(self) -> None:
        symbols, params = self.rows(
            """
class Example:
    left = right = 5
"""
        )
        self.assertEqual(
            symbols,
            [
                ("m", "Example.left", "attribute", "defined", "0", "5"),
                ("m", "Example.right", "attribute", "defined", "0", "5"),
            ],
        )
        self.assertEqual(params, [])


if __name__ == "__main__":
    unittest.main()
