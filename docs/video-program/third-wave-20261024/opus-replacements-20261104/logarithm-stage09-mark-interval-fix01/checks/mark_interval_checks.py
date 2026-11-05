"""Regressions for the L1 mark-interval fix (root's Mail 4018, bead 87do.63). From the repository root:

    python3 -I -B .../logarithm-stage09-mark-interval-fix01/checks/mark_interval_checks.py FRESH_SCRATCH

LavenderPike. Read-only on the repository; writes only under FRESH_SCRATCH. Bronze's sealed helper (base) and the
staged helper of this bundle are imported side by side, and their functions are called in process on the real
stage09 binding. Selections, answers and decode rows written here are TEST FIXTURES. The decoded WAVs are
zero-filled or synthetic noise, never a decode of a recording. M09 assembles synthetic noise with a byte copy of the
pinned assembler. No ffmpeg, recognition, production decode or production assembly runs here; no take is chosen,
and nothing here is a listening check.
"""
import ast
import contextlib
import difflib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import random
import shutil
import subprocess
import sys
import types
import wave

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
UNDER = f"{P}/logarithm-stage09-audio-preparation01"
UNDER_SEAL = "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13"
BASE_SHA = "c6c854ce62f86ecb4025a9999e883ff04e28a29492788fec05c9060238cdb1e3"
REVIEW = f"{P}/logarithm-stage09-audio-independent01"
REVIEW_SEAL = "3e7217bf13f793fd0b0738925f3c36488b3f18fd5da05c2c3e9b03ddac01bea9"
HERE = f"{P}/logarithm-stage09-mark-interval-fix01"
STAGED = f"{HERE}/staged/prepare_audio.py"
PATCH = f"{HERE}/prepare_audio.patch"
BINDING = f"{UNDER}/bindings/logarithm-stage09-oliver.json"
TIMING = f"{P}/logarithm-stage09-timing-inspection01"
TIMING_SEAL = "8c18a24363d745e65be1a3ffec89b1d1bfc8d4c2a77185c85b0a2fcd805490bf"
KIT = "crates/explainer_kit/src/narration.rs"
KIT_SHA = "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8"
CHANGED = ("04_4", "08_5")
FIXTURE = {"by": "LavenderPike TEST FIXTURE (not root)", "basis": "TEST FIXTURE ONLY: nothing measured or heard"}
# The films' rule for placing a phrase in its sentence (product_rule.rs phrase_seconds, the 08_5 consumer).
FILM_RULE = ("matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' '", "opens.push(i + 2);",
             "opens.iter().rposition(|&open| open <= byte)", "starts.get(j + 1).copied().unwrap_or(len)")
KIT_RULE = ("starts.push(t as f64 / rate);", "|| time >= lengths[id]")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha(Path(path).read_bytes())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_wav(path, pcm):
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(24000)
        w.writeframes(pcm)


def verify_sums(folder):
    sums = (REPO / folder / "SHA256SUMS").read_bytes()
    rows = [line.split("  ", 1) for line in sums.decode().splitlines()]
    return sha(sums), len(rows), [n for h, n in rows if sha_file(REPO / folder / n) != h]


def film_opens(text):
    """product_rule.rs phrase_seconds, in bytes: sentence 0 opens at 0, the next two bytes after each . ? ! + space."""
    data = text.encode()
    return [0] + [i + 2 for i in range(len(data) - 1) if data[i] in b".?!" and data[i + 1] == ord(" ")]


def sentence_of(opens, byte):
    return max(j for j, at in enumerate(opens) if at <= byte)


