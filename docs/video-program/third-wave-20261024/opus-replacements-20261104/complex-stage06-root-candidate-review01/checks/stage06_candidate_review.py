#!/usr/bin/env python3
"""complex-stage06-root-candidate-review01: the check behind REVIEW.md.

Independent read-only review of root's stage06 candidate (Mails 4620, 4632):
EC/combined-repair-stage06-root01, seal 44cb2a3f, against stage05, and of the
two later source deltas in the native-stage06-root03 build workspace (the zeta
branch near 0 and the oracle's exterior Cauchy average). Run from the
repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage06-root-candidate-review01/checks/stage06_candidate_review.py

It writes nothing. It runs the pinned bridge's source side on root's candidate
in place, with the candidate's own hashes swapped in memory, and runs the
candidate's and root03's oracle.py once each, as root's facts-check does.
Sections: 1 pins, 2 file scope, 3 script and storyboard, 4 units and map,
5 hooks and holds, 6 the five corrections and their mathematics, 7 root03
deltas, 8 requests and takes (text only), 9 hygiene (from the bundle only).

Exit 0 PASS, 1 FAIL, 2 usage, 3 an input moved. Nothing here builds, renders,
listens, decodes, synthesizes or submits.
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
TOPIC = "complex_analysis_visual"
BASE = f"{EC}/combined-repair-stage05/explainer_corpus/{TOPIC}"
SEALED = f"{EC}/combined-repair-stage06-root01"
CAND = f"{SEALED}/explainer_corpus/{TOPIC}"
WORK = f"{EC}/native-stage06-root03/workspace/explainer_corpus/{TOPIC}"
NATIVE05 = f"{EC}/native-stage05/crate"
TAKES5 = f"{EC}/frederick-stage05-01"
TAKES6 = f"{EC}/frederick-stage06-01"
BRIDGE_DIR = f"{P}/complex-frederick-timing-bridge-repair01"
BUNDLE = f"{P}/complex-stage06-root-candidate-review01"
OLD_MAP = "docs/narration-direction-map-oliver12.json"
NEW_MAP = "docs/narration-direction-map-frederick-stage06.json"

PINS = {
    f"{SEALED}/SHA256SUMS": "44cb2a3fc1aadc44fb8599a88b794368915c5fd64f588d34d81bd570be497001",
    f"{BASE}/script.tsv": "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3",
    f"{BASE}/narration_units.json": "e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6",
    f"{BASE}/{OLD_MAP}": "70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c",
    f"{BASE}/docs/storyboard.md": "e204cf7726dadefe46dccb419e0cc0a641acbeeb4e884c3f55ff6d39a980536b",
    f"{BASE}/src/chapters/closing.rs": "373a0340a0f86d1bd3c899e218a7ea7c1f7993e1a4914a2d6a1f238c27d5b93e",
    f"{BASE}/src/model.rs": "aa4bdcd9afcf19b7741eb9df5862001c61f7f0ba9e28affe8032c63755523f4d",
    f"{BASE}/oracle.py": "3d661ca5471cc0484e7e574d9751e54397c762be7efbbd70add65f81b509b794",
    f"{WORK}/src/model.rs": "57981602dee6540b661ca3cbdf099502bc3641bc43085aa88cb2be81a327d251",
    f"{WORK}/oracle.py": "2a596b38c0b3ffcea797c39b801926451d892cb2a44e938f262b37b2ecf6e82d",
    f"{SEALED}/root-correction-report01.json": "e662a4ad0b83e2428fd1b9209b9778ce8a22b67a4fb00f24bb2fb7d441b0d309",
    f"{EC}/native-stage06-root03/integration01.json": "26c5982bb3556a86da2cfd650d2f1123ea9b78b1d5c924b8c87a9d54497aa3bc",
    f"{EC}/native-stage06-root02/oracle-cauchy-correction01.json":
        "83e29563cd652bfb89aceee4d06c55198f86f776e3b9d1da7c4069af5e9470e9",
    f"{BRIDGE_DIR}/frederick_timing_bridge.py": "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39",
    f"{BRIDGE_DIR}/SHA256SUMS": "d5a413dd5856d91338c3f3c248bd194cc7feb80a9a9d635a65043a904c3b025d",
    "tools/storyboard.py": "3faed8c1ded451c432eddfcdd8d2034a7244594f3f99d753a63dc47e487cb0b7",
}
# what root's candidate says it changed (Mail 4620), and the span each correction replaced
CHANGED_FILES = ["docs/storyboard.md", "narration_units.json", "script.tsv", "src/chapters/closing.rs"]
CORRECTIONS = {  # cue: (finding of review01, old span, new span)
    "05_8": ("F1", "No zero found so far sits off that line. Whether all of them lie on it",
             "Apart from those at minus two, minus four, and on, no zero found so far sits off that line. "
             "Whether all the rest lie on it"),
    "10_3": ("F3", "misses the curve completely.", "misses the whole rise."),
    "14_1": ("F5", "We know it everywhere.", "We know it everywhere except at i and minus i."),
    "14_3": ("F4", "Re-centre the series", "Recentre the series"),
    "15_2": ("F2", "On an open region, that makes", "On an open region, away from critical points, that makes"),
}
FREDERICK = {"voice": "Frederick Surrey - Smooth and Velvety", "model": "Eleven v4", "stability": 0.5,
             "similarity": 0.75, "clone_strength": 0, "output": "WAV 48 kHz Lossless"}

failures = []


class PinMoved(Exception):
    pass


def check(name, ok, detail):
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")
    if not ok:
        failures.append(name)


def note(text):
    print(f"   {text}")


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def tree(root):
    root = Path(root)
    return {str(p.relative_to(root)): sha_file(p) for p in sorted(root.rglob("*"))
            if p.is_file() and not {"__pycache__", "target"} & set(p.relative_to(root).parts)}


def script_rows(path):
    rows = [line.split("\t", 1) for line in Path(path).read_text().splitlines() if line and not line.startswith("#")]
    return [r[0] for r in rows], dict(rows)


def fixed(x, places):
    s = f"{x:.{places}f}"
    return s[1:] if s.startswith("-") and set(s[1:]) <= set("0.") else s


# ------------------------------------------------------------------ 1 pins
def section_pins():
    print("== 1. Pins")
    moved = [p for p, s in PINS.items() if sha_file(p) != s]
    sums = [line.split("  ", 1) for line in Path(f"{SEALED}/SHA256SUMS").read_text().splitlines() if line]
    bad = [n for s, n in sums if sha_file(f"{SEALED}/{n}") != s]
    if moved or bad:
        raise PinMoved(f"moved: {moved} {bad}")
    bridge = load("frederick_timing_bridge", f"{BRIDGE_DIR}/frederick_timing_bridge.py")
    intake = bridge.dep("intake")
    check("p1-pins", True,
          f"root's stage06 seal 44cb2a3f verifies ({len(sums)} files); the stage05 files it changed, stage05 and "
          f"root03 model.rs and oracle.py, root's three records, the bridge 47069583 and tools/storyboard.py are at "
          f"their pins")
    return bridge, intake, sums


# ------------------------------------------------------------- 2 file scope
def section_scope(sums):
    print("== 2. File scope")
    base, cand, work = tree(BASE), tree(CAND), tree(WORK)
    changed = sorted(k for k in base if k in cand and base[k] != cand[k])
    added, gone = sorted(set(cand) - set(base)), sorted(set(base) - set(cand))
    sealed = sorted(n.split(f"explainer_corpus/{TOPIC}/", 1)[1] for _s, n in sums if n.startswith("explainer_corpus/"))
    later = sorted(k for k in cand if k in work and cand[k] != work[k])
    cargo = all(sha_file(f"{WORK}/{f}") == sha_file(f"{NATIVE05}/{f}") for f in ("Cargo.toml", "Cargo.lock"))
    check("s1-scope", changed == CHANGED_FILES and added == [NEW_MAP] and not gone and sealed == sorted(cand)
          and set(work) == set(cand) and later == ["Cargo.lock", "Cargo.toml", "oracle.py", "src/model.rs"] and cargo,
          f"candidate against stage05: {len(changed)} files changed ({', '.join(changed)}), 1 added ({NEW_MAP}), none "
          f"removed, the other {len(base) - len(changed)} byte-equal, and the seal lists exactly the {len(cand)} crate "
          f"files; the root03 build workspace equals the candidate except {', '.join(later)}, and its Cargo files "
          f"are native-stage05's reviewed copies")
    hook_state = [k for k in cand if k.startswith(".ee/")]
    note(f"inside the sealed crate since stage05 (byte-equal there): {hook_state}, a session-start hook file, "
         f"unused by the build")


# ---------------------------------------------------- 3 script and storyboard
def spans(old, new):
    a = 0
    while a < min(len(old), len(new)) and old[a] == new[a]:
        a += 1
    b = 0
    while b < min(len(old), len(new)) - a and old[-1 - b] == new[-1 - b]:
        b += 1
    return old[a:len(old) - b], new[a:len(new) - b]


def section_script(bron):
    print("== 3. Script and storyboard")
    ids0, s0 = script_rows(f"{BASE}/script.tsv")
    ids1, s1 = script_rows(f"{CAND}/script.tsv")
    changed = [c for c in ids0 if s0[c] != s1.get(c)]
    exact = all(s0[c].replace(CORRECTIONS[c][1], CORRECTIONS[c][2], 1) == s1[c] and s0[c].count(CORRECTIONS[c][1]) == 1
                for c in changed if c in CORRECTIONS)
    sentences = {c: (len(bron.kit_sentences(s0[c])), len(bron.kit_sentences(s1[c]))) for c in changed}
    plain = all(not re.search(r"[0-9=+<>/()\[\]{}*_|]", s1[c]) for c in changed)
    sb = load("storyboard_tool", "tools/storyboard.py")
    agree = {}
    for name, root, script in (("stage05", BASE, s0), ("candidate", CAND, s1)):
        rows = [r for ch in sb.parse(Path(root) / "docs/storyboard.md") for r in ch["rows"]]
        spoken = {r["id"]: sb.spoken(r["words"]) for r in rows if sb.spoken(r["words"])}
        problems, _report = sb.check(TOPIC, sb.parse(Path(root) / "docs/storyboard.md"))
        agree[name] = (spoken == script and list(spoken) == list(script), problems)
    header = [Path(f"{r}/script.tsv").read_text().split("\n", 1)[0] for r in (BASE, CAND)]
    check("t1-script", ids0 == ids1 and len(ids1) == 105 and sorted(changed) == sorted(CORRECTIONS) and exact
          and all(a == b for a, b in sentences.values()) and plain
          and all(ok for ok, _problems in agree.values()) and agree["stage05"][1] == agree["candidate"][1],
          f"105 ids in the same order; exactly {len(changed)} texts changed ({', '.join(changed)}), each by the one "
          f"span below and nothing else; the other {105 - len(changed)} byte-equal; kit sentence counts kept "
          f"{sentences}; no digit or symbol added; on both stage05 and the candidate, tools/storyboard.py's parser "
          f"gives exactly the script's 105 lines, and its gate check reports the same "
          f"{len(agree['candidate'][1])} problems on both")
    for c in changed:
        old, new = spans(s0[c], s1[c])
        note(f"{c} ({CORRECTIONS[c][0]}): {old!r} -> {new!r} ({len(s1[c].split()) - len(s0[c].split()):+d} words)")
    note(f"script header line (a comment, not a cue) changed: {header[0][:60]!r}... -> {header[1][:60]!r}...")
    note(f"storyboard gate problems present in stage05 too, unchanged: {agree['candidate'][1]}")
    return s0, s1


# --------------------------------------------------------- 4 units and map
def json_diff(a, b, path=""):
    if type(a) is not type(b):
        yield path
    elif isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            yield from ([f"{path}/{k}"] if k not in a or k not in b else json_diff(a[k], b[k], f"{path}/{k}"))
    elif isinstance(a, list):
        if len(a) != len(b):
            yield f"{path}#len"
        for i, (x, y) in enumerate(zip(a, b)):
            yield from json_diff(x, y, f"{path}[{i}]")
    elif a != b:
        yield path


def section_units_map(s1):
    print("== 4. Units and map")
    u0, u1 = (json.loads(Path(f"{r}/narration_units.json").read_text()) for r in (BASE, CAND))
    m0, m1 = json.loads(Path(f"{BASE}/{OLD_MAP}").read_text()), json.loads(Path(f"{CAND}/{NEW_MAP}").read_text())
    script_sha, map_sha = sha_file(f"{CAND}/script.tsv"), sha_file(f"{CAND}/{NEW_MAP}")
    idx = {u["line_ids"][0]: k for k, u in enumerate(u0["units"])}
    want_u = sorted(["/direction_contract/map/path", "/direction_contract/map/sha256", "/direction_contract/script_sha256",
                     "/scope", "/script_sha256"] + [f"/units[{idx[c]}]/text" for c in CORRECTIONS])
    over = {o["cue"]: k for k, o in enumerate(m0["overrides"])}
    holds = {h["cue"]: k for k, h in enumerate(m0["visual_holds"])}
    want_m = sorted(["/scope", f"/scripts/{TOPIC}", f"/visual_holds[{holds['15_3']}]/sha256"]
                    + [f"/overrides[{over[c]}]/after" for c in CORRECTIONS if c in over])
    du, dm = sorted(json_diff(u0, u1)), sorted(json_diff(m0, m1))
    bound = (u1["script_sha256"] == script_sha and u1["direction_contract"]["script_sha256"] == script_sha
             and u1["direction_contract"]["map"] == {"path": f"explainer_corpus/{TOPIC}/{NEW_MAP}", "sha256": map_sha}
             and m1["scripts"] == {TOPIC: script_sha})
    texts = [u["text"] for u in u1["units"]] == list(s1.values()) and [u["line_ids"] for u in u1["units"]] == [
        [c] for c in s1] and [u["id"] for u in u1["units"]] == [u["id"] for u in u0["units"]]
    afters = all(o["after"] == s1[o["cue"]] for o in m1["overrides"])
    tree_root = Path(CAND).parents[1]
    sites = m1["visual_holds"] + m1["helpers"]
    valid = [s for s in sites if sha_file(tree_root / s["path"]) == s["sha256"]
             and s["marker"] in (tree_root / s["path"]).read_text().splitlines()[s["line"] - 1]]
    check("u1-units-map", du == want_u and dm == want_m and bound and texts and afters and len(valid) == len(sites),
          f"units differ from stage05 only in the 5 texts, the script hash (twice), the map path and hash and the "
          f"scope; the new map differs from oliver12 only in {len([d for d in dm if 'overrides' in d])} override "
          f"texts (05_8, 14_3), the script hash, hold 15_3's closing.rs hash and the scope; all 105 unit ids, line "
          f"ids and texts equal the script; all {len(m1['overrides'])} overrides quote their cue exactly; all "
          f"{len(m1['visual_holds'])} holds and {len(m1['helpers'])} helpers match their file's hash and carry their "
          f"marker on their line in the candidate")
    note(f"new hashes: script {script_sha[:12]}, units {sha_file(f'{CAND}/narration_units.json')[:12]}, map "
         f"{map_sha[:12]}; oliver12 stays in the crate, bound to stage05, and no source file names a map")


# --------------------------------------------------------- 5 hooks and holds
def section_hooks(bridge, intake):
    print("== 5. Hooks and holds: the bridge's source side, stage05 against the candidate")
    base, _s, _f = bridge.source_side(Path(BASE))
    saved = dict(intake.PINS), intake.MAP_REL, bridge.CHAPTER_SOURCES_SHA256
    chapters = sorted((Path(CAND) / "src/chapters").glob("*.rs"))
    listing = "".join(f"{p.name} {sha_file(p)}\n" for p in chapters)
    try:
        intake.PINS.update(script=sha_file(f"{CAND}/script.tsv"), units=sha_file(f"{CAND}/narration_units.json"),
                           map=sha_file(f"{CAND}/{NEW_MAP}"))
        intake.MAP_REL, bridge.CHAPTER_SOURCES_SHA256 = NEW_MAP, sha_bytes(listing.encode())
        cand, _s, _f = bridge.source_side(Path(CAND))
    finally:
        intake.PINS.clear()
        intake.PINS.update(saved[0])
        intake.MAP_REL, bridge.CHAPTER_SOURCES_SHA256 = saved[1], saved[2]

    def anchors(side, with_byte):
        return sorted((a["cue"], a["phrase"], a["sentence"], a["at_sentence_start"], a["occurrences_in_cue"],
                       a["byte"] if with_byte or a["cue"] not in CORRECTIONS else None,
                       tuple((h["file"], h["line"], h["call"], h["via"]) for h in a["hooks"])) for a in side["anchors"])

    def dispositions(side):
        return sorted((d["override_index"], d["cue"], d["kind"], d["hold_kind"], d["hold_line"]) for d in side["dispositions"])

    moved = sorted({a["cue"] for a in cand["anchors"]} & set(CORRECTIONS))
    check("k1-hooks-holds", anchors(base, False) == anchors(cand, False) and not cand["anchor_problems"]
          and not cand["disposition_problems"] and dispositions(base) == dispositions(cand)
          and base["hook_sites"] == cand["hook_sites"] and base["unit_ids"] == cand["unit_ids"]
          and base["film_ignores_measured_sentence_starts"] == cand["film_ignores_measured_sentence_starts"],
          f"with the candidate's own hashes swapped in memory, the bridge's full source side (chapter listing and "
          f"indirect hooks included) finds the same {len(cand['anchors'])} anchors at {cand['hook_sites']} hook sites: "
          f"same phrase, sentence, start flag, file, line and call, and the same byte except inside the corrected "
          f"cues {moved}; no anchor or disposition problem; the same {len(cand['dispositions'])} end-of-cue holds")
    for a in sorted(cand["anchors"], key=lambda r: (r["cue"], r["byte"])):
        if a["cue"] in CORRECTIONS:
            note(f"{a['cue']} keeps {a['phrase']!r} once, in sentence {a['sentence']}")


# ------------------------------------- 6 the corrections and their mathematics
def bernoulli(n):
    b = [Fraction(1)]
    for m in range(1, n + 1):
        b.append(-sum(math.comb(m + 1, k) * b[k] for k in range(m)) / (m + 1))
    return b


B = bernoulli(60)
BORWEIN_N = 60


def borwein_d(n):
    d, total = [], Fraction(0)
    for i in range(n + 1):
        total += Fraction(n * math.factorial(n + i - 1) * 4 ** i, math.factorial(n - i) * math.factorial(2 * i))
        d.append(total)
    return [float((x - d[-1]) / d[-1]) for x in d]


D = borwein_d(BORWEIN_N)


def zeta_borwein(s):
    eta = -sum((-1) ** k * D[k] / (k + 1) ** s for k in range(BORWEIN_N))
    return eta / (1 - 2 ** (1 - s))


def zeta_em(s, n=40, m=25):
    """Euler-Maclaurin with exact Bernoulli numbers: this check's reference, valid for Re s > 1 - 2m."""
    total = sum(k ** -s for k in range(1, n)) + n ** (1 - s) / (s - 1) + 0.5 * n ** -s
    rising = s
    for k in range(1, m + 1):
        total += float(B[2 * k] / math.factorial(2 * k)) * rising * n ** (-s - 2 * k + 1)
        rising *= (s + 2 * k - 1) * (s + 2 * k)
    return total


