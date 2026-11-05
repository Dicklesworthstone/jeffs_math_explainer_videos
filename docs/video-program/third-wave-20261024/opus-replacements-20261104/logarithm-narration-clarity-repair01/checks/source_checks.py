#!/usr/bin/env python3
"""Source-only checks for logarithm-narration-clarity.patch. Compiles, renders and synthesizes nothing.

    python3 -I -B source_checks.py ROOT_LANE NEW_EMPTY_SCRATCH_DIR [DUMP_INVENTORY_TO]

ROOT_LANE: root's narration_lab/.../elevenlabs-logarithm-rebuild-20261104 folder, read only. The
patch goes against its combined-repair-stage08/candidate. NEW_EMPTY_SCRATCH_DIR: a folder that does
not exist yet; two copies of the candidate are made there, one left as it is (the control) and one
patched. DUMP_INVENTORY_TO, optional: write the inventory this script computes to that new file
instead of comparing it with ../binding-inventory.json.

Checks:
  * baseline: the candidate is root's stage08 (integration01 rows, the baseline rows, and the
    units -> map -> script hash chain);
  * evidence: the recognition review's four transcripts say "space" for "base" and "Having" for
    "Halving", and the four takes it read were made from the current (old) cue text;
  * replay: `git apply --check` and `patch -p1` take the patch; the expected rows match;
  * wording: only 04_4 and 08_5 change, each by exactly one replacement inside one sentence; the
    sentence count, every other sentence and the mathematics stay; no digits; at most 100 words;
    04_4 now says the words of its own on-screen card;
  * hooks: the candidate's check_cues.py passes, unchanged in output, on both copies; a mirror of
    the Rust test the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays passes
    on old text with the old opener and on new text with the new one, and fails crossed (so the
    test edit is needed, and enough); only three product_rule.rs lines change, no line moves, and
    the changed lines are a comment and the test; rustfmt (pinned nightly, the repository's
    width) finds nothing in either copy;
  * bindings: every hash, byte, word index and word count that the change moves, recomputed from
    source with formulas that first reproduce root's recorded values on the unpatched copy; the
    map's interval hashes are recomputed and do not change;
  * inventory: all of that equals ../binding-inventory.json.
A PASS says nothing about compiling, rendering, how any voice says the words, or timing.
"""
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

if len(sys.argv) not in (3, 4):
    sys.exit(__doc__)
LANE, SCRATCH = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
DUMP = Path(sys.argv[3]).resolve() if len(sys.argv) == 4 else None
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
REPO = BUNDLE.parents[4]
PATCH = BUNDLE / "logarithm-narration-clarity.patch"
CAND = LANE / "combined-repair-stage08" / "candidate"
PLAN = LANE / "combined-repair-stage07" / "bound-plan03"
TAKES = LANE / "ui-takes01"
RECOG = LANE / "targeted-recognition-small01"
REQUESTS = LANE / "narration-request-final80-02" / "requests.json"
CRATE = "explainer_corpus/logarithm_mysteries"
SCRIPT = f"{CRATE}/script.tsv"
UNITS = f"{CRATE}/narration_units.json"
MAP = f"{CRATE}/docs/narration-direction-map-oliver15.json"
PRODUCT = f"{CRATE}/src/chapters/product_rule.rs"
UNDOING = f"{CRATE}/src/chapters/undoing_growth.rs"
TOOLCHAIN = "+nightly-2026-08-31"
CHANGES = {"04_4": ("this base", "base e"), "08_5": ("Halving", "Taking half of")}
OPENER = {"old": "Halving the lost lap", "new": "Taking half of the lost lap"}
CLOSE = "The root of a product"
PHRASES = {"HALF_TURN_PHRASE": "leaves a half turn", "FACTOR_PHRASE": "a factor of minus one"}
CARD = "that is why base e is called natural"
MATHS = {
    "04_4": ["slope equal to its height", "slope one over x", "At one, that slope is exactly one.",
             "the natural logarithm of one point zero one is almost exactly zero point zero one",
             "A small percentage change becomes the same small step", "is called natural."],
    "08_5": ["a square root is e to the power of half a logarithm", "principal angle pi",
             "the two roots together turn by pi and point at minus one",
             "has principal angle zero, not two pi", "the lost lap leaves a half turn",
             "a half turn is a factor of minus one",
             "The root of a product equals the product of the roots only up to that sign."],
}
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha(path):
    return sha_bytes(path.read_bytes())


def sha_text(s):
    return sha_bytes(s.encode("utf-8"))


