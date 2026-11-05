#!/usr/bin/env python3
"""Checks for logarithm-clarity-integration01. Run from the repo root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-clarity-integration01/checks/run_checks.py FRESH_SCRATCH

Reads the repository and writes only under FRESH_SCRATCH, a folder that must not exist yet. The transcript
names scratch paths as SCRATCH, so two runs print the same bytes. Nothing is compiled, rendered, synthesized,
measured or heard, and no audio is decoded; WAV files are only hashed. The one tool run besides Python and
git apply is rustfmt --check, which formats nothing.

Sections:
  pins        pins.sha256 matches every input and PINS_SHA256; --print-pins reports no difference;
  replay      --check passes on the delivered stage09/; two fresh --out runs equal it byte for byte;
  mirror      the helper run in a scratch copy of only its pinned inputs writes the same bytes (so it reads
              nothing unpinned and nothing depends on where it runs), and refuses with exit 3, writing
              nothing, when one input there is changed, added or removed;
  patch       git apply of the sealed patch on a scratch copy of stage08 gives the delivered script and source,
              and every other candidate file is stage08's bytes;
  recompute   own code, not the helper's: all 33 anchors (binder03's word rule, keyed by cue and phrase),
              template marks, 15 hold intervals and file hashes, words, unit texts, every crate hash the plan
              names, and the 78 untouched cues in script, units, template, cue delta and take binding;
  consumers   tools/coherent_narration.py read_units, direction_contract and review_directions accept the
              stage09 plan laid out as root would install it, and refuse stale units, a stale map hash and
              bound-plan03's evidence; docs/check_cues.py and rustfmt --check pass; the 08_5 Rust test and
              phrase_seconds mirrored: marks at 267/306 are found, marks at 260/299 miss and fall back;
  rejections  --validate-timings, --validate-receipt and --verify each pass their control and refuse every
              planted defect: old bytes, old text hashes, missing, extra or renamed marks, a changed
              neighbour, an old take, a stale map, units or hold hash, a stray file, another voice;
  lavender    LavenderPike's sealed rebind_acceptance.py (review SHA256SUMS e78a99b4) on the delivered
              candidate: its one FAIL is the units' scope sentence, which the helper rewrites on purpose,
              and with stage08's sentence put back every check passes;
  hygiene     no machine paths, user names, addresses or gendered pronouns in the authored files.
Exit 0 when every check passes, 1 when one fails, 2 for usage.
"""
import hashlib
import json
import math
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-clarity-integration01"
HELPER = REPO / HERE / "integrate_clarity.py"
DELIVERED = REPO / HERE / "stage09"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE08 = REPO / EL / "combined-repair-stage08/candidate"
PLAN3 = REPO / EL / "combined-repair-stage07/bound-plan03"
INSTALL = f"{EL}/combined-repair-stage09"
TAKES = REPO / EL / "ui-takes01"
PATCH = REPO / P / "logarithm-narration-clarity-repair01/logarithm-narration-clarity.patch"
REVIEW = REPO / P / "logarithm-narration-clarity-review01"
REVIEW_SUMS = "e78a99b4f1ecaf7dbe47b12636e7f97b5452b241a39f763875a4014701808c9e"
REL = "explainer_corpus/logarithm_mysteries"
MAP16 = "narration-direction-map-oliver16.json"
NEW_SCRIPT = "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a"
OLD_SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
NEW_PRODUCT = "bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635"
OLD_PRODUCT = "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db"
CHANGED = ("04_4", "08_5")
OLD_TAKES = {
    "04_4-download01.wav": "de3abb952c675c2112c949813b2d70f962b95a3adf90d8f3289c6a4e29dce9ab",
    "04_4-download02.wav": "573d2360f72d4c8d451cd684ff8b3febf68168b24add6b070886433080185164",
    "08_5-download01.wav": "9b5d2b73ec6400569a32e03fe2c0d2b91c19308565ea6ee433b9ca99ca0d1ae5",
    "08_5-download02.wav": "5ec634e9c591e1768d69ea5195e6b07b91a3da1c6c7cecefa07799b3a269c493",
}
TEST_NEEDLES = [
    'kit.until_sentence(stage, "08_5", 3)',
    "fade_in_all(stage, &[root1, not_two, one_dot], 0.5)",
    'kit.until_sentence(stage, "08_5", 4)',
    "fade_in_all(stage, &[halving], 0.5)",
    'until_phrase(&kit, stage, "08_5", HALF_TURN_PHRASE)',
    "tween(half_turn, 1.0, HALF_TURN_S)",
    "fade_in_all(stage, &[half_word], 0.3)",
    'until_phrase(&kit, stage, "08_5", FACTOR_PHRASE)',
    "swap(stage, &[halving], &[halved], 0.3)",
    'kit.until_sentence(stage, "08_5", 5)',
    "kit.finish(stage)",
]
RUSTFMT = ["rustfmt", "+nightly-2026-08-31", "--edition", "2024", "--config", "max_width=120", "--check"]

results = []


