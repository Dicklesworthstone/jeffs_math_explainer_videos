"""Unit tests for structural_facts.py (fm-5wq.36): encoding, ordering, diff, exclusions, hooks.

Pure Python over stand-in mobjects, so this runs anywhere. The real two-engine
comparison is `structural_facts.py check` (gate) and the parity e2e scenario.
"""
from __future__ import annotations

import copy
import json
import math
from pathlib import Path
import sys
import types
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import structural_facts as sf  # noqa: E402


class _Dtype:
    names = ("point", "stroke_rgba")


class _Data:
    dtype = _Dtype()


class Mobject:
    def __init__(self, points=(), children=(), z_index=0):
        self.points = [list(p) for p in points]
        self.submobjects = list(children)
        self.z_index = z_index
        self.data = _Data()

    def get_points(self):
        return self.points

    def get_bounding_box(self):
        family = self._all_points()
        if not family:
            return [[0.0, 0.0, 0.0]] * 3
        lo = [min(p[i] for p in family) for i in range(3)]
        hi = [max(p[i] for p in family) for i in range(3)]
        return [lo, [(a + b) / 2 for a, b in zip(lo, hi)], hi]

    def _all_points(self):
        out = list(self.points)
        for child in self.submobjects:
            out.extend(child._all_points())
        return out


class VMobject(Mobject):
    fill = ("#FFFFFF", 0.5)

    def get_fill_color(self):
        return self.fill[0].lower()

    def get_fill_opacity(self):
        return self.fill[1]

    def get_stroke_color(self):
        return "#ffffff"

    def get_stroke_width(self):
        return 4.0

    def get_stroke_opacity(self):
        return 1.0


class Group(Mobject):
    pass


class VGroup(Group, VMobject):
    pass


class Polygon(VMobject):
    def get_vertices(self):
        return self.points[::2]


class Square(Polygon):
    pass


class _Private(Square):
    pass


NS = types.SimpleNamespace(Mobject=Mobject, VMobject=VMobject, Group=Group, VGroup=VGroup, Polygon=Polygon,
                           Square=Square, _Private=_Private)
SQUARE = [(1, 1, 0), (-1, 1, 0), (-1, -1, 0), (1, -1, 0), (1, 1, 0)]


def facts(roots, subject="s", **kwargs):
    return sf.extract(subject, roots, namespace=NS, **kwargs)


class Encoding(unittest.TestCase):
    def test_quantization_rounds_half_to_even_in_integer_quanta(self):
        self.assertEqual(sf.quantize([0.0005, 0.0015, -0.0005, 0.0025]), [0, 2, 0, 2])
        self.assertEqual(sf.quantize(1.23456), 1235)

    def test_non_finite_values_and_huge_values_stay_integral_and_distinct(self):
        q = sf.quantize([math.nan, math.inf, -math.inf, 1e300, -1e300])
        self.assertTrue(all(isinstance(v, int) for v in q))
        self.assertEqual(len(set(q)), 5)

    def test_canonical_encoding_is_key_order_independent(self):
        self.assertEqual(sf.canonical({"b": 1, "a": [2]}), sf.canonical({"a": [2], "b": 1}))
        self.assertEqual(sf.canonical({"a": 1}), '{"a":1}')

    def test_records_hold_only_integers_strings_lists_dicts_and_null(self):
        record = facts([VGroup(children=[Square(SQUARE)])], points_mode="full")

        def walk(value):
            if isinstance(value, dict):
                for item in value.values():
                    walk(item)
            elif isinstance(value, list):
                for item in value:
                    walk(item)
            else:
                self.assertIsInstance(value, (int, str, bool, type(None)))

        walk(record)


