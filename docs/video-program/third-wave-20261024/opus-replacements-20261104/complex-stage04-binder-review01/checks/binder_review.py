"""Focused, read-only checks behind REVIEW.md (complex-stage04-binder-review01). Run from the repo root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage04-binder-review01/checks/binder_review.py

It prints its report to stdout and writes nothing. It imports neither binder: every rule here is ported from the Rust
source it names (the kit's narration.rs, the crate's draw.rs, tools/coherent_narration.py) and re-derived from the
stage04 files. It does not repeat BronzeJaguar's 64 checks. Times it prints are source estimates, labelled as such;
the only measured numbers are WAV file lengths of four existing Oliver takes, read from the sealed audio inventory.
"""
import hashlib
import json
import pathlib
import re
import sys

REPO = pathlib.Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BX = f"{P}/complex-stage03-binder01"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
CR = f"{EC}/combined-repair-stage04/explainer_corpus/complex_analysis_visual"
BOUND = f"{BX}/bound-plan-stage04/explainer_corpus/complex_analysis_visual"
PINS = {
    f"{EC}/combined-repair-stage04-integration01.json": "295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b",
    f"{CR}/script.tsv": "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3",
    f"{CR}/narration_units.json": "3c4f53d7973857295e4f99de681e8537ead08676d1ebdfe9625e69b4bc86702a",
    f"{BX}/SHA256SUMS": "c66742a07d4323ff4664887ee4092e99e6f0c653abd850458373ec3dddbbc32e",
    f"{BOUND}/docs/narration-direction-map-oliver12.json": "70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c",
    f"{BOUND}/narration_units.json": "e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    f"{P}/complex-stage04-audio-reuse01/SHA256SUMS": "6336b5659f76e9fb2f6822d0337d440fb32eb474c52f0fe9dd6e659b33222676",
}
FAILS = []


def say(*a):
    print(" ".join(str(x) for x in a))


def check(ok, label, detail=""):
    say("PASS" if ok else "FAIL", label + (f": {detail}" if detail else ""))
    if not ok:
        FAILS.append(label)


def raw(rel):
    return (REPO / rel).read_bytes()


def sha(b):
    return hashlib.sha256(b).hexdigest()


# ------------------------------------------------------------------ ports of the three sentence splitters
def kit_sentences(text):
    """explainer_kit narration.rs `sentences`: split after . ? ! followed by a space or the end; trim; drop empties."""
    out, frm, b = [], 0, text
    for i, c in enumerate(b):
        if c in ".?!" and (i + 1 == len(b) or b[i + 1] == " "):
            s = b[frm : i + 1].strip()
            if s:
                out.append(s)
            frm = i + 1
    rest = b[frm:].strip()
    if rest:
        out.append(rest)
    return out


def crate_offsets(text):
    """draw.rs `sentence_offsets`: char offsets where Spoken's sentences start."""
    out = [0]
    for i in range(max(0, len(text) - 2)):
        if text[i] in ".?!" and text[i + 1] == " ":
            out.append(i + 2)
    return out


def report_sentences(text):
    """tools/coherent_narration.py `sentences`: the rows a narration report carries."""
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


def film_code(src):
    """The source before the one `#[cfg(test)]` + `mod tests {` block (the split the crate's tests use)."""
    m = list(re.finditer(r"#\[cfg\(test\)\]\s*mod tests \{", src))
    return src if not m else src[: m[0].start()]


