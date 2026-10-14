#!/usr/bin/env python3
"""Facts oracle for Video G, "Why Robot Arms Think in Lie Groups".

Run from this folder (`./x lie_robotics facts-check` does):

    uv run --no-project --with numpy python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `lie facts` prints
(src/facts.rs), in the rows of docs/fact-sheet.md: G-A (the arm), G-B (the
book), G-E (three angles), G-H (halfway), G-R (spins), G-P (the base), G-S
(screws), G-K (the arm's poses), G-J (velocities), G-I (steps and paths), G-C
(the coda), G-X (the edges of the always sentences), G-Q (the quizzes) and
G-L (cited).

The oracle may print more than the video: G-D rows are numbers docs/description.md
states, each computed here from first principles by the functions below (not as
a formula over another row's rounded value) and cross-checked. The video prints
G-D.seam.*, G-D.halfway.from_each and G-D.leftover.turn_deg, because its scenes
show them; every other G-D row is a research row only the oracle prints
(tools/description_numbers.py reads the oracle's values as facts).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (src/model.rs,
which works in closed forms: Rodrigues' formula, the screw exponential and
logarithm of Lynch and Park, the product of exponentials, the analytic space
Jacobian):
  * turns about x, y and z are typed-out matrices, never an exponential;
  * the matrix exponential is a generic scaled Taylor series and the matrix
    logarithm is generic inverse scaling and squaring (Denman-Beavers square
    roots, then the series of log(I + X)); neither knows about rotations;
  * the arm's pose is the URDF chain itself, joint by joint (origin, roll
    pitch yaw, then the joint's turn about its own z), never a product of
    exponentials;
  * Jacobians are central differences of that chain; ranks are NumPy's
    singular values;
  * levers, the elbow's circle and the elbow's speed near full stretch come
    from triangle geometry (point to line distance, Heron, the law of
    cosines), never from a Jacobian;
  * the base's arc is the generic exponential of its 3 by 3 velocity matrix,
    cross-checked by stepping the motion in small time steps;
  * CITED literals from the sources listed in docs/fact-sheet.md.

When SciPy is importable (`--with scipy`), every exponential and logarithm
used here is also compared with scipy.linalg and every angle with
scipy.spatial.transform.Rotation. When the environment variable
CMAES_EXPLAINER_DIR names a checkout of the owner's cmaes_explainer project,
the arm's numbers are re-read from vendor/iiwa14.xacro and the humanoid's
joint count from vendor/g1_29dof.urdf (those files are never copied here).
"""
import math
import os
import pathlib
import re
import sys

import numpy as np

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

DEG = math.pi / 180.0
PI = math.pi
# The small turns of the last quiz, degrees (src/facts.rs `shown::SMALL_TURNS`).
SMALL_TURNS_DEG = (1, 10, 90)


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def fx(x, places=4):
    """Fixed decimals, never a negative zero."""
    s = f"{float(x):.{places}f}"
    return s[1:] if s.startswith("-") and float(s) == 0.0 else s


def vec(v, places=4):
    return ",".join(fx(x, places) for x in v)


def ints(v):
    out = []
    for x in v:
        r = round(float(x))
        assert abs(float(x) - r) < 1e-9, f"not whole: {x}"
        out.append(str(int(r)))
    return ",".join(out)


def grid(m):
    return ";".join(ints(row) for row in np.asarray(m))


# ---------------------------------------------------------------- matrices

def rotx(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[1.0, 0, 0], [0, c, -s], [0, s, c]])


def roty(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, 0, s], [0, 1.0, 0], [-s, 0, c]])


def rotz(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1.0]])


def expm(a):
    """Scaling and squaring around the Taylor series; knows nothing of turns."""
    a = np.asarray(a, float)
    n = len(a)
    norm = np.linalg.norm(a, 1)
    k = max(0, int(math.ceil(math.log2(norm))) + 4) if norm > 0 else 0
    b = a / 2.0**k
    out, term = np.eye(n), np.eye(n)
    for j in range(1, 30):
        term = term @ b / j
        out = out + term
    for _ in range(k):
        out = out @ out
    return out


def sqrtm(a):
    """Denman-Beavers."""
    y, z = np.array(a, float), np.eye(len(a))
    for _ in range(80):
        y2 = 0.5 * (y + np.linalg.inv(z))
        z2 = 0.5 * (z + np.linalg.inv(y))
        done = np.abs(y2 - y).max() < 1e-15
        y, z = y2, z2
        if done:
            break
    return y


def logm(a):
    """Inverse scaling and squaring, then the series of log(I + X)."""
    a = np.array(a, float)
    n = len(a)
    k = 0
    while np.linalg.norm(a - np.eye(n), 1) > 1e-2 and k < 48:
        a = sqrtm(a)
        k += 1
    x = a - np.eye(n)
    out, power = np.zeros((n, n)), np.eye(n)
    for j in range(1, 30):
        power = power @ x
        out = out + ((-1.0) ** (j + 1)) * power / j
    return out * 2.0**k


def hat(w):
    return np.array([[0, -w[2], w[1]], [w[2], 0, -w[0]], [-w[1], w[0], 0.0]])


def vee(m):
    return np.array([m[2, 1], m[0, 2], m[1, 0]])


def pose(r, p):
    t = np.eye(4)
    t[:3, :3] = r
    t[:3, 3] = p
    return t


def twist_of(m):
    """The six numbers of a 4 by 4 velocity matrix: spin first, then slide."""
    return np.concatenate([vee(m[:3, :3]), m[:3, 3]])


def angle_axis(r):
    """Degrees and the unit axis of a turn (not for an exact half turn)."""
    w = vee(logm(r))
    a = np.linalg.norm(w)
    return a / DEG, (w / a if a > 1e-12 else np.zeros(3))


def screw_of(t):
    """A pose as one screw: degrees, axis, a point on it, and the slide along it."""
    v6 = twist_of(logm(t))
    a = np.linalg.norm(v6[:3])
    if a < 1e-12:
        d = np.linalg.norm(v6[3:])
        return 0.0, (v6[3:] / d if d > 0 else np.zeros(3)), np.zeros(3), d
    w, v = v6[:3] / a, v6[3:] / a
    return a / DEG, w, np.cross(w, v), float(w @ v) * a


def from_screw(deg, axis, point, slide):
    """The pose a screw makes, built from typed-out pieces (no exponential):
    slide to the axis, turn about it, slide back, then slide along it."""
    a = deg * DEG
    if abs(a) < 1e-15:
        return pose(np.eye(3), axis * slide)
    # a frame whose z is the axis
    z = axis / np.linalg.norm(axis)
    x = np.cross(z, [1.0, 0, 0]) if abs(z[0]) < 0.9 else np.cross(z, [0, 1.0, 0])
    x = x / np.linalg.norm(x)
    f = np.column_stack([x, np.cross(z, x), z])
    r = f @ rotz(a) @ f.T
    return pose(r, point - r @ point + z * slide)