class Extraction(unittest.TestCase):
    def test_members_are_depth_first_with_index_paths(self):
        tree = VGroup(children=[VGroup(children=[Square(SQUARE), Square(SQUARE)]), Square(SQUARE)])
        paths = [m["path"] for m in facts([tree, Square(SQUARE)])["members"]]
        self.assertEqual(paths, ["0", "0.0", "0.0.0", "0.0.1", "0.1", "1"])

    def test_public_mro_drops_private_and_unexported_classes(self):
        self.assertEqual(sf.public_mro(_Private, NS), ["Square", "Polygon", "VMobject", "Mobject"])
        self.assertEqual(sf.public_mro(VGroup, NS), ["VGroup", "Group", "VMobject", "Mobject"])

    def test_declared_getters_and_style_are_recorded(self):
        member = facts([Square(SQUARE)])["members"][0]
        self.assertEqual(member["style"]["fill_color"], "#FFFFFF")
        self.assertEqual(member["style"]["fill_opacity"], 500)
        self.assertEqual(member["getters"]["get_vertices"][0], [1000, 1000, 0])

    def test_absent_and_raising_getters_are_facts(self):
        class Broken(Square):
            def get_vertices(self):
                raise ValueError("no")

        NS.Broken = Broken
        try:
            self.assertEqual(facts([Broken(SQUARE)])["members"][0]["getters"]["get_vertices"],
                             "raises ValueError")
        finally:
            del NS.Broken

    def test_extraction_is_deterministic(self):
        tree = VGroup(children=[Square(SQUARE)])
        self.assertEqual(sf.canonical(facts([tree], points_mode="full")),
                         sf.canonical(facts([tree], points_mode="full")))


class Diff(unittest.TestCase):
    def verdict(self, a, b, exclusions=(), mode="full"):
        return sf.diff_subject(facts([a], points_mode=mode), facts([b], points_mode=mode), exclusions)

    def test_identical_structures_are_equal(self):
        self.assertEqual(self.verdict(Square(SQUARE), Square(SQUARE))["verdict"], "equal")

    def test_sub_quantum_change_is_equal(self):
        moved = [(x + 1e-12, y, z) for x, y, z in SQUARE]
        self.assertEqual(self.verdict(Square(SQUARE), Square(moved))["verdict"], "equal")

    def test_one_quantum_boundary_straddle_is_equal_but_two_quanta_differ(self):
        near = [(x + 0.0012, y, z) for x, y, z in SQUARE]
        self.assertEqual(self.verdict(Square(SQUARE), Square(near))["verdict"], "equal")
        far = [(x + 0.005, y, z) for x, y, z in SQUARE]
        result = self.verdict(Square(SQUARE), Square(far))
        self.assertEqual(result["verdict"], "differs")
        self.assertEqual(result["first_difference"]["path"], "0")

    def test_digest_mode_compares_points_exactly(self):
        near = [(x + 0.0012, y, z) for x, y, z in SQUARE]
        result = self.verdict(Square(SQUARE), Square(near), mode="digest")
        self.assertIn("points_sha256", [d["fact"] for d in result["differences"]])

    def test_counts_are_exact_not_tolerant(self):
        result = self.verdict(Square(SQUARE), Square(SQUARE + [(1, 1, 0)]))
        self.assertIn("n_points", [d["fact"] for d in result["differences"]])

    def test_type_change_differs_at_the_member(self):
        result = self.verdict(VGroup(children=[Square(SQUARE)]),
                              VGroup(children=[VMobject(SQUARE)]))
        self.assertEqual(result["verdict"], "differs")
        first = result["first_difference"]
        self.assertEqual((first["path"], first["fact"]), ("0.0", "class"))

    def test_missing_member_is_reported_with_its_path(self):
        result = self.verdict(VGroup(children=[Square(SQUARE), Square(SQUARE)]),
                              VGroup(children=[Square(SQUARE)]))
        self.assertEqual(result["first_difference"]["path"], "0.1")
        self.assertEqual(result["first_difference"]["fact"], "member")

    def test_errors_of_the_same_class_are_equal_and_otherwise_differ(self):
        a = sf.extract_error("s", AttributeError("x has no y"))
        b = sf.extract_error("s", AttributeError("different words"))
        self.assertEqual(sf.diff_subject(a, b)["verdict"], "equal")
        c = facts([Square(SQUARE)])
        self.assertEqual(sf.diff_subject(a, c)["verdict"], "differs")