def golden_min(f, a, b, steps=80):
    g = (math.sqrt(5) - 1) / 2
    c, d = b - g * (b - a), a + g * (b - a)
    for _ in range(steps):
        if f(c) < f(d):
            b, d = d, c
            c = b - g * (b - a)
        else:
            a, c = c, d
            d = a + g * (b - a)
    return (a + b) / 2


def angle(a, b):
    return math.degrees(cmath.phase(b / a)) % 360


def run_oracle(root):
    run = subprocess.run([sys.executable, "-I", "-B", f"{root}/oracle.py"], capture_output=True, text=True)
    facts = dict(line.split("\t", 1) for line in run.stdout.splitlines() if "\t" in line)
    return run.returncode, facts, run.stderr.splitlines()


def section_corrections(s0, s1):
    print("== 6. The corrections against the film and the mathematics")
    _code, facts, _err = run_oracle(CAND)
    # F1: the trivial zeros are zeros and lie off Re s = 1/2; the four ringed zeros lie on it
    zeta_neg = {n: (-1) ** n * B[n + 1] / (n + 1) for n in range(1, 13)}
    trivial = [-n for n, v in zeta_neg.items() if v == 0]
    shown = [int(float(x)) for x in facts["gallery.zeta.trivial"].split(",")]
    heights = [float(x) for x in facts["gallery.zeta.zeros"].split(",")]
    found = [golden_min(lambda t: abs(zeta_borwein(complex(0.5, t))), h - 0.01, h + 0.01) for h in heights]
    smallest = [abs(zeta_borwein(complex(0.5, t))) for t in found]
    control = abs(zeta_borwein(2) - math.pi ** 2 / 6)
    f1 = (trivial == [-2, -4, -6, -8, -10, -12] and shown == trivial[:3] and zeta_neg[1] == Fraction(-1, 12)
          and all(v < 1e-9 for v in smallest) and [fixed(t, 2) for t in found] == facts["gallery.zeta.zeros"].split(",")
          and control < 1e-12 and "Zeros at minus two, minus four, and on." in s1["05_7"])
    check("f1-05_8", f1,
          f"zeta(-n) = (-1)^n B(n+1)/(n+1) is 0 exactly for n = 2, 4, ..., 12 and not for odd n (zeta(-1) = -1/12): "
          f"the zeros 05_7 names ('Zeros at minus two, minus four, and on.', on screen {shown} ...) are real zeros "
          f"with Re s = -2, -4, ... off the line; the four ringed zeros have |zeta(1/2 + it)| down to "
          f"{max(smallest):.0e} at t = {', '.join(fixed(t, 6) for t in found)} (Borwein, n = 60; control zeta(2) "
          f"error {control:.0e}), matching the printed heights; so 'Apart from those ..., no zero found so far sits "
          f"off that line' is true of everything the chapter shows")
    # F2: critical points multiply angles; away from them a small piece is a stretch and a turn
    h = 1e-5
    at0 = angle(h ** 2, (1j * h) ** 2), angle(h ** 3, (1j * h) ** 3)
    z0 = 0.6 + 0.8j
    near = [((z0 + w) ** 2 - z0 ** 2) / w for w in (h, 1j * h)]
    keep = angle(near[0] * h, near[1] * 1j * h)
    caption = re.search(r'pub const CAPTIONS: \[\[&str; 2\]; 5\] = \[\n    \["([^"]+)", "([^"]+)"\]',
                        Path(f"{CAND}/src/chapters/closing.rs").read_text()).groups()
    em = {**{c: 0.278 for c in "il,.:;"}, **{c: 0.306 for c in "jf"}, " ": 0.333, **{c: 0.39 for c in "rst"},
          **{c: 0.444 for c in "acez"}, **{c: 0.5 for c in "go"}, **{c: 0.528 for c in "kqvxy"},
          **{c: 0.556 for c in "bdhnpu"}, "w": 0.722, "m": 0.833}
    width = sum(em.get(c, 0.75) for c in caption[0]) * 18 * 0.01
    f2 = (abs(at0[0] - 180) < 1e-9 and abs(at0[1] - 270) < 1e-9 and abs(keep - 90) < 1e-3
          and abs(abs(near[0]) - abs(2 * z0)) < 1e-4 and "So a nonzero derivative keeps angles." in s1["07_2"]
          and "A point where the derivative is zero is a critical point, and there angles get multiplied." in s1["07_2"]
          and caption == ("nonzero derivative:", "stretch and turn") and len(caption[0]) <= len("at most one extension;"))
    check("f2-15_2", f2,
          f"at the critical point 0, z^2 takes a right angle to {at0[0]:.0f} degrees and z^3 to {at0[1]:.0f}; at "
          f"0.6 + 0.8i, where the derivative is not 0, z^2 keeps it ({keep:.4f} degrees) and stretches by |2z| = 2: "
          f"'away from critical points' is the condition 07_2 states ('So a nonzero derivative keeps angles.'), and "
          f"the caption {caption[0]!r} / {caption[1]!r} says it in 07_2's words ({len(caption[0])} characters, under "
          f"the 22 of the longest caption; about {width:.2f} of the 2.3 panel by the CMR10 estimate)")
    # F3: the zero series matches the curve exactly where it is flat and misses it exactly where it rises
    flat = lambda x: math.exp(-1 / x) if x > 0 else 0.0
    left = [-3 + k / 100 for k in range(301)]
    right = [k / 100 for k in range(1, 401)]
    f3 = (all(flat(x) == 0.0 for x in left) and all(flat(x) > 0 for x in right)
          and all(flat(b) > flat(a) for a, b in zip(right, right[1:]))
          and "It agrees with zero on a whole half line" in s1["10_3"])
    check("f3-10_3", f3,
          f"e^(-1/x) on the graph's window: the series 0 equals the curve at all {len(left)} grid points of "
          f"[-3, 0] and differs at all {len(right)} of (0, 4], where the curve strictly rises: the series misses "
          f"'the whole rise' and nothing else, as the cue's own 'It agrees with zero on a whole half line' says")
    # F5: 1 + z^2 = 0 only at +-i, so the bump is known everywhere else
    grid = [Fraction(k, 50) for k in range(-100, 101)]
    poles = [(a, b) for a in grid for b in grid if 1 + a * a - b * b == 0 and a * b == 0]
    card = re.search(r'pub const EVERYWHERE: &str = "([^"]+)";', Path(f"{CAND}/src/chapters/continuing.rs").read_text())
    f5 = poles == [(0, -1), (0, 1)] and cmath.sqrt(-4) / 2 == 1j and card and card.group(1) == (
        "the bump: defined everywhere but ±i")
    check("f5-14_1", f5,
          f"exactly, 1 + z^2 = 0 at only {len(poles)} of the {len(grid) ** 2} points of the 1/50 grid on [-2, 2]^2, "
          f"z = -i and i (the roots of z^2 + 1); the cue now says what the same cue's card says, "
          f"{card.group(1) if card else None!r}")
    # F4: one spelling in the script; the 14_3 card keeps its hyphen
    step = re.search(r'pub const STEP: &str = "([^"]+)";', Path(f"{CAND}/src/chapters/continuing.rs").read_text())
    spelled = sorted(c for c, t in s1.items() if "ecentre" in t or "e-centre" in t)
    f4 = (spelled == ["09_8", "14_3"] and all("Recentre" in s1[c] and "e-centre" not in s1[c] for c in spelled)
          and step and step.group(1) == "step by step: re-centre inside the disk")
    check("f4-14_3", f4,
          f"the script now spells it one way, 'Recentre', in {', '.join(spelled)}; the card 14_3 shows on screen "
          f"(continuing.rs STEP) still reads {step.group(1) if step else None!r} (finding R1)")
    doc = Path(f"{CAND}/src/chapters/closing.rs").read_text().splitlines()[6:8]
    note(f"closing.rs:7-8 still reads {' '.join(x.strip('/! ') for x in doc)!r} (finding R2)")


