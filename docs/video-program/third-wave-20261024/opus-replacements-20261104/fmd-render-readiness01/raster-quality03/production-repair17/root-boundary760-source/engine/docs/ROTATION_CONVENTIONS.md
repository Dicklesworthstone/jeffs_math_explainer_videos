# Rotation conventions — normative

**Status:** Normative. Owner: W2 (Chisel). Bead: fm-ngx. Plan: §7.5, §2.2, §6.1.
**Implemented by:** `crates/fmn-geom/src/rotation.rs`, `crates/fmn-geom/src/space_ops.rs`.
**Locked by:** `crates/fmn-geom/tests/space_ops_parity.rs` (457 recorded
Reference/scipy cases) and `crates/fmn-geom/tests/rotation_properties.rs`
(laws + singularities).

A rotation has many equally valid descriptions, and every one of them is a
*choice*: which end of the quaternion holds the real part, which sign the
double cover gets, which order a composition applies, where an Euler
decomposition cuts its branch, what happens at a pole. The Reference makes
all of those choices by delegating to `scipy.spatial.transform.Rotation`,
and user camera code — `frame.reorient(theta, phi, gamma)`,
`get_euler_angles()`, orientation interpolation — is written against them.

So these are **semantics, not implementation detail** (D-05). This page
states them; the code implements exactly this and nothing cuter.

## 1. Quaternion storage

