"""Structural facts: one extractor and one diff for the Reference and the portal (fm-5wq.36).

This module runs **unmodified** under the pinned Reference (3b1b/manim @
6199a00d, `scripts/manim_ref`) and under FrankenManim's `manimlib` portal. It
touches only the public manimlib API: `submobjects`, `get_points`,
`get_bounding_box`, `data`, the style getters, and a small declared table of
zero-argument getters. So the same code measures both engines, and the class
sweep, the corpus differential and the differential fuzzer (fm-5wq.26, .33,
.34) share one answer about quantization, ordering and naming.

Schema `fmn.structural-facts` version 1 (docs/ratchet/STRUCTURAL_FACTS.md):
one JSON object per subject (a construction, a class, or a scene's
`self.mobjects` at `tear_down`). The object holds depth-first members keyed
by index path, and the canonical encoding holds only integers and strings.
Every length is an integer count of quanta: `QUANTA_PER_UNIT` per scene unit,
rounded half to even.

Command line:
  python structural_facts.py extract --engine-id ID [--points full|digest] OUT.ndjson
      Construct every entry of CONSTRUCTIONS in the running engine and write
      one fact line per subject.
  python structural_facts.py diff REFERENCE.ndjson PORTAL.ndjson [--exclusions X.json]
      Compare two fact files and write one verdict line per subject.
      Exit 0 = no unexcluded difference, no stale open-bead exclusion.
"""
from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import math
import sys
import traceback
from pathlib import Path

SCHEMA = "fmn.structural-facts"
VERSION = 1
DIFF_SCHEMA = "fmn.structural-diff"
QUANTA_PER_UNIT = 1000
# The two engines compute in f64 along different paths. One quantum of slack
# absorbs values that straddle a rounding boundary. A move of two or more
# quanta (2e-3 scene units) is a difference.
TOLERANCE_QUANTA = 1
MAX_MEMBERS = 20_000
MAX_FULL_POINTS = 200_000
# Sentinels keep non-finite coordinates integral and distinguishable.
_NAN, _POS_INF, _NEG_INF = -(2**62), 2**62 - 1, -(2**62 - 1)
_LIMIT = 2**61

# Zero-argument getters recorded when a member's public class chain contains
# the key. Values are quantized when numeric. This table is part of the
# schema: changing it is a version bump.
GETTERS = {
    "Arc": ("get_arc_center", "get_start_angle"),
    "Circle": ("get_radius",),
    "Line": ("get_start", "get_end", "get_length", "get_angle"),
    "Polygon": ("get_vertices",),
    "DecimalNumber": ("get_value",),
    "NumberLine": ("x_min", "x_max", "x_step"),
    "CoordinateSystem": ("get_origin",),
}

# The fixed construction set: source text evaluated in the manimlib namespace
# of whichever engine is running. Identifiers are stable; append-only.
CONSTRUCTIONS = (
    ("square", "Square()"),
    ("circle", "Circle(radius=1.5)"),
    ("rectangle", "Rectangle(width=3, height=1)"),
    ("line", "Line(LEFT, RIGHT + UP)"),
    ("arrow", "Arrow(LEFT, RIGHT)"),
    ("dot", "Dot(UP)"),
    ("arc", "Arc(start_angle=0, angle=PI / 3, radius=2)"),
    ("polygon", "Polygon(ORIGIN, RIGHT, UP)"),
    ("regular_polygon", "RegularPolygon(6)"),
    ("star", "Star()"),
    ("annulus", "Annulus()"),
    ("vgroup_arranged", "VGroup(Square(), Circle(), Triangle()).arrange(RIGHT)"),
    ("shifted_scaled", "Square().shift(2 * RIGHT).scale(0.5).rotate(PI / 7)"),
    ("styled", "Circle().set_fill(RED, opacity=0.5).set_stroke(BLUE, width=6)"),
    ("text", 'Text("Hello")'),
    ("tex", r'Tex(r"x^2 + \frac{1}{2}")'),
    ("decimal_number", "DecimalNumber(3.14159, num_decimal_places=3)"),
    ("number_line", "NumberLine((-3, 3, 1))"),
    ("axes", "Axes((-2, 2), (-1, 1))"),
    ("number_plane", "NumberPlane((-2, 2), (-1, 1))"),
    ("surrounding_rectangle", "SurroundingRectangle(Circle())"),
    ("brace", "Brace(Square(), DOWN)"),
    ("sphere", "Sphere()"),
    ("cone", "Cone()"),
    ("vcube", "VCube()"),
    # fm-5wq.28 (corpus _2023/gauss_int/herschel.py RationalNumbers): live
    # numbers at the Reference's family indices survive copy() as DecimalNumber.
    ("tex_changeable_fraction_copy",
     r'(t := Tex(r"1 \over 2"), t.make_number_changeable("1"), t.make_number_changeable("2"),'
     r' c := t.copy(), c[0].set_value(3), c[2].set_value(7), c)[-1]'),
)


# ---------------------------------------------------------------- encoding


def canonical(value) -> str:
    """The one serialization: sorted keys, no whitespace, integers and strings only."""
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def _quantize_scalar(x) -> int:
    x = float(x)
    if math.isnan(x):
        return _NAN
    if math.isinf(x):
        return _POS_INF if x > 0 else _NEG_INF
    q = x * QUANTA_PER_UNIT
    if abs(q) >= _LIMIT:
        return _LIMIT if q > 0 else -_LIMIT
    # round() on a float is round-half-to-even.
    return int(round(q))