def check(ok, label):
    results.append(bool(ok))
    print(("PASS " if ok else "FAIL ") + label)
    return ok


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_t(text):
    return sha(text.encode("utf-8"))


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def dump(path, obj):
    Path(path).write_text(json.dumps(obj, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def tree(root):
    root = Path(root)
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in sorted(root.rglob("*")) if p.is_file()}


def rows(data):
    out = {}
    for line in data.decode("utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            cue, text = line.split("\t")
            out[cue] = text
    return out


def direction_words(text):
    """tools/coherent_narration.py direction_words(), written out again here."""
    return [m.lower().replace("’", "'") for m in re.findall(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?", text)]


def opens(text):
    raw = text.encode("utf-8")
    return [0] + [i + 2 for i in range(len(raw) - 1) if raw[i] in b".?!" and raw[i + 1] == 0x20]


def anchor(text, phrase):
    raw, want = text.encode("utf-8"), phrase.encode("utf-8")
    byte = raw.find(want)
    if byte < 0 or raw.count(want) != 1:
        return None
    return {
        "byte": byte,
        "byte_end": byte + len(want),
        "word_index": len(direction_words(raw[:byte].decode("utf-8"))),
        "sentence_index": max(k for k, o in enumerate(opens(text)) if o <= byte),
    }


def interval(lines, say, marker):
    return sha_t("\n".join(lines[say - 1 : marker]))


SCRATCH = None


def run(args, cwd=REPO):
    r = subprocess.run([sys.executable, "-I", "-B", str(HELPER), *args], cwd=cwd, capture_output=True, text=True)
    out = (r.stdout + r.stderr).replace(str(SCRATCH), "SCRATCH") if SCRATCH else r.stdout + r.stderr
    return r.returncode, out


def section(name):
    print(f"== {name}")


def main(argv):
    global SCRATCH
    if len(argv) != 1 or not HELPER.is_file() or Path(argv[0]).exists():
        print(__doc__.split("\n\n")[1], file=sys.stderr)
        return 2
    SCRATCH = Path(argv[0]).resolve()
    SCRATCH.mkdir(parents=True)
    pins_path = REPO / HERE / "pins.sha256"
    pins = {}
    for line in pins_path.read_text(encoding="utf-8").splitlines():
        h, p = line.split("  ", 1)
        pins[p] = h
    delivered = tree(DELIVERED)
    cand = {k[len("candidate/") :]: v for k, v in delivered.items() if k.startswith("candidate/")}
    plan = {k[len("bound-plan/") :]: v for k, v in delivered.items() if k.startswith("bound-plan/")}
    s08 = tree(STAGE08)
    new, old = rows(cand[f"{REL}/script.tsv"]), rows(s08[f"{REL}/script.tsv"])

    # ------------------------------------------------------------------ pins
    section("pins")
    stale = [p for p, h in pins.items() if not (REPO / p).is_file() or sha((REPO / p).read_bytes()) != h]
    check(not stale, f"pins.sha256: {len(pins)} inputs, every one present with its pinned hash ({stale or 'none stale'})")
    const = re.search(r'^PINS_SHA256 = "([0-9a-f]{64})"$', HELPER.read_text(encoding="utf-8"), re.M)
    check(const and const.group(1) == sha(pins_path.read_bytes()), f"PINS_SHA256 in the helper is pins.sha256's hash {sha(pins_path.read_bytes())[:12]}")
    code, out = run(["--print-pins"])
    check(code == 0 and "differs" not in out and out.count("\n") == len(pins) + 1, "--print-pins: every current input equals its pin, and no input is unlisted")
    check(plan["inputs.sha256"] == pins_path.read_bytes(), "bound-plan/inputs.sha256 is the pin table")

    # ------------------------------------------------------------------ replay
    section("replay")
    code, out = run(["--check"])
    check(code == 0 and out.strip().endswith("PASS"), "--check: the delivered stage09/ equals a fresh build")
    runs = []
    for n in (1, 2):
        code, _ = run(["--out", str(SCRATCH / f"out{n}")])
        runs.append(tree(SCRATCH / f"out{n}") if code == 0 else None)
    check(runs[0] is not None and runs[0] == runs[1] == delivered, f"two fresh --out runs: {len(delivered)} files, byte-identical to each other and to stage09/")
    code, out = run(["--out", str(SCRATCH / "out1")])
    check(code == 1 and "exists" in out, "--out refuses a folder that exists")

    # ------------------------------------------------------------------ mirror
    section("mirror")
    mirror = SCRATCH / "mirror"
    for p in [*pins, f"{HERE}/pins.sha256"]:
        (mirror / p).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / p, mirror / p)
    code, _ = run(["--out", str(SCRATCH / "mirror-out")], cwd=mirror)
    check(code == 0 and tree(SCRATCH / "mirror-out") == delivered, f"in a copy of its {len(pins)} pinned inputs alone, the helper writes stage09/'s bytes")
    for guarded in (f"{EL}/combined-repair-stage08/candidate/x", f"{REL}/x", f"{EL}/ui-takes01/x", f"{P}/logarithm-narration-clarity-repair01/x"):
        code, out = run(["--out", guarded], cwd=mirror)  # in the mirror, so a guard bug could only write to scratch
        check(code == 1 and "refused" in out and not (mirror / guarded).exists(), f"--out refuses {guarded.split('/')[-2]}/")
    product = mirror / EL / "combined-repair-stage08/candidate" / REL / "src/chapters/product_rule.rs"
    template = mirror / EL / "combined-repair-stage07/bound-plan03/phrase-timings-unmeasured-template.json"
    patch = mirror / P / "logarithm-narration-clarity-repair01/logarithm-narration-clarity.patch"
    take = mirror / EL / "ui-takes01/08_5-download01.wav"
    stray = mirror / EL / "combined-repair-stage08/candidate" / REL / "src/chapters/product_rule.rs.rej"
    pinsfile = mirror / HERE / "pins.sha256"

    def plant(label, do, undo):
        do()
        target = SCRATCH / f"refused-{len(results)}"
        code, out = run(["--out", str(target)], cwd=mirror)
        undo()
        check(code == 3 and "STALE INPUT" in out and not target.exists(), f"stale {label}: exit {code}, nothing written")

    for label, path in (("stage08 product_rule.rs", product), ("bound-plan03 template", template), ("clarity patch", patch), ("pins.sha256", pinsfile)):
        keep = path.read_bytes()
        plant(label, lambda p=path, k=keep: p.write_bytes(k + b" "), lambda p=path, k=keep: p.write_bytes(k))
    keep = take.read_bytes()
    plant("original 08_5 take removed", lambda: take.unlink(), lambda: take.write_bytes(keep))
    plant("file added to the stage08 candidate", lambda: stray.write_bytes(b"x"), lambda: stray.unlink())
    check(tree(mirror / EL / "combined-repair-stage08/candidate") == s08, "the mirror's stage08 is restored to root's bytes after the plants")

    # ------------------------------------------------------------------ patch
    section("patch")
    lane = SCRATCH / "git-apply"
    shutil.copytree(STAGE08, lane)
    r = subprocess.run(["git", "apply", "--check", str(PATCH)], cwd=lane, capture_output=True, text=True)
    r2 = subprocess.run(["git", "apply", str(PATCH)], cwd=lane, capture_output=True, text=True) if r.returncode == 0 else r
    applied = tree(lane)
    check(r2.returncode == 0, "git apply of the sealed patch 2463334a on a scratch copy of stage08")
    check(applied[f"{REL}/script.tsv"] == cand[f"{REL}/script.tsv"] and applied[f"{REL}/src/chapters/product_rule.rs"] == cand[f"{REL}/src/chapters/product_rule.rs"],
          "git apply and the helper's applier give the same script.tsv and product_rule.rs")
    check(sha(cand[f"{REL}/script.tsv"]) == NEW_SCRIPT and sha(cand[f"{REL}/src/chapters/product_rule.rs"]) == NEW_PRODUCT, "script 559afa36, product_rule.rs bc9fbe1d")
    others = [k for k in s08 if k not in (f"{REL}/script.tsv", f"{REL}/src/chapters/product_rule.rs", f"{REL}/narration_units.json")]
    check(len(others) == 44 and all(cand[k] == s08[k] for k in others), f"the other {len(others)} stage08 files are byte-identical in the candidate")
    check(set(cand) == set(s08) | {f"{REL}/docs/{MAP16}"}, "the candidate is stage08's 47 files plus docs/narration-direction-map-oliver16.json")
    check(cand[f"{REL}/docs/narration-direction-map-oliver15.json"] == s08[f"{REL}/docs/narration-direction-map-oliver15.json"], "oliver15 is kept as it was")
    check(list(new) == list(old) and [c for c in old if old[c] != new[c]] == list(CHANGED), "the patched script changes 04_4 and 08_5 only; 78 rows are stage08's bytes")

    # ------------------------------------------------------------------ recompute
    section("recompute")
    a3, a9 = load(PLAN3 / "phrase-anchors.json"), json.loads(plan["phrase-anchors.json"])
    key3 = {(x["cue"], x["phrase"]): x for x in a3["anchors"]}
    key9 = {(x["cue"], x["phrase"]): x for x in a9["anchors"]}
    check(set(key3) == set(key9) and len(key9) == 33, f"the same {len(key9)} anchors as bound-plan03, keyed by cue and phrase (10_3's consts are null)")
    fields = ("byte", "byte_end", "word_index", "sentence_index")
    wrong = [k for k, x in key9.items() if anchor(new[k[0]], k[1]) != {f: x[f] for f in fields}]
    check(not wrong, f"every anchor's byte, byte_end, word_index and sentence_index recomputed from the stage09 text ({wrong or 'all equal'})")
    moved = sorted((k[0], key3[k]["byte"], key9[k]["byte"]) for k in key9 if key9[k]["byte"] != key3[k]["byte"])
    check(moved == [("08_5", 260, 267), ("08_5", 299, 306)], f"moved anchors: {moved}")
    still = [k for k in key9 if k[0] != "08_5" and any(key9[k][f] != key3[k][f] for f in fields)]
    check(not still, "the other 31 anchors keep bound-plan03's values")
    w = {k: anchor(new[k[0]], k[1])["word_index"] for k in key9 if k[0] == "11_20"}
    check(all(len(new["11_20"].encode()[: key9[k]["byte"]].decode().split()) + 1 == w[k] for k in w), "11_20's anchors count 'sixty-six' as two words, binder03's rule (whitespace would give one less)")
    t9 = json.loads(plan["phrase-timings-unmeasured-template.json"])
    t3 = load(PLAN3 / "phrase-timings-unmeasured-template.json")
    by_cue = {}
    for (cue, phrase), x in key9.items():
        by_cue.setdefault(cue, []).append((x["byte"], phrase))
    bad = [c for c in new if [(m["byte"], m["phrase"]) for m in t9["cues"][c]["marks"]] != sorted(by_cue.get(c, []))]
    check(not bad and t9["script_sha256"] == NEW_SCRIPT, f"template: script 559afa36; each cue's marks are its anchors' bytes and phrases, in byte order ({bad or 'all'})")
    check([m["byte"] for m in t9["cues"]["08_5"]["marks"]] == [267, 306] and t9["cues"]["04_4"]["marks"] == [], "template: 08_5 marks at 267 and 306; 04_4 has none")
    check(all(t9["cues"][c]["text_sha256"] == sha_t(new[c]) for c in new), "template: every text_sha256 is the stage09 text's")
    check(all(t9["cues"][c] == t3["cues"][c] for c in new if c not in CHANGED), "template: the 78 other entries are bound-plan03's")
    check(all(e["wav_sha256"] is None and all(m["seconds"] is None for m in e["marks"]) for e in t9["cues"].values()), "template: every WAV and every second is null, so the kit cannot load it")
    hold = json.loads(plan["hold-audit.json"])
    m16 = json.loads(cand[f"{REL}/docs/{MAP16}"])
    says = {(h["cue"], h["file"]): h["say_line"] for h in hold["prediction_holds"] + hold["reading_holds"]}
    bad = []
    for r in m16["visual_holds"]:
        lines = cand[r["path"]].decode("utf-8").split("\n") if r["path"] in cand else None
        if lines is None or r["sha256"] != sha(cand[r["path"]]) or r["evidence"]["sha256"] != r["sha256"] or interval(lines, says[(r["cue"], Path(r["path"]).name)], r["line"]) != r["interval_sha256"]:
            bad.append(r["cue"])
    check(not bad and len(m16["visual_holds"]) == 15, f"map: 15 holds; each file hash is the candidate's and each interval recomputes ({bad or 'all'})")
    bad = []
    for r in hold["prediction_holds"] + hold["reading_holds"]:
        rel = f"{REL}/src/chapters/{r['file']}"
        lines = cand[rel].decode("utf-8").split("\n")
        if r["source_sha256"] != sha(cand[rel]) or interval(lines, r["say_line"], r["marker_line"]) != r["interval_sha256"] or r["marker"] not in lines[r["marker_line"] - 1]:
            bad.append(r["cue"])
    check(not bad, f"hold audit: 15 holds' source hashes current, intervals and markers re-read ({bad or 'all'})")
    words = {c: len(t.split()) for c, t in new.items()}
    check(hold["counts"]["words_total"] == sum(words.values()) == 4594 and hold["counts"]["words_max"] == max(words.values()) == 95 and {r["cue"]: r["words"] for r in hold["cues"]} == words,
          "hold audit: 4594 words, at most 95, each cue's count from the stage09 text (08_5: 83)")
    check(hold["script_sha256"] == NEW_SCRIPT and hold["map"] == f"{REL}/docs/{MAP16}" and all(r["sha256"] == sha(cand[f"{REL}/src/chapters/{r['file']}"]) for r in hold["registered_chapters"]),
          "hold audit: script 559afa36, map oliver16, every registered chapter's hash current")
    units = json.loads(cand[f"{REL}/narration_units.json"])
    u08 = json.loads(s08[f"{REL}/narration_units.json"])
    check(all(u["text"] == " ".join(new[c] for c in u["line_ids"]) for u in units["units"]), "units: every text is its stage09 script rows")
    check([u["text"] for u in units["units"] if u["id"] not in ("oliver_04_4", "oliver_08_5")] == [u["text"] for u in u08["units"] if u["id"] not in ("oliver_04_4", "oliver_08_5")],
          "units: the 78 other texts are stage08's")
    check(units["script_sha256"] == units["direction_contract"]["script_sha256"] == NEW_SCRIPT and units["direction_contract"]["map"] == {"path": f"{REL}/docs/{MAP16}", "sha256": sha(cand[f"{REL}/docs/{MAP16}"])},
          "units: both script hashes 559afa36; the contract names oliver16 by path and hash")
    check(plan[f"{REL}/narration_units.json"] == cand[f"{REL}/narration_units.json"] and plan[f"{REL}/docs/{MAP16}"] == cand[f"{REL}/docs/{MAP16}"], "bound-plan's units and map are the candidate's bytes")
    m15 = json.loads(s08[f"{REL}/docs/narration-direction-map-oliver15.json"])
    loose = ("sha256", "evidence")
    check(m16["overrides"] == m15["overrides"] and m16["hold_kinds"] == m15["hold_kinds"] and m16["shared_helpers"] == m15["shared_helpers"]
          and [{k: v for k, v in r.items() if k not in loose} for r in m16["visual_holds"]] == [{k: v for k, v in r.items() if k not in loose} for r in m15["visual_holds"]],
          "map: overrides, hold kinds, shared helpers and every hold's other fields are oliver15's")
    check(m16["scripts"] == {"logarithm_mysteries": NEW_SCRIPT}, "map: scripts.logarithm_mysteries is 559afa36")
    disp = json.loads(plan["direction-dispositions-proposed.json"])
    check(disp["map_sha256"] == sha(cand[f"{REL}/docs/{MAP16}"]) and all(r["evidence"] == {"path": f"{INSTALL}/bound-plan/hold-audit.json", "sha256": sha(plan["hold-audit.json"])} for r in disp["direction_dispositions"]),
          "dispositions: oliver16's hash; each evidence is the stage09 hold audit at root's install path")
    cd9, cd3 = json.loads(plan["cue-delta.json"]), load(PLAN3 / "cue-delta.json")
    check(all(r9 == r3 for r9, r3 in zip(cd9["cues"], cd3["cues"]) if r9["cue"] not in CHANGED), "cue delta: the 78 other rows are bound-plan03's")
    ch = [r for r in cd9["cues"] if r["cue"] in CHANGED]
    check(all(r["text_sha256"] == sha_t(new[r["cue"]]) and r["current_request"]["path"] is None and r["superseded_request"]["sha256"] == sha_t(old[r["cue"]]) for r in ch),
          "cue delta: 04_4 and 08_5 carry their stage09 hashes, no current request, and their stage08 request as superseded")
    tb = json.loads(plan["take-binding.json"])
    inv = load(REPO / EL / "full80-retained-inventory01.json")
    hashed = {}
    for t in inv["takes"]:
        rel = t["wav"][t["wav"].find("narration_lab/") :]
        hashed[rel] = sha((REPO / rel).read_bytes())
    check(all(hashed[t["wav"][t["wav"].find("narration_lab/") :]] == t["sha256"] for t in inv["takes"]), f"all {len(inv['takes'])} retained WAVs re-hashed here equal root's inventory (hashed, not decoded)")
    ok78 = all(len(r["takes"]) >= 2 and not r["old_text_takes"] and r["text_sha256"] == sha_t(new[r["cue"]]) for r in tb["cues"] if r["cue"] not in CHANGED)
    check(ok78, "take binding: each of the 78 cues has at least two retained Oliver takes of its exact stage09 text")
    old4 = sorted(t["sha256"] for r in tb["cues"] if r["cue"] in CHANGED for t in r["old_text_takes"])
    check(old4 == sorted(OLD_TAKES.values()) and all(not r["takes"] for r in tb["cues"] if r["cue"] in CHANGED), "take binding: 04_4 and 08_5 have no bindable take; their four original takes are listed as preserved")
    check(all(sha((TAKES / n).read_bytes()) == h for n, h in OLD_TAKES.items()), "the four original 04_4 and 08_5 takes are in ui-takes01 with their hashes")
    tokens = [OLD_SCRIPT, OLD_PRODUCT, "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4", "0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c",
              "4040d484c48e930348726fca61a6ee531cdd58f08211e6c6a237c2ad138f1c9a", "narration-direction-map-oliver15.json", sha_t(old["04_4"]), sha_t(old["08_5"])]
    leftovers = []
    for name in ("hold-audit.json", "phrase-anchors.json", "phrase-timings-unmeasured-template.json", "direction-dispositions-proposed.json", f"{REL}/narration_units.json"):
        text = plan[name].decode("utf-8")
        leftovers += [f"{name}:{t[:8]}" for t in tokens if t in text]
    cd_text = json.dumps({k: v for k, v in cd9.items() if k != "current_request_list"} | {"cues": [{k: v for k, v in r.items() if k != "superseded_request"} for r in cd9["cues"]]})
    leftovers += [f"cue-delta.json:{t[:8]}" for t in tokens if t in cd_text]
    check(not leftovers, f"no old script, source, units, map, hold-audit or 04_4/08_5 text hash left in the rebound plan, outside the superseded rows ({leftovers or 'none'})")
    crate_refs = []

    def walk(x, name):
        if isinstance(x, dict):
            if isinstance(x.get("path"), str) and x["path"].startswith(REL + "/") and isinstance(x.get("sha256"), str):
                crate_refs.append((name, x["path"], x["sha256"]))
            for v in x.values():
                walk(v, name)
        elif isinstance(x, list):
            for v in x:
                walk(v, name)

    for name in ("hold-audit.json", "phrase-anchors.json", f"{REL}/docs/{MAP16}"):
        walk(json.loads(plan[name]), name)
    unref = [r for r in crate_refs if r[1] not in cand or sha(cand[r[1]]) != r[2]]
    check(crate_refs and not unref, f"{len(crate_refs)} crate (path, sha256) pairs in the hold audit, anchors and map: each is the candidate's file ({unref or 'all current'})")

    # ------------------------------------------------------------------ consumers
    section("consumers")
    inst = SCRATCH / "install"
    shutil.copytree(DELIVERED / "candidate", inst / INSTALL / "candidate")
    shutil.copytree(DELIVERED / "bound-plan", inst / INSTALL / "bound-plan")
    consumer = r'''
import importlib.util, json, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("coherent_narration", sys.argv[1])
cn = importlib.util.module_from_spec(spec); spec.loader.exec_module(cn)
cn.REPO = Path(sys.argv[2]).resolve()
script, units_path, disp_path = sys.argv[3], sys.argv[4], sys.argv[5]
try:
    _, units = cn.read_units(script, units_path, require_directions=True)
    contract = cn.direction_contract(script, json.loads(Path(units_path).read_text()))
    cn.review_directions(contract, units, json.loads(Path(disp_path).read_text()))
    print("ACCEPTED", len(units), len(contract["resolved"]))
except Exception as e:
    print("REFUSED", type(e).__name__, e)
'''
    lane9 = inst / INSTALL / "candidate"

    def consume(units_path, disp_path, script=None):
        r = subprocess.run([sys.executable, "-I", "-B", "-c", consumer, str(REPO / "tools/coherent_narration.py"), str(lane9),
                            str(script or lane9 / REL / "script.tsv"), str(units_path), str(disp_path)], cwd=inst, capture_output=True, text=True)
        return (r.stdout + r.stderr).strip().replace(str(SCRATCH), "SCRATCH")

    out = consume(lane9 / REL / "narration_units.json", inst / INSTALL / "bound-plan/direction-dispositions-proposed.json")
    check(out == "ACCEPTED 80 15", f"coherent_narration read_units + direction_contract + review_directions accept stage09 as installed: {out}")
    out = consume(STAGE08 / REL / "narration_units.json", inst / INSTALL / "bound-plan/direction-dispositions-proposed.json")
    check(out.startswith("REFUSED") and "current script" in out, f"the same consumers refuse stage08's units on the stage09 script: {out}")
    stale_units = json.loads(cand[f"{REL}/narration_units.json"])
    stale_units["direction_contract"]["map"]["sha256"] = "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4"
    dump(SCRATCH / "units-stale-map.json", stale_units)
    out = consume(SCRATCH / "units-stale-map.json", inst / INSTALL / "bound-plan/direction-dispositions-proposed.json")
    check(out.startswith("REFUSED") and "map changed" in out, f"... and units naming oliver15's hash for oliver16: {out}")
    out = consume(lane9 / REL / "narration_units.json", PLAN3 / "direction-dispositions-proposed.json")
    check(out.startswith("REFUSED"), f"... and bound-plan03's dispositions, whose evidence is bound-plan03's hold audit: {out}")
    for label, lane in (("stage09", lane9), ("stage08", STAGE08)):
        r = subprocess.run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=lane / REL, capture_output=True, text=True)
        check(r.returncode == 0 and r.stdout.strip().endswith("OK"), f"docs/check_cues.py on {label}: exit {r.returncode}, OK")
    r = subprocess.run(RUSTFMT + [str(lane9 / REL / "src/chapters/product_rule.rs"), str(STAGE08 / REL / "src/chapters/product_rule.rs")], capture_output=True, text=True)
    check(r.returncode == 0 and not r.stdout.strip(), "rustfmt +nightly-2026-08-31 --check (edition 2024, width 120): stage09 and stage08 product_rule.rs")

    def mirror_test(text, source):
        opener = re.search(r'let open = text\.find\("([^"]+)"\)', source).group(1)
        scene = source.split("#[cfg(test)]")[0]
        o, close, after = text.find(opener), text.find("The root of a product"), text.find(opener)
        if o < 0 or close < 0:
            return False
        for phrase in ("leaves a half turn", "a factor of minus one"):
            b = text.find(phrase)
            if text.count(phrase) != 1 or not (after < b < close) or text[b - 1] != " ":
                return False
            after = b
        order = [scene.find(n) for n in TEST_NEEDLES]
        return all(x >= 0 for x in order) and order == sorted(order) and len(set(order)) == len(order)

    src9, src8 = cand[f"{REL}/src/chapters/product_rule.rs"].decode(), s08[f"{REL}/src/chapters/product_rule.rs"].decode()
    check(mirror_test(new["08_5"], src9) and mirror_test(old["08_5"], src8) and not mirror_test(new["08_5"], src8) and not mirror_test(old["08_5"], src9),
          "the_half_turn_and_its_sign_wait... mirrored: new text + new test and old + old pass; crossed fail")

    def phrase_seconds(text, phrase, marks, starts, length):
        """product_rule.rs phrase_seconds(), written out: a mark counts only at text.find(phrase)."""
        byte = text.encode().find(phrase.encode())
        if byte in marks:
            return marks[byte], "mark"
        o = opens(text)
        j = max(k for k, at in enumerate(o) if at <= byte)
        close = o[j + 1] if j + 1 < len(o) else len(text.encode())
        t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
        raw = text.encode()
        before = len(raw[o[j] : byte].decode())
        whole = max(len(raw[o[j] : close].decode().rstrip()), 1)
        return t0 + (t1 - t0) * before / whole, "share"

    starts = [0.0, 3.0, 7.0, 11.0, 15.0, 21.0]
    fresh = {267: 16.1, 306: 17.9}
    staled = {260: 16.1, 299: 17.9}
    got = [phrase_seconds(new["08_5"], p, fresh, starts, 25.0)[1] for p in ("leaves a half turn", "a factor of minus one")]
    lost = [phrase_seconds(new["08_5"], p, staled, starts, 25.0)[1] for p in ("leaves a half turn", "a factor of minus one")]
    check(got == ["mark", "mark"] and lost == ["share", "share"], f"phrase_seconds mirrored on 08_5 with illustrative times: marks at 267/306 are read ({got}); at 260/299 both fall back ({lost})")

    # ------------------------------------------------------------------ rejections
    section("rejections")
    tpath = DELIVERED / "bound-plan/phrase-timings-unmeasured-template.json"

    def timings(label, mutate, want, template=True):
        doc = json.loads(tpath.read_text(encoding="utf-8")) if template else json.loads(json.dumps(filled))
        mutate(doc)
        f = SCRATCH / f"timings-{len(results)}.json"
        f.write_text(json.dumps(doc, indent=2, ensure_ascii=False), encoding="utf-8")
        code, out = run(["--validate-timings", str(f)] + (["--template"] if template else []))
        check(code == want, f"{'template' if template else 'measured'} {label}: exit {code}" + ("" if code == 0 else f" ({out.strip().splitlines()[0][5:90]})"))

    def set_marks(cue, marks):
        return lambda d: d["cues"][cue].__setitem__("marks", marks)

    timings("control: the delivered template", lambda d: None, 0)
    timings("08_5 marks left at 260/299", lambda d: [m.__setitem__("byte", b) for m, b in zip(d["cues"]["08_5"]["marks"], (260, 299))], 1)
    timings("08_5 one mark at 260", lambda d: d["cues"]["08_5"]["marks"][0].__setitem__("byte", 260), 1)
    timings("08_5 text hash left at stage08's", lambda d: d["cues"]["08_5"].__setitem__("text_sha256", sha_t(old["08_5"])), 1)
    timings("04_4 text hash left at stage08's", lambda d: d["cues"]["04_4"].__setitem__("text_sha256", sha_t(old["04_4"])), 1)
    timings("08_5 missing a mark", lambda d: d["cues"]["08_5"]["marks"].pop(), 1)
    timings("08_5 extra mark", lambda d: d["cues"]["08_5"]["marks"].append({"byte": 340, "seconds": None, "phrase": "only up to that sign"}), 1)
    timings("08_5 marks renamed", lambda d: d["cues"]["08_5"]["marks"][0].__setitem__("phrase", "a half turn"), 1)
    timings("08_5 marks swapped", lambda d: d["cues"]["08_5"].__setitem__("marks", d["cues"]["08_5"]["marks"][::-1]), 1)
    timings("neighbour 08_4 text hash changed", lambda d: d["cues"]["08_4"].__setitem__("text_sha256", sha_t(new["08_4"] + " ")), 1)
    timings("neighbour 04_5 given a mark", set_marks("04_5", [{"byte": 0, "seconds": None, "phrase": "And"}]), 1)
    timings("cue 12_21 missing", lambda d: d["cues"].pop("12_21"), 1)
    timings("script hash left at b89eab60", lambda d: d.__setitem__("script_sha256", OLD_SCRIPT), 1)
    filled = json.loads(tpath.read_text(encoding="utf-8"))
    filled.pop("claim")
    for row in tb["cues"]:
        e = filled["cues"][row["cue"]]
        e["wav_sha256"] = row["takes"][0]["sha256"] if row["takes"] else sha(f"fixture new take {row['cue']}".encode())
        for k, m in enumerate(e["marks"]):
            m["seconds"] = 1.0 + k
    timings("control: filled with retained takes and two fixture hashes", lambda d: None, 0, template=False)
    timings("08_5 bound to an original take", lambda d: d["cues"]["08_5"].__setitem__("wav_sha256", OLD_TAKES["08_5-download01.wav"]), 1, template=False)
    timings("04_4 bound to an original take", lambda d: d["cues"]["04_4"].__setitem__("wav_sha256", OLD_TAKES["04_4-download02.wav"]), 1, template=False)
    timings("04_4 bound to another cue's retained take", lambda d: d["cues"]["04_4"].__setitem__("wav_sha256", tb["cues"][0]["takes"][0]["sha256"]), 1, template=False)
    timings("neighbour 04_5 bound to a non-retained hash", lambda d: d["cues"]["04_5"].__setitem__("wav_sha256", sha(b"not a retained take")), 1, template=False)
    timings("marks at 260/299 with a new take", lambda d: [m.__setitem__("byte", b) for m, b in zip(d["cues"]["08_5"]["marks"], (260, 299))], 1, template=False)
    timings("08_5 seconds going backwards", lambda d: [m.__setitem__("seconds", s) for m, s in zip(d["cues"]["08_5"]["marks"], (5.0, 4.0))], 1, template=False)
    timings("08_5 seconds not finite", lambda d: d["cues"]["08_5"]["marks"][0].__setitem__("seconds", math.nan), 1, template=False)
    timings("08_5 seconds null in a measured file", lambda d: d["cues"]["08_5"]["marks"][0].__setitem__("seconds", None), 1, template=False)

    base = load(TAKES / "08_5-request01.json")

    def receipt(cue, **change):
        r = dict(base)
        r.update({"cue": cue, "text": new[cue], "text_sha256": sha_t(new[cue]), "script_sha256": NEW_SCRIPT,
                  "script_path": f"{INSTALL}/candidate/{REL}/script.tsv", "utc": "fixture", "state": "fixture"})
        r.update(change)
        return r

    def receipts(label, r, want, lavender=None):
        f = SCRATCH / f"receipt-{len(results)}.json"
        f.write_text(json.dumps(r, indent=2), encoding="utf-8")
        code, out = run(["--validate-receipt", str(f)])
        note = "" if code == 0 else " (" + next((x.strip() for x in out.splitlines() if x.startswith("     ")), "")[:100] + ")"
        check(code == want, f"receipt {label}: exit {code}{note}".replace(f"SCRATCH/{f.name}: ", ""))
        if lavender is not None:
            lav = subprocess.run([sys.executable, "-I", "-B", str(REVIEW / "checks/rebind_acceptance.py"), str(DELIVERED / "candidate" / REL), "--receipt", str(f)], cwd=REPO, capture_output=True, text=True)
            line = next((x for x in lav.stdout.splitlines() if x.startswith(("PASS receipt", "FAIL receipt"))), "")
            check(line.startswith("PASS" if lavender else "FAIL"), f"   Lavender's receipt check agrees: {line[:4]}")

    receipts("control 08_5, Oliver", receipt("08_5"), 0, lavender=True)
    receipts("control 04_4, the long Oliver label", receipt("04_4", voice="Oliver - Clean, British and Steady", format="WAV 48 kHz (Lossless)"), 0)
    receipts("08_5 voiced by Frederick Surrey", receipt("08_5", voice="Frederick Surrey"), 1, lavender=False)
    receipts("08_5 with the stage08 wording", receipt("08_5", text=old["08_5"], text_sha256=sha_t(old["08_5"])), 1)
    receipts("neighbour 04_5", receipt("04_5"), 1)
    receipts("08_5 at stability 0.6", receipt("08_5", stability=0.6), 1)
    receipts("08_5 on model Eleven v3", receipt("08_5", model="Eleven v3"), 1)
    receipts("08_5 as MP3", receipt("08_5", format="MP3 44.1 kHz"), 1)
    receipts("08_5 with previous_text context", receipt("08_5", previous_text=new["08_4"]), 1)
    receipts("08_5 bound to script b89eab60", receipt("08_5", script_sha256=OLD_SCRIPT), 1)
    receipts("08_5 noting a Frederick history", receipt("08_5", history="Frederick Surrey take 1"), 1)

    def verify(label, mutate, want):
        target = SCRATCH / f"verify-{len(results)}"
        shutil.copytree(DELIVERED, target)
        mutate(target)
        code, out = run(["--verify", str(target)])
        first = next((x for x in out.splitlines() if x.startswith(("EXTRA", "MISSING", "DIFFERS"))), "")
        check(code == want, f"--verify {label}: exit {code}" + (f" ({first.split('  (')[0][:100]})" if first else ""))

    def edit_json(rel, fn):
        def go(t):
            d = load(t / rel)
            fn(d)
            dump(t / rel, d)
        return go

    U, M = f"candidate/{REL}/narration_units.json", f"candidate/{REL}/docs/{MAP16}"
    verify("control: a copy of stage09/", lambda t: None, 0)
    verify("units keeping script b89eab60", edit_json(U, lambda d: d.__setitem__("script_sha256", OLD_SCRIPT)), 1)
    verify("units keeping 08_5's stage08 text", edit_json(U, lambda d: d["units"][54].__setitem__("text", old["08_5"])), 1)
    verify("map keeping product_rule.rs f2c8684a", edit_json(M, lambda d: d["visual_holds"][9].__setitem__("sha256", OLD_PRODUCT)), 1)
    verify("template keeping 260/299", edit_json("bound-plan/phrase-timings-unmeasured-template.json", lambda d: [m.__setitem__("byte", b) for m, b in zip(d["cues"]["08_5"]["marks"], (260, 299))]), 1)
    verify("anchors keeping 260", edit_json("bound-plan/phrase-anchors.json", lambda d: d["anchors"][10].__setitem__("byte", 260)), 1)
    verify("hold audit keeping f2c8684a", edit_json("bound-plan/hold-audit.json", lambda d: d["prediction_holds"][9].__setitem__("source_sha256", OLD_PRODUCT)), 1)
    verify("dispositions keeping bound-plan03's evidence", edit_json("bound-plan/direction-dispositions-proposed.json", lambda d: d["direction_dispositions"][0]["evidence"].__setitem__("sha256", "4040d484c48e930348726fca61a6ee531cdd58f08211e6c6a237c2ad138f1c9a")), 1)
    neighbour = f"candidate/{REL}/script.tsv"
    verify("neighbour 04_5 reworded in the script", lambda t: (t / neighbour).write_bytes((t / neighbour).read_bytes().replace(b"And this is the ruler again.", b"And this is the ruler, again.")), 1)
    verify("a stray .orig file", lambda t: (t / f"candidate/{REL}/src/chapters/product_rule.rs.orig").write_bytes(s08[f"{REL}/src/chapters/product_rule.rs"]), 1)
    verify("oliver16 missing", lambda t: (t / M).unlink(), 1)

    # ------------------------------------------------------------------ lavender
    section("lavender")
    sums = (REVIEW / "SHA256SUMS").read_bytes()
    listed = [line.split("  ", 1) for line in sums.decode().splitlines()]
    check(sha(sums) == REVIEW_SUMS and all(sha((REVIEW / p).read_bytes()) == h for h, p in listed), f"LavenderPike's review bundle is as sealed (SHA256SUMS {REVIEW_SUMS[:8]}, {len(listed)} files)")
    args = ["--anchors", "phrase-anchors.json", "--template", "phrase-timings-unmeasured-template.json", "--hold-audit", "hold-audit.json"]

    def lavender(crate, plan_dir):
        argv = [a if a.startswith("--") else str(plan_dir / a) for a in args]
        r = subprocess.run([sys.executable, "-I", "-B", str(REVIEW / "checks/rebind_acceptance.py"), str(crate), *argv], cwd=REPO, capture_output=True, text=True)
        return r.returncode, [x for x in r.stdout.splitlines() if x.startswith("FAIL")], sum(1 for x in r.stdout.splitlines() if x.startswith("PASS"))

    code, fails, passes = lavender(DELIVERED / "candidate" / REL, DELIVERED / "bound-plan")
    check(code == 1 and fails == ["FAIL the units' other fields are stage08's"], f"rebind_acceptance.py on the delivered candidate: {passes} PASS, and its one FAIL is the units' other fields: {fails}")
    diff = sorted(k for k in set(units) | set(u08) if k not in ("units", "script_sha256", "direction_contract") and units.get(k) != u08.get(k))
    check(diff == ["scope"], f"the only such field that differs is scope: {diff}")
    restored = SCRATCH / "lavender-scope"
    shutil.copytree(DELIVERED / "candidate" / REL, restored)
    u = load(restored / "narration_units.json")
    u["scope"] = u08["scope"]
    dump(restored / "narration_units.json", u)
    code, fails, passes = lavender(restored, DELIVERED / "bound-plan")
    check(code == 0 and not fails, f"with stage08's scope sentence put back (a scratch copy), rebind_acceptance.py passes: {passes} PASS")

    # ------------------------------------------------------------------ hygiene
    section("hygiene")
    handoff = REPO / HERE / "HANDOFF.md"
    check(handoff.is_file(), "HANDOFF.md is present")
    authored = [REPO / HERE / "integrate_clarity.py", REPO / HERE / "checks/run_checks.py", REPO / HERE / "pins.sha256"]
    authored += [handoff] if handoff.is_file() else []
    authored +=[DELIVERED / "bound-plan" / k for k in plan] + [DELIVERED / "candidate" / REL / "narration_units.json", DELIVERED / "candidate" / REL / "docs" / MAP16]
    machine = re.compile(r"/Users/|/private/|/home/|jemanuel|mortimeremanuel|scratchpad|claude-501|/tmp/|@gmail|@[a-z0-9-]+\.(com|org|net)\b")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern

    def scanned(p):
        """The file's text, without the two pattern lines above when the file is this one."""
        return "\n".join(line for line in p.read_text(encoding="utf-8").splitlines() if not line.endswith("# hygiene-pattern"))

    bad_m = [p.name for p in authored if machine.search(scanned(p))]
    bad_p = [p.name for p in authored if pronoun.search(scanned(p))]
    check(not bad_m, f"no machine path, user name or address in {len(authored)} authored files ({bad_m or 'none'})")
    check(not bad_p, f"no gendered pronoun in them ({bad_p or 'none'})")
    hidden = [p for p in (REPO / HERE).rglob("*") if p.name.startswith(".") or p.suffix in (".rej", ".orig") and "stage09/candidate" not in p.as_posix()]
    stage08_orig = sorted(k for k in s08 if k.endswith(".orig"))
    check(not hidden and sorted(k for k in cand if k.endswith((".orig", ".rej"))) == stage08_orig,
          f"no hidden, .rej or .orig file of mine; the candidate's only .orig is stage08's own {stage08_orig}")

    print("RESULT " + ("PASS" if all(results) else f"FAIL ({results.count(False)} of {len(results)})") + f", {len(results)} checks")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
