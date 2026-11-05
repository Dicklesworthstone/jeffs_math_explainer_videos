"""The frozen measure_lab.py, taught the two subjects probe-plate-lab.patch adds. The frozen script is imported, not edited.

    python3 -B .../checks/measure_probe.py OUT [--sheets NEW_DIR]     a probe run's folder
    python3 -B .../checks/measure_probe.py --self-test NEW_DIR        mirror frames of both new subjects through every section

The mirror's two functions follow model.rs: Lacunary sums 12 terms z^(2^k) and stops at |z| >= 0.98; AbsSquare is |z|^2.
The self-test proves the wiring and the mirror's own consistency. It says nothing of a real render.
"""
import importlib.util
import pathlib
import sys

import numpy as np
from PIL import Image

sys.dont_write_bytecode = True
FROZEN = pathlib.Path(__file__).resolve().parents[2] / "complex-color-repair03/checks/measure_lab.py"
spec = importlib.util.spec_from_file_location("measure_lab", FROZEN)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
em = m.em

TERMS, LIMIT = 12, 0.98


def lacunary(z):
    z = np.asarray(z)
    total, p = np.zeros_like(z), z
    for _ in range(TERMS):
        total = total + p
        p = p * p
    return np.where(np.abs(z) >= LIMIT, np.nan + 0j, total)


em.FUNCS["lacunary"] = lacunary
em.FUNCS["abs2"] = lambda z: np.abs(z) ** 2 + 0j
m.FUNC_OF["lacunary"] = "lacunary"
m.FUNC_OF["abs2"] = "abs2"

PLANES = {
    "lacunary": ((-3.0, -0.35), 5.4, (-1.05, 1.05), (-1.05, 1.05), 1.0),
    "abs2": ((-2.8, -0.25), 5.4, (-2.0, 2.0), (-2.0, 2.0), 1.0),
}


def self_test(new_dir):
    out = pathlib.Path(new_dir)
    out.mkdir(parents=True, exist_ok=False)
    res = (1280, 720)
    scale = res[1] / em.FRAME_HEIGHT
    lines = ["subject\tres\tx0\ty0\tx1\ty1\trows_now\tcols_now\tleft\ttop\twidth\theight\tenhance\tre0\tre1\tim0\tim1"]
    for key, (center, width, x, y, enhance) in PLANES.items():
        p = em.Plate(center, width, x, y)
        bx0, bx1 = int(np.floor(res[0] / 2 + p.left * scale)), int(np.ceil(res[0] / 2 + (p.left + p.width) * scale))
        by0, by1 = int(np.floor(res[1] / 2 - p.top * scale)), int(np.ceil(res[1] / 2 - (p.top - p.height) * scale))
        lines.append(f"{key}\t{res[0]}x{res[1]}\t{bx0}\t{by0}\t{bx1}\t{by1}\t{p.rows}\t{p.cols}\t{p.left}\t{p.top}\t{p.width}\t{p.height}\t{enhance}\t{x[0]}\t{x[1]}\t{y[0]}\t{y[1]}")
    (out / "plates.tsv").write_text("\n".join(lines) + "\n")
    (out / "receipt.tsv").write_text("kind\tsubject\tcandidate\tframes\trows\tcols\tevaluate_s\tbuild_s\trender_s\ts_per_frame\trecords\n")
    lab = m.Lab(out)
    for key in PLANES:
        for candidate in ("now", "tile"):
            aa, overlap = m.STYLES[candidate]
            frame = m.mirror_frame(lab, key, aa, overlap)
            d = out / "stills" / f"{key}__{candidate}"
            d.mkdir(parents=True)
            Image.fromarray((np.clip(frame, 0.0, 1.0) * 255.0 + 0.5).astype(np.uint8)).save(d / "frame_000000.png")
    report = m.analyse(out)
    bad = [k for k, v in report["mirror"].items() if not v["r"] >= 0.99]
    wants = [("lacunary", "now"), ("lacunary", "tile"), ("abs2", "now"), ("abs2", "tile")]
    missing = [k for k in wants if k not in report["mirror"]]
    print(f"\nSELF-TEST mirror rows on both new subjects: {len(report['mirror'])} scenes, below r 0.99: {bad}, missing: {missing}")
    return not bad and not missing


if __name__ == "__main__":
    argv = sys.argv[1:]
    if len(argv) == 2 and argv[0] == "--self-test":
        sys.exit(0 if self_test(argv[1]) else 1)
    sys.exit(m.main(argv))
