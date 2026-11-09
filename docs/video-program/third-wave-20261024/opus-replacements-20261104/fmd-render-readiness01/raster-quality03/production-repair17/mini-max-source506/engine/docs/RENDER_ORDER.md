# RENDER_ORDER.md — the draw-order model (§8.5)

**Status:** normative (W3, fm-jsc). Lumen's W5 beads cite this document;
`crates/fmn-mobject/src/order.rs` implements it and
`crates/fmn-mobject/tests/render_order.rs` is the ordering-trace corpus that
pins it.

Draw order is **meaning, not pixels**. A scene whose overlay lands under its
diagram is wrong however beautifully it rasterizes, so the Reference's actual
ordering semantics are engine-side behavior here, not a renderer detail to be
rediscovered per bug. Every ruling below carries its citation into the pinned
Reference — `3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13` — read
for its *semantics*, never for its pixels (D-16). This archaeology is done
once, in one place.

---

## 1. The model: two levels, one sequence

**Level 1 — the scene list.** `Stage::roots()` is the draw list, back to
front. Adjacent members sharing `(program, batch key, z_index)` form one
**render group** (`Scene.assemble_render_groups`, `scene.py:300`, over
`batch_by_property`, `utils/iterables.py:48`).

**Level 2 — the families.** Each group draws its members' families in
depth-first order with pointless members skipped
(`Mobject.get_shader_wrapper_list`, `mobject.py:2056`, over
`family_members_with_points`, `mobject.py:435`), and adjacent members sharing
a batch key draw as one call.

**R-1. A batch never crosses a render-group boundary.** This is the one
consequence of the two-level shape that a flat "batch the flattened list"
model gets wrong: two adjacent top-level members with different `z_index`
are two groups even when their first family members share a key, so the two
sides cannot merge into one call. Pinned by
`render_order.rs::a_batch_never_crosses_a_group_boundary`.

**R-13. A bare `Mobject`'s points never draw, and it ends the batch.** The
Reference's base `Mobject` (so `Group`, `Point`, a bare `PMobject`) has an
empty `shader_folder` (`mobject.py:69`); its wrapper compiles no program
(`shader_wrapper.py:83-88`) but keeps its own shader id in the batching. Its
points, set by `Group.set_points(...)` for example, are data. Here such a
record has the Reference's base dtype (`point`, `rgba`) on the vector
primitive: `Entry::paints` is false, it yields no draw item, and it closes
the open batch. Before this ruling the vector plan validated those points
as a quad path and refused the whole frame on an even count (corpus
`DirectMGFInterpretation`, fm-5wq.37). Pinned by
`render_order.rs::a_bare_mobjects_points_never_draw_and_end_the_batch`.

**R-2. The plan is a pure function of scene state.** `Stage::draw_plan()`
reads the draw list and the entries and nothing else — no map iteration
order, no allocation addresses, no handle numbers leaking into ordering. The
retained render plan (§10.8, fm-gw7) caches this; keeping the model itself
pure is what lets the cache be checked against it.

---

## 2. The batch key

The Reference hashes a shader id over: the program code, the mobject
uniforms, `depth_test`, the render primitive, and the texture paths
(`ShaderWrapper.refresh_id`, `shader_wrapper.py:117`), and `VShaderWrapper`
folds `stroke_behind` in on top (`shader_wrapper.py:354`).

`BatchKey` is that material, typed: a `ProgramKind` plus the per-object
`Uniforms` inventory (which already carries `is_fixed_in_frame`, `shading`,
`clip_planes`, `anti_alias_width`, `joint_type`, `flat_stroke`,
`scale_stroke_with_zoom`, `stroke_behind`, `depth_test`).

**R-3. Float uniforms compare bitwise.** Two objects whose
`anti_alias_width` differs by one ulp are two batches — exactly as two
shader-id hashes differ. Batching is an optimization that must never change
what is drawn, so its key is exact, never approximate.

