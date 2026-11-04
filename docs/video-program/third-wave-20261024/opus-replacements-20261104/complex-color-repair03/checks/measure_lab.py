"""Measure a plate_lab run: row seams, stairs against truth, fades, and whether vertex colours survive. Measures, not a verdict.

    python3 -B .../checks/measure_lab.py OUT [--sheets NEW_DIR]     a real run's folder, from examples/plate_lab.rs
    python3 -B .../checks/measure_lab.py --self-test NEW_DIR        synthetic frames from engine_mirror, then planted defects

Plain python3 -B: numpy and PIL. Reads OUT and writes nothing there. --sheets writes comparison sheets for a person's eye
to a folder that must not exist. Exit 1 if a control fails, which means the run or this script cannot be trusted; a
candidate looking worse than another is a result, and exits 0.

What each section can and cannot say:
  A row seams   Rows from the frame alone (residual after a running mean) and against the same subject's tile candidate.
                Against tile, all that differs is what happens at row boundaries, so it isolates the seams.
  B the mirror  engine_mirror's prediction of the same frame. Agreement here is the first test of the mirror above 720p.
  C stairs      rms against truth (the portrait at 16 samples a pixel). Truth is a reference: no plate is meant to reach
                it. A lower number is not a judgement that the picture looks better. That is for a person with the sheets.
  D fades       Each frame against the ideal: the scene's own hold frame composited at the frame's fitted opacity. Bright
                rows are double cover; a large rms over the plate is colour that moved.
  E colours     The hold frame against the still of the same candidate, and flat_control against now, which has to fail.
  F repaints    That the crossfade changed the pixels, and landed where the mirror says the target function is.
  G cost        From receipt.tsv: seconds a frame, and bytes a frame from the files.
"""
import csv
import math
import pathlib
import sys

import numpy as np
from PIL import Image, ImageDraw

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_mirror as em  # noqa: E402

FUNC_OF = {"z": "z", "z2": "z^2", "tan": "tan", "zoom": "z^3-1", "sinrecip": "sin(1/z)", "sin": "sin", "recip": "1/z", "bump": "bump", "openbump": "bump"}
SAME_GRID = ("now", "overlap", "overlap06", "tile", "tile_raw")
GUARD = 1e-4
BG = em.decode(np.array(em.BACKGROUND))


def read_tsv(path):
    with open(path, encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=float) / 255.0


def rms(a, b=None):
    d = a if b is None else a - b
    return float(np.sqrt(np.mean(d * d)))


class Lab:
    def __init__(self, out):
        self.out = pathlib.Path(out)
        self.plates = {r["subject"]: r for r in read_tsv(self.out / "plates.tsv")}
        w, h = next(iter(self.plates.values()))["res"].split("x")
        self.w, self.h = int(w), int(h)
        self.scale = self.h / em.FRAME_HEIGHT
        self.window = int(round(9 * self.h / 720)) | 1

    def box(self, subject):
        """The plate's box inset past its edges and its round caps, so only the picture is measured."""
        r = self.plates[subject]
        m = int(math.ceil(0.035 * self.scale)) + 1
        return int(r["x0"]) + m, int(r["y0"]) + m, int(r["x1"]) - m, int(r["y1"]) - m

    def mirror_plate(self, subject):
        r = self.plates[subject]
        left, top, width, height = (float(r[k]) for k in ("left", "top", "width", "height"))
        return em.Plate((left + width / 2.0, top - height / 2.0), width, (float(r["re0"]), float(r["re1"])), (float(r["im0"]), float(r["im1"])))

    def enhance(self, subject):
        return float(self.plates[subject]["enhance"])

    def scene(self, kind, subject, candidate):
        d = self.out / kind / f"{subject}__{candidate}"
        return sorted(d.glob("*.png")) if d.is_dir() else []

    def still(self, subject, candidate):
        files = self.scene("stills", subject, candidate)
        return load(files[0]) if files else None

    def scenes(self, kind):
        d = self.out / kind
        if not d.is_dir():
            return []
        return sorted(tuple(x.name.split("__")) for x in d.iterdir() if x.is_dir())


def crop(a, box):
    x0, y0, x1, y1 = box
    return a[y0:y1, x0:x1]


def row_series(a, box):
    return em.luma(crop(a, box) * 255.0).mean(axis=1)


