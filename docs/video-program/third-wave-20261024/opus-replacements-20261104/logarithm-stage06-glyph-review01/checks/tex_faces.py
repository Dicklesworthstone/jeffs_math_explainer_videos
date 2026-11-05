"""Which bundled face draws each TeX symbol on the registry, and is the ell on the page that face's ell?

    python3 -B .../checks/tex_faces.py [OUT_PNG]     (from the repo root; PIL and numpy need site packages)

Reads only: the probe binary, registry.tsv, the probe pages and the pinned fmd-math source.

1. Reads the sfnt tables embedded in target/debug/logarithm_mysteries and walks fmd-math's own resolution
   (faces.rs: FaceSet::resolve + default_math_chain, then the roster in FaceId order; parse.rs:170 maps '-' to U+2212).
   The chain and the symbol table are parsed from the pinned source, not retyped.
2. Draws the suspect from each candidate face with PIL and scores it against the page's ink at that glyph's own bbox.
   The page's own neighbours are the controls: L, k and ( must score best on the face the chain predicts.
3. Measures stroke weight: median horizontal ink run per glyph, on the page.
4. Optional: writes OUT_PNG, a side-by-side for the eye.
"""
import hashlib
import io
import re
import statistics
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

BINARY = Path("target/debug/logarithm_mysteries")
RUN = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-glyph01")
PAGES = RUN / "glyphs/pages"
FMD = Path.home() / ".cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/fmd-math/src"
# FaceId order, faces.rs:28-42. Offsets from the skill's sfnt_scan.py for this binary.
ROSTER = [
    ("CM Regular", 0x00998710),
    ("CM Italic", 0x009D1F98),
    ("CM Bold", 0x00A18994),
    ("CM Bold Italic", 0x00A4EC64),
    ("CM Typewriter", 0x00A9460C),
    ("IBM Plex Sans", 0x00AC8C0C),
    ("Noto Sans Math", 0x00AF9B40),
]
REGULAR, ITALIC, SYMBOLS = 0, 1, 6


def sfnt(data, at):
    num = struct.unpack_from(">H", data, at + 4)[0]
    end = 0
    for i in range(num):
        _, _, off, length = struct.unpack_from(">4sIII", data, at + 12 + 16 * i)
        end = max(end, off + length)
    return data[at : at + end]


def cmap_of(font):
    num = struct.unpack_from(">H", font, 4)[0]
    tables = {}
    for i in range(num):
        tag, _, off, length = struct.unpack_from(">4sIII", font, 12 + 16 * i)
        tables[tag.decode("latin-1")] = (off, length)
    base = tables["cmap"][0]
    mapped = {}
    for i in range(struct.unpack_from(">H", font, base + 2)[0]):
        pid, _, off = struct.unpack_from(">HHI", font, base + 4 + 8 * i)
        at = base + off
        fmt = struct.unpack_from(">H", font, at)[0]
        if fmt == 4 and pid in (0, 3):
            segs = struct.unpack_from(">H", font, at + 6)[0] // 2
            end_at = at + 14
            start_at = end_at + 2 * segs + 2
            delta_at = start_at + 2 * segs
            range_at = delta_at + 2 * segs
            for s in range(segs):
                e = struct.unpack_from(">H", font, end_at + 2 * s)[0]
                b = struct.unpack_from(">H", font, start_at + 2 * s)[0]
                d = struct.unpack_from(">h", font, delta_at + 2 * s)[0]
                r = struct.unpack_from(">H", font, range_at + 2 * s)[0]
                for c in range(b, e + 1):
                    if c == 0xFFFF:
                        continue
                    gid = (c + d) & 0xFFFF
                    if r:
                        gid = struct.unpack_from(">H", font, range_at + 2 * s + r + 2 * (c - b))[0]
                        gid = (gid + d) & 0xFFFF if gid else 0
                    if gid:
                        mapped[c] = gid
        elif fmt == 12:
            for g in range(struct.unpack_from(">I", font, at + 12)[0]):
                b, e, first = struct.unpack_from(">III", font, at + 16 + 12 * g)
                for c in range(b, e + 1):
                    mapped[c] = first + c - b
    return mapped


def decode(literal):
    """The registry holds Rust Debug strings; TeX rows only need \\\\ and \\" undone."""
    return literal[1:-1].replace('\\"', '"').replace("\\\\", "\\")


def italicized(ch):
    return (ch.isascii() and ch.isalpha()) or "α" <= ch <= "ω" or ch in "ϵϑϖϱςφϕ"


def resolve(ch, cmaps):
    prefer = [ITALIC, REGULAR, SYMBOLS] if italicized(ch) else [REGULAR, SYMBOLS, ITALIC]
    for i in prefer + list(range(len(cmaps))):
        if ord(ch) in cmaps[i]:
            return i
    return None


def ink_box(arr, level=0.5):
    bg, top = arr.min(), arr.max()
    ys, xs = np.where(arr >= bg + level * (top - bg))
    return xs.min(), ys.min(), xs.max() + 1, ys.max() + 1