def rows(path):
    return [line.split(maxsplit=1) for line in path.read_text().splitlines()]


def run(args, cwd):
    done = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return done.returncode, (done.stdout + done.stderr).strip()


def script_rows(path):
    out = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if line and not line.startswith("#") and "\t" in line:
            cue, text = line.split("\t", 1)
            out[cue] = text
    return out


def sentences(text):
    # the kit's rule, as product_rule.rs phrase_seconds counts it: a sentence opens after . ? or !
    # and a space
    opens = [0] + [i + 2 for i in range(len(text) - 1) if text[i] in ".?!" and text[i + 1] == " "]
    return [text[a:b].strip() for a, b in zip(opens, opens[1:] + [len(text)])]


def words(text):
    return len(text.split())


def anchor(text, phrase):
    raw = text.encode("utf-8")
    byte = raw.find(phrase.encode("utf-8"))
    before = raw[:byte].decode("utf-8")
    opens = [0] + [i + 2 for i in range(len(raw) - 1) if raw[i:i + 1] in (b".", b"?", b"!") and raw[i + 1:i + 2] == b" "]
    return {"byte": byte, "byte_end": byte + len(phrase.encode("utf-8")), "word_index": words(before),
            "sentence_index": max(j for j, o in enumerate(opens) if o <= byte)}


def rust_test_mirror(text, opener):
    """The text half of product_rule.rs's the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays."""
    open_at, close = text.find(opener), text.find(CLOSE)
    if open_at < 0 or close < 0:
        return False
    after = open_at
    for phrase in PHRASES.values():
        byte = text.find(phrase)
        if text.count(phrase) != 1 or not (after < byte < close) or text[byte - 1] != " ":
            return False
        after = byte
    return True


def interval(lines, say, mark):
    return sha_text("\n".join(lines[say - 1:mark]))


def receipt_text_sha(receipt):
    if "text" in receipt:
        return sha_text(receipt["text"])
    return receipt.get("text_sha256") or receipt.get("sha256")


print("-- baseline")
for digest, rel in rows(BUNDLE / "baseline-candidate.sha256"):
    check(sha(CAND / rel) == digest, f"candidate {rel} is the baseline {digest[:12]}")
integration = json.loads((LANE / "combined-repair-stage08" / "integration01.json").read_text())
for row in integration["rows"]:
    check(sha(CAND / row["path"]) == row["sha256"], f"candidate {row['path']} is stage08's {row['sha256'][:12]}")
units = json.loads((CAND / UNITS).read_text())
dmap = json.loads((CAND / MAP).read_text())
old_script_sha = sha(CAND / SCRIPT)
check(units["script_sha256"] == old_script_sha == units["direction_contract"]["script_sha256"]
      == dmap["scripts"]["logarithm_mysteries"], f"units, contract and map bind the script {old_script_sha[:12]}")
check(units["direction_contract"]["map"]["path"] == MAP and units["direction_contract"]["map"]["sha256"] == sha(CAND / MAP),
      "the units' contract binds this map by path and hash")

print("-- evidence (machine recognition, not listening)")
old_rows = script_rows(CAND / SCRIPT)
review = json.loads((RECOG / "review01.json").read_text())
recog_inputs = {Path(f["path"]).name: f["sha256"] for f in json.loads((RECOG / "inputs.json").read_text())["files"]}
heard = {"04_4": ("this space is called", "this base is called"), "08_5": ("Having the lost lap", "Halving the lost lap")}
takes = []
for cue, (said, written) in heard.items():
    rows_for = [r for r in review["rows"] if Path(r["file"]).name.startswith(cue + "-")]
    check(len(rows_for) == 2 and all(said in r["transcript"] and written not in r["transcript"] for r in rows_for),
          f"{cue}: both small.en transcripts have {said!r} where the script has {written!r}")
    check(written in old_rows[cue], f"{cue}: the current script says {written!r}")
    downloads = json.loads((TAKES / f"{cue}-downloads01.json").read_text())["downloads"]
    request = json.loads((TAKES / f"{cue}-request01.json").read_text())
    check(request["text"] == old_rows[cue] and request["text_sha256"] == sha_text(old_rows[cue]),
          f"{cue}: its takes were requested with the current text {sha_text(old_rows[cue])[:12]}")
    for d in downloads:
        name = Path(d["retained"]).name
        check(sha(TAKES / name) == d["sha256"] == recog_inputs[name], f"{cue}: {name} is the take the review read")
        takes.append({"cue": cue, "file": f"ui-takes01/{name}", "sha256": d["sha256"], "voice": "Oliver"})
