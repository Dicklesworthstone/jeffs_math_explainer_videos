"""Is the bundled math-italic face the cause of the theta form and of the z-subscript collision?

python3 -B italic_face_check.py FONTS_DIR OUT.png   (from the repo root; PIL needs site packages)

FONTS_DIR is fmd-font's fonts/computer-modern folder in the franken_markdown checkout at the rev
franken_manim pins (126a530). The script checks that these exact font files sit inside the probe
binary, reports which code points each maps, measures how far each subscript base's ink runs past
its advance, and draws the theta code points from the faces into OUT.png, for looking.
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from font_faces import cmap_of

BINARY = Path("target/debug/complex_analysis_visual")
SIZE = 1000  # px per em, so a length in px reads directly in thousandths of an em


def main():
    fonts, out = Path(sys.argv[1]), Path(sys.argv[2])
    binary = BINARY.read_bytes()
    faces = {}
    for name in ("cmunti.ttf", "cmunrm.ttf"):
        data = (fonts / name).read_bytes()
        print(f"{name}: {len(data)} B; inside the probe binary: {binary.find(data) >= 0}")
        faces[name] = ImageFont.truetype(str(fonts / name), SIZE)

    italic = faces["cmunti.ttf"]
    roman = faces["cmunrm.ttf"]

    cmap = cmap_of((fonts / "cmunti.ttf").read_bytes())
    for code in (0x03B8, 0x03D1):
        print(f"cmunti maps U+{code:04X}: {code in cmap} ({cmap.get(code)})")

    print("\nitalic ink past the advance, thousandths of an em (positive means it overhangs):")
    for ch in "zcpuvatkxh":
        length = italic.getlength(ch)
        right = italic.getbbox(ch)[2]
        print(f"  {ch}: advance {length:.0f}, ink right {right}, overhang {right - length:+.0f}")

    sheet = Image.new("L", (900, 300), 18)
    draw = ImageDraw.Draw(sheet)
    small_i = ImageFont.truetype(str(fonts / "cmunti.ttf"), 200)
    small_r = ImageFont.truetype(str(fonts / "cmunrm.ttf"), 200)
    draw.text((40, 20), "θ", font=small_i, fill=235)
    draw.text((340, 20), "θ", font=small_r, fill=235)
    if 0x03D1 in cmap:
        draw.text((640, 20), "ϑ", font=small_i, fill=235)
    sheet.save(out)


if __name__ == "__main__":
    main()
