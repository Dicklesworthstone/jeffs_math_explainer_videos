# Authored annotations and derived lines

`Cross` and `Underline` construct through `VGroup` and `Line`. Their normal
`init_data`, `init_points`, `init_uniforms`, and `init_colors` hooks execute,
and public `replace`, `set_width`, and `next_to` overrides act on the geometry
that is displayed. Stroke profiles are bounded to 4096 samples. Cross uses the
Reference's 20 additional subdivisions; Underline inserts enough curves for
its authored width profile. Native records and interpolation remain authoritative.

The surrounding-rectangle lifecycle retains the portal's empty-target rule:
an empty target suppresses the root outline. Its authored root records are
saved for later regrowth, including custom fields. Copying or pickling an empty
matcher preserves that state. This is not a rollback of arbitrary Python hook
side effects; hook-created decorations remain separately authored children.

`DashedLine` now constructs the actual source contour through `Line`, then calls
`calculate_num_dashes` and the existing `DashedVMobject` slicer. The default count
uses native arclength of that authored, buffered contour; it does not reconstruct
an unrelated arc from stale endpoints. After dashing, the existing endpoint-based
count query remains available. An override can choose a different count, including
zero, within the native 4096-dash limit. Slicing uses native length-to-curve
intervals and the public `get_subcurve` hook. Returned families are checked before
the source points are cleared; failures do not publish a half-built pattern.
The ordinary copier still determines how each slice copies source descendants.

```python
class ThreeDashes(DashedLine):
    def calculate_num_dashes(self, dash_length, positive_space_ratio):
        return 3

    def get_subcurve(self, a, b):
        return super().get_subcurve(a, b).shift(UP)
```

`TangentLine` samples `vmob.pfp` exactly twice at the clipped `alpha - d_alpha`
and `alpha + d_alpha` positions, then constructs through `Line` and scales the
result through its public methods. Default `pfp` is native true-arclength sampling;
authored overrides are not bypassed. Nonfinite parameters or returned points
are refused. Empty paths stay empty and coincident samples stay finite, with
no division by zero. Copies do not rerun sampling or initialization.

`shape_matcher_lifecycle.py` and `derived_line_lifecycle.py` exercise actual
native records, copies, callback failures, and PNG frames against independently
constructed geometry at one and four threads. Both are registered in the portal
runtime script. Targeted acceptance is not full-workspace or certification proof.