print("  review note, verbatim: " + review["review"])

print("-- replay")
if SCRATCH.exists():
    sys.exit(f"{SCRATCH.name} exists; give a new folder")
control, patched = SCRATCH / "control", SCRATCH / "patched"
shutil.copytree(CAND, control)
shutil.copytree(CAND, patched)
leftovers = sorted(str(p.relative_to(patched)) for p in patched.rglob("*") if p.suffix in (".rej", ".orig"))
code, out = run(["git", "apply", "--check", str(PATCH)], patched)
check(code == 0, f"git apply --check exit {code} {out}")
code, out = run(["patch", "-p1", "--forward", "--batch", "-i", str(PATCH)], patched)
check(code == 0, f"patch -p1 exit {code}: {' / '.join(out.splitlines())}")
for digest, rel in rows(BUNDLE / "expected-candidate.sha256"):
    check(sha(patched / rel) == digest, f"patched {rel} is {digest[:12]}")
after = sorted(str(p.relative_to(patched)) for p in patched.rglob("*") if p.suffix in (".rej", ".orig"))
check(after == leftovers, f"no new .rej or .orig files (the candidate already had {leftovers})")
changed = sorted(str(p.relative_to(patched)) for p in patched.rglob("*")
                 if p.is_file() and p.read_bytes() != (control / p.relative_to(patched)).read_bytes())
check(changed == sorted([SCRIPT, PRODUCT]), f"exactly two files change: {changed}")

print("-- wording")
new_rows = script_rows(patched / SCRIPT)
check(list(new_rows) == list(old_rows) and len(new_rows) == 80, f"same {len(new_rows)} cues in the same order")
check([c for c in old_rows if old_rows[c] != new_rows[c]] == list(CHANGES), "only 04_4 and 08_5 change")
cue_rows = []
for cue, (old_words, new_words) in CHANGES.items():
    old, new = old_rows[cue], new_rows[cue]
    check(old.count(old_words) == 1 and new == old.replace(old_words, new_words),
          f"{cue}: one replacement, {old_words!r} -> {new_words!r}, and nothing else")
    so, sn = sentences(old), sentences(new)
    moved = [j for j in range(len(so)) if so[j] != sn[j]] if len(so) == len(sn) else None
    check(moved is not None and len(moved) == 1, f"{cue}: {len(so)} sentences before and after, one of them reworded")
    check(not re.search(r"[0-9]", new) and words(new) <= 100, f"{cue}: no digits, {words(new)} words (at most 100)")
    kept = [m for m in MATHS[cue] if m in old and m in new]
    check(kept == MATHS[cue], f"{cue}: all {len(MATHS[cue])} statements of the mathematics are still said")
    cue_rows.append({"cue": cue, "sentence_index": moved[0], "sentences": len(sn), "old_sentence": so[moved[0]],
                     "new_sentence": sn[moved[0]], "old_text_sha256": sha_text(old), "new_text_sha256": sha_text(new),
                     "words": [words(old), words(new)], "chars": [len(old), len(new)]})
check("this base" not in new_rows["04_4"] and "Halving" not in new_rows["08_5"], "the two misheard words are gone")
undoing = (patched / UNDOING).read_text()
check(undoing.count(f'"{CARD}"') == 1 and new_rows["04_4"].endswith("which is why base e is called natural."),
      f"04_4 now ends on its card's words: {CARD!r}")
check("Other bases are the same ruler stretched." in new_rows["04_5"], "04_5 still follows with other bases")

print("-- hooks")
outputs = {}
for name, tree in (("control", control), ("patched", patched)):
    code, out = run(["python3", "-I", "-B", "docs/check_cues.py"], tree / CRATE)
    outputs[name] = out
    check(code == 0 and out.endswith("OK"), f"check_cues.py on the {name} copy: exit {code}, last line {out.splitlines()[-1]!r}")
check(outputs["control"] == outputs["patched"], "check_cues.py prints the same report for both copies")
mirror = {(t, o): rust_test_mirror(new_rows["08_5"] if t == "new" else old_rows["08_5"], OPENER[o])
          for t in ("old", "new") for o in ("old", "new")}
