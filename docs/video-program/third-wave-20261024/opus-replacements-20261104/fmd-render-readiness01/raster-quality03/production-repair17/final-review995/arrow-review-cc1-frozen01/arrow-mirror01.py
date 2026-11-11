"""Read-only float mirror of fmn a399d626 `Arrow::build_with_tip_index` (straight arrows only,
path_arc == 0) for the 01_6 preview arrow in build268 the_bump.rs previews().

Mirrors, line for line:
  fmn-library/src/line.rs 462-758 (Arrow::new defaults, key_dimensions, trim, outline,
  rotation about the bounding-box centre, shift so arrow_start sits on the trimmed start),
  line.rs 928-944 (arrow_start = 0.5*(p[0]+p[-3]), arrow_end = p[tip_index]),
  fmn-geom/src/space_ops.rs (normalize(0) = 0, angle_of_vector = atan2(y, x)),
  explainer_kit Frame2::p and draw.rs Plane::new (lp: centre (5.6, 0), width 1.5, x,y in (-1.2, 1.2)).
The out-of-plane tilt is the identity for a planar arrow (pi/2 - acos(0) = 0) and is skipped.
Rust libm and Python libm may differ in the last bit of cos/sin, so the collapsed case's
direction (set by rounding residue) is a prediction only; the stills are the evidence for it.

usage: python3 -I -B arrow-mirror01.py
"""
import math

PX_TILE = 67.5          # 960x540 tile pixels per scene unit
PX_4K = 270.0           # 3840x2160 pixels per scene unit
MED_SMALL_BUFF = 0.25
THICKNESS_MULTIPLIER = 0.015


def frame_p(center, width, height, xr, yr, x, y):
    u = (x - xr[0]) / (xr[1] - xr[0]) - 0.5
    w = (y - yr[0]) / (yr[1] - yr[0]) - 0.5
    return (center[0] + u * width, center[1] + w * height)


def lp_p(re, im):
    xr, yr, width = (-1.2, 1.2), (-1.2, 1.2), 1.5
    height = width * (yr[1] - yr[0]) / (xr[1] - xr[0])
    return frame_p((5.6, 0.0), width, height, xr, yr, re, im)


def norm(v):
    return math.sqrt(v[0] * v[0] + v[1] * v[1])


def normalize(v):
    n = norm(v)
    return (v[0] / n, v[1] / n) if n > 0.0 else (0.0, 0.0)


def key_dimensions(length, thickness=3.0, tip_width_ratio=5.0, tip_angle=math.pi / 3.0,
                   max_tip=0.5, max_width=0.1):
    def fdiv(a, b):
        return math.inf if b == 0.0 else a / b
    width = thickness * THICKNESS_MULTIPLIER
    w_ratio = fdiv(max_width, fdiv(width, length))
    if w_ratio < 1.0:
        width *= w_ratio
    tip_width = tip_width_ratio * width
    tip_length = tip_width / (2.0 * math.tan(tip_angle / 2.0))
    t_ratio = fdiv(max_tip, fdiv(tip_length, length))
    if t_ratio < 1.0:
        tip_length *= t_ratio
        tip_width *= t_ratio
    return width, tip_width, tip_length


def line_pts(a, b):
    """QuadPath add_line_to records (start, mid, end) triples; only the anchors matter here."""
    return [a, ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2), b]


def build(start, end, buff):
    vect = (end[0] - start[0], end[1] - start[1])
    length = max(norm(vect), 1e-8)
    unit = normalize(vect)
    width, tip_width, tip_length = key_dimensions(length - buff)
    s = (start[0] + unit[0] * buff, start[1] + unit[1] * buff)
    e = (end[0] - unit[0] * buff, end[1] - unit[1] * buff)
    vect2 = (e[0] - s[0], e[1] - s[1])
    length2 = norm(vect2)
    shaft = length2 - tip_length
    base = [(shaft * f, width / 2.0) for f in (1.0, 0.5, 0.0)]
    mirrored = [(p[0], p[1] - width) for p in reversed(base)]
    # path: base anchors, line to (0, tw/2), line to apex (-tl, 0) [tip_index], line to (0, -tw/2),
    # line to mirrored[0], mirrored anchors, line back to base[0]
    poly = list(base) + [(0.0, tip_width / 2.0), (-tip_length, 0.0), (0.0, -tip_width / 2.0)]
    poly += mirrored + [base[0]]
    tip = (-tip_length, 0.0)
    # point records: p[0] = base[0]; p[-3] = start of the closing line = mirrored[-1]
    a_start = ((base[0][0] + mirrored[-1][0]) / 2.0, (base[0][1] + mirrored[-1][1]) / 2.0)
    current = (tip[0] - a_start[0], tip[1] - a_start[1])
    theta = math.atan2(vect2[1], vect2[0]) - math.atan2(current[1], current[0])
    xs, ys = [p[0] for p in poly], [p[1] for p in poly]
    cx, cy = (min(xs) + max(xs)) / 2.0, (min(ys) + max(ys)) / 2.0
    c, sn = math.cos(theta), math.sin(theta)

    def rot(p):
        dx, dy = p[0] - cx, p[1] - cy
        return (cx + c * dx - sn * dy, cy + sn * dx + c * dy)
    poly_r = [rot(p) for p in poly]
    tip_r, a_start_r = rot(tip), rot(a_start)
    shift = (s[0] - a_start_r[0], s[1] - a_start_r[1])
    poly_f = [(p[0] + shift[0], p[1] + shift[1]) for p in poly_r]
    tip_f = (tip_r[0] + shift[0], tip_r[1] + shift[1])
    base_mid_f = (a_start_r[0] + shift[0], a_start_r[1] + shift[1])
    # visual head direction: from the head's base centre (0,0 in the drawing frame) to the apex
    hb = rot((0.0, 0.0))
    hb_f = (hb[0] + shift[0], hb[1] + shift[1])
    head_dir = math.degrees(math.atan2(tip_f[1] - hb_f[1], tip_f[0] - hb_f[0]))
    return dict(length=length, width=width, tip_width=tip_width, tip_length=tip_length,
                trimmed_start=s, trimmed_end=e, trimmed_vect=vect2, trimmed_length=length2,
                shaft=shaft, current=current, theta_deg=math.degrees(theta), poly=poly_f,
                poly_local=poly, tip=tip_f, arrow_start=base_mid_f, head_base=hb_f,
                head_dir_deg=head_dir)