def quantize(value):
    """Quantize a number or a (nested) sequence of numbers to integer quanta."""
    if hasattr(value, "tolist"):
        value = value.tolist()
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return value
    if isinstance(value, (int, float)):
        return _quantize_scalar(value)
    if isinstance(value, (list, tuple)):
        return [quantize(item) for item in value]
    return repr(value)


def _flatten(values):
    for item in values:
        if isinstance(item, list):
            yield from _flatten(item)
        else:
            yield item


def digest_ints(values) -> str:
    """sha256 over the flattened integers, each as a signed 64-bit little-endian word."""
    h = hashlib.sha256()
    for item in _flatten(values):
        h.update(int(item).to_bytes(8, "little", signed=True))
    return h.hexdigest()


# ---------------------------------------------------------------- extraction


def _engine_namespace():
    import manimlib

    return manimlib


def public_mro(cls, namespace) -> list[str]:
    """Names in the class chain that the engine's manimlib exports under the same name.

    Private names, and classes from the standard library (typing.Generic, abc.ABC)
    that leak into the namespace, are not part of the manim API surface.
    """
    stdlib = getattr(sys, "stdlib_module_names", frozenset())
    names = []
    for klass in cls.__mro__:
        name = klass.__name__
        if name.startswith("_") or getattr(namespace, name, None) is not klass:
            continue
        if klass.__module__.split(".")[0] in stdlib or klass.__module__ == "builtins":
            continue
        names.append(name)
    return names


def _call_getter(mob, name):
    attr = getattr(mob, name, _ABSENT)
    if attr is _ABSENT:
        return "absent"
    try:
        value = attr() if callable(attr) else attr
    except Exception as error:  # noqa: BLE001 - a raising getter is itself a fact
        return f"raises {type(error).__name__}"
    return quantize(value)


_ABSENT = object()


def _style(mob, namespace):
    vmobject = getattr(namespace, "VMobject", None)
    if vmobject is None or not isinstance(mob, vmobject):
        return None
    style = {}
    for key, getter in (
        ("fill_color", "get_fill_color"),
        ("fill_opacity", "get_fill_opacity"),
        ("stroke_color", "get_stroke_color"),
        ("stroke_width", "get_stroke_width"),
        ("stroke_opacity", "get_stroke_opacity"),
    ):
        value = _call_getter(mob, getter)
        if isinstance(value, str) and value.startswith("#"):
            value = value.upper()
        style[key] = value
    return style


def _member_facts(mob, path, namespace, points_mode):
    facts = {"path": path, "class": type(mob).__name__, "mro": public_mro(type(mob), namespace)}
    try:
        points = mob.get_points()
        rows = points.tolist() if hasattr(points, "tolist") else [list(p) for p in points]
    except Exception as error:  # noqa: BLE001
        facts["points"] = f"raises {type(error).__name__}"
        rows = None
    if rows is not None:
        q = quantize(rows)
        facts["n_points"] = len(q)
        facts["points_sha256"] = digest_ints(q)
        if points_mode == "full" and len(q) <= MAX_FULL_POINTS:
            facts["points"] = q
    try:
        facts["bbox"] = quantize(mob.get_bounding_box())
    except Exception as error:  # noqa: BLE001
        facts["bbox"] = f"raises {type(error).__name__}"
    data = getattr(mob, "data", None)
    names = getattr(getattr(data, "dtype", None), "names", None)
    facts["data_fields"] = list(names) if names else None
    facts["z_index"] = quantize(getattr(mob, "z_index", None))
    string = getattr(mob, "string", None)
    if isinstance(string, str):
        facts["string"] = string
    style = _style(mob, namespace)
    if style is not None:
        facts["style"] = style
    getters = {}
    for klass in facts["mro"]:
        for name in GETTERS.get(klass, ()):
            getters[name] = _call_getter(mob, name)
    if getters:
        facts["getters"] = getters
    return facts


def extract(subject: str, roots, *, namespace=None, points_mode: str = "digest") -> dict:
    """Facts for a list of root mobjects, members in depth-first order.

    Paths are dot-joined submobject indices under the root's index, so the
    first root is "0", its second child "0.1".
    """
    if points_mode not in ("digest", "full"):
        raise ValueError(f"points_mode must be 'digest' or 'full', not {points_mode!r}")
    namespace = namespace or _engine_namespace()
    members = []
    truncated = False
    stack = [(str(index), root) for index, root in reversed(list(enumerate(roots)))]
    while stack:
        path, mob = stack.pop()
        if len(members) >= MAX_MEMBERS:
            truncated = True
            break
        members.append(_member_facts(mob, path, namespace, points_mode))
        children = list(getattr(mob, "submobjects", ()) or ())
        stack.extend((f"{path}.{i}", child) for i, child in reversed(list(enumerate(children))))
    record = {
        "schema": SCHEMA,
        "version": VERSION,
        "quanta_per_unit": QUANTA_PER_UNIT,
        "subject": subject,
        "members": members,
    }
    if truncated:
        record["truncated_at"] = MAX_MEMBERS
    return record


def extract_error(subject: str, error: BaseException) -> dict:
    """A construction that raised is a fact too: its exception class and first line."""
    message = str(error).splitlines()[0][:200] if str(error) else ""
    return {
        "schema": SCHEMA,
        "version": VERSION,
        "quanta_per_unit": QUANTA_PER_UNIT,
        "subject": subject,
        "error": f"{type(error).__name__}: {message}",
    }


