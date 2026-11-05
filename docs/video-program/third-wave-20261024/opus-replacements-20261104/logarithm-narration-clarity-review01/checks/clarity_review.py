"""Independent source review of WhiteSwan's logarithm-narration-clarity-repair01 against stage08. Run from the repo root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-narration-clarity-review01/checks/clarity_review.py FRESH_SCRATCH

Reads the repository and writes only under FRESH_SCRATCH (a folder that must not exist yet). Two copies of the stage08 crate
are made there: a control copy left as it is and a patched copy. Nothing is compiled, rendered, synthesized, measured or heard,
and no audio is decoded: the four Oliver takes are only hashed.

Sections:
  pins       every input is the version reviewed (exit 3 when one has moved; nothing else runs);
  baseline   04_4 and 08_5 in stage08: texts, the kit's sentences, every hook in each say-to-next-say span, root's anchors,
             holds, template marks and unit texts, the test that quotes "Halving the lost lap", and binder03's stage07 pins;
  replay     git apply on the patched copy, expected hashes, nothing else changes, docs/check_cues.py the same on both copies;
  meaning    one sentence per cue changes and the sentence count stays; gap-class inputs at all four boundaries unchanged;
             04_4 now says its card's words; 08_5's Rust test mirrored from the test's own source; both phrases in sentence 4;
             found(...) order; char-share estimates; old mark bytes in the new text; intervals; the mathematics in floats;
  oliver     owner refinement 03:22:06Z: logarithm keeps Oliver. Receipts carry no context, settings, 78 of 80 still bound,
             the four old takes preserved, and the repair bundle's voice premise as written;
  inventory  every file under the crate, bound-plan03 and the two request folders that holds an old hash or the old words,
             and whether the repair bundle names it.
Exit 0 when every check passes, 1 when one fails, 3 when a pin has moved, 2 for usage.
"""
import hashlib
import json
import math
import cmath
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
REPAIR = f"{P}/logarithm-narration-clarity-repair01"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
CAND = f"{EL}/combined-repair-stage08/candidate"
REL = "explainer_corpus/logarithm_mysteries"
CRATE = f"{CAND}/{REL}"
PLAN = f"{EL}/combined-repair-stage07/bound-plan03"
TAKES = f"{EL}/ui-takes01"
BINDER3 = f"{P}/logarithm-stage07-binder03/bind_final80-stage07-r3.py"
PINS = {
    f"{REPAIR}/SHA256SUMS": "e9fea0edbd1c61643b633143d8b507c0166406239951016351e090c93ef92a66",
    f"{REPAIR}/logarithm-narration-clarity.patch": "2463334a21069f8d725ef722b362bc8b29aebe22a1b4f98bee0e77e4d515f736",
    f"{REPAIR}/binding-inventory.json": "3a3cee8ccd3a4b1c1dca38fe258d81e964d1af68be315a3ca2399e33dfc349e7",
    f"{REPAIR}/HANDOFF.md": "fbf8d154bb8c070907f70eff1e67ac2ea2d9111bdad0fd8c15b63ee04199e9bf",
    f"{REPAIR}/expected-candidate.sha256": "b7f5332c177766c11588e524f8b20979b1647a71a400fda13458030ee05fe524",
    f"{EL}/combined-repair-stage08/integration01.json": "9ba49565e975a74b404876c4904778450821ee2b65387e96cd220c0fe0526987",
    f"{CRATE}/script.tsv": "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e",
    f"{CRATE}/src/chapters/product_rule.rs": "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db",
    f"{CRATE}/src/chapters/undoing_growth.rs": "4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef",
    f"{CRATE}/src/chapters/mod.rs": "ec18cd537b8c81bd4a1341d752c8b632a4355f083281776863d33c3a03a8ccfa",
    f"{CRATE}/narration_units.json": "0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c",
    f"{CRATE}/docs/narration-direction-map-oliver15.json": "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4",
    f"{CRATE}/docs/check_cues.py": "61fb809dbb4ef291ee868784b6edb7b114bdda47ac4c8adafe63cec7088c7217",
    f"{CRATE}/docs/storyboard.md": "7db2b8edeed482f51c9a99a8e93ba6efc51df3337cccbbde8dc51fefcab772e1",
    f"{PLAN}/phrase-anchors.json": "a5930e689359c846e215863b394334c186b1bb0994047b9dd75e879fb6ea6970",
    f"{PLAN}/hold-audit.json": "4040d484c48e930348726fca61a6ee531cdd58f08211e6c6a237c2ad138f1c9a",
    f"{PLAN}/phrase-timings-unmeasured-template.json": "9169b6004507f6a1a735aa457beb28a5a71b2f9a4d759dc5721ff778b6ef6d28",
    f"{PLAN}/cue-delta.json": "2263bb51a62e41972cd12adca21660b67b515f2fda9781291e6761a4bb56cdac",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    "tools/coherent_narration.py": "e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0",
    BINDER3: "91d5c1a883f540c44e98552d70d3dd8749a81d18bcb232737add0f02b7e0e470",
    f"{EL}/targeted-recognition-small01/review01.json": "2c3779042bf55d728cc0c0c1ee42973ca38c1716e15c90e6175c38529ba2b35a",
    f"{EL}/remaining-recognition01/transcript-review01.json": "bf3727e27c28967c6442d3143470c21779f8951506f32e363ce902f85275c8ef",
    f"{EL}/opening-recognition01/recognition-comparison01.json": "6e894b9aacad3e2ea26ae17f219bbcb6b505709197126586177a329a5be45d78",
    f"{TAKES}/04_4-download01.wav": "de3abb952c675c2112c949813b2d70f962b95a3adf90d8f3289c6a4e29dce9ab",
    f"{TAKES}/04_4-download02.wav": "573d2360f72d4c8d451cd684ff8b3febf68168b24add6b070886433080185164",
    f"{TAKES}/08_5-download01.wav": "9b5d2b73ec6400569a32e03fe2c0d2b91c19308565ea6ee433b9ca99ca0d1ae5",
    f"{TAKES}/08_5-download02.wav": "5ec634e9c591e1768d69ea5195e6b07b91a3da1c6c7cecefa07799b3a269c493",
}
NEW_SCRIPT = "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a"
NEW_PRODUCT = "bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635"
OLD_TEXT = {"04_4": "6a85a6bdd44b5c9942fa03e43b58ddf00f19acd0e69622ed0f45be218b8591a6",
            "08_5": "f91292cc74fa243f405db101c25532df45d16465a66208ac33819baba40f7216"}