# ---------------------------------------------------------------- the arm
# KUKA LBR iiwa 14 R820. Joint origins (xyz in metres, roll pitch yaw in
# radians) as the owner's cmaes_explainer vendors them: vendor/iiwa14.xacro
# (lines 48, 81, 114, 147, 180, 213, 246 and 279), byte-identical to
# aica-technology/kuka-iiwa-description v0.1.0, robots/iiwa14.xacro (GPL-3.0;
# cited, never copied). Every joint turns about its own +z.
JOINTS = [
    ((0, 0, 0.1575), (0, 0, 0)),
    ((0, 0, 0.2025), (PI / 2, 0, PI)),
    ((0, 0.2045, 0), (PI / 2, 0, PI)),
    ((0, 0, 0.2155), (PI / 2, 0, 0)),
    ((0, 0.1845, 0), (-PI / 2, PI, 0)),
    ((0, 0, 0.2155), (PI / 2, 0, 0)),
    ((0, 0.081, 0), (-PI / 2, PI, 0)),
]
FLANGE = (0, 0, 0.045)
OFFSETS = [0.1575, 0.2025, 0.2045, 0.2155, 0.1845, 0.2155, 0.081, 0.045]
# KUKA, LBR iiwa 14 R820 data sheet (V1.2, 01.06.2022), p. 1: motion range
# and speed with rated payload, axes A1 to A7.
LIMITS_DEG = [170, 120, 170, 120, 170, 120, 175]
SPEED_DEG_S = [85, 85, 100, 75, 130, 135, 135]
N = 7


def rpy(roll, pitch, yaw):
    """URDF's fixed-axis roll, pitch, yaw."""
    return rotz(yaw) @ roty(pitch) @ rotx(roll)


def chain(q):
    """The tip's pose and every joint's frame, by walking the chain."""
    t, frames = np.eye(4), []
    for (xyz, angles), qi in zip(JOINTS, q):
        t = t @ pose(rpy(*angles), xyz) @ pose(rotz(qi), (0, 0, 0))
        frames.append(t.copy())
    return t @ pose(np.eye(3), FLANGE), frames


def fk(q):
    return chain(q)[0]


def space_jacobian(q, h=1e-6):
    """Column i: the twist of the tip per unit speed of joint i, by central differences."""
    t0 = fk(q)
    inv = np.linalg.inv(t0)
    j = np.zeros((6, N))
    for i in range(N):
        d = np.zeros(N)
        d[i] = h
        j[:, i] = twist_of((fk(q + d) - fk(q - d)) / (2 * h) @ inv)
    return j


def adjoint(t):
    r, p = t[:3, :3], t[:3, 3]
    a = np.zeros((6, 6))
    a[:3, :3] = r
    a[3:, 3:] = r
    a[3:, :3] = hat(p) @ r
    return a


def body_jacobian(q):
    return np.linalg.inv(adjoint(fk(q))) @ space_jacobian(q)


def rank(j):
    return int((np.linalg.svd(j, compute_uv=False) > 1e-6).sum())


def degs(*angles):
    return np.array(angles, float) * DEG


def point_line(p, a, direction):
    """Distance from p to the line through a along `direction`."""
    d = direction / np.linalg.norm(direction)
    r = p - a
    return float(np.linalg.norm(r - (r @ d) * d))


# ---------------------------------------------------------------- optional sources

def optional_source_checks():
    root = os.environ.get("CMAES_EXPLAINER_DIR")
    if not root:
        print("# skip the xacro and URDF re-read (set CMAES_EXPLAINER_DIR)", file=sys.stderr)
        return
    xacro = (pathlib.Path(root) / "vendor" / "iiwa14.xacro").read_text()
    blocks = re.findall(r'<joint name="\$\{prefix\}joint(\d)" type="(\w+)">(.*?)</joint>', xacro, re.S)
    def angle(part):
        """A number, or the xacro's `${PI}`, `${PI / 2}`, `${-PI / 2}` and `${- PI / 2}`."""
        m = re.fullmatch(r"\$\{(-?)PI(?:/(\d+))?\}", part.replace(" ", ""))
        if not m:
            return float(part)
        return (-1.0 if m.group(1) else 1.0) * PI / float(m.group(2) or 1)

    seen = []
    for number, kind, body in blocks:
        origin = re.search(r'<origin xyz="([^"]+)" rpy="([^"]+)"', body)
        xyz = tuple(float(x) for x in origin.group(1).split())
        angles = tuple(angle(part) for part in re.findall(r"\$\{[^}]*\}|[-\d.]+", origin.group(2)))
        seen.append((int(number), kind, xyz, angles))
    ok = len(seen) == 8
    for (number, kind, xyz, angles), want in zip(seen, JOINTS + [(FLANGE, (0, 0, 0))]):
        ok = ok and np.allclose(xyz, want[0]) and np.allclose(angles, want[1])
        ok = ok and kind == ("revolute" if number < 8 else "fixed")
    check("the typed joint origins are vendor/iiwa14.xacro's", ok)
    check("every revolute joint of the xacro turns about its own +z", xacro.count('<axis xyz="0 0 1"/>') == 7)
    urdf = re.sub(r"<!--.*?-->", "", (pathlib.Path(root) / "vendor" / "g1_29dof.urdf").read_text(), flags=re.S)
    check("vendor/g1_29dof.urdf has 29 revolute joints", len(re.findall(r'type="revolute"', urdf)) == G1_JOINTS)
    children = set(re.findall(r'<child link="([^"]+)"', urdf))
    roots = [n for n in re.findall(r'<link name="([^"]+)"', urdf) if n not in children]
    check("its root link is the pelvis", roots == ["pelvis"])


def scipy_checks(samples):
    try:
        import scipy.linalg as sl
        from scipy.spatial.transform import Rotation
    except ImportError:
        print("# skip the SciPy comparison (run with --with scipy)", file=sys.stderr)
        return
    worst = 0.0
    for m in samples:
        worst = max(worst, np.abs(expm(m) - sl.expm(m)).max())
        t = sl.expm(m)
        worst = max(worst, np.abs(logm(t) - sl.logm(t).real).max())
    check(f"the series exponential and logarithm agree with scipy.linalg on {len(samples)} matrices", worst < 1e-10)
    r = roty(90 * DEG) @ rotx(90 * DEG)
    a = Rotation.from_matrix(r).magnitude() / DEG
    check("scipy.spatial.transform reads the book's turn as 120 degrees", abs(a - 120) < 1e-9)
    e = Rotation.from_euler("ZYX", [30, 90, 0], degrees=True).as_matrix()
    check("scipy's ZYX Euler angles are the yaw, pitch, roll used here", np.allclose(e, rpy(0, 90 * DEG, 30 * DEG)))


