# The Boundary Trick Behind Stokes' Theorem

**Research narration and visual draft.** Owner-requested Stokes video, 2026-11-02. No production source, recording, render or publication is represented by this file. Timing estimates, on-screen typography, source claims and spoken phrasing need mathematical, pedagogical, accessibility and independent audiovisual review. Proposed pauses follow the question, never its answer. ElevenLabs is the current narration direction; no voice take has been generated or selected for this draft.

Audience: a curious calculus learner; no prior differential forms assumed. The box calculation is an actual arbitrary-dimensional proof for that domain. The manifold extension states and uses standard chart, change-of-variables and partition-of-unity lemmas; it is not a self-contained proof of those lemmas. Chapter divisions and run time are provisional and must be measured after real narration.

## 01 — The disappearing seams

**Picture:** Two neighboring translucent square tiles. Each has a bright counterclockwise arrow running around its four edges. On the shared edge, draw two separately visible arrows in opposite directions. Show the question without a cancellation animation. Hold after the spoken question.

**Narration:** Imagine that we walk around the edge of one square, keeping the square on our left. Now add a second square next to it, and walk around that one the same way. On their shared edge, the two walks go in opposite directions. If we add both walks, which edges survive?

**Picture after hold:** Cancel the two shared arrows. Expand to a moving grid; only the outside boundary remains highlighted. Then bend the tiled grid into a cap, and finally replace it with a wire loop threaded by a changing magnetic field. Return to the grid.

**Narration:** The interior seam cancels. Only the outside edge is left. That small cancellation is the visible side of a theorem that connects a change inside a region to an accumulation around its boundary. We will start on a line, build up to surfaces and volumes, and then prove the same pattern in any dimension. At the end, it will explain one of Maxwell's equations.

## 02 — The one-dimensional theorem

**Picture:** An oriented interval from a to b subdivides into three intervals. At every subdivision draw separate plus and minus endpoint badges. Tie their removal to displayed terms; do not remove the two end badges.

**Narration:** On a line, a boundary is just a pair of endpoints, with a sign. The boundary of the interval from a to b is b minus a. If we cut the interval in two, the cut point is the positive end of one piece and the negative end of the other. Add the boundaries and that point disappears.

**Picture:** Animate the three differences `f(x₁)−f(a)`, `f(x₂)−f(x₁)`, `f(b)−f(x₂)` into a single `f(b)−f(a)`. Then display the integral of f prime on the original interval.

**Narration:** The same cancellation happens with changes of a function. The change across the whole interval is the sum of the changes across its pieces. For a continuously differentiable function, the fundamental theorem of calculus says that adding the derivative over the interval gives the value at b minus the value at a. This is our first Stokes theorem: an integral over an interval equals a signed value on its boundary.

## 03 — From endpoints to edges

**Picture:** Revisit two oriented square tiles. Use distinct colors or line styles for opposite shared-edge orientations, not color alone. Then reveal the same cancellation on a curved patch; number of tiles is finite here.

**Narration:** For a region on a plane, the boundary is no longer a pair of points. It is a collection of oriented curves. Give each little tile a counterclockwise boundary. Neighboring tiles traverse their common edge in opposite directions, so the signed contributions from that edge cancel. When we add every tile, the outside edge survives.

**Picture:** A small arrow representing a step moves through a field of arrows. Its dot product with a local field arrow becomes a signed scalar. Reverse the step, and the sign changes.

**Narration:** What are we adding along those edges? One useful answer is work: along a small step, take the component of a vector field in the direction of that step. Reverse the step and the contribution changes sign. A differential one-form is a general rule for assigning such a signed contribution to a small oriented step. The vector field is a familiar picture; the one-form is the object we integrate.

## 04 — A square you can calculate

**Picture:** Unit square with coordinates, counterclockwise rim and `ω = x dy`. Do not show the numerical edge contributions until the question has been spoken and held.

**Narration:** Here is a one-form simple enough to calculate by hand: omega equals x times d y. Go around the unit square counterclockwise. Which edge contributes anything? Take a moment.

**Picture after hold:** Bottom and top arrows get `dy=0`; left gets `x=0`; right gets `x=1`, `dy` upward. Reveal result 1. Inside, show `dω = dx∧dy`, the oriented area form; highlight unit area.

**Narration:** Along the horizontal edges, d y is zero. Along the left edge, x is zero. On the right edge, x is one and y increases by one, so the whole boundary integral is one. The exterior derivative of x d y is d x wedge d y. On the unit square, its integral is the area, also one.

**Picture:** Reverse the region's orientation and boundary arrows together; both displayed integrals change sign.

**Narration:** That wedge symbol records oriented area. Reversing the region reverses its induced boundary direction, and both sides change sign. Signs are part of the geometry, not an optional decoration.

