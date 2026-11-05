"""Desk composites of the acted division and the plated 0, with the control that makes them worth reading.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/checks/desk_composites.py

The colours, the curve and the root positions come from a program built by rustc from the film's real
model.rs and the staged `turned_by` and `turned_curve`. The control repaints the field at t = 0 and
requires it to match preview02's frame 356; the base is preview02's frame 362 with its old claim painted
out. The words use the film's own Computer Modern text faces, set by hand here, with the minus and the
not-equal drawn as strokes because those faces lack them: fmn's text and TeX layout did not run, so glyph
placement is approximate and the formulas are not fmn's. These are layout studies, stamped as such. They
are not renders, and nothing about timing or sound can be read from them.
Requires Pillow and rustc. Writes only into checks/desk-composites/ and a new temporary directory.
"""
import glob
import math
import os
import re
import subprocess
import tempfile

from PIL import Image, ImageDraw, ImageFont

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/"
OUT = HERE + "checks/desk-composites/"
STAGED = HERE + "explainer_corpus/complex_analysis_visual/src/chapters/counting.rs"
FROZEN = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/desk-composites/"
CRATE = "explainer_corpus/complex_analysis_visual/src/"
FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting/"
CM = glob.glob(os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-*/*/fmd-font/fonts/computer-modern/"))[0]
ROMAN, ITALIC = CM + "cmunrm.ttf", CM + "cmunti.ttf"

SCALE = 4
BG = (12, 17, 20)
INK = (236, 239, 244)
HERO = (255, 216, 102)
OUTPUT = (131, 193, 103)
PANEL = (26, 33, 48)
EDGE = (59, 67, 82)
FIELD = (78, 139, 582, 643)
TURNS = [0.0, 0.5, 1.0]
# 11_8's output window: centre and pixels per window unit, measured on preview02
WINDOW_CENTRE, WINDOW_UNIT = (410.5, 310.5), 63.0
STAMP = "DESK COMPOSITE: real model.rs colours, curve and roots; hand-set words. NOT a render. Timing and sound unknown."


def native(x, y):
    return 640 + 90 * x, 360 - 90 * y


def read(path):
    return open(path, encoding="utf-8").read()


def balanced(text, start):
    depth, at = 0, text.index("{", start)
    while True:
        depth += {"{": 1, "}": -1}.get(text[at], 0)
        if text[at] == "}" and depth == 0:
            return text[start : at + 1]
        at += 1


def compute(work):
    """Field cells, curve and roots from the real sources, for each t."""
    staged, draw = read(STAGED), read(CRATE + "chapters/draw.rs")
    lift = [
        re.search(r"const LEADING: C = .*;", staged).group(0),
        re.search(r"const ROOT_WALK_SQUASH: f64 = .*;", staged).group(0),
        re.search(r"pub const OUT_R: f64 = .*;", staged).group(0),
        re.search(r"pub const WALK_SAMPLES: usize = .*;", draw).group(0),
        balanced(draw, draw.index("pub fn squash(")),
        balanced(staged, staged.index("fn turned_by(")),
        balanced(staged, staged.index("fn turned_curve(")),
    ]
    with open(os.path.join(work, "model.rs"), "w", encoding="utf-8") as out:
        out.write(read(CRATE + "model.rs"))
    with open(os.path.join(work, "main.rs"), "w", encoding="utf-8") as out:
        out.write(
            "#![allow(dead_code, unused_imports)]\nmod model;\nuse model::{C, Func, ONE, ZERO};\nuse std::f64::consts::TAU;\nuse std::io::Write;\n"
            + "\n".join(lift)
            + """
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let t: f64 = args[1].parse().unwrap();
    let by = turned_by(t);
    let (cols, rows) = (160usize, 160usize);
    let mut bytes = Vec::with_capacity(cols * rows * 3);
    for r in 0..rows {
        for c in 0..cols {
            let z = model::cell_input((-2.0, 2.0), (-2.0, 2.0), cols, rows, r, c);
            let px = model::portrait(Func::Quintic.eval(z).map(|w| by * w), 1.0).unwrap_or([12.0 / 255.0, 17.0 / 255.0, 20.0 / 255.0]);
            for k in 0..3 {
                bytes.push((px[k].clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
    }
    let mut f = std::fs::File::create(&args[2]).unwrap();
    f.write_all(&bytes).unwrap();
    let mut g = std::fs::File::create(&args[3]).unwrap();
    for w in turned_curve(by) {
        writeln!(g, "{} {}", w.re, w.im).unwrap();
    }
    let mut h = std::fs::File::create(&args[4]).unwrap();
    for z in model::quintic_roots() {
        writeln!(h, "{} {}", z.re, z.im).unwrap();
    }
}
"""
        )
    binary = os.path.join(work, "desk.bin")
    subprocess.run(["rustc", "--edition", "2024", os.path.join(work, "main.rs"), "-o", binary, "--cap-lints", "allow", "-O"], check=True, capture_output=True)
    pictures, roots = {}, []
    for t in TURNS:
        cells, curve, rooted = (os.path.join(work, f"{name}{t}") for name in ("cells", "curve", "roots"))
        subprocess.run([binary, str(t), cells, curve, rooted], check=True)
        field = Image.frombytes("RGB", (160, 160), open(cells, "rb").read())
        points = [tuple(float(n) for n in line.split()) for line in open(curve, encoding="utf-8")]
        roots = [tuple(float(n) for n in line.split()) for line in open(rooted, encoding="utf-8")]
        pictures[t] = (field, points)
    return pictures, roots


def control(field):
    """t = 0 against the film's own picture of the quintic: frame 356, the marks not yet back on it."""
    real = Image.open(FRAMES + "frame-0356.png").convert("RGB").crop(FIELD)
    mine = field.resize(real.size, Image.NEAREST)
    pa, pb = real.load(), mine.load()
    diffs, near = [], 0
    for y in range(real.size[1]):
        for x in range(real.size[0]):
            d = max(abs(pa[x, y][i] - pb[x, y][i]) for i in range(3))
            diffs.append(d)
            near += d <= 24
    return sum(diffs) / len(diffs), near / len(diffs)


class Canvas:
    """Everything drawn at SCALE times the size, then reduced, for smooth edges."""

    def __init__(self, base):
        self.big = base.resize((base.width * SCALE, base.height * SCALE), Image.BICUBIC)
        self.draw = ImageDraw.Draw(self.big)

    def font(self, path, pt):
        return ImageFont.truetype(path, round(pt * 0.9 * SCALE))

    def rect(self, box, fill):
        self.draw.rectangle([v * SCALE for v in box], fill=fill)

    def paste_field(self, field):
        w, h = FIELD[2] - FIELD[0], FIELD[3] - FIELD[1]
        self.big.paste(field.resize((w * SCALE, h * SCALE), Image.NEAREST), (FIELD[0] * SCALE, FIELD[1] * SCALE))

    def line(self, pts, color, width):
        self.draw.line([(x * SCALE, y * SCALE) for x, y in pts], fill=color, width=round(width * 0.9 * SCALE), joint="curve")

    def disc(self, centre, r, color):
        x, y = centre
        self.draw.ellipse([(x - r) * SCALE, (y - r) * SCALE, (x + r) * SCALE, (y + r) * SCALE], fill=color)

    def advance(self, s, face, pt):
        """Width in native pixels; the minus and the not-equal have cmr10's 0.7778 em."""
        font, total = self.font(face, pt), 0.0
        for part in re.split("([−≠])", s):
            total += 0.7778 * pt * 0.9 if part in ("−", "≠") else font.getlength(part) / SCALE
        return total

    def run_width(self, runs, pt):
        return sum(self.advance(s, face, pt * ratio) for s, face, _, ratio, _ in runs)

    def runs(self, runs, pt, centre):
        """A centred row of (text, face, colour, size ratio, raise in ems), returning its width."""
        width = self.run_width(runs, pt)
        x, baseline = centre[0] - width / 2, centre[1] + pt * 0.9 * 0.25
        for s, face, color, ratio, lift in runs:
            size = pt * ratio
            font, y = self.font(face, size), baseline - lift * pt * 0.9
            em = size * 0.9
            for part in re.split("([−≠])", s):
                if part in ("−", "≠"):
                    self.draw.rectangle([(x + 0.09 * em) * SCALE, (y - 0.27 * em) * SCALE, (x + 0.69 * em) * SCALE, (y - 0.23 * em) * SCALE], fill=color)
                    if part == "≠":
                        self.draw.rectangle([(x + 0.09 * em) * SCALE, (y - 0.42 * em) * SCALE, (x + 0.69 * em) * SCALE, (y - 0.38 * em) * SCALE], fill=color)
                        self.draw.line([(x + 0.5 * em) * SCALE, (y - 0.62 * em) * SCALE, (x + 0.28 * em) * SCALE, (y - 0.02 * em) * SCALE], fill=color, width=round(0.045 * em * SCALE))
                    x += 0.7778 * em
                elif part:
                    self.draw.text((x * SCALE, y * SCALE), part, font=font, fill=color, anchor="ls")
                    x += font.getlength(part) / SCALE
        return width

    def card(self, centre, w, h):
        x, y = centre
        box = [(x - w / 2) * SCALE, (y - h / 2) * SCALE, (x + w / 2) * SCALE, (y + h / 2) * SCALE]
        self.draw.rounded_rectangle(box, radius=11 * SCALE, fill=PANEL, outline=EDGE, width=2 * SCALE)

    def reduced(self):
        return self.big.resize((self.big.width // SCALE, self.big.height // SCALE), Image.LANCZOS)

    def finish(self, path):
        small = self.reduced()
        d = ImageDraw.Draw(small)
        d.rectangle([0, 700, 1280, 720], fill=(60, 10, 10))
        d.text((8, 703), STAMP, fill=(255, 220, 220), font=ImageFont.truetype(ROMAN, 13))
        small.save(path)


def r(s, color=INK, ratio=1.0, lift=0.0):
    return (s, ROMAN, color, ratio, lift)


def i(s, color=INK, ratio=1.0, lift=0.0):
    return (s, ITALIC, color, ratio, lift)


BEFORE = [i("f"), r("("), i("z"), r(") = "), r("2", HERO), i("i", HERO), i("z"), r("5", INK, 0.7, 0.36), r(" − 6"), i("iz"), r(" + 2"), i("i")]
AFTER = [i("p"), r("("), i("z"), r(") = "), i("f"), r("("), i("z"), r(") ÷ "), r("2", HERO), i("i", HERO), r(" = "), i("z"), r("5", INK, 0.7, 0.36), r(" − 3"), i("z"), r(" + 1")]
DEGREE = [i("p"), r("("), i("z"), r(") = "), i("z"), i("n", INK, 0.7, 0.36), r(" + ···")]
CLAIM = "nonconstant polynomials have roots"
EVERY_OUTPUT = "every output ÷ 2i  (2i ≠ 0)"
ZERO_KEPT = "0 ÷ 2i = 0, and nothing else lands on 0"
SAME = "same five roots"


def study(pictures, marks, t, rows, above_output=None, same=False):
    base = Image.open(FRAMES + "frame-0362.png").convert("RGB")
    canvas = Canvas(base)
    canvas.rect((60, 20, 690, 660), BG)
    field, curve = pictures[t]
    canvas.paste_field(field)
    canvas.line([(329.5 + 126 * math.cos(math.radians(k)), 391.5 + 126 * math.sin(math.radians(k))) for k in range(361)], HERO, 5)
    for x, y in marks:
        canvas.disc((x, y), 0.11 * 90, BG)
        canvas.disc((x, y), 0.07 * 90, INK)
    # the output plane: origin (932.5, 391.5), 150 px a unit
    canvas.line([(932.5 + 150 * a, 391.5 - 150 * b) for a, b in curve], OUTPUT, 4)
    widths = {"claim": canvas.runs([r(CLAIM, HERO)], 30, native(-3.45, 3.5))}
    for runs, pt in rows:
        widths["row"] = canvas.runs(runs, pt, native(-3.45, 2.83))
    if above_output:
        widths["over output"] = canvas.runs([r(above_output)], 24, native(3.25, 3.15))
    if same:
        centre = (329.5, 391.5 + 126 * 1.72)
        w = canvas.run_width([r(SAME)], 24)
        canvas.card(centre, w + 45, 47)
        widths["card"] = w + 45
        canvas.runs([r(SAME)], 24, centre)
    return canvas, widths


def plated(im):
    canvas = Canvas(im)
    centre = (WINDOW_CENTRE[0] + WINDOW_UNIT * -0.25, WINDOW_CENTRE[1] + WINDOW_UNIT * 0.3)
    canvas.disc(centre, 0.105 * 90, BG)
    canvas.runs([r("0")], 20, centre)
    return canvas


def plates():
    """11_8's name over the line and the rim: full frames for the probe, and one sheet to read."""
    crop = (340, 260, 480, 370)
    sheet = Image.new("RGB", (1260, 660), BG)
    for k, n in enumerate((546, 547, 548)):
        raw = Image.open(FRAMES + f"frame-{n:04d}.png").convert("RGB")
        sheet.paste(raw.crop(crop).resize((420, 330), Image.LANCZOS), (k * 420, 0))
        sheet.paste(plated(raw).reduced().crop(crop).resize((420, 330), Image.LANCZOS), (k * 420, 330))
    d = ImageDraw.Draw(sheet)
    d.rectangle([0, 640, 1260, 660], fill=(60, 10, 10))
    d.text((8, 643), "DESK COMPOSITE, not a render. Top: preview02 frames 546, 547, 548. Bottom: the same with the staged plate and name pasted on.", fill=(255, 220, 220), font=ImageFont.truetype(ROMAN, 13))
    sheet.save(OUT + "zero-plate-frames-546-548.png")
    # the frozen folder's label composites, or the raw frame where it has none, with the plate on
    for n in (535, 544, 546, 547, 548, 549, 550, 551):
        frozen = f"{FROZEN}reach-from-frame-{n:04d}.png"
        im = Image.open(frozen if os.path.exists(frozen) else FRAMES + f"frame-{n:04d}.png").convert("RGB")
        plated(im).finish(f"{OUT}plate-from-frame-{n:04d}.png")


with tempfile.TemporaryDirectory(prefix="complex11-desk-") as work:
    computed, zeros = compute(work)
mean, share = control(computed[0.0][0])
print(f"control: field at t = 0 against frame 356, mean largest-channel difference {mean:.1f}, {share:.1%} of pixels within 24 levels")
good = share > 0.9
print(("PASS" if good else "FAIL") + " control: the desk field is the film's own picture")
marks_at = [(329.5 + 126 * a, 391.5 - 126 * b) for a, b in zeros]
print("marks at native " + ", ".join(f"({x:.0f}, {y:.0f})" for x, y in marks_at) + "; frame 340 shows (372, 391), (482, 391), (154, 391), (319, 224), (319, 558)")
good = good and all(min(math.dist(m, known) for known in [(372, 391), (482, 391), (154, 391), (319, 224), (319, 558)]) < 4 for m in marks_at)

os.makedirs(OUT, exist_ok=True)
studies = [
    ("a-end-of-sentence-0-t1.png", 1.0, [(BEFORE, 32)], None, False),
    ("b-mid-turn-sentence-1-t05.png", 0.5, [(BEFORE, 32)], EVERY_OUTPUT, False),
    ("c-which-leaves-t0.png", 0.0, [(AFTER, 32)], ZERO_KEPT, True),
    ("d-sentence-2-t0.png", 0.0, [(DEGREE, 36)], ZERO_KEPT, True),
]
for name, t, rows, over, same in studies:
    canvas, widths = study(computed, marks_at, t, rows, over, same)
    canvas.finish(OUT + name)
    print(name, {k: round(v) for k, v in widths.items()})
plates()
print("RESULT " + ("PASS" if good else "FAIL"))