def engine_identity(engine_id: str) -> dict:
    namespace = _engine_namespace()
    portal = bool(getattr(namespace, "__franken_manim__", False))
    return {"engine": "franken_manim" if portal else "reference", "engine_id": engine_id}


def public_methods(mob) -> list[str]:
    """Public methods of an instance's class: the callable API surface.

    Instance data attributes are left out. The Reference's include GL renderer
    state (shader_wrapper, outer_vert_indices, ...) that the portal does not
    have by design (ADR-0021).
    """
    cls = type(mob)
    return sorted(name for name in dir(cls)
                  if not name.startswith("_") and callable(getattr(cls, name, None)))


def class_sweep_constructions(sweep: dict):
    """(subject, source) pairs for a class sweep: each class's default call, then its
    declared parameterized calls, subjects `Class` and `Class#1`, `Class#2`, ..."""
    for name in sweep["classes"]:
        yield name, f"{name}()"
        for index, source in enumerate(sweep.get("calls", {}).get(name, ()), 1):
            yield f"{name}#{index}", source


def extract_constructions(engine_id: str, points_mode: str = "digest", only=None,
                          constructions=CONSTRUCTIONS, api_names: bool = False):
    """Yield one fact record per construction (the fixed set by default)."""
    namespace = _engine_namespace()
    scope = {name: getattr(namespace, name) for name in dir(namespace) if not name.startswith("_")}
    identity = engine_identity(engine_id)
    for subject, source in constructions:
        if only is not None and subject not in only:
            continue
        try:
            # Constructions are checked-in constant text. They are evaluated over the
            # manimlib namespace only, with no builtins, so they can reach nothing else.
            code = compile(source, f"<construction {subject}>", "eval")
            mob = eval(code, {"__builtins__": {}, **scope})  # noqa: S307
            record = extract(subject, [mob], namespace=namespace, points_mode=points_mode)
            # A class's methods do not depend on its arguments: record them once,
            # on the default call (subject without "#").
            if api_names and "#" not in subject:
                record["public_methods"] = public_methods(mob)
        except Exception as error:  # noqa: BLE001
            record = extract_error(subject, error)
        record["source"] = source
        record["engine"] = identity
        yield record


def extract_class(name: str, args=(), kwargs=None, *, points_mode: str = "digest") -> dict:
    """Construct `manimlib.<name>(*args, **kwargs)` in the running engine and record its facts.

    This is the class-level hook for the Appendix-A sweep (fm-5wq.26). A missing
    class or a raising constructor is recorded as an `error` fact, never skipped.
    """
    namespace = _engine_namespace()
    subject = name if not args and not kwargs else f"{name}{canonical([list(args), kwargs or {}])}"
    try:
        cls = getattr(namespace, name)
        record = extract(subject, [cls(*args, **(kwargs or {}))], namespace=namespace,
                         points_mode=points_mode)
    except Exception as error:  # noqa: BLE001
        record = extract_error(subject, error)
    record["class"] = name
    return record


def scene_facts(scene, namespace=None, points_mode: str = "digest") -> dict:
    """Facts for `scene.mobjects` at the end of `construct`.

    The Reference keeps its CameraFrame in `scene.mobjects` (scene.py:117). It is
    dropped from the roots, so content paths align, and its count is recorded as
    the scene-level fact `camera_frame_roots`.
    """
    namespace = namespace or _engine_namespace()
    roots = [m for m in scene.mobjects if type(m).__name__ != "CameraFrame"]
    record = extract(type(scene).__name__, roots, namespace=namespace, points_mode=points_mode)
    record["camera_frame_roots"] = len(scene.mobjects) - len(roots)
    time = getattr(scene, "time", None)
    record["time"] = quantize(
        None if time is None else time + getattr(scene, "_fmn_skipped_frame_rounding", 0.0))
    bn02 = getattr(scene, "_fmn_bn02_frame_rounding", None)
    if time is not None and bn02 is not None:
        record["time_bn02"] = quantize(time + bn02)
    return record


def _observe_reference_playback_clock(manimlib):
    """Make the Reference's `time` fact its playback clock under `-s`.

    Played, the Reference advances a segment over `arange(0, run_time, 1/fps)
    + 1/fps`, ending on the frame at or after `run_time` (scene.py:477).
    Skipped, it advances by `run_time` itself (scene.py:474), so its clock
    under `-s` differs from the same scene played: `wait(0.25)` at 30 fps is
    0.2667 s played, 0.25 s skipped. The portal keeps one clock for both
    (BN-02). This records, per skipped segment, the rounding playback would
    add; the Reference's behavior is untouched.

    It also records the same segments counted by BN-02's rule,
    `ceil(run_time * fps)` frames on the exact value of the f64 duration
    (`time_bn02`). `arange` can count one frame fewer when the f64 sits just
    above a frame boundary (`wait(0.1)` at 30 fps: 3 frames, BN-02 4).
    Returns the restorer.
    """
    from fractions import Fraction

    import numpy as np

    original = manimlib.Scene.get_time_progression

    def get_time_progression(self, run_time, *args, **kwargs):
        times = original(self, run_time, *args, **kwargs)
        override = kwargs.get("override_skip_animations", args[2] if len(args) > 2 else False)
        if self.skip_animations and not override:
            fps = self.camera.fps
            duration = float(run_time)  # exact for NumPy scalars too
            played = np.arange(0, run_time, 1 / fps) + 1 / fps
            if len(played):
                self._fmn_skipped_frame_rounding = (
                    getattr(self, "_fmn_skipped_frame_rounding", 0.0)
                    + float(played[-1]) - duration)
            frames = math.ceil(Fraction(duration) * Fraction(float(fps))) if duration > 0 else 0
            self._fmn_bn02_frame_rounding = (
                getattr(self, "_fmn_bn02_frame_rounding", 0.0) + frames / fps - duration)
        return times

    manimlib.Scene.get_time_progression = get_time_progression
    return lambda: setattr(manimlib.Scene, "get_time_progression", original)