def residual(series, window):
    kernel = np.ones(window) / window
    half = window // 2
    return series[half:-half] - np.convolve(series, kernel, mode="valid")


def downscale2(a):
    h, w = a.shape[0] // 2 * 2, a.shape[1] // 2 * 2
    lin = em.decode(a[:h, :w])
    return em.encode((lin[::2, ::2] + lin[1::2, ::2] + lin[::2, 1::2] + lin[1::2, 1::2]) / 4.0)


def mirror_frame(lab, subject, candidate_aa, overlap, opacity=1.0, flatten=False):
    plate = lab.mirror_plate(subject)
    colours = plate.colours(em.FUNCS[FUNC_OF[subject]], lab.enhance(subject))
    if flatten:
        colours = np.repeat(colours[:, :1], colours.shape[1], axis=1)
    stack = plate.stack(lab.h, candidate_aa, overlap)
    x0, y0, x1, y1 = (int(lab.plates[subject][k]) for k in ("x0", "y0", "x1", "y1"))
    window, _ = em.composite(plate, stack, colours, x0, x1, y0, y1, lab.h, opacity=opacity)
    # the plate's own left and right edges: the mirror paints the column grid out to the box, so clear what lies outside
    left_px = lab.w / 2.0 + plate.left * lab.scale
    right_px = left_px + plate.width * lab.scale
    xs = np.arange(x0, x1) + 0.5
    outside = (xs < left_px) | (xs >= right_px)
    window[:, outside] = np.array(em.BACKGROUND)
    frame = np.tile(np.array(em.BACKGROUND), (lab.h, lab.w, 1))
    frame[y0:y1, x0:x1] = window
    return frame


STYLES = {
    # candidate: aa px, overlap in scene units
    "now": (0.5, 0.0),
    "overlap": (0.5, 0.012),
    "overlap06": (0.5, 0.6 / 270.0),
    "tile": (0.0, GUARD),
    "tile_raw": (0.0, 0.0),
}


def section_a(lab, report):
    print("A. Row seams. Each cell: rms % of the row residual, lowest, highest (luma), and the share of pixel rows at least 1 luma dark. One dark row is the picture's own (the background cell at a zero). A share of them is a lattice.")
    print("   vs tile: the row means' difference from the same subject's tile: rms, lowest, highest.")
    report["seam"] = {}
    for subject in lab.plates:
        base = lab.still(subject, "tile")
        box = lab.box(subject)
        if base is None and lab.still(subject, "now") is None:
            continue
        print(f"  {subject}")
        report["seam"][subject] = {}
        for candidate in ("now", "overlap", "overlap06", "tile", "tile_raw", "tile_p3", "blend", "rows2", "cells2", "cells4", "flat_control"):
            a = lab.still(subject, candidate)
            if a is None:
                continue
            s = row_series(a, box)
            r = residual(s, lab.window)
            dark = 100.0 * float(np.mean(r <= -1.0))
            out = {"rms_pct": 100.0 * rms(r) / float(np.mean(s)), "low": float(r.min()), "high": float(r.max()), "dark": dark}
            text = f"    {candidate:14s} {out['rms_pct']:6.2f}% {out['low']:+7.2f} {out['high']:+7.2f} {dark:5.1f}%"
            if base is not None and candidate in SAME_GRID:
                d = s - row_series(base, box)
                out.update(vs_rms=rms(d), vs_low=float(d.min()), vs_high=float(d.max()))
                text += f"   vs tile {out['vs_rms']:6.3f} {out['vs_low']:+7.2f} {out['vs_high']:+7.2f}"
            report["seam"][subject][candidate] = out
            print(text)


def section_b(lab, report):
    print("\nB. The mirror against the frame, row residuals. r on the same rows, r one row off, and rms of frame and mirror. Pixels: rms of 255 over the plate.")
    report["mirror"] = {}
    for subject in lab.plates:
        for candidate in ("now", "tile"):
            a = lab.still(subject, candidate)
            if a is None:
                continue
            box = lab.box(subject)
            aa, overlap = STYLES[candidate]
            m = mirror_frame(lab, subject, aa, overlap)
            fr, mr = residual(row_series(a, box), lab.window), residual(row_series(m, box), lab.window)
            r = em.correlation(fr, mr) if rms(mr) > 1e-9 and rms(fr) > 1e-9 else float("nan")
            off = em.correlation(fr[:-1], mr[1:]) if rms(mr) > 1e-9 and rms(fr) > 1e-9 else float("nan")
            pixels = 255.0 * rms(crop(a, box), crop(m, box))
            report["mirror"][(subject, candidate)] = {"r": r, "off": off, "pixels": pixels}
            print(f"  {subject:9s} {candidate:6s} r {r:+.3f} off1 {off:+.3f}  rms {rms(fr):.3f} / {rms(mr):.3f}  pixels {pixels:.2f}")