# -------------------------------------------------------------- 7 root03 deltas
def section_root03():
    print("== 7. Root03 deltas: zeta near 0, and the oracle's exterior Cauchy average")
    model = Path(f"{WORK}/src/model.rs").read_text()
    n_terms = int(re.search(r"const ZETA_N: u32 = (\d+);", model).group(1))
    m_terms = int(re.search(r"const ZETA_M: usize = (\d+);", model).group(1))
    table = re.search(r"const BERNOULLI_EVEN: \[f64; 12\] = \[(.*?)\];", model, re.S).group(1)
    even = [float(Fraction(*map(int, re.fullmatch(r"(-?\d+)\.0 / (\d+)\.0", x.strip().replace("_", "")).groups())))
            for x in table.split(",") if x.strip()]

    def zeta_model(s):  # model.rs zeta_em, line for line
        n = float(n_terms)
        total = sum(float(k) ** -s for k in range(1, n_terms))
        n_pow = n ** -s
        total += n_pow * n / (s - 1) + n_pow * 0.5
        rising, fact, n_power = s, 2.0, n_pow / n
        for k in range(1, m_terms + 1):
            total += rising * n_power * (even[k - 1] / fact)
            j = 2.0 * k
            rising *= (s + (j - 1)) * (s + j)
            fact *= (j + 1) * (j + 2)
            n_power /= n * n
        return total

    disk = [0j] + [cmath.rect(r, 2 * math.pi * k / 16) for r in (0.05, 0.1, 0.15, 0.2, 0.2499) for k in range(16)]
    wide = [cmath.rect(r, 2 * math.pi * k / 16) for r in (0.5, 0.75, 1.0) for k in range(16)
            if cmath.rect(r, 2 * math.pi * k / 16).real < 0.5]
    err_model = max(abs(zeta_model(s) - zeta_em(s)) for s in disk)
    err_borwein = max(abs(zeta_borwein(s) - zeta_em(s)) for s in disk)
    err_wide = max(abs(zeta_model(s) - zeta_em(s)) for s in wide)
    derivative = [(zeta_model(w) - zeta_model(-w)) / (2 * w) for w in (1e-5, 1e-5j)]
    exact_even = [float(B[2 * k]) for k in range(1, 13)]
    z1 = (even == exact_even and abs(zeta_model(0j) + 0.5) < 1e-14 and abs(zeta_borwein(0j) + 0.5) < 1e-14
          and err_model < 1e-13 and err_borwein < 1e-13 and err_wide < 1e-12
          and all(abs(d + math.log(2 * math.pi) / 2) < 1e-8 for d in derivative)
          and "if s.re >= 0.5 || s.abs() < 0.25 {" in model)
    check("z1-zeta-near-0", z1,
          f"model.rs now uses Euler-Maclaurin (N = {n_terms}, {m_terms} Bernoulli terms, its table equal to exact "
          f"B2..B24) for |s| < 0.25; against this check's own Euler-Maclaurin (N = 40, 25 exact terms) at "
          f"{len(disk)} points of the disk it is within {err_model:.0e}, and within {err_wide:.0e} at "
          f"{len(wide)} points with |s| up to 1 and Re s < 1/2, so the branch is sound well past its edge; the "
          f"oracle's Borwein sum, now also used there although its docstring says Re s > 0, is within "
          f"{err_borwein:.0e}; zeta(0) = -1/2 and zeta'(0) = -log(2 pi)/2 along both axes")
    # the 16-point average of the bump on |z| = 2: every surviving term of the exterior series is negative
    sampled = sum(1 / (1 + (2 * cmath.exp(2j * math.pi * k / 16)) ** 2) for k in range(16)) / 16
    exact, old = -1 / (2 ** 16 - 1), -(2.0 ** -16) / (1 + 2.0 ** -16)
    code5, facts5, _e5 = run_oracle(CAND)
    code3, facts3, err3 = run_oracle(WORK)
    differ = sorted(k for k in set(facts5) | set(facts3) if facts5.get(k) != facts3.get(k))
    new_checks = [x for x in err3 if "exterior Cauchy" in x or "opposite denominator" in x]
    z2 = (abs(sampled.real - exact) < 1e-15 and abs(sampled.imag) < 1e-15 and abs(old - exact) > 4e-10
          and fixed(old, 6) == fixed(exact, 6) == facts3["cauchy.r_two.n16"] and code5 == code3 == 0 and not differ
          and len(new_checks) == 2 and all(x.startswith("# ok") for x in new_checks))
    check("z2-cauchy-exterior", z2,
          f"the sampled 16-point average is {sampled.real:.10e}, equal to -1/65535 to {abs(sampled.real - exact):.0e}; "
          f"stage05's closed form -2^-16/(1 + 2^-16) was off by {abs(old - exact):.1e}, below the printed "
          f"{fixed(exact, 6)}; the stage05 and root03 oracles both exit 0 and print the same {len(facts3)} facts, "
          f"and root03's two new cross-checks pass")