def run_scene(facts_path: str, engine: str, engine_argv, points_mode: str = "digest") -> int:
    """Run one scene through an engine's own CLI and write its facts at `tear_down`.

    `engine` is `reference` (manimlib.__main__) or `portal` (fmn_python.__main__).
    The base `Scene.tear_down` is wrapped before the CLI runs, so scene source
    stays unedited. The facts file is written even when the scene fails, with an
    `error` record when the scene never reached `tear_down`.
    """
    program = "manimlib" if engine == "reference" else "fmn-python"
    sys.argv = [program, *engine_argv]  # the Reference parses sys.argv at import
    import manimlib

    records = []
    original = manimlib.Scene.tear_down

    def tear_down(self):
        try:
            records.append(scene_facts(self, manimlib, points_mode))
        except Exception as error:  # noqa: BLE001 - an extraction failure is a fact
            records.append(extract_error(type(self).__name__, error))
        return original(self)

    manimlib.Scene.tear_down = tear_down
    restore_clock = _observe_reference_playback_clock(manimlib) if engine == "reference" else None
    code = 1
    try:
        if engine == "reference":
            from manimlib.__main__ import main as engine_main
        else:
            from fmn_python.__main__ import main as engine_main
        result = engine_main()
        code = result if isinstance(result, int) else 0
    except SystemExit as exit_:
        code = exit_.code if isinstance(exit_.code, int) else (0 if exit_.code is None else 1)
    finally:
        manimlib.Scene.tear_down = original
        if restore_clock is not None:
            restore_clock()
        if not records:
            records.append(extract_error("scene", RuntimeError("tear_down was never reached")))
        Path(facts_path).write_text("".join(canonical(r) + "\n" for r in records), encoding="utf-8")
    return code


def scene_hook(scene_class, sink):
    """Return a subclass of `scene_class` whose `tear_down` records `self.mobjects`.

    Both engines call `tear_down` after `construct` (Reference scene.py:164). The
    subclass keeps the user's name so tracebacks and outputs stay recognizable.
    `sink` receives the fact record.
    """

    def tear_down(self):
        try:
            sink(extract(type(self).__name__, list(self.mobjects)))
        finally:
            super(hooked, self).tear_down()

    hooked = type(scene_class.__name__, (scene_class,), {"tear_down": tear_down})
    hooked.__module__ = scene_class.__module__
    hooked.__qualname__ = scene_class.__qualname__
    return hooked


# ---------------------------------------------------------------- diff


def load_exclusions(path) -> list[dict]:
    return validate_exclusions(json.loads(Path(path).read_text())["exclusions"])


def _covers(row, fact) -> bool:
    return _matches(fact, row["fact"])


def _numeric_envelope(envelope) -> bool:
    return isinstance(envelope, dict) and set(envelope) == {"size_rel", "center_abs"}


def validate_exclusions(rows) -> list[dict]:
    """Check the table's shape and its envelope rule; return the rows unchanged.

    A behavior-note or ADR row that admits a `bbox` difference must bound it with an
    `envelope`: numbers ({"size_rel", "center_abs"}) or {"bounded_by": "ancestor"}
    on a row with `under`, where every `under` class is itself bounded by a numeric
    envelope (or by an open-bead row: a known bug). A Behavior Note justifies a
    different outline, never an unbounded size or position. An open-bead row may
    state its bug's magnitude as an envelope; it need not.
    """
    for row in rows:
        missing = {"id", "kind", "ref", "fact", "reason"} - set(row)
        if missing:
            raise ValueError(f"exclusion {row.get('id')!r} lacks {sorted(missing)}")
        if row["kind"] not in ("behavior-note", "adr", "open-bead"):
            raise ValueError(f"exclusion {row['id']!r}: kind must be behavior-note, adr or open-bead")
        scopes = row.get("scope", "any")
        scopes = [scopes] if isinstance(scopes, str) else scopes
        if not set(scopes) <= {"any", "constructions", "classes", "scenes"}:
            raise ValueError(f"exclusion {row['id']!r}: scope must be any, constructions, classes or scenes")
        envelope = row.get("envelope")
        if envelope is None:
            if _covers(row, "bbox") and row["kind"] != "open-bead":
                raise ValueError(f"exclusion {row['id']!r} admits bbox differences without an envelope")
            continue
        if not _covers(row, "bbox"):
            raise ValueError(f"exclusion {row['id']!r}: an envelope needs a row that covers bbox")
        if envelope == {"bounded_by": "ancestor"}:
            if "under" not in row:
                raise ValueError(f"exclusion {row['id']!r}: an ancestor-bounded envelope needs `under`")
            continue
        if not _numeric_envelope(envelope) or not all(
                isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v) and v > 0
                for v in envelope.values()):
            raise ValueError(f"exclusion {row['id']!r}: envelope must be positive finite "
                             "{size_rel, center_abs} or {bounded_by: ancestor}")
    for row in rows:
        if row.get("envelope") != {"bounded_by": "ancestor"}:
            continue
        under = [row["under"]] if isinstance(row["under"], str) else row["under"]
        for name in under:
            if not any(_covers(other, "bbox") and _matches(name, other.get("class", "*"))
                       and (_numeric_envelope(other.get("envelope")) or other["kind"] == "open-bead")
                       for other in rows):
                raise ValueError(f"exclusion {row['id']!r}: ancestor class {name!r} has no bounded bbox row")
    return rows