G1_JOINTS = 29
CUBE_AXES = [
    (1, 0, 0), (0, 1, 0), (0, 0, 1),
    (1, 1, 0), (1, -1, 0), (1, 0, 1), (1, 0, -1), (0, 1, 1), (0, 1, -1),
    (1, 1, 1), (1, 1, -1), (1, -1, 1), (-1, 1, 1),
]


def unit(v):
    v = np.array(v, float)
    return v / np.linalg.norm(v)


def turn(axis, deg):
    """A turn about any axis, as the generic exponential of its spin matrix."""
    return expm(hat(unit(axis)) * deg * DEG)


def main():
    samples = []

    # ------------------------------------------------------------ G-A the arm
    emit("G-A.joints", N)
    emit("G-A.offsets", vec(OFFSETS))
    home, frames = chain(np.zeros(N))
    emit("G-A.shoulder_height", fx(frames[1][2, 3]))
    emit("G-A.upper_arm", fx(np.linalg.norm(frames[3][:3, 3] - frames[1][:3, 3])))
    emit("G-A.forearm", fx(np.linalg.norm(frames[5][:3, 3] - frames[3][:3, 3])))
    emit("G-A.wrist_to_tip", fx(np.linalg.norm(home[:3, 3] - frames[5][:3, 3])))
    emit("G-A.height", fx(home[2, 3]))
    emit("G-A.reach", fx(np.linalg.norm(frames[5][:3, 3] - frames[1][:3, 3])))
    emit("G-A.home.p", vec(home[:3, 3]))
    emit("G-A.home.R", grid(home[:3, :3]))
    for i, f in enumerate(frames, 1):
        emit(f"G-A.axis.{i}", ints(f[:3, 2]))
        emit(f"G-A.axis_height.{i}", fx(f[2, 3]))
    emit("G-A.limits_deg", ints(LIMITS_DEG))
    emit("G-A.speed_deg_s", ints(SPEED_DEG_S))
    check("the eight offsets add up to the arm's height", abs(sum(OFFSETS) - home[2, 3]) < 1e-12)
    check("every joint origin at home is on the vertical line through the base",
          all(np.allclose(f[:2, 3], 0) for f in frames))

    # ------------------------------------------------------------ G-B the book
    # x runs left to right, y away from the viewer, z up. "Tilt": a quarter
    # turn about x, the far edge rising. "Roll": a quarter turn about y,
    # clockwise for the viewer. Turns are made about the fixed axes (two
    # lines on the table), so the later turn multiplies on the left.
    tilt, roll = rotx(90 * DEG), roty(90 * DEG)
    tw = roll @ tilt  # tilt, then roll
    wt = tilt @ roll  # roll, then tilt
    emit("G-B.tilt_then_roll.R", grid(tw))
    emit("G-B.roll_then_tilt.R", grid(wt))
    emit("G-B.tilt_then_roll.cover", ints(tw @ [0, 0, 1]))
    emit("G-B.roll_then_tilt.cover", ints(wt @ [0, 0, 1]))
    for name, r in (("tilt_then_roll", tw), ("roll_then_tilt", wt)):
        a, ax = angle_axis(r)
        emit(f"G-B.{name}.angle", fx(a))
        emit(f"G-B.{name}.axis", vec(ax))
    a, ax = angle_axis(tw @ wt.T)
    emit("G-B.between.angle", fx(a))
    emit("G-B.between.axis", vec(ax))
    slide_a, slide_b = np.array([0.1, 0, 0]), np.array([0, 0.1, 0])
    emit("G-B.slides.same", int(np.array_equal(slide_a + slide_b, slide_b + slide_a)))
    emit("G-B.turns.same", int(np.allclose(tw, wt)))
    # the two spins held together for one second: the sum of the two arrows
    both = expm(hat([PI / 2, PI / 2, 0]))
    a, ax = angle_axis(both)
    emit("G-B.both_at_once.angle", fx(a))
    emit("G-B.both_at_once.axis", vec(ax))
    for small in SMALL_TURNS_DEG:
        s = small * DEG
        left = (roty(s) @ rotx(s)) @ (rotx(s) @ roty(s)).T
        a, ax = angle_axis(left)
        emit(f"G-B.leftover.{small}.angle", fx(a))
        emit(f"G-B.leftover.{small}.axis", vec(ax))
        emit(f"G-B.leftover.{small}.product", fx(s * s / DEG))
    check("for one-degree turns the leftover is the product of the angles to 1 part in 1000",
          abs(angle_axis((roty(DEG) @ rotx(DEG)) @ (rotx(DEG) @ roty(DEG)).T)[0] / (DEG * DEG / DEG) - 1) < 1e-3)
    samples += [hat([PI / 2, PI / 2, 0])]

    # ------------------------------------------------------------ G-E three angles
    # yaw about z (the outer ring), pitch about y (the middle ring), roll
    # about x (the inner ring): R = Rz(yaw) Ry(pitch) Rx(roll).
    def ypr(yaw, pitch, roll):
        return rotz(yaw * DEG) @ roty(pitch * DEG) @ rotx(roll * DEG)

    emit("G-E.rings.order", "z,y,x")
    emit("G-E.lock.same", fx(np.abs(ypr(30, 90, 0) - ypr(0, 90, -30)).max()))
    emit("G-E.free.differ", fx(np.abs(ypr(30, 0, 0) - ypr(0, 0, -30)).max()))
    # how much of the third direction is left: the three ring axes, as the
    # columns of a matrix, span a volume of cos(pitch)
    for pitch in (0, 70, 85, 90):
        axes = np.column_stack([
            [0, 0, 1.0],
            rotz(0) @ [0, 1.0, 0],
            rotz(0) @ roty(pitch * DEG) @ [1.0, 0, 0],
        ])
        emit(f"G-E.left.{pitch}", fx(abs(np.linalg.det(axes))))
    check("the ring axes span cos(pitch)", all(
        abs(abs(np.linalg.det(np.column_stack([[0, 0, 1.0], [0, 1.0, 0], roty(p * DEG) @ [1.0, 0, 0]]))) - abs(math.cos(p * DEG))) < 1e-12
        for p in range(0, 91, 5)))
    near_a, near_b = ypr(0, 89, 0), ypr(90, 89, 90)
    emit("G-E.near.apart", fx(angle_axis(near_a.T @ near_b)[0]))
    emit("G-E.near.triples", "0,89,0;90,89,90")
    emit("G-E.near.differ", ints(np.abs(np.array([90, 89, 90]) - np.array([0, 89, 0]))))

    # ------------------------------------------------------------ G-H halfway
    ha, hb = 170.0, -170.0
    emit("G-H.a", ints([ha]))
    emit("G-H.b", ints([hb]))
    emit("G-H.mean", fx((ha + hb) / 2))
    ra, rb = rotz(ha * DEG), rotz(hb * DEG)
    step = logm(ra.T @ rb)  # the turn from a to b, on the flat piece at a
    gap = vee(step)[2] / DEG
    half = ra @ expm(step / 2)
    halfway = math.atan2(half[1, 0], half[0, 0]) / DEG
    emit("G-H.gap", fx(gap))
    emit("G-H.halfway", fx(abs(halfway)))
    naive = rotz((ha + hb) / 2 * DEG)
    emit("G-H.off_by", fx(angle_axis(naive.T @ half @ rotz(1e-9))[0]))  # a half turn, nudged off the cut
    check("the honest halfway is 10 degrees from each heading",
          abs(angle_axis(ra.T @ half)[0] - 10) < 1e-9 and abs(angle_axis(half.T @ rb)[0] - 10) < 1e-9)

    # ------------------------------------------------------------ G-R spins
    diagonal = unit([1, 1, 1])
    rate = 120.0
    spun = expm(hat(diagonal) * rate * DEG)  # held for one second
    emit("G-R.diag.rate_deg", ints([rate]))
    emit("G-R.diag.R", grid(spun))
    emit("G-R.diag.arrow", vec(diagonal * rate * DEG))
    emit("G-R.diag.is_roll_then_tilt", int(np.allclose(spun, wt)))
    emit("G-R.diag.edges", ";".join(ints(spun @ e) for e in np.eye(3)))
    p = np.array([1.0, 0, 0])
    along = (p @ diagonal) * diagonal
    emit("G-R.rod.along", vec(along))
    emit("G-R.rod.around", vec(p - along))
    emit("G-R.rod.result", vec(spun @ p))
    emit("G-R.quarter_z.R", grid(expm(hat([0, 0, 1.0]) * 90 * DEG)))
    emit("G-R.numbers.arrow", 3)
    emit("G-R.numbers.matrix", spun.size)
    check("the exponential of a spin held one second is the typed-out turn",
          np.allclose(expm(hat([0, 0, 1.0]) * 90 * DEG), rotz(90 * DEG)))
    samples += [hat(diagonal) * rate * DEG]

    # ------------------------------------------------------------ G-P the base
    # body velocity: forward v, sideways 0, spin w, held for t seconds
    v, w_deg, t = 0.5, 30.0, 6.0
    w = w_deg * DEG

    def planar(vx, vy, spin, time):
        m = np.array([[0, -spin, vx], [spin, 0, vy], [0, 0, 0.0]])
        return expm(m * time)

    end = planar(v, 0, w, t)
    emit("G-P.v", fx(v, 1))
    emit("G-P.spin_deg", ints([w_deg]))
    emit("G-P.t", ints([t]))
    emit("G-P.end", vec(end[:2, 2]))
    emit("G-P.heading", fx(abs(math.atan2(end[1, 0], end[0, 0]) / DEG)))
    emit("G-P.naive", vec([v * t, 0.0]))
    emit("G-P.radius", fx(v / w))
    emit("G-P.centre", vec([0.0, v / w]))
    emit("G-P.arc_length", fx(v * t))
    emit("G-P.full_circle_t", ints([360 / w_deg]))
    emit("G-P.full.end", vec(planar(v, 0, w, 360 / w_deg)[:2, 2]))
    # the same six seconds at other spins: the quiz's two other choices
    emit("G-P.spin0.end", vec(planar(v, 0, 0.0, t)[:2, 2]))
    emit("G-P.spin60.end", vec(planar(v, 0, 60 * DEG, t)[:2, 2]))
    # the same motion in 60,000 small steps
    x = y = heading = 0.0
    dt = t / 60000
    for _ in range(60000):
        x += v * math.cos(heading + w * dt / 2) * dt
        y += v * math.sin(heading + w * dt / 2) * dt
        heading += w * dt
    check("stepping the base in small steps reaches the exponential's end", abs(x - end[0, 2]) < 1e-6 and abs(y - end[1, 2]) < 1e-6)
    # wheel speeds from the rolling condition, wheel by wheel (Lynch and Park
    # eqs. 13.4 to 13.6): the wheel's own velocity, then its drive direction
    # and its roller's slide angle (minus 45 degrees front left and rear
    # right, plus 45 the other two)
    half_length, half_track, radius = 1.0, 1.0, 1.0  # only the signs are shown

    def wheels(vx, vy, spin):
        out = []
        for (wx, wy), slide in (((half_length, half_track), -45), ((half_length, -half_track), 45),
                                ((-half_length, half_track), 45), ((-half_length, -half_track), -45)):
            ux, uy = vx - spin * wy, vy + spin * wx
            out.append((ux + uy * math.tan(slide * DEG)) / radius)
        return out

    def signs(u):
        return ",".join("+" if s > 0 else "-" for s in u)

    emit("G-P.wheels.forward", signs(wheels(1, 0, 0)))
    emit("G-P.wheels.sideways", signs(wheels(0, 1, 0)))
    emit("G-P.wheels.spin", signs(wheels(0, 0, 1)))
    samples += [np.array([[0, -w, v], [w, 0, 0], [0, 0, 0.0]]) * t / 2]  # a quarter circle: a half turn has no single logarithm

    # ------------------------------------------------------------ G-S screws
    toss = pose(rotz(90 * DEG), (1.0, 1.0, 0.5))
    a, ax, pt, slide = screw_of(toss)
    emit("G-S.toss.turn_deg", fx(a))
    emit("G-S.toss.axis", vec(ax))
    emit("G-S.toss.point", vec(pt))
    emit("G-S.toss.slide", fx(slide))
    emit("G-S.toss.rebuilt", fx(np.abs(from_screw(a, ax, pt, slide) - toss).max()))
    emit("G-S.numbers.pose", 3 + 3)
    emit("G-S.numbers.twist", 3 + 3)
    emit("G-S.numbers.turn_matrix", 3 * 3)
    emit("G-S.numbers.pose_matrix", 4 * 4)
    emit("G-S.numbers.quaternion", 4)
    samples += [logm(toss)]

    # ------------------------------------------------------------ G-K the arm's poses
    ell = degs(0, 90, 0, 90, 0, 0, 0)
    t_ell, _ = chain(ell)
    emit("G-K.ell.q", ints(ell / DEG))
    emit("G-K.ell.p", vec(t_ell[:3, 3]))
    emit("G-K.ell.R", grid(t_ell[:3, :3]))
    emit("G-K.ell.after_elbow", vec(fk(degs(0, 0, 0, 90, 0, 0, 0))[:3, 3]))
    emit("G-K.quarter.p", vec(fk(degs(90, 90, 0, 90, 0, 0, 0))[:3, 3]))
    # which joint origins and the tip move when only the elbow turns
    _, before = chain(np.zeros(N))
    t_after, after = chain(degs(0, 0, 0, 90, 0, 0, 0))
    moved = [str(i) for i, (a0, a1) in enumerate(zip(before, after), 1) if not np.allclose(a0[:3, 3], a1[:3, 3])]
    if not np.allclose(home[:3, 3], t_after[:3, 3]):
        moved.append("tip")
    emit("G-K.elbow.moves", ",".join(moved))

    # ------------------------------------------------------------ G-J velocities
    emit("G-J.shape", "6x7")
    tip_p = t_ell[:3, 3]
    _, f_ell = chain(ell)
    for i, f in enumerate(f_ell, 1):
        emit(f"G-J.ell.lever.{i}", fx(point_line(tip_p, f[:3, 3], f[:3, 2])))
    j_ell = space_jacobian(ell)
    emit("G-J.ell.col2", vec(j_ell[:, 1]))
    # the grid of chapter 10, by central differences of the URDF chain
    emit("G-J.ell.J", ";".join(vec(row, 2) for row in j_ell))
    stretch = degs(0, 90, 0, 0, 0, 0, 0)
    elbow_straight = degs(0, 60, 0, 0, 0, 60, 0)
    swing = degs(0, 90, 0, 90, 0, 60, 0)
    elbow_only = degs(0, 0, 0, 90, 0, 0, 0)
    ell_wrist_turned = degs(0, 90, 0, 90, 90, 0, 0)  # the L shape, joint 5 a quarter turn, wrist straight
    for name, q in (("home", np.zeros(N)), ("stretch", stretch), ("elbow_straight", elbow_straight),
                    ("elbow_only", elbow_only), ("ell", ell), ("ell_wrist_turned", ell_wrist_turned),
                    ("swing", swing)):
        emit(f"G-J.rank.{name}", rank(space_jacobian(q)))
    # the L shape with one joint taken away: a six-joint arm
    for k in (3, 7):
        emit(f"G-J.rank.ell.without_{k}", rank(np.delete(space_jacobian(ell), k - 1, axis=1)))
    emit("G-J.elbow_straight.p", vec(fk(elbow_straight)[:3, 3]))
    emit("G-J.elbow_only.reach", fx(np.linalg.norm(chain(elbow_only)[1][5][:3, 3] - chain(elbow_only)[1][1][:3, 3])))

    def push_along_arm(q, h=1e-6):
        """The most any one joint, turning at one radian a second, moves the
        wrist along the line from shoulder to wrist (m/s): by moving each
        joint a little and watching the wrist."""
        _, f = chain(q)
        along = f[5][:3, 3] - f[1][:3, 3]
        along = along / np.linalg.norm(along)
        most = 0.0
        for i in range(N):
            d = np.zeros(N)
            d[i] = h
            moved = (chain(q + d)[1][5][:3, 3] - chain(q - d)[1][5][:3, 3]) / (2 * h)
            most = max(most, abs(float(moved @ along)))
        return most

    emit("G-J.elbow_straight.push_along_arm", fx(push_along_arm(elbow_straight)))
    emit("G-J.swing.push_along_arm", fx(push_along_arm(swing)))
    js = space_jacobian(elbow_straight)
    # the elbow's speed for the tip to pull straight back at 0.1 m/s, from the
    # triangle shoulder, elbow, wrist alone: d^2 = a^2 + b^2 + 2ab cos(bend)
    upper = float(np.linalg.norm(frames[3][:3, 3] - frames[1][:3, 3]))
    fore = float(np.linalg.norm(frames[5][:3, 3] - frames[3][:3, 3]))
    pull = 0.1
    emit("G-J.blowup.pull", fx(pull, 1))
    for bend in (30, 10, 3, 1):
        d = math.sqrt(upper**2 + fore**2 + 2 * upper * fore * math.cos(bend * DEG))
        emit(f"G-J.blowup.{bend}", fx(pull * d / (upper * fore * math.sin(bend * DEG)) / DEG, 2))
    emit("G-J.blowup.limit", SPEED_DEG_S[3])

    def elbow_speed(bend):
        d_ = math.sqrt(upper**2 + fore**2 + 2 * upper * fore * math.cos(bend * DEG))
        return pull * d_ / (upper * fore * math.sin(bend * DEG)) / DEG

    lo, hi = 10.0, 30.0  # the speed falls as the bend grows
    for _ in range(60):
        mid_ = (lo + hi) / 2
        lo, hi = (mid_, hi) if elbow_speed(mid_) > SPEED_DEG_S[3] else (lo, mid_)
    emit("G-J.blowup.bend_at_limit", fx(lo, 1))
    emit("G-J.blowup.1.turns_per_s", fx(elbow_speed(1) / 360, 2))
    # the elbow's circle for the swing pose: the triangle's height (Heron)
    _, f_swing = chain(swing)
    d = float(np.linalg.norm(f_swing[5][:3, 3] - f_swing[1][:3, 3]))
    s = (upper + fore + d) / 2
    emit("G-J.elbow.radius", fx(2 * math.sqrt(s * (s - upper) * (s - fore) * (s - d)) / d))
    emit("G-J.spare", N - 6)
    # a set of joint speeds that moves nothing: the last right singular vector
    nullv = np.linalg.svd(space_jacobian(swing))[2][-1]
    emit("G-J.swing.null_twist", fx(np.abs(space_jacobian(swing) @ nullv).max()))
    emit("G-J.swing.p", vec(fk(swing)[:3, 3]))

    # ------------------------------------------------------------ G-I steps and paths
    start = degs(0, 30, 0, -60, 0, 90, 0)
    t_start = fk(start)
    goal_p = np.array([0.40, 0.30, 0.25])
    t_goal = pose(rotz(90 * DEG) @ t_start[:3, :3], goal_p)
    emit("G-I.start.q", ints(start / DEG))
    emit("G-I.start.p", vec(t_start[:3, 3]))
    emit("G-I.start.R", grid(t_start[:3, :3]))
    emit("G-I.goal.p", vec(goal_p))
    emit("G-I.goal.R", grid(t_goal[:3, :3]))
    # the error at the start in the gripper's frame (a series logarithm)
    emit("G-I.error.twist", vec(twist_of(logm(np.linalg.inv(t_start) @ t_goal)), 3))
    tol_spin, tol_slide = 1e-3, 1e-4  # Lynch and Park, Example 6.1
    q = start.copy()
    dist, angle = [], []
    for step in range(50):
        now = fk(q)
        error = twist_of(logm(np.linalg.inv(now) @ t_goal))
        dist.append(float(np.linalg.norm(now[:3, 3] - goal_p)) * 1000)
        angle.append(float(np.linalg.norm(error[:3])) / DEG)
        if np.linalg.norm(error[:3]) < tol_spin and np.linalg.norm(error[3:]) < tol_slide:
            break
        q = q + np.linalg.pinv(body_jacobian(q)) @ error
    emit("G-I.ik.steps", len(dist) - 1)
    emit("G-I.ik.dist_mm", vec(dist, 2))
    emit("G-I.ik.angle_deg", vec(angle, 4))
    emit("G-I.ik.q_deg", vec(q / DEG, 2))
    emit("G-I.ik.within_limits", int(all(abs(a) <= lim for a, lim in zip(q / DEG, LIMITS_DEG))))
    emit("G-I.ik.tolerances", "0.001,0.0001")
    check("the joint angles found put the tip on the goal", np.abs(fk(q) - t_goal).max() < 1e-3)
    # the one screw from the start pose to the goal (in the room's frame)
    a, ax, pt, slide = screw_of(t_goal @ np.linalg.inv(t_start))
    emit("G-I.screw.angle", fx(a))
    emit("G-I.screw.axis", vec(ax))
    emit("G-I.screw.point", vec(pt[:2]))
    emit("G-I.screw.slide", fx(slide))
    blend = (t_start[:3, :3] + t_goal[:3, :3]) / 2
    emit("G-I.blend.det", fx(np.linalg.det(blend)))
    # a quarter of the way: the blend, repaired to the nearest turn (the polar
    # part, from a singular value decomposition), against the screw
    quarter = 0.75 * t_start[:3, :3] + 0.25 * t_goal[:3, :3]
    u_, _, vt_ = np.linalg.svd(quarter)
    emit("G-I.blend.quarter_turned", fx(angle_axis(t_start[:3, :3].T @ (u_ @ vt_))[0]))
    whole = logm(t_goal @ np.linalg.inv(t_start))
    bow = 0.0
    for k in range(101):
        f = k / 100
        on_screw = (expm(whole * f) @ t_start)[:3, 3]
        on_line = t_start[:3, 3] + f * (goal_p - t_start[:3, 3])
        bow = max(bow, float(np.linalg.norm(on_screw - on_line)))
    emit("G-I.bow", fx(bow))
    mid = expm(whole * 0.5) @ t_start
    emit("G-I.screw.mid.det", fx(np.linalg.det(mid[:3, :3])))
    emit("G-I.screw.mid.turned", fx(angle_axis(t_start[:3, :3].T @ mid[:3, :3])[0]))
    emit("G-I.screw.quarter_turned", fx(angle_axis(t_start[:3, :3].T @ (expm(whole * 0.25) @ t_start)[:3, :3])[0]))
    samples += [whole]

    # ------------------------------------------------------------ G-C the coda
    emit("G-C.g1.joints", G1_JOINTS)
    emit("G-C.g1.velocity", 6 + G1_JOINTS)
    emit("G-C.g1.stored", 3 + 4 + G1_JOINTS)
    spin = hat(unit([1, 1, 1]))  # one radian per second
    steps, dt = 1000, 0.01
    added, stepped, one = np.eye(3), np.eye(3), expm(spin * dt)
    for _ in range(steps):
        added = added + dt * spin @ added
        stepped = one @ stepped
    emit("G-C.drift.steps", steps)
    emit("G-C.drift.dt", fx(dt, 2))
    emit("G-C.drift.add", fx(np.linalg.det(added)))
    emit("G-C.drift.exp", fx(np.linalg.det(stepped)))
    emit("G-C.drift.width", fx(np.linalg.svd(added, compute_uv=False)[0]))
    check("adding the spin swells the body by (1 + dt^2) per step", abs(np.linalg.det(added) - (1 + dt * dt) ** steps) < 1e-9)

    # ------------------------------------------------------------ G-D numbers docs/description.md states
    # Chapter 04's seam, as the code panel (04_4) and the trap of 13_2 print it: the two headings as numbers.
    emit("G-D.seam.a_minus_b", ints([ha - hb]))
    emit("G-D.seam.b_minus_a", ints([hb - ha]))
    check("the headings differ as numbers by the gap measured on the flat piece less a full turn",
          abs((hb - ha) - (gap - 360.0)) < 1e-9)
    # 04_4: half of the step on the flat piece, wrapped back, is this far round the circle from either heading
    emit("G-D.halfway.from_each", fx(angle_axis(ra.T @ half)[0]))
    # 15_3, 15_4: the sizes of the small turns, and how far the leftover's axis leans from -z (the third axis)
    emit("G-D.leftover.turn_deg", ints(SMALL_TURNS_DEG))
    lean = []
    for small in SMALL_TURNS_DEG:
        s = small * DEG
        _, ax_ = angle_axis((roty(s) @ rotx(s)) @ (rotx(s) @ roty(s)).T)
        lean.append(math.atan2(math.hypot(ax_[0], ax_[1]), -ax_[2]) / DEG)
    emit("G-D.leftover.off_z_deg", vec(lean))
    check("the leftover's axis leans from -z by about (sqrt 2 / 2) times the turn, for the 1 and 10 degree turns",
          all(abs(lean[k] / (SMALL_TURNS_DEG[k] * math.sqrt(0.5)) - 1) < 1e-2 for k in (0, 1)))
    # 06: the base stepped in pieces of 0.01 s, the velocity turned by the heading each piece (forward Euler)
    euler_dt = 0.01
    ex = ey = eh = 0.0
    for _ in range(round(t / euler_dt)):
        ex += v * math.cos(eh) * euler_dt
        ey += v * math.sin(eh) * euler_dt
        eh += w * euler_dt
    euler_miss_mm = math.hypot(ex - end[0, 2], ey - end[1, 2]) * 1000
    emit("G-D.platform.euler.dt", fx(euler_dt, 2))
    emit("G-D.platform.euler.miss_mm", fx(euler_miss_mm, 2))
    # the same sum in closed form: a geometric series of the unit heading vectors
    series = v * euler_dt * (1 - np.exp(1j * w * round(t / euler_dt) * euler_dt)) / (1 - np.exp(1j * w * euler_dt))
    check("Euler steps of 0.01 s end where the geometric series of their headings says",
          abs(ex - series.real) < 1e-9 and abs(ey - series.imag) < 1e-9)
    # 11_3: at the elbow-straight pose, what the tip moving 1 m/s along the shoulder-wrist line costs in turning
    f_es = chain(elbow_straight)[1]
    tip_es = fk(elbow_straight)[:3, 3]
    along_es = f_es[5][:3, 3] - f_es[1][:3, 3]
    along_es = along_es / np.linalg.norm(along_es)
    j_es = space_jacobian(elbow_straight)
    pull_row = along_es @ (j_es[3:] - hat(tip_es) @ j_es[:3])  # tip velocity along the line, per joint speed
    spin_gram = j_es[:3].T @ j_es[:3]
    cheapest = 1 / math.sqrt(pull_row @ np.linalg.pinv(spin_gram) @ pull_row)  # least |spin| at 1 m/s along the line
    emit("G-D.elbow_straight.turn_per_m_s", fx(cheapest))
    check("the pull along the line can be had by turning (it is in the range of the spin rows)",
          np.linalg.norm(pull_row - spin_gram @ np.linalg.pinv(spin_gram) @ pull_row) < 1e-6)
    check("that least turn is 1 over the tip's distance from the line, from triangle geometry",
          abs(cheapest - 1 / point_line(tip_es, f_es[1][:3, 3], along_es)) < 1e-6)
    # 13_2: the same two triples of ring angles with the rings nested the other way round (x outside, z inside)
    def xyz_rings(outer, middle, inner):
        return rotx(outer * DEG) @ roty(middle * DEG) @ rotz(inner * DEG)

    other = angle_axis(xyz_rings(0, 89, 0).T @ xyz_rings(90, 89, 90))[0]
    emit("G-D.near.apart_other_order", fx(other))
    cos_other = (np.trace(xyz_rings(0, 89, 0).T @ xyz_rings(90, 89, 90)) - 1) / 2
    check("the other nesting order's angle agrees with the trace formula",
          abs(other - math.acos(max(-1.0, min(1.0, cos_other))) / DEG) < 1e-4)
    # 11_4: the same gripper pose reached at rank 6, polished from a one-decimal start by the same Newton steps as G-I
    target = fk(elbow_only)
    q_w = degs(-111.8, 39.0, 68.2, 90, -69.1, 41.0, 110.9)
    for _ in range(30):
        q_w = q_w + np.linalg.pinv(body_jacobian(q_w)) @ twist_of(logm(np.linalg.inv(fk(q_w)) @ target))
    q_w_deg = q_w / DEG
    given = degs(-111.8, 39.0, 68.2, 90, -69.1, 41.0, 110.9)
    miss = np.linalg.inv(target) @ fk(given)
    emit("G-D.witness.q_deg", vec(np.round(q_w_deg, 1), 1))
    emit("G-D.witness.exact_q_deg", vec(q_w_deg, 3))
    emit("G-D.witness.rank", rank(space_jacobian(q_w)))
    emit("G-D.witness.within_limits", int(all(abs(a_) <= lim for a_, lim in zip(q_w_deg, LIMITS_DEG))))
    emit("G-D.witness.rounded_miss", f"{fx(np.linalg.norm(miss[:3, 3]) * 1000, 2)},{fx(angle_axis(miss[:3, :3])[0], 3)}")
    check("the witness reaches the pose of the elbow's quarter turn exactly, at rank 6 where that one has 5",
          np.abs(fk(q_w) - target).max() < 1e-9 and rank(space_jacobian(q_w)) == 6 and rank(space_jacobian(elbow_only)) == 5)
    check("the witness is the elbow at a quarter turn and its other angles, one decimal each, as the description gives them",
          abs(q_w_deg[3] - 90) < 1e-9 and np.abs(np.round(q_w_deg, 1) - given / DEG).max() < 1e-9)
    # 11_6: the L shape with one more joint taken away (G-J has joints 3 and 7)
    for k in (1, 2, 5):
        emit(f"G-D.rank.ell.without_{k}", rank(np.delete(j_ell, k - 1, axis=1)))
    # 13_6: how much each Newton step gains, from the unrounded distances (G-I.ik.dist_mm prints them to 0.01 mm)
    factors = [dist[i] / dist[i + 1] for i in range(len(dist) - 1)]
    gains = [dist[i] - dist[i + 1] for i in range(len(dist) - 1)]
    emit("G-D.ik.factors", vec(factors, 2))
    emit("G-D.ik.gains_mm", vec(gains, 2))
    check("each Newton step cuts the distance by a bigger factor than the last, while the gain in millimetres falls",
          all(x < y for x, y in zip(factors, factors[1:])) and all(x > y for x, y in zip(gains, gains[1:])))
    # 15_1: the width of the Euler-stepped body across its axis, as a percentage over the rigid 1
    width = np.linalg.svd(added, compute_uv=False)[0]
    emit("G-D.drift.width_percent", fx((width - 1) * 100, 2))
    check("across its axis the body grows by (1 + dt^2) per step to the half power",
          abs(width - (1 + dt * dt) ** (steps / 2)) < 1e-9)

    # ------------------------------------------------------------ G-X the edges
    angles = [0, 30, 60, 90, 120, 150]
    turns = [turn(axis, a) for axis in CUBE_AXES for a in angles]
    held = total = 0
    for i in range(0, len(turns), 5):
        for k in range(0, len(turns), 7):
            r = turns[i] @ turns[k]
            total += 1
            values, vectors = np.linalg.eig(r)
            one_axis = np.abs(values - 1).min() < 1e-9
            held += int(np.allclose(r.T @ r, np.eye(3)) and abs(np.linalg.det(r) - 1) < 1e-9 and one_axis)
    emit("G-X1.one_turn", f"{held}/{total}")
    # the 24 turns that carry a cube onto itself: which pairs do not care about order
    cube = [np.eye(3)]
    grew = True
    while grew:
        grew = False
        for a_ in list(cube):
            for g in (tilt, roll):
                c = np.round(g @ a_)
                if not any(np.array_equal(c, b_) for b_ in cube):
                    cube.append(c)
                    grew = True
    swaps = sum(int(np.array_equal(a_ @ b_, b_ @ a_)) for a_ in cube for b_ in cube)
    emit("G-X2.cube_turns", len(cube))
    emit("G-X2.order_free_pairs", f"{swaps}/{len(cube) ** 2}")
    emit("G-X2.same_axis_swap", int(np.allclose(rotx(40 * DEG) @ rotx(70 * DEG), rotx(70 * DEG) @ rotx(40 * DEG))))
    emit("G-X2.half_turns_swap", int(np.allclose(rotx(PI) @ roty(PI), roty(PI) @ rotx(PI))))
    # every pose is one screw: turns of 30 to 150 degrees about 13 axes and no
    # turn (counted once: it is the same for every axis), with 4 slides
    distinct = [(CUBE_AXES[0], 0)] + [(axis, a) for axis in CUBE_AXES for a in angles if a != 0]
    slides = [(0, 0, 0), (0.3, 0, 0), (0.2, -0.1, 0.4), (-0.5, 0.25, 0.1)]
    held = total = 0
    for axis, a in distinct:
        for s in slides:
            t_ = pose(turn(axis, a), s)
            total += 1
            held += int(np.abs(from_screw(*screw_of(t_)) - t_).max() < 1e-9)
    emit("G-X3.one_screw", f"{held}/{total}")
    emit("G-X3.pure_slide.turn", fx(screw_of(pose(np.eye(3), (0.3, 0, 0)))[0]))
    # a blend of two turns is a turn only when the two are the same
    held = total = 0
    for axis, a in distinct:
        r = turn(axis, a)
        d = np.linalg.det((np.eye(3) + r) / 2)
        total += 1
        held += int(abs(d - math.cos(a * DEG / 2) ** 2) < 1e-12 and (d < 1 - 1e-12) == (a != 0))
    emit("G-X4.blend", f"{held}/{total}")
    # on a floor, a steady velocity is a turn about a point, or a slide
    held = total = 0
    for vx in (-0.5, 0.0, 0.5):
        for vy in (-0.5, 0.0, 0.5):
            for spin_deg in (-60, -30, 0, 30, 60):
                total += 1
                spin_ = spin_deg * DEG
                pts = [planar(vx, vy, spin_, tt)[:2, 2] for tt in (0.5, 1.0, 2.5, 4.0, 7.0)]
                if spin_deg == 0:
                    ok = all(np.allclose(pt_, np.array([vx, vy]) * tt) for pt_, tt in zip(pts, (0.5, 1.0, 2.5, 4.0, 7.0)))
                else:
                    centre = np.array([-vy / spin_, vx / spin_])
                    ok = all(abs(np.linalg.norm(pt_ - centre) - np.linalg.norm(centre)) < 1e-9 for pt_ in pts)
                held += int(ok)
    emit("G-X5.floor", f"{held}/{total}")
    # near gimbal lock the three ring axes never span the full volume
    # every order of three rings loses a direction at some middle angle: the
    # twelve orders with neighbouring rings about different axes
    about = {"x": rotx, "y": roty, "z": rotz}
    unit_of = {"x": np.array([1.0, 0, 0]), "y": np.array([0, 1.0, 0]), "z": np.array([0, 0, 1.0])}
    orders = [(a, b, c) for a in "xyz" for b in "xyz" for c in "xyz" if a != b and b != c]
    locked, where = 0, {}
    for a, b, c in orders:
        flat = [m for m in range(0, 181)
                if abs(np.linalg.det(np.column_stack([unit_of[a], unit_of[b], about[b](m * DEG) @ unit_of[c]]))) < 1e-9]
        locked += int(bool(flat))
        where["".join((a, b, c))] = flat
    emit("G-X6.ring_orders", f"{locked}/{len(orders)}")
    emit("G-X6.bad_angle.zyx", ",".join(str(m) for m in where["zyx"]))
    emit("G-X6.bad_angle.zyz", ",".join(str(m) for m in where["zyz"]))

    # ------------------------------------------------------------ G-Q the quizzes
    quiz = {
        "01_1": (5, "different" if not np.allclose(tw, wt) else "same"),
        "02_2": (5, ints([angle_axis(tw)[0]])),
        "03_2": (4, "same" if np.allclose(ypr(30, 90, 0), ypr(0, 90, -30)) else "different"),
        "04_2": (5, ints([abs(halfway)])),
        "05_4": (5, "next" if np.allclose(spun @ np.eye(3)[0], np.eye(3)[1]) else "other"),
        "06_2": (6, "beside" if abs(end[0, 2]) < 1e-9 and end[1, 2] > 1 else "other"),
        "07_1": (4, 6),
        "08_3": (4, "past" if moved == ["5", "6", "7", "tip"] else "other"),
        "09_4": (4, "side" if abs(fk(degs(90, 90, 0, 90, 0, 0, 0))[0, 3]) < 1e-9 else "other"),
        "10_4": (5, 1 + int(np.argmax([point_line(tip_p, f[:3, 3], f[:3, 2]) for f in f_ell]))),
        "11_2": (4, "lost" if rank(js) < 6 else "all"),
        "12_2": (6, "moves" if N - rank(space_jacobian(swing)) >= 1 else "fixed"),
        "13_5": (5, len(dist) - 1),
        "14_2": (4, "pose" if abs(np.linalg.det(blend) - 1) < 1e-9 else "not"),
        "15_2": (4, "differ" if angle_axis((roty(DEG) @ rotx(DEG)) @ (rotx(DEG) @ roty(DEG)).T)[0] > 0 else "same"),
    }
    for line, (seconds, answer) in quiz.items():
        emit(f"G-Q.{line}.seconds", seconds)
        emit(f"G-Q.{line}.answer", answer)

    # ------------------------------------------------------------ G-L cited
    emit("G-L.kuka.reach_mm", 820)
    emit("G-L.kuka.payload_kg", 14)
    emit("G-L.kuka.axes", 7)
    emit("G-L.kmr.speed_kmh", "3.6")
    emit("G-L.kmr.speed_m_s", fx(3.6 / 3.6, 1))
    emit("G-L.kmr.lateral_kmh", "2.0")
    emit("G-L.apollo.lamp_deg", 70)
    emit("G-L.apollo.lost_deg", 85)
    emit("G-L.apollo.gimbals", 3)
    emit("G-L.euler.year", 1775)
    emit("G-L.mozzi.year", 1763)
    emit("G-L.chasles.year", 1830)
    emit("G-L.rodrigues.year", 1840)
    emit("G-L.lie.year", 1873)
    emit("G-L.lie.born", 1842)
    emit("G-L.lie.died", 1899)
    emit("G-L.brockett.year", 1983)
    check("KUKA's reach is the upper arm plus the forearm, in millimetres", abs((upper + fore) * 1000 - 820) < 1e-9)
    check("the base's quiz speed is within KUKA's published top speed", v <= 3.6 / 3.6)

    scipy_checks(samples)
    optional_source_checks()
    print(
        f"# {len(_EMITTED)} keys, {_CHECKS[0] - len(_FAILED)} of {_CHECKS[0]} cross-checks passed",
        file=sys.stderr,
    )
    if _FAILED:
        print("# FAILED: " + "; ".join(_FAILED), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