## 05 — A surface in three-dimensional space

**Picture:** Flat unit disk with counterclockwise rim viewed from above and upward normal. Morph the interior into `z = 1 − x² − y²`, keeping the boundary circle fixed. Overlay a mesh whose interior oriented edges cancel.

**Narration:** Now lift the interior of a disk into a cap, keeping the circular rim fixed. This curved surface sits in three-dimensional space, but it is still two-dimensional: two coordinates locate a point on it. Orient the cap upward. The matching boundary direction is counterclockwise when viewed from above. Your right hand links the two: curl your fingers with the rim and your thumb points along the chosen normal.

**Picture:** Put `F = (−y/2, x/2, 0)` next to both the disk and cap. Trace a unit circle, and show `curl F = (0,0,1)`. Reveal `π` on each side after tracing.

**Narration:** Take the vector field whose components are minus y over two, x over two, and zero. Its curl is the constant upward vector. Around the unit circle, the circulation is pi. The upward flux of the curl is also pi, whether we integrate over the flat disk or over this cap. The surfaces share an oriented boundary. We are comparing the flux of the curl, not claiming that every vector field has the same flux through either surface.

## 06 — A volume has faces

**Picture:** Build a solid from translucent cubes. Every cube has outward face arrows; make opposing arrows on a shared face cancel. Remove the interior faces until only the outer shell remains.

**Narration:** Now the object itself is three-dimensional. Its boundary is a two-dimensional surface. Build a solid out of cubes and give each cube outward-facing arrows. On a shared face, the two outward directions oppose each other. Their flux contributions cancel. Only flux through the outside surface remains.

**Picture:** A unit ball with `F=(x,y,z)/3`; inside, `div F = 1`. On its surface, show `F·n = 1/3`. Animate `1 × 4π/3` and `(1/3) × 4π` to the same value.

**Narration:** For a concrete example, take the field x, y, z, divided by three. Its divergence is one. The integral of that divergence over a unit ball is the ball's volume, four pi over three. On the unit sphere, the outward component of the field is one third; multiplying by the sphere's area gives the same answer. This is the divergence theorem. It is a volume version of the same boundary relationship, not the same dimensional surface integral we just used for curl.

## 07 — The boundary you might forget

**Picture:** A field around the origin with an annulus marked `1 ≤ r ≤ 2`; the origin is visibly excluded. Ask the question before revealing either orientation arrow.

**Narration:** Here is a trap. Imagine an annulus: a region with a hole in its middle. Its outside edge is a boundary. Is that the whole boundary? Decide which way each edge should be traversed.

**Picture after hold:** Outer arrow counterclockwise, inner arrow clockwise, keeping the shaded region on the left. Show the angular form and the two signed integrals separately, then their sum.

**Narration:** The inner edge is a boundary too, with the opposite orientation. Consider the angular one-form, minus y d x plus x d y, divided by x squared plus y squared. On this annulus it is smooth, and its exterior derivative is zero. The outer integral is two pi; the clockwise inner integral is minus two pi. Together they give zero, exactly as the theorem predicts.

**Picture:** Try to fill the inner circle with a disk; the undefined origin flashes and the attempted surface is rejected.

**Narration:** If we keep only the counterclockwise loop around the origin, we cannot use a disk containing the origin for this form: the form is undefined there. The missing hypothesis matters. A hole is not a counterexample to Stokes; forgetting a boundary or including a singularity is the mistake.

## 08 — What changes in higher dimensions?

**Picture:** A signed endpoint, an oriented edge, an oriented patch and an oriented volume appear as a sequence, each paired with its boundary. Label dimensions without suggesting that a flat screen displays a literal fourth spatial dimension.

**Narration:** We have integrated values at points, one-forms along curves, and area-type quantities over surfaces. The dimension tells us what kind of object we integrate. At each point, a k-form is an alternating multilinear rule on k tangent directions. In pictures, it assigns an oriented quantity to a tiny k-dimensional piece: scaling a direction scales the result, and swapping two directions reverses its sign. Its exterior derivative is a k-plus-one-form: it measures the boundary accumulation around a tiny piece one dimension higher.

**Picture:** Show the general equation piece by piece: oriented n-dimensional region M, boundary, `(n−1)`-form omega, `d omega` on M. Display assumptions in a clear side panel.

**Narration:** For a compact oriented smooth n-dimensional manifold with boundary, and a smooth form of degree n minus one, generalized Stokes says: integrate the exterior derivative over the manifold, or integrate the original form over its induced boundary. The answers agree. We will first prove it for a box, then say exactly what extra tools take us to a curved manifold.

## 09 — The shape of an n-dimensional form

