#!/usr/bin/env python3
"""complex-frederick-remaining-cues-review01: the check behind FINDINGS.md.

Read-only pre-synthesis audit of the 50 complex cues that had no Frederick
downloads when root asked for it (Mail 4481). Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-remaining-cues-review01/checks/remaining_cues_review.py

It writes nothing. It reads root's stage05 crate (script, units, map, chapter
sources, facts oracle), the prepared requests and the sealed timing review,
and runs root's `oracle.py` once, as root's facts-check does, to read the
numbers the scenes print. Sections:

  1. pins            every input at its hash (exit 3 if one moved)
  2. registry        the remaining cues, chapter by chapter, each with one scene block
  3. requests        each request is the script's exact text with Frederick's settings
  4. hooks and holds the bridge's anchors and holds against this check's own scan
  5. mathematics     every spoken number and claim, recomputed with the standard library
  6. notation        spoken notation, and the words that drew content questions before
  7. findings        the evidence for each finding; each proposed wording keeps its hooks
  8. hygiene         (from the bundle only) the bundle's files

Exit 0 PASS, 1 FAIL, 2 usage, 3 an input moved. Nothing here listens, decodes,
synthesizes or submits; a level or a recognizer word is not a claim about sound.
"""
import cmath
import hashlib
import importlib.util
import json
import math
import re
import subprocess
import sys
from fractions import Fraction
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
CRATE = f"{EC}/combined-repair-stage05/explainer_corpus/complex_analysis_visual"
TAKES = f"{EC}/frederick-stage05-01"
BRIDGE = f"{P}/complex-frederick-timing-bridge-repair01/frederick_timing_bridge.py"
BUNDLE = f"{P}/complex-frederick-remaining-cues-review01"

PINS = {
    BRIDGE: "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39",
    f"{P}/complex-frederick-timing-bridge-repair01/SHA256SUMS":
        "d5a413dd5856d91338c3f3c248bd194cc7feb80a9a9d635a65043a904c3b025d",
    f"{P}/complex-frederick-current-timing-review01/SHA256SUMS":
        "42f34a752736025b6a9b0634e8773b2e2856fa5d7d60cb57f670ea25d984c4c1",
    f"{P}/complex-frederick-current-timing-review01/candidates.json":
        "241ac7106a953de335c236813fb3eb01fa37fa1e28bbdfc73151e5ae43c932a5",
    f"{CRATE}/oracle.py": "3d661ca5471cc0484e7e574d9751e54397c762be7efbbd70add65f81b509b794",
    f"{CRATE}/src/facts.rs": "12f2fef0d6807540d3642972a1125c89e683d5f9615689f4100e6aea522530bc",
    f"{CRATE}/src/model.rs": "aa4bdcd9afcf19b7741eb9df5862001c61f7f0ba9e28affe8032c63755523f4d",
    f"{TAKES}/09_5-submission01.json": "dbeb60fe6eec69396f2b386fef6ebaefd0ee2e37607057192a4c0eee25fb8f28",
    f"{TAKES}/09_5-submission-observation01.json":
        "1ccd7bd36ec41e1685370997ac547e1f97194136ff0c6fe314395b25ecbadcdf",
    "explainer_corpus/complex_analysis_visual/script.tsv":
        "2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e",
}
REMAINING = [
    "05_8", "05_9", "06_8", "06_9", "09_5", "09_6", "09_7", "09_8", "09_9",
    "10_1", "10_2", "10_3", "10_4", "10_5", "10_6",
    "11_1", "11_2", "11_3", "11_4", "11_5", "11_6", "11_7", "11_8",
    "12_1", "12_2", "12_3", "12_4",
    "13_1", "13_2", "13_3", "13_4", "13_5", "13_6", "13_7", "13_8", "13_9", "13_10", "13_11",
    "14_1", "14_2", "14_3", "14_4", "14_5", "14_6", "14_7", "14_8",
    "15_1", "15_2", "15_3", "15_4",
]
REQUEST_SHA256 = {  # each cue's request01.json, as root prepared it
    "05_8": "91516f69c1b042e7bd123104d4758e3ea713f383bc47a0bfc13c7f29bc4aff85",
    "05_9": "154ee0609f50156f8753c38631e23db448f1f583dcc895a76796a32373e41777",
    "06_8": "fbb0b58c629c0aa9a80a519a88e584896cc8ab04a886a3b09e75b6f36008a095",
    "06_9": "1ae7470d6eb995665cd6e1a4654df1e18549a4dfd23b27b855ec72816ec48d1f",
    "09_5": "035022fa88c59304cd18522cce0f4b8771154ed6d284e19585c61caa8594277c",
    "09_6": "537d1cad10864599041a00af8d90f9a0a65a78d77983c2123109213080fa717f",
    "09_7": "cf703a1cffadd70bb21c6083c873fcdc5d166728442334743c640a1947fef9f8",
    "09_8": "b9043573acc92ed4a8c90693b8edd20554a97764e4c1f640e956e39cd6681346",
    "09_9": "8297b688a5297b9e1ab1e1f23033ca1cbf1219f3aa857c2197952221df728dd9",
    "10_1": "4740fcc3f26d6b4b73ff6e6883ae4d33321df3b9b054c907cc9d083e65424a63",
    "10_2": "3721382a3e09621087afebf05ebf0c5033d3264750c9eea6b5553dfb0cd18c5d",
    "10_3": "1da5429afb36a410191aaed5d76782fb5867efed6bebf064c59c64656b8396c4",
    "10_4": "a9e7a0e3d1b7baa74275957f5338f6be46a53f47d5771ad5564a2c7ba6779984",
    "10_5": "c0f8de20c09c21a692567fe272426340d294c6e9d1d9e19dbd8dfd990e5b883a",
    "10_6": "deddd958de30a379edd65bc5d796e1880f0644c7eb0d1bb2184d46d6c919f7e6",
    "11_1": "2da4c66b4c369bf42f475c425dd18acfe4d05b42a1ae023c6abeaf530f65ee93",
    "11_2": "ec7fd3d6599b5bec97e42062054fed06e17708205a20f2079b08e210c42b8627",
    "11_3": "91e7ad3d7923fff185cd59c0fe29e528c3c744d2b6fd99996f29084705212abc",
    "11_4": "e1db4a9fd2f8342e36021cd3ee317395b4bdd7e2a32142e69a5623659cffdd52",
    "11_5": "0ac11bff7fe70ce07249fc5eabe31c3144f6bdab63ea20184fd59d605b48e5b3",
    "11_6": "ddf90acdad06dc01fa4360181504c33addfc0417809bdcaf81bd41532ba96b4b",
    "11_7": "fcf5800dd9f0e5059410ff717641729a368585e30a10dc405902c9e85fac66a5",
    "11_8": "596d1781abf268ab5cc884425992ece241f7f28c6b8eb65e2705ef87004647a6",
    "12_1": "04b0a41330085b8740052edd3ceff4e63c95f392da9bc87b26816fdc7c4ca318",
    "12_2": "ab3b2da652a2d0ed60375948365970cdd75290a15f715712a7398de63570ea5a",
    "12_3": "8ba6228b80c2a539b195477aa06d64dad9adeef84636721e3f9beac8dd42f093",
    "12_4": "268a72ca93cecc2d953aa9168aa708adc3148f4bbb5eed7878708ef555cfe49e",
    "13_1": "44d1c753479f4717ea33c6af1350b91770841a0fe736d7fb176d17f07d955215",
    "13_2": "a487e28e2c72e668244e8c931d2296203b9152d8cc10518d969073e932af2387",
    "13_3": "38eda947ebb67f3b7f6832df95bda1cdf4cd9a437d697b18203b8b86895eaf41",
    "13_4": "396d2cc914c54cbf918710f5cae13ab7611901f25aecb4dd9f3a32fe179e80fb",
    "13_5": "a5cc1795bba4ee53c17b7f9cd7b9bb094a887cafa344ef1b574d3f984c846e55",
    "13_6": "f4c6200d8b5e207d339311a9c4199fe1cd4a800cf86a0ad0fdd89fb0864ceecf",
    "13_7": "9221c00211fa5bb82bb01043df459b48c3f4c7137fa7219a704a3c635c33982e",
    "13_8": "618ca34859063208aecf29cdd56aa59f1bbc0061ab7b6822feb99684a2ada0cf",
    "13_9": "463fbf73b0c686715c7ed4ec9083e76cb9c3b150ea843253e0a97625b935c35b",
    "13_10": "010cc25a9947452c17cfd00191f82d0c9cc76c08b9752281a5151f1710f41bd4",
    "13_11": "265779fc4afda38707952b0310cacceb5de2c976e3a1c178a4cbd4bf216a5cc3",
    "14_1": "06503e94b9df6f5f72a8af44a9bb5b3cc283976dd8df3a64df11fdb378543d28",
    "14_2": "bb459a298f63af3801f162fcb34021ddbd0e32f3a67aeff0ce1e321d3063e8c4",
    "14_3": "b12bf9dcc46d1a5d09f1afdd553cd30e7710547970c23c4fd8440cd1f8b7b073",
    "14_4": "64f415343721bb08ecc897d259eff06df61c58700c40e69ed47219213fe9fde7",
    "14_5": "9c30552bb84a45c2c45109fcacf3e6b1fa1250218197c09abe7865fa2462bc59",
    "14_6": "f090f76f91584927c12109166aa8491c9c7c72946ca49af90cfbd805adf3ef76",
    "14_7": "349ac14f0c37686538c624590525954e09fa31cafe0ea1d219cd3304c14b34bd",
    "14_8": "e8ef669c81d32958a90b5306ceee06dd5f9fa2b38c41045d6105ba2cdef2df3a",
    "15_1": "3c775e59cc07519ebc74361c1551804a3d6bac2bf67a198ac0052b83b8a28326",
    "15_2": "7035c8f167078a84d8ad4c194fffd97a889d49ff4f9912c1dff9ace91f4542bd",
    "15_3": "bbbaf0f5b6f9ddeed61ed866fb4881cde243a472046dff5ebbc71d8853fe43e4",
    "15_4": "d5069acac41c3536b651e22b282a225649f1ad725c04e59cc397948a201456c2",
}
SETTINGS = {"voice": "Frederick Surrey - Smooth and Velvety", "model": "Eleven v4", "stability": 0.5,
            "similarity": 0.75, "clone_strength": 0, "output": "WAV 48 kHz Lossless"}