check(mirror[("old", "old")] and mirror[("new", "new")], "Rust test mirror passes: old text with old opener, new with new")
check(not mirror[("new", "old")] and not mirror[("old", "new")], "Rust test mirror fails crossed: the test edit is needed")
old_lines = (control / PRODUCT).read_text().split("\n")
new_lines = (patched / PRODUCT).read_text().split("\n")
diff_lines = [i + 1 for i, (a, b) in enumerate(zip(old_lines, new_lines)) if a != b]
tests_at = (patched / PRODUCT).read_text().split("\n#[cfg(test)]\nmod tests {")[0].count("\n") + 2
check(len(old_lines) == len(new_lines) and diff_lines == [432, 722, 723], f"product_rule.rs: same length, lines {diff_lines} change")
check(new_lines[431].strip().startswith("//") and new_lines[721].strip().startswith("//") and 723 > tests_at,
      f"the changed lines are a scene comment and the test (tests start at line {tests_at})")
check(f'text.find("{OPENER["new"]}")' in new_lines[722], "the test now opens on the new words")
for const, phrase in PHRASES.items():
    check(f'const {const}: &str = "{phrase}";' in (patched / PRODUCT).read_text(), f"{const} is still {phrase!r}")
config = (REPO / "rustfmt.toml").read_text()
check('edition = "2024"' in config and "max_width = 120" in config, "repository rustfmt.toml: edition 2024, width 120")
code, out = run(["rustfmt", TOOLCHAIN, "--version"], patched)
print("  " + out)
for name, tree in (("control", control), ("patched", patched)):
    code, out = run(["rustfmt", TOOLCHAIN, "--check", "--edition", "2024", "--config", "max_width=120,skip_children=true",
                     PRODUCT], tree)
    check(code == 0 and not out, f"rustfmt --check product_rule.rs, {name} copy: exit {code}" + (f": {out}" if out else ""))

print("-- bindings")
hold = json.loads((PLAN / "hold-audit.json").read_text())
anchors = json.loads((PLAN / "phrase-anchors.json").read_text())
check(hold["script_sha256"] == anchors["script_sha256"] == old_script_sha, "root's bound plan binds the same script")
total = [sum(words(t) for t in old_rows.values()), sum(words(t) for t in new_rows.values())]
check(total[0] == hold["counts"]["words_total"] and all(r["words"] == words(old_rows[r["cue"]]) for r in hold["cues"]),
      f"word counts reproduce the hold audit ({total[0]} words)")
old_product, new_product = sha(control / PRODUCT), sha(patched / PRODUCT)
hold_rows = []
for r in hold["prediction_holds"]:
    if r["file"] != "product_rule.rs":
        continue
    o, n = interval(old_lines, r["say_line"], r["marker_line"]), interval(new_lines, r["say_line"], r["marker_line"])
    check(r["source_sha256"] == old_product and o == r["interval_sha256"],
          f"{r['cue']} hold: the interval formula reproduces root's {o[:12]} (lines {r['say_line']}-{r['marker_line']})")
    check(n == o and new_lines[r["marker_line"] - 1].strip() == r["marker"], f"{r['cue']} hold: interval and marker unchanged")
    hold_rows.append({"cue": r["cue"], "say_line": r["say_line"], "marker_line": r["marker_line"], "interval_sha256": o})
map_rows = []
for r in dmap["visual_holds"]:
    if r["path"] != PRODUCT:
        check(sha(patched / r["path"]) == r["sha256"], f"map {r['cue']} hold: {Path(r['path']).name} unchanged")
        continue
    h = next(x for x in hold_rows if x["cue"] == r["cue"])
    check(r["sha256"] == r["evidence"]["sha256"] == old_product and r["line"] == h["marker_line"]
          and r["interval_sha256"] == h["interval_sha256"], f"map {r['cue']} hold binds product_rule.rs {old_product[:12]}")
    map_rows.append({"cue": r["cue"], "fields": ["sha256", "evidence.sha256"], "old": old_product, "new": new_product,
                     "line": r["line"], "interval_sha256_unchanged": r["interval_sha256"]})
check(json.dumps(dmap).count(old_product) == 2 * len(map_rows) == 4, "the map names product_rule.rs's hash 4 times, all listed")
anchor_rows = []
for a in anchors["anchors"]:
    if a["source_path"] != PRODUCT:
        continue
    const = a["const"]
    o, n = anchor(old_rows[a["cue"]], a["phrase"]), anchor(new_rows[a["cue"]], a["phrase"])
    recorded = {k: a[k] for k in o}
    check(o == recorded and a["cue_text_sha256"] == sha_text(old_rows[a["cue"]]) and a["source_sha256"] == old_product,
          f"anchor {const}: the formula reproduces root's {recorded}")
    sites = [s["line"] for s in a["hook_sites"]]
    check(all(f'until_phrase(&kit, stage, "{a["cue"]}", {const})' in new_lines[s - 1] for s in sites),
          f"anchor {const}: hook still at line(s) {sites}")
    anchor_rows.append({"cue": a["cue"], "const": const, "phrase": a["phrase"], "hook_lines": sites,
                        "old": {**o, "cue_text_sha256": a["cue_text_sha256"], "source_sha256": old_product},
                        "new": {**n, "cue_text_sha256": sha_text(new_rows[a["cue"]]), "source_sha256": new_product}})
