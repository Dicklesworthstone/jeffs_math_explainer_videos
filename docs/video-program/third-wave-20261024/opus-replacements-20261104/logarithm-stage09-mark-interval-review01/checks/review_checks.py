#!/usr/bin/env python3
"""Independent review checks of Lavender's L1 mark-interval fix (Mail 4079, bead .63).

Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-mark-interval-review01/checks/review_checks.py

Read-only: it writes nothing and prints its transcript. It imports Bronze's sealed stage09 helper
(the base) and Lavender's staged helper side by side, in process, and calls their functions on the
real report entries and on synthetic entries. Its oracle mirrors the native rule in Python, and
every mirrored source line must still be present in its pinned source (exit 3 otherwise):
  - the kit (narration.rs): sentences split after . ? ! before a space or the end; a mark at or past
    the line length is refused; report_starts sums out_samples + gap_after_samples over meta.rate;
  - the films (phrase_seconds in five chapters, sentence_offsets in doing_nothing.rs): a byte belongs
    to the last opening at or before it, openings [0] + (i + 2) after each `.`/`?`/`!` + space;
    sentence j runs from starts[j] to starts[j + 1], or to the line length for the last;
  - the assembler (coherent_narration.py): its sentence regex.
A FINDING line reports a staged behaviour that differs from the native rule or the request. A FAIL
line is a check of this review that did not hold. Fixture answers are labelled "review fixture";
they are not answers for production. Nothing here is listening.
"""

import difflib
import hashlib
import importlib.util
import json
import os
import random
import re
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
FIX = f"{P}/logarithm-stage09-mark-interval-fix01"
FIX_SUMS = "60f5f63ad958a1e41c3d6b668f1166e9599dc76977480616eb08c50ae5b115a7"
BASE_DIR = f"{P}/logarithm-stage09-audio-preparation01"
BASE_SUMS = "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13"
INSPECTION_DIR = f"{P}/logarithm-stage09-timing-inspection01"
INSPECTION_SUMS = "8c18a24363d745e65be1a3ffec89b1d1bfc8d4c2a77185c85b0a2fcd805490bf"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
CRATE = f"{EL}/combined-repair-stage09/candidate/explainer_corpus/logarithm_mysteries"
ANCHORS = f"{EL}/combined-repair-stage09/bound-plan/phrase-anchors.json"
PINS = {
    f"{BASE_DIR}/prepare_audio.py": "c6c854ce62f86ecb4025a9999e883ff04e28a29492788fec05c9060238cdb1e3",
    f"{FIX}/staged/prepare_audio.py": "2f080ab7492638320697882bc6cc596e6fdb1330164df9cf980ba89ec9e2e73c",
    f"{FIX}/prepare_audio.patch": "1265a1be490701f0a12392862aa88afe0173122d50f0bed8067873afeaf38844",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    "tools/coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
    f"{CRATE}/script.tsv": "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a",
    ANCHORS: "557d74a065260303689df29fedefafa54b084a691e912844b3f8e9171b1b7c84",
}
FILMS = [f"{CRATE}/src/chapters/{n}.rs" for n in ("product_rule", "lifting_loops", "unreachable", "complex_turns", "what_it_forgets")]
OFFSETS_FILM = f"{CRATE}/src/chapters/doing_nothing.rs"
MIRRORED = {
    "crates/explainer_kit/src/narration.rs": [
        "if matches!(c, b'.' | b'?' | b'!') && (i + 1 == b.len() || b[i + 1] == b' ') {",
        "let s = text[from..=i].trim();", "let rest = text[from..].trim();",
        "|| time >= lengths[id]", "lengths.insert(id.clone(), f.seconds());", "starts.push(t as f64 / rate);",
        "t += (n(\"out_samples\")? + n(\"gap_after_samples\")?) as usize;"],
    "tools/coherent_narration.py": ['return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]'],
    **{f: ["if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {", "opens.push(i + 2);",
           "let j = opens.iter().rposition(|&open| open <= byte).unwrap_or(0);"] for f in FILMS},
    OFFSETS_FILM: ["if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {", "at.push(i + 2);"],
}
RATE = 24000
FIXTURE = {"by": "review fixture", "basis": "review fixture"}
results = {"pass": 0, "fail": 0, "finding": 0}


def sha_file(p):
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def report(name, ok, detail):
    results["pass" if ok else "fail"] += 1
    print(f"{name} {'PASS' if ok else 'FAIL'}: {detail}")


