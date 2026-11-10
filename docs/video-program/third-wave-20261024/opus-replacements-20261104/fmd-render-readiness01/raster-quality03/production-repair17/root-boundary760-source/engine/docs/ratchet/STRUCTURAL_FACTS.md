# Structural facts — schema `fmn.structural-facts` v1

One extractor, one diff, both engines (fm-5wq.36). The code lives in
`crates/fmn-conformance/python/structural_facts.py` and runs unmodified in the
pinned Reference (3b1b/manim @ `6199a00d`, under `xvfb-run`) and in the
FrankenManim portal. It reads only the public manimlib API, so both engines
are measured by the same code. The class sweep (fm-5wq.26), the corpus
differential (fm-5wq.33) and the differential fuzzer (fm-5wq.34) consume it
and add no second extractor.

## Record

One JSON object per *subject*: a construction, a class, or a scene's
`self.mobjects` at `tear_down`.

| key | meaning |
|---|---|
| `schema`, `version` | `fmn.structural-facts`, `1` |
| `quanta_per_unit` | `1000`: every length is an integer count of 1e-3 scene units |
| `subject` | stable identifier (construction id, class name, scene name) |
| `source` | the construction's source text (constructions only) |
| `engine` | `{engine: reference or franken_manim, engine_id}`, excluded from the diff |
| `members` | depth-first members of every root, see below |
| `error` | instead of `members` when construction raised: `"<ExceptionClass>: <first line>"` |
| `truncated_at` | present only if the family exceeded 20,000 members |

Each member has these fields:

| key | meaning |
|---|---|
| `path` | dot-joined submobject indices under the root index: `"0"`, `"0.1"`, `"0.1.3"` |
| `class` | `type(m).__name__` |
| `mro` | the class chain restricted to names the engine's `manimlib` exports as the same object. Private names and stdlib classes (`Generic`, `ABC`) are dropped. |
| `n_points`, `points_sha256` | point count, and sha256 of the quantized points as signed 64-bit little-endian integers |
| `points` | the quantized point rows (only in `--points full`, up to 200,000 points) |
| `bbox` | quantized `get_bounding_box()` (min, center, max) |
| `data_fields` | `data.dtype.names` |
| `z_index` | quantized |
| `string` | `m.string` for StringMobject members |
| `style` | VMobject only: fill/stroke color (uppercase hex), fill/stroke opacity and stroke width (quantized) |
| `getters` | declared zero-argument getters per public class (`GETTERS` table), quantized when numeric. The value `"absent"` or `"raises <Class>"` is itself a fact. |

**Quantization** is `round(x * 1000)`, half to even. Non-finite values map to
distinct integer sentinels, and magnitudes are clamped at 2^61.

**Canonical form** is JSON with sorted keys and no whitespace. Records hold
only integers, strings, booleans, null, lists and objects, so equal facts are
equal bytes.

Changing the `GETTERS` table, the member fields, ordering or quantization is
a **version bump**.

## Diff (`fmn.structural-diff` v1)

- Members are aligned by `path`. A path present on one side is a `member`
  difference.
- Quantized lengths (`points`, `bbox`, `z_index`, `getters.*`, `style.*`)
  compare with one quantum of slack. That absorbs values straddling a
  rounding boundary between two f64 paths; a move of two quanta (2e-3) is a
  difference. Everything else is exact: class, MRO, counts, field names,
  strings, digests.
- A subject that raised on both sides with the same exception class is equal.
- Verdicts are `equal`, `equal-with-exclusions`, `differs` and `one-sided`.
  The first difference is reported with its path, fact, and both values.

## Exclusions (`crates/fmn-conformance/fixtures/structural_facts/exclusions.json`)

Each row names `subject`, `class` (Reference side) and `fact`, each a glob or
a list of globs, and cites its `ref`. The first matching row wins.

- `behavior-note` rows cite a BN and cover exactly what the note documents.
- `open-bead` rows cite a bead for a known bug. A row that matches nothing in
  a run is **stale** and fails the diff, so a fixed bug forces its row out.

### Envelopes (fm-5wq.45)

A Behavior Note justifies a different outline or point count. It never
justifies an unbounded size or position: a formula twice as tall, or a brace
of zero width at the origin, is a bug under any BN. So every `behavior-note`
or `adr` row that covers `bbox` carries an `envelope`, and validation (the
loader, and `check` before it extracts anything) refuses a row without one:

- `{"size_rel": r, "center_abs": c}`: per axis, the portal's extent is within
  `r` of the Reference's extent. Its centre is within `c` units, plus half
  the extent change, because an object placed by one edge (`next_to`,
  `to_edge`) moves its centre by half of any size change. Quantization slack
  is two quanta on an extent and one on a centre.
- `{"bounded_by": "ancestor"}`, on a row with `under`: glyph members align
  by index only, so they defer to their text or native root. Every `under`
  class must itself have a numeric envelope (or an open-bead row).

A `bbox` outside a row's envelope falls through to later rows. An `open-bead`
row may state its bug's measured magnitude as an envelope (Tex formulas up to
40% off in size, `fm-tex-metrics-glyphs-ru72`), so the bug stays admitted, a
worse one still fails, and a fix makes the row stale. A `bbox` that no row
admits is a difference with an `envelope_violation` naming the first envelope it
broke, the size change and the centre offset. The diff record lists every
violation under `envelope_violations`.

## Running it