class Exclusions(unittest.TestCase):
    ROWS = [
        {"id": "bn-x", "kind": "behavior-note", "ref": "BN-00", "subject": "s",
         "class": ["Square"], "fact": ["points", "n_points", "bbox", "getters.*"], "reason": "test"},
        {"id": "bead-y", "kind": "open-bead", "ref": "fm-test", "fact": "never-seen",
         "reason": "test"},
    ]

    def test_behavior_note_row_suppresses_only_its_facts(self):
        far = [(x + 0.5, y, z) for x, y, z in SQUARE]
        ref = facts([Square(SQUARE)], points_mode="full")
        portal = facts([Square(far)], points_mode="full")
        result = sf.diff_subject(ref, portal, self.ROWS)
        self.assertEqual(result["verdict"], "equal-with-exclusions")
        # points, bbox and getters.get_vertices moved; n_points did not.
        self.assertEqual(result["excluded"], {"bn-x": 3})
        # A fact outside the row (the class) still differs.
        portal["members"][0]["class"] = "VMobject"
        self.assertEqual(sf.diff_subject(ref, portal, self.ROWS)["verdict"], "differs")

    def test_unmatched_open_bead_row_is_stale(self):
        record = facts([Square(SQUARE)])
        _, summary = sf.diff_files([record], [record], self.ROWS)
        self.assertEqual(summary["stale_open_bead_exclusions"], ["bead-y"])

    def test_one_sided_subjects_are_reported(self):
        results, summary = sf.diff_files([facts([Square(SQUARE)], "a")], [facts([Square(SQUARE)], "b")])
        self.assertEqual(summary["summary"], {"one-sided": 2})

    def test_the_checked_in_table_is_well_formed(self):
        path = Path(__file__).resolve().parents[1] / "fixtures/structural_facts/exclusions.json"
        rows = sf.load_exclusions(path)
        self.assertTrue(rows)
        self.assertEqual(len({row["id"] for row in rows}), len(rows))
        for row in rows:
            self.assertRegex(row["ref"], r"^(BN-\d\d|ADR-\d{4}|fm-[a-z0-9.-]+)$")