def draw_char(font_bytes, ch, height):
    """The character from this face, scaled so its ink is `height` px tall. Returns a float array of its ink box."""
    font = ImageFont.truetype(io.BytesIO(font_bytes), 400)
    im = Image.new("L", (900, 900), 0)
    ImageDraw.Draw(im).text((200, 100), ch, fill=255, font=font)
    a = np.asarray(im, dtype=float)
    x0, y0, x1, y1 = ink_box(a)
    crop = im.crop((x0, y0, x1, y1))
    w = max(1, round(crop.width * height / crop.height))
    return crop.resize((w, height), Image.LANCZOS), crop.width / crop.height


def score(page_box, drawn):
    """Correlation between the page's ink and the drawn glyph, at the page box's size. Also compares aspect."""
    a = np.asarray(page_box.resize(drawn.size, Image.LANCZOS), dtype=float).ravel()
    b = np.asarray(drawn, dtype=float).ravel()
    return float(np.corrcoef(a, b)[0, 1])


def median_run(arr, level=0.5):
    bg, top = arr.min(), arr.max()
    cut = bg + level * (top - bg)
    runs = []
    for row in arr:
        n = 0
        for v in row:
            if v >= cut:
                n += 1
            elif n:
                runs.append(n)
                n = 0
        if n:
            runs.append(n)
    return statistics.median(runs) if runs else 0, len(runs)


# page, name, char, box from touch_test.py at level 0.5 (x0, y0, x1 + 1, y1 + 1), and the row it sits in
GLYPHS = [
    (47, "ell in L(z) - ell", "ℓ", (584, 874, 604, 913), 565),
    (47, "ell in e^ell", "ℓ", (317, 865, 331, 893), 565),
    (41, "ell in (ell)", "ℓ", (72, 403, 92, 441), 497),
    (47, "L, control", "L", (443, 878, 471, 912), 565),
    (47, "paren, control", "(", (476, 875, 487, 925), 565),
    (47, "z, control", "z", (493, 890, 513, 913), 565),
]