def winding(poly, q):
    """Winding number of the closed polygon around q."""
    wn = 0
    for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]):
        if y0 <= q[1] < y1 and (x1 - x0) * (q[1] - y0) - (q[0] - x0) * (y1 - y0) > 0:
            wn += 1
        elif y1 <= q[1] < y0 and (x1 - x0) * (q[1] - y0) - (q[0] - x0) * (y1 - y0) < 0:
            wn -= 1
    return wn


def tile_px(p):
    return (480.0 + p[0] * PX_TILE, 270.0 - p[1] * PX_TILE)


O = lp_p(0.0, 0.0)
END = lp_p(0.8 * math.cos(0.9), 0.8 * math.sin(0.9))
R = 0.8 * 1.5 / 2.4               # ring radius in scene units (lp.unit() = 0.625)
RING_HALF = 4.0 * 0.01 / 2.0      # stroke width 4.0 -> 0.04 scene units full width
print("lp.p(0) =", O, " lp.p(polar(0.8,0.9)) =", END, " requested length =", norm((END[0] - O[0], END[1] - O[1])))
print("ring radius %.6f, stroke occupies r in [%.4f, %.4f]" % (R, R - RING_HALF, R + RING_HALF))
print("ring centre tile px (%.1f, %.1f)" % tile_px(O))
for buff in (MED_SMALL_BUFF, 0.0):
    b = build(O, END, buff)
    print("\n=== buff %.2f ===" % buff)
    print("key_dimensions(length - buff = %.4f): width %.5f  tip_width %.5f  tip_length %.5f"
          % (b["length"] - buff, b["width"], b["tip_width"], b["tip_length"]))
    print("trimmed start", b["trimmed_start"], "trimmed end", b["trimmed_end"])
    print("trimmed vect", b["trimmed_vect"], "trimmed length %.3e" % b["trimmed_length"])
    print("shaft length (trimmed length - tip_length) = %.5f" % b["shaft"])
    print("arrow_vector before rotation (apex - arrow_start) =", b["current"])
    print("rotation applied %.4f deg" % b["theta_deg"])
    tip, st = b["tip"], b["arrow_start"]
    r_tip = norm((tip[0] - O[0], tip[1] - O[1]))
    r_start = norm((st[0] - O[0], st[1] - O[1]))
    print("apex at r = %.5f (ring centreline %.5f)  shaft back edge centre at r = %.5f" % (r_tip, R, r_start))
    print("apex tile px (%.1f, %.1f); head base centre tile px (%.1f, %.1f)" % (tile_px(tip) + tile_px(b["head_base"])))
    print("visual head direction %.2f deg (requested radius direction %.2f deg)" % (b["head_dir_deg"], math.degrees(0.9)))
    print("head %.1f x %.1f tile px (%.1f x %.1f px at 4K); shaft width %.1f tile px (%.1f px at 4K)"
          % (b["tip_length"] * PX_TILE, b["tip_width"] * PX_TILE, b["tip_length"] * PX_4K,
             b["tip_width"] * PX_4K, b["width"] * PX_TILE, b["width"] * PX_4K))
    # overlap of the head with the ring band, along the radius
    if r_tip > R - RING_HALF:
        d = r_tip - (R - RING_HALF)
        half_w = d * math.tan(math.pi / 6.0)
        print("apex enters the ring band by %.4f units (%.1f px at 4K, %.1f tile px); head half-width at the"
              " band's inner edge %.4f units (%.1f px at 4K); ring band beyond apex %.4f units (%.1f px at 4K)"
              % (d, d * PX_4K, d * PX_TILE, half_w, half_w * PX_4K, R + RING_HALF - r_tip, (R + RING_HALF - r_tip) * PX_4K))
    else:
        print("apex stays %.4f units inside the ring band (%.1f px at 4K)" % ((R - RING_HALF) - r_tip, ((R - RING_HALF) - r_tip) * PX_4K))
    # winding of probes in the local drawing frame (before rotation): shaft centre and head flank
    loc = b["poly_local"]
    tl, w, tw = b["tip_length"], b["width"], b["tip_width"]
    probes = {"shaft centre": (b["shaft"] / 2.0 if b["shaft"] > 0 else -tl / 2.0, 0.0),
              "head flank": (-tl * 0.25, tw * 0.3)}
    for k, q in probes.items():
        print("winding at %s %s = %d" % (k, tuple(round(t, 4) for t in q), winding(loc, q)))