# ------------------------------------------------- 8 requests and takes (text)
def section_takes(s0, s1):
    print("== 8. Requests and takes, by text only")
    names5 = {p.name for p in Path(TAKES5).iterdir()}
    names6 = {p.name for p in Path(TAKES6).iterdir()}
    saved5 = sorted(n.split("-")[0] for n in names5 if n.endswith("-downloads01.json"))
    saved6 = sorted(n.split("-")[0] for n in names6 if n.endswith("-downloads01.json"))
    script_sha = sha_file(f"{CAND}/script.tsv")
    reqs = [json.loads((Path(TAKES6) / n).read_text()) for n in sorted(names6) if "-request" in n]
    good = [r for r in reqs if r["text"] == s1[r["cue"]] and sha_bytes(r["text"].encode()) == r["text_sha256"]
            and r["script_sha256"] == script_sha and all(r[k] == v for k, v in FREDERICK.items())]
    check("r1-takes", not set(saved5) & set(CORRECTIONS) and all(s0[c] == s1[c] for c in saved5)
          and set(CORRECTIONS) <= set(saved6) and len(good) == len(reqs) and set(saved5) | set(saved6) == set(s1),
          f"{len(saved5)} cues have stage05 Frederick downloads, none of them corrected, and their texts are "
          f"byte-equal in stage06; {len(reqs)} stage06 requests all carry the stage06 text, its hash, script "
          f"{script_sha[:8]} and Frederick's settings, and {len(saved6)} have downloads, the five corrected cues "
          f"among them; together the folders cover all {len(s1)} cues (text and receipts only; nothing heard)")


# ---------------------------------------------------------------- 9 hygiene
def section_hygiene():
    here = Path(__file__).resolve()
    if here.parent.parent != (Path.cwd() / BUNDLE).resolve():
        return
    print("== 9. Hygiene")
    root = Path(BUNDLE)
    names = sorted(str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()
                   and "__pycache__" not in p.relative_to(root).parts)
    want = ["REVIEW.md", "checks/outputs/stage06_candidate_review.txt", "checks/stage06_candidate_review.py"]
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
        bridge, intake, sums = section_pins()
    except Exception as exc:  # this check's PinMoved, or the bridge's or intake's own
        if type(exc).__name__ != "PinMoved":
            raise
        print(f"PIN MOVED: {exc}")
        return 3
    section_scope(sums)
    s0, s1 = section_script(bridge.dep("log_preparation"))
    section_units_map(s1)
    section_hooks(bridge, intake)
    section_corrections(s0, s1)
    section_root03()
    section_takes(s0, s1)
    section_hygiene()
    print(f"RESULT {'FAIL' if failures else 'PASS'}, {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