def _matches(value, pattern) -> bool:
    """A row field is one glob or a list of globs; any match admits the value."""
    patterns = [pattern] if isinstance(pattern, str) else pattern
    return any(fnmatch.fnmatchcase(value, p) for p in patterns)


def envelope_violation(envelope, reference, portal):
    """None when a portal bbox lies within `envelope` of the Reference's, else the excess.

    Per axis, the extent may differ by `size_rel` of the Reference's extent, and the
    centre may move by `center_abs` scene units plus half the extent difference: an
    object placed by one edge (next_to, to_edge, align_to) keeps that edge, so its
    centre moves by half of any size change. Each carries the quantization slack
    (two quanta on an extent, one on a centre). A bbox that is not a pair of
    [min, centre, max] triples violates any envelope.
    """
    def triples(b):
        return isinstance(b, list) and len(b) == 3 and all(
            isinstance(p, list) and len(p) == 3 and all(isinstance(v, int) for v in p) for p in b)

    if not (triples(reference) and triples(portal)):
        return {"size_rel": None, "center_offset": None}
    size_excess, centre_excess = 0.0, 0.0
    size_seen, centre_seen = 0.0, 0.0
    for axis in range(3):
        ref_extent = max(reference[2][axis] - reference[0][axis], 0)
        portal_extent = max(portal[2][axis] - portal[0][axis], 0)
        gap = abs(portal_extent - ref_extent)
        allowed = envelope["size_rel"] * ref_extent + 2 * TOLERANCE_QUANTA
        relative = gap / ref_extent if ref_extent else (0.0 if gap <= 2 * TOLERANCE_QUANTA else math.inf)
        size_seen = max(size_seen, relative)
        size_excess = max(size_excess, gap - allowed)
        shift = abs(portal[1][axis] - reference[1][axis])
        centre_seen = max(centre_seen, shift / QUANTA_PER_UNIT)
        centre_excess = max(centre_excess, shift - envelope["center_abs"] * QUANTA_PER_UNIT
                            - gap / 2 - TOLERANCE_QUANTA)
    if size_excess <= 0 and centre_excess <= 0:
        return None
    return {"size_rel": None if math.isinf(size_seen) else round(size_seen, 4),
            "center_offset": round(centre_seen, 4)}


def _excluded_by(rows, subject, member_class, fact, ancestors=(), values=None, empty=False):
    """The first row covering this difference (see `_resolve`), or None."""
    return _resolve(rows, subject, member_class, fact, ancestors, values, empty)[0]


def _resolve(rows, subject, member_class, fact, ancestors=(), values=None, empty=False):
    """(row, violation): the first row admitting this difference. Rows name the Reference-side class.

    Optional row fields: `under` requires an ancestor whose class matches,
    `reference_values` requires the Reference-side value to be one of those,
    `empty_only` requires the Reference member to have no points, and
    `mro_missing` admits an `mro` difference only when the Reference chain equals
    the portal chain plus exactly those names. A numeric `envelope` admits a
    `bbox` difference only inside it; outside, matching continues down the table,
    and if no later row admits it the first envelope it broke is the violation.
    """
    violation = None
    for row in _candidate_rows(rows, subject, member_class, fact, ancestors, values, empty):
        envelope = row.get("envelope")
        if fact == "bbox" and _numeric_envelope(envelope):
            excess = envelope_violation(envelope, *(values or (None, None)))
            if excess is not None:
                if violation is None:
                    violation = {"row": row["id"], "envelope": dict(envelope), **excess}
                continue
        return row, None
    return None, violation


def _candidate_rows(rows, subject, member_class, fact, ancestors, values, empty):
    for row in rows:
        if not (_matches(subject, row.get("subject", "*"))
                and _matches(member_class or "", row.get("class", "*"))
                and _matches(fact, row["fact"])):
            continue
        if "under" in row and not any(_matches(a, row["under"]) for a in ancestors):
            continue
        if row.get("empty_only") and not empty:
            continue
        if "reference_values" in row and (values is None or values[0] not in row["reference_values"]):
            continue
        if "mro_missing" in row:
            ref_mro, portal_mro = values if values else (None, None)
            if not (isinstance(ref_mro, list) and isinstance(portal_mro, list)
                    and [n for n in ref_mro if n not in row["mro_missing"]] == portal_mro
                    and set(ref_mro) - set(portal_mro) == set(row["mro_missing"])):
                continue
        yield row


# Facts that carry positions or sizes. The structure tier ignores them: under
# BN-05, text metrics move everything laid out relative to text.
GEOMETRY_FACTS = frozenset({"points", "points_sha256", "bbox", "getters"})