NEW_TEXT = {"04_4": "7e4e96121ff25f997a75bc882f70884bd61b3b039203e8df9c9d46fe545d2e83",
            "08_5": "8e26bcb04dba0673057649b76a1fce2ce0375bcc1ff1a8fffb7d8379c13f4d82"}
SPANS = {  # cue: (file, say line, next say or finish line, [(line, sentence)], [(line, const)])
    "04_4": ("undoing_growth.rs", 458, 562, [(522, 1), (531, 2), (534, 3), (540, 4), (550, 5)], []),
    "08_5": ("product_rule.rs", 349, 468, [(352, 1), (362, 2), (408, 3), (431, 4), (458, 5)],
             [(438, "HALF_TURN_PHRASE"), (456, "FACTOR_PHRASE")]),
}
PHRASES = {"HALF_TURN_PHRASE": "leaves a half turn", "FACTOR_PHRASE": "a factor of minus one"}
CARD = "that is why base e is called natural"
fails = []


def check(ok, label):
    print(("PASS " if ok else "FAIL ") + label)
    if not ok:
        fails.append(label)


def sha_b(b):
    return hashlib.sha256(b).hexdigest()


def sha_f(p):
    return sha_b(Path(p).read_bytes())


def sha_t(s):
    return sha_b(s.encode("utf-8"))


def script_rows(path):
    rows = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if line and not line.startswith("#") and "\t" in line:
            cue, text = line.split("\t", 1)
            rows[cue] = text
    return rows


def kit_sentences(text):
    """explainer_kit narration.rs sentences(): split after . ? ! followed by a space or the end; trim; drop empties."""
    b, out, start = text.encode("utf-8"), [], 0
    for i, c in enumerate(b):
        if c in b".?!" and (i + 1 == len(b) or b[i + 1] == 0x20):
            s = b[start:i + 1].decode("utf-8").strip()
            if s:
                out.append(s)
            start = i + 1
    rest = b[start:].decode("utf-8").strip()
    if rest:
        out.append(rest)
    return out