def section_c(lab, report):
    print("\nC. Against truth, encoded 0..255 over the plate: rms at this size, rms after a 2x2 box, largest neighbour jump in luma. A reference, not a target, and not a judgement of the picture.")
    report["truth"] = {}
    for subject in lab.plates:
        path = lab.out / "truth" / f"{subject}.ppm"
        if not path.exists():
            continue
        r = lab.plates[subject]
        origin = int(r["x0"]), int(r["y0"])
        x0, y0, x1, y1 = lab.box(subject)
        truth = load(path)[y0 - origin[1] : y1 - origin[1], x0 - origin[0] : x1 - origin[0]]
        small = downscale2(truth)
        print(f"  {subject}, truth's own largest jump {jump(truth):.0f}")
        for candidate in ("now", "tile", "tile_p3", "blend", "rows2", "cells2", "cells4"):
            a = lab.still(subject, candidate)
            if a is None:
                continue
            c = crop(a, (x0, y0, x1, y1))
            out = {"rms": 255.0 * rms(c, truth), "rms2": 255.0 * rms(downscale2(c), small), "jump": jump(c)}
            report["truth"][(subject, candidate)] = out
            print(f"    {candidate:10s} {out['rms']:6.2f} {out['rms2']:6.2f} {out['jump']:6.0f}")


def jump(a):
    y = em.luma(a * 255.0)
    return float(max(np.abs(np.diff(y, axis=0)).max(), np.abs(np.diff(y, axis=1)).max()))


def fit_alpha(frame, hold):
    a = em.decode(hold) - BG
    b = em.decode(frame) - BG
    return float(np.sum(a * b) / max(float(np.sum(a * a)), 1e-12))


def section_d(lab, report):
    print("\nD. Fades. Each frame against its ideal, the scene's hold frame at the frame's fitted opacity. opacity, pixel rms of 255 over the plate, then the row means' lowest and highest difference in luma.")
    report["fade"] = {}
    for subject, candidate in lab.scenes("fades"):
        files = lab.scene("fades", subject, candidate)
        box = lab.box(subject)
        frames = [crop(load(f), box) for f in files]
        distances = [float(np.mean(np.abs(em.decode(f) - BG))) for f in frames]
        hold = frames[int(np.argmax(distances))]
        print(f"  {subject} {candidate}: {len(frames)} frames")
        rows = []
        for i, f in enumerate(frames):
            alpha = fit_alpha(f, hold)
            ideal = em.encode(BG + (em.decode(hold) - BG) * alpha)
            d = (em.luma(f * 255.0) - em.luma(ideal * 255.0)).mean(axis=1)
            line = {"alpha": alpha, "pixels": 255.0 * rms(f, ideal), "low": float(d.min()), "high": float(d.max())}
            rows.append(line)
            print(f"    {i:2d}  {alpha:5.3f}  {line['pixels']:6.2f}  {line['low']:+7.2f} {line['high']:+7.2f}")
        mid = [r for r in rows if 0.05 < r["alpha"] < 0.95]
        report["fade"][(subject, candidate)] = {
            "frames": len(frames),
            "mid": len(mid),
            "high": max((r["high"] for r in mid), default=float("nan")),
            "low": min((r["low"] for r in mid), default=float("nan")),
            "pixels": max((r["pixels"] for r in mid), default=float("nan")),
        }