def _within_tolerance(a, b) -> bool:
    """Equality with TOLERANCE_QUANTA slack on integers (quantized lengths only)."""
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(_within_tolerance(x, y) for x, y in zip(a, b))
    if isinstance(a, int) and isinstance(b, int) and not isinstance(a, bool) and not isinstance(b, bool):
        return abs(a - b) <= TOLERANCE_QUANTA
    return a == b


# Facts whose integers are quantized lengths or angles get the rounding slack;
# every other fact (class names, counts, field names, strings) is exact.
QUANTIZED_FACTS = frozenset({"points", "bbox", "z_index", "getters", "style"})


_TAU_QUANTA = _quantize_scalar(2 * math.pi)


def _same_angle(a, b) -> bool:
    """Angles in quanta, equal modulo a full turn. atan2 returns -pi or +pi for
    the same direction depending on the sign of a zero component."""
    if not (isinstance(a, int) and isinstance(b, int)) or isinstance(a, bool) or isinstance(b, bool):
        return a == b
    gap = abs(a - b) % _TAU_QUANTA
    return min(gap, _TAU_QUANTA - gap) <= TOLERANCE_QUANTA + 1


def _same(key, a, b, sub=None) -> bool:
    if key == "getters" and sub is not None and sub.endswith("angle"):
        return _same_angle(a, b)
    return _within_tolerance(a, b) if key in QUANTIZED_FACTS else a == b


def _member_differences(ref, portal):
    """Yield (fact, reference_value, portal_value) for one aligned member pair."""
    for key in sorted(set(ref) | set(portal)):
        if key in ("path", "points_sha256"):
            continue
        a, b = ref.get(key, "absent"), portal.get(key, "absent")
        if key == "points" and (a == "absent" or b == "absent"):
            continue  # digest mode on one side: compare the digest below
        if isinstance(a, dict) and isinstance(b, dict):
            for sub in sorted(set(a) | set(b)):
                x, y = a.get(sub, "absent"), b.get(sub, "absent")
                if not _same(key, x, y, sub):
                    yield f"{key}.{sub}", x, y
        elif not _same(key, a, b):
            yield key, a, b
    if "points" not in ref or "points" not in portal:
        if ref.get("points_sha256") != portal.get("points_sha256"):
            yield "points_sha256", ref.get("points_sha256"), portal.get("points_sha256")


def diff_subject(ref: dict, portal: dict, exclusions=(), limit: int = 50,
                 ignore=frozenset()) -> dict:
    """Compare one subject's facts. Differences are reported in depth-first path order.

    `ignore` names top-level facts to skip; `GEOMETRY_FACTS` gives the structure tier.
    """
    subject = ref.get("subject", portal.get("subject"))
    differences, excluded = [], {}
    classes = {m["path"]: m["class"] for m in ref.get("members", ())}
    # Points over each member's whole family: a group root has no points of its
    # own, so `empty_only` means a point-less family, not a point-less root.
    ref_points = {}
    for m in ref.get("members", ()):
        parts = m["path"].split(".")
        for depth in range(1, len(parts) + 1):
            prefix = ".".join(parts[:depth])
            ref_points[prefix] = ref_points.get(prefix, 0) + (m.get("n_points") or 0)
    for m in portal.get("members", ()):
        classes.setdefault(m["path"], m["class"])

    def drawn(record):
        # Point-bearing members in each subtree: an envelope violation between families of
        # different sizes may be a reshaped tree, not a moved object.
        counts = {}
        for m in record.get("members", ()):
            if m.get("n_points"):
                parts = m["path"].split(".")
                for depth in range(1, len(parts) + 1):
                    prefix = ".".join(parts[:depth])
                    counts[prefix] = counts.get(prefix, 0) + 1
        return counts

    ref_drawn, portal_drawn = drawn(ref), drawn(portal)

    def ancestors(path):
        parts = path.split(".")
        return [classes[".".join(parts[:i])] for i in range(1, len(parts)) if ".".join(parts[:i]) in classes]

    violations = []

    def note(path, member_class, fact, a, b):
        if fact.split(".")[0] in ignore:
            return
        row, violation = _resolve(exclusions, subject, member_class, fact, ancestors(path), (a, b),
                                  empty=ref_points.get(path) == 0)
        if row is not None:
            excluded[row["id"]] = excluded.get(row["id"], 0) + 1
            return
        if violation is not None:
            violation["family"] = [ref_drawn.get(path, 0), portal_drawn.get(path, 0)]
            if len(violations) < limit:
                violations.append({"path": path, "class": member_class, **violation})
        if len(differences) < limit:
            entry = {"path": path, "class": member_class, "fact": fact,
                     "reference": _short(a), "portal": _short(b)}
            if violation is not None:
                entry["envelope_violation"] = violation
            differences.append(entry)
        else:
            differences.append(None)

    if "error" in ref or "error" in portal:
        a, b = ref.get("error", "none"), portal.get("error", "none")
        if a.split(":")[0] != b.split(":")[0]:
            note("", None, "error", a, b)
    else:
        by_path = {m["path"]: m for m in portal["members"]}
        seen = set()
        for member in ref["members"]:
            other = by_path.get(member["path"])
            if other is None:
                note(member["path"], member["class"], "member", member["class"], "absent")
                continue
            seen.add(member["path"])
            for fact, a, b in _member_differences(member, other):
                note(member["path"], member["class"], fact, a, b)
        for member in portal["members"]:
            if member["path"] not in seen:
                note(member["path"], member["class"], "member", "absent", member["class"])
    # The API surface: each Reference public name the portal instance lacks is a gap.
    # Names only the portal has are not compared (extensions are allowed).
    if "public_methods" in ref and "public_methods" in portal:
        root_class = ref["members"][0]["class"] if ref.get("members") else None
        for name in sorted(set(ref["public_methods"]) - set(portal["public_methods"])):
            note("", root_class, f"public_methods.{name}", "present", "absent")
    # Scene-level facts from scene_facts(): exact count, quantized time.
    for key, same in (("camera_frame_roots", lambda a, b: a == b), ("time", _within_tolerance)):
        if key in ref or key in portal:
            a, b = ref.get(key, "absent"), portal.get(key, "absent")
            if not same(a, b):
                # A time the Reference's segments reach under BN-02's frame
                # count is the fact `time.bn02`, for that Behavior Note's row.
                bn02 = key == "time" and "time_bn02" in ref and same(ref["time_bn02"], b)
                note("", None, "time.bn02" if bn02 else key, a, b)
    shown = [d for d in differences if d is not None]
    verdict = "differs" if differences else ("equal-with-exclusions" if excluded else "equal")
    return {
        "schema": DIFF_SCHEMA,
        "version": VERSION,
        "subject": subject,
        "verdict": verdict,
        "difference_count": len(differences),
        "first_difference": shown[0] if shown else None,
        "differences": shown,
        "excluded": dict(sorted(excluded.items())),
        "envelope_violations": violations,
    }


