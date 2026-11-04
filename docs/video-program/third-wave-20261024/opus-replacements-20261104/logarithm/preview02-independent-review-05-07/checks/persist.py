"""Scratch only: for each defect, the selected frames whose box matches the reference frame's box.
Mean absolute difference per channel on native frame PNGs; ranges printed with sampled times."""
import json
import pathlib
from PIL import Image, ImageChops, ImageStat

B = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02")
CH = {"05": "05_complex_turns", "06": "06_lifting_loops", "07": "07_branch_cut"}
ROWS = [
    ("D1 lap_word", "05", 248, (222, 276, 420, 300)),
    ("D2 2i+zw", "05", 133, (345, 200, 392, 232)),
    ("D3 i on zw", "05", 46, (345, 300, 362, 325)),
    ("D4a closes", "05", 199, (752, 110, 1014, 140)),
    ("D4b dial", "05", 410, (762, 110, 1004, 140)),
    ("D5 no log", "05", 386, (925, 400, 1065, 428)),
    ("D6 length2", "05", 360, (717, 178, 881, 202)),
    ("D7 laps", "05", 248, (1050, 195, 1100, 218)),
    ("D8 aid", "05", 248, (673, 600, 1092, 622)),
    ("D9 0 label", "05", 248, (268, 364, 284, 382)),
    ("D10 -1 dot", "05", 300, (790, 357, 822, 389)),
    ("E10 theorem", "06", 400, (400, 260, 1000, 320)),
    ("E1a one lap pi", "06", 118, (593, 482, 681, 508)),
    ("E1b 2pi stub", "06", 118, (657, 377, 695, 399)),
    ("E7 arrow", "06", 276, (748, 490, 776, 600)),
    ("E5 gold dot", "06", 165, (371, 374, 395, 398)),
    ("E6 not inside", "06", 160, (200, 505, 420, 560)),
    ("F-blocked", "07", 30, (205, 305, 285, 335)),
    ("F-sides above", "07", 170, (118, 338, 210, 362)),
    ("F-sides below", "07", 170, (118, 409, 210, 433)),
    ("F-edge", "07", 200, (476, 565, 1120, 590)),
    ("F-zero ring strip1", "07", 137, (318, 371, 350, 403)),
    ("F-zero ring strip2", "07", 200, (318, 371, 350, 403)),
    ("F-leap tangent", "07", 219, (318, 150, 360, 620)),
    ("F-bp spiral", "07", 295, (355, 412, 480, 442)),
    ("F-bp circle", "07", 315, (355, 412, 480, 442)),
    ("S1 removed card", "07", 270, (770, 232, 1210, 262)),
]
cache = {}


def frames(ch):
    if ch not in cache:
        d = B / CH[ch]
        t = json.load(open(d / "sampling.json"))["times"]
        cache[ch] = (t, [Image.open(d / f"frame-{i:04d}.png").convert("RGB") for i in range(len(t))])
    return cache[ch]


for name, ch, ref, box in ROWS:
    t, im = frames(ch)
    r = im[ref].crop(box)
    hits = []
    for i, f in enumerate(im):
        d = sum(ImageStat.Stat(ImageChops.difference(f.crop(box), r)).mean) / 3
        if d < 5.0:
            hits.append(i)
    ranges = []
    for i in hits:
        if ranges and i == ranges[-1][1] + 1:
            ranges[-1][1] = i
        else:
            ranges.append([i, i])
    print(name, ch, "ref", ref, " ".join(f"{a}-{b}({t[a]:.1f}-{t[b]:.1f}s)" for a, b in ranges))


# Sparse boxes need a colour test instead of a difference test.
def ranges_of(hits, t):
    out = []
    for i in hits:
        if out and i == out[-1][1] + 1:
            out[-1][1] = i
        else:
            out.append([i, i])
    return " ".join(f"{a}-{b}({t[a]:.1f}-{t[b]:.1f}s)" for a, b in out)


t, im = frames("06")
gold = [i for i, f in enumerate(im) if (lambda p: p[0] > 200 and p[1] > 170 and p[2] < 140)(f.getpixel((383, 386)))]
word = [
    i
    for i, f in enumerate(im)
    if sum(1 for x in range(232, 348) for y in range(520, 542) if (lambda p: p[1] > 130 and p[2] > 110 and p[0] < 140)(f.getpixel((x, y)))) > 60
]
print("E5 gold z=1 dot, pixel (383,386) 06", ranges_of(gold, t))
print("E6 '0 is not inside', teal pixels in word band 06", ranges_of(word, t))
