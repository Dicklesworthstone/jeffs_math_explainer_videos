"""First pass: anchors, static spans, then crops into the scratchpad so I can look before keeping any."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import measure as M

M.OUT = Path(__file__).parent / "crops1"

print("lint anchors in chapter 11 samples (nearest frame, time, sheet):")
for t in (29.8, 30.0, 60.8, 142.8, 168.2, 176.2, 197.4, 206.2, 233.1):
    print(f"  {t}: {M.nearest('11', t)}")

for ch in ("11", "12"):
    print(f"static spans >= 5 s, chapter {ch}")
    for a, b in M.spans(ch, 5.0):
        print(f"  f{a} {M.TIMES[ch][a]:.3f} .. f{b} {M.TIMES[ch][b]:.3f}  = {M.TIMES[ch][b] - M.TIMES[ch][a]:.2f} s")

print("crops")
M.crop("11", 24, (140, 150, 560, 560), "11_f024_before.png", 2)
M.crop("11", 33, (140, 150, 560, 560), "11_f033_dashes.png", 2)
M.crop("11", 43, (140, 150, 560, 620), "11_f043_red.png", 2)
M.crop("11", 69, (660, 120, 1000, 560), "11_f069_readout.png", 2)
M.crop("11", 170, (140, 280, 560, 620), "11_f170_arrows.png", 2)
M.crop("11", 365, (140, 150, 560, 620), "11_f365_axis.png", 2)
M.crop("11", 375, (880, 380, 1200, 440), "11_f375_key.png", 4)
M.crop("11", 285, (140, 150, 560, 620), "11_f285_routes.png", 2)
M.crop("12", 41, (1090, 130, 1340, 380), "12_f041_pointer.png", 3)
M.crop("12", 62, (40, 500, 640, 640), "12_f062_lower_left.png", 2)
M.crop("12", 80, (720, 500, 1000, 640), "12_f080_flows.png", 3)
M.crop("12", 293, (200, 200, 540, 620), "12_f293_red.png", 2)
M.crop("12", 287, (200, 200, 540, 620), "12_f287_red.png", 2)
