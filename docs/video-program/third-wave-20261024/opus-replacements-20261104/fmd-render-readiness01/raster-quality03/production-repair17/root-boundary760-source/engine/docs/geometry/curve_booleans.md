# Curve-aware path booleans: degeneracy-class topology specification

Status: normative for `fmn_geom::boolean` Stage-2 routing (fm-qjy, building on
fm-8dx). This document is the written topology specification the route
discipline requires *before* any curve-aware degeneracy class is admitted.

The permanent certified implementation is Stage 1 (`path_boolean_flattened`):
quadratics are flattened under a caller-visible error bound and the boolean is
computed on the resulting line arrangement. Stage 2 never replaces it. A
curve-aware class is *admitted* only when its acceptance proof obligation
below has landed as executable tests; anything unsupported or unproved routes
to Stage 1 permanently. Routing is observable per result via
`BooleanResult::route`.

Nothing in Stage 2 claims coordinate exactness. Where a numerical criterion is
used, this document states the criterion and its bound; double-double
arithmetic is used only where a sign decision must match the Stage-1
arrangement's predicates (orientation of control triples, area
accumulation), never as a claim of exact geometry.

## 1. Degeneracy-class taxonomy

Two closed quadratic operands (subject, clip) interact through their curve
*pieces* (quadratic Bézier segments; straight segments are quadratics whose
handle lies on the chord, and an open subpath contributes its implicit
fill-closing chord as a straight piece, matching Stage-1 fill semantics).
Every operand pair falls into exactly one of these classes. Classes C2–C5 are
degeneracy classes; they are the ratchet's backlog.

### C0 — separated

The filled sets cannot meet. Proved form: `separated-control-hulls`
(fm-8dx) — complete quadratic control-point bounds strictly separated on x or
y, both operands with nonzero-area contours, NonZero fill. Equality at a
bound is deliberately *not* C0.

### C1 — transversal interiors

Every intersection between any two pieces of *different* operands is a proper
crossing strictly inside both pieces: the crossing parameter lies in the open
interval (with the margin of §3.4), the tangents are linearly independent past
the sine threshold of §3.4, and the intersection solver converges under its
documented bounds (§3.2, §3.3). Same-operand piece pairs meet only at shared
contour endpoints (§3.5). Zero cross-operand intersections is the vacuous
member of the class: the operands are then nested or disjoint without contact
(hulls may overlap — that is what distinguishes it from C0), and the
classification phase decides which contours are boundaries.

Admission additionally requires:

- both operands have at least one contour whose exact quadratic boundary
  integral has nonzero double-double sign (zero-area inputs are C5). This
  admits genuine two-anchor lenses whose anchor shoelace is zero while their
  curved boundary encloses area;
- at least one genuinely curved piece across the two operands: a piece
  whose handle is *meaningfully* off the chord, concretely
  `|cross(p1−p0, p2−p0)| / |p2−p0| > maxᵢ resolution(pᵢ)`, where
  `resolution([x,y]) = 256·ε·max(|x|,|y|)` (and the smallest positive f64 at
  the origin). The threshold matters: `QuadPath` line segments and affine
  transforms store handles through rounded arithmetic, so a geometric line's
  handle can sit a few ulp off its chord — an exact-sign or chord-only test
  would misread translated slanted polygons as curves. All-line inputs lose
  nothing under flattening — Stage 1 is exact for polygons — so they route to
  Stage 1 by construction and the captured skia-pathops fixtures (all
  polygonal) permanently exercise Stage 1;
- every piece is non-degenerate: `p0 ≠ p1`, `p1 ≠ p2`, `p0 ≠ p2` (endpoint
  derivatives nonzero, no closed-loop single pieces);
- within one operand, all anchors are pairwise distinct except the closure
  identification of a contour's first and last anchor (pinch points and
  self-touching contours are C2/C5);
- fill rules: NonZero and EvenOdd are both admitted — classification consumes
  winding *counts*, and both rules are functions of those counts.

### C2 — shared endpoints