CHAPTER_FILES = {"05": "gallery.rs", "06": "one_derivative.rs", "09": "boundary.rs", "10": "rigidity.rs",
                 "11": "counting.rs", "12": "residues.rs", "13": "real_integral.rs", "14": "continuing.rs",
                 "15": "closing.rs"}
CANONICAL_CHANGED = ["11_7", "15_2"]  # stage05 against the frozen canonical script
# The spoken forms behind the completed takes' notation questions (Mail 4394 review): "one half", "one
# quarter" (03_2), "minus i" (04_7), "one plus i" and "two plus two i" (06_4), "to the fortieth" (01_3),
# "nonzero" (07_2), "f of z equals z" (08_2). The same forms in the remaining cues:
FORMS = {
    "half": r"\b(?:one|a) half\b",
    "quarter": r"\b(?:one|a) quarter\b",
    "minus": r"\bminus (?:i|p|n|R|z)\b",
    "plus": r"\bplus (?:\w+ )?(?:i|z|one)\b",
    "to the fortieth": r"\bto the (?:\w+th|n)\b",
    "nonzero": r"\bnon\w{4,}",
    "of z equals": r"\bof z equals\b",
    "one plus i": r"\bone plus i\b",
}

# ---------------------------------------------------------------- findings
# Each proposal replaces one exact span of one cue. They are wording options for root, not edits.
PROPOSALS = {
    "05_8": ("No zero found so far sits off that line. Whether all of them lie on it",
             "Apart from those at minus two, minus four, and on, no zero found so far sits off that line. "
             "Whether all the rest lie on it"),
    "15_2": ("On an open region, that makes every small piece of the map a stretch and a turn.",
             "On an open region, away from critical points, that makes every small piece of the map a "
             "stretch and a turn."),
    "10_3": ("so its power series misses the curve completely.", "so its power series misses the whole rise."),
}
EVIDENCE = [  # (cue, exact text the finding rests on)
    ("05_7", "Zeros at minus two, minus four, and on."),
    ("05_7", "And more zeros climbing the line where the real part is one half."),
    ("05_8", "No zero found so far sits off that line."),
    ("07_1", "Where the derivative is zero, the picture changes."),
    ("07_1", "The angle has doubled, from a right angle to a straight line."),
    ("07_2", "A point where the derivative is zero is a critical point, and there angles get multiplied."),
    ("15_2", "that makes every small piece of the map a stretch and a turn."),
    ("10_3", "It agrees with zero on a whole half line, yet it is not zero."),
    ("10_3", "so its power series misses the curve completely."),
    ("09_8", "Recentre the recipe"),
    ("14_3", "Re-centre the series"),
    ("14_1", "Of course not. We know it everywhere."),
    ("13_10", "less than one three hundredth"),
]
SOURCE_EVIDENCE = [  # (file, exact source text)
    ("gallery.rs", "let rings: Vec<Mob> = zeta_zeros()"),
    ("gallery.rs", 'const TRIVIAL: &str = "zeros at ";'),
    ("closing.rs", '["on an open region:", "stretch and turn"]'),
    ("rigidity.rs", "polyline(vec![g.p(0.0, 0.0), g.p(g.x.1, 0.0)])"),
    ("rigidity.rs", "for k in 1..=7 {"),
    ("rigidity.rs", 'const ZERO_RECIPE_TEX: &str = r"0 + 0\\,x + 0\\,x^2 + \\cdots = 0";'),
    ("continuing.rs", 'const STEP: &str = "step by step: re-centre inside the disk";'),
    ("boundary.rs", 'const ASK_RECENTRE: [&str; 2] = ["recipe centred at 1:", "works from ? to ?"];'),
    ("continuing.rs", 'const EVERYWHERE: &str = "the bump: defined everywhere but ±i";'),
    ("real_integral.rs", 'const UNDER: &str = " < 1/";'),
]

failures = []


def check(name, ok, detail):
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")
    if not ok:
        failures.append(name)


def note(text):
    print(f"   {text}")


class PinMoved(Exception):
    pass


def fixed(x, places):
    """x to `places` decimals, never "-0": the printing rule of root's facts.rs and oracle."""
    s = f"{x:.{places}f}"
    return s[1:] if s.startswith("-") and set(s[1:]) <= set("0.") else s


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


def load_bridge():
    spec = importlib.util.spec_from_file_location("frederick_timing_bridge", BRIDGE)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ------------------------------------------------------------------ 1 pins
def section_pins():
    print("== 1. Pins")
    moved = [p for p, s in PINS.items() if sha_file(p) != s]
    moved += [f"{TAKES}/{c}-request01.json" for c, s in REQUEST_SHA256.items()
              if sha_file(f"{TAKES}/{c}-request01.json") != s]
    if len(REQUEST_SHA256) != len(REMAINING):
        moved.append("REQUEST_SHA256 does not pin all 50 requests")
    sums = Path(f"{P}/complex-frederick-timing-bridge-repair01/SHA256SUMS").read_text().split("\n")
    listed = {line.split("  ", 1)[1]: line.split("  ", 1)[0] for line in sums if line}
    bad_seal = [n for n, s in listed.items() if sha_file(f"{P}/complex-frederick-timing-bridge-repair01/{n}") != s]
    if moved or bad_seal:
        raise PinMoved(f"moved: {moved} {bad_seal}")
    bridge = load_bridge()
    intake = bridge.dep("intake")
    script, files = intake.check_binding(Path(CRATE))
    side, _script, _files = bridge.source_side(Path(CRATE))
    check("p1-pins", True,
          f"the repaired bridge 47069583 and its seal ({len(listed)} files), the sealed timing review 42f34a75, "
          f"root's oracle, facts.rs and model.rs, the 50 requests and 09_5's two receipts are at their pins; "
          f"the bridge binds the stage05 script {intake.PINS['script'][:8]}, units {intake.PINS['units'][:8]}, "
          f"map {intake.PINS['map'][:8]} and the chapter sources {bridge.CHAPTER_SOURCES_SHA256[:8]}")
    return bridge, intake, script, files, side


