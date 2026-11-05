"""complex-stage03-review-fixes01: root-accepted F3-F6 of complex-stage03-cross-review01.

    python3 -I -B checks/review_fixes.py make
    python3 -I -B checks/review_fixes.py check <fresh-scratch-dir>

Run both from the repo root. `make` writes patches/ once, from the exact edits below applied to
the stage03 bytes the receipt hashes. `check` copies those bytes into the scratch folder, applies
the patches there and checks what they change. It writes only into the scratch folder. Nothing is
compiled, rendered, synthesized or heard. The only tool run on Rust is `rustfmt --check`, and only
on scratch copies. The candidate tree is never written to: the copies are proven byte-equal to it.
"""
import difflib
import hashlib
import json
import math
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path.cwd()
HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
CL = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
ST = f"{CL}/combined-repair-stage03"
CRATE = "explainer_corpus/complex_analysis_visual"
CR = f"{ST}/{CRATE}"
RECEIPT = f"{CL}/combined-repair-stage03-integration01.json"
RECEIPT_SHA = "3f652c2d8bc510fbbd62e027b9db57943d8d5a1c3e297c0fecdfb87de6dec5e2"
OLIVER11 = f"{P}/complex11-counting-teaching02/holds/narration-direction-map-oliver11.json"
DELTA = f"{P}/complex-stage03-cross-review01/proposed/hold-rebind-stage03.json"
KIT = "crates/explainer_kit/src/kit.rs"

# the stage03 bytes the patches are made against (receipt `files`, and still on disk)
SOURCES = {
    "script.tsv": "1e24aba8b520f2e61e8b2bc392a218b664cf5556ad788ceca17fbc45bb92efab",
    "docs/storyboard.md": "57508d96103f0a4a86ed2265a51bb65d28fc68d8e6c96b3bd4438d2d97c83b5f",
    "src/chapters/counting.rs": "c1df16ea04d4283d490598ccabe039cdc846be96458668d63544aef9ac81d36a",
    "src/chapters/draw.rs": "865599e94331038782be128189fc1acb642e1d00b2f7fe32bd9fe68c958951b9",
}
# stage03 files the checks read but no patch touches: every other source file, and the plan
CHAPTERS = sorted(p.stem for p in (REPO / CR / "src/chapters").glob("*.rs"))
READ_ONLY = [f"src/chapters/{n}.rs" for n in CHAPTERS if n not in ("counting", "draw")]
READ_ONLY += ["src/main.rs", "src/facts.rs", "src/model.rs", "src/score.rs", "src/plate_layout.rs", "narration_units.json"]
# the chapter files that are compiled: `mod name;` in chapters/mod.rs (the rest are old, undeclared files)
COMPILED = re.findall(r"^(?:pub )?mod (\w+);$", (REPO / CR / "src/chapters/mod.rs").read_text(encoding="utf-8"), flags=re.M)

OLD_15_2_CLAUSE = "connected region, though not always everywhere, and along"
NEW_15_2_CLAUSE = "connected region, though there may be none, and along"
NEW_15_2 = (
    "Everything since came from one demand. The complex derivative must be the same from every direction. "
    "On an open region, that makes every small piece of the map a stretch and a turn. "
    "It lets boundary values decide the inside. "
    "It lets a loop count zeros and poles, and it turns a real integral into a sum of residues. "
    "It allows at most one extension to a given connected region, though there may be none, "
    "and along different paths a continuation can branch."
)

F4_TEST = """    /// `Turning` builds with `portrait`, which colours at the plate's samples per cell, and
    /// repaints with `portrait_colors`, which takes one. They agree only at one sample, so the
    /// takeover and the turn back to 11_6's colours are exact only while `PLATE` keeps one.
    #[test]
    fn the_turning_field_repaints_as_the_plate_paints() {
        assert_eq!(super::super::draw::PLATE.samples, 1);
    }"""
F4_ANCHOR = "    }\n\n    /// 11_7's acted picture follows its words"

F5_OLD = """        // the widest picture the chapters draw: 6.4 × 6.4 scene units
        let p = Plane::new(v(0.0, 0.0), 6.4, (-2.0, 2.0), (-2.0, 2.0));
        let (cols, rows) = picture_cells(&p, &PLATE);
        assert!(rows <= 800 && cols <= 800, "{cols} × {rows}");
        assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT);
    }"""