def opens(text):
    """product_rule.rs phrase_seconds(): sentence opens from byte windows of two (. ? ! then a space)."""
    b = text.encode("utf-8")
    return [0] + [i + 2 for i in range(len(b) - 1) if b[i] in b".?!" and b[i + 1] == 0x20]


def direction_words(text):
    """tools/coherent_narration.py direction_words(), which binder03 uses for word_index ("sixty-six" is two words)."""
    return [m.lower().replace("’", "'") for m in re.findall(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?", text)]


def anchor(text, phrase):
    b = text.encode("utf-8")
    byte = b.find(phrase.encode("utf-8"))
    o = opens(text)
    j = max(k for k, x in enumerate(o) if x <= byte)
    return {"byte": byte, "byte_end": byte + len(phrase.encode("utf-8")),
            "word_index": len(direction_words(b[:byte].decode("utf-8"))), "sentence_index": j}


def char_share(text, phrase):
    """phrase_seconds() without a measured mark: the share of its sentence before the phrase, by characters."""
    byte = text.encode("utf-8").find(phrase.encode("utf-8"))
    o = opens(text)
    j = max(k for k, x in enumerate(o) if x <= byte)
    close = o[j + 1] if j + 1 < len(o) else len(text.encode("utf-8"))
    b = text.encode("utf-8")
    before = len(b[o[j]:byte].decode("utf-8"))
    whole = max(1, len(b[o[j]:close].decode("utf-8").rstrip()))
    return before / whole


def rust_test(text, opener):
    """The text half of the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays (ASCII text: str = byte offsets)."""
    start, close = text.find(opener), text.find("The root of a product")
    if start < 0:
        return "panics: no opener"
    if close < 0:
        return "panics: no last sentence"
    after = start
    for name, phrase in PHRASES.items():
        if text.count(phrase) != 1:
            return f"fails: {name} count {text.count(phrase)}"
        at = text.find(phrase)
        if not (after < at < close):
            return f"fails: {name} out of place"
        if text[at - 1:at] != " ":
            return f"fails: {name} does not start a word"
        after = at
    return "passes"


def test_body(src):
    start = src.index("fn the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays()")
    return src[start:src.index("\n    #[test]", start)]


def found_order(src):
    body = test_body(src)
    needles = [re.sub(r'\\(.)', r"\1", m) for m in re.findall(r'found\("((?:[^"\\]|\\.)*)"\)', body)]
    scene = src.split("#[cfg(test)]")[0]
    return needles, [scene.find(n) for n in needles]


def interval(lines, say, marker):
    return sha_t("\n".join(lines[say - 1:marker]))


def gap_inputs(prev, nxt):
    end = prev.rstrip().rstrip("\"')]”’")
    return (end[-1:], nxt.strip().split()[0])


def main(argv):
    if len(argv) != 1:
        print(__doc__.split("\n\n")[1], file=sys.stderr)
        return 2
    scratch = Path(argv[0])
    if scratch.exists():
        print("FRESH_SCRATCH exists; give a new folder", file=sys.stderr)
        return 2

    print("== pins")
    moved = [rel for rel, h in PINS.items() if not (REPO / rel).is_file() or sha_f(REPO / rel) != h]
    print(f"{len(PINS) - len(moved)}/{len(PINS)} inputs are the reviewed bytes")
    if moved:
        for rel in moved:
            print("MOVED " + rel)
        return 3
    sums = (REPO / REPAIR / "SHA256SUMS").read_text(encoding="utf-8").split("\n")
    sealed = {line.split("  ", 1)[1]: line.split("  ", 1)[0] for line in sums if line}
    check(all(sha_f(REPO / REPAIR / n) == h for n, h in sealed.items()) and len(sealed) == 7,
          f"the repair bundle verifies against its SHA256SUMS ({len(sealed)} files)")

    print("== baseline (stage08, read only)")
    old = script_rows(REPO / CRATE / "script.tsv")
    for cue in ("04_4", "08_5"):
        s = kit_sentences(old[cue])
        check(sha_t(old[cue]) == OLD_TEXT[cue] and len(s) == 6 and len(old[cue].split()) == {"04_4": 71, "08_5": 81}[cue],
              f"{cue}: text {OLD_TEXT[cue][:8]}, {len(s)} kit sentences, {len(old[cue].split())} words")
        for j, x in enumerate(s):
            print(f"  {cue} s{j}: {x}")
    for cue, (name, say, end, hooks, phrase_hooks) in SPANS.items():
        lines = (REPO / CRATE / "src/chapters" / name).read_text(encoding="utf-8").split("\n")
        seen = [(n, int(m.group(1))) for n in range(say, end + 1)
                for m in [re.search(r'until_sentence\(stage, "' + cue + r'", (\d+)\)', lines[n - 1])] if m]
        ph = [(n, m.group(1)) for n in range(say, end + 1)
              for m in [re.search(r'until_phrase\(&kit, stage, "' + cue + r'", (\w+)\)', lines[n - 1])] if m]
        check(f'kit.say(stage, "{cue}")' in lines[say - 1] and seen == hooks and ph == phrase_hooks,
              f"{cue}: {name}:{say}-{end} hooks are sentence {[j for _, j in seen]} at {[n for n, _ in seen]}, phrases {ph}")
        other = [n for n in range(say + 1, end) if re.search(r'until_(sentence|phrase)\(|sentence_at\(', lines[n - 1])
                 and n not in [x for x, _ in seen] + [x for x, _ in ph]]
        check(not other, f"{cue}: no other sentence or phrase wait in the span ({other or 'none'})")
    ug = (REPO / CRATE / "src/chapters/undoing_growth.rs").read_text(encoding="utf-8").split("\n")
    check(any(f'"{CARD}"' in ug[n - 1] for n in range(550, 562)), f"04_4: the card shown at sentence 5 reads {CARD!r}")
    anchors = json.loads((REPO / PLAN / "phrase-anchors.json").read_text(encoding="utf-8"))["anchors"]
    rows85 = [a for a in anchors if a["cue"] == "08_5"]
    check(len(rows85) == 2 and not [a for a in anchors if a["cue"] == "04_4"], "bound-plan03: 08_5 has 2 anchors, 04_4 none")
    for a in rows85:
        mine = anchor(old["08_5"], a["phrase"])
        check(all(a[k] == v for k, v in mine.items()) and a["cue_text_sha256"] == OLD_TEXT["08_5"]
              and a["source_sha256"] == PINS[f"{CRATE}/src/chapters/product_rule.rs"],
              f"anchor {a['const']}: root's {[a[k] for k in mine]} = phrase_seconds' rule on stage08 text")
    coherent = (REPO / "tools/coherent_narration.py").read_text(encoding="utf-8")
    exact = [a for a in anchors if all(a[k] == v for k, v in anchor(old[a["cue"]], a["phrase"]).items())]
    split_off = sorted(f"{a['cue']} {a['const']} ({len(old[a['cue']].encode()[:a['byte']].decode().split())} vs {a['word_index']})"
                       for a in anchors if len(old[a["cue"]].encode()[:a["byte"]].decode().split()) != a["word_index"])
    check("re.findall(r\"[A-Za-z0-9]+(?:['’][A-Za-z]+)?\",text)" in coherent and len(exact) == len(anchors) == 33,
          f"all {len(exact)} bound-plan03 anchors reproduce with coherent_narration.direction_words (binder03's word rule)")
    check(split_off == ["11_20 BLOCKS (15 vs 16)", "11_20 MATCHING (22 vs 23)"],
          f"a whitespace split gives a different word_index for {split_off} ('sixty-six' is two direction words)")
    hold = json.loads((REPO / PLAN / "hold-audit.json").read_text(encoding="utf-8"))
    cue_rows = {r["cue"]: r for r in hold["cues"]}
    check(cue_rows["04_4"]["boundary"] == "say_breath" and cue_rows["08_5"]["boundary"] == "chapter_end"
          and cue_rows["08_4"]["boundary"] == "prediction", "hold audit: 04_4 say_breath, 08_4 prediction, 08_5 chapter_end")
    template = json.loads((REPO / PLAN / "phrase-timings-unmeasured-template.json").read_text(encoding="utf-8"))["cues"]
    check(template["04_4"]["text_sha256"] == OLD_TEXT["04_4"] and template["04_4"]["marks"] == []
          and template["08_5"]["text_sha256"] == OLD_TEXT["08_5"]
          and [m["byte"] for m in template["08_5"]["marks"]] == [260, 299],
          "template: 04_4 and 08_5 bound to the old texts; 08_5 marks at bytes [260, 299]")
    units = json.loads((REPO / CRATE / "narration_units.json").read_text(encoding="utf-8"))
    check(all(u["text"] == " ".join(old[i] for i in u["line_ids"]) for u in units["units"]) and len(units["units"]) == 80,
          "units: all 80 unit texts are the stage08 script rows, 04_4 and 08_5 included")
    pr_old = (REPO / CRATE / "src/chapters/product_rule.rs").read_text(encoding="utf-8")
    check(pr_old.count('text.find("Halving the lost lap")') == 1 and test_body(pr_old).count("Halving") == 2
          and pr_old.split("\n")[431].strip().startswith('// "Halving the lost lap"'),
          "product_rule.rs: the test finds \"Halving the lost lap\" (and its comment says \"Halving ...\"); line 432 quotes it")
    b3 = (REPO / BINDER3).read_text(encoding="utf-8")
    check('STAGE = STAGE07_ROOT / "candidate" / REL' in b3 and PINS[f"{CRATE}/script.tsv"] in b3
          and PINS[f"{CRATE}/src/chapters/product_rule.rs"] in b3 and "sys.exit(3)" in b3,
          "binder03 reads stage07 and pins script b89eab60 and product_rule.rs f2c8684a: it cannot bind the patched bytes")

    print("== replay (scratch copies)")
    control, patched = scratch / "control", scratch / "patched"
    for tree in (control, patched):
        shutil.copytree(REPO / CRATE, tree / REL)
    r = subprocess.run(["git", "apply", "--check", str(REPO / REPAIR / "logarithm-narration-clarity.patch")], cwd=patched,
                       capture_output=True, text=True)
    check(r.returncode == 0, f"git apply --check exit {r.returncode}")
    r = subprocess.run(["git", "apply", str(REPO / REPAIR / "logarithm-narration-clarity.patch")], cwd=patched,
                       capture_output=True, text=True)
    check(r.returncode == 0, f"git apply exit {r.returncode}")
    check(sha_f(patched / REL / "script.tsv") == NEW_SCRIPT
          and sha_f(patched / REL / "src/chapters/product_rule.rs") == NEW_PRODUCT,
          f"patched script.tsv {NEW_SCRIPT[:8]}, product_rule.rs {NEW_PRODUCT[:8]}")
    changed = sorted(str(p.relative_to(patched)) for p in patched.rglob("*")
                     if p.is_file() and p.read_bytes() != (control / p.relative_to(patched)).read_bytes())
    added = sorted(str(p.relative_to(patched)) for p in patched.rglob("*") if not (control / p.relative_to(patched)).exists())
    check(changed == [f"{REL}/script.tsv", f"{REL}/src/chapters/product_rule.rs"] and not added,
          f"only script.tsv and product_rule.rs differ; nothing added ({len(changed)} changed)")
    reports = []
    for tree in (control, patched):
        r = subprocess.run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=tree / REL, capture_output=True, text=True)
        reports.append((r.returncode, r.stdout, r.stderr))
    check(reports[0] == reports[1] and reports[0][0] == 0 and reports[0][1].rstrip().endswith("OK"),
          f"docs/check_cues.py: exit 0 and the same report on both copies (last line {reports[1][1].rstrip().splitlines()[-1]!r})")

    print("== meaning")
    new = script_rows(patched / REL / "script.tsv")
    check(list(new) == list(old) and [c for c in old if old[c] != new[c]] == ["04_4", "08_5"], "only rows 04_4 and 08_5 change")
    for cue, j in (("04_4", 5), ("08_5", 4)):
        so, sn = kit_sentences(old[cue]), kit_sentences(new[cue])
        check(len(so) == len(sn) == len(opens(new[cue])) == 6 and [k for k in range(6) if so[k] != sn[k]] == [j]
              and sha_t(new[cue]) == NEW_TEXT[cue],
              f"{cue}: 6 kit sentences = 6 phrase_seconds opens; only s{j} changes; text {NEW_TEXT[cue][:8]}")
        print(f"  {cue} s{j} old: {so[j]}")
        print(f"  {cue} s{j} new: {sn[j]}")
        check(not re.search(r"\d", new[cue]) and len(new[cue].split()) <= 100 and new[cue].isascii(),
              f"{cue}: ASCII, no digits, {len(new[cue].split())} words")
    pairs = [("04_3", "04_4"), ("04_4", "04_5"), ("08_4", "08_5"), ("08_5", "09_1")]
    check(all(gap_inputs(old[a], old[b]) == gap_inputs(new[a], new[b]) for a, b in pairs),
          "classify_gap inputs (previous line's end mark, next line's first word) unchanged at all four boundaries")
    s5 = kit_sentences(new["04_4"])[5]
    check(s5.endswith("why base e is called natural.") and CARD.split("why ")[1] in s5 and "this base" not in new["04_4"],
          "04_4: the voice now says the card's words 'base e is called natural'")
    words = s5.rstrip(".").split()
    before_base = words[words.index("base") - 1]
    check(before_base == "why", f"04_4: the word before 'base' is {before_base!r} (ends in a vowel, no /s/ run into /b/)")
    lone_e = sorted(c for c, t in new.items() if re.search(r"(^|\s)e(\s|[,.])", t))
    check({"04_1", "04_3", "04_5", "08_5"} <= set(lone_e), f"a spoken lone 'e' already occurs in {len(lone_e)} cues: {lone_e}")
    check("alving" not in new["08_5"] and "aving" not in new["08_5"], "08_5: no 'halving' or 'having' left")
    pr_new = (patched / REL / "src/chapters/product_rule.rs").read_text(encoding="utf-8")
    op_old = re.search(r'text\.find\("([^"]+)"\)\.expect\("08_5 has the halving"\)', pr_old).group(1)
    op_new = re.search(r'text\.find\("([^"]+)"\)\.expect\("08_5 has the halving"\)', pr_new).group(1)
    grid = {(t, o): rust_test(text, op) for t, text in (("old", old["08_5"]), ("new", new["08_5"]))
            for o, op in (("old", op_old), ("new", op_new))}
    check(grid[("old", "old")] == grid[("new", "new")] == "passes" and grid[("new", "old")].startswith("panics"),
          f"Rust test mirror (opener read from each copy's test): {grid}")
    for name, src in (("control", pr_old), ("patched", pr_new)):
        needles, pos = found_order(src)
        check(len(needles) == 11 and all(p >= 0 for p in pos) and pos == sorted(pos) and len(set(pos)) == 11,
              f"{name}: the test's 11 found(...) calls, read from its source, are present and in order in the scene source")
    for name, phrase in PHRASES.items():
        a_old, a_new = anchor(old["08_5"], phrase), anchor(new["08_5"], phrase)
        check(a_new["sentence_index"] == 4 and new["08_5"].count(phrase) == 1 and f'const {name}: &str = "{phrase}";' in pr_new,
              f"{name}: still once, in sentence 4; byte {a_old['byte']} -> {a_new['byte']}, word {a_old['word_index']} -> "
              f"{a_new['word_index']}; char share {char_share(old['08_5'], phrase):.3f} -> {char_share(new['08_5'], phrase):.3f}")
    nb = new["08_5"].encode("utf-8")
    stale = {b: nb[b:b + 14].decode("utf-8") for b in (260, 299)}
    fresh = sorted(anchor(new["08_5"], p)["byte"] for p in PHRASES.values())
    check(fresh == [267, 306] and all(b < len(nb) and nb[b:b + 1] != b"" for b in stale) and not set(stale) & set(fresh),
          f"old mark bytes in the new text land at {stale}: in range, so the kit would load them, and neither is a phrase "
          f"(phrase bytes now {fresh}); phrase_seconds would fall back to the char share")
    lines_old, lines_new = pr_old.split("\n"), pr_new.split("\n")
    diff = [n + 1 for n, (a, b) in enumerate(zip(lines_old, lines_new)) if a != b]
    tests_at = pr_old.split("\n").index("#[cfg(test)]") + 1
    check(len(lines_old) == len(lines_new) and diff == [432, 722, 723] and 349 < 432 < 468 and lines_new[431].strip().startswith("//")
          and tests_at < 722 and max(len(lines_new[n - 1]) for n in diff) <= 120,
          f"product_rule.rs: lines {diff} change, no line moves; 432 is a comment inside 08_5's span, 722-723 are in tests "
          f"(from line {tests_at}); longest {max(len(lines_new[n - 1]) for n in diff)} chars")
    for h in hold["prediction_holds"]:
        if h["file"] not in ("product_rule.rs", "undoing_growth.rs"):
            continue
        src_old = (control / REL / "src/chapters" / h["file"]).read_text(encoding="utf-8").split("\n")
        src_new = (patched / REL / "src/chapters" / h["file"]).read_text(encoding="utf-8").split("\n")
        i_old, i_new = interval(src_old, h["say_line"], h["marker_line"]), interval(src_new, h["say_line"], h["marker_line"])
        touched = diff if h["file"] == "product_rule.rs" else []
        check(i_old == h["interval_sha256"] == i_new and not any(h["say_line"] <= n <= h["marker_line"] for n in touched),
              f"hold {h['cue']} ({h['file']}:{h['say_line']}-{h['marker_line']}): binder interval {i_old[:8]} reproduced, unchanged")
    print("  mathematics, in floats:")
    ln101 = math.log(1.01)
    log_m1 = cmath.log(-1)
    root_m1 = cmath.exp(0.5 * log_m1)
    prod_roots, root_prod = root_m1 * root_m1, cmath.exp(0.5 * cmath.log(1))
    lost = (log_m1 + log_m1) - cmath.log(1)
    checks = [abs(1 / 1.0 - 1) < 1e-15, abs(ln101 - 0.01) / 0.01 < 0.006, abs(log_m1 - math.pi * 1j) < 1e-15,
              abs(prod_roots + 1) < 1e-12, abs(root_prod - 1) < 1e-15, abs(lost - 2j * math.pi) < 1e-12,
              abs(0.5 * lost - math.pi * 1j) < 1e-12, abs(cmath.exp(0.5 * lost) + 1) < 1e-12,
              abs(prod_roots / root_prod - cmath.exp(0.5 * lost)) < 1e-12]
    print(f"  ln 1.01 = {ln101:.8f}; Log(-1) = {log_m1.imag / math.pi:.0f}*pi*i; sqrt(-1)*sqrt(-1) = {prod_roots.real:.0f}; "
          f"sqrt(1) = {root_prod.real:.0f}; lost = {lost.imag / math.pi:.0f}*pi*i; half of it = {0.5 * lost.imag / math.pi:.0f}*pi*i; "
          f"e^(half) = {cmath.exp(0.5 * lost).real:.0f}")
    check(all(checks), "slope 1/x is 1 at 1; ln 1.01 within 0.6% of 0.01; half the lost lap is pi*i, a factor -1 = (product of roots)/(root of product)")

    print("== oliver retained for logarithm")
    receipts = sorted((REPO / TAKES).glob("*-request*.json"))
    rec = {p.name.split("-")[0]: json.loads(p.read_text(encoding="utf-8")) for p in receipts}
    ctx = sorted({k for r in rec.values() for k in r if re.search(r"(?i)previous|next|context|stitch|prior", k)})
    check(len(receipts) == len(rec) == 80 and not ctx, f"{len(receipts)} Oliver request receipts, one per cue; context fields: {ctx or 'none'}")
    quoting = sorted(c for c, r in rec.items() if c not in ("04_4", "08_5")
                     and any(w in json.dumps(r) for w in ("this base is called", "Halving the lost lap")))
    check(not quoting, f"no other cue's receipt carries 04_4's or 08_5's changed words ({quoting or 'none'})")
    heard = []  # (cue, model, transcript) from root's three machine-recognition runs; machine text, not listening
    opening = json.loads((REPO / EL / "opening-recognition01/recognition-comparison01.json").read_text(encoding="utf-8"))
    heard += [(r["cue"], "large-v3-turbo", r["recognition"]) for r in opening["records"]]
    remaining = json.loads((REPO / EL / "remaining-recognition01/transcript-review01.json").read_text(encoding="utf-8"))
    heard += [(r["cue"], "large-v3-turbo", r["transcript"]) for r in remaining["rows"]]
    small = json.loads((REPO / EL / "targeted-recognition-small01/review01.json").read_text(encoding="utf-8"))
    heard += [(Path(r["file"]).name.split("-")[0], "small.en", r["transcript"]) for r in small["rows"]]
    halv = {c: re.findall(r"\b[Hh]alv\w*", t) for c, t in old.items() if re.search(r"\b[Hh]alv", t)}
    tally = {}
    for cue, model, text in heard:
        if cue in halv:
            tally.setdefault((cue, model), []).append(" ".join(re.findall(r"\b(?:[Hh]alv\w*|[Hh]aving)\b", text)) or "-")
    for (cue, model), got in sorted(tally.items()):
        print(f"  {cue} says {halv[cue]}; {model} wrote {got}")
    check(tally.get(("08_5", "large-v3-turbo")) == ["Having"] * 2 and tally.get(("08_5", "small.en")) == ["Having"] * 2
          and tally.get(("01_2", "large-v3-turbo")) == ["Halving"] * 2 and sorted(halv) == ["01_2", "07_5", "08_5", "11_3"],
          "08_5 'Halving' written 'Having' 4/4 by two models; 01_2's sentence-initial 'Halving' (retained take) written 'Halving' 2/2")
    sets = sorted({(r.get("model"), r.get("stability"), r.get("similarity"), r.get("clone_strength")) for r in rec.values()})
    voices = sorted({r.get("voice") for r in rec.values()})
    check(sets == [("Eleven v4", 0.5, 0.75, 0)] and all(v.startswith("Oliver") for v in voices),
          f"one setting for all 80: {sets}; voice names {voices}")
    bound = sorted(c for c, r in rec.items() if r.get("text_sha256") == sha_t(new.get(c, "")) and r.get("text") == new.get(c))
    check(len(bound) == 78 and sorted(set(new) - set(bound)) == ["04_4", "08_5"],
          f"{len(bound)} of 80 receipts match the patched text; unbound: {sorted(set(new) - set(bound))}")
    check(all(r["text_sha256"] == OLD_TEXT[c] for c, r in rec.items() if c in OLD_TEXT),
          "the 04_4 and 08_5 receipts bind the old texts; their four takes are present with their recorded hashes (pins)")
    inv = json.loads((REPO / REPAIR / "binding-inventory.json").read_text(encoding="utf-8"))
    handoff = (REPO / REPAIR / "HANDOFF.md").read_text(encoding="utf-8")
    check("Frederick Surrey for new production" in inv["voice"] and "needs a Frederick take anyway" in handoff
          and "root's call" in inv["map_rebind"]["naming"],
          "repair bundle, as sealed: voice premise is Frederick for the whole film (superseded for logarithm at 03:22:06Z)")

    print("== inventory: files holding an old hash or the old words")
    needles = {"04_4 text": OLD_TEXT["04_4"], "08_5 text": OLD_TEXT["08_5"],
               "product_rule.rs": PINS[f"{CRATE}/src/chapters/product_rule.rs"], "script": PINS[f"{CRATE}/script.tsv"],
               "words 04_4": "this base is called", "words 08_5": "Halving the lost lap"}
    named_text = json.dumps(inv) + handoff
    roots = [REPO / CRATE, REPO / PLAN, REPO / EL / "narration-request-final80-01", REPO / EL / "narration-request-final80-02"]
    table = []
    for root in roots:
        for p in sorted(root.rglob("*")):
            if not p.is_file() or p.suffix in (".wav", ".lock"):
                continue
            try:
                body = p.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            hit = [k for k, v in needles.items() if v in body]
            if hit:
                rel = str(p.relative_to(REPO / EL))
                table.append((rel, hit, p.name in named_text or (p.name == "hold-audit.json" and "hold_audit" in inv)))
    for rel, hit, named in table:
        print(f"  {'named  ' if named else 'UNNAMED'} {rel}: {', '.join(hit)}")
    unnamed = sorted(r for r, _, n in table if not n)
    want = ["combined-repair-stage07/bound-plan03/cue-delta.json", "combined-repair-stage07/bound-plan03/increment-delta.json",
            "combined-repair-stage07/bound-plan03/inputs.sha256",
            "combined-repair-stage07/bound-plan03/phrase-timings-unmeasured-template.json",
            "combined-repair-stage07/bound-plan03/stage07-delta.json"]
    check(unnamed == want, f"the repair bundle leaves {len(unnamed)} binding files unnamed (template and cue-delta matter; "
          f"the two deltas and inputs.sha256 record the past bind)")

    print()
    print("RESULT", "PASS" if not fails else f"FAIL ({len(fails)})")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
