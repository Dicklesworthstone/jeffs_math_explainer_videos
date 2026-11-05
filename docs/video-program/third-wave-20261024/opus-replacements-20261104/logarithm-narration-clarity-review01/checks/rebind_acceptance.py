"""Is a rebound crate exactly stage08 + the clarity patch (2463334a) + a consistent rebind? Read-only. Run from the repo root:

    python3 -I -B .../logarithm-narration-clarity-review01/checks/rebind_acceptance.py CRATE \
        [--anchors F] [--template F] [--hold-audit F] [--receipt F]...

CRATE is a crate folder (an explainer_corpus/logarithm_mysteries) such as WhiteSwan's integration candidate or root's stage09.
The optional files are the rebound phrase-anchors.json, unmeasured phrase-timings template and hold-audit.json; each
--receipt is an ElevenLabs request receipt for a new 04_4 or 08_5 take, in ui-takes01's layout. Exit 0 pass, 1 fail, 2 usage.
(For a filled phrase-timings.json, the anchors section gives the bytes its 08_5 marks must use: 267 and 306.)

Pass means, against root's stage08 candidate and bound-plan03 (read from the repo; nothing is pinned to the candidate under test):
  files     every stage08 file is present with its stage08 hash, except script.tsv (559afa36), product_rule.rs (bc9fbe1d),
            narration_units.json and the map the units name; the only possible new file is that map;
  script    04_4 and 08_5 are the reviewed texts (7e4e9612, 8e26bcb0); the 78 other rows are stage08's bytes;
  units     80 units with stage08's ids and line ids; texts are the new rows; both script hashes are 559afa36; the contract
            names the map present in the crate by path and hash;
  map       same keys, overrides and hold kinds as oliver15; scripts = 559afa36; each visual hold equals oliver15's row
            except sha256/evidence.sha256 (and wording of basis/reason, reported); every row's file hash is the crate's
            file and its interval, recomputed with binder03's formula, is the recorded one; shared helpers' hashes are current;
  anchors   script 559afa36; the 33 anchors of bound-plan03; each byte, byte_end, word_index and sentence_index recomputed
            from the new text by phrase_seconds' rule; text and source hashes current; 08_5 at bytes 267 and 306;
  template  one entry per cue; text hashes of the new rows; marks exactly at the anchors' bytes (08_5: 267, 306);
            the 78 other entries equal bound-plan03's;
  holds     script 559afa36; words 4594 (08_5: 83); each prediction/reading hold's source hash is the crate's file and its
            interval is unchanged;
  receipts  text and hash of the new row; Oliver, Eleven v4, stability 0.5, similarity 0.75, clone strength 0, 48 kHz WAV;
            and the four original 04_4/08_5 takes are still present with their hashes.
Source text and hashes only. A PASS says nothing about compiling, rendering, how a take sounds, or timing.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

REPO = Path.cwd()
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
REL = "explainer_corpus/logarithm_mysteries"
STAGE08 = REPO / EL / "combined-repair-stage08/candidate" / REL
PLAN = REPO / EL / "combined-repair-stage07/bound-plan03"
TAKES = REPO / EL / "ui-takes01"
MAP15 = "docs/narration-direction-map-oliver15.json"
NEW_SCRIPT = "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a"
NEW_PRODUCT = "bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635"
NEW_TEXT = {"04_4": "7e4e96121ff25f997a75bc882f70884bd61b3b039203e8df9c9d46fe545d2e83",
            "08_5": "8e26bcb04dba0673057649b76a1fce2ce0375bcc1ff1a8fffb7d8379c13f4d82"}
ORIGINAL_TAKES = {"04_4-download01.wav": "de3abb952c675c2112c949813b2d70f962b95a3adf90d8f3289c6a4e29dce9ab",
                  "04_4-download02.wav": "573d2360f72d4c8d451cd684ff8b3febf68168b24add6b070886433080185164",
                  "08_5-download01.wav": "9b5d2b73ec6400569a32e03fe2c0d2b91c19308565ea6ee433b9ca99ca0d1ae5",
                  "08_5-download02.wav": "5ec634e9c591e1768d69ea5195e6b07b91a3da1c6c7cecefa07799b3a269c493"}
STAGE08_SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
OLIVER = {"model": "Eleven v4", "stability": 0.5, "similarity": 0.75, "clone_strength": 0}


def sha_b(b):
    return hashlib.sha256(b).hexdigest()


def sha_t(s):
    return sha_b(s.encode("utf-8"))


def rows(path):
    out = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if line and not line.startswith("#") and "\t" in line:
            cue, text = line.split("\t", 1)
            out[cue] = text
    return out


def opens(text):
    b = text.encode("utf-8")
    return [0] + [i + 2 for i in range(len(b) - 1) if b[i] in b".?!" and b[i + 1] == 0x20]


def direction_words(text):
    """tools/coherent_narration.py direction_words(), which binder03 uses for word_index ("sixty-six" is two words)."""
    return [m.lower().replace("’", "'") for m in re.findall(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?", text)]


def anchor(text, phrase):
    b = text.encode("utf-8")
    byte = b.find(phrase.encode("utf-8"))
    if byte < 0:
        return None
    return {"byte": byte, "byte_end": byte + len(phrase.encode("utf-8")),
            "word_index": len(direction_words(b[:byte].decode("utf-8"))),
            "sentence_index": max(k for k, o in enumerate(opens(text)) if o <= byte)}


def interval(lines, say, marker):
    return sha_t("\n".join(lines[say - 1:marker]))


def parse(argv):
    if not argv or argv[0].startswith("--"):
        return None
    opts = {"crate": Path(argv[0]), "anchors": None, "template": None, "hold-audit": None, "receipt": []}
    rest = argv[1:]
    while rest:
        if len(rest) < 2 or rest[0][2:] not in opts or rest[0] == "--crate":
            return None
        key, value = rest[0][2:], Path(rest[1])
        if key == "receipt":
            opts["receipt"].append(value)
        else:
            opts[key] = value
        rest = rest[2:]
    return opts


def main(argv):
    opts = parse(argv)
    if opts is None or not (opts["crate"] / "script.tsv").is_file():
        print(__doc__.split("\n\n")[1], file=sys.stderr)
        return 2
    crate = opts["crate"]
    fails = []

    def check(ok, label):
        print(("PASS " if ok else "FAIL ") + label)
        if not ok:
            fails.append(label)

    def file_sha(repo_rel):
        """A repo-relative path, read from the crate under test when it lies in the crate, else from the repo."""
        p = crate / repo_rel[len(REL) + 1:] if repo_rel.startswith(REL + "/") else REPO / repo_rel
        return sha_b(p.read_bytes()) if p.is_file() else None

    base = {str(p.relative_to(STAGE08)): sha_b(p.read_bytes()) for p in sorted(STAGE08.rglob("*")) if p.is_file()}
    disk = {str(p.relative_to(crate)): sha_b(p.read_bytes()) for p in sorted(crate.rglob("*")) if p.is_file()}
    old, new = rows(STAGE08 / "script.tsv"), rows(crate / "script.tsv")
    units = json.loads((crate / "narration_units.json").read_text(encoding="utf-8"))
    units08 = json.loads((STAGE08 / "narration_units.json").read_text(encoding="utf-8"))
    contract = units.get("direction_contract", {}).get("map", {})
    map_rel = str(contract.get("path", ""))[len(REL) + 1:]

    print("== files")
    check(sha_b((STAGE08 / "script.tsv").read_bytes()) == STAGE08_SCRIPT, "the stage08 reference is still script b89eab60")
    changed = sorted(p for p in base if disk.get(p) != base[p])
    added = sorted(set(disk) - set(base))
    check(not [p for p in base if p not in disk], "no stage08 file is missing")
    allowed = {"script.tsv", "src/chapters/product_rule.rs", "narration_units.json", map_rel}
    check(set(changed) <= allowed and {"script.tsv", "src/chapters/product_rule.rs", "narration_units.json"} <= set(changed),
          f"changed: {changed} (allowed: script, product_rule.rs, units, the units' map)")
    check(added in ([], [map_rel]), f"new files: {added or 'none'} (allowed: only the units' map)")

    print("== script")
    check(disk.get("script.tsv") == NEW_SCRIPT and disk.get("src/chapters/product_rule.rs") == NEW_PRODUCT,
          f"script.tsv {str(disk.get('script.tsv'))[:8]} (want 559afa36), product_rule.rs "
          f"{str(disk.get('src/chapters/product_rule.rs'))[:8]} (want bc9fbe1d)")
    moved = sorted(c for c in old if c not in NEW_TEXT and new.get(c) != old[c])
    check(list(new) == list(old) and not moved and all(sha_t(new[c]) == h for c, h in NEW_TEXT.items()),
          f"04_4 and 08_5 are the reviewed texts; the 78 other rows are stage08's bytes (moved: {moved or 'none'})")

    print("== units")
    shape = [(u["id"], u["line_ids"]) for u in units.get("units", [])]
    check(shape == [(u["id"], u["line_ids"]) for u in units08["units"]], "80 units with stage08's ids and line ids, in order")
    stale = [u["id"] for u in units.get("units", []) if u["text"] != " ".join(new.get(i, "") for i in u["line_ids"])]
    check(not stale, f"every unit text is its new script row ({stale or 'all'})")
    differs = sorted(u["id"] for u, w in zip(units.get("units", []), units08["units"]) if u["text"] != w["text"])
    check(differs == ["oliver_04_4", "oliver_08_5"], f"only oliver_04_4 and oliver_08_5 change text ({differs})")
    check(units.get("script_sha256") == NEW_SCRIPT == units.get("direction_contract", {}).get("script_sha256"),
          "units script_sha256 and direction_contract.script_sha256 are 559afa36")
    check(map_rel.startswith("docs/narration-direction-map-") and disk.get(map_rel) == contract.get("sha256"),
          f"the contract names {map_rel or 'nothing'} by path and hash, and that file is in the crate")
    keep = {k: v for k, v in units.items() if k not in ("units", "script_sha256", "direction_contract")}
    keep08 = {k: v for k, v in units08.items() if k not in ("units", "script_sha256", "direction_contract")}
    check(keep == keep08, "the units' other fields are stage08's")

    print("== map")
    hold = json.loads((PLAN / "hold-audit.json").read_text(encoding="utf-8"))
    says = {(h["cue"], h["file"]): h["say_line"] for h in hold["prediction_holds"] + hold["reading_holds"]}
    m15 = json.loads((STAGE08 / MAP15).read_text(encoding="utf-8"))
    m = json.loads((crate / map_rel).read_text(encoding="utf-8")) if map_rel and (crate / map_rel).is_file() else {}
    check(set(m) == set(m15) and m.get("overrides") == m15["overrides"] and m.get("hold_kinds") == m15["hold_kinds"],
          "same keys, overrides and hold kinds as oliver15")
    check(m.get("scripts") == {"logarithm_mysteries": NEW_SCRIPT}, "scripts.logarithm_mysteries is 559afa36")
    helpers = m.get("shared_helpers", [])
    check([h["path"] for h in helpers] == [h["path"] for h in m15["shared_helpers"]]
          and all(file_sha(h["path"]) == h["sha256"] for h in helpers), "shared helpers: oliver15's list, each hash current")
    vh, vh15 = m.get("visual_holds", []), m15["visual_holds"]
    check(len(vh) == len(vh15), f"{len(vh)} visual holds (oliver15 has {len(vh15)})")
    reworded = []
    for r, r15 in zip(vh, vh15):
        loose = {"sha256", "evidence", "basis", "reason"}
        same = {k: v for k, v in r.items() if k not in loose} == {k: v for k, v in r15.items() if k not in loose}
        reworded += [f"{r['cue']}.{k}" for k in ("basis", "reason") if r.get(k) != r15.get(k)]
        current = file_sha(r["path"])
        src = (crate / r["path"][len(REL) + 1:]).read_text(encoding="utf-8").split("\n")
        say = says.get((r["cue"], Path(r["path"]).name))
        recomputed = interval(src, say, r["line"]) if say else None
        check(same and r["sha256"] == r["evidence"]["sha256"] == current and r["evidence"]["path"] == r["path"]
              and recomputed == r["interval_sha256"] == r15["interval_sha256"],
              f"hold {r['cue']} {Path(r['path']).name}:{r['line']}: file {str(r['sha256'])[:8]} is current; interval "
              f"{str(recomputed)[:8]} recomputed = recorded; other fields oliver15's")
    if reworded:
        print(f"NOTE basis/reason wording differs from oliver15 in {reworded}")

    plan_anchors = json.loads((PLAN / "phrase-anchors.json").read_text(encoding="utf-8"))
    new_bytes = {}
    if opts["anchors"]:
        print("== anchors")
        a = json.loads(opts["anchors"].read_text(encoding="utf-8"))
        check(a.get("script_sha256") == NEW_SCRIPT, "anchors bind script 559afa36")
        # keyed by (cue, phrase): some anchors (10_3's) have no const
        got = {(x["cue"], x["phrase"]): x for x in a.get("anchors", [])}
        want = {(x["cue"], x["phrase"]): x for x in plan_anchors["anchors"]}
        check(set(got) == set(want) and len(a.get("anchors", [])) == len(plan_anchors["anchors"]),
              f"the same {len(want)} anchors as bound-plan03 ({len(a.get('anchors', []))} found)")
        wrong = []
        for key, w in want.items():
            g = got.get(key, {})
            mine = anchor(new[key[0]], w["phrase"])
            ok = mine is not None and all(g.get(k) == v for k, v in mine.items()) and g.get("const") == w["const"] \
                and g.get("hook_sites") == w["hook_sites"] and g.get("cue_text_sha256") == sha_t(new[key[0]]) \
                and g.get("source_sha256") == file_sha(w["source_path"]) and g.get("source_path") == w["source_path"]
            if key[0] == "08_5":
                ok = ok and g.get("byte") == {"leaves a half turn": 267, "a factor of minus one": 306}.get(key[1])
                check(ok, f"anchor 08_5 {w['const']}: byte {g.get('byte')}, word {g.get('word_index')}, sentence "
                          f"{g.get('sentence_index')}; text and source hashes current")
            elif not ok:
                wrong.append(f"{key[0]} {key[1]!r}")
            new_bytes.setdefault(key[0], []).append(mine["byte"] if mine else None)
        check(not wrong, f"the other {sum(1 for k in want if k[0] != '08_5')} anchors equal their recomputed values "
                         f"({wrong or 'all'})")
    else:
        for w in plan_anchors["anchors"]:
            got = anchor(new[w["cue"]], w["phrase"])
            new_bytes.setdefault(w["cue"], []).append(got["byte"] if got else None)

    if opts["template"]:
        print("== template")
        t = json.loads(opts["template"].read_text(encoding="utf-8"))
        t3 = json.loads((PLAN / "phrase-timings-unmeasured-template.json").read_text(encoding="utf-8"))
        cues = t.get("cues", {})
        check(t.get("script_sha256") in (NEW_SCRIPT, None) and sorted(cues) == sorted(new),
              f"one entry per cue ({len(cues)}); script hash {str(t.get('script_sha256'))[:8]}")
        bad_text = sorted(c for c in new if cues.get(c, {}).get("text_sha256") != sha_t(new[c]))
        check(not bad_text, f"every entry's text_sha256 is its new row ({bad_text or 'all'})")
        bad_marks = sorted(c for c in new if [x["byte"] for x in cues.get(c, {}).get("marks", [])] != sorted(new_bytes.get(c, [])))
        check(not bad_marks, f"every cue's marks sit exactly at its anchors' bytes in the new text ({bad_marks or 'all'})")
        check([x["byte"] for x in cues.get("08_5", {}).get("marks", [])] == [267, 306], "08_5 marks at bytes 267 and 306")
        others = sorted(c for c in new if c not in NEW_TEXT and cues.get(c) != t3["cues"][c])
        check(not others, f"the 78 other entries are bound-plan03's ({others or 'all equal'})")

    if opts["hold-audit"]:
        print("== hold audit")
        h = json.loads(opts["hold-audit"].read_text(encoding="utf-8"))
        check(h.get("script_sha256") == NEW_SCRIPT and h.get("counts", {}).get("words_total") == 4594,
              f"script 559afa36, words_total {h.get('counts', {}).get('words_total')} (want 4594)")
        words = {r["cue"]: r["words"] for r in h.get("cues", [])}
        check(words == {c: len(t.split()) for c, t in new.items()} and words.get("08_5") == 83, "per-cue words are the new rows'")
        for r in h.get("prediction_holds", []) + h.get("reading_holds", []):
            path = f"{REL}/src/chapters/{r['file']}"
            src = (crate / "src/chapters" / r["file"]).read_text(encoding="utf-8").split("\n")
            ok = r["source_sha256"] == file_sha(path) and interval(src, r["say_line"], r["marker_line"]) == r["interval_sha256"]
            if r["file"] == "product_rule.rs" or not ok:
                check(ok, f"hold {r['cue']} {r['file']}: source {r['source_sha256'][:8]} current, interval unchanged")

    if opts["receipt"]:
        print("== receipts")
        for path in opts["receipt"]:
            r = json.loads(path.read_text(encoding="utf-8"))
            cue = r.get("cue")
            ok = cue in NEW_TEXT and r.get("text") == new.get(cue) and r.get("text_sha256") == NEW_TEXT[cue] \
                and str(r.get("voice", "")).startswith("Oliver") and all(r.get(k) == v for k, v in OLIVER.items()) \
                and "WAV" in str(r.get("format", "")) and "48" in str(r.get("format", ""))
            check(ok, f"receipt {path.name}: cue {cue}, voice {r.get('voice')!r}, model {r.get('model')!r}, "
                      f"text {str(r.get('text_sha256'))[:8]}")
        kept = all((TAKES / n).is_file() and sha_b((TAKES / n).read_bytes()) == h for n, h in ORIGINAL_TAKES.items())
        check(kept, "the four original 04_4/08_5 Oliver takes are present with their hashes")

    print("REBIND:", "PASS" if not fails else f"FAIL ({len(fails)})")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
