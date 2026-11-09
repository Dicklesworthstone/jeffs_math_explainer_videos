# BN-04 — Callable color fields on native record schemas

**Status:** Draft. Implemented with native runtime witnesses; not a full certification gate.
**Plan:** §8.2, §10.2, §15 · **Related beads:** fm-sq8, fm-5wq.4.143


`set_color_by_rgb_func` and `set_color_by_rgba_func` now operate on the native
record schema: point and surface records keep their `rgba` column, while
VMobjects receive the same callable colors in `fill_rgba` and `stroke_rgba`.
Previously, these inherited methods raised a missing-`rgba` exception on common
vector shapes such as `Square`, `Circle`, and `Line`. This is a deliberate
correction, not reproduction of the Reference's fixed-column limitation.

The callback still receives each point-bearing family member's live point
array in deterministic family order. Empty members are skipped, shared members
are evaluated once, and `recurse=False` stays local to the receiver. Both a
constant color vector and one row per point are admitted. Values must be real,
finite and f32-representable; values outside the usual zero-to-one color range
are not silently clipped. The RGB method applies its explicit opacity, whose
default remains one.

All callback outputs are frozen and validated before publishing any paint. A
late callback exception or invalid result therefore leaves earlier members'
paint untouched. Reentrant coloring and callback-induced family/geometry edits
refuse the obsolete paint plan. Authored side effects are not rolled back.
Public `set_rgba_array` overrides still receive the final writes and retain
responsibility for their own effects; failures in arbitrary authored setters
are not advertised as rollback-safe.

No shader language is admitted by this correction: `set_color_by_code` and
the GLSL-snippet form of `set_color_by_xyz_func` still give the existing named
capability refusal. Callable fields evaluate on the host, write existing live
record columns, and use the same native fill/stroke renderer and cache.

Evidence: `crates/fmn-python/tests/functional_color.py` exercises real native
record layouts, bound views, mixed families, shared children, override dispatch,
invalid-input recovery and actual rendered stroke pixels. Full fill pixel and
animation witnesses are described in `BN-06-analytic-fill.md`.
