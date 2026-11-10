# BN-18 — ingest_submobjects consumes children instead of double-drawing

**Status:** Draft. Landed in W10 (fm-5wq.4); becomes Final when the
Python-portal gate passes.

## What changed

Classic manim's `PMobject.ingest_submobjects` vstacks the family's point
records onto the parent and *keeps* the submobjects. Every ingested point is
then drawn twice: once from the parent buffer and once from the still-attached
children.

FrankenManim still concatenates the family in preorder, self first. After that
write, the children are removed. The parent's live records are the single
authoritative copy of the ingested cloud.

## Migration guidance

- After `ingest_submobjects()`, `submobjects` is empty and `get_num_points()`
  is the former family total. Do not iterate children expecting the stacked
  points to still live there.
- Code that relied on the double-draw (brighter overlapping sprites, or
  independent child transforms after ingest) should keep the children and skip
  ingest, or copy the children before ingesting.
- `PGroup` membership is unchanged: construction still refuses non-PMobject
  members and grafts the original proxies in argument order.

## Evidence

- `crates/fmn-library/src/pointcloud.rs`: `PMobject::ingest_submobjects`
  consumes children after concatenating points and RGBAs.
- `crates/fmn-python/python/manimlib_bootstrap.py`: portal `ingest_submobjects`
  concatenates live family records then `clear()`s the children.
- `crates/fmn-python/tests/bridge.py`: stacked point count and empty membership
  after ingest.
