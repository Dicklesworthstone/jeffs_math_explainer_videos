"""Turn a chapter and a scratch-time range into first/last sampled frame and sheet, from coverage/sheets.tsv.

python3 -I -B locate.py     (from the repo root; the ranges are listed below)

Reads the manifest only. Prints one row per range.
"""
from pathlib import Path

F = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01")
RANGES = [
    ("02-freeze", "02_turning_numbers", 57.0, 65.5),
    ("02-G", "02_turning_numbers", 84.0, 96.3),
    ("03-empty", "03_two_planes", 15.5, 22.0),
    ("03-flip", "03_two_planes", 78.0, 82.5),
    ("04-still z2", "04_reading_color", 41.5, 75.6),
    ("04-still 1/z", "04_reading_color", 79.6, 84.7),
    ("04-A2", "04_reading_color", 26.0, 118.2),
    ("04 band", "04_reading_color", 67.7, 76.0),
    ("04 title z2", "04_reading_color", 39.3, 40.0),
    ("04 title 1/z", "04_reading_color", 76.2, 77.5),
    ("04 title bump", "04_reading_color", 85.233, 86.0),
    ("04 card order 2", "04_reading_color", 56.8, 62.5),
    ("05 pictures", "05_gallery", 0.8, 164.95),
    ("05 tan flats", "05_gallery", 67.5, 80.4),
    ("05 zoom", "05_gallery", 155.5, 164.5),
    ("05 pole struck", "05_gallery", 126.5, 146.6),
    ("05 tile late", "05_gallery", 153.2, 157.5),
    ("05 crowd", "05_gallery", 135.0, 146.5),
    ("05 series still", "05_gallery", 107.0, 113.0),
    ("05 bracket", "05_gallery", 40.5, 45.72),
    ("05 RH early", "05_gallery", 134.5, 138.75),
    ("05 pole card", "05_gallery", 23.0, 26.3),
    ("05 sign flash", "05_gallery", 25.5, 25.5),
    ("05 laps", "05_gallery", 8.0, 13.5),
    ("05 caption stack", "05_gallery", 140.5, 146.5),
    ("05 arrow head", "05_gallery", 62.5, 66.8),
    ("05 tan still", "05_gallery", 76.0, 80.4),
    ("05 mosaic names", "05_gallery", 150.0, 154.0),
    ("05 ghost", "05_gallery", 106.0, 106.6),
]


def main():
    header, *rows = (F / "coverage/sheets.tsv").read_text(encoding="utf-8").splitlines()
    names = header.split("\t")
    print("columns:", names)
    print("first row:", rows[0])
    frames = {}
    for line in rows:
        cells = dict(zip(names, line.split("\t")))
        chapter = cells["chapter"]
        sheet = int(cells["sheet"].split("-")[1].split(".")[0])
        pairs = zip(cells["frames"].split(","), cells["times"].split(","))
        for frame, time in pairs:
            frames.setdefault(chapter, []).append((int(frame), float(time), sheet))
    for name, chapter, start, end in RANGES:
        inside = [row for row in frames[chapter] if start <= row[1] <= end]
        first, last = inside[0], inside[-1]
        print(f"{name}\t{chapter}\t{start}-{end} s\tframes {first[0]}-{last[0]} ({first[1]}-{last[1]} s)\tsheets {first[2]}-{last[2]}")


if __name__ == "__main__":
    main()