def top_level(text):
    tree = ast.parse(text)
    defs = {n.name: ast.get_source_segment(text, n) for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    rest = [ast.dump(n) for n in tree.body if not isinstance(n, (ast.FunctionDef, ast.ClassDef))]
    return defs, rest, [n.name for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))]


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]):
        print("usage: from the repository root, mark_interval_checks.py FRESH_SCRATCH (must not exist)", file=sys.stderr)
        return 2
    scratch = Path(sys.argv[1])
    scratch.mkdir(parents=True)
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        for path in sorted({str(scratch.resolve()), str(scratch)}, key=len, reverse=True):
            detail = detail.replace(path, "SCRATCH")
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    # ------------------------------------------------------------------ M01 the sealed inputs are untouched
    u_sum, u_n, u_bad = verify_sums(UNDER)
    r_sum, r_n, r_bad = verify_sums(REVIEW)
    check("M01-sealed-inputs", u_sum == UNDER_SEAL and u_n == 14 and not u_bad and sha_file(REPO / UNDER / "prepare_audio.py") == BASE_SHA
          and r_sum == REVIEW_SEAL and r_n == 3 and not r_bad,
          f"Bronze's bundle SHA256SUMS {u_sum[:12]} ({u_n} files verify, differing {u_bad}), helper {BASE_SHA[:12]}; "
          f"my independent review SHA256SUMS {r_sum[:12]} ({r_n} files verify, differing {r_bad})")

    # ------------------------------------------------------------------ M02 the patch, and that it is narrow
    base_text = (REPO / UNDER / "prepare_audio.py").read_text(encoding="utf-8")
    staged_text = (REPO / STAGED).read_text(encoding="utf-8")
    remade = "".join(difflib.unified_diff(base_text.splitlines(keepends=True), staged_text.splitlines(keepends=True),
                                          "a/prepare_audio.py", "b/prepare_audio.py"))
    apply_dir = scratch / "apply"
    apply_dir.mkdir()
    shutil.copyfile(REPO / UNDER / "prepare_audio.py", apply_dir / "prepare_audio.py")
    r = subprocess.run(["patch", "-p1", "-d", str(apply_dir), "-i", str(REPO / PATCH)], capture_output=True, text=True)
    applied_ok = r.returncode == 0 and sha_file(apply_dir / "prepare_audio.py") == sha_file(REPO / STAGED)
    bdefs, brest, border = top_level(base_text)
    sdefs, srest, sorder = top_level(staged_text)
    added, removed = sorted(set(sdefs) - set(bdefs)), sorted(set(bdefs) - set(sdefs))
    changed = sorted(n for n in bdefs if n in sdefs and bdefs[n] != sdefs[n])
    per_fn = {}
    for n in changed:
        lines = [x for x in difflib.unified_diff(bdefs[n].splitlines(), sdefs[n].splitlines(), lineterm="", n=0)
                 if x[:1] in "+-" and not x.startswith(("+++", "---"))]
        per_fn[n] = f"+{sum(x[0] == '+' for x in lines)}/-{sum(x[0] == '-' for x in lines)}"
    plus = sum(1 for x in remade.splitlines() if x.startswith("+") and not x.startswith("+++"))
    minus = sum(1 for x in remade.splitlines() if x.startswith("-") and not x.startswith("---"))
    mutants = [m for m in load("bronze_mutation_check", REPO / UNDER / "checks/mutation_check.py").MUTANTS if m[2] is not None]
    targets_once = all(base_text.count(old) == staged_text.count(old) == 1 for _t, _w, old, _n in mutants)
    check("M02-narrow-patch", remade == (REPO / PATCH).read_text(encoding="utf-8") and applied_ok
          and added == ["mark_outside_sentence", "sentence_opens"] and not removed
          and changed == ["cmd_prepare", "cmd_timings", "resolved_points"] and per_fn["cmd_prepare"] == "+1/-1"
          and brest == srest and [n for n in sorder if n not in added] == border and len(mutants) == 8 and targets_once,
          f"prepare_audio.patch is the unified diff base -> staged (+{plus}/-{minus} lines) and `patch -p1` turns a copy "
          f"of the base into the staged file ({sha_file(REPO / STAGED)[:12]}); new functions {added}; changed {per_fn}; "
          f"every other top-level function, class and statement is unchanged and in its place; each of the {len(mutants)} "
          f"target texts of Bronze's mutation_check.py occurs exactly once in the staged file, as in the base")

    base = load("prepare_audio_base", REPO / UNDER / "prepare_audio.py")
    staged = load("prepare_audio_staged", REPO / STAGED)
    b = staged.load_binding(BINDING)
    script = staged.read_script(b["script"])

    # A byte copy of the pinned assembler, its cache and the candidate's map (also the M09 workspace).
    ws = scratch / "prepared-template" / "workspace"
    for src, dst in ((b["assembler"], "tools/coherent_narration.py"), (b["assembler_cache"], "tools/narration_cache.py"),
                     (b["map"], b["map_in_tree"])):
        (ws / dst).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / src, ws / dst)
    asm = load("assembler_copy", ws / "tools/coherent_narration.py")

    # ------------------------------------------------------------------ M03 which sentence holds a byte
    product_rule = (REPO / b["tree"] / "explainer_corpus" / b["topic"] / "src/chapters/product_rule.rs").read_text(encoding="utf-8")
    kit = (REPO / KIT).read_text(encoding="utf-8")
    rule_ok = all(staged.sentence_opens(t) == film_opens(t) and len(film_opens(t)) == len(asm.sentences(t)) for t in script.values())
    plan = json.loads((REPO / b["phrase_anchors"]).read_text())["anchors"]
    placed = [(a["cue"], a["byte"], sentence_of(staged.sentence_opens(script[a["cue"]]), a["byte"]),
               sentence_of(film_opens(script[a["cue"]]), a["byte"])) for a in plan]
    mb = "ππππππππ. A b. C d."
    mb_byte = mb.encode().index(b"b.")
    char_opens = [0] + [a for a, _z in staged.kit_sentences(mb)[1:]]
    check("M03-byte-to-sentence", rule_ok and all(k == f for _c, _b, k, f in placed) and len(placed) == 33
          and [(c, byt, k) for c, byt, k, _f in placed if c == "08_5"] == [("08_5", 267, 4), ("08_5", 306, 4)]
          and all(x in product_rule for x in FILM_RULE) and sha(kit.encode()) == KIT_SHA and all(x in kit for x in KIT_RULE)
          and staged.sentence_opens(mb) == film_opens(mb) == [0, 18, 23] and char_opens == [0, 10, 15]
          and sentence_of(staged.sentence_opens(mb), mb_byte) == 1 and sentence_of(char_opens, mb_byte) == 2,
          f"on all {len(script)} stage09 cues, sentence_opens equals the films' rule (product_rule.rs phrase_seconds) and the "
          f"pinned assembler's sentence count; all {len(placed)} anchors land in the same sentence both ways (08_5 bytes "
          f"267 and 306 in sentence 4); the kit ({KIT_SHA[:8]}) reads onsets as samples / rate and refuses a mark at or past "
          f"the length; no stage09 cue has a non-ASCII byte, so a synthetic {mb!r} shows the byte rule ({mb_byte} -> "
          f"sentence 1) where a character rule would say sentence 2")

    # ------------------------------------------------------------------ M04 every real recognized take
    files_b, q_b, rec_b, inp_b = base.build_report(b)
    files_s, questions, recordings, inputs = staged.build_report(b)
    report_same = files_s == files_b and all(files_s[n].encode() == (REPO / UNDER / "report" / n).read_bytes() for n in files_s)
    accepted, equal, refused, margins = 0, 0, [], []
    onsets = marks_n = 0
    for e in recordings:
        text = script[e["cue"]]
        f24 = (e["frames"] + 1) // 2
        onsets += len(e["sentences"]) - 1
        marks_n += len(e["phrase_anchors"])
        try:
            got = staged.resolved_points(e, {}, f24, text)
        except staged.Refused as error:
            refused.append(f"{e['label']}: {error}")
            continue
        accepted += 1
        equal += got == base.resolved_points(e, {}, f24)
        bounds = [x / 24000 for x in [0] + got[1][:-1]]
        for m in got[2]:
            k = sentence_of(staged.sentence_opens(text), m["byte"])
            hi = bounds[k + 1] if k + 1 < len(bounds) else f24 / 24000
            margins.append((round(m["seconds"] - bounds[k], 3), round(hi - m["seconds"], 3), e["cue"], m["byte"], k))
    lead = min(margins)
    room = min(margins, key=lambda x: x[1])
    check("M04-all-real-takes", report_same and len(recordings) == 162 and {e["rate"] for e in recordings} == {48000}
          and accepted == 162 and equal == 162 and not refused and len(margins) == marks_n == 66 and onsets == 570,
          f"the staged report equals the base report and Bronze's sealed report/ byte for byte; all {len(recordings)} "
          f"inventoried recordings resolve from their recognizer onsets ({onsets} sentence onsets, {marks_n} marks) with "
          f"no refusal and exactly the base's starts, ends and marks; smallest lead of a mark over its sentence's onset "
          f"{lead[0]} s ({lead[2]} byte {lead[3]}, sentence {lead[4]}), smallest room before the next onset or the end "
          f"{room[1]} s ({room[2]} byte {room[3]}, sentence {room[4]}); refused {refused}")

    # ------------------------------------------------------------------ M05/M06 shifted onsets and the answers that resolve them
    clar_dir = b["inventories"][1]["takes_dir"]
    first = {}
    for e in recordings:
        first.setdefault(e["cue"], e)
    takes = {c: (f"{clar_dir}/{c}-download01.wav" if c in CHANGED else first[c]["file"]) for c in script}
    sel_doc = {"schema": "audio-preparation-selection-v1", "binding_sha256": b["sha256"], "script_sha256": b["pins"][b["script"]],
               "by": FIXTURE["by"], "basis": "TEST FIXTURE ONLY: first recording of each cue; nobody chose or heard anything"}
    (scratch / "selection-full.json").write_text(json.dumps(dict(sel_doc, takes=takes)), encoding="utf-8")
    _sel, chosen = staged.load_selection(b, str(scratch / "selection-full.json"), inputs, None)
    e85 = next(e for e in recordings if e["path"] == takes["08_5"])
    s1, s4, s5 = e85["sentences"][1], e85["sentences"][4], e85["sentences"][5]
    m267, m306 = e85["phrase_anchors"]
    f85 = (e85["frames"] + 1) // 2
    text85 = script["08_5"]

    def answers_file(tag, answers):
        path = scratch / f"answers-{tag}.json"
        path.write_text(json.dumps({"schema": "audio-preparation-answers-v1", "binding_sha256": b["sha256"],
                                    "answers": {k: dict(FIXTURE, **v) for k, v in answers.items()}}), encoding="utf-8")
        return str(path)

    def outcome(module, tag, answers):
        applied = staged.check_answers(b, answers_file(tag, answers), questions, recordings, chosen)
        try:
            got = (module.resolved_points(e85, applied, f85, text85) if module is staged
                   else module.resolved_points(e85, applied, f85))
            return "accepted " + str([m["byte"] for m in got[2]])
        except module.Refused as error:
            return "refused: " + str(error)

    moved4 = round(m267["asr_start_s"] * 24000) + 2400  # sentence 4's onset 0.1 s after the 'leaves a half turn' mark
    moved5 = 600000  # sentence 5's onset 25.0 s, before the 'a factor of minus one' mark
    a4 = {s4["id"]: {"decision": "sample", "sample_24k": moved4}}
    a5 = {s5["id"]: {"decision": "sample", "sample_24k": moved5}}
    base4, staged4 = outcome(base, "s4-base", a4), outcome(staged, "s4", a4)
    base5, staged5 = outcome(base, "s5-base", a5), outcome(staged, "s5", a5)
    check("M05-shifted-onset-refused", base4 == "accepted [267, 306]" and base5 == "accepted [267, 306]"
          and staged4.startswith("refused: ") and staged4.count("\n") == 0 and m267["id"] in staged4 and "sentence 4" in staged4
          and m306["id"] not in staged4
          and staged5.startswith("refused: ") and staged5.count("\n") == 0 and m306["id"] in staged5
          and "the onset of sentence 5 at 25.0 s" in staged5 and m267["id"] not in staged5,
          f"08_5-download01 (onsets {[s['sample_24k'] for s in e85['sentences']]}, marks {m267['asr_start_s']} / "
          f"{m306['asr_start_s']} s): sentence 4 moved to sample {moved4} (0.1 s after the first mark) -> base {base4}, "
          f"staged {staged4!r}; sentence 5 moved to sample {moved5} (before the second mark) -> base {base5}, staged {staged5!r}")

    cases = [
        ("s4 moved, mark omitted", "a", {**a4, m267["id"]: {"decision": "omit_mark"}}, "accepted [306]"),
        ("s4 moved, mark at the new onset", "b", {**a4, m267["id"]: {"decision": "sample", "sample_24k": moved4}}, "accepted [267, 306]"),
        ("s4 moved, mark one sample before it", "c", {**a4, m267["id"]: {"decision": "sample", "sample_24k": moved4 - 1}}, m267["id"]),
        ("s5 moved, mark omitted", "d", {**a5, m306["id"]: {"decision": "omit_mark"}}, "accepted [267]"),
        ("s5 moved, mark one sample before it", "e", {**a5, m306["id"]: {"decision": "sample", "sample_24k": moved5 - 1}}, "accepted [267, 306]"),
        ("s5 moved, mark at it", "f", {**a5, m306["id"]: {"decision": "sample", "sample_24k": moved5}}, m306["id"]),
        ("Bronze's D answer: s1 + 12000", "g", {s1["id"]: {"decision": "sample", "sample_24k": s1["sample_24k"] + 12000}}, "accepted [267, 306]"),
        ("s1 at the end of its 'Here' segment", "h", {s1["id"]: {"decision": "sample", "sample_24k": round(s1["asr_end_s"] * 24000)}}, "accepted [267, 306]"),
    ]
    rows, ok6 = [], True
    for label, tag, answers, want in cases:
        got = outcome(staged, f"case-{tag}", answers)
        ok = got == want if want.startswith("accepted") else got.startswith("refused: ") and want in got
        ok6 = ok6 and ok
        rows.append(f"{label} -> " + (f"refused naming {':'.join(want.split(':')[:3])}" if got.startswith("refused") and ok else got))
    check("M06-answers-resolve", ok6, "; ".join(rows))

    # ------------------------------------------------------------------ M07 edges, on synthetic entries
    def entry(text, onsets_24k, anchors):
        return {"cue": "TEST", "file": "test.wav",
                "sentences": [{"index": 0, "status": "file_start", "sample_24k": 0}]
                + [{"id": f"onset:TEST:s{k}:test.wav", "index": k, "status": "supported", "sample_24k": s} for k, s in enumerate(onsets_24k, 1)],
                "phrase_anchors": [{"id": f"anchor:TEST:b{byt}:test.wav", "byte": byt, "phrase": ph, "status": "supported", "asr_start_s": sec}
                                   for byt, ph, sec in anchors]}

    def edge(text, onsets_24k, anchors, f24, samples, want):
        """'accepted', or 'refused' when the refusal names the mark and contains `want`."""
        e = entry(text, onsets_24k, anchors)
        applied = {f"anchor:TEST:b{byt}:test.wav": dict(FIXTURE, decision="sample", sample_24k=s) for byt, s in samples.items()}
        try:
            staged.resolved_points(e, applied, f24, text)
            return "accepted"
        except staged.Refused as error:
            named = all(f"anchor:TEST:b{byt}:test.wav" in str(error) for byt in samples)
            return "refused" if named and want in str(error) and str(error).count("\n") == 0 else f"refused otherwise: {error}"

    two = "One two. Three four five."
    last = [(4, "two", 0.0), (15, "four", 1.5)]
    kit_accepts = lambda seconds, length: 0 <= seconds < length  # narration.rs load: refuses time >= length
    edges = [
        ("last sentence, mark at the last sample", edge(two, [24000], last, 48000, {15: 47999}, ""), "accepted", kit_accepts(47999 / 24000, 2.0)),
        ("last sentence, mark at the length", edge(two, [24000], last, 48000, {15: 48000}, "sentence 1, which runs from 1.0 s up to the "
                                                   "recording's end at 2.0 s"), "refused", kit_accepts(48000 / 24000, 2.0)),
        ("mark at its own sentence's onset", edge(two, [24000], last, 48000, {15: 24000}, ""), "accepted", None),
        ("mark one sample before its sentence", edge(two, [24000], last, 48000, {15: 23999}, "(0.9999583333333333 s) is not inside its "
                                                     "sentence 1"), "refused", None),
        ("sentence 0, mark at 0.0 s", edge(two, [24000], last, 48000, {}, ""), "accepted", None),
        ("sentence 0, mark at the next onset", edge(two, [24000], last, 48000, {4: 24000}, "sentence 0, which runs from 0.0 s up to the "
                                                    "onset of sentence 1 at 1.0 s"), "refused", None),
        ("multi-byte text, byte sentence 1", edge(mb, [24000, 36000], [(mb_byte, "b", 1.25)], 48000, {}, ""), "accepted", None),
    ]
    ok7 = all(got == want for _l, got, want, _k in edges) and edges[0][3] is True and edges[1][3] is False
    check("M07-edges", ok7, "; ".join(f"{label} -> {got}" + ("" if k is None else f" (kit loader: {'accepts' if k else 'refuses'})")
                                      for label, got, _w, k in edges)
          + f"; the multi-byte mark at 1.25 s sits in byte sentence 1 [1.0, 1.5) s, while a character rule would put it in "
          f"sentence 2 [1.5, 2.0) s")

    # ------------------------------------------------------------------ M08 prepare refuses before it writes anything
    class Reached(Exception):
        pass

    def stop_at_out(b_, out):
        raise Reached(out)

    ch08 = [c for c in script if c.startswith("08_")]
    (scratch / "selection-08.json").write_text(json.dumps(dict(sel_doc, takes={c: takes[c] for c in ch08})), encoding="utf-8")
    _sel08, chosen08 = staged.load_selection(b, str(scratch / "selection-08.json"), inputs, "08")
    dec = scratch / "prepare-decoded"
    dec.mkdir()
    drows = []
    for cue, t in chosen08.items():
        n = (t["frames"] + 1) // 2
        write_wav(dec / f"{cue}.wav", b"\0\0" * n)
        drows.append({"cue": cue, "text": script[cue], "source": t["rel"], "source_sha256": t["sha256"], "source_pcm_sha256": t["pcm"],
                      "source_frames": t["frames"], "source_rate": t["rate"], "downloads_receipt": t.get("receipt"),
                      "wav": str(dec / f"{cue}.wav"), "wav_sha256": sha_file(dec / f"{cue}.wav"), "samples": n, "rate": 24000})
    (dec / "decode-receipt.json").write_text(json.dumps({
        "source": b["voice"]["receipt_source"], "voice": b["voice"]["name"], "provider": b["voice"]["provider"], "production": True,
        "binding": staged.binding_summary(b), "tool": "TEST FIXTURE", "selection_sha256": sha_file(scratch / "selection-08.json"),
        "chapter": "08", "records": drows, "claim": "TEST FIXTURE: zero-filled stand-ins of the decoded length, not a decode"}), encoding="utf-8")

    def prepare(module, tag, answers):
        args = types.SimpleNamespace(selection=str(scratch / "selection-08.json"), answers=answers_file(f"prepare-{tag}", answers),
                                     decoded=str(dec), reviewer="LavenderPike TEST", out=str(scratch / "never-written"),
                                     chapter="08", test_only=False)
        original, module.make_out = module.make_out, stop_at_out  # the step after every refusal; nothing is written
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                module.cmd_prepare(b, args)
            return "finished"
        except Reached:
            return "reached the output step"
        except module.Refused as error:
            return "refused: " + str(error)
        finally:
            module.make_out = original

    p_base = prepare(base, "base-s4", a4)
    p_staged = prepare(staged, "s4", a4)
    p_omit = prepare(staged, "s4-omit", {**a4, m267["id"]: {"decision": "omit_mark"}})
    p_plain = prepare(staged, "plain", {})
    check("M08-prepare-wiring", p_base == "reached the output step" and p_staged.startswith("refused: ") and p_staged.count("\n") == 0
          and m267["id"] in p_staged and p_omit == p_plain == "reached the output step" and not (scratch / "never-written").exists(),
          f"cmd_prepare in process, chapter 08 fixture selection (08_5 by its clarity path), zero-filled stand-ins of the decoded "
          f"length: with sentence 4 moved -> base {p_base}, staged {p_staged!r}; with that mark omitted -> {p_omit}; "
          f"recognizer onsets only -> {p_plain}")

    # ------------------------------------------------------------------ M09 timings checks the onsets the kit will read
    rng = random.Random(20261105)
    tree = ast.parse(staged_text)
    decode_fn = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "cmd_decode")
    keysets = [{k.value for k in d.keys if isinstance(k, ast.Constant)} for d in ast.walk(decode_fn) if isinstance(d, ast.Dict)]
    record_keys = next(k for k in keysets if "wav_sha256" in k)
    receipt_keys = next(k for k in keysets if "records" in k)
    world = scratch / "timings-world"
    (world / "decoded").mkdir(parents=True)
    stand_in = world / "source-stand-in.bin"
    stand_in.write_bytes(b"synthetic source stand-in, not audio")
    unit_ids = {u["line_ids"][0]: u["id"] for u in json.loads((REPO / b["units"]).read_text())["units"]}
    recs, units, bounds = [], [], {}
    for cue in ch08:
        k = len(asm.sentences(script[cue]))
        length = 24000 + 7919 * k + rng.randrange(1, 997)
        wav = world / "decoded" / f"{cue}.wav"
        write_wav(wav, rng.randbytes(2 * length))
        ends = [length * (j + 1) // k for j in range(k)]
        bounds[cue] = ([0] + ends[:-1], length)
        row = {key: None for key in record_keys}
        row.update(cue=cue, text=script[cue], source=str(stand_in), source_sha256=sha_file(stand_in), wav=str(wav),
                   wav_sha256=sha_file(wav), samples=length, rate=24000)
        recs.append(row)
        units.append({"id": unit_ids[cue], "wav": str(wav), "sha256": sha_file(wav), "generation_receipt": str(world / "decode-receipt.json"),
                      "sentence_ends": ends, "gap_after_samples": [0] * k})
    receipt = {key: None for key in receipt_keys}
    receipt.update(source=b["voice"]["receipt_source"], voice=b["voice"]["name"], provider=b["voice"]["provider"], records=recs,
                   claim="TEST FIXTURE: synthetic noise, LavenderPike mark-interval check")
    (world / "decode-receipt.json").write_text(json.dumps(receipt, indent=1), encoding="utf-8")
    disp = [d for d in json.loads((REPO / b["dispositions"]).read_text())["direction_dispositions"] if d["cue"] in ch08]
    template = scratch / "prepared-template"
    (template / "spans.json").write_text(json.dumps({
        "schema_version": 1, "units_sha256": b["pins"][b["units"]], "units": units, "direction_dispositions": disp,
        "boundary_review": {"reviewer": "LavenderPike TEST", "basis": "TEST FIXTURE: synthetic noise; whole takes, nothing "
                            "removed or inserted", "listening": "unreviewed"}}, indent=2) + "\n", encoding="utf-8")
    (template / "prepare-receipt.json").write_text(json.dumps({
        "tool": "TEST FIXTURE", "binding": staged.binding_summary(b), "production": True, "chapter": "08",
        "spans_sha256": sha_file(template / "spans.json"), "cues": {c: {"recording": "synthetic"} for c in ch08}}), encoding="utf-8")
    narration = world / "narration"
    r = subprocess.run([sys.executable, "-I", "-B", str(ws / "tools/coherent_narration.py"), "assemble", "--script", b["script"],
                        "--units", b["units"], "--require-directions", "--chapter", "08", "--spans", str(template / "spans.json"),
                        "--out", str(narration)], cwd=REPO, capture_output=True, text=True)
    anchors08 = sorted((a["cue"], a["byte"]) for a in plan if a["cue"] in ch08)

    def in_sentence_plan():
        out = {}
        for cue in ch08:
            starts, length = bounds[cue]
            opens = staged.sentence_opens(script[cue])
            mine = [byt for c, byt in anchors08 if c == cue]
            rows_ = []
            for byt in mine:
                k = sentence_of(opens, byt)
                same = [x for x in mine if sentence_of(opens, x) == k]
                lo, hi = starts[k], starts[k + 1] if k + 1 < len(starts) else length
                rows_.append({"byte": byt, "seconds": (lo + (hi - lo) * (same.index(byt) + 1) // (len(same) + 1)) / 24000,
                              "basis": "TEST FIXTURE"})
            out[cue] = rows_
        return out

    good = in_sentence_plan()
    early = json.loads(json.dumps(good))
    s4_85, s5_85 = bounds["08_5"][0][4] / 24000, bounds["08_5"][0][5] / 24000
    early["08_5"][0]["seconds"] = round(s4_85 - 0.1, 6)  # in order and in range; before its sentence's onset
    at_next = json.loads(json.dumps(good))
    at_next["08_5"][1]["seconds"] = s5_85  # exactly the next sentence's onset

    def timings(module, tag, marks):
        prepared = scratch / f"prepared-{tag}"
        shutil.copytree(template, prepared)
        (prepared / "phrase-marks-plan.json").write_text(json.dumps({"claim": "TEST FIXTURE", "cues": marks}), encoding="utf-8")
        out = scratch / f"timings-{tag}"
        # Outputs go to scratch: only make_out's repository-path guard is replaced, after every check has run.
        original, module.make_out = module.make_out, lambda b_, o: (os.makedirs(o), o)[1]
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                module.cmd_timings(b, types.SimpleNamespace(prepared=str(prepared), narration=str(narration), out=str(out), test_only=False))
            written = json.loads((out / "phrase-timings-fragment-08.json").read_text())
            return "written " + str(sum(len(e["marks"]) for e in written["cues"].values())) + " marks"
        except module.Refused as error:
            return "refused: " + str(error)
        finally:
            module.make_out = original

    t_good = timings(staged, "staged-good", good)
    t_early_base, t_early = timings(base, "base-early", early), timings(staged, "staged-early", early)
    t_next_base, t_next = timings(base, "base-next", at_next), timings(staged, "staged-next", at_next)
    n08 = len(anchors08)
    check("M09-timings-wiring", r.returncode == 0 and t_good == f"written {n08} marks"
          and t_early_base == t_next_base == f"written {n08} marks"
          and t_early.startswith("refused: 08_5: the mark at byte 267") and "sentence 4" in t_early and t_early.count("\n") == 0
          and t_next.startswith("refused: 08_5: the mark at byte 306") and "the onset of sentence 5" in t_next and t_next.count("\n") == 0,
          f"chapter 08 ({len(ch08)} cues, {n08} anchors) of synthetic noise assembled by the pinned assembler copy "
          f"(--require-directions); cmd_timings in process with marks inside their sentences -> {t_good}; the 08_5 'leaves a "
          f"half turn' mark 0.1 s before sentence 4's onset (still in order and range) -> base {t_early_base}, staged "
          f"{t_early!r}; the 'a factor of minus one' mark exactly at sentence 5's onset -> base {t_next_base}, staged {t_next!r}"
          + ("" if r.returncode == 0 else f"; assemble: {(r.stderr or r.stdout).strip()[-300:]}"))

    # ------------------------------------------------------------------ M10 Bronze's measured onsets as answers (Mail 4061)
    t_sum, t_n, t_bad = verify_sums(TIMING)
    with open(REPO / TIMING / "outputs/timing-rows.tsv", encoding="utf-8") as f:
        header, *lines = [x.split("\t") for x in f.read().splitlines()]
    trows = [dict(zip(header, x)) for x in lines]
    early_rows = [x for x in trows if x["class"] == "recognizer_early"]
    point_ids = {p["id"]: e for e in recordings for p in e["sentences"][1:] + e["phrase_anchors"]}
    variants = {"leave_24k for all early points": {x["id"]: int(x["leave_24k"]) for x in early_rows}}
    variants["the same, with Bronze's exception for 08_5 download02 s2"] = dict(variants["leave_24k for all early points"])
    exception = next(x["id"] for x in early_rows if x["cue"] == "08_5" and x["file"] == "08_5-download02.wav" and x["id"].split(":")[2] == "s2")
    variants["the same, with Bronze's exception for 08_5 download02 s2"][exception] = 167040
    rows10, ok10 = [], (t_sum == TIMING_SEAL and not t_bad and len(trows) == 636 and len(early_rows) == 181
                        and all(x["id"] in point_ids for x in early_rows))
    for name, moves in variants.items():
        refused, closest = [], []
        for e in recordings:
            applied = {pid: dict(FIXTURE, decision="sample", sample_24k=s) for pid, s in moves.items() if point_ids.get(pid) is e}
            text, f24 = script[e["cue"]], (e["frames"] + 1) // 2
            try:
                _st, ends, marks = staged.resolved_points(e, applied, f24, text)
            except staged.Refused as error:
                refused.append(str(error))
                continue
            bounds = [x / 24000 for x in [0] + ends[:-1]]
            for m in marks:
                k = sentence_of(staged.sentence_opens(text), m["byte"])
                hi = bounds[k + 1] if k + 1 < len(bounds) else f24 / 24000
                closest.append((round(min(m["seconds"] - bounds[k], hi - m["seconds"]), 3), e["cue"], m["byte"]))
        ok10 = ok10 and not refused and len(closest) == 66
        rows10.append(f"{name}: {sum(1 for e in recordings for p in moves if point_ids.get(p) is e)} answers, "
                      f"{162 - len(refused)}/162 recordings accepted, closest mark to a sentence edge {min(closest)[0]} s "
                      f"({min(closest)[1]} byte {min(closest)[2]}); refused {refused}")
    check("M10-measured-onset-answers", ok10,
          f"Bronze's sealed timing inspection (SHA256SUMS {t_sum[:12]}, {t_n} files verify, differing {t_bad}) lists "
          f"{len(early_rows)} recognizer_early points ({sum(x['kind'] == 'sentence' for x in early_rows)} onsets, "
          f"{sum(x['kind'] != 'sentence' for x in early_rows)} mark), each a point id of the report; given as root's sample answers "
          f"to the staged resolved_points: " + "; ".join(rows10))

    print(f"MARK-INTERVAL: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
