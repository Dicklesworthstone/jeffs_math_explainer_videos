"""Second pass: contrast, clearances and better-aimed crops."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import measure as M

M.OUT = Path(__file__).parent / "crops2"

print("contrast")
M.contrast("11", 24, (322, 343, 375, 427), "faint before-F fill vs field")
M.contrast("11", 24, (322, 300, 375, 318), "before word, dim")
M.contrast("11", 33, (258, 343, 308, 427), "gold after-F fill vs field")
M.contrast("11", 524, (560, 130, 720, 200), "chapter 11 e^(i pi) region a")
M.contrast("11", 524, (540, 130, 760, 190), "chapter 11 e^(i pi) region b")
M.contrast("11", 375, (915, 390, 1072, 412), "where A puts the F, red 20 pt")
M.contrast("12", 96, (50, 590, 372, 610), "complex exponential forgets the laps, purple 20 pt")
M.contrast("12", 320, (800, 595, 1000, 615), "turning line tick labels, dim 16 pt")
M.contrast("12", 350, (72, 672, 1200, 696), "precisely line, chapter 12")
M.contrast("12", 350, (765, 668, 1100, 682), "convention line, dim 18 pt")
M.contrast("12", 300, (693, 226, 736, 240), "time word, dim")
M.contrast("12", 300, (980, 226, 1012, 240), "laps value, purple")

print("clearances, chapter 11")
M.bbox("11", 69, (675, 410, 695, 450), '"0" label f69')
M.bbox("11", 69, (696, 415, 760, 445), "zero line f69", 100)
M.bbox("11", 69, (690, 80, 1000, 108), "signed area word f69")
M.bbox("11", 69, (690, 110, 760, 150), "live value f69")
M.bbox("11", 137, (690, 80, 1000, 108), "det word f137")
M.bbox("11", 137, (690, 110, 760, 150), "live value f137")
M.bbox("11", 313, (830, 440, 1280, 560), "gap caption f313")
M.bbox("11", 375, (880, 386, 918, 416), "key F f375", 100)
M.bbox("11", 375, (925, 386, 1280, 416), "key word f375")
M.bbox("11", 386, (880, 386, 918, 416), "key F f386", 100)
M.bbox("11", 386, (925, 386, 1280, 416), "key word f386")
M.bbox("11", 432, (830, 340, 1280, 395), "equal stretches f432")
M.bbox("11", 450, (830, 420, 1280, 480), "2x2 lines f450")
M.bbox("11", 503, (830, 500, 1280, 640), "rule lines f503")
M.bbox("11", 230, (1050, 330, 1280, 360), "same end point f230")
M.bbox("11", 283, (1000, 130, 1280, 300), "X card f283")

print("clearances, chapter 12")
M.bbox("12", 62, (300, 505, 640, 530), "Log line f62")
M.bbox("12", 62, (300, 530, 400, 582), "minus-3-pi label f62")
M.bbox("12", 62, (40, 588, 400, 612), "forgets the laps sentence f62")
M.bbox("12", 62, (130, 570, 140, 588), "axis stub f62", 25)
M.bbox("12", 350, (40, 672, 1279, 696), "precisely line f350")

print("plate shades, chapter 11 f170: pixels next to the x½ label and open field")
a = M.load("11", 170)
for name, (x, y) in {"open field": (900, 650), "inside plate left": (340, 442), "inside plate right": (435, 440)}.items():
    print(f"  {name} ({x},{y}): {a[y, x].tolist()}")

print("crops")
M.crop("12", 41, (730, 110, 960, 350), "12_f041_dial.png", 3)
M.crop("12", 79, (720, 410, 1000, 590), "12_f079_flows.png", 3)
M.crop("12", 80, (720, 410, 1000, 590), "12_f080_flows.png", 3)
M.crop("12", 81, (720, 410, 1000, 590), "12_f081_flows.png", 3)
M.crop("11", 524, (540, 110, 800, 240), "11_f524_purple.png", 3)