A quaternion is `[x, y, z, w]` — **scalar last**, as scipy stores it and as
the Reference states outright (`space_ops.quaternion_mult`: "the real part
is the last entry, so as to follow the scipy Rotation conventions"). The
camera's `orientation` uniform is four floats in this order, so the layout
is API surface, not a private encoding.

Type: `fmn_geom::rotation::Quat = [f64; 4]`.

## 2. Axis–angle

A rotation of `θ` about a unit axis `u` is

```text
q = [ u·sin(θ/2), cos(θ/2) ]
```

built by `quat_from_rotvec(θ·u)`. Below `θ = 1e-3` rad the scale factor is
evaluated by its series (`1/2 − θ²/48 + θ⁴/3840`) rather than
`sin(θ/2)/θ`, matching scipy and keeping full relative precision at
near-identity rotations — the regime every incremental camera drag lives
in.

Going back (`rotvec_from_quat`) **normalizes the double cover first**: a
quaternion with a negative real part is negated, so the returned rotation
vector always has magnitude in `[0, π]`. A 3π/2 turn about `+z` therefore
comes back as a π/2 turn about `−z`. That is the same rotation, reported
the short way round.

`angle_axis_from_quaternion` returns `None` for the identity: there is no
axis, and the Reference's `rot_vec / norm` would produce NaNs.

## 3. Composition order

```text
compose_quat(p, q)  applies q first, then p
```

equivalently `M(compose_quat(p, q)) == M(p) · M(q)`. The Reference's
`quaternion_mult(*quats)` folds left with this product, and
`space_ops::quaternion_mult(&[])` is the identity quaternion.

## 4. Matrices

`matrix_from_unit_quat` returns scipy's `as_matrix()`: row-major, and the
matrix that maps a **column** vector (`v ↦ M v`). Rotation matrices are
orthonormal with determinant `+1`.

The Reference's two names are inverted relative to that, and we keep the
inversion because scene code calls both:

| Name | Returns |
|---|---|
| `rotation_matrix_transpose_from_quaternion(q)` | scipy's `as_matrix()` — the column-vector map |
| `rotation_matrix_from_quaternion(q)` | its transpose |

`rotation_matrix(angle, axis)` is built through the quaternion, exactly as
the Reference does (`Rotation.from_rotvec(angle * normalize(axis)).as_matrix()`),
so the small-angle series and the element ordering agree bit for bit. A
zero axis yields the identity, because `normalize` of a zero vector is the
zero vector (§6 below).

`rotate_vector(v, angle, axis)` is the column-vector map applied to `v`.
The cardinal cases, which every scene relies on by eye: a quarter turn
about `OUT` sends `RIGHT → UP → LEFT → DOWN → RIGHT`; a quarter turn about
`RIGHT` sends `UP → OUT`.

## 5. Euler angles

### 5.1 Sequences

A sequence is three characters. **Lowercase is extrinsic** (rotations about
the fixed frame's axes), **uppercase is intrinsic** (about the rotating
body's axes). Mixed case is refused, as are repeated adjacent axes and any
character outside `xyz`/`XYZ`. `EulerSeq::parse` returns `None` for all of
those.

For the extrinsic sequence `"zxz"` with angles `[a₁, a₂, a₃]`:

```text
R = Rz(a₃) · Rx(a₂) · Rz(a₁)
```

— the first-listed angle is applied *first*, so it appears rightmost. The
Reference's `CameraFrame` defaults to `"zxz"` and reverses this list into
`(theta, phi, gamma)`, i.e. `theta = a₃`, `gamma = a₁`
(`camera_frame.py:74, :145`). It also supports `"zxy"`.

A sequence whose first and third axes are equal (`zxz`) is **symmetric**
(proper Euler); one using all three (`zxy`) is **asymmetric** (Tait–Bryan).
The two families differ in their ranges and in where their poles are.

### 5.2 Ranges

Every returned angle lies in `[-π, π]`. The second angle is further
constrained by family:

| Family | Second angle |
|---|---|
| Symmetric (`zxz`, `xyx`, …) | `[0, π]` — a polar angle |
| Asymmetric (`zxy`, `xyz`, …) | `[-π/2, π/2]` — a pitch |

### 5.3 Gimbal lock

When the second angle reaches its pole (`0` or `π` for symmetric; `±π/2`
for asymmetric), the first and third rotations act about the same axis and
only their sum (or difference) is recoverable. The window is `1e-7` rad,
scipy's.

In that case:

- the **third** angle is set to `0`,
- the **first** angle carries the whole recoverable combination —
  `2·half_sum` at the `0` pole, `∓2·half_diff` at the `π` pole,
- and `EulerAngles::gimbal_lock` is `true`.

Note the asymmetry with the non-degenerate path: the zero goes into
`angles[2]` and the value into `angles[0]` **by index**, for intrinsic
sequences too — scipy's behavior, and the reason the Reference's camera
code has its own fix-up on top (`camera_frame.py:76-82`).

Where scipy raises a `UserWarning`, we return `gimbal_lock` as a field.
A warning is not a value, and the caller has to branch on this.

### 5.4 Round-trip

`euler_from_quat` followed by `quat_from_euler` reproduces the original
rotation (possibly as the other double-cover representative). It does *not*
in general reproduce the original *angles* — at a pole it cannot, and away
from one it returns the canonical representative of the same rotation.

## 6. Degeneracies: defined outputs, never NaN

A NaN escaping this layer propagates into a bounding box, then a layout,
then a frame, and is diagnosed nowhere near where it was born. So every
degenerate input has a stated answer:

| Input | Result |
|---|---|
| `normalize(0)` | the zero vector (the Reference's `np.zeros`) |
| `normalize_or(0, fb)` | `fb`, verbatim — *not* normalized |
| `angle_between_vectors` with a zero operand | `0` |
| `angle_of_vector(0)` | `0` (`atan2(0, 0)`) |
| `get_unit_normal` of aligned vectors | a normal in the plane they share with `z`; `DOWN` if that degenerates too |
| `rotation_matrix(θ, 0)` | the identity |
| `rotation_between_vectors` of aligned vectors | falls back to `v₁ × RIGHT`, then `v₁ × UP` |
| `find_intersection` under its threshold | `p0`, verbatim |
| `get_closest_point_on_line(a, a, p)` | `a` |
| `center_of_mass(&[])` | the origin |
| `angle_axis_from_quaternion(identity)` | `None` |
| any conversion of a zero-norm quaternion | `None` |

The Reference raises on `line_intersection` of parallel lines; that is
`None` here. Everything else above matches the Reference's own behavior.

## 7. Numerics

Object-space rotation math is `f64` (§6.1). Every transcendental routes
through `fmn-dmath` via `crate::scalar`, so certified renders get
bit-identical Euler and axis-angle conversions across the platform matrix
— camera animations are transcendental-dense, and `std`'s trig differs
between glibc, macOS, and WASM. `sqrt` is called directly: IEEE 754
requires correct rounding, so it is already reproducible.

## 8. What is not here

`earclip_triangulation` lives in the Reference's `space_ops` module but is
triangulation, not space ops; it lands with the ear-clipper (fm-81u).
`Rotation.from_matrix` has no consumer at the pin and is not implemented;
it should land with whichever bead first needs it, under this page's
conventions.