# -------------------------------------------------------------- 2 registry
def blocks(name):
    """Each say() block of a chapter file, up to its test module: {cue: (first line, lines)}."""
    text = (Path(CRATE) / "src/chapters" / name).read_text()
    body = text.split("\n#[cfg(test)]\nmod tests {")[0]
    lines = body.split("\n")
    starts = [(k, m.group(1)) for k, x in enumerate(lines)
              for m in [re.search(r'say\(&kit, stage, "(\d\d_\d+)"\)', x)] if m]
    out = {}
    for n, (k, cue) in enumerate(starts):
        stop = starts[n + 1][0] if n + 1 < len(starts) else len(lines)
        out.setdefault(cue, []).append((k + 1, lines[k:stop]))
    return out, body


def section_registry(script):
    print("== 2. Registry: the remaining cues and their scene blocks")
    mod = (Path(CRATE) / "src/chapters/mod.rs").read_text()
    keys = re.findall(r'key: "(\d\d)_[a-z_]+",\s*make: \|kit\| Box::new\((\w+)::', mod)
    modules = {k: m for k, m in keys}
    names = {p.name for p in Path(TAKES).iterdir()}
    now = [c for c in script if f"{c}-downloads01.json" not in names]
    found, all_blocks = {}, {}
    for chapter, name in CHAPTER_FILES.items():
        b, _ = blocks(name)
        all_blocks[name] = b
        for cue, sites in b.items():
            found.setdefault(cue, []).extend((name, k) for k, _ in sites)
    ok = (len(script) == 105 and len(keys) == 15
          and all(modules.get(c[:2]) + ".rs" == CHAPTER_FILES[c[:2]] for c in REMAINING)
          and all(len(found.get(c, [])) == 1 and found[c][0][0] == CHAPTER_FILES[c[:2]] for c in REMAINING))
    chapters = {}
    for c in REMAINING:
        chapters.setdefault(c[:2], []).append(c)
    check("q1-registry", ok,
          f"105 script cues in 15 registered chapters; the 50 without Frederick downloads at review time are "
          + ", ".join(f"{k} ({len(v)})" for k, v in chapters.items())
          + "; each has exactly one say() block, in its chapter's registered module")
    if now != REMAINING:
        note(f"NOTE q1: the take folder has moved on: {len(REMAINING) - len([c for c in REMAINING if c in now])} "
             f"of the 50 now have downloads; this review covers the 50 as prepared")
    return all_blocks


# -------------------------------------------------------------- 3 requests
def section_requests(script, files):
    print("== 3. Requests: exact text and Frederick's settings")
    units = {u["line_ids"][0]: u["text"] for u in json.loads(Path(files["units"]).read_text())["units"]}
    mapdata = json.loads(Path(files["map"]).read_text())
    after = {o["cue"]: o["after"] for o in mapdata["overrides"]}
    names = sorted(p.name for p in Path(TAKES).iterdir())
    latest = {}
    for n in names:
        m = re.match(r"(\d\d_\d+)-request(\d\d)\.json$", n)
        if m:
            latest[m.group(1)] = n
    every = [json.loads((Path(TAKES) / latest[c]).read_text()) for c in script if c in latest]
    bad, newer = [], []
    script_sha = sha_file(files["script"])
    for c in REMAINING:
        r = json.loads((Path(TAKES) / f"{c}-request01.json").read_text())
        if latest.get(c) != f"{c}-request01.json":
            newer.append(latest.get(c))
        problems = [k for k in SETTINGS if r.get(k) != SETTINGS[k]]
        if r["text"] != script[c] or units.get(c) != script[c]:
            problems.append("text")
        if r["text_sha256"] != sha_bytes(r["text"].encode("utf-8")) or r["script_sha256"] != script_sha:
            problems.append("hash")
        if r.get("submitted") is not False or r.get("cue") != c:
            problems.append("state")
        if c in after and after[c] != script[c]:
            problems.append("map after")
        if problems:
            bad.append((c, problems))
    voices = sorted({r["voice"] for r in every})
    canonical = {}
    for line in Path("explainer_corpus/complex_analysis_visual/script.tsv").read_text().split("\n"):
        if line and not line.startswith("#"):
            k, text = line.split("\t", 1)
            canonical[k] = text.split("\t")[0]
    changed = sorted(c for c in script if canonical.get(c) != script[c])
    with_map = [c for c in REMAINING if c in after]
    check("q2-requests", not bad and voices == [SETTINGS["voice"]] and changed == CANONICAL_CHANGED,
          f"all 50 request01 files carry the stage05 script's exact text (= units text), its text and script "
          f"hashes, submitted false, and Frederick's settings (Eleven v4, 0.5/0.75/0, WAV 48 kHz); all "
          f"{len(every)} requests in the folder name Frederick and none Oliver; {len(with_map)} of the 50 have a "
          f"map override whose 'after' text equals the script; stage05 differs from the frozen canonical script "
          f"in {', '.join(changed)} only {bad}")
    sub = json.loads(Path(f"{TAKES}/09_5-submission01.json").read_text())
    obs = json.loads(Path(f"{TAKES}/09_5-submission-observation01.json").read_text())
    note(f"09_5: submission01 ({sub['at'][11:19]}Z) says {sub['observation'][:60]!r}...; observation01 "
         f"({obs['at'][11:19]}Z) supersedes it: {obs['state']!r}")
    header = Path(files["script"]).read_text().split("\n", 1)[0]
    scope = json.loads(Path(files["units"]).read_text())["scope"]
    note(f"provenance text, not a voice: the script header {'names' if 'ElevenLabs Oliver' in header else 'omits'} "
         f"'ElevenLabs Oliver', the units scope {'names' if 'ElevenLabs Oliver' in scope else 'omits'} it, "
         f"and the map file is {Path(files['map']).name}")
    if newer:
        note(f"NOTE q2: newer request files exist: {newer}")
    return with_map, after


# ------------------------------------------------------- 4 hooks and holds
def literal_consts(body):
    out = {}
    for m in re.finditer(r'const (\w+): \[&str; \d+\] = \[(.*?)\];', body, re.S):
        out[m.group(1)] = re.findall(r'"([^"]*)"', m.group(2))
    return out