def finding(name, detail):
    results["finding"] += 1
    print(f"{name} FINDING: {detail}")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ----------------------------------------------------------------------------- the native oracle

def film_opens(text):
    b = text.encode()
    return [0] + [i + 2 for i in range(len(b) - 1) if b[i] in b".?!" and b[i + 1] == 0x20]


def kit_count(text):
    b, n, start = text.encode(), 0, 0
    for i, c in enumerate(b):
        if c in b".?!" and (i + 1 == len(b) or b[i + 1] == 0x20):
            n += bool(b[start:i + 1].decode().strip())
            start = i + 1
    return n + bool(b[start:].decode().strip())


def assembler_count(text):
    return len([s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()])


def native_sentence(text, byte):
    return max(j for j, o in enumerate(film_opens(text)) if o <= byte)


def native_inside(text, onsets_s, length_s, byte, seconds):
    j = native_sentence(text, byte)
    hi = onsets_s[j + 1] if j + 1 < len(onsets_s) else length_s
    return onsets_s[j] <= seconds < hi


def word_starts(text):
    return [len(text[:m.start()].encode()) for m in re.finditer(r"\S+", text) if m.group(0) not in ".?!"]


def synthetic(cue, text, onset_samples, marks):
    """A report-shaped entry: sentence onsets in 24 kHz samples, marks as (byte, recognizer seconds)."""
    file = "review-fixture.wav"
    return {"cue": cue, "file": file, "label": file,
            "sentences": [{"index": 0, "status": "file_start", "sample_24k": 0}] +
                         [{"id": f"onset:{cue}:s{k}:{file}", "index": k, "status": "supported", "asr_start_s": n / RATE, "sample_24k": n}
                          for k, n in enumerate(onset_samples[1:], 1)],
            "phrase_anchors": [{"id": f"anchor:{cue}:b{b}:{file}", "byte": b, "phrase": text.encode()[b:].decode().split(" ")[0],
                                "status": "supported", "asr_start_s": s, "sample_24k": round(s * RATE)} for b, s in marks]}