```bash
# Reference facts (regenerate only on purpose; the header line records the command).
# Run from an empty directory whose custom_config.yml gives the Reference a private
# base directory (its latex_cache) and SVG cache. The default cache, ~/.cache/manim,
# is shared by every Reference run on the host, and concurrent runs poison it
# (Appendix C-22, fm-0v8k).
mkdir ref-facts && cd ref-facts
printf 'directories:\n  base: "%s/out"\n  cache: "%s/svg"\n' "$PWD" "$PWD" > custom_config.yml
PYTHONPATH=$REPO/scripts/manim_ref xvfb-run -a <ref-venv>/bin/python \
  $REPO/crates/fmn-conformance/python/structural_facts.py extract \
  --engine-id 3b1b/manim@6199a00d4c1b1127ebe45cb629c3f22538b10e13 --points full OUT.ndjson

# Gate: installed portal against the checked-in Reference facts (check_portal_runtime.sh)
python3 crates/fmn-conformance/python/structural_facts.py check --engine-id installed-wheel \
  --reference crates/fmn-conformance/fixtures/structural_facts/reference_constructions.v1.ndjson \
  --exclusions crates/fmn-conformance/fixtures/structural_facts/exclusions.json
```

The e2e scenario `parity.structural_facts.v1` runs the same check in the
embedded portal. `test_structural_facts.py` holds the unit tests.

## Class sweep (fm-5wq.26)

`crates/fmn-conformance/fixtures/structural_facts/class_sweep.json` lists every public mobject class in plan Appendix A (153 classes; private helpers such as `_AnimationBuilder` are excluded) and declares parameterized calls for the classes whose arguments matter (131 calls). Each class is constructed with its default call (subject `Class`), and each declared call gets subject `Class#n`. A default call that raises in both engines with the same exception class is equal; the error text records why the class is unconstructible without arguments.

Sweep records add `public_methods`: the public callables of the instance's class, recorded once per class on the default call. A Reference method missing from the portal is a difference (`public_methods.<name>`). Portal extras are not compared. Instance data attributes are not compared either: the Reference's include GL renderer state that the portal deliberately lacks (ADR-0021).

```bash
python3 crates/fmn-conformance/python/structural_facts.py check --engine-id installed-wheel \
  --sweep crates/fmn-conformance/fixtures/structural_facts/class_sweep.json \
  --reference crates/fmn-conformance/fixtures/structural_facts/reference_classes.v1.ndjson \
  --exclusions crates/fmn-conformance/fixtures/structural_facts/exclusions.json
```

The same check runs in the portal gate and, embedded, as the e2e scenario `parity.class_sweep.v1`. It takes seconds: 19 s for the Reference and 8 s for the portal on the dev host.

**First measurement (portal wheel from `35c65d6e`):**
- No Reference public method is missing from the portal on any class.
- 162 constructions equal, 122 equal only through cited rows, 0 untriaged.
- Planted negative: re-introducing origin-relative Cone scaling fails the gate on `Cone#1`'s bbox.

## Exclusion row conditions

Besides `subject`, `class` and `fact` (a glob or list of globs each), a row may carry:
- `under` (an ancestor's class);
- `empty_only` (the Reference member's whole family has no points);
- `mro_missing` (the difference is exactly those missing names);
- `scope` (`constructions`, `classes`, `scenes`, a list, or `any`).

`kind` is one of:
- `behavior-note` (cites BN-nn);
- `adr` (cites ADR-nnnn);
- `open-bead` (cites fm-…; stale when it matches nothing in its scope, which fails the run).

## Corpus differential (fm-5wq.33)

`scripts/corpus_differential.py` runs candidate corpus scenes through both
engines' own CLIs under `structural_facts.py run-scene`. `run-scene` wraps
`Scene.tear_down` and records `scene.mobjects`; scene source is never edited.
Both engines run in skip mode (`-s`, 320×180) from the corpus workdir.

A scene's `time` fact is its clock at `tear_down`. Skipped, the Reference
advances by each segment's `run_time` while played it ends on the frame at or
after it (Appendix C-28), so for the Reference `run-scene` records the clock
playback would reach, without changing what the Reference does. It also
records `time_bn02`: the same segments counted by BN-02's rule, which can be
one frame longer than `arange`'s count (`wait(0.1)` at 30 fps). A portal time
equal to `time_bn02` is the fact `time.bn02`, which the `bn02-frame-count`
row admits; any other time difference is a difference.

Each scene gets two verdicts:
- **structure**: the diff with `GEOMETRY_FACTS` ignored;
- **geometry**: the full diff.

Under BN-05, text metrics move everything laid out relative to text, so a
text-bearing scene can be structurally equal and still differ in geometry.

Scenes that ran in only one engine are labeled by rule-based triage:
`reference-defect` (for example C-18), `environment`, `portal-leniency` or
`untriaged`. Final frames get SSIM/MAE smoke metrics, which are never gates.

`--scene MODULE:SCENE` targets single scenes, and `--minimize MODULE:SCENE`
delta-debugs `construct` to a minimal structure-differing variant, written
under `--out` only; with `--minimize-envelope` the target is the scene's first
envelope violation. `--rediff DIR` recomputes verdicts from saved facts under
the current table and reports stale open-bead rows. `--report DIR --dashboard
PATH` renders the outcome-code dashboard, including envelope violations
grouped by (class, broken envelope). Records carry scene identifiers and outcome codes only, because
the corpus is CC BY-NC-SA.

## First measurement (2026-09-27, portal wheel from `fac2e515`)

25 constructions: 15 equal, 10 equal with exclusions, 0 unexcluded.

- **BN rows:** BN-05 glyph geometry, BN-08 Brace and DecimalNumber structure,
  and BN-09 Annulus density. The Reference's `quadratic_bezier_points_for_arc`
  defaults to 8 components; BN-09's one rule gives 16.
- **Bugs found and filed, carried as open-bead rows:**
  - fm-16u6: `VGroup` is not a `Group`;
  - fm-78b6: style getter defaults;
  - fm-15k9: glyph submobject classes.

The Reference facts are byte-identical across two extractions.