Any cross-operand intersection at (within the interior margin of) a piece
endpoint: corner touches, endpoint-on-curve, shared vertices. **Not admitted;
routes to Stage 1.** Admission requires a vertex-topology proof (degree-6+
arrangement vertices and endpoint winding transitions) that has not landed.

### C3 — tangencies

Any cross-operand contact where the tangents are parallel past the sine
threshold (touching without crossing), including higher-order contacts.
**Not admitted; routes to Stage 1.** The solver detects these as
transversality failures or as convergence stalls; both refuse the class.

### C4 — overlaps

Any pair of pieces whose intersection is a curve segment, not a finite point
set (coincident quadratics, including differently parameterized copies of the
same curve). The solver detects these as non-shrinking clip intervals.
**Not admitted; routes to Stage 1.**

### C5 — zero-area and degenerate inputs

Backtracking contours, zero-length pieces, repeated anchors, pinch points,
and operands with no nonzero-area contour. **Not admitted; routes to
Stage 1**, which cancels zero-area edge pairs by winding multiplicity.

## 2. The admitted C1 pipeline

Stage 2 computes the same arrangement as Stage 1, but its atomic edges are
quadratic pieces, not line segments:

1. **Piece collection.** Each subpath becomes closed pieces; a last anchor
   within `options.tolerance` of the first is snapped to it (mirroring Stage
   1's flatten snap), otherwise the closing chord is appended as a straight
   piece. C5 degenerates refuse the class here.
2. **Intersection.** Every same-operand pair is proved to meet only at shared
   endpoints; every cross-operand pair is solved (§3). Each cross-operand
   root must verify the C1 predicate (§3.4), else the class is refused.
3. **Splitting.** Pieces are split at their root parameters by de Casteljau
   subdivision (§3.6).
4. **Winding classification.** Each atomic edge's left face is sampled and
   both operands' winding numbers are computed by ray casting over the atomic
   edges (§4). The operation's truth value on both sides selects boundary
   half-edges.
5. **Stitching.** Boundary half-edges are traced into closed loops by the
   same angular-sort face traversal as Stage 1, with tangents in place of
   segment directions (§5). Loops emit as closed quadratic subpaths.

Determinism: every decision is a function of the input f64 values only; no
hash iteration order, no wall clock, no allocation-address dependence.

## 3. Intersection machinery

### 3.1 Fat lines

For quadratic piece `B` with controls `(b0, b1, b2)`, take the baseline
through `b0, b2` with unit normal `n`. The signed distance of `B(t)` from the
baseline is `2·d1·t·(1−t)` where `d1 = n·(b1 − b0)`, so `B` lies in the *fat
line* `[min(0, d1/2), max(0, d1/2)]` — the tight quadratic bound (Sederberg).
For a straight piece `d1 = 0` and the fat line is the piece itself. Piece `A`
is pruned when its control-point distances to the baseline all lie outside
the fat line.

### 3.2 Bézier clipping with bounded subdivision

`A`'s distance polynomial (Bernstein coefficients = control-point distances)
is intersected with the fat line; the convex hull of the in-band parameter
set clips `A`'s interval. The roles alternate. Convergence is quadratic at
transversal roots. Bounds (documented, not adaptive):

- at most 24 clip rounds per interval pair per recursion node;
- if both intervals shrink by less than 20% for two consecutive rounds, the
  wider interval is bisected and both halves are processed;
- recursion depth at most 6, at most 512 clip nodes per piece pair;
- an interval pair converges when both widths drop below `1e-10`.

Exhausting any bound refuses the pair, which refuses the class. Roots from
different nodes closer than `1e-9` in both parameters are merged; two roots
on one piece closer than `1e-9` refuse the class (near-double root — C3
territory).

Two consequences of the pruning invariant are part of the contract:

- The in-band set of the distance polynomial is computed from its cuts
  against the band edges, and a contact at an isolated parameter (an
  endpoint touch or a tangency against the band edge) has measure zero:
  in-band cut *points* themselves extend the convex hull, and a cut that
  rounds to just outside `[0, 1]` (an exact endpoint root) is admitted
  within the root-merge band and clamped. A fat-line miss is then a
  *proved* empty branch: it contributes no roots and is not a solver
  failure. Only budget exhaustion, convergence stall past the bisection
  bound, or a degenerate bisection refuses the pair.
- When clipping collapses an interval to a single parameter, the contact
  is resolved exactly rather than iterated on a degenerate subcurve:
  point-on-curve incidence of the collapsed point against the other
  piece is solved by per-axis quadratic root-finding (bounded,
  cancellation-screened), and a miss prunes the node.

### 3.3 Bounded Newton polish

Each clip root `(t, u)` seeds Newton iteration on `F(t, u) = A(t) − B(u) = 0`
against the *original* (unsplit) pieces:

- at most 16 iterations;
- converged when `|F| ≤ 4·ε·max(1, s)` where `s` is the largest absolute
  control coordinate of the pair and `ε` is machine epsilon — the root is
  then representable, and repeated polishes from different seeds in the same
  basin land on the same f64 point to within one ulp;
- a Jacobian determinant below `1e-13·|A'|·|B'|` declares a singular contact
  (C3/C5) and refuses the class;
- divergence or leaving `[0, 1]²` refuses the class.

### 3.4 The C1 acceptance predicate

A polished root is a transversal interior crossing iff all of:

- `t, u ∈ (m, 1−m)` with interior margin `m = 1e-9` (else C2);
- `|cross(A'(t), B'(u))| ≥ 1e-6·|A'(t)|·|B'(u)|` — the crossing sine is
  bounded away from zero (else C3);
- both derivative magnitudes are nonzero (else C5 cusp).

Every cross-operand root of every pair must pass; any failure refuses the
class. These are *screening* thresholds: they never assert geometric truth
beyond their stated tolerances, they only decide routing.

### 3.5 Same-operand admissibility

For pieces of the same operand, every solver root must lie (within the
interior margin) at an endpoint shared by the two pieces — the contour
joints. Any other root is a self-intersection, pinch, or overlap (C2/C4/C5)
and refuses the class.

### 3.6 Exact quadratic splitting

A quadratic split at parameter `t` by de Casteljau subdivision is exact in
the curve's parameterization: the two halves re-parameterize the same point
set, and the shared anchor is computed by one evaluation so the halves join
by construction. Coordinates are f64-rounded, never claimed exact. At a
cross-operand root the two operands' split anchors are snapped to one shared
vertex — the midpoint of `A(t)` and `B(u)` after polish — so the arrangement
is watertight; the snap moves each curve by at most the §3.3 residual. This
vertex snap is the only coordinate mutation Stage 2 performs, and it is
accounted for in the differential band of §6.

## 4. Winding classification

For each atomic edge, the left-face sample is the edge midpoint offset along
the left normal of the mid-derivative by

`offset = min(tolerance, chord, clearance) · 0.25`,

where `clearance` lower-bounds the distance to every other atomic edge by its
control hull (the curve lies inside that hull, so hull distance bounds curve
distance from below). When the midpoint lies inside a broad control triangle,
that zero bound is tightened through at most 8 levels of midpoint de Casteljau
subdivision: the union of the child hulls still contains the complete curve,
so the minimum child-hull distance remains a valid lower bound. A zero bound
that survives the fixed depth, an offset at or below the coordinate resolution
of the sample, or a sample whose offset exceeds the clearance after repeated
halving refuses the class.

Winding numbers at a sample are ray-cast: crossings of the ray `y = y_s`,
`x > x_s` with each atomic edge are the roots of the edge's quadratic
`y(t) = y_s` on the half-open interval `t ∈ [0, 1)`, signed by `y'(t)` —
the same half-open convention as Stage 1, so split edges and unsplit pieces
pieces contribute identically (a root within the snap band of `t = 1` is
assigned to the next atomic edge, and a contact strictly behind the ray
origin cannot affect the count, so its endpoint status is irrelevant).
The quadratic is solved in cancellation-stable form; the naive formula
maps the near-linear case (a line piece whose rounded midpoint handle
leaves the second coefficient nonzero) to a spurious exact `t = 0`,
which the endpoint rule would refuse for every retry. A ray whose
discriminant vanishes within rounding of an edge apex, that is tangent
to an edge (`y'(t)` vanishing within rounding at a root), or that passes
exactly through it (`x(t) = x_s`) refuses the *sample*, and
classification retries with a new offset. Retries halve the offset and
add a deterministic tangential nudge (attempt index times `1e-3` of the
offset): halving alone cannot move the ray height when the mid-normal is
horizontal, and the nudge keeps the sample on the left face and inside
the clearance ball around the midpoint.
Crossing an atomic edge changes its operand's winding by the edge's unit
contribution (admission excludes every multiplicity-carrying class), so the
right-face winding is the left-face winding minus the contribution, and the
operation's truth value on both sides selects boundary half-edges exactly as
in Stage 1.

## 5. Stitching

Half-edges carry departure tangents at each vertex; outgoing half-edges are
angularly sorted per vertex by the same double-double orientation comparator
as Stage 1. Face traversal from each unvisited boundary half-edge follows
the same next-before-the-twin rule as Stage 1. At a C1 crossing the vertex
has degree 4; at a contour joint, degree 2; both are handled by the one
rule. A traced loop whose double-double signed area — the exact quadratic
boundary integral `⅓·cross(p0,p1) + ⅙·cross(p0,p2) + ⅓·cross(p1,p2)` per
piece — has sign zero refuses the class (a sliver is C5 topology, not C1
output). Loops are emitted as closed quadratic subpaths in deterministic
(sorted) order.

## 6. Acceptance proof obligation (per admitted class)

For each admitted class, all of the following must land as executable tests
before the route is observable:

1. **Differential point-in-fill proof** against forced Stage 1 on jittered
   grids, excluding only samples inside the documented band (Stage-1 flatten
   tolerance plus the §3.6 vertex-snap residual) around the flattened input
   boundary.
2. **Winding equivalence**: at grid samples, the result's winding is nonzero
   (NonZero) resp. odd (EvenOdd) iff the operation's truth value of the
   operands' windings holds.
3. **Components/holes/Euler characteristic**: component and hole counts, and
   `components − holes`, agree with forced Stage 1, including nested-hole
   scenes.
4. **Boolean identities**: `A∪B == B∪A`, `(A∩B) ⊔ (A−B) == A`,
   `A∩B == A−(A−B)`, `(A∪B)−A == B−A`, verified as point-in-fill equalities,
   including nested evaluations that feed boolean outputs back as inputs.
5. **Multi-scale raster checks**: both routes' outputs rasterized at several
   resolutions; mismatch cells must be boundary cells (a 3×3 neighborhood of
   a Stage-1 transition) and below a documented fraction.
6. **Captured skia-pathops fixtures**: the polygonal corpus must still route
   to Stage 1 (C1 excludes all-line inputs by construction) and pass its
   captured topology assertions unchanged.
7. **Deterministic resource-budget errors**: with any phase budget zeroed,
   `path_boolean` produces exactly the forced-Stage-1 outcome (typed
   `ResourceLimit` or success), deterministically; curve-route work counters
   never exceed declared budgets.
8. **Adversarial fuzzing**: 2000+ deterministic cases mixing admissible and
   degenerate shapes — never panic, budgets respected, route discipline held
   (refused classes route to Stage 1), differential fill agreement outside
   the documented band. The conformance fuzz campaign's boolean target
   covers the never-panic contract on raw bytes.

## 7. Current ratchet state

| Class | Status | Route |
|---|---|---|
| C0 separated-control-hulls | admitted (fm-8dx) | `CurveAwareSeparated` |
| C1 transversal interiors | admitted (fm-qjy) | `CurveAwareTransversal` |
| C2 shared endpoints | specified, unproved | `FlattenClip` permanently |
| C3 tangencies | specified, unproved | `FlattenClip` permanently |
| C4 overlaps | specified, unproved | `FlattenClip` permanently |
| C5 zero-area/degenerate | specified, unproved | `FlattenClip` permanently |

"Permanently" is the ratchet's default: a future class-specific proof battery
(§6) is the only mechanism that moves a row, and moving a row never removes
or weakens the Stage-1 fallback.