**R-4. `depth_test` and `is_fixed_in_frame` partition; they do not
reorder.** Both are in the key, so depth-tested content never shares a call
with flat content and a frame-fixed overlay never shares one with world
content. The *sequence* is still painter order over the scene list: the depth
test is a per-fragment operation inside that sequence (§10.4), and
`is_fixed_in_frame` is a camera-space choice, not a sort key.

**R-5. Within one object, fill draws before stroke — unless `stroke_behind`,
which swaps them** (`VShaderWrapper.render`, `shader_wrapper.py:277`).
Reported per item as `PassOrder`, and the reason `stroke_behind` belongs in
the key rather than being resolved downstream.

---

## 3. The scene-list operations

| Operation | Semantics | Citation |
|---|---|---|
| `add_many_to_scene` | remove the union of all new families from the old list, append the batch, then **stable-sort by `(z_index, position)`** | `scene.py:327`, `:338` |
| `bring_to_front` | exactly `add` | `scene.py:389` |
| `bring_to_back` | remove, then prepend — **no sort** | `scene.py:394` |
| `remove_from_scene` | recursive ungrouping (below) | `scene.py:371`, `utils/family_ops.py:23` |
| `replace_in_scene` | splice the replacements into the member's position, only if it is on stage — **no sort** | `scene.py:360` |
| `set_z_index` | write the value over the family | `mobject.py:1238` |

**R-6. Equal `z_index` keeps insertion order.** The sort key is
`(z_index, current position)`, so it is stable by construction: adding three
mobjects at the same z_index draws them in the order they were added.

**R-7. Only `add` sorts.** `bring_to_back` deliberately puts a member behind
everything *regardless* of its z_index — re-sorting there would make the call
a silent no-op for any member with a high z_index — and `remove` and
`replace` edit positions without renormalizing. The list is therefore
z-ordered **as of the last add**, and the next add re-normalizes it. This is
a sharp edge, and it is semantics: a scene that mixes `bring_to_back` with
z_index gets the Reference's answer, which is the one its author saw.

**R-8. Removing a family member ungroups its ancestor.** `remove(m)` where
`m` sits inside a rooted group replaces the group in the draw list with the
group's **other** children, spliced in place, recursively
(`recursive_mobject_remove`). Removing one element of a rooted group leaves
the rest on stage, ungrouped, in order — which is what makes a remover
animation on a submobject behave. Nothing is deleted: handles and entries
survive (the rooted-lifetime rule, D-11).

**R-9. A child's `z_index` orders nothing.** The scene sort reads the draw
list, and a child is not in it. `set_z_index` still recurses over the family
(as the Reference does) because the value is per-object data and a child may
be promoted to a root later. Stated here so it is never rediscovered as a
bug.

**R-10. Setting `z_index` does not re-sort a scene that already holds the
mobject.** `add_to_scene`/`bring_to_front` is what renormalizes — the
Reference's ordering happens in `add`, not in the setter.

---

## 4. Shared mobjects

**R-11. A mobject reachable twice inside one family is drawn once.**
`Stage::family` deduplicates (fm-jru); the Reference's `get_family`
concatenates sub-families with no dedup (`mobject.py:426`), so a diamond
draws twice there — which double-composites transparent content and is
visible. Deduplicating is a **deliberate divergence (D5)**: strictly better,
and the correct base for §10.8's occlusion pruning, which reasons about
each drawn object once.

**R-12. A mobject under two separate roots added in the same `Scene.add`
call is drawn once per root.** That is two placements in the scene, not one
object drawn twice by accident; each carries its own `root` in the plan so
the renderer (and inspector) can say which placement it is looking at. The
batch boundary matters: adding the second parent later first removes its
complete family from the old scene list, so the child moves from the earlier
placement into the new one (`scene.py:333`).

---

## 5. What the plan reports

`DrawPlan` is the sequence, back to front, one `DrawItem` per drawn object:
the object, the `root` whose placement put it there, its `group`, its
`batch`, its `key`, and its `passes` order. `sequence()` and `batch_trace()`
are the two halves the fixtures compare — *what* is drawn in *what order*,
and *how many calls* it takes. Geometry is deliberately not part of the
trace: this document is about order, and a trace that also hashed points
would fail for reasons that have nothing to do with order.