class RowScoping(unittest.TestCase):
    def test_under_requires_a_matching_ancestor(self):
        rows = [{"id": "r", "kind": "behavior-note", "ref": "BN-00", "under": "VGroup",
                 "fact": ["class", "mro", "getters", "getters.*"], "reason": "t"}]
        inside = sf.diff_subject(facts([VGroup(children=[Square(SQUARE)])]),
                                 facts([VGroup(children=[VMobject(SQUARE)])]), rows)
        self.assertEqual(inside["verdict"], "equal-with-exclusions")
        top = sf.diff_subject(facts([Square(SQUARE)]), facts([VMobject(SQUARE)]), rows)
        self.assertEqual(top["verdict"], "differs")

    def test_mro_missing_admits_only_that_exact_gap(self):
        rows = [{"id": "g", "kind": "open-bead", "ref": "fm-x", "fact": "mro",
                 "mro_missing": ["Group"], "reason": "t"}]

        class FlatGroup(VMobject):
            pass

        FlatGroup.__name__ = "VGroup"
        NS.VGroup, original = FlatGroup, NS.VGroup
        try:
            portal = facts([FlatGroup()])
        finally:
            NS.VGroup = original
        ref = facts([VGroup()])
        self.assertEqual(sf.diff_subject(ref, portal, rows)["verdict"], "equal-with-exclusions")
        portal["members"][0]["mro"] = ["VGroup", "Mobject"]  # VMobject missing too
        self.assertEqual(sf.diff_subject(ref, portal, rows)["verdict"], "differs")

    def test_scene_scoped_rows_are_not_stale_in_construction_runs(self):
        rows = [{"id": "s", "kind": "open-bead", "ref": "fm-x", "scope": "scenes",
                 "fact": "camera_frame_roots", "reason": "t"}]
        record = facts([Square(SQUARE)])
        _, summary = sf.diff_files([record], [record], rows, scope="constructions")
        self.assertEqual(summary["stale_open_bead_exclusions"], [])
        _, summary = sf.diff_files([record], [record], rows, scope="scenes")
        self.assertEqual(summary["stale_open_bead_exclusions"], ["s"])

    def test_structure_tier_ignores_geometry_but_not_classes_or_counts(self):
        far = [(x + 0.5, y, z) for x, y, z in SQUARE]
        ref, moved = facts([Square(SQUARE)]), facts([Square(far)])
        self.assertEqual(sf.diff_subject(ref, moved, ignore=sf.GEOMETRY_FACTS)["verdict"], "equal")
        self.assertEqual(sf.diff_subject(ref, moved)["verdict"], "differs")
        retyped = facts([VMobject(SQUARE)])
        self.assertEqual(sf.diff_subject(ref, retyped, ignore=sf.GEOMETRY_FACTS)["verdict"], "differs")

    def test_angle_getters_compare_modulo_a_full_turn(self):
        a = facts([Square(SQUARE)])
        b = facts([Square(SQUARE)])
        a["members"][0]["getters"]["get_angle"] = -3142
        b["members"][0]["getters"]["get_angle"] = 3142
        self.assertEqual(sf.diff_subject(a, b)["verdict"], "equal")
        b["members"][0]["getters"]["get_angle"] = 3000
        self.assertEqual(sf.diff_subject(a, b)["verdict"], "differs")

    def test_scene_level_facts_are_compared(self):
        a = dict(facts([Square(SQUARE)]), camera_frame_roots=1, time=2000)
        b = dict(facts([Square(SQUARE)]), camera_frame_roots=0, time=2000)
        first = sf.diff_subject(a, b)["first_difference"]
        self.assertEqual((first["fact"], first["reference"], first["portal"]), ("camera_frame_roots", 1, 0))
        c = dict(b, camera_frame_roots=1, time=2001)
        self.assertEqual(sf.diff_subject(a, c)["verdict"], "equal")

    def test_a_time_equal_to_the_bn02_recount_is_only_that_fact(self):
        rows = [{"id": "bn02", "kind": "behavior-note", "ref": "BN-02", "scope": "scenes",
                 "fact": "time.bn02", "reason": "t"}]
        ref = dict(facts([Square(SQUARE)]), time=3000, time_bn02=3067)
        counted = sf.diff_subject(ref, dict(facts([Square(SQUARE)]), time=3067), rows)
        self.assertEqual((counted["verdict"], counted["excluded"]), ("equal-with-exclusions", {"bn02": 1}))
        other = sf.diff_subject(ref, dict(facts([Square(SQUARE)]), time=3100), rows)
        self.assertEqual(other["first_difference"]["fact"], "time")