**Picture:** Begin with a three-dimensional box. Each pair of opposite faces is labeled by the coordinate omitted from its area element. Extend the notation to n slots, highlighting the missing slot; show a table for i equals one, two and three before the general row.

**Narration:** Orient an n-dimensional box using the coordinate order one through n. An n-minus-one-form has one term for each missing coordinate. We can write each coefficient beside the wedge of every coordinate differential except the missing one. Choose an alternating sign for each term. That sign is bookkeeping for the orientation of the corresponding faces.

**Picture:** Make the middle three-dimensional term concrete: show `−f₂ dx∧dz` and `+f₂ dz∧dx` on the same face. Swap the two ordered directions and visibly flip the sign; show that these are two descriptions of the same oriented form. Return to the coordinate-ordered convention before the n-dimensional table.

**Narration:** For example, in three dimensions, minus f two d x wedge d z is the same form as plus f two d z wedge d x. Swap the order and the sign flips. The minus sign is not a new physical effect; it keeps the orientation consistent when we put the directions in our chosen order.

**Picture:** Build the formula `ω=Σᵢ(−1)ⁱ⁻¹ fᵢ dx¹∧···∧omit(dxⁱ)∧···∧dxⁿ`. Then display the derivative of one coefficient: `dfᵢ=Σⱼ ∂ⱼfᵢ dxʲ`.

**Narration:** Now take the exterior derivative. For each coefficient, its differential includes a contribution in every coordinate direction. What happens when we wedge one of those differentials with the differentials already in the term?

## 10 — Why only one derivative survives

**Picture:** Hold the question. Show a repeated differential in two matching outlines; then remove it. Move the omitted `dxⁱ` into coordinate order, count its swaps and pair the two signs.

**Narration:** A wedge product vanishes if the same differential appears twice. Every direction already present disappears. Only the derivative in the missing direction survives. Moving that differential into coordinate order produces exactly the sign that cancels the sign we placed in front of the term.

**Picture:** Sum the surviving terms: `dω=(Σᵢ ∂ᵢfᵢ) dx¹∧···∧dxⁿ`. Show three-dimensional divergence beside it as a familiar special case.

**Narration:** Add the terms and the exterior derivative is the sum of the partial derivatives, multiplied by the oriented n-dimensional volume form. The resemblance to divergence is deliberate. Now we can prove the boundary identity by applying the one-dimensional theorem, one coordinate at a time.

## 11 — The n-dimensional box proof

**Picture:** Select one term `∂ᵢfᵢ` inside an n-dimensional box. Freeze all other coordinates; highlight a line parallel to the i-th axis, with its lower and upper endpoints. Then sweep those lines across the other coordinates.

**Narration:** Fix all coordinates except the i-th. Integrating the derivative in that direction gives the coefficient on the upper face minus the coefficient on the lower face. Now integrate those differences over the remaining coordinates. Fubini's theorem lets us perform the integrations in this order for our smooth functions on a bounded box.

**Picture:** Show the two opposing faces. Add outward normals and the induced orientations. The upper face gets a plus contribution and the lower face a minus; accumulate every pair of faces into `∫∂box ω`.

**Narration:** The outward-first convention for boundary orientation gives exactly those two signs. Do this for every coordinate and add the results. Every term of the interior integral becomes the matching pair of boundary faces. We have proved generalized Stokes for an oriented box in any number of dimensions. We used the ordinary fundamental theorem, the wedge signs and Fubini; no picture of a four-dimensional object was needed.

## 12 — Curved manifolds: the actual bridge

**Picture:** A curved oriented surface is covered by overlapping colored charts. A boundary patch unfolds to a half-plane; an interior patch unfolds to a full plane. Fade in smooth weight functions whose sum is one, without implying that charts are disjoint.

**Narration:** What about a curved manifold? The box proof is the local engine, but there are real steps left. We use orientation-preserving coordinate charts. Interior charts look locally like ordinary space; boundary charts look locally like a half-space. We also use a partition of unity: smooth weights, supported inside the charts, that add to one.

**Picture:** On a half-space chart, isolate the normal-coordinate derivative. Integrate from zero outwards; compact support makes the distant term vanish, leaving minus the value on the boundary plane. Draw the outward normal toward negative coordinate direction. Tangential-coordinate terms vanish by compact support.

**Narration:** Multiply the form by a chart's weight and work in that chart. On an interior chart, compact support makes the opposite contributions vanish. On a boundary chart, the normal-coordinate calculation leaves the boundary-plane term with the outward-first sign. This is again one-variable calculus inside a higher-dimensional integral.

## 13 — Why the patches add back together