F5_NEW = """        // the widest pictures the chapters paint (05's sin and Γ, 8.4 wide) and the tallest
        // (04's plane and 05's exp, 6.4 high)
        for p in [
            super::super::gallery::trig_plane(),
            super::super::gallery::gamma_plane(),
            super::super::reading_color::plane(),
            super::super::gallery::exp_plane(),
        ] {
            let (cols, rows) = picture_cells(&p, &PLATE);
            assert!(rows <= 800 && cols <= 960, "{cols} × {rows}");
            assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT);
        }
    }"""

F6 = [
    ("04_2", 'the legend gains a size scale "band: size × 1.69" (`color.band_ratio`)',
     'the legend gains a size scale "next band: size × 1.69" (`color.band_ratio`)'),
    ("04_6", 'violet card "bump: where are the poles? how many turns?"',
     'violet card "where are the bump\'s poles?" / "how many turns at each?"'),
    ("05_7", 'dashed line Re s = 1 labelled "series converges to the right"',
     'dashed line Re s = 1 labelled "series converges: Re s > 1"'),
]

# patch name -> [(file, old, new)], each old found exactly once in the file as the patch meets it
PATCHES = {
    "F3-15_2-clause.patch": [
        ("script.tsv", OLD_15_2_CLAUSE, NEW_15_2_CLAUSE),
        ("docs/storyboard.md", OLD_15_2_CLAUSE, NEW_15_2_CLAUSE),
    ],
    "F4-turning-one-sample-guard.patch": [
        ("src/chapters/counting.rs", F4_ANCHOR,
         "    }\n\n" + F4_TEST + "\n\n    /// 11_7's acted picture follows its words"),
    ],
    "F5-budget-painted-planes.patch": [("src/chapters/draw.rs", F5_OLD, F5_NEW)],
    "F6-storyboard-quotes.patch": [("docs/storyboard.md", old, new) for _, old, new in F6],
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def stage_text(rel):
    return (REPO / CR / rel).read_text(encoding="utf-8")


def edit(texts, name):
    """`texts` (file -> text) with patch `name`'s edits applied."""
    out = dict(texts)
    for rel, old, new in PATCHES[name]:
        assert out[rel].count(old) == 1, f"{name}: {rel}: anchor found {out[rel].count(old)} times"
        out[rel] = out[rel].replace(old, new)
    return out


def unified(name, before, after):
    """The patch text for `name`: one unified diff per file it changes, paths from the stage root."""
    parts = []
    for rel in dict.fromkeys(rel for rel, _, _ in PATCHES[name]):
        assert before[rel].endswith("\n") and after[rel].endswith("\n")
        path = f"{CRATE}/{rel}"
        parts.extend(difflib.unified_diff(before[rel].splitlines(keepends=True), after[rel].splitlines(keepends=True),
                                          fromfile=f"a/{path}", tofile=f"b/{path}", n=3))
    return "".join(parts)


def make():
    target = OUT / "patches"
    assert not target.exists(), "patches/ already written"
    texts = {rel: stage_text(rel) for rel in SOURCES}
    for rel, want in SOURCES.items():
        assert sha(texts[rel].encode("utf-8")) == want, rel
    target.mkdir()
    for name in PATCHES:
        (target / name).write_text(unified(name, texts, edit(texts, name)), encoding="utf-8")
        print("wrote", f"patches/{name}")


# ------------------------------------------------------------------------------------------ check
RESULTS = []


def check(label, ok, detail=""):
    RESULTS.append((label, bool(ok)))
    print(("PASS " if ok else "FAIL ") + label + (f"  [{detail}]" if detail else ""))


def film_part(text):
    """The text before the test module (all of it, in a file without one)."""
    return text.partition("#[cfg(test)]\nmod tests")[0]


def sentences(text):
    # draw.rs `sentence_offsets`: a sentence ends after . ? or ! followed by a space
    return re.split(r"(?<=[.?!]) ", text)


def script_rows(text):
    return dict(line.split("\t", 1) for line in text.splitlines() if "\t" in line and not line.startswith("id\t"))


def board_cells(line):
    cells = re.split(r"(?<!\\)\|", line)
    return [c.strip() for c in cells[1:-1]]


def board_row(text, cue):
    rows = [line for line in text.splitlines() if line.startswith(f"| {cue} |")]
    assert len(rows) == 1, cue
    return board_cells(rows[0])


def plane_size(src, fn):
    """(width, height) of the single `Plane::new` literal in `fn fn()`'s body (square units)."""
    head = src.index(f"pub fn {fn}() -> Plane {{")
    body = src[head:src.index("\n}\n", head)]
    num = r"(-?\d+(?:\.\d+)?)"
    found = re.findall(rf"Plane::new\(\s*v\([^)]*\),\s*{num},\s*\({num}, {num}\),\s*\({num}, {num}\),?\s*\)", body)
    assert len(found) == 1, fn
    w, x0, x1, y0, y1 = map(float, found[0])
    return w, w * (y1 - y0) / (x1 - x0)


def cells(w, h, cell):
    # draw.rs `picture_cells`: (width / col).round().max(8), (height / row).round().max(8)
    for q in (w / cell, h / cell):
        assert abs(q - math.floor(q) - 0.5) > 1e-6, "a half: Rust and Python round differently"
    return max(round(w / cell), 8), max(round(h / cell), 8)


def run_check(scratch):
    assert not scratch.exists(), "the scratch folder must be fresh"
    scratch.mkdir(parents=True)

    print("== A. The bytes the patches are made against")
    check("receipt is the one reviewed", sha((REPO / RECEIPT).read_bytes()) == RECEIPT_SHA)
    receipt = json.loads((REPO / RECEIPT).read_text(encoding="utf-8"))
    hashed = {row["path"]: row["sha256"] for row in receipt["files"]}
    for rel, want in SOURCES.items():
        now = sha((REPO / CR / rel).read_bytes())
        check(f"{rel}: receipt and disk are {want[:8]}", hashed.get(f"{CRATE}/{rel}") == want and now == want)
    for rel in READ_ONLY:
        now = sha((REPO / CR / rel).read_bytes())
        check(f"{rel} (read only): disk equals the receipt", hashed.get(f"{CRATE}/{rel}") == now)
    before = {rel: stage_text(rel) for rel in SOURCES}

    print("\n== B. The patches are the edits, byte for byte")
    for name in PATCHES:
        made = unified(name, before, edit(before, name))
        have = (OUT / "patches" / name).read_text(encoding="utf-8")
        check(f"patches/{name} regenerates byte-equal", made == have)
        heads = re.findall(r"^(?:---|\+\+\+) (\S+)", have, flags=re.M)
        check(f"patches/{name} names only relative stage paths",
              all(h.startswith(("a/" + CRATE + "/", "b/" + CRATE + "/")) for h in heads) and "\r" not in have)
    whole = dict(before)
    for name in PATCHES:
        whole = edit(whole, name)
    after = whole

    print("\n== C. Each patch applies alone, and all apply in either order, exactly (GNU patch, scratch copies)")
    orders = {f"alone-{n.split('-')[0]}": [n] for n in PATCHES}
    orders["all-forward"] = list(PATCHES)
    orders["all-reverse"] = list(reversed(PATCHES))
    for tree_name, names in orders.items():
        tree = scratch / tree_name
        for rel in SOURCES:
            (tree / CRATE / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / CR / rel, tree / CRATE / rel)
        clean = True
        for name in names:
            r = subprocess.run(["patch", "-p1", "-F0", "--no-backup-if-mismatch", "-d", str(tree), "-i",
                                str(OUT / "patches" / name)], capture_output=True, text=True)
            said = (r.stdout + r.stderr).lower()
            clean &= r.returncode == 0 and not any(w in said for w in ("offset", "fuzz", "fail", "reject"))
        want = dict(before)
        for name in names:
            want = edit(want, name)
        same = all((tree / CRATE / rel).read_text(encoding="utf-8") == want[rel] for rel in SOURCES)
        strays = sorted(p.name for p in tree.rglob("*") if p.suffix in (".orig", ".rej"))
        check(f"{tree_name}: applies with no offset or fuzz, gives the edits, leaves no .orig/.rej",
              clean and same and not strays, ", ".join(strays))
    for rel in SOURCES:
        print(f"     {rel}: {sha(before[rel].encode())[:8]} -> {sha(after[rel].encode())}")

    print("\n== D. F3: 15_2's last clause")
    a_rows, b_rows = script_rows(before["script.tsv"]), script_rows(after["script.tsv"])
    changed = [k for k in a_rows if a_rows[k] != b_rows.get(k)]
    check("script: only 15_2 changes, and nothing is added or removed", changed == ["15_2"] and a_rows.keys() == b_rows.keys())
    check("script: 15_2 is the proposed sentence", b_rows["15_2"] == NEW_15_2)
    sa, sb = sentences(a_rows["15_2"]), sentences(b_rows["15_2"])
    check("15_2 keeps six sentences; sentences 0-4 untouched", len(sa) == len(sb) == 6 and sa[:5] == sb[:5])
    check("15_2 sentence 5 differs only in the clause", sa[5].replace("though not always everywhere", "though there may be none") == sb[5])
    check("15_2 has no digit", not re.search(r"\d", b_rows["15_2"]))
    closing = stage_text("src/chapters/closing.rs")
    cue = closing[closing.index('say(&kit, stage, "15_2")'):closing.index('say(&kit, stage, "15_3")')]
    waits = sorted({int(k) for k in re.findall(r"wait_sentence\(stage, (\d+)\)", cue)})
    phrases = re.findall(r'line\.(?:wait_for|at|until)\(stage, "([^"]+)"', cue)
    check("closing.rs 15_2 hooks: sentences 1-5 and one phrase", waits == [1, 2, 3, 4, 5] and phrases == ["and it turns a real integral"],
          f"sentences {waits}, phrases {phrases}")
    check("every 15_2 hook still lands: each phrase once, in the same sentence; last sentence index exists",
          all(b_rows["15_2"].count(ph) == 1 and [i for i, s in enumerate(sb) if ph in s] == [i for i, s in enumerate(sa) if ph in s]
              for ph in phrases) and max(waits) < len(sb))
    for side, texts in (("before", before), ("after", after)):
        check(f"storyboard 15_2 words equal the script ({side}; main.rs check_video compares them)",
              board_row(texts["docs/storyboard.md"], "15_2")[1] == script_rows(texts["script.tsv"])["15_2"])
    plan = stage_text("narration_units.json")
    check("the stage plan holds the old 15_2 clause once and not the new one (the binder updates it)",
          plan.count("though not always everywhere") == 1 and "though there may be none" not in plan)
    print(f"     script.tsv sha256 after F3: {sha(edit(before, 'F3-15_2-clause.patch')['script.tsv'].encode())}")

    print("\n== E. F4: the turning field's one-sample guard")
    check("counting.rs film code is byte-equal", film_part(before["src/chapters/counting.rs"]) == film_part(after["src/chapters/counting.rs"]))
    tests = after["src/chapters/counting.rs"].partition("#[cfg(test)]\nmod tests")[2]
    check("the guard is one test, in the test module", tests.count("fn the_turning_field_repaints_as_the_plate_paints()") == 1
          and after["src/chapters/counting.rs"].count("the_turning_field_repaints_as_the_plate_paints") == 1)
    check("counting tests that count source text still count the same: Some(Func::Quintic) = 3",
          after["src/chapters/counting.rs"].count("Some(Func::Quintic)") == before["src/chapters/counting.rs"].count("Some(Func::Quintic)") == 3)
    draw = before["src/chapters/draw.rs"]
    check("PlateStyle.samples is pub u32 and PLATE is a pub const", "    pub samples: u32," in draw and "pub const PLATE: PlateStyle = PLATE_CELLS4;" in draw)
    styles = {}
    for m in re.finditer(r"pub const (PLATE_\w+): PlateStyle = PlateStyle \{(.*?)\n\};", draw, flags=re.S):
        base = re.search(r"\.\.(PLATE_\w+)", m.group(2))
        own = re.search(r"samples: (\d+)", m.group(2))
        styles[m.group(1)] = (base and base.group(1), own and int(own.group(1)))

    def samples(name):
        base, own = styles[name]
        return own if own is not None else samples(base)

    chain = ["PLATE_CELLS4"]
    while styles[chain[-1]][1] is None:
        chain.append(styles[chain[-1]][0])
    check("PLATE's samples today: 1 (so the guard holds)", samples("PLATE_CELLS4") == 1, " <- ".join(chain))
    for at, text in (("draw.rs `if samples <= 1`", "    if samples <= 1 {\n        return portrait_colors(f, p"),
                     ("counting.rs builds with portrait", "let strips = portrait(stage, Func::Quintic, &inp, 1.0)?;"),
                     ("counting.rs repaints with portrait_colors", "let colors = portrait_colors(")):
        src = draw if at.startswith("draw") else before["src/chapters/counting.rs"]
        check(f"what the guard is about is still there: {at}", text in src)

    print("\n== F. F5: the budget test reads the planes the film paints")
    check("draw.rs film code is byte-equal", film_part(before["src/chapters/draw.rs"]) == film_part(after["src/chapters/draw.rs"]))
    gallery, reading = stage_text("src/chapters/gallery.rs"), stage_text("src/chapters/reading_color.rs")
    mod_rs = stage_text("src/chapters/mod.rs")
    check("gallery and reading_color are modules of chapters", "\nmod gallery;" in mod_rs and "\nmod reading_color;" in mod_rs)
    check("the test's four planes exist as pub fns", all(f"pub fn {fn}() -> Plane {{" in src for src, fn in
          ((gallery, "trig_plane"), (gallery, "gamma_plane"), (gallery, "exp_plane"), (reading, "plane"))))
    cell = float(re.search(r"pub const CELL: f64 = ([\d.]+);", draw).group(1))
    check("PLATE_CELLS4 cells are CELL / 4 both ways", "    row: CELL / 4.0,\n    col: CELL / 4.0,\n    ..PLATE_BLEND" in draw)
    q = cell / 4
    limit = int(re.search(r"pub const GROUP_TRANSFORM_LIMIT: usize = ([\d_]+);", (REPO / KIT).read_text(encoding="utf-8")).group(1).replace("_", ""))
    # every picture the film paints (Painter::build and portrait in film code), and its plane
    src = {n: stage_text(f"src/chapters/{n}.rs") for n in COMPILED}
    check("chapters/mod.rs declares 17 modules, draw and thumbnail among them", len(COMPILED) == 17 and {"draw", "thumbnail"} <= set(COMPILED))
    for n in ("counting", "draw"):
        assert "#[cfg(test)]\nmod tests" in before[f"src/chapters/{n}.rs"]
    painted = {
        ("boundary", "p"): plane_size(src["boundary"], "picture"),
        ("continuing", "bp"): plane_size(src["continuing"], "bump_plane"),
        ("continuing", "lp"): plane_size(src["continuing"], "lacunary_plane"),
        ("continuing", "gp"): plane_size(src["continuing"], "gamma_plane"),
        ("gallery", "sp"): plane_size(gallery, "square_plane"),
        ("gallery", "ep"): plane_size(gallery, "exp_plane"),
        ("gallery", "tp"): plane_size(gallery, "trig_plane"),
        ("gallery", "gp"): plane_size(gallery, "gamma_plane"),
        ("gallery", "zp"): plane_size(gallery, "zeta_plane"),
        ("one_point", "p"): plane_size(src["one_point"], "painted"),
        ("reading_color", "p"): plane_size(reading, "plane"),
        ("residues", "bp"): plane_size(src["residues"], "bump_plane"),
        ("rigidity", "sp"): plane_size(src["rigidity"], "sin_recip_plane"),
        ("rigidity", "sn"): plane_size(src["rigidity"], "sine_plane"),
    }
    special = [  # planes not built by one literal Plane::new, each read from its own words
        (("gallery", "st"), (2.0, 5.9), gallery, "            width: 2.0,\n            height: 5.9,"),
        (("gallery", "p"), (4.2, 2.6), gallery, "pub const PANEL: (f64, f64) = (4.2, 2.6);"),
        (("gallery", "zp9"), (5.0, 5.0), gallery, "        v(2.6, -0.3),\n        5.0,\n        (c.re - 0.65, c.re + 0.65),\n        (c.im - 0.65, c.im + 0.65),"),
        (("residues", "sp"), (2.8, 2.8), src["residues"], "[-4.9, -1.7, 1.5].map(|x| Plane::new(v(x, -1.0), 2.8, (-1"),
        (("the_bump", "small"), (1.5, 1.5), src["the_bump"], "let small = Plane::new(v(x, 2.0), 1.5, (-1.2, 1.2), (-1.2, 1.2));"),
        (("counting", "inp"), (5.6, 5.6), src["counting"], "        Plane::new(v(-3.45, -0.35), 5.6, (-2.0, 2.0), (-2.0, 2.0)),"),
    ]
    for key, size, text, words in special:
        check(f"{key[0]}.rs `{key[1]}` is {size[0]} x {size[1]}", words in text)
        painted[key] = size
    calls, sites = set(), 0
    for name, text in src.items():
        for line in film_part(text).splitlines():
            if "Painter::build(stage" in line or re.search(r"[^:\w]portrait\(stage, ", line):
                sites += 1
                m = re.search(r"Painter::build\(stage, [^,]+, (\w+),|[^:\w]portrait\(stage, [^,]+, &?(\w+),", line)
                calls.add((name, m and (m.group(1) or m.group(2))))
    check(f"every painted call site in the {len(COMPILED)} compiled chapter files is in the table, and every table row is painted",
          calls == set(painted), f"{sites} call sites; {sorted(calls ^ set(painted))}")
    sizes = {k: cells(w, h, q) for k, (w, h) in painted.items()}
    widest = max(c for c, _ in sizes.values())
    tallest = max(r for _, r in sizes.values())
    at_w = sorted(k for k, (c, _) in sizes.items() if c == widest)
    at_h = sorted(k for k, (_, r) in sizes.items() if r == tallest)
    check("widest painted picture: 960 columns, 05's sin and Gamma only", widest == 960 and at_w == [("gallery", "gp"), ("gallery", "tp")], str(at_w))
    check("tallest painted picture: 731 rows, 04's plane and 05's exp only", tallest == 731 and at_h == [("gallery", "ep"), ("reading_color", "p")], str(at_h))
    test_planes = [plane_size(gallery, "trig_plane"), plane_size(gallery, "gamma_plane"), plane_size(reading, "plane"), plane_size(gallery, "exp_plane")]
    got = [cells(w, h, q) for w, h in test_planes]
    check("the new test's asserts hold for its four planes (mirror)", all(r <= 800 and c <= 960 and r < limit for c, r in got), str(got))
    check("the old test's bound would refuse the widest painted picture (why F5)", not all(c <= 800 for c, _ in got))

    print("\n== G. F6: the storyboard quotes what the film draws")
    facts = stage_text("src/facts.rs")
    ratio = f"{math.exp(math.tau / 12):.2f}"
    band = re.search(r'pub const BAND: &str = "([^"]+)";', reading).group(1)
    ask = re.search(r'pub const ASK: \[&str; 2\] = \["([^"]+)", "([^"]+)"\];', reading).groups()
    conv = re.search(r'pub const CONVERGES: &str = "([^"]+)";', gallery).group(1)
    check("the band ratio the film shows is 1.69", ratio == "1.69" and 'assert_eq!(get("color.band_ratio"), "1.69");' in facts
          and '&format!("{BAND}{}", fact("color.band_ratio")?)' in reading)
    shown = {"04_2": [band + ratio], "04_6": list(ask), "05_7": [conv]}
    for cue, old, new in F6:
        b_cells, a_cells = board_row(after["docs/storyboard.md"], cue), board_row(before["docs/storyboard.md"], cue)
        quoted = re.findall(r'"([^"]+)"', new)
        check(f"storyboard {cue}: quotes the film's strings", quoted == shown[cue] and old not in after["docs/storyboard.md"], str(quoted))
        check(f"storyboard {cue}: only the picture cell changes; five cells", len(a_cells) == len(b_cells) == 5
              and [a_cells[i] == b_cells[i] for i in range(5)] == [True, True, False, True, True])
    a_lines, b_lines = before["docs/storyboard.md"].splitlines(), after["docs/storyboard.md"].splitlines()
    moved = [i + 1 for i, (x, y) in enumerate(zip(a_lines, b_lines)) if x != y]
    check("storyboard: lines 134, 138, 151 (F6) and 284 (F3) change, no line added", moved == [134, 138, 151, 284] and len(a_lines) == len(b_lines), str(moved))

    print("\n== H. What the hold rows need after the fixes (map = teaching02 oliver11 + cross-review01 delta)")
    base = json.loads((REPO / OLIVER11).read_text(encoding="utf-8"))
    delta = json.loads((REPO / DELTA).read_text(encoding="utf-8"))
    rows = {s: [dict(r) for r in base[s]] for s in ("visual_holds", "helpers")}
    for d in delta["rows"]:
        r = rows[d["section"]][d["index"]]
        assert (r["path"], r["sha256"], r["line"], r["marker"]) == (d["path"], d["old"]["sha256"], d["old"]["line"], d["old"]["marker"])
        r.update(d["new"])
    stage_ok, touched = True, []
    for section, items in rows.items():
        for r in items:
            rel = r["path"].removeprefix(CRATE + "/")
            now = (REPO / CR / rel).read_bytes()
            stage_ok &= sha(now) == r["sha256"] and now.decode("utf-8").splitlines()[r["line"] - 1].strip() == r["marker"].strip()
            if rel in SOURCES:
                lines = after[rel].splitlines()
                touched.append((section, r.get("cue", "helper"), rel, r["line"], lines[r["line"] - 1].strip() == r["marker"].strip()))
    check("all 28 rows bind stage03 as it stands", stage_ok and sum(map(len, rows.values())) == 28)
    check("rows on patched files keep their line and marker; only the file sha changes", all(t[4] for t in touched),
          "; ".join(f"{s}:{c}:{rel.split('/')[-1]}:{line}" for s, c, rel, line, _ in touched))
    for rel in ("src/chapters/counting.rs", "src/chapters/draw.rs"):
        print(f"     {rel}: rows' sha256 {SOURCES[rel][:8]} -> {sha(after[rel].encode())}")
    check("the map's scripts field binds stage03's script today", base["scripts"]["complex_analysis_visual"] == SOURCES["script.tsv"])
    print(f"     scripts.complex_analysis_visual after F3: {sha(after['script.tsv'].encode())}")

    print("\n== I. rustfmt --check (scratch copies; the repo's rustfmt.toml and toolchain)")
    fmt = scratch / "rustfmt"
    for side, texts in (("before", before), ("after", after)):
        for rel in ("src/chapters/counting.rs", "src/chapters/draw.rs"):
            f = fmt / side / rel.split("/")[-1]
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(texts[rel], encoding="utf-8")
    bad = fmt / "control" / "draw.rs"
    bad.parent.mkdir(parents=True)
    assert after["src/chapters/draw.rs"].count("assert!(rows <= 800 && cols <= 960") == 1
    bad.write_text(after["src/chapters/draw.rs"].replace("assert!(rows <= 800 && cols <= 960", "assert!( rows <= 800 && cols <= 960"),
                   encoding="utf-8")
    for f in sorted(fmt.rglob("*.rs")):
        r = subprocess.run(["rustfmt", "--edition", "2024", "--config-path", str(REPO / "rustfmt.toml"), "--check", str(f)],
                           cwd=REPO, capture_output=True, text=True)
        role = f.parent.name
        want = r.returncode != 0 if role == "control" else r.returncode == 0
        check(f"rustfmt --check {role}/{f.name}: " + ("refused (control)" if role == "control" else "clean"), want)

    print("\nFAILURES: " + (", ".join(label for label, ok in RESULTS if not ok) or "none"))
    return all(ok for _, ok in RESULTS)


if __name__ == "__main__":
    if sys.argv[1:2] == ["make"]:
        make()
    elif sys.argv[1:2] == ["check"] and len(sys.argv) == 3:
        sys.exit(0 if run_check(pathlib.Path(sys.argv[2])) else 1)
    else:
        sys.exit(__doc__)