class ClassSweep(unittest.TestCase):
    def test_sweep_lists_each_default_then_its_declared_calls(self):
        sweep = {"classes": ["Square", "Circle"], "calls": {"Square": ["Square(2)", "Square(3)"]}}
        self.assertEqual(list(sf.class_sweep_constructions(sweep)),
                         [("Square", "Square()"), ("Square#1", "Square(2)"),
                          ("Square#2", "Square(3)"), ("Circle", "Circle()")])

    def test_public_methods_are_class_callables_only(self):
        square = Square(SQUARE)
        square.data_attribute = 5
        methods = sf.public_methods(square)
        self.assertIn("get_vertices", methods)
        self.assertNotIn("data_attribute", methods)
        self.assertNotIn("points", methods)

    def test_missing_reference_method_is_a_gap_and_portal_extras_are_not(self):
        ref = dict(facts([Square(SQUARE)]), public_methods=["get_vertices", "shift"])
        extra = dict(facts([Square(SQUARE)]), public_methods=["get_vertices", "shift", "extension"])
        self.assertEqual(sf.diff_subject(ref, extra)["verdict"], "equal")
        missing = dict(facts([Square(SQUARE)]), public_methods=["get_vertices"])
        first = sf.diff_subject(ref, missing)["first_difference"]
        self.assertEqual(first["fact"], "public_methods.shift")

    def test_the_checked_in_sweep_covers_appendix_a_and_only_known_classes(self):
        path = Path(__file__).resolve().parents[1] / "fixtures/structural_facts/class_sweep.json"
        sweep = json.loads(path.read_text(encoding="utf-8"))
        self.assertGreaterEqual(len(sweep["classes"]), 150)
        self.assertTrue({"VMobject", "VGroup", "Group"} <= set(sweep["classes"]))
        self.assertTrue(set(sweep["calls"]) <= set(sweep["classes"]))
        self.assertFalse([c for c in sweep["classes"] if c.startswith("_")])


class RowConditions(unittest.TestCase):
    def test_empty_only_applies_to_point_less_reference_members(self):
        rows = [{"id": "e", "kind": "open-bead", "ref": "fm-x", "class": "VGroup",
                 "fact": "bbox", "empty_only": True, "reason": "t"}]
        empty_ref, empty_portal = facts([VGroup()]), facts([VGroup()])
        empty_ref["members"][0]["bbox"] = [[1000, 0, 0]] * 3
        self.assertEqual(sf.diff_subject(empty_ref, empty_portal, rows)["verdict"], "equal-with-exclusions")
        full_ref = facts([VGroup(children=[Square(SQUARE)])])
        full_portal = facts([VGroup(children=[Square(SQUARE)])])
        full_ref["members"][0]["bbox"] = [[5000, 0, 0]] * 3
        self.assertEqual(sf.diff_subject(full_ref, full_portal, rows)["verdict"], "differs")

    def test_adr_kind_and_scope_lists_validate(self):
        import tempfile

        rows = {"exclusions": [
            {"id": "a", "kind": "adr", "ref": "ADR-0020", "scope": ["classes", "scenes"],
             "fact": "error", "reason": "t"}]}
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump(rows, handle)
        loaded = sf.load_exclusions(handle.name)
        self.assertEqual([r["id"] for r in sf.in_scope(loaded, "classes")], ["a"])
        self.assertEqual(sf.in_scope(loaded, "constructions"), [])
        rows["exclusions"][0]["scope"] = ["nowhere"]
        with open(handle.name, "w") as again:
            json.dump(rows, again)
        with self.assertRaises(ValueError):
            sf.load_exclusions(handle.name)


class ClassHook(unittest.TestCase):
    def setUp(self):
        self.original = sf._engine_namespace
        sf._engine_namespace = lambda: NS

    def tearDown(self):
        sf._engine_namespace = self.original

    def test_constructs_by_name_and_records_facts(self):
        record = sf.extract_class("Square", (SQUARE,))
        self.assertEqual(record["class"], "Square")
        self.assertEqual(record["members"][0]["mro"], ["Square", "Polygon", "VMobject", "Mobject"])

    def test_missing_class_and_raising_constructor_are_error_facts(self):
        self.assertTrue(sf.extract_class("NoSuchMobject")["error"].startswith("AttributeError"))
        self.assertTrue(sf.extract_class("Square", (1, 2, 3, 4))["error"].startswith("TypeError"))


class Engines(unittest.TestCase):
    def test_summary_names_both_engine_identities(self):
        a = dict(facts([Square(SQUARE)]), engine={"engine": "reference", "engine_id": "r"})
        b = dict(facts([Square(SQUARE)]), engine={"engine": "franken_manim", "engine_id": "p"})
        _, summary = sf.diff_files([a], [b])
        self.assertIn('"engine_id":"r"', summary["engines"]["reference"][0])
        self.assertIn('"engine_id":"p"', summary["engines"]["portal"][0])