def main():
    # ------------------------------------------------------------- pins
    for rel, want in PINS.items():
        got = sha(raw(rel))
        check(got == want, f"pin {rel.rsplit('/', 2)[-2]}/{rel.rsplit('/', 1)[1]}", got[:8])
    bad = [line for line in raw(f"{BX}/SHA256SUMS").decode().splitlines()
           if sha(raw(f"{BX}/" + line.split("  ", 1)[1])) != line.split("  ", 1)[0]]
    check(not bad, "BronzeJaguar's sealed bundle verifies against its SHA256SUMS", f"{len(bad)} mismatches")

    script = raw(f"{CR}/script.tsv").decode("utf-8")
    cues = dict(re.findall(r"^(\d\d_\d+)\t(.*)$", script, re.M))
    order = list(cues)
    check(len(cues) == 105, "stage04 script has 105 cues")

    # ------------------------------------------------------------- 1. the three sentence splitters agree on every cue
    disagree = []
    for c, t in cues.items():
        k, off, rep = kit_sentences(t), crate_offsets(t), report_sentences(t)
        pieces = [t[a:b].strip() for a, b in zip(off, off[1:] + [len(t)])]
        if not (len(k) == len(off) == len(rep) and k == pieces == rep):
            disagree.append(c)
    check(not disagree, "kit sentences(), draw.rs sentence_offsets() and coherent_narration sentences() split all 105 cues alike",
          f"disagree at {disagree or 'none'}")
    say("NOTE draw.rs Spoken uses measured sentence starts only when their count equals sentence_offsets' count, "
        "and falls back to a character share silently; agreement above means no cue falls back once report.json exists")

    # ------------------------------------------------------------- 2. literal hooks at their own call sites
    mod_rs = raw(f"{CR}/src/chapters/mod.rs").decode("utf-8")
    compiled = sorted(set(re.findall(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*;", mod_rs, re.M)))
    said, sites, helper_sites = {}, [], []
    for name in compiled:
        rel = f"{CR}/src/chapters/{name}.rs"
        src = film_code(raw(rel).decode("utf-8"))
        lines_at = [0] + [m.end() for m in re.finditer("\n", src)]

        def line_of(pos):
            lo = 0
            for i, start in enumerate(lines_at):
                if start <= pos:
                    lo = i
            return lo + 1

        binds = [(m.start(), m.group(1), m.group(2)) for m in re.finditer(r'let\s+(\w+)\s*=\s*say\(&kit,\s*stage,\s*"(\d\d_\d+)"\)', src)]
        fns = [m.start() for m in re.finditer(r"\bfn\s+\w+", src)]
        for m in re.finditer(r'say\(&kit,\s*stage,\s*"(\d\d_\d+)"\)', src):
            said.setdefault(m.group(1), []).append(f"{name}.rs:{line_of(m.start())}")
        pat = re.compile(r'\b(\w+)\.(wait_for|until|at)\(\s*(?:stage,\s*)?"((?:[^"\\]|\\.)*)"'
                         r'|\b(\w+)\.(wait_sentence|sentence)\(\s*(?:stage,\s*)?(\d+)\s*\)')
        for m in pat.finditer(src):
            recv, call, value = (m.group(1), m.group(2), m.group(3)) if m.group(1) else (m.group(4), m.group(5), int(m.group(6)))
            # a binding counts only inside the call's own function: a helper's `line` is its parameter
            in_fn = max((f for f in fns if f < m.start()), default=0)
            prior = [(pos, c) for pos, r, c in binds if r == recv and in_fn < pos < m.start()]
            entry = {"file": f"{name}.rs", "line": line_of(m.start()), "call": call, "value": value, "recv": recv}
            if prior:
                entry["cue"] = prior[-1][1]
                sites.append(entry)
            else:
                helper_sites.append(entry)
    dup = {c: v for c, v in said.items() if len(v) != 1}
    check(set(said) == set(cues) and not dup, "each of the 105 cues is said exactly once in the 17 compiled modules",
          f"{len(compiled)} modules, {len(said)} cues said, repeats {dup or 'none'}")

    problems = []
    for e in sites:
        t = cues[e["cue"]]
        if e["call"] in ("wait_for", "until", "at"):
            n = t.count(e["value"])
            at = t.find(e["value"])
            prev = t[at - 1] if at > 0 else " "
            e["char"] = at
            e["sentence"] = max(j for j, o in enumerate(crate_offsets(t)) if o <= at) if at >= 0 else None
            if n != 1 or prev.isalnum() or prev in "'’":
                problems.append((e["cue"], e["file"], e["line"], e["value"], f"occurs {n}x, char before {prev!r}"))
        else:
            e["sentence"] = e["value"]
            if e["value"] >= len(crate_offsets(t)):
                problems.append((e["cue"], e["file"], e["line"], e["value"], f"only {len(crate_offsets(t))} sentences"))
    check(not problems, f"{len(sites)} literal hooks at their call sites: each phrase once in its cue at a word start "
          "(apostrophe counts as inside a word), each sentence index in range", f"{problems or 'none'}")
    say(f"NOTE {len(helper_sites)} literal hook calls sit in helpers or on receivers bound elsewhere; "
        "the binder expands those at their call sites, this scan does not")

    # order within each cue, in source order of the call sites (loops and branches can legitimately reorder: listed)
    by_cue = {}
    for e in sites:
        by_cue.setdefault(e["cue"], []).append(e)
    backwards = []
    for c, es in by_cue.items():
        pos = [(e["char"] if "char" in e else crate_offsets(cues[c])[e["value"]]) for e in sorted(es, key=lambda x: (x["file"], x["line"]))]
        if any(b < a for a, b in zip(pos, pos[1:])):
            backwards.append(c)
    say("NOTE cues whose literal call sites, read top to bottom, step backwards in the text:", backwards or "none")

    # ------------------------------------------------------------- 3. against BronzeJaguar's strict audit
    audit = json.loads(raw(f"{BX}/bound-plan-stage04/strict-audit.json"))
    theirs = {(c, h["file"], h["line"], h["call"], json.dumps(h["value"])) for c, hs in audit["hooks"].items() for h in hs}
    mine = {(e["cue"], e["file"], e["line"], e["call"], json.dumps(e["value"])) for e in sites}
    check(mine <= theirs, "every literal call-site hook found here is in the strict audit's table at the same cue, file, line, call and value",
          f"missing from theirs: {sorted(mine - theirs) or 'none'}")
    # helper sites: which cue(s) the audit expands each to, and whether the literal is in that cue's text
    for e in helper_sites:
        got = sorted(x[0] for x in theirs if x[1:] == (e["file"], e["line"], e["call"], json.dumps(e["value"])))
        fits = all((e["value"] in cues[c]) if isinstance(e["value"], str) else e["value"] < len(crate_offsets(cues[c])) for c in got)
        check(bool(got) and fits, f"helper hook {e['file']}:{e['line']} {e['call']} {e['value']!r} is expanded by the audit and fits its cue(s)",
              f"{got or 'not expanded'}")
    lit_theirs = {x for x in theirs if x[1] in {f"{n}.rs" for n in compiled}}
    say(f"NOTE strict audit rows: {len(theirs)}; matched by this literal scan: {len(mine & theirs)}; "
        "the rest are helper expansions, loop unrolls and dynamic values (S2/S3), which this scan does not re-derive")
    off = [(c, h["file"], h["line"], h["value"], h["char"], h.get("sentence"))
           for c, hs in audit["hooks"].items() for h in hs
           if isinstance(h["value"], str) and h["call"] in ("wait_for", "until", "at")
           and (cues[c].find(h["value"]) != h["char"] or max(j for j, o in enumerate(crate_offsets(cues[c])) if o <= h["char"]) != h["sentence"])]
    check(not off, "the strict audit's char and sentence of every phrase hook equal first-occurrence find() on the stage04 text",
          f"{off or 'none'}")
    del lit_theirs

    # ------------------------------------------------------------- 4. map rows against the stage04 files
    m = json.loads(raw(f"{BOUND}/docs/narration-direction-map-oliver12.json"))
    rows = [("hold", r) for r in m["visual_holds"]] + [("helper", r) for r in m["helpers"]]
    wrong = []
    for kind, r in rows:
        b = raw(f"{EC}/combined-repair-stage04/{r['path']}")
        text_line = b.decode("utf-8").splitlines()[r["line"] - 1].strip()
        if sha(b) != r["sha256"] or text_line != r["marker"].strip():
            wrong.append((kind, r.get("cue", r["path"].rsplit("/", 1)[1]), r["line"]))
    check(not wrong, f"all {len(rows)} map rows: file hash equals stage04's and the marker is the text of its line", f"{wrong or 'none'}")
    pred = dict(re.findall(r'\("(\d\d_\d+)",\s*([\d.]+)\)', mod_rs[mod_rs.index("PREDICTS"):mod_rs.index("READING_WINDOWS")]))
    read = dict(re.findall(r'\("(\d\d_\d+)",\s*([\d.]+)\)', mod_rs[mod_rs.index("READING_WINDOWS"):mod_rs.index("pub const TEXT")]))
    mk = {(r["cue"], r["kind"]) for r in m["visual_holds"]}
    check(mk == {(c, "prediction") for c in pred} | {(c, "reading") for c in read},
          "map holds are exactly mod.rs PREDICTS (prediction) and READING_WINDOWS (reading)", f"{len(pred)} + {len(read)}")
    nxt = [r["cue"] for r in m["visual_holds"] if order[order.index(r["cue"]) + 1] != r["next_cue"]]
    check(not nxt, "each hold's next_cue is the script's next cue", f"{nxt or 'none'}")
    ov = [o["cue"] for o in m["overrides"] if o["after"] != cues[o["cue"]]]
    check(not ov and {o["cue"] for o in m["overrides"]} == {c for c, _ in mk},
          "the 23 overrides quote their cue's stage04 text byte for byte, one per hold", f"{ov or 'none'}")

    # ------------------------------------------------------------- 5. units: bound plan against script and the installed plan
    u = json.loads(raw(f"{BOUND}/narration_units.json"))
    ids = [x["id"] for x in u["units"]]
    ok = (ids == [f"oliver_{c}" for c in order] and all(x["line_ids"] == [x["id"][7:]] and x["text"] == cues[x["id"][7:]] for x in u["units"])
          and u["script_sha256"] == PINS[f"{CR}/script.tsv"] == u["direction_contract"]["script_sha256"]
          and u["direction_contract"]["map"]["sha256"] == sha(raw(f"{BOUND}/docs/narration-direction-map-oliver12.json")))
    check(ok, "bound units: 105 one-cue units in script order, each text the stage04 cue, script and map hashes right")
    inst = json.loads(raw(f"{CR}/narration_units.json"))
    idiff = sorted(x["id"][7:] for x, y in zip(inst["units"], u["units"]) if x != y)
    say(f"NOTE installed stage04 units ({sha(raw(f'{CR}/narration_units.json'))[:8]}) differ from the bound ones in units {idiff}, "
        f"and in script_sha256 {inst['script_sha256'][:8]} -> {u['script_sha256'][:8]}, map {inst['direction_contract']['map']['path'].rsplit('-', 1)[1]} -> "
        f"{u['direction_contract']['map']['path'].rsplit('-', 1)[1]}")

    # ------------------------------------------------------------- 5b. stage04 against stage03, from the two receipts and the files
    r3 = json.loads(raw(f"{EC}/combined-repair-stage03-integration01.json"))
    r4 = json.loads(raw(f"{EC}/combined-repair-stage04-integration01.json"))
    h3 = {f["path"]: f["sha256"] for f in r3["files"]}
    h4 = {f["path"]: f["sha256"] for f in r4["files"]}
    differ = sorted(p for p in set(h3) | set(h4) if h3.get(p) != h4.get(p))
    check(differ == sorted(r4["changed"]), "stage04 differs from stage03 in exactly the four files its receipt names", f"{[p.rsplit('/', 1)[1] for p in differ]}")
    for name in ("counting.rs", "draw.rs"):
        a = film_code(raw(f"{EC}/combined-repair-stage03/explainer_corpus/complex_analysis_visual/src/chapters/{name}").decode("utf-8"))
        b = film_code(raw(f"{CR}/src/chapters/{name}").decode("utf-8"))
        check(a == b, f"{name}: film code (before `#[cfg(test)] mod tests {{`) byte-equal in stage03 and stage04", f"{a.count(chr(10))} lines")

    # ------------------------------------------------------------- 5c. which hook moments a take measures, and which stay estimates
    kinds = {}
    for c, hs in audit["hooks"].items():
        for h in hs:
            kinds[h["call"]] = kinds.get(h["call"], 0) + 1
    sent = sum(v for k, v in kinds.items() if k in ("wait_sentence", "sentence", "wait_last"))
    phr = sum(v for k, v in kinds.items() if k in ("wait_for", "until", "at"))
    say(f"strict audit hooks by call: {sorted(kinds.items())}")
    say(f"  {sent} sentence hooks take measured sentence onsets once the take's report.json exists (narration.rs:884-905); "
        f"{phr} phrase hooks stay character-share estimates inside their sentence (draw.rs:142-157) unless a phrase-timings.json "
        "bound to the WAV and text hashes exists (narration.rs:906-951); line.end() takes the WAV length")

    # ------------------------------------------------------------- 6. 15_2: the recap strip, as source estimates
    t = cues["15_2"]
    w = len(t.split())
    off = crate_offsets(t)
    k = kit_sentences(t)
    say(f"15_2: {w} words, {len(t)} chars, {len(k)} sentences; sentence 5 is {len(k[5])} chars: {k[5]!r}")
    old = t.replace("though there may be none", "though not always everywhere")
    say(f"15_2 before F3: sentence 5 {len(kit_sentences(old)[5])} chars; sentences 0-4 byte-equal: {kit_sentences(old)[:5] == k[:5]}")
    inv = json.loads(raw(f"{P}/complex-stage04-audio-reuse01/inventory/recordings.json"))
    spw = sorted({(r["stage04_cue"], round(r["seconds"] / r["submitted_words"], 4)) for r in inv if r["status"] == "current"})
    say("measured: WAV seconds per word of the four existing current Oliver takes (file length, not aligned):", spw)
    rates = {"scratch kit estimate (2.6 words/s + 0.4 s)": None}
    rates.update({f"Oliver file rate {s:.3f} s/word": s for s in (min(x for _, x in spw), max(x for _, x in spw))})
    phrase = "and it turns a real integral"
    for label, s in rates.items():
        if s is None:
            speech = w / 2.6
            length = speech + 0.4
            parts = kit_sentences(t)
            total = sum(len(p) for p in parts)
            starts, before = [], 0
            for p in parts:
                starts.append(speech * before / total)
                before += len(p)
        else:
            length = w * s
            starts = [length * o / len(t) for o in off]
        i = t.find(phrase)
        j = max(n for n, o in enumerate(off) if o <= i)
        to, t1 = (off[j + 1], starts[j + 1]) if j + 1 < len(off) else (len(t), length)
        at_phrase = starts[j] + (t1 - starts[j]) * (i - off[j]) / max(1, to - off[j])
        panel = [starts[2], starts[3], starts[4], at_phrase, starts[5]]
        lit = [p + 0.9 for p in panel]  # picture 0.5 s then caption 0.4 s, played one after the other (closing.rs:342-343)
        overrun = [n for n in range(4) if lit[n] > panel[n + 1]]
        gone = length + 1.2  # the strip starts to fade as 15_3 is said (closing.rs:353-357)
        say(f"  {label}: line {length:.2f} s; panels lit at " + ", ".join(f"{p:.2f}" for p in panel)
            + f"; caption complete at {lit[-1]:.2f}; overrun into the next panel: {overrun or 'none'}; "
            f"each panel complete for " + ", ".join(f"{gone - x:.2f}" for x in lit) + " s before the strip starts to fade; "
            f"last panel complete for {gone - lit[-1]:.2f} s, of which {length - lit[-1]:.2f} s under the voice")
    say("15_2 after the voice: wait_until(line.end()) then pause(1.2) (closing.rs:347-349). main.rs:39 sets LineGaps::Constant, so "
        "kit.hold (closing.rs:352) waits only to busy_until + BREATH (0.6 s) and say(15_3) to busy_until + BREATH: both already "
        "past. The strip is held 1.2 s after the voice (rounded up to a frame), not 1.2 + 0.6 s, then fades over 0.5 s under 15_3.")
    say("the same holds under LineGaps::Thought: hold waits at most BREATH, and say waits class_gap(15_2, 15_3) = "
        f"Beat 0.81 s less the takes' quiet edges (15_3 opens {cues['15_3'].split()[0]!r}, not a topic opener or connective), "
        "under 1.2 s")
    caps = [["on an open region:", "stretch and turn"], ["the boundary", "decides the inside"], ["output turns count", "zeros − poles"],
            ["real integrals", "from residues"], ["at most one extension;", "paths can branch"]]
    src = raw(f"{CR}/src/chapters/closing.rs").decode("utf-8")
    check(all(f'["{a}", "{b}"]' in src for a, b in caps), "closing.rs CAPTIONS are the five two-line captions used above")
    say("caption chars (both lines): " + ", ".join(str(len(a) + len(b)) for a, b in caps)
        + "; chars/15 (the map's own lower-bound rule of thumb): " + ", ".join(f"{(len(a) + len(b)) / 15:.1f}" for a, b in caps) + " s")

    # ------------------------------------------------------------- 7. is there a take of 15_2 at all?
    takes = [r for r in inv if r.get("stage04_cue") == "15_2" or "15_2" in r.get("submitted_text", "")[:0]]
    prior = [r["path"] for r in inv if "though not always everywhere" in r.get("submitted_text", "")
             or "though there may be none" in r.get("submitted_text", "")]
    say(f"sealed audio inventory: recordings of stage04 15_2: {len(takes)}; recordings whose text holds either 15_2 clause: {prior or 'none'}")

    say("FAILURES:", FAILS or "none")
    return 1 if FAILS else 0


if __name__ == "__main__":
    sys.exit(main())
