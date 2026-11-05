"""A look at C3 on preview02's old 720p frames: the digit moved by hand onto zero, with its plate.

NOT A RENDER. The disk, line and marks are the old frames'. Nothing here came from the staged code,
and these frames are on the scratch voice. It is only there so root can judge the idea before a build.
The top row is the frames as they are; the bottom row has the old digit painted out, a ground disk at
the origin of the plate's size, and a clean copy of the digit from frame 552 pasted on it.

Needs PIL, so run without -I: python3 -B c3_synthetic.py FRAMES_DIR OUT_PNG
"""

import sys

from PIL import Image, ImageDraw

frames_dir, out_path = sys.argv[1:3]
FRAMES = [544, 546, 547, 548, 550]
ORIGIN, DIGIT = (410.5, 310.5), (394.75, 329.4)  # 720p: the output window's 0, and the old name
PX = 90.0  # pixels per scene unit at 720p
PLATE_R = 0.105 * PX
BOX = (230, 130, 530, 430)
SCALE = 1.2
GROUND = (11, 15, 20)


def load(n):
    return Image.open(f"{frames_dir}/frame-{n:04}.png").convert("RGB")


clean = load(552)
half_w, half_h = 8, 10
digit_box = (int(DIGIT[0] - half_w), int(DIGIT[1] - half_h), int(DIGIT[0] + half_w), int(DIGIT[1] + half_h))
digit = clean.crop(digit_box)
ground = clean.getpixel((int(DIGIT[0]), int(DIGIT[1] - half_h - 6)))


def inked(px):
    r, g, b = px
    return min(r, g, b) > 70 and max(r, g, b) - min(r, g, b) < 45


def off_ground(px):
    return max(abs(px[c] - ground[c]) for c in range(3)) > 8


def moved(frame):
    out = frame.copy()
    pixels, clean_px = out.load(), clean.load()
    # only where the clean digit has ink, one pixel wider: the line and rim elsewhere stay
    for x in range(digit_box[0], digit_box[2]):
        for y in range(digit_box[1], digit_box[3]):
            if any(off_ground(clean_px[x + i, y + j]) for i in (-1, 0, 1) for j in (-1, 0, 1)):
                r, g, b = pixels[x, y]
                if not g > r + 25:  # the teal rim keeps its pixels
                    pixels[x, y] = ground
    draw = ImageDraw.Draw(out)
    r = PLATE_R
    draw.ellipse((ORIGIN[0] - r, ORIGIN[1] - r, ORIGIN[0] + r, ORIGIN[1] + r), fill=ground)
    at = (int(ORIGIN[0] - half_w), int(ORIGIN[1] - half_h))
    source = digit.load()
    target = out.load()
    for i in range(digit.size[0]):
        for j in range(digit.size[1]):
            if inked(source[i, j]):
                target[at[0] + i, at[1] + j] = source[i, j]
    return out


cell = int((BOX[2] - BOX[0]) * SCALE)
sheet = Image.new("RGB", (cell * len(FRAMES), cell * 2 + 24), (0, 0, 0))
for k, n in enumerate(FRAMES):
    frame = load(n)
    for row, picture in enumerate((frame, moved(frame))):
        crop = picture.crop(BOX).resize((cell, cell), Image.LANCZOS)
        name = f"frame {n}" if row == 0 else f"frame {n}, digit moved by hand"
        ImageDraw.Draw(crop).text((6, 6), name, fill=(255, 235, 90))
        sheet.paste(crop, (k * cell, row * cell))
ImageDraw.Draw(sheet).text((8, cell * 2 + 6), "NOT A RENDER: old preview02 frames on the scratch voice, edited by hand. Breaks in the white line and specks at the old digit's place are scars of the edit. Judge the idea, not the pixels.", fill=(255, 150, 150))
sheet.save(out_path)
print("wrote the sheet:", sheet.size)