class SceneHook(unittest.TestCase):
    def test_tear_down_records_the_scene_mobjects_then_runs_the_original(self):
        calls = []

        class Scene:
            def tear_down(self):
                calls.append("original")

        class User(Scene):
            def __init__(self):
                self.mobjects = [Square(SQUARE)]

        sink = []
        hooked = sf.scene_hook(User, sink.append)
        self.assertEqual(hooked.__name__, "User")
        original_namespace = sf._engine_namespace
        sf._engine_namespace = lambda: NS
        try:
            hooked().tear_down()
        finally:
            sf._engine_namespace = original_namespace
        self.assertEqual(calls, ["original"])
        self.assertEqual(sink[0]["subject"], "User")
        self.assertEqual(sink[0]["members"][0]["class"], "Square")


class Envelopes(unittest.TestCase):
    """fm-5wq.45: a Behavior Note bounds a bbox difference; it never blankets it."""
    FIXTURES = Path(__file__).resolve().parents[1] / "fixtures/structural_facts"

    @staticmethod
    def note(envelope=None, **row):
        return {"id": "bn-e", "kind": "behavior-note", "ref": "BN-00", "class": "Square",
                "fact": ["bbox", "points", "points_sha256", "getters.*"],
                "envelope": envelope or {"size_rel": 0.1, "center_abs": 0.05}, "reason": "t", **row}

    @staticmethod
    def square(transform):
        return facts([Square([transform(x, y, z) for x, y, z in SQUARE])])

    def test_inside_the_envelope_is_excluded_and_outside_names_the_broken_envelope(self):
        ref, rows = self.square(lambda x, y, z: (x, y, z)), [self.note()]
        inside = sf.diff_subject(ref, self.square(lambda x, y, z: (1.05 * x, y, z)), rows)
        self.assertEqual(inside["verdict"], "equal-with-exclusions")
        self.assertEqual(inside["envelope_violations"], [])
        outside = sf.diff_subject(ref, self.square(lambda x, y, z: (1.5 * x, y, z)), rows)
        self.assertEqual(outside["verdict"], "differs")
        self.assertEqual(outside["first_difference"]["fact"], "bbox")
        violation = outside["first_difference"]["envelope_violation"]
        self.assertEqual((violation["row"], violation["size_rel"]), ("bn-e", 0.5))
        self.assertEqual(outside["envelope_violations"][0]["path"], "0")

    def test_a_broken_envelope_falls_through_to_a_bounded_bug_row(self):
        bug = {"id": "bug", "kind": "open-bead", "ref": "fm-test", "class": "Square", "fact": ["bbox"],
               "envelope": {"size_rel": 0.6, "center_abs": 0.05}, "reason": "measured bug"}
        ref, wide = self.square(lambda x, y, z: (x, y, z)), self.square(lambda x, y, z: (1.5 * x, y, z))
        result = sf.diff_subject(ref, wide, [self.note(), bug])
        self.assertEqual(result["verdict"], "equal-with-exclusions")
        self.assertEqual(result["excluded"]["bug"], 1)
        twice = sf.diff_subject(ref, self.square(lambda x, y, z: (2.0 * x, y, z)), [self.note(), bug])
        self.assertEqual(twice["first_difference"]["envelope_violation"]["row"], "bn-e")

    def test_an_edge_placed_object_may_move_its_centre_by_half_its_size_change(self):
        ref, rows = self.square(lambda x, y, z: (x, y, z)), [self.note()]
        # Left edge pinned, 8% wider: the centre moves 0.08, half the extent change.
        grown = self.square(lambda x, y, z: (x + 0.08 * (x + 1), y, z))
        self.assertEqual(sf.diff_subject(ref, grown, rows)["verdict"], "equal-with-exclusions")
        # The same 0.08 as a pure translation is a position difference.
        moved = sf.diff_subject(ref, self.square(lambda x, y, z: (x + 0.08, y, z)), rows)
        self.assertEqual(moved["first_difference"]["envelope_violation"]["center_offset"], 0.08)

    def test_a_behavior_note_without_an_envelope_fails_validation_and_check(self):
        row = self.note()
        del row["envelope"]
        with self.assertRaisesRegex(ValueError, "without an envelope"):
            sf.validate_exclusions([row])
        summary = sf.check_constructions("", json.dumps({"exclusions": [row]}), "unit")
        self.assertEqual(summary["failing"][0]["verdict"], "invalid-exclusions")
        # A known bug may still be acknowledged without a magnitude.
        sf.validate_exclusions([dict(row, kind="open-bead", ref="fm-test")])

    def test_envelope_shapes_are_validated(self):
        child = {"id": "c", "kind": "behavior-note", "ref": "BN-00", "fact": ["bbox"],
                 "envelope": {"bounded_by": "ancestor"}, "reason": "t"}
        for rows, message in (
                ([child], "needs `under`"),
                ([dict(child, under=["Square"])], "no bounded bbox row"),
                ([self.note({"size_rel": 0, "center_abs": 0.05})], "positive finite"),
                ([self.note({"size_rel": 0.1})], "positive finite"),
                ([dict(self.note(), fact=["points"])], "covers bbox")):
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                sf.validate_exclusions(rows)
        sf.validate_exclusions([self.note(), dict(child, under=["Square"])])

    def test_planted_tall_tex_and_flat_brace_fail_the_checked_in_table(self):
        text = (self.FIXTURES / "reference_constructions.v1.ndjson").read_text(encoding="utf-8")
        ref = {r["subject"]: r for r in sf.parse_ndjson(text)}
        rows = sf.in_scope(sf.load_exclusions(self.FIXTURES / "exclusions.json"), "constructions")
        tall = copy.deepcopy(ref["tex"])
        (x0, y0, z0), (cx, cy, cz), (x1, y1, z1) = tall["members"][0]["bbox"]
        tall["members"][0]["bbox"] = [[x0, cy - (y1 - y0), z0], [cx, cy, cz], [x1, cy + (y1 - y0), z1]]
        flat = copy.deepcopy(ref["brace"])
        flat["members"][0]["bbox"] = [[0, 0, 0]] * 3
        for planted, row in ((tall, "bn05-text-geometry"), (flat, "bn08-brace-path-family")):
            subject = planted["subject"]
            with self.subTest(subject=subject):
                self.assertEqual(sf.diff_subject(ref[subject], ref[subject], rows)["verdict"], "equal")
                result = sf.diff_subject(ref[subject], planted, rows)
                self.assertEqual(result["verdict"], "differs")
                self.assertEqual(result["first_difference"]["path"], "0")
                self.assertEqual(result["first_difference"]["envelope_violation"]["row"], row)


class Fixture(unittest.TestCase):
    def test_reference_fixture_covers_every_construction_with_provenance(self):
        path = Path(__file__).resolve().parents[1] / "fixtures/structural_facts/reference_constructions.v1.ndjson"
        text = path.read_text(encoding="utf-8")
        header = json.loads(text.splitlines()[0][2:])
        self.assertEqual(header["engine"]["engine"], "reference")
        self.assertIn("6199a00d4c1b1127ebe45cb629c3f22538b10e13", header["engine"]["engine_id"])
        subjects = [r["subject"] for r in sf.parse_ndjson(text)]
        self.assertEqual(subjects, [s for s, _ in sf.CONSTRUCTIONS])
        for record in sf.parse_ndjson(text):
            self.assertEqual((record["schema"], record["version"]), (sf.SCHEMA, sf.VERSION))


if __name__ == "__main__":
    unittest.main()