check([r["const"] for r in anchor_rows] == list(PHRASES), "the two product_rule.rs anchors are 08_5's, both listed")
check(not [a for a in anchors["anchors"] if a["cue"] == "04_4"], "04_4 has no phrase anchor")
matching = {}
for name in ("control", "patched"):
    current = script_rows((control if name == "control" else patched) / SCRIPT)
    bound = set()
    for p in sorted(TAKES.glob("*-request01.json")):
        cue = p.name.split("-")[0]
        if cue in current and receipt_text_sha(json.loads(p.read_text())) == sha_text(current[cue]):
            bound.add(cue)
    matching[name] = bound
check(matching["control"] - matching["patched"] == set(CHANGES) and matching["patched"] <= matching["control"],
      f"Oliver request receipts matching the text: {len(matching['control'])} before, {len(matching['patched'])} after; only 04_4 and 08_5 drop")
requests = json.loads(REQUESTS.read_text())
superseded = []
for r in requests["requests"]:
    if r["cue"] in CHANGES:
        check(r["text_sha256"] == sha_text(old_rows[r["cue"]]), f"final80-02 request {r['cue']} binds the old text")
        superseded.append({"cue": r["cue"], "list": "narration-request-final80-02/requests.json",
                           "text_path": "/".join(r["text_path"].split("/")[-2:]), "text_sha256": r["text_sha256"]})

print("-- inventory")
inventory = {
    "schema_version": 1,
    "topic": "logarithm_mysteries",
    "claim": "Computed by checks/source_checks.py from the stage08 candidate and root's bound-plan03, read only. "
             "Source text and hashes only: nothing compiled, rendered, synthesized, measured or heard.",
    "voice": "Owner direction 2026-11-05T03:14:34Z: ElevenLabs Frederick Surrey for new production. Every take "
             "listed here is Oliver, historical and test input only; root verifies the provider voice and synthesizes.",
    "patch": {"sha256": sha(PATCH), "against": "combined-repair-stage08/candidate"},
    "script": {"path": SCRIPT, "old": old_script_sha, "new": sha(patched / SCRIPT)},
    "cues": cue_rows,
    "source": {"path": PRODUCT, "old": old_product, "new": new_product, "lines_changed": diff_lines,
               "line_count": len(new_lines), "unchanged": {UNDOING: sha(patched / UNDOING)}},
    "map_rebind": {"path": MAP, "sha256": sha(CAND / MAP),
                   "scripts.logarithm_mysteries": {"old": old_script_sha, "new": sha(patched / SCRIPT)},
                   "visual_holds": map_rows, "naming": "root's call; the file name says oliver"},
    "units_rebind": {"path": UNITS, "sha256": sha(CAND / UNITS),
                     "script_sha256": {"old": old_script_sha, "new": sha(patched / SCRIPT)},
                     "direction_contract.script_sha256": {"old": old_script_sha, "new": sha(patched / SCRIPT)},
                     "direction_contract.map": "the rebound map's path and hash, root's call",
                     "texts": [{"id": f"oliver_{r['cue']}", "old_text_sha256": r["old_text_sha256"],
                                "new_text_sha256": r["new_text_sha256"]} for r in cue_rows]},
    "phrase_anchors": anchor_rows,
    "hold_audit": {"product_rule.rs_holds": hold_rows, "words_total": total,
                   "words": {r["cue"]: r["words"] for r in cue_rows}},
    "requests_superseded": superseded,
    "oliver_takes_of_the_old_text": takes,
    "oliver_receipts_matching_text": {"before": len(matching["control"]), "after": len(matching["patched"])},
}
if DUMP:
    if DUMP.exists():
        sys.exit(f"{DUMP.name} exists")
    DUMP.write_text(json.dumps(inventory, indent=1, ensure_ascii=False) + "\n")
    print(f"  wrote {DUMP.name}; not compared")
    failures.append("inventory dumped, not compared")
else:
    shipped = json.loads((BUNDLE / "binding-inventory.json").read_text())
    check(shipped == inventory, "binding-inventory.json equals what this run computed")

print()
print("RESULT " + ("PASS" if not failures else f"FAIL {len(failures)}"))
sys.exit(1 if failures else 0)