def _short(value):
    text = canonical(value)
    return value if len(text) <= 240 else f"{text[:200]}… ({len(text)} chars, sha256 {hashlib.sha256(text.encode()).hexdigest()[:16]})"


def in_scope(rows, scope: str):
    """The rows that apply to a `constructions`, `classes` or `scenes` run. A row's
    `scope` is one name or a list; `any` (the default) applies everywhere."""
    def scopes(row):
        value = row.get("scope", "any")
        return [value] if isinstance(value, str) else value

    return [row for row in rows if "any" in scopes(row) or scope in scopes(row)]


def diff_files(reference_lines, portal_lines, exclusions=(), scope: str = "constructions"):
    """Verdict records for every subject, then a summary with stale open-bead exclusions."""
    exclusions = in_scope(exclusions, scope)
    ref = {r["subject"]: r for r in reference_lines}
    portal = {r["subject"]: r for r in portal_lines}
    results, used = [], set()
    for subject in list(ref) + [s for s in portal if s not in ref]:
        if subject not in ref or subject not in portal:
            results.append({"schema": DIFF_SCHEMA, "version": VERSION, "subject": subject,
                            "verdict": "one-sided",
                            "present_in": "reference" if subject in ref else "portal"})
            continue
        result = diff_subject(ref[subject], portal[subject], exclusions)
        used.update(result["excluded"])
        results.append(result)
    stale = sorted(row["id"] for row in exclusions if row["kind"] == "open-bead" and row["id"] not in used)
    counts = {}
    for result in results:
        counts[result["verdict"]] = counts.get(result["verdict"], 0) + 1

    def identities(records):
        return sorted({canonical(r["engine"]) for r in records.values() if "engine" in r})

    summary = {"schema": DIFF_SCHEMA, "version": VERSION, "summary": dict(sorted(counts.items())),
               "envelope_violations": sum(len(r.get("envelope_violations", ())) for r in results),
               "stale_open_bead_exclusions": stale,
               "engines": {"reference": identities(ref), "portal": identities(portal)}}
    return results, summary


def parse_ndjson(text: str):
    """Fact lines; `#` lines carry provenance and are skipped."""
    return [json.loads(line) for line in text.splitlines() if line.strip() and not line.startswith("#")]


def read_ndjson(path):
    return parse_ndjson(Path(path).read_text(encoding="utf-8"))


def provenance_line(engine_id: str, argv) -> str:
    """The `#` header of an extract: enough to regenerate the file on purpose."""
    try:
        import numpy

        numpy_version = numpy.__version__
    except ImportError:  # pragma: no cover - both engines ship NumPy
        numpy_version = None
    return "# " + canonical({
        "schema": SCHEMA, "version": VERSION, "engine": engine_identity(engine_id),
        "python": sys.version.split()[0], "numpy": numpy_version,
        "command": ["structural_facts.py", *argv],
    })


def check_constructions(reference_text: str, exclusions_text: str, engine_id: str,
                        sweep_text: str | None = None) -> dict:
    """Extract a construction set in the running engine and diff it.

    The fixed construction set by default; with `sweep_text` (a class-sweep JSON),
    every Appendix-A class's default and declared calls, public methods included.
    This is the gate and e2e entry point. It returns the diff summary plus
    `compared` (subjects on both sides) and `failing` (subjects that differ
    or are one-sided). A zero-subject comparison is a failure, never a pass.
    """
    try:
        exclusions = validate_exclusions(json.loads(exclusions_text)["exclusions"])
    except ValueError as error:
        return {"schema": DIFF_SCHEMA, "version": VERSION, "compared": 0,
                "stale_open_bead_exclusions": [],
                "failing": [{"subject": "*", "verdict": "invalid-exclusions", "error": str(error)}]}
    reference = parse_ndjson(reference_text)
    if sweep_text is None:
        portal = list(extract_constructions(engine_id, points_mode="full"))
        scope = "constructions"
    else:
        sweep = json.loads(sweep_text)
        portal = list(extract_constructions(engine_id, points_mode="full", api_names=True,
                                            constructions=list(class_sweep_constructions(sweep))))
        scope = "classes"
    results, summary = diff_files(reference, portal, exclusions, scope=scope)
    compared = sum(1 for r in results if r["verdict"] != "one-sided")
    failing = [r for r in results if r["verdict"] in ("differs", "one-sided")]
    summary.update(compared=compared, failing=failing)
    if compared == 0:
        summary["failing"].append({"subject": "*", "verdict": "no-subjects-compared"})
    return summary