**Picture:** Show `ω = Σ ρⱼω`; then `dω = Σ d(ρⱼω)`. Expand one term, color `dρⱼ∧ω`, and combine all such terms into `d(Σρⱼ)∧ω=0`. On the boundary, restrict the same weights and sum.

**Narration:** Apply the local identity to each weighted piece and add. The weights sum to one, so the pieces reassemble the original form on the boundary. Inside, differentiating the weights produces extra terms, but together they contain the derivative of the sum of the weights. That sum is the constant one, whose derivative is zero. The extra terms cancel.

**Picture:** Place three named tools beside the finished equation: chart pullback commutes with the exterior derivative; orientation-aware change of variables; partition of unity. Link to a companion proof, without marking the lemmas as proven on screen.

**Narration:** To make that argument rigorous, we use the facts that pullback commutes with the exterior derivative, that integration respects oriented changes of coordinates, and that the required partitions of unity exist. Those are mathematical ingredients, not something the tile picture alone proves. With them, the local calculation gives Stokes on the manifold.

## 14 — From the theorem to Faraday's law

**Picture:** A fixed wire loop and a fixed spanning surface. A time-varying magnetic field threads the surface; animated arrows show an induced electric field around the loop. Show a stationary-loop label.

**Narration:** Now for a physical payoff. Hold a wire loop fixed, and let the magnetic field through it change. Faraday's law says that the circulation of the electric field around the loop equals minus the rate of change of magnetic flux through the surface. The minus sign is the direction dictated by the law, not a new sign invented by Stokes.

**Picture:** Replace the loop integral by a surface integral of curl E. For a fixed surface, move the time derivative inside the magnetic-flux integral. Show the local equality only after the surface is allowed to vary among suitable fixed surfaces.

**Narration:** Stokes replaces the loop circulation with the integral of curl E over the surface. For a fixed surface, the time derivative passes inside the flux integral under the usual smoothness conditions. If the resulting equality holds for every suitable small surface, the local fields satisfy curl E equals minus the time derivative of B. That is the differential Maxwell–Faraday equation. A moving loop needs extra care; this demonstration keeps the loop and surface fixed.

## 15 — What survives

**Picture:** Reverse the opening sequence: glowing boundary → canceled seams → line endpoints. Finish with the general equation and the applicability conditions; show the annulus inner edge as a small reminder.

**Narration:** The common pattern is not that the boundary tells us every detail of the interior. Different interior changes can cancel. The theorem says that the particular accumulation described by an exterior derivative is exactly balanced by the original quantity on the oriented boundary, when its hypotheses hold.

**Picture:** Take the boundary of an oriented square, then the boundary of its four oriented edges. At each corner, incoming and outgoing endpoint signs cancel. Place `∂∂ = 0` beside `d(dω) = 0`; keep them distinct from the Stokes equation.

**Narration:** Take the boundary of a boundary and the signed corners cancel. The matching operation on forms also squares to zero: take an exterior derivative twice and you get zero. Those facts fit the same structure, but they do not prove Stokes on their own. The theorem is the bridge between integration, the boundary and the exterior derivative.

**Picture:** Display `∫_M dω = ∫_{∂M} ω` with M oriented and omega smooth, then fade to a simple line segment with plus and minus endpoints.

**Narration:** On a line, that is the fundamental theorem. On a surface, it connects curl to circulation. In a volume, it connects divergence to flux. In any dimension, the proof reduces locally to the same one-dimensional endpoint subtraction, and the charts tell us how to assemble those local facts. Every interior seam has a partner. The boundary is what remains.

---

## Working claim and production notes

- Sources: [MIT theorem and intrinsic dimensional cases](https://math.mit.edu/~dyatlov/18.155/155-notes.pdf#page=237); [Eliashberg, Stanford, §10.2](https://math.stanford.edu/~eliash/Public/177-2015/52htext-2015.pdf#page=171); [OpenStax, fixed-surface Faraday derivation](https://openstax.org/books/calculus-volume-3/pages/6-7-stokes-theorem); and the local [visual brief](stokes-theorem-visual-brief-20261102.md).
- The half-space computation, orientation convention, one-form assumptions and physics statement require independent mathematical review. The chart, partition-of-unity and change-of-variables lemmas are invoked, not proven here.
- Explicit user-facing text is a proposed script. Do not call it a verified take, validated timing or a finished video. No competitors have been fully watched for this script.
- The mathematical numbers and orientations are based on the author checks in `stokes-examples-check-20261102.py`; those checks do not replace an independent oracle or review of the later scenes.
- ElevenLabs generation, native source creation, full renders and publication remain subject to the applicable user-provided planning-workspace restrictions and any subsequent authorization. Preserve all full takes. Continuous listening, selection, fine synchronization, accessibility and independent mathematics, pedagogy and audiovisual review remain open.