def main():
    data = BINARY.read_bytes()
    print(f"binary {hashlib.sha256(data).hexdigest()[:16]}")
    fonts, cmaps = [], []
    for name, at in ROSTER:
        font = sfnt(data, at)
        fonts.append(font)
        cmaps.append(cmap_of(font))
        print(f"  face {len(fonts) - 1} {name}: {len(font)} B, sha256 {hashlib.sha256(font).hexdigest()[:12]}, {len(cmaps[-1])} code points")

    commands = (FMD / "commands.rs").read_text()
    table = dict(re.findall(r"\"(\w+)\"(?:\s*\|\s*\"\w+\")*\s*=>\s*sym\('(.)',", commands))
    for alias_line in re.findall(r"((?:\"\w+\"\s*\|\s*)+\"\w+\")\s*=>\s*sym\('(.)',", commands):
        for alias in re.findall(r"\"(\w+)\"", alias_line[0]):
            table[alias] = alias_line[1]
    faces_rs = (FMD / "faces.rs").read_text()
    print("\nchain source, faces.rs:", "ok" if "&[FACE_REGULAR, FACE_SYMBOLS, FACE_ITALIC]" in faces_rs and "&[FACE_ITALIC, FACE_REGULAR, FACE_SYMBOLS]" in faces_rs else "CHANGED")
    print("parse.rs maps - to U+2212:", "yes" if "'-' => '−'" in (FMD / "parse.rs").read_text() else "NO")

    rows = []
    for number, line in enumerate((RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines(), 1):
        kind, chapters, literal = line.split("\t", 2)
        if kind == "Tex":
            rows.append((number, chapters, decode(literal)))
    print(f"\nTeX rows: {len(rows)}")

    used = {}
    unknown = set()
    for number, _, source in rows:
        # \text{...} interiors are upright text and take their own chain, so only math-mode symbols are walked here.
        stripped = re.sub(r"\\text\{[^{}]*\}", " ", source)
        for name in re.findall(r"\\([A-Za-z]+)", stripped):
            if name in table:
                used.setdefault(table[name], set()).add(number)
            elif name not in {"frac", "tfrac", "sqrt", "left", "right", "text", "mathrm", "operatorname", "quad", "qquad", "ln", "log", "cos", "sin"}:
                unknown.add(name)
        rest = re.sub(r"\\[A-Za-z]+", " ", stripped)
        rest = re.sub(r"\\.", " ", rest)
        for ch in rest:
            if ch == "-":
                used.setdefault("−", set()).add(number)
            elif not ch.isspace() and ch not in "{}^_":
                used.setdefault(ch, set()).add(number)
    print("control sequences not in the sym table and not a known structural command:", sorted(unknown) or "none")

    print("\n# math-mode characters by the face fmd-math's chain resolves them to")
    by_face = {}
    for ch in sorted(used):
        by_face.setdefault(resolve(ch, cmaps), []).append(ch)
    for face, chars in sorted(by_face.items(), key=lambda kv: (kv[0] is None, kv[0])):
        label = ROSTER[face][0] if face is not None else "NO FACE"
        print(f"  {label}: " + " ".join(chars))
    print("\n# math-mode characters outside the CM faces: rows")
    for ch in sorted(used):
        face = resolve(ch, cmaps)
        if face not in (REGULAR, ITALIC):
            print(f"  {ch} U+{ord(ch):04X} {ROSTER[face][0] if face is not None else 'NO FACE'}: {sorted(used[ch])}")

    print("\n# U+2113 in each face")
    for i, (name, _) in enumerate(ROSTER):
        print(f"  {name}: {'mapped gid ' + str(cmaps[i][0x2113]) if 0x2113 in cmaps[i] else 'not mapped'}")

    print("\n# page ink against each candidate face (correlation at the glyph's own box; aspect = width/height)")
    candidates = [(i, ROSTER[i][0]) for i in (REGULAR, ITALIC, SYMBOLS, 5)]
    sheet_rows = []
    for page, name, ch, box, row in GLYPHS:
        im = Image.open(PAGES / f"frame_{page:06d}.png").convert("L").crop(box)
        arr = np.asarray(im, dtype=float)
        run, n = median_run(arr)
        aspect = im.width / im.height
        predicted = resolve(ch, cmaps)
        scores = []
        for i, label in candidates:
            if ord(ch) not in cmaps[i]:
                scores.append((None, label))
                continue
            drawn, drawn_aspect = draw_char(fonts[i], ch, max(40, im.height * 4))
            scores.append((score(im, drawn), f"{label} (aspect {drawn_aspect:.2f})"))
        got = [s for s in scores if s[0] is not None]
        best = max(got, key=lambda s: s[0])[1].split(" (")[0] if got else "none"
        text = "; ".join(f"{label} {'unmapped' if s is None else format(s, '.3f')}" for s, label in scores)
        verdict = "OK" if best == ROSTER[predicted][0] else "DIFFERS"
        print(f"  p{page} row {row} {name}: box {box}, aspect {aspect:.2f}, median horizontal ink run {run} px over {n} runs")
        print(f"      chain predicts {ROSTER[predicted][0]}; best on page {best} [{verdict}]; {text}")
        sheet_rows.append((page, box))

    # Sizes: 1 em = size x 0.01 frame units, 135 px per unit at 1080p (the skill's fmn-text width rule, re-checked by row 360 below).
    print("\n# the sign on a live readout against the sign on a fixed label (drawn at 1000 px per em, thickness in ems)")
    thickness = {}
    for label, ch, face in (("en dash, DecimalNumber", "–", REGULAR), ("minus, fixed labels", "−", SYMBOLS)):
        font = ImageFont.truetype(io.BytesIO(fonts[face]), 1000)
        im = Image.new("L", (2000, 2000), 0)
        ImageDraw.Draw(im).text((200, 200), ch, fill=255, font=font)
        x0, y0, x1, y1 = ink_box(np.asarray(im, dtype=float))
        thickness[label] = (y1 - y0) / 1000
        print(f"  {label} from {ROSTER[face][0]}: thickness {(y1 - y0) / 1000:.3f} em, length {(x1 - x0) / 1000:.3f} em")
    for size, where in ((26.0, "chapter 07 angle, branch_cut.rs:543"), (20.0, "chapter 12 laps so far, what_it_forgets.rs:1073")):
        px_1080 = size * 0.01 * 135
        print(f"  size {size:g}, {where}: en dash {thickness['en dash, DecimalNumber'] * px_1080:.2f} px at 1080p, {2 * thickness['en dash, DecimalNumber'] * px_1080:.2f} px at 4K; a fixed minus would be {thickness['minus, fixed labels'] * px_1080:.2f} / {2 * thickness['minus, fixed labels'] * px_1080:.2f} px")

    if len(sys.argv) > 1:
        out = Path(sys.argv[1])
        words = [("as drawn on p47", None), ("Noto Sans Math U+2113", SYMBOLS), ("CM Italic l (U+006C)", ITALIC), ("CM Regular l (U+006C)", REGULAR)]
        sheet = Image.new("L", (1500, 300), 18)
        d = ImageDraw.Draw(sheet)
        page_im = Image.open(PAGES / "frame_000047.png").convert("L").crop((540, 868, 612, 918))
        sheet.paste(page_im.resize((page_im.width * 4, page_im.height * 4), Image.LANCZOS), (10, 40))
        x = 330
        for label, face in words[1:]:
            font = ImageFont.truetype(io.BytesIO(fonts[face]), 180)
            d.text((x, 30), "ℓ" if face == SYMBOLS else "l", fill=235, font=font)
            x += 390
        small = ImageFont.truetype(io.BytesIO(fonts[REGULAR]), 22)
        x = 10
        for label, _ in words:
            d.text((x, 260), label, fill=170, font=small)
            x = 330 if x == 10 else x + 390
        sheet.save(out)
        print(f"\nwrote {out.name}: the page crop, then PIL draws straight from each bundled face. Not the engine's layout.")


if __name__ == "__main__":
    main()