def envelope_drill(reference_text: str, exclusions_text: str, engine_id: str, subject: str = "tex") -> dict:
    """One envelope violation, end to end (fm-5wq.45).

    Extract `subject` from the construction set in the running engine and diff it
    against the Reference facts under the table; then double its root's bbox height
    about its centre (the 2x-tall Tex the blanket rows once hid) and diff again.
    Returns both verdicts and the envelope the planted defect broke.
    """
    rows = in_scope(validate_exclusions(json.loads(exclusions_text)["exclusions"]), "constructions")
    reference = {r["subject"]: r for r in parse_ndjson(reference_text)}[subject]
    portal = next(extract_constructions(engine_id, points_mode="full", only={subject}))
    clean = diff_subject(reference, portal, rows)
    planted = json.loads(canonical(portal))
    (x0, y0, z0), (cx, cy, cz), (x1, y1, z1) = planted["members"][0]["bbox"]
    planted["members"][0]["bbox"] = [[x0, cy - (y1 - y0), z0], [cx, cy, cz], [x1, cy + (y1 - y0), z1]]
    result = diff_subject(reference, planted, rows)
    return {"subject": subject, "clean_verdict": clean["verdict"], "planted_verdict": result["verdict"],
            "violation": (result["first_difference"] or {}).get("envelope_violation")}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    ext = sub.add_parser("extract")
    ext.add_argument("--engine-id", required=True)
    ext.add_argument("--points", choices=("digest", "full"), default="digest")
    ext.add_argument("--only", nargs="*")
    ext.add_argument("--sweep", help="a class-sweep JSON: extract its classes (public methods included)")
    ext.add_argument("out")
    dif = sub.add_parser("diff")
    dif.add_argument("reference")
    dif.add_argument("portal")
    dif.add_argument("--exclusions")
    chk = sub.add_parser("check", help="extract here and diff against a Reference fact file")
    chk.add_argument("--engine-id", required=True)
    chk.add_argument("--reference", required=True)
    chk.add_argument("--exclusions", required=True)
    chk.add_argument("--sweep", help="a class-sweep JSON: check its classes instead of the construction set")
    run = sub.add_parser("run-scene", help="run one scene through an engine CLI, recording facts")
    run.add_argument("--facts", required=True)
    run.add_argument("--engine", choices=("reference", "portal"), required=True)
    run.add_argument("--points", choices=("digest", "full"), default="digest")
    run.add_argument("engine_argv", nargs=argparse.REMAINDER)
    original = list(sys.argv[1:] if argv is None else argv)
    args = parser.parse_args(argv)
    if args.command == "run-scene":
        engine_argv = args.engine_argv[1:] if args.engine_argv[:1] == ["--"] else args.engine_argv
        return run_scene(args.facts, args.engine, engine_argv, args.points)
    # The Reference's manimlib parses sys.argv when imported (manimlib/config.py).
    # It must see a bare invocation, not this tool's flags.
    sys.argv = sys.argv[:1]
    if args.command == "extract":
        out = Path(args.out)
        if out.exists():
            print(f"refusing to overwrite {out}", file=sys.stderr)
            return 2
        if args.sweep:
            sweep = json.loads(Path(args.sweep).read_text(encoding="utf-8"))
            records = extract_constructions(args.engine_id, args.points, args.only, api_names=True,
                                            constructions=list(class_sweep_constructions(sweep)))
        else:
            records = extract_constructions(args.engine_id, args.points, args.only)
        lines = [canonical(r) for r in records]
        header = provenance_line(args.engine_id, original)
        out.write_text("".join(line + "\n" for line in [header, *lines]), encoding="utf-8")
        print(canonical({"schema": SCHEMA, "written": str(out), "subjects": len(lines)}))
        return 0
    if args.command == "check":
        summary = check_constructions(Path(args.reference).read_text(encoding="utf-8"),
                                      Path(args.exclusions).read_text(encoding="utf-8"),
                                      args.engine_id,
                                      Path(args.sweep).read_text(encoding="utf-8") if args.sweep else None)
        print(canonical(summary))
        return 1 if summary["failing"] or summary["stale_open_bead_exclusions"] else 0
    exclusions = load_exclusions(args.exclusions) if args.exclusions else []
    results, summary = diff_files(read_ndjson(args.reference), read_ndjson(args.portal), exclusions)
    for result in results:
        print(canonical(result))
    print(canonical(summary))
    failing = any(r["verdict"] in ("differs", "one-sided") for r in results)
    return 1 if failing or summary["stale_open_bead_exclusions"] else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        sys.exit(1)
    except Exception:  # noqa: BLE001
        traceback.print_exc()
        sys.exit(3)