def section_e(lab, report):
    print("\nE. Vertex colours in pixels. Hold frame against the same candidate's still, then flat_control against now: rms of 255 over the plate, and the largest difference.")
    report["hold"] = {}
    for subject, candidate in lab.scenes("fades"):
        files = lab.scene("fades", subject, candidate)
        still = lab.still(subject, candidate)
        if still is None or not files:
            continue
        box = lab.box(subject)
        frames = [crop(load(f), box) for f in files]
        hold = frames[int(np.argmax([float(np.mean(np.abs(em.decode(f) - BG))) for f in frames]))]
        c = crop(still, box)
        report["hold"][(subject, candidate)] = {"rms": 255.0 * rms(hold, c), "max": 255.0 * float(np.abs(hold - c).max())}
        print(f"  {subject} {candidate}: hold vs still rms {report['hold'][(subject, candidate)]['rms']:.3f}, largest {report['hold'][(subject, candidate)]['max']:.1f}")
    flat, now = lab.still("z", "flat_control"), lab.still("z", "now")
    report["flat"] = None
    if flat is not None and now is not None:
        box = lab.box("z")
        report["flat"] = 255.0 * rms(crop(flat, box), crop(now, box))
        print(f"  z flat_control vs now: rms {report['flat']:.2f}")


def section_f(lab, report):
    print("\nF. Repaints. First frame against last, rms of 255, and the last frame's luma against the mirror's picture of the target, z^3-1 on the same plate.")
    report["repaint"] = {}
    for subject, candidate in lab.scenes("repaints"):
        files = lab.scene("repaints", subject, candidate)
        if len(files) < 2:
            continue
        box = lab.box(subject)
        first, last = crop(load(files[0]), box), crop(load(files[-1]), box)
        plate = lab.mirror_plate(subject)
        colours = plate.colours(em.FUNCS["z^3-1"], lab.enhance(subject))
        x0, y0, x1, y1 = box
        target, _ = em.composite(plate, plate.stack(lab.h, 0.0, GUARD), colours, x0, x1, y0, y1, lab.h)
        r = em.correlation(em.luma(last).ravel(), em.luma(target).ravel())
        moved = 255.0 * rms(first, last)
        report["repaint"][(subject, candidate)] = {"moved": moved, "r": r}
        print(f"  {subject} {candidate}: moved {moved:.1f}, last frame r against the target {r:+.3f}")


def section_g(lab, report):
    print("\nG. Cost, from this run's receipt and files. Not a promise of what a chapter costs.")
    rows = read_tsv(lab.out / "receipt.tsv")
    report["records_bad"] = [r for r in rows if any(word in r["records"] for word in ("FAILED", "MOVED", "NOTHING"))]
    print("  kind, subject, candidate, frames, rows x cols, evaluate s, build s, s a frame, MB a frame, records")
    for r in rows:
        files = lab.scene(r["kind"], r["subject"], r["candidate"])
        mb = sum(f.stat().st_size for f in files) / max(len(files), 1) / 1e6
        print(f"  {r['kind']:8s} {r['subject']:9s} {r['candidate']:12s} {r['frames']:>3s} {r['rows']:>4s} x {r['cols']:<4s} {float(r['evaluate_s']):7.3f} {float(r['build_s']):7.3f} {float(r['s_per_frame']):8.2f} {mb:7.2f}  {r['records']}")


def analyse(out):
    lab = Lab(out)
    report = {}
    print(f"run {lab.out.name}: {lab.w}x{lab.h}, {lab.scale} px a unit, residual window {lab.window} px rows\n")
    section_a(lab, report)
    section_b(lab, report)
    section_c(lab, report)
    section_d(lab, report)
    section_e(lab, report)
    section_f(lab, report)
    section_g(lab, report)
    return report


def controls(report):
    """Whether this run can be read at all. Each entry: name, passed, what was seen. Missing scenes are not failures."""
    out = []
    z = report.get("seam", {}).get("z", {})
    if "now" in z:
        out.append(("now shows seams on z", z["now"]["dark"] >= 3.0, f"{z['now']['dark']:.1f}% of rows dark, lowest {z['now']['low']:+.2f} luma"))
    key = ("z", "now")
    if key in report.get("mirror", {}):
        out.append(("the mirror lands on now's rows on z", report["mirror"][key]["r"] >= 0.9, f"r {report['mirror'][key]['r']:+.3f}"))
    if report.get("flat") is not None:
        out.append(("flat_control fails the colour test", report["flat"] >= 5.0, f"rms {report['flat']:.2f}"))
    key = ("z", "overlap")
    if key in report.get("fade", {}):
        out.append(("Mail 2167's overlap shows bright rows mid-fade", report["fade"][key]["high"] >= 3.0, f"highest {report['fade'][key]['high']:+.2f} luma"))
    for scene, f in report.get("fade", {}).items():
        out.append((f"{scene[0]} {scene[1]} fade was caught mid-way", f["mid"] >= 2, f"{f['mid']} of {f['frames']} frames"))
    out.append(("no receipt line reports moved or missing records", not report.get("records_bad"), f"{len(report.get('records_bad', []))} lines"))
    return out