def main():
    # R01 pins and seals
    missing = [f"{f}: {ln}" for f, lines in MIRRORED.items() for ln in lines if ln not in open(f, encoding="utf8").read()]
    stale = [p for p, h in PINS.items() if sha_file(p) != h]
    for d, h in ((FIX, FIX_SUMS), (BASE_DIR, BASE_SUMS), (INSPECTION_DIR, INSPECTION_SUMS)):
        if sha_file(f"{d}/SHA256SUMS") != h:
            stale.append(f"{d}/SHA256SUMS")
            continue
        listed = dict(reversed(line.split("  ", 1)) for line in open(f"{d}/SHA256SUMS", encoding="utf8").read().splitlines())
        found = {os.path.relpath(os.path.join(r, f), d) for r, _x, fs in os.walk(d) for f in fs} - {"SHA256SUMS"}
        stale += [f"{d}/{rel}" for rel, want in listed.items() if not os.path.isfile(f"{d}/{rel}") or sha_file(f"{d}/{rel}") != want]
        stale += [f"{d}/{rel} (unlisted)" for rel in sorted(found - set(listed))]
    if missing or stale:
        print("STALE:\n" + "\n".join(missing + stale))
        return 3
    report("R01", True, f"3 seals verify, no unlisted file (fix {FIX_SUMS[:8]}, stage09 {BASE_SUMS[:8]}, inspection {INSPECTION_SUMS[:8]}); "
                        f"{len(PINS)} pins hold; {sum(len(v) for v in MIRRORED.values())} mirrored source lines found in {len(MIRRORED)} files")

    base = load("review_base_prepare_audio", f"{BASE_DIR}/prepare_audio.py")
    staged = load("review_staged_prepare_audio", f"{FIX}/staged/prepare_audio.py")

    # R02 the patch is exactly the diff base -> staged, and only these hunks
    a = open(f"{BASE_DIR}/prepare_audio.py", encoding="utf8").read().splitlines(keepends=True)
    b = open(f"{FIX}/staged/prepare_audio.py", encoding="utf8").read().splitlines(keepends=True)
    diff = "".join(difflib.unified_diff(a, b, "a/prepare_audio.py", "b/prepare_audio.py", n=3))
    patch = open(f"{FIX}/prepare_audio.patch", encoding="utf8").read()
    plus = sum(1 for ln in diff.splitlines() if ln.startswith("+") and not ln.startswith("+++"))
    minus = sum(1 for ln in diff.splitlines() if ln.startswith("-") and not ln.startswith("---"))
    defs = []
    for h in re.finditer(r"^@@ -\d+(?:,\d+)? \+(\d+)", diff, re.M):
        line = int(h.group(1)) + 3
        defs.append(next(re.match(r"def (\w+)", b[k]).group(1) for k in range(line, -1, -1) if b[k].startswith("def ")))
    report("R02", diff == patch and (plus, minus) == (41, 4),
           f"difflib(base, staged) equals prepare_audio.patch byte for byte: {diff == patch}; +{plus}/-{minus}; hunks in {defs}")

    script = base.read_script(f"{CRATE}/script.tsv")

    # R03 splitters on the 80 real cues
    bad = []
    for cue, text in script.items():
        f, s = film_opens(text), staged.sentence_opens(text)
        h = [0] + [len(text[:x].encode()) for x, _y in base.kit_sentences(text)[1:]]
        if not (f == s == h) or len({len(f), kit_count(text), assembler_count(text)}) != 1:
            bad.append(cue)
    n_sent = sum(len(film_opens(t)) for t in script.values())
    non_ascii = sum(1 for t in script.values() if any(ord(c) > 127 for c in t))
    report("R03", not bad and len(script) == 80,
           f"{len(script)} cues, {n_sent} sentences: film openings = staged sentence_opens = base kit_sentences in bytes, and kit, film and "
           f"assembler counts agree; disagreeing {bad}; cues with a non-ASCII character {non_ascii}")

    # R04 the 33 bound anchors
    rows = json.load(open(ANCHORS))["anchors"]
    bad, first = [], []
    for an in rows:
        text = script[an["cue"]]
        tb = text.encode()
        ok = 0 <= an["byte"] < len(tb) and tb[an["byte"]:].startswith(an["phrase"].encode())
        js = max(j for j, o in enumerate(staged.sentence_opens(text)) if o <= an["byte"])
        if not ok or js != native_sentence(text, an["byte"]):
            bad.append(f"{an['cue']}:b{an['byte']}")
        if an["byte"] in film_opens(text) or an["byte"] in staged.sentence_opens(text):
            first.append(f"{an['cue']}:b{an['byte']}")
    report("R04", not bad and len(rows) == 33,
           f"{len(rows)} anchors at their phrase, staged sentence = native sentence; bad {bad}; anchors on a sentence's first word {first}")

    # R05 every real recording: recognizer points, and the inspection's leave_24k answers
    entries = json.load(open(f"{BASE_DIR}/report/anchors.json"))["recordings"]
    insp = {r["id"]: r for r in json.load(open(f"{INSPECTION_DIR}/outputs/timing-inspection.json"))["rows"]}
    lines = []
    all_ok = len(entries) == 162
    for view in ("recognizer", "leave_24k answers", "leave_24k answers, 08_5 dl02 s2 = 167040"):
        refused, differ, onsets, marks, lead, room = [], [], 0, 0, (9e9, ""), (9e9, "")
        n_answers = 0
        for e in entries:
            frames24 = (e["frames"] + 1) // 2
            text = script[e["cue"]]
            applied = {}
            if view != "recognizer":
                for p in e["sentences"][1:] + e["phrase_anchors"]:
                    if insp.get(p.get("id"), {}).get("class") == "recognizer_early":
                        samp = insp[p["id"]]["leave_24k"]
                        if view.endswith("167040") and p["id"].startswith("onset:08_5:s2:") and p["id"].endswith("oliver-clarity-takes01/08_5-download02.wav"):
                            samp = 167040
                        applied[p["id"]] = {"decision": "sample", **FIXTURE, "sample_24k": samp}
            n_answers += len(applied)
            try:
                got = staged.resolved_points(e, applied, frames24, text)
            except staged.Refused as x:
                refused.append(f"{e['label']}: {x}")
                continue
            if got != base.resolved_points(e, applied, frames24):
                differ.append(e["label"])
            _starts, ends, ms = got
            onsets += len(ends) - 1
            marks += len(ms)
            bounds = [x / RATE for x in [0] + ends[:-1]] + [frames24 / RATE]
            for m in ms:
                j = native_sentence(text, m["byte"])
                if not bounds[j] <= m["seconds"] < bounds[j + 1]:
                    refused.append(f"{e['label']} b{m['byte']}: accepted by staged, outside by the oracle")
                tag = f"{e['cue']} b{m['byte']} {os.path.basename(e['label'])}"
                lead = min(lead, (round(m["seconds"] - bounds[j], 6), tag))
                room = min(room, (round(bounds[j + 1] - m["seconds"], 6), tag))
        all_ok &= not refused and not differ
        lines.append(f"[{view}: {n_answers} fixture answers; {onsets} onsets, {marks} marks; refused {len(refused)} {refused[:1]}; "
                     f"staged differs from base {len(differ)}; smallest lead {lead[0]} s ({lead[1]}); smallest room {room[0]} s ({room[1]})]")
    report("R05", all_ok, f"{len(entries)} recordings in 3 views, each mark also checked by the oracle: " + " ".join(lines))

    # R06 seeded random differential against the native rule
    vocab = ["one", "two", "log", "base", "e", "turn", "half", "minus", "pi", "zero", "3.14", "e.g.",
             "π", "e−1", "naïve", "it’s", "½", "Möbius", "2π", "—x", "😀"]
    joiners = [" "] * 14 + ["  ", " \t", "  "]

    def cases(seed, n):
        rnd = random.Random(seed)
        for _ in range(n):
            parts = []
            for k in range(rnd.randint(1, 5)):
                words = " ".join(rnd.choice(vocab) for _w in range(rnd.randint(1, 7)))
                end = rnd.choice(".?!") if k < 4 and rnd.random() < 0.9 else ""
                parts.append(words + end)
            text = parts[0]
            for p in parts[1:]:
                text += rnd.choice(joiners) + p
            k = len(film_opens(text))
            onsets = [0]
            for _j in range(k - 1):
                onsets.append(onsets[-1] + rnd.randint(2400, 72000))
            length = onsets[-1] + rnd.randint(2400, 72000)
            byte = rnd.choice(word_starts(text))
            j = native_sentence(text, byte)
            lo, hi = onsets[j], onsets[j + 1] if j + 1 < k else length
            sample = rnd.choice([lo, lo - 1, lo + 1, hi, hi - 1, length, length - 1, rnd.randint(lo, hi - 1), rnd.randint(0, length)])
            yield text, [o / RATE for o in onsets], length / RATE, byte, sample / RATE

    def disagreements(mod):
        n = d = power = 0
        for text, onsets_s, length_s, byte, sec in cases(4079, 20000):
            n += 1
            if len(mod.sentence_opens(text)) != len(onsets_s):
                d += 1
                continue
            got = mod.mark_outside_sentence(text, onsets_s, length_s, byte, sec) is None
            d += got != native_inside(text, onsets_s, length_s, byte, sec)
            ch = [0] + [x for x, _y in base.kit_sentences(text)[1:]]
            power += max(j for j, o in enumerate(ch) if o <= byte) != native_sentence(text, byte)
        return n, d, power

    n, d, power = disagreements(staged)
    report("R06", d == 0, f"{n} seeded random texts (multi-byte words, ? and !, e.g. and 3.14, double space, tab and no-break space after a split, "
                          f"unterminated last sentence) with a mark at, one sample inside and outside each edge, or random: staged disagrees with "
                          f"the native rule in {d}; texts where comparing the byte with character openings picks another sentence {power}")

    # R07 mutants of the staged rule; the R06 differential must catch each
    mutants = {}
    m = load("review_mutant_char", f"{FIX}/staged/prepare_audio.py")
    m.sentence_opens = lambda text, _k=m.kit_sentences: [0] + [x for x, _y in _k(text)[1:]]
    mutants["character openings"] = m
    for label, cmp in (("upper edge included", lambda lo, s, hi: lo <= s <= hi), ("lower edge excluded", lambda lo, s, hi: lo < s < hi),
                       ("last sentence unbounded", None)):
        mm = load(f"review_mutant_{len(mutants)}", f"{FIX}/staged/prepare_audio.py")

        def outside(text, onsets_s, length_s, byte, seconds, _mm=mm, _cmp=cmp):
            k = max(j for j, at in enumerate(_mm.sentence_opens(text)) if at <= byte)
            hi = onsets_s[k + 1] if k + 1 < len(onsets_s) else (float("inf") if _cmp is None else length_s)
            return None if (_cmp or (lambda a_, s_, b_: a_ <= s_ < b_))(onsets_s[k], seconds, hi) else "outside"
        mm.mark_outside_sentence = outside
        mutants[label] = mm
    caught = {label: disagreements(mm)[1] for label, mm in mutants.items()}
    report("R07", all(v > 0 for v in caught.values()), f"mutants caught by the R06 differential (disagreements of 20000): {caught}")

    # R08 a real entry: 08_5 download01 (clarity), both marks in sentence 4 of 0..5
    e = next(x for x in entries if x["label"].endswith("oliver-clarity-takes01/08_5-download01.wav"))
    text, frames24 = script["08_5"], (e["frames"] + 1) // 2
    s = {p["id"].split(":")[2]: p for p in e["sentences"][1:]}
    mk = {p["byte"]: p for p in e["phrase_anchors"]}
    m267, m306, s4, s5 = mk[267], mk[306], s["s4"], s["s5"]
    assert native_sentence(text, 267) == native_sentence(text, 306) == 4 and len(film_opens(text)) == 6

    def run(applied, entry=e, t=text, f24=frames24):
        try:
            staged.resolved_points(entry, applied, f24, t)
            return "accepted"
        except staged.Refused as x:
            return "refused " + "+".join(ln.split(":")[2] for ln in str(x).splitlines() if ln.startswith(("anchor:", "onset:"))) \
                if any(ln.startswith("anchor:") for ln in str(x).splitlines()) else "refused (" + str(x)[:60] + ")"

    def sample(pid, n):
        return {pid: {"decision": "sample", **FIXTURE, "sample_24k": n}}
    mark267 = round(m267["asr_start_s"] * RATE)
    cand = json.loads(json.dumps(e))
    c267 = next(p for p in cand["phrase_anchors"] if p["byte"] == 267)
    c267.update(status="no_agreeing_word", candidate={"heard": "review fixture", "asr_start_s": s5["asr_start_s"] + 0.5, "sample_24k": 0})
    short = json.loads(json.dumps(e))
    short["sentences"] = short["sentences"][:-1]
    plan8 = [
        ("s4 moved 0.1 s after b267", sample(s4["id"], mark267 + 2400), None, "refused b267"),
        ("same, b267 omitted", {**sample(s4["id"], mark267 + 2400), m267["id"]: {"decision": "omit_mark", **FIXTURE}}, None, "accepted"),
        ("s4 moved after both marks (26.0 s)", sample(s4["id"], 26 * RATE), None, "refused b267+b306"),
        ("b306 at s5's onset", sample(m306["id"], s5["sample_24k"]), None, "refused b306"),
        ("b306 one sample before s5", sample(m306["id"], s5["sample_24k"] - 1), None, "accepted"),
        ("b267 at s4's onset", sample(m267["id"], s4["sample_24k"]), None, "accepted"),
        ("b267 one sample before s4", sample(m267["id"], s4["sample_24k"] - 1), None, "refused b267"),
        ("b267 use_candidate inside sentence 5", {c267["id"]: {"decision": "use_candidate", **FIXTURE}}, cand, "refused b267"),
        ("entry one sentence short of the text", {}, short, "refused b267+b306"),
    ]
    got = {label: run(applied, entry or e) for label, applied, entry, _w in plan8}
    ok = all(got[label].startswith(w) if w.endswith("(") else got[label] == w for label, _a, _e, w in plan8)
    report("R08", ok, "; ".join(f"{label}: {got[label]}" for label, *_r in plan8))

    # R09 synthetic edges: a multi-byte text, the last sentence and sentence 0
    t9 = "ππππ x. Ab cd. Last words here."
    x9, last9 = t9.encode().index(b"x"), t9.encode().index(b"words")
    f9 = 30 * RATE
    multibyte = synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [(x9, 1.0)])
    char_rule = max(j for j, o in enumerate([0] + [a for a, _b in base.kit_sentences(t9)[1:]]) if o <= x9)
    edges = {
        "multi-byte: mark at 'x' (byte 9, char 5) at 1.0 s in sentence 0": (run({}, multibyte, t9, f9), "accepted"),
        "last sentence: mark at the last sample": (run(sample(f"anchor:zz_9:b{last9}:review-fixture.wav", f9 - 1),
                                                      synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [(last9, 12.0)]), t9, f9), "accepted"),
        "last sentence: mark at the length": (run(sample(f"anchor:zz_9:b{last9}:review-fixture.wav", f9),
                                                 synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [(last9, 12.0)]), t9, f9), "refused b" + str(last9)),
        "sentence 0: mark at 0.0 s": (run({}, synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [(x9, 0.0)]), t9, f9), "accepted"),
        "sentence 0: mark at sentence 1's onset": (run({}, synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [(x9, 2.0)]), t9, f9), "refused b" + str(x9)),
    }
    report("R09", all(g == w for g, w in edges.values()) and native_sentence(t9, x9) == 0 and char_rule == 1,
           "; ".join(f"{k}: {g}" for k, (g, _w) in edges.items()) + f"; the character rule would put byte {x9} in sentence {char_rule}")

    # R10 a mark on a sentence's first word, at its onset's own recognizer time
    first9 = t9.encode().index(b"Ab")
    out10 = {}
    for asr in (14.72, 14.720001, 14.719999):
        ent = synthetic("zz_9", t9, [0, 2 * RATE, 10 * RATE], [])
        ent["sentences"][1].update(asr_start_s=asr, sample_24k=round(asr * RATE))
        ent["sentences"][2].update(asr_start_s=20.0, sample_24k=20 * RATE)
        ent["phrase_anchors"] = [{"id": f"anchor:zz_9:b{first9}:review-fixture.wav", "byte": first9, "phrase": "Ab", "status": "supported",
                                  "asr_start_s": asr, "sample_24k": round(asr * RATE)}]
        out10[asr] = run({}, ent, t9, f9)
    rounded_up = sum(1 for x in entries for p in x["sentences"][1:] if p["status"] == "supported" and p["sample_24k"] / RATE > p["asr_start_s"])
    marks_up = sum(1 for x in entries for p in x["phrase_anchors"] if p["status"] == "supported" and p["sample_24k"] / RATE > p["asr_start_s"])
    total_on = sum(1 for x in entries for p in x["sentences"][1:] if p["status"] == "supported")
    report("R10", out10[14.72] == "accepted" and out10[14.720001] == "accepted",
           f"a first-word mark sharing its onset's recognizer word: asr 14.72 {out10[14.72]}, 14.720001 {out10[14.720001]}, "
           f"14.719999 {out10[14.719999]}")
    if out10[14.719999] != "accepted":
        finding("R10", f"at 14.719999 s the onset is rounded to sample {round(14.719999 * RATE)} = {round(14.719999 * RATE) / RATE} s while the "
                       f"mark keeps its unrounded recognizer seconds, so a mark on the very word that opens its sentence is refused by 1 µs. "
                       f"The kit itself has no such check. Real supported onsets whose sample lies after their recognizer time: {rounded_up} of "
                       f"{total_on}; real marks the same way: {marks_up}. No real anchor is on a first word (R04), so stage09 is unaffected; the "
                       f"refusal names the mark and a sample answer at the onset resolves it")

    # R11 the timings path (static, on the staged source) and the integration constant
    src = open(f"{FIX}/staged/prepare_audio.py", encoding="utf8").read()
    tim = src[src.index("def cmd_timings"):]
    tim = tim[:tim.index("\ndef ", 1)]
    needed = ['report = load_json(f"{narration}/report.json")', 'onsets.append(t / report["meta"]["rate"])',
              't += row["out_samples"] + row["gap_after_samples"]', "script = read_script(b[\"script\"])",
              'why = mark_outside_sentence(script[cue], onsets, length, m["byte"], m["seconds"])', "length = frames / RATE_OUT"]
    zero_gaps = '"sentence_ends": ends, "gap_after_samples": [0] * len(ends)' in src
    here = re.search(r'^HERE = "([^"]+)"', src, re.M).group(1)
    report("R11", all(x in tim for x in needed) and zero_gaps and here == BASE_DIR,
           "cmd_timings sums out_samples + gap_after_samples over meta.rate as report_starts does, bounds the last sentence by the WAV "
           f"length as the kit's lengths do, and checks every plan mark; prepare writes all-zero gaps, so both see the same onsets: {zero_gaps}; "
           f"staged HERE is still the base folder ({here == BASE_DIR}), so root must set it in the new helper (Lavender's integration item 2)")

    print(f"RESULT {'PASS' if results['fail'] == 0 else 'FAIL'}: {results['pass']} pass, {results['fail']} fail, {results['finding']} finding")
    return 0 if results["fail"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