def section_hooks(script, side, all_blocks, files):
    print("== 4. Hooks and holds: the bridge's source side against this check's own scan")
    mine = set()
    holds = {}
    helpers = []
    for name, b in all_blocks.items():
        consts = literal_consts(blocks(name)[1])
        for cue, sites in b.items():
            if cue not in REMAINING:
                continue
            (_k, lines), = sites
            text = "\n".join(lines)
            for m in re.finditer(r'(?:wait_for|until)\(stage, (?:"([^"]+)"|(\w+)\[\w+\])', text):
                for phrase in ([m.group(1)] if m.group(1) else consts.get(m.group(2), [])):
                    owner = cue
                    if phrase not in script[cue]:  # a helper defined below this block, called from another cue
                        owners = [c for c in script if phrase in script[c]]
                        owner = owners[0] if len(owners) == 1 else f"unowned:{phrase}"
                        helpers.append((cue, owner, phrase))
                    mine.add((owner, phrase))
            if re.search(r"predict\(&kit, stage, &line\b", text):
                holds[cue] = "prediction"
            if re.search(r"reading_window\(&kit, stage, &line\)", text):
                holds[cue] = "reading"
    anchors = [r for r in side["anchors"] if r["cue"] in REMAINING]
    theirs = {(r["cue"], r["phrase"]) for r in anchors}
    once = all(script[c].count(p) == 1 for c, p in theirs)
    mod = (Path(CRATE) / "src/chapters/mod.rs").read_text()
    table = {k: dict(re.findall(r'\("(\d\d_\d+)", ([\d.]+)\)', mod.split(f"pub const {k}")[1].split("];")[0]))
             for k in ("PREDICTS", "READING_WINDOWS")}
    mapdata = json.loads(Path(files["map"]).read_text())
    vh = {h["cue"]: h["kind"] for h in mapdata["visual_holds"] if h["cue"] in REMAINING}
    disp = {d["cue"] for d in side["dispositions"] if d["cue"] in REMAINING}
    seconds = {c: (table["PREDICTS"].get(c) or table["READING_WINDOWS"].get(c)) for c in vh}
    kinds_ok = all((k == "prediction") == (c in table["PREDICTS"]) and (k == "reading") == (c in table["READING_WINDOWS"])
                   for c, k in vh.items())
    declared = set(re.findall(r"^(?:pub )?mod (\w+);", mod, re.M))
    present = {p.stem for p in (Path(CRATE) / "src/chapters").glob("*.rs")} - {"mod"}
    undeclared = sorted(present - declared)
    hook_files = {h["file"].split("/")[-1][:-3] for r in side["anchors"] for h in r["hooks"]}
    ok = (mine == theirs and once and not side["anchor_problems"] and not side["disposition_problems"]
          and holds == vh and set(vh) == disp and kinds_ok and not (hook_files & set(undeclared)))
    check("k1-hooks-holds", ok,
          f"{len(theirs)} anchor phrases in {len({c for c, _ in theirs})} of the 50 cues; this check's scan of "
          f"each scene block finds the same {len(mine)} (cue, phrase) pairs; each phrase occurs once in its "
          f"cue; no anchor or hold problems; {len(vh)} holds ({sum(k == 'prediction' for k in vh.values())} "
          f"prediction, {sum(k == 'reading' for k in vh.values())} reading) where the block ends in predict() "
          f"or reading_window(), matching the map and mod.rs; none of the bridge's {len(side['anchors'])} anchors "
          f"(whole film) has a hook in the {len(undeclared)} chapter files mod.rs does not declare "
          f"({', '.join(undeclared)})")
    note("helper sites, below one cue's say() and owned by another cue's words: "
         + "; ".join(f"'{p}' below {c}, owned by {o}" for c, o, p in helpers))
    note("holds (p prediction, r reading; seconds from mod.rs): "
         + ", ".join(f"{c} {vh[c][0]}{seconds[c]}" for c in REMAINING if c in vh))
    if mine != theirs:
        note(f"only this scan: {sorted(mine - theirs)}; only the bridge: {sorted(theirs - mine)}")
    return {c: [p for cc, p in sorted(theirs) if cc == c] for c in REMAINING}


# ----------------------------------------------------------- 5 mathematics
def ring(n, a, r):
    return [a + r * cmath.exp(2j * math.pi * k / n) for k in range(n)]


def average(f, a, r, n=256, twist=0):
    return sum(f(z) * cmath.exp(-1j * twist * 2 * math.pi * k / n) for k, z in enumerate(ring(n, a, r))) / n


def contour(f, a, r, n=4096):
    """The loop integral of f around the circle |z - a| = r, counterclockwise (trapezoid rule)."""
    zs = ring(n, a, r)
    return sum(f(z) * (z - a) * 1j for z in zs) * (2 * math.pi / n)


def winding(f, a, r, n=4096):
    zs = ring(n, a, r) + [a + r]
    return round(sum(cmath.phase(f(zs[k + 1]) / f(zs[k])) for k in range(n)) / (2 * math.pi))


def bump(z):
    return 1 / (1 + z * z)


def limit_residue(f, p, eps=1e-6):
    return sum((e * f(p + e)) for e in (eps, -eps, 1j * eps, -1j * eps)) / 4