def print_controls(results):
    print("\nControls")
    for name, passed, seen in results:
        print(f"  {'PASS' if passed else 'FAIL'}  {name}: {seen}")
    return all(p for _, p, _ in results)


def write_sheets(lab, new_dir):
    """Each subject's same 100% window across candidates, for a person's eye, truth first. A sheet is a real frame's crop."""
    new_dir = pathlib.Path(new_dir)
    new_dir.mkdir(parents=True, exist_ok=False)
    side = int(round(lab.h * 0.21))
    for subject in lab.plates:
        cells = []
        x0, y0, x1, y1 = lab.box(subject)
        cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
        window = (cx - side, cy - side, cx, cy)
        path = lab.out / "truth" / f"{subject}.ppm"
        r = lab.plates[subject]
        if path.exists():
            t = load(path)
            ox, oy = int(r["x0"]), int(r["y0"])
            cells.append(("truth, not a render", t[window[1] - oy : window[3] - oy, window[0] - ox : window[2] - ox]))
        for candidate in ("now", "overlap06", "tile", "tile_p3", "blend", "rows2", "cells2", "cells4"):
            a = lab.still(subject, candidate)
            if a is not None:
                cells.append((candidate, crop(a, window)))
        if len(cells) < 2:
            continue
        per_row = 3
        rows = math.ceil(len(cells) / per_row)
        sheet = Image.new("RGB", (per_row * (side + 8), rows * (side + 28)), (20, 20, 20))
        draw = ImageDraw.Draw(sheet)
        for k, (name, c) in enumerate(cells):
            x, y = (k % per_row) * (side + 8), (k // per_row) * (side + 28)
            draw.text((x + 4, y + 6), f"{subject} {name}", fill=(235, 235, 235))
            sheet.paste(Image.fromarray((np.clip(c, 0, 1) * 255.0 + 0.5).astype(np.uint8)), (x, y + 24))
        sheet.save(new_dir / f"{subject}.png")
        print(f"sheet {new_dir / (subject + '.png')}: {len(cells)} cells, {side} px each at 100%")


# ------------------------------------------------------------------ self-test


def self_test(new_dir):
    """Synthetic frames from the mirror at 720p, written as a lab run would write them. The script has to read them, pass
    its controls, and fail on each planted defect. This proves the measures can fail, and says nothing of a real render."""
    root = pathlib.Path(new_dir)
    root.mkdir(parents=True, exist_ok=False)
    res = (1280, 720)
    subjects = {
        "z": ((-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0), 0.0),
        "tan": ((-2.4, -0.1), 8.4, (-6.6, 6.6), (-3.3, 3.3), 1.0),
    }
    fade_opacities = [0.26, 0.74, 1.0, 1.0, 0.74, 0.26, 0.0]

    def base_run(name):
        out = root / name
        out.mkdir()
        lines = ["subject\tres\tx0\ty0\tx1\ty1\trows_now\tcols_now\tleft\ttop\twidth\theight\tenhance\tre0\tre1\tim0\tim1"]
        for key, (center, width, x, y, enhance) in subjects.items():
            p = em.Plate(center, width, x, y)
            scale = res[1] / em.FRAME_HEIGHT
            bx0 = int(math.floor(res[0] / 2 + p.left * scale))
            bx1 = int(math.ceil(res[0] / 2 + (p.left + p.width) * scale))
            by0 = int(math.floor(res[1] / 2 - p.top * scale))
            by1 = int(math.ceil(res[1] / 2 - (p.top - p.height) * scale))
            lines.append(f"{key}\t{res[0]}x{res[1]}\t{bx0}\t{by0}\t{bx1}\t{by1}\t{p.rows}\t{p.cols}\t{p.left}\t{p.top}\t{p.width}\t{p.height}\t{enhance}\t{x[0]}\t{x[1]}\t{y[0]}\t{y[1]}")
        (out / "plates.tsv").write_text("\n".join(lines) + "\n")
        (out / "receipt.tsv").write_text("kind\tsubject\tcandidate\tframes\trows\tcols\tevaluate_s\tbuild_s\trender_s\ts_per_frame\trecords\n")
        for d in ("stills", "fades", "repaints", "truth"):
            (out / d).mkdir()
        return out

    def save(path, frame):
        path.parent.mkdir(parents=True, exist_ok=True)
        Image.fromarray((np.clip(frame, 0.0, 1.0) * 255.0 + 0.5).astype(np.uint8)).save(path)

    def populate(out, flat_fade=False, seamy_tile=False, honest_flat=True, overlap_is_tile=False):
        lab = Lab(out)
        for subject in subjects:
            for candidate in ("now", "tile", "overlap"):
                aa, overlap = STYLES[candidate]
                if candidate == "tile" and seamy_tile:
                    aa, overlap = STYLES["now"]
                if candidate == "overlap" and overlap_is_tile:
                    aa, overlap = STYLES["tile"]
                save(out / "stills" / f"{subject}__{candidate}" / "frame_000000.png", mirror_frame(lab, subject, aa, overlap))
                if subject == "z":
                    for k, opacity in enumerate(fade_opacities):
                        frame = mirror_frame(lab, subject, aa, overlap, opacity=opacity, flatten=flat_fade and candidate == "tile" and 0.0 < opacity < 1.0)
                        save(out / "fades" / f"{subject}__{candidate}" / f"frame_{k:06d}.png", frame)
        aa, overlap = STYLES["now"]
        save(out / "stills" / "z__flat_control" / "frame_000000.png", mirror_frame(lab, "z", aa, overlap, flatten=honest_flat))

    print("\n=== self-test: a clean synthetic run ===")
    clean = base_run("clean")
    populate(clean)
    report = analyse(clean)
    results = controls(report)
    ok = print_controls(results)
    verdicts = [("clean run passes every control", ok)]
    z = report["seam"]["z"]
    verdicts.append(("tile is free of a dark lattice where now is not", z["tile"]["dark"] < 1.0 and z["now"]["dark"] > 3.0))
    fade = report["fade"]
    verdicts.append(("overlap's mid-fade rows are brighter than tile's", fade[("z", "overlap")]["high"] > fade[("z", "tile")]["high"] + 2.0))
    verdicts.append(("tile's fade stays on its ideal", fade[("z", "tile")]["pixels"] < 1.0))

    print("\n=== self-test: planted, tile drawn with now's seams ===")
    bad = base_run("seamy_tile")
    populate(bad, seamy_tile=True)
    z = analyse(bad)["seam"]["z"]
    verdicts.append(("a seamy tile is caught: its rows read dark", z["tile"]["dark"] > 3.0))

    print("\n=== self-test: planted, tile's colours flattened in the middle of its fade ===")
    bad = base_run("flat_fade")
    populate(bad, flat_fade=True)
    fade = analyse(bad)["fade"]
    verdicts.append(("flattened colours in a fade are caught: pixels far off the ideal", fade[("z", "tile")]["pixels"] > 5.0))

    print("\n=== self-test: planted, flat_control not actually flat ===")
    bad = base_run("not_flat")
    populate(bad, honest_flat=False)
    results = controls(analyse(bad))
    verdicts.append(("the flat control's own failure is caught", not dict((n, p) for n, p, _ in results)["flat_control fails the colour test"]))

    print("\n=== self-test: planted, the overlap candidate drawn without overlap ===")
    bad = base_run("no_overlap")
    populate(bad, overlap_is_tile=True)
    results = controls(analyse(bad))
    verdicts.append(("a bright-row detector with nothing to detect fails its control", not dict((n, p) for n, p, _ in results)["Mail 2167's overlap shows bright rows mid-fade"]))

    print("\n=== self-test result ===")
    for name, passed in verdicts:
        print(f"{'PASS' if passed else 'FAIL'}  {name}")
    passed = all(p for _, p in verdicts)
    print("SELF-TEST", "OK" if passed else "FAILED")
    return passed


def main(argv):
    if len(argv) == 2 and argv[0] == "--self-test":
        return 0 if self_test(argv[1]) else 1
    if len(argv) == 1 or (len(argv) == 3 and argv[1] == "--sheets"):
        report = analyse(argv[0])
        ok = print_controls(controls(report))
        if len(argv) == 3:
            write_sheets(Lab(argv[0]), argv[2])
        print("\nCONTROLS", "HOLD: these numbers can be read. They choose what to look at. They are not a verdict on 4K." if ok else "FAILED: do not read the numbers above as evidence.")
        return 0 if ok else 1
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
