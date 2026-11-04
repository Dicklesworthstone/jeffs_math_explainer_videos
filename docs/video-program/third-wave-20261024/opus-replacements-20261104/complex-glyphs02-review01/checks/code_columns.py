"""Do the comment marks of the code panel line up once indentation is restored?

python3 -B code_columns.py   (from the repo root; PIL needs site packages)

The probe drops the leading spaces that CodePanel::build turns into an x offset, so rows 731 and
732 (four columns of indent in the film) start at the page margin while row 733 (no indent) does
too. All three put "#" at film column 34, so on the page 733's mark must sit four columns right
of the other two, and one column is measured from rows 727 and 728 the way CodePanel measures it.
"""
from pathlib import Path

from PIL import Image

PAGE = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02/glyphs/pages/frame_000066.png")
ROWS = {731: (397, 439), 732: (469, 512), 733: (542, 584)}
ZEROS = {727: (135, 164), 728: (198, 227)}


def main():
    im = Image.open(PAGE).convert("L")
    px = im.load()
    bg = px[im.width - 2, im.height - 2]

    def inked(x, y0, y1):
        return any(abs(px[x, y] - bg) > 60 for y in range(y0, y1))

    right = {}
    for row, (y0, y1) in ZEROS.items():
        right[row] = max(x for x in range(im.width) if inked(x, y0, y1))
    column = (right[728] - right[727]) / 10
    print(f"right ink edge: row 727 x={right[727]}, row 728 x={right[728]}; one column = {column:.1f} px")
    marks = {}
    for row, (y0, y1) in ROWS.items():
        marks[row] = next(x for x in range(700, im.width) if inked(x, y0, y1))
        print(f"row {row}: first ink at or after x=700 is x={marks[row]}")
    print(f"row 733 minus row 732 = {marks[733] - marks[732]} px; four columns = {4 * column:.1f} px")


if __name__ == "__main__":
    main()