def simpson(f, a, b, n):
    h = (b - a) / n
    return h / 3 * (f(a) + f(b) + 4 * sum(f(a + (2 * k - 1) * h) for k in range(1, n // 2 + 1))
                    + 2 * sum(f(a + 2 * k * h) for k in range(1, n // 2)))


def quintic_roots():
    coef = [1, 0, 0, 0, -3, 1]
    p = lambda z: sum(c * z ** (5 - k) for k, c in enumerate(coef))
    zs = [(0.4 + 0.9j) ** k for k in range(5)]
    for _ in range(500):
        zs = [z - p(z) / math.prod(z - w for w in zs if w is not z) for z in zs]
    return zs, p


def bernoulli(n):
    b = [Fraction(1)]
    for m in range(1, n + 1):
        b.append(-sum(math.comb(m + 1, k) * b[k] for k in range(m)) / (m + 1))
    return b


def taylor_at(a, k):
    """The bump's k-th Taylor coefficient at the real point a, from 1/(1+z^2) = (1/2i)(1/(z-i) - 1/(z+i))."""
    return (-(1 / (2j)) * ((1j - a) ** -(k + 1) - (-1j - a) ** -(k + 1))).real


def mathematics():
    rows = []  # (cue, words in the cue, what was computed, value, spoken check ok, oracle key, oracle value)

    def row(cue, words, what, value, ok, key=None, printed=None):
        rows.append((cue, words, what, value, ok, key, printed))

    # 06_9: the reflection and the squash
    h = 1e-3
    conj_right = (complex(0.3 + h, 0.2).conjugate() - complex(0.3, 0.2).conjugate()) / h
    conj_up = (complex(0.3, 0.2 + h).conjugate() - complex(0.3, 0.2).conjugate()) / (1j * h)
    squash = lambda z: complex(z.real, 2 * z.imag)
    sq_up = (squash(complex(0.3, 0.2 + h)) - squash(complex(0.3, 0.2))) / (1j * h)
    row("06_9", "the ratio is one", "conj: step right", conj_right.real, abs(conj_right - 1) < 1e-12,
        "deriv.conj", f"{round(conj_right.real)},{round(conj_up.real)}")
    row("06_9", "the ratio is minus one", "conj: step up", conj_up.real, abs(conj_up + 1) < 1e-12)
    row("06_9", "squares become tall rectangles", "x + 2iy: step up / step right", sq_up.real,
        abs(sq_up - 2) < 1e-12, "deriv.squash", f"1,{round(sq_up.real)}")
    # 09_5: the mean value on r = 2 and each pole's share
    avg2 = average(bump, 0, 2.0)
    shares = [limit_residue(lambda z: bump(z) / z, p) for p in (1j, -1j)]
    row("09_5", "It is zero.", "average of the bump on |z| = 2", abs(avg2), abs(avg2) < 1e-9, "cauchy.r_two.limit",
        fixed(avg2.real, 0))
    row("09_5", "Each pole pulls the average down by one half.", "residue of bump(z)/z at i and -i",
        [round(s.real, 6) for s in shares], all(abs(s + 0.5) < 1e-6 for s in shares), "cauchy.pole_share",
        fixed(shares[0].real, 1))
    # 09_6: turned-back averages give the coefficients
    coef = [average(bump, 0, 0.5, twist=n) / 0.5 ** n for n in range(5)]
    slope = (bump(1e-4) - bump(-1e-4)) / 2e-4
    second = (bump(1e-3) - 2 * bump(0) + bump(-1e-3)) / 1e-6
    row("09_6", "That gives the slope at the centre.", "c1 against the bump's slope at 0", round(coef[1].real, 9),
        abs(coef[1] - slope) < 1e-6, "cauchy.coef", ",".join(f"{c.real:.3f}" for c in coef).replace("-0.000", "0.000"))
    row("09_6", "you get the next coefficient of the recipe", "c2 against f''(0)/2", round(coef[2].real, 9),
        abs(coef[2] - second / 2) < 1e-5)
    # 09_7 and 15_1: radius one, set by the poles
    root_test = abs(taylor_at(0.0, 40)) ** (-1 / 40)
    row("09_7", "so the recipe works out to radius one", "distance 0 to +-i; root test on c_40", root_test,
        abs(root_test - 1) < 1e-9 and min(abs(1j), abs(-1j)) == 1, "bump.radius0", fixed(root_test, 0))
    row("09_7", "since the bump blows up at i, no further", "|bump| at 0.999i", abs(bump(0.999j)),
        abs(bump(0.999j)) > 400)
    # 09_9: recentred at 1
    r1 = abs(1 - 1j)
    left, right = 1 - r1, 1 + r1
    at2 = sum(taylor_at(1.0, k) * 1.0 ** k for k in range(60))
    at26 = sum(taylor_at(1.0, k) * 1.6 ** k for k in range(60))
    row("09_9", "the square root of two away", "|1 - i|", r1, abs(r1 - math.sqrt(2)) < 1e-15, "recentre1.radius",
        f"{r1:.3f}")
    row("09_9", "from about minus point four one to about two point four one", "1 -+ sqrt 2, to 2 places",
        (round(left, 2), round(right, 2)), (round(left, 2), round(right, 2)) == (-0.41, 2.41),
        "recentre1.left,right", f"{left:.2f},{right:.2f}")
    row("09_9", "A computer check of sixty terms agrees.", "60 terms at x = 2 against bump(2); at 2.6",
        (round(at2, 6), round(at26)), abs(at2 - 0.2) < 1e-8 and abs(at26) > 100,
        "recentre1.check.at2,at2_6", f"{at2:.6f},{at26:.0f}")
    # 10_2: sine squared plus cosine squared at 3i
    s2, c2 = cmath.sin(3j) ** 2, cmath.cos(3j) ** 2
    row("10_2", "sine squared is about minus one hundred, cosine squared is about one hundred and one",
        "sin(3i)^2, cos(3i)^2", (round(s2.real, 2), round(c2.real, 2)),
        round(s2.real) == -100 and round(c2.real) == 101 and abs(s2.imag) + abs(c2.imag) < 1e-12,
        "rigid.trig", f"{s2.real:.2f},{c2.real:.2f}")
    row("10_2", "they still add to one", "sum", (s2 + c2).real, abs(s2 + c2 - 1) < 1e-12, "rigid.trig.sum",
        f"{(s2 + c2).real:.6f}")
    # 10_3: zero for x <= 0, smooth, every derivative zero at 0 (a check of the claim's two sides)
    smooth = lambda x: math.exp(-1 / x) if x > 0 else 0.0
    flat = [smooth(0.005) / 0.005 ** n for n in (1, 5, 10, 20)]
    row("10_3", "exactly zero for every negative x", "f on x <= 0", max(smooth(-x / 10) for x in range(0, 40)),
        all(smooth(-x / 10) == 0 for x in range(0, 40)))
    row("10_3", "every one is zero", "f(x)/x^n at x = 0.005 for n = 1, 5, 10, 20 (flatter than any power)",
        f"{max(flat):.1e}", max(flat) < 1e-30 and smooth(1.0) > 0.3)
    # 10_4: the zeros of sin(1/z)
    zs = [1 / (k * math.pi) for k in (1, 2, 3)]
    row("10_4", "one over pi, one over two pi, one over three pi", "1/(k pi) and |sin(1/z)| there",
        [round(z, 3) for z in zs], all(abs(cmath.sin(1 / z)) < 1e-12 for z in zs), "rigid.sin_recip.zeros",
        ",".join(f"{z:.3f}" for z in zs))
    # 10_6: sine bounded on the line, huge off it
    s3 = cmath.sin(3j)
    row("10_6", "must grow huge somewhere else", "max |sin x| on the line; sin(3i)", (1.0, round(s3.imag, 1)),
        max(abs(math.sin(x / 100)) for x in range(-700, 700)) <= 1 and abs(s3) > 10, "gallery.sin.at_3i",
        f"{s3.real:.1f},{s3.imag:.1f}")
    # 11_1, 11_2: turns around zeros and poles
    local = [winding(f, 0, 0.5) for f in (lambda z: z, lambda z: z * z, lambda z: 1 / z)]
    several = winding(lambda z: (z - 1) / (z + 1) ** 2, 0, 1.6)
    row("11_1", "one full turn ... makes two ... one turn the other way", "turns of z, z^2, 1/z", local,
        local == [1, 2, -1], "argument.local", ",".join(map(str, local)))
    row("11_2", "the zeros inside minus the poles inside, each counted by its order",
        "turns of (z-1)/(z+1)^2 on |z| = 1.6", several, several == 1 - 2, "argument.several.turns", str(several))
    # 11_3, 11_4: the counting example (z - 0.45)/(z - 1.35)
    mob = lambda z: (z - 0.45) / (z - 1.35)
    t08, t17 = winding(mob, 0, 0.8), winding(mob, 0, 1.7)
    row("11_3", "encloses only the zero, and the output turns once", "turns on |z| = 0.8", t08, t08 == 1,
        "argument.rational.r0_8", str(t08))
    row("11_3", "so the count is zero", "turns on |z| = 1.7", t17, t17 == 0, "argument.rational.r1_7", str(t17))
    row("11_4", "the loop runs through the pole", "|f| at 1.35 - 1e-9 on |z| = 1.35", abs(mob(1.35 - 1e-9)),
        abs(mob(1.35 - 1e-9)) > 1e8)
    # 11_5, 11_6, 11_7: z^5 - 3z + 1
    roots, quint = quintic_roots()
    inside = sorted(abs(z) for z in roots if abs(z) < 1)
    leash1 = max(abs(z ** 5 + 1) for z in ring(4096, 0, 1))
    row("11_5", "has five roots", "|p| at five roots", max(abs(quint(z)) for z in roots),
        max(abs(quint(z)) for z in roots) < 1e-12 and len({round(z.real, 6) + 1j * round(z.imag, 6) for z in roots}) == 5)
    row("11_6", "One.", "roots inside |z| < 1; smallest |root|", (len(inside), round(inside[0], 3)),
        len(inside) == 1, "argument.quintic.unit,smallest", f"{winding(quint, 0, 1)},{inside[0]:.3f}")
    row("11_6", "always bigger than the other two terms combined", "max |z^5 + 1| on |z| = 1 against 3",
        round(leash1, 9), leash1 <= 2 + 1e-12)
    walker = 3 ** 5
    leash3 = max(abs(-3 * z + 1) for z in ring(4096, 0, 3))
    row("11_7", "so the dog circles zero n times too", "turns on |z| = 3; walker 3^5 against longest leash",
        (winding(quint, 0, 3), walker, round(leash3, 6)),
        winding(quint, 0, 3) == 5 and leash3 < walker and all(abs(z) < 3 for z in roots),
        "argument.quintic.big,walker,leash", f"{winding(quint, 0, 3)},{walker},{leash3:.0f}")
    lead = 2j
    row("11_7", "Divide by its leading coefficient, which leaves its roots unchanged", "|2i p(z)| at the roots",
        max(abs(lead * quint(z)) for z in roots), max(abs(lead * quint(z)) for z in roots) < 1e-11)
    # 11_8: z^8 on the unit circle, eight steps and sixty four
    f8 = lambda z: z ** 8
    def stepped(n):
        zs = ring(n, 0, 1) + [1]
        turns = sum(cmath.phase(f8(zs[k + 1]) / f8(zs[k])) for k in range(n)) / (2 * math.pi)
        reach = 8 * (2 * math.pi / n)
        return round(turns), reach, all(reach < abs(f8(z)) for z in zs)
    eight, sixty_four = stepped(8), stepped(64)
    row("11_8", "samples alone can miss a turn made quickly between them", "8 steps: turns read, reach, passes",
        (eight[0], round(eight[1], 2), eight[2]), eight[0] == 0 and not eight[2], "argument.coarse.naive,reach",
        f"{eight[0]},{eight[1]:.2f},{sixty_four[1]:.2f}")
    row("11_8", "the count is certain", "64 steps: turns read, reach, passes", (sixty_four[0], round(sixty_four[1], 2),
        sixty_four[2]), sixty_four == (8, sixty_four[1], True), "argument.coarse.certified", str(sixty_four[0]))
    # 12_1 to 12_4: residues
    p0 = 0.3 + 0.2j
    loops = [contour(lambda z: (z - p0) ** -2, p0, 0.4), contour(lambda z: 1 / (z - p0), p0, 0.4),
             contour(lambda z: 1 + (z - p0) / 2, p0, 0.4)]
    row("12_1", "all of those terms integrate to zero, except one over z minus p", "loops of (z-p)^-2, (z-p)^-1, h",
        [round(abs(x), 9) for x in loops], abs(loops[0]) < 1e-9 and abs(loops[1] - 2j * math.pi) < 1e-9
        and abs(loops[2]) < 1e-9)
    near_i = contour(bump, 1j, 0.5)
    row("12_2", "two pi i times the sum of the residues inside", "loop of the bump around i only, against 2 pi i/(2i)",
        round(near_i.real, 9), abs(near_i - 2j * math.pi / (2j)) < 1e-9)
    gh = 1 / (2 * 1j)
    row("12_2", "the residue is g over the derivative of h", "g/h' at i against the limit (z - i) bump(z)",
        (gh.real, gh.imag), abs(limit_residue(bump, 1j) - gh) < 1e-6)
    logs = []
    for f, df in ((lambda z: z, lambda z: 1), (lambda z: z * z, lambda z: 2 * z), (lambda z: 1 / z, lambda z: -1 / z ** 2)):
        logs.append(round((contour(lambda z: df(z) / f(z), 0, 0.5) / (2j * math.pi)).real))
    row("12_3", "f prime over f has residue m ... its residue is minus n", "residues of f'/f for z, z^2, 1/z", logs,
        logs == [1, 2, -1])
    row("12_4", "So the residue of the bump at i is one over two i", "limit (z - i) bump(z)",
        (round(limit_residue(bump, 1j).real, 6), round(limit_residue(bump, 1j).imag, 6)),
        abs(limit_residue(bump, 1j) - 1 / (2j)) < 1e-6, "residue.bump.at_i",
        f"{fixed(limit_residue(bump, 1j).real, 1)},{fixed(limit_residue(bump, 1j).imag, 1)}")
    # 13_1 to 13_3: the bump's area
    area = 2 * (simpson(lambda x: 1 / (1 + x * x), 0, 1000, 200000) + 1 / 1000 - 1 / (3 * 1000 ** 3))
    row("13_1", "the area is pi", "integral over the line (Simpson on [0, 1000] plus tails)", round(area, 9),
        abs(area - math.pi) < 1e-8)
    loop = 2j * math.pi * (1 / (2j))
    row("13_2", "That is exactly pi.", "2 pi i * 1/(2i)", loop.real, abs(loop - math.pi) < 1e-15,
        "integral.bump.loop", f"{loop.real:.5f}")
    arc_rows = []
    for R in (2, 5, 10):
        zs = [R * cmath.exp(1j * math.pi * k / 2000) for k in range(2001)]
        low = min(abs(1 + z * z) for z in zs)
        actual = abs(sum(bump(zs[k]) * (zs[k + 1] - zs[k]) for k in range(2000)))
        arc_rows.append((R, low, math.pi * R / (R * R - 1), actual))
    row("13_3", "the bump is at most one over R squared minus one", "min |1 + z^2| on each half circle against R^2 - 1",
        [round(lo - (R * R - 1), 9) for R, lo, _b, _a in arc_rows], all(lo >= R * R - 1 - 1e-9 for R, lo, _b, _a in arc_rows))
    row("13_3", "which shrinks to zero as R grows", "pi R/(R^2 - 1) at R = 2, 5, 10; the arc's integral is below it",
        [round(b, 3) for _R, _lo, b, _a in arc_rows], all(a <= b + 1e-9 for _R, _lo, b, a in arc_rows)
        and arc_rows[0][2] > arc_rows[1][2] > arc_rows[2][2], "integral.bump.arc",
        ",".join(f"{b:.3f}" for _R, _lo, b, _a in arc_rows))
    # 13_4: the antiderivative the picture shows
    s = math.sqrt(2)
    anti = lambda x: (math.log((x * x + s * x + 1) / (x * x - s * x + 1)) / (4 * s)
                      + (math.atan(s * x + 1) + math.atan(s * x - 1)) / (2 * s))
    worst = max(abs((anti(x + 1e-5) - anti(x - 1e-5)) / 2e-5 - 1 / (1 + x ** 4)) for x in (-3, -1, -0.3, 0.2, 0.7, 2.5))
    row("13_4", "Its antiderivative exists, but it looks like this.", "the card's formula differentiated, worst error",
        f"{worst:.1e}", worst < 1e-8)
    # 13_5 to 13_9: the quartic
    ps = [cmath.exp(1j * math.pi * (2 * k + 1) / 4) for k in range(4)]
    degs = [round(math.degrees(cmath.phase(p)) % 360) for p in ps]
    row("13_5", "forty five, one hundred thirty five, two hundred twenty five and three hundred fifteen degrees",
        "angles of the roots of z^4 = -1", degs, degs == [45, 135, 225, 315] and all(abs(p ** 4 + 1) < 1e-12 for p in ps),
        "integral.quartic.roots_deg", ",".join(map(str, degs)))
    upper = [p for p in ps if p.imag > 0]
    row("13_6", "The two above the real line.", "roots with Im > 0", len(upper), len(upper) == 2,
        "integral.quartic.inside", str(len(upper)))
    q = lambda z: 1 / (1 + z ** 4)
    res = [limit_residue(q, p) for p in upper]
    row("13_7", "one over four p cubed ... minus p over four", "limit residues against 1/(4p^3) and -p/4",
        max(abs(r - 1 / (4 * p ** 3)) + abs(r + p / 4) for r, p in zip(res, upper)),
        all(abs(r - 1 / (4 * p ** 3)) < 1e-6 and abs(r + p / 4) < 1e-6 for r, p in zip(res, upper)))
    total = sum(res)
    row("13_8", "The real parts of p one and p two cancel, and each has imaginary part the square root of two over two",
        "p1 + p2; Im p", (round((upper[0] + upper[1]).real, 12), round(upper[0].imag, 6)),
        abs((upper[0] + upper[1]).real) < 1e-12 and all(abs(p.imag - s / 2) < 1e-12 for p in upper))
    row("13_8", "minus i times the square root of two, over four", "sum of the residues", (round(total.real, 5),
        round(total.imag, 5)), abs(total + 1j * s / 4) < 1e-6, "integral.quartic.sum",
        f"{fixed(total.real, 5)},{fixed(total.imag, 5)}")
    value = (2j * math.pi * (-1j * s / 4)).real
    row("13_9", "the area is pi over the square root of two. About two point two two", "2 pi i * (-i sqrt2/4)",
        round(value, 4), abs(value - math.pi / s) < 1e-12 and round(value, 2) == 2.22,
        "integral.quartic.value,value2", f"{value:.4f},{value:.2f}")
    # 13_10, 13_11
    qarc = [math.pi * R / (R ** 4 - 1) for R in (2, 10)]
    numeric = 2 * simpson(q, 0, 200, 200000)
    row("13_10", "At R equals ten, that is already less than one three hundredth", "pi R/(R^4 - 1) at 2 and 10",
        [round(x, 4) for x in qarc], qarc[1] < 1 / 300, "integral.quartic.arc", ",".join(f"{x:.4f}" for x in qarc[:1])
        + f",{qarc[1]:.4f}")
    row("13_10", "A numerical sum along the real line agrees with pi over the square root of two",
        "Simpson on [-200, 200]", round(numeric, 4), round(numeric, 4) == round(math.pi / s, 4),
        "integral.quartic.numeric", f"{numeric:.4f}")
    row("13_11", "The real integral converges on its own", "integral of |f| over [-200, 200] plus both tails",
        round(numeric + 2 / (3 * 200 ** 3), 6), abs(numeric + 2 / (3 * 200 ** 3) - math.pi / s) < 1e-6)
    # 14_2: the geometric series
    half = sum(Fraction(1, 2) ** k for k in range(11))
    onetwo = sum(Fraction(6, 5) ** k for k in range(11))
    row("14_2", "At one half, the sums settle on two.", "11 terms at 1/2; the limit", float(half),
        2 - half == Fraction(1, 1024), "series.half.n10", f"{float(half):.6f}")
    row("14_2", "At one point two, they grow without limit.", "11 terms at 1.2; 41 terms", (round(float(onetwo), 2),
        round(float(sum(Fraction(6, 5) ** k for k in range(41))))), float(onetwo) > 30, "series.one_two.n10",
        f"{float(onetwo):.2f}")
    row("14_2", "It equals minus five.", "1/(1 - 1.2)", 1 / (1 - Fraction(6, 5)), 1 / (1 - Fraction(6, 5)) == -5,
        "series.one_two.function", str(1 / (1 - Fraction(6, 5))))
    # 14_3: the first re-centred disk
    centre = -0.5
    row("14_3", "its new disk pokes outside the old one", "re-centred at -0.5: radius to the pole at 1; farthest |z|",
        (abs(1 - centre), abs(centre) + abs(1 - centre)), abs(centre) + abs(1 - centre) > 1,
        "continuation.first_disk.radius", f"{abs(1 - centre):.1f}")
    # 14_5: no sum; the values
    b = bernoulli(4)
    zeta_m1 = -b[2] / 2
    row("14_5", "The partial sums grow without limit.", "11 terms of 1 + 2 + 4 + ...", 2 ** 11 - 1,
        sum(2 ** k for k in range(11)) == 2047, "series.doubling.n10", str(sum(2 ** k for k in range(11))))
    row("14_5", "The continued function equals minus one at two", "1/(1 - 2)", 1 / (1 - 2), 1 / (1 - 2) == -1,
        "continuation.two", fixed(1 / (1 - 2), 0))
    row("14_5", "Minus one twelfth is the value of the continued zeta function at minus one",
        "zeta(-1) = -B_2/2 (Bernoulli)", str(zeta_m1), zeta_m1 == Fraction(-1, 12), "continuation.zeta_minus1",
        f"{float(zeta_m1):.6f}")
    # 14_6: the square root and the logarithm carried round 0
    w, log = 1 + 0j, 0j
    zs = ring(4096, 0, 1) + [1]
    for k in range(4096):
        r = cmath.sqrt(zs[k + 1])
        w = r if abs(r - w) < abs(-r - w) else -r
        log += cmath.log(zs[k + 1] / zs[k])
    row("14_6", "When you come back, it has become minus one.", "sqrt carried once round counterclockwise",
        (round(w.real, 9), round(w.imag, 9)), abs(w + 1) < 1e-9, "continuation.sqrt.end",
        f"{fixed(w.real, 0)},{fixed(w.imag, 0)}")
    row("14_6", "gaining two pi i each time around", "log carried once round counterclockwise", round(log.imag, 9),
        abs(log - 2j * math.pi) < 1e-9)
    # 14_7: z + z^2 + z^4 + ...: unbounded towards every 2^m-th root of 1
    lac = lambda z, n: sum(z ** (2 ** k) for k in range(n))
    toward = [abs(lac(r * cmath.exp(2j * math.pi / 8), 60)) for r in (0.9, 0.99, 0.999, 0.99999)]
    row("14_7", "near a dense set of points on the edge it blows up",
        "|sum| toward the 8th root of 1 at r = 0.9, 0.99, 0.999, 0.99999",
        [round(t, 2) for t in toward], all(a < b for a, b in zip(toward, toward[1:])) and toward[-1] > 10)
    # 14_8: gamma
    steps = [abs(math.gamma(x) - math.gamma(x + 1) / x) / abs(math.gamma(x)) for x in (0.5, -0.5, -1.5, -2.5)]
    poles = []
    for x in (0.0, -1.0, -2.0, -3.0, -4.0):
        try:
            math.gamma(x)
        except ValueError:
            poles.append(x)
    row("14_8", "gamma of z equals gamma of z plus one, divided by z", "relative error at 0.5, -0.5, -1.5, -2.5",
        f"{max(steps):.1e}", max(steps) < 1e-13)
    row("14_8", "each step adds a new pole, at zero, minus one, minus two", "math.gamma refuses", poles,
        poles == [0.0, -1.0, -2.0, -3.0, -4.0], "gallery.gamma.poles", ",".join(fixed(p, 0) for p in poles))
    # 15_4: recentred at 3
    r3 = abs(3 - 1j)
    row("15_4", "the square root of ten away. About three point one six in each direction", "|3 - i|; 3 -+ |3 - i|",
        (round(r3, 3), round(3 - r3, 2), round(3 + r3, 2)), abs(r3 * r3 - 10) < 1e-12 and round(r3, 2) == 3.16,
        "recentre3.radius,ends", f"{r3:.3f},{3 - r3:.2f},{3 + r3:.2f}")
    return rows


ORACLE_KEYS = {  # this check's key name -> the oracle's printed keys, joined in order
    "recentre1.left,right": ["recentre1.left", "recentre1.right"],
    "recentre1.check.at2,at2_6": ["recentre1.check.at2", "recentre1.check.at2_6"],
    "argument.quintic.unit,smallest": ["argument.quintic.unit", "argument.quintic.smallest"],
    "argument.quintic.big,walker,leash": ["argument.quintic.big", "argument.quintic.walker", "argument.quintic.leash"],
    "argument.coarse.naive,reach": ["argument.coarse.naive", "argument.coarse.reach"],
    "integral.quartic.value,value2": ["integral.quartic.value", "integral.quartic.value2"],
    "recentre3.radius,ends": ["recentre3.radius", "recentre3.ends"],
}


def section_mathematics(script):
    print("== 5. Mathematics: every spoken number and claim, recomputed (standard library only)")
    run = subprocess.run([sys.executable, "-I", "-B", f"{CRATE}/oracle.py"], capture_output=True, text=True)
    facts = dict(line.split("\t", 1) for line in run.stdout.splitlines() if "\t" in line)
    rows = mathematics()
    bad = []
    for cue, words, what, value, ok, key, printed in rows:
        in_text = all(part.strip() in script[cue] for part in words.split("..."))
        oracle = ",".join(facts[k] for k in ORACLE_KEYS.get(key, [key])) if key else None
        agrees = key is None or oracle == printed
        if not (ok and in_text and agrees):
            bad.append((cue, words, ok, in_text, oracle, printed))
        print(f"   {'ok ' if ok and in_text and agrees else 'BAD'} {cue} \"{words}\": {what} = {value}"
              + (f"; oracle {key} = {oracle}" if key else ""))
    keyed = sum(1 for r in rows if r[5])
    keys = sorted({k for r in rows if r[5] for k in ORACLE_KEYS.get(r[5], [r[5]])})
    scenes = "".join((Path(CRATE) / "src/chapters" / n).read_text() for n in sorted(set(CHAPTER_FILES.values())))
    unread = [k for k in keys if f'"{k}"' not in scenes]
    cues = sorted({r[0] for r in rows})
    check("m1-mathematics", run.returncode == 0 and not bad and unread == ["integral.quartic.value"],
          f"{len(rows)} spoken numbers and claims in {len(cues)} of the 50 cues recomputed with cmath, math and "
          f"fractions; each one's words are in its cue and each value matches what is said; {keyed} of them "
          f"carry facts keys ({len(keys)} keys), root's oracle prints the same value for every one ({len(facts)} "
          f"facts, exit {run.returncode}), and these chapters' scenes read {len(keys) - len(unread)} of the "
          f"{len(keys)} keys (not {', '.join(unread)}) {bad}")
    rest = [c for c in REMAINING if c not in cues]
    note(f"cues with no number or checkable claim beyond the picture's: {', '.join(rest)}")


# --------------------------------------------------------- 6 spoken notation
def section_notation(script):
    print("== 6. Spoken notation, and the words that drew content questions in the completed takes")
    texts = {c: script[c] for c in REMAINING}
    odd = sorted({ch for t in texts.values() for ch in t if not (ch.isalpha() or ch in " .,?'")})
    digits = [c for c, t in texts.items() if re.search(r"\d", t)]
    hyphen = sorted({w for t in texts.values() for w in re.findall(r"\b\w+-\w+\b", t)})
    review = json.loads(Path(f"{P}/complex-frederick-current-timing-review01/candidates.json").read_text())
    done = sorted({r["id"].split(":")[1] for r in review["late_points"] + review["other_points"]})
    asked = {}
    for r in review["content"]:
        for d in r["differences"]:
            asked.setdefault(d["script"], set()).add((r["cue"], r["class"], d["raw"]))
    counts = []
    for phrase, seen in sorted(asked.items()):
        # a word heard as another word counts as the word; notation counts in the form it had before
        pat = re.compile(FORMS.get(phrase, r"\b" + re.escape(phrase) + r"\b"), re.I)
        hits = {c: len(pat.findall(t)) for c, t in texts.items() if pat.search(t)}
        if hits:
            counts.append((phrase, sorted(seen), sum(hits.values()), hits))
    known = " ".join(script[c] for c in done)
    names = ["Riemann", "Cauchy's", "Liouville's", "Gamma's", "gamma", "zeta"]
    name_rows = [(n, [c for c, t in texts.items() if re.search(r"\b" + n + r"\b", t)],
                  bool(re.search(r"\b" + n.rstrip("'s") + r"\b", known))) for n in names]
    check("n1-notation", not digits and odd == ["-"] and hyphen == ["Re-centre"] and len(done) == 47,
          f"no digit or symbol in the 50 texts; the only character beyond letters and . , ? ' is the hyphen of "
          f"{hyphen}; {len(counts)} script phrases that drew content questions in the 47 processed cues recur "
          f"here {sum(n for _p, _s, n, _h in counts)} times")
    for phrase, seen, n, hits in counts:
        classes = sorted({k for _c, k, _r in seen})
        note(f"'{phrase}' as /{FORMS.get(phrase, 'the word')}/ ({'/'.join(classes)} before; raw "
             f"{sorted({r for _c, _k, r in seen})}): {n}x in "
             + ", ".join(f"{c}{'x' + str(k) if k > 1 else ''}" for c, k in hits.items()))
    for n, where, seen in name_rows:
        note(f"name '{n}': {', '.join(where) or '-'}; in a processed cue before: {'yes' if seen else 'no'}")


# ----------------------------------------------------------- 7 findings
def section_findings(script, bridge, hooks, after):
    print("== 7. Findings: the evidence, and what each proposed wording keeps")
    bron = bridge.dep("log_preparation")
    missing = [(c, t) for c, t in EVIDENCE if t not in script[c]]
    src = {}
    for name, text in SOURCE_EVIDENCE:
        body = (Path(CRATE) / "src/chapters" / name).read_text()
        src[(name, text)] = body.count(text)
    bad_src = [k for k, n in src.items() if n < 1]
    b = bernoulli(5)
    zeta_m2, zeta_m4 = b[3] / 3, b[5] / 5  # zeta(-n) = (-1)^n B_{n+1}/(n+1); n = 2, 4
    lines = [1, 1j]
    images = [z * z for z in lines]
    angle_in = abs(cmath.phase(lines[1] / lines[0]))
    angle_out = abs(cmath.phase(images[1] / images[0]))
    check("f1-evidence", not missing and not bad_src and zeta_m2 == 0 and zeta_m4 == 0
          and abs(angle_out - 2 * angle_in) < 1e-12,
          f"the {len(EVIDENCE)} quoted phrases are in their cues and the {len(SOURCE_EVIDENCE)} quoted source lines "
          f"in their chapter files; zeta(-2) = zeta(-4) = 0 off the line Re s = 1/2 (Bernoulli); squaring at 0 "
          f"turns a right angle into a straight one ({math.degrees(angle_in):.0f} -> {math.degrees(angle_out):.0f} "
          f"degrees) {missing} {bad_src}")
    rows = []
    for cue, (old, new) in PROPOSALS.items():
        text = script[cue]
        proposed = text.replace(old, new)
        kept = all(proposed.count(p) == 1 for p in hooks[cue])
        same = len(bron.kit_sentences(proposed)) == len(bron.kit_sentences(text))
        words = len(proposed.split()) - len(text.split())
        rows.append((cue, text.count(old) == 1, kept, same, words, cue in after))
        note(f"{cue}: {len(hooks[cue])} hook(s) {hooks[cue]} kept: {kept}; sentences {len(bron.kit_sentences(text))} "
             f"-> {len(bron.kit_sentences(proposed))}; words {words:+d}; map override text to change too: "
             f"{'yes' if cue in after else 'no'}")
        note(f"   proposed: {proposed}")
    check("f2-proposals", all(r[1] and r[2] and r[3] for r in rows),
          "each proposal replaces one exact span, keeps every anchor phrase once and keeps the sentence count, "
          "so no hook moves to another sentence")


# -------------------------------------------------------------- 8 hygiene
def section_hygiene():
    here = Path(__file__).resolve()
    if here.parent.parent != (Path.cwd() / BUNDLE).resolve():
        return
    print("== 8. Hygiene")
    root = Path(BUNDLE)
    names = sorted(str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()
                   and "__pycache__" not in p.relative_to(root).parts)
    want = ["FINDINGS.md", "checks/outputs/remaining_cues_review.txt", "checks/remaining_cues_review.py"]
    machine = re.compile(r"/Users/|/private/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    dirty = [n for n in want if (root / n).exists() and any(
        (machine.search(x) or pronoun.search(x)) and not x.endswith("# hygiene-pattern")
        for x in (root / n).read_text(errors="replace").splitlines())]
    check("h1-hygiene", names in (want, sorted(want + ["SHA256SUMS"])) and not dirty,
          f"exactly the {len(want)} bundle files (and SHA256SUMS once sealed); no machine path, user name, "
          f"address or gendered pronoun {names if names not in (want, sorted(want + ['SHA256SUMS'])) else ''}{dirty}")


def main():
    if len(sys.argv) != 1:
        print(__doc__)
        return 2
    try:
        bridge, intake, script, files, side = section_pins()
    except Exception as exc:  # this check's PinMoved, or the bridge's or intake's own
        if type(exc).__name__ != "PinMoved":
            raise
        print(f"PIN MOVED: {exc}")
        return 3
    all_blocks = section_registry(script)
    _with_map, after = section_requests(script, files)
    hooks = section_hooks(script, side, all_blocks, files)
    section_mathematics(script)
    section_notation(script)
    section_findings(script, bridge, hooks, after)
    section_hygiene()
    print(f"RESULT {'FAIL' if failures else 'PASS'}, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
