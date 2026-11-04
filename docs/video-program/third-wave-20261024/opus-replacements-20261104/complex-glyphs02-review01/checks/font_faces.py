"""Which bundled face draws each character of the Text and Mono rows.

Run from the repo root with python3 -I -B. Reads the probe binary and the registry only.

It finds the fonts embedded in target/debug/complex_analysis_visual (the same detection as
the franken-manim skill's sfnt_scan.py) and reads their cmap tables, then walks the engine's
chain at pin a399d62, crates/fmn-text/src/font.rs, glyph_fallback: the selected family first,
then Computer Modern, then the Noto Sans Math symbol face, then IBM Plex Sans.
"""
import collections
import hashlib
import struct
import sys
from pathlib import Path

BINARY = Path("target/debug/complex_analysis_visual")
RUN = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02")
# Offsets and hashes printed by sfnt_scan.py for this binary; the hash is re-checked below.
FACES = {
    "CM Roman": (0x009BC160, 235654, "c7cdebc16fc4"),
    "CM Typewriter": (0x00AB805C, 214526, "5647bcfccd1a"),
    "IBM Plex Sans": (0x00AEC65C, 200500, "975dcda37d80"),
    "Noto Sans Math": (0x00B1D590, 72908, "634b9964bd8a"),
}


def cmap_of(font):
    num = struct.unpack_from(">H", font, 4)[0]
    tables = {}
    for i in range(num):
        tag, _, off, length = struct.unpack_from(">4sIII", font, 12 + 16 * i)
        tables[tag.decode("latin-1")] = (off, length)
    base = tables["cmap"][0]
    count = struct.unpack_from(">H", font, base + 2)[0]
    mapped = {}
    for i in range(count):
        pid, eid, off = struct.unpack_from(">HHI", font, base + 4 + 8 * i)
        at = base + off
        fmt = struct.unpack_from(">H", font, at)[0]
        if fmt == 4 and pid in (0, 3):
            segs = struct.unpack_from(">H", font, at + 6)[0] // 2
            end = at + 14
            start = end + 2 * segs + 2
            delta = start + 2 * segs
            ranges = delta + 2 * segs
            for s in range(segs):
                e = struct.unpack_from(">H", font, end + 2 * s)[0]
                b = struct.unpack_from(">H", font, start + 2 * s)[0]
                d = struct.unpack_from(">h", font, delta + 2 * s)[0]
                r = struct.unpack_from(">H", font, ranges + 2 * s)[0]
                for c in range(b, e + 1):
                    if c == 0xFFFF:
                        continue
                    if r == 0:
                        gid = (c + d) & 0xFFFF
                    else:
                        gid = struct.unpack_from(">H", font, ranges + 2 * s + r + 2 * (c - b))[0]
                        if gid:
                            gid = (gid + d) & 0xFFFF
                    if gid:
                        mapped[c] = gid
        elif fmt == 12:
            groups = struct.unpack_from(">I", font, at + 12)[0]
            for g in range(groups):
                b, e, first = struct.unpack_from(">III", font, at + 16 + 12 * g)
                for c in range(b, e + 1):
                    mapped[c] = first + c - b
    return mapped


def decode(literal):
    body = literal[1:-1]
    out, i = [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = body[i + 1]
        if n == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3 : end], 16)))
            i = end + 1
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


def main():
    data = BINARY.read_bytes()
    cmaps = {}
    for name, (off, size, want) in FACES.items():
        font = data[off : off + size]
        got = hashlib.sha256(font).hexdigest()[:12]
        print(f"{name}: {size} B, sha256 {got}, {'ok' if got == want else 'HASH MISMATCH ' + want}")
        cmaps[name] = cmap_of(font)
        print(f"  {len(cmaps[name])} code points mapped")

    def face(kind, ch):
        chain = ["CM Typewriter", "CM Roman", "Noto Sans Math", "IBM Plex Sans"] if kind == "Mono" else ["CM Roman", "Noto Sans Math", "IBM Plex Sans"]
        for name in chain:
            if ord(ch) in cmaps[name]:
                return name
        return None

    rows = []
    for number, line in enumerate((RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines(), 1):
        kind, chapters, literal = line.split("\t", 2)
        rows.append((number, kind, chapters, decode(literal)))

    table = collections.defaultdict(lambda: collections.Counter())
    examples = collections.defaultdict(list)
    for number, kind, chapters, text in rows:
        if kind == "Tex":
            continue
        for ch in sorted(set(text)):
            if ch.isspace():
                continue
            where = face(kind, ch)
            table[(kind, where)][ch] += 1
            if where != ("CM Typewriter" if kind == "Mono" else "CM Roman"):
                examples[(kind, ch, where)].append(number)

    print("\n# characters by the face that draws them")
    for (kind, where), chars in sorted(table.items(), key=lambda kv: (kv[0][0], str(kv[0][1]))):
        ascii_chars = "".join(sorted(c for c in chars if ord(c) < 128))
        other = " ".join(f"{c}(U+{ord(c):04X})" for c in sorted(c for c in chars if ord(c) >= 128))
        print(f"  {kind} from {where}: ascii [{ascii_chars}] other: {other}")

    print("\n# rows that use a character outside the row's own face")
    for (kind, ch, where), numbers in sorted(examples.items(), key=lambda kv: (kv[0][0], str(kv[0][2]), kv[0][1])):
        print(f"  {kind} {ch} U+{ord(ch):04X} from {where}: {len(numbers)} rows: {numbers[:60]}")

    fallback_rows = sorted({n for ns in examples.values() for n in ns})
    in_film = [r for r in rows if r[0] in fallback_rows and r[2] != "supplement"]
    print(f"\n# Text or Mono rows with at least one fallback glyph: {len(fallback_rows)}; captured from a chapter: {len(in_film)}")
    per_chapter = collections.Counter()
    for r in in_film:
        for c in r[2].split(","):
            if c != "supplement":
                per_chapter[c] += 1
    print("  " + " ".join(f"{c}:{n}" for c, n in sorted(per_chapter.items())))


if __name__ == "__main__":
    main()
