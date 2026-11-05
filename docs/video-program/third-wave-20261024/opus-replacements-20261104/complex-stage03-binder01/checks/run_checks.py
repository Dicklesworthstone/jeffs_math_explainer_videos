#!/usr/bin/env python3
"""Checks for complex-stage03-binder01. Run from the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage03-binder01/checks/run_checks.py FRESH_SCRATCH

FRESH_SCRATCH must not exist. Everything this writes goes there; the repository is only
read. The output names no absolute path and no time, so a clean rerun is byte-equal to
checks/outputs/run_checks-01.txt.

A  Pinned inputs, recomputed here.
B  The sealed bound plans are in step: the wrapper's --check on both stages.
C  Independent recomputation, importing neither binder: the stage04 receipt and its
   delta from stage03, the script, storyboard and film code, the units, every map row
   against its file and line, the map against oliver10, every literal and dynamic
   phrase hook found by a separate scan, the reviewed hook tables against source lines,
   closing's two beats, and no absolute paths in the outputs.
D  Real mismatches in scratch mirrors (fresh copy per test, kept; nothing deleted):
   stale pins exit 3; real older files exit 1; consistent forgeries (receipt and its pin
   rewritten) with a forged control that binds; each semantic plant passes RubyRaven's
   binder alone and is refused by the strict wrapper with its own check's message.
"""

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
L = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
HERE = f"{P}/complex-stage03-binder01"
CR = "explainer_corpus/complex_analysis_visual"
S3, S4 = f"{L}/combined-repair-stage03", f"{L}/combined-repair-stage04"
R3, R4 = f"{L}/combined-repair-stage03-integration01.json", f"{L}/combined-repair-stage04-integration01.json"
S2C = f"{L}/combined-repair-stage02/candidate"
WRAPPER = f"{HERE}/bind_complex_stage03.py"
BINDER = f"{HERE}/binder/bind_oliver_plan.py"
RF = f"{P}/complex-stage03-review-fixes01/patches"

EXPECT = {
    BINDER: "26e10809c39b497cec168e19f462ae13cef74f74bcd671db1ab54fbe07e2efd0",
    f"{P}/complex02-05-cross-review04/binder/bind_oliver_plan.py": "26e10809c39b497cec168e19f462ae13cef74f74bcd671db1ab54fbe07e2efd0",
    R3: "3f652c2d8bc510fbbd62e027b9db57943d8d5a1c3e297c0fecdfb87de6dec5e2",
    R4: "295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b",
    "crates/explainer_kit/src/kit.rs": "ef035faf236498e7485fb60a66d862b383e1a5040fa17962e1633596ed99e106",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    f"{P}/complex-stage03-cross-review01/proposed/hold-rebind-stage03.json": "aa7450d4c7908e23a35548edb0545d6c51fe5e21011509f9ca0761b77191f9ea",
    f"{P}/complex11-counting-teaching02/holds/narration-direction-map-oliver11.json": "759e0eba1d40fbd2ff4f715719493dccc35eed27da0f730c6e8c8344a7f1bb91",
    f"{P}/complex11-counting-teaching02/patches/narration-units.patch": "34199ed235b12d8a243a47c22550f76af63963418c668f46286478ff62df2c23",
    f"{RF}/F3-15_2-clause.patch": "0fa7cb79a41615885a96f7917832ac4021ce172b86c8edffa18067f1ac4b2320",
    f"{RF}/F4-turning-one-sample-guard.patch": "8e90cf7e7fa0aa47813a2ac5e07ce66ce2637f07b02d98c359487da5f03328b8",
    f"{RF}/F5-budget-painted-planes.patch": "f3f5907519ed2234046ba82a87cb6e6bf6d24947525d5e99a3eb026c4e1dcd7a",
    f"{RF}/F6-storyboard-quotes.patch": "ad92990c8217ea74656d981eebb8591c390e12eb33e205a76340a5bd89018341",
    f"{HERE}/baseline/canonical-oliver10/narration_units.json": "3c4f53d7973857295e4f99de681e8537ead08676d1ebdfe9625e69b4bc86702a",
    f"{HERE}/baseline/canonical-oliver10/narration-direction-map-oliver10.json": "341356cda34ac129fbca1988699f9f20f1cb583d63a5463f780226a13ab3c5b4",
    f"{HERE}/baseline/canonical-oliver10/source-audit.json": "d3423e0d20d4067a3c44e5e1b828795f6380ec395ae1ebf9a4de0e7b256d425f",
    f"{HERE}/baseline/canonical-oliver10/script.tsv": "2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e",
}
DECLARED = {
    f"{CR}/script.tsv": ("1e24aba8b520f2e61e8b2bc392a218b664cf5556ad788ceca17fbc45bb92efab", "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3"),
    f"{CR}/docs/storyboard.md": ("57508d96103f0a4a86ed2265a51bb65d28fc68d8e6c96b3bd4438d2d97c83b5f", "e204cf7726dadefe46dccb419e0cc0a641acbeeb4e884c3f55ff6d39a980536b"),
    f"{CR}/src/chapters/counting.rs": ("c1df16ea04d4283d490598ccabe039cdc846be96458668d63544aef9ac81d36a", "d41d414134c4ffc553df9773bc1b082fab376f1a4344d248ab7bdb3f443fe8f9"),
    f"{CR}/src/chapters/draw.rs": ("865599e94331038782be128189fc1acb642e1d00b2f7fe32bd9fe68c958951b9", "3f1c581dbc028a4d1a50a46d15028a0e005e0a76cf58709c6fbbbce2d30ea2db"),
}
F3_OLD, F3_NEW = "though not always everywhere", "though there may be none"
OUT4, OUT3 = f"{HERE}/bound-plan-stage04", f"{HERE}/bound-plan-stage03"
MAP4 = f"{OUT4}/{CR}/docs/narration-direction-map-oliver12.json"
UNITS4 = f"{OUT4}/{CR}/narration_units.json"
MAP3 = f"{OUT3}/{CR}/docs/narration-direction-map-oliver12-stage03.json"

results = []
SCRATCH_NAMES = []  # the scratch path as given and as resolved, shown as <scratch>


def sha(b):
    return hashlib.sha256(b).hexdigest()


def rb(path):
    with open(path, "rb") as h:
        return h.read()


def clean(text):
    """The scratch path, as given or resolved, shown as <scratch> (before any truncation)."""
    for name in SCRATCH_NAMES:
        text = text.replace(name, "<scratch>")
    return text


def record(ok, label, detail=""):
    results.append(ok)
    detail = clean(detail)
    print(f"{'PASS' if ok else 'FAIL'} {label}{(': ' + detail) if detail else ''}")


def script_rows(data):
    rows = {}
    for row in data.decode("utf8").splitlines():
        if row.strip() and not row.startswith("#"):
            k, v = row.split("\t", 1)
            rows[k] = v
    return rows


def sentence_starts(text):
    out = [0]
    for i in range(len(text) - 2):
        if text[i] in ".?!" and text[i + 1] == " ":
            out.append(i + 2)
    return out


def receipt_table(path):
    return {f["path"]: f["sha256"] for f in json.loads(rb(path))["files"]}


# ----------------------------------------------------------------------------- A

def section_a():
    print("== A pinned inputs")
    for rel, want in EXPECT.items():
        got = sha(rb(rel)) if os.path.isfile(rel) else None
        record(got == want, f"A {rel.replace(P + '/', 'P/').replace(L + '/', 'L/')}", want[:12])


# ----------------------------------------------------------------------------- B

def section_b():
    print("== B sealed bound plans in step")
    for stage, out in (("stage04", OUT4), ("stage03", OUT3)):
        r = subprocess.run([sys.executable, "-I", "-B", WRAPPER, "--stage", stage, "--out", out, "--check"], capture_output=True, text=True)
        first = r.stdout.splitlines()[0] if r.stdout else r.stderr.strip()[:200]
        record(r.returncode == 0 and first.startswith("in step"), f"B --check {stage}", first[:90])


# ----------------------------------------------------------------------------- C

def strip_tests(raw):
    """Film code: everything before the one `#[cfg(test)]` / `mod tests {` block. Not the
    first bare #[cfg(test)]: several files carry test-only items above their scenes."""
    block = "\n#[cfg(test)]\nmod tests {"
    assert raw.count(block) == 1, "expected exactly one test module"
    return raw[: raw.index(block)]


def blank(src):
    """Comments, string and char contents to spaces, offsets kept (written apart from both binders)."""
    out = list(src)
    i, n = 0, len(src)

    def fill(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            fill(i, j)
            i = j
        elif src.startswith("/*", i):
            j = src.find("*/", i) + 2
            fill(i, j)
            i = j
        elif src[i] == "r" and re.match(r'r(#*)"', src[i : i + 12]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            m = re.match(r'r(#*)"', src[i : i + 12])
            end = '"' + m.group(1)
            j = src.find(end, i + m.end())
            fill(i + m.end(), j)
            i = j + len(end)
        elif src[i] == '"':
            j = i + 1
            while src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            fill(i + 1, j)
            i = j + 1
        elif src[i] == "'" and re.match(r"'(\\.|[^\\'])'", src[i : i + 5]):
            m = re.match(r"'(\\.|[^\\'])'", src[i : i + 5])
            fill(i + 1, i + m.end() - 1)
            i += m.end()
        else:
            i += 1
    return "".join(out)


def code_only(raw):
    """Film code (before the tests), with a blanked twin at the same offsets."""
    film = strip_tests(raw)
    return film, blank(film)


def fn_spans(text):
    spans = {}
    for m in re.finditer(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", text):
        depth, i = 0, m.end() - 1
        while True:
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        j = i
        while j < len(text) and text[j] not in "{;":
            j += 1
        if j >= len(text) or text[j] == ";":
            continue
        depth, k = 0, j
        while k < len(text):
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        spans.setdefault(m.group(1), (j, k))
    return spans


def top_args(text, open_at, content=None):
    """Top-level arguments of the call opening at open_at: depth read on `text` (blanked),
    the argument text taken from `content` (the film code) at the same offsets."""
    content = text if content is None else content
    depth, start, parts = 0, open_at + 1, []
    i = open_at
    while True:
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                parts.append(content[start:i].strip())
                return parts
        elif c == "," and depth == 1:
            parts.append(content[start:i].strip())
            start = i + 1
        i += 1


def in_code(blanked, pos):
    """True when the match at pos starts in code, not in a comment or a string."""
    return blanked[pos] != " "


SAY = re.compile(r'say\(&kit, stage, "(\d\d_\d+)"\)')


def says_in(film, blanked, a, b):
    return [(x.start(), x.group(1)) for x in SAY.finditer(film) if a <= x.start() <= b and in_code(blanked, x.start())]


def independent_phrases(crate, script):
    """(cue, phrase) for every phrase hook, by a scan separate from both binders."""
    chapters = os.path.join(crate, "src", "chapters")
    mods = re.findall(r"^(?:pub )?mod (\w+);", rb(os.path.join(chapters, "mod.rs")).decode(), re.M)
    pairs = set()
    lit = re.compile(r'line\.(?:wait_for|until)\(\s*stage,\s*"([^"]+)"|line\.at\(\s*"([^"]+)"')
    for m in mods:
        name = f"{m}.rs"
        if name in ("draw.rs", "thumbnail.rs"):
            continue
        raw = rb(os.path.join(chapters, name)).decode()
        film, blanked = code_only(raw)
        spans = fn_spans(blanked)
        if "construct" not in spans:
            continue
        a, b = spans["construct"]
        says = says_in(film, blanked, a, b)

        def owner(pos):
            prior = [c for at, c in says if at < pos]
            return prior[-1] if prior else None

        for x in lit.finditer(film):
            if not in_code(blanked, x.start()):
                continue
            phrase = x.group(1) or x.group(2)
            fn = next((f for f, (s, e) in spans.items() if s <= x.start() <= e and f != "construct"), None)
            if fn is None:
                pairs.add((owner(x.start()), phrase))
            else:
                for c in re.finditer(r"(?:Self::)?\b" + fn + r"\s*\(", blanked[a:b]):
                    pairs.add((owner(a + c.start()), phrase))
        # helper parameter phrases: a helper with `phrase: &str` that hooks on `phrase`
        for fn, (s, e) in spans.items():
            if fn == "construct" or not re.search(r"line\.(?:wait_for|until)\(\s*stage,\s*phrase\b", blanked[s:e]):
                continue
            head = re.search(r"\bfn\s+" + fn + r"\s*\(", blanked)
            params = [p.split(":")[0].strip() for p in top_args(blanked, head.end() - 1)]
            if "phrase" not in params:
                continue
            at = params.index("phrase")
            for c in re.finditer(r"(?:Self::)?\b" + fn + r"\s*\(", blanked[a:b]):
                args = top_args(blanked, a + c.end() - 1, film)
                q = re.fullmatch(r'"([^"]+)"', args[at]) if len(args) > at else None
                if q:
                    pairs.add((owner(a + c.start()), q.group(1)))
    # dynamic loops: the values from the consts and arrays themselves, the owner from the
    # say before the loop's hook in construct
    def loop_pairs(name, hook, values_from):
        raw = rb(os.path.join(chapters, name)).decode()
        film, blanked = code_only(raw)
        a, b = fn_spans(blanked)["construct"]
        says = says_in(film, blanked, a, b)
        at = blanked.find(hook, a)
        cue = [c for p, c in says if p < at][-1]
        return {(cue, v) for v in values_from(raw, film)}

    pairs |= loop_pairs(
        "reading_color.rs",
        "line.wait_for(stage, phrase)?;",
        lambda raw, _t: re.findall(r'\(\s*"([^"]+)"', re.search(r"const NAMED: \[\(&str, f64\); 4\] = \[(.*?)\];", raw, re.S).group(1)),
    )
    pairs |= loop_pairs(
        "gallery.rs",
        "line.wait_for(stage, phrase)?;",
        lambda _r, t: re.findall(r'"([^"]+)"', re.search(r"let named = \[(.*?)\];", t).group(1)),
    )
    pairs |= loop_pairs(
        "counting.rs",
        "line.wait_for(stage, NEEDS_SAID[i])?;",
        lambda raw, _t: re.findall(r'"([^"]+)"', re.search(r"const NEEDS_SAID: \[&str; 3\] = \[(.*?)\];", raw).group(1)),
    )
    pairs |= loop_pairs(
        "the_bump.rs",
        "line.wait_for(stage, phrase)?;",
        lambda _r, t: re.findall(r'"([^"]+)"', re.search(r"for \(deg, phrase\) in \[(.*?)\] \{", t).group(1)),
    )
    return pairs


def section_c():
    print("== C independent recomputation")
    t3, t4 = receipt_table(R3), receipt_table(R4)
    on_disk = sorted(
        os.path.relpath(os.path.join(dp, f), S4).replace(os.sep, "/") for dp, _d, fs in os.walk(S4) for f in fs
    )
    record(sorted(t4) == on_disk and all(sha(rb(os.path.join(S4, p))) == h for p, h in t4.items()), "C stage04 receipt lists exactly the 55 files on disk with their hashes", str(len(t4)))
    diff = {p: (t3.get(p), t4.get(p)) for p in set(t3) | set(t4) if t3.get(p) != t4.get(p)}
    record(diff == DECLARED, "C stage04 differs from stage03 in exactly the four declared files, at the declared hashes", ", ".join(sorted(p.split("/")[-1] for p in diff)))
    a = rb(os.path.join(S3, CR, "script.tsv"))
    b = rb(os.path.join(S4, CR, "script.tsv"))
    record(a.count(F3_OLD.encode()) == 1 and b == a.replace(F3_OLD.encode(), F3_NEW.encode()), "C stage04 script is stage03's with only the F3 clause", sha(b)[:12])
    sa, sb = script_rows(a), script_rows(b)
    changed = [c for c in sa if sa[c] != sb.get(c)]
    oa, ob = sentence_starts(sa["15_2"]), sentence_starts(sb["15_2"])
    same04 = all(sa["15_2"][oa[k] : oa[k + 1]] == sb["15_2"][ob[k] : ob[k + 1]] for k in range(5))
    record(changed == ["15_2"] and len(sa) == len(sb) == 105 and len(oa) == len(ob) == 6 and same04, "C 15_2 keeps six sentences; sentences 0-4 byte-equal; no other cue changes")
    la = rb(os.path.join(S3, CR, "docs/storyboard.md")).decode().split("\n")
    lb = rb(os.path.join(S4, CR, "docs/storyboard.md")).decode().split("\n")
    lines = [i + 1 for i in range(max(len(la), len(lb))) if (la[i] if i < len(la) else None) != (lb[i] if i < len(lb) else None)]
    words = [c.strip() for c in lb[283].split("|")][2]
    record(lines == [134, 138, 151, 284] and words == sb["15_2"], "C storyboard differs only at 134, 138, 151, 284; 284's words equal the new 15_2", str(lines))
    for name in ("counting.rs", "draw.rs"):
        x = rb(os.path.join(S3, CR, "src/chapters", name)).decode()
        y = rb(os.path.join(S4, CR, "src/chapters", name)).decode()
        record(strip_tests(x) == strip_tests(y) and x != y, f"C {name}: film code before its test module byte-equal, tests differ")
    units = json.loads(rb(UNITS4))
    mp_bytes = rb(MAP4)
    mp = json.loads(mp_bytes)
    ok = (
        [u["id"] for u in units["units"]] == [f"oliver_{c}" for c in sb]
        and all(u["text"] == sb[u["id"][7:]] and u["line_ids"] == [u["id"][7:]] for u in units["units"])
        and units["script_sha256"] == units["direction_contract"]["script_sha256"] == sha(b)
        and units["direction_contract"]["map"] == {"path": f"{CR}/docs/narration-direction-map-oliver12.json", "sha256": sha(mp_bytes)}
    )
    record(ok, "C stage04 units: 105 units equal the script, bound to it and to the oliver12 map's bytes")
    bad = []
    for section in ("visual_holds", "helpers"):
        for row in mp[section]:
            path = os.path.join(S4, row["path"])
            text = rb(path)
            if sha(text) != row["sha256"] or text.decode().split("\n")[row["line"] - 1].strip() != row["marker"]:
                bad.append(row["path"].split("/")[-1] + ":" + str(row["line"]))
    record(not bad and mp["scripts"] == {"complex_analysis_visual": sha(b)}, f"C every oliver12 row's file hash and marker line hold in stage04 ({len(mp['visual_holds'])} holds, {len(mp['helpers'])} helpers)", ", ".join(bad))
    m3 = json.loads(rb(MAP3))
    moves = []
    others = []
    for section in ("visual_holds", "helpers"):
        for r3, r4 in zip(m3[section], mp[section]):
            for k in set(r3) | set(r4):
                if r3.get(k) != r4.get(k):
                    (moves if k == "sha256" else others).append((section, r3.get("cue") or r3["marker"].split("(")[0], k))
    top = [k for k in set(m3) | set(mp) if k not in ("visual_holds", "helpers") and m3.get(k) != mp.get(k)]
    record(len(moves) == 6 and not others and top == ["scripts"], "C oliver12 vs stage03's binding: six row hashes and the script hash, no line or marker", str(sorted(x[1] for x in moves)))
    o10 = json.loads(rb(f"{HERE}/baseline/canonical-oliver10/narration-direction-map-oliver10.json"))
    changed_rows = [(s, r.get("cue") or r["marker"].split("(")[0]) for s in ("visual_holds", "helpers") for r, q in zip(o10[s], mp[s]) if r != q]
    fields = {k for s in ("visual_holds", "helpers") for r, q in zip(o10[s], mp[s]) for k in r if r.get(k) != q.get(k)}
    marker_rows = [r["cue"] for r, q in zip(o10["visual_holds"], mp["visual_holds"]) if r["marker"] != q["marker"]]
    record(len(changed_rows) == 25 and fields == {"sha256", "line", "marker"} and marker_rows == ["03_3"], "C oliver12 vs oliver10: 20 holds + 5 helpers move; only 03_3's marker changes", f"{len(changed_rows)} rows")
    audit = json.loads(rb(f"{OUT4}/strict-audit.json"))
    tool = {(cue, r["value"]) for cue, rows in audit["hooks"].items() for r in rows if isinstance(r["value"], str)}
    mine = independent_phrases(os.path.join(S4, CR), sb)
    record(tool == mine, f"C phrase hooks: a separate scan finds the same {len(mine)} (cue, phrase) pairs as the strict audit", str(sorted(tool ^ mine))[:200])
    bad = []
    for cue, phrase in sorted(mine):
        text = sb.get(cue, "")
        n = sum(1 for i in range(len(text)) if text.startswith(phrase, i))
        at = text.find(phrase)
        if n != 1 or (at > 0 and text[at - 1].isalnum()):
            bad.append((cue, phrase, n))
    record(not bad, "C every phrase occurs once in its cue and starts a word", str(bad))
    cnt = rb(os.path.join(S4, CR, "src/chapters/counting.rs")).decode().split("\n")
    clo = rb(os.path.join(S4, CR, "src/chapters/closing.rs")).decode().split("\n")
    want_lines = {
        525: 'line.wait_for(stage, "complex polynomial")?;',
        529: "let rt = (line.sentence(1)? - stage.scene().time().to_f64()).clamp(0.7, 1.4);",
        532: "line.wait_sentence(stage, 1)?;",
        536: 'let rt = line.until(stage, "which leaves", 1.2)?;',
        554: 'line.wait_for(stage, "with z")?;',
        563: "line.wait_sentence(stage, 3)?;",
        571: "line.wait_sentence(stage, 4)?;",
        573: "let rt = (line.sentence(6)? - stage.scene().time().to_f64()).max(5.0);",
    }
    s117 = sb["11_7"]
    o117 = sentence_starts(s117)
    lead = {1: "Divide by its leading coefficient", 3: "On a huge circle", 4: "Picture z to the n as a walker", 6: "So there are exactly n roots inside"}
    ok = all(cnt[n - 1].strip() == t for n, t in want_lines.items()) and all(s117[o117[k] :].startswith(v) for k, v in lead.items())
    record(ok, "C 11_7's reviewed hooks: source lines 525-573 and the sentences they name")
    want_close = {
        321: "line.wait_sentence(stage, 1)?;",
        329: "0 => line.wait_sentence(stage, 2)?,",
        330: "1 => line.wait_sentence(stage, 3)?,",
        331: "2 => line.wait_sentence(stage, 4)?,",
        332: '3 => line.wait_for(stage, "and it turns a real integral")?,',
        333: "_ => line.wait_sentence(stage, 5)?,",
        306: "wait_until(stage, line.end())?;",
        308: "pause(stage, 1.0)?;",
        311: "kit.hold(stage)?;",
        312: 'let line = say(&kit, stage, "15_2")?;',
        347: "wait_until(stage, line.end())?;",
        349: "pause(stage, 1.2)?;",
        352: "kit.hold(stage)?;",
        353: 'let line = say(&kit, stage, "15_3")?;',
    }
    between = [clo[n - 1].strip() for n in (307, 309, 310, 348, 350, 351)]
    ok = all(clo[n - 1].strip() == t for n, t in want_close.items()) and all(x == "" or x.startswith("//") for x in between)
    record(ok, "C closing: 15_2's hooks; after each beat's voice end only comments, the pause, kit.hold, the next say")
    absolute = []
    for root in (OUT4, OUT3, f"{HERE}/baseline"):
        for dp, _d, fs in os.walk(root):
            for f in fs:
                t = rb(os.path.join(dp, f))
                if any(n in t for n in (b"/Us" + b"ers/", b"/priv" + b"ate/", b"/ho" + b"me/")):
                    absolute.append(f)
    record(not absolute, "C no absolute path in the bound plans or the baseline", str(absolute))


# ----------------------------------------------------------------------------- D

MIRROR_FILES = [WRAPPER, BINDER, *[k for k in EXPECT if k not in (R3, R4)], R3, R4]


def copy_tree(src, dst):
    for dp, _d, fs in os.walk(src):
        for f in fs:
            s = os.path.join(dp, f)
            d = os.path.join(dst, os.path.relpath(s, src))
            os.makedirs(os.path.dirname(d), exist_ok=True)
            shutil.copyfile(s, d)


def mirror(scratch, name, stage02=False):
    m = os.path.join(scratch, "mirrors", name)
    os.makedirs(m)
    for rel in MIRROR_FILES:
        os.makedirs(os.path.dirname(os.path.join(m, rel)), exist_ok=True)
        shutil.copyfile(rel, os.path.join(m, rel))
    for tree in (S3, S4) + ((S2C,) if stage02 else ()):
        copy_tree(tree, os.path.join(m, tree))
    return m


def edit(m, rel, old, new, count=1):
    path = os.path.join(m, rel)
    text = rb(path).decode()
    if text.count(old) != count:
        raise SystemExit(f"plant text not found {count}x in {rel}: {old[:60]}")
    with open(path, "w", encoding="utf8", newline="") as h:
        h.write(text.replace(old, new))


def forge(m, stage, extra_changed=(), declared_after=None):
    """Rewrite the stage's receipt to the planted bytes, and the mirror wrapper's pin to it."""
    root, receipt = (S4, R4) if stage == "stage04" else (S3, R3)
    data = json.loads(rb(os.path.join(m, receipt)))
    tree = os.path.join(m, root)
    data["files"] = [
        {"path": p, "sha256": sha(rb(os.path.join(tree, p)))}
        for p in sorted(os.path.relpath(os.path.join(dp, f), tree).replace(os.sep, "/") for dp, _d, fs in os.walk(tree) for f in fs)
    ]
    data["changed"] = sorted(set(data["changed"]) | set(extra_changed))
    new = (json.dumps(data, indent=2) + "\n").encode()
    old_sha = sha(rb(os.path.join(m, receipt)))
    with open(os.path.join(m, receipt), "wb") as h:
        h.write(new)
    edit(m, WRAPPER, old_sha, sha(new))
    if declared_after:
        for old, nw in declared_after:
            edit(m, WRAPPER, old, nw)


def wrapper(m, stage, out="out", extra=()):
    r = subprocess.run(
        [sys.executable, "-I", "-B", os.path.join(m, WRAPPER), "--repo", m, "--stage", stage, "--out", os.path.join(m, out), *extra],
        capture_output=True,
        text=True,
        cwd=m,
    )
    return r.returncode, r.stdout + r.stderr


def binder_alone(m, stage):
    root = S4 if stage == "stage04" else S3
    r = subprocess.run(
        [sys.executable, "-I", "-B", os.path.join(m, BINDER), m, os.path.join(m, root, CR), os.path.join(m, "binder-alone"), "--map-version", "plant"],
        capture_output=True,
        text=True,
        cwd=m,
    )
    return r.returncode


def expect(label, got, want_code, needle):
    code, text = got
    text = clean(text)
    ok = code == want_code and needle in text
    first = next((x for x in text.splitlines() if needle in x), text.splitlines()[0] if text else "")
    record(ok, f"D {label}", f"exit {code}; {first.replace(chr(10), ' ')[:150]}")


def semantic(scratch, label, stage, plants, needle, extra_changed=(), declared_after=None, binder_should_pass=True):
    m = mirror(scratch, label.split()[0], stage02=(stage == "stage03"))
    for rel, old, new in plants:
        edit(m, rel, old, new)
    forge(m, stage, extra_changed=extra_changed, declared_after=declared_after)
    alone = binder_alone(m, stage)
    code, text = wrapper(m, stage)
    text = clean(text)
    ok = code == 1 and needle in text and ((alone == 0) if binder_should_pass else (alone != 0))
    first = next((x for x in text.splitlines() if needle in x), "")
    record(ok, f"D {label}", f"binder alone exit {alone}; wrapper exit {code}; {first[:150]}")


def section_d(scratch):
    print("== D real mismatches in scratch mirrors")
    m = mirror(scratch, "d00-control")
    code, text = wrapper(m, "stage04")
    same = all(rb(os.path.join(m, "out", rel)) == rb(os.path.join(OUT4, rel)) for rel in (f"{CR}/narration_units.json", f"{CR}/docs/narration-direction-map-oliver12.json", "source-audit.json", "strict-audit.json", "delta-from-oliver10.json", "delta-from-stage03.json")) if code == 0 else False
    record(code == 0 and same, "D d00 unchanged mirror binds stage04 to the sealed bytes", f"exit {code}")
    for name, rel in (("d01-stale-receipt", R4), ("d02-stale-kit", "crates/explainer_kit/src/narration.rs"), ("d03-stale-binder", BINDER), ("d04-stale-f3-patch", f"{RF}/F3-15_2-clause.patch")):
        m = mirror(scratch, name)
        with open(os.path.join(m, rel), "ab") as h:
            h.write(b" ")
        expect(f"{name} one byte appended to a pinned input", wrapper(m, "stage04"), 3, "STALE: ")
    m = mirror(scratch, "d05-old-script")
    shutil.copyfile(os.path.join(S3, CR, "script.tsv"), os.path.join(m, S4, CR, "script.tsv"))
    expect("d05 stage03's real script.tsv in stage04", wrapper(m, "stage04"), 1, "script.tsv differs from its receipt hash")
    m = mirror(scratch, "d06-old-counting")
    shutil.copyfile(os.path.join("explainer_corpus/complex_analysis_visual/src/chapters/counting.rs"), os.path.join(m, S4, CR, "src/chapters/counting.rs"))
    expect("d06 canonical's real counting.rs (0ecda9cf) in stage04", wrapper(m, "stage04"), 1, "counting.rs differs from its receipt hash")
    m = mirror(scratch, "d07-forged-control")
    forge(m, "stage04")
    code, text = wrapper(m, "stage04")
    record(code == 0, "D d07 forged receipt and pin with no source change still binds", f"exit {code}")
    semantic(scratch, "d08 S3 NAMED middle phrase", "stage03", [(f"{S3}/{CR}/src/chapters/reading_color.rs", '("Pointing up is yellow green", FRAC_PI_2),', '("up is yellow green", FRAC_PI_2),')], "S3 reading_color.rs: source gives", extra_changed=[f"{CR}/src/chapters/reading_color.rs"])
    semantic(
        scratch,
        "d09 S3 walk_and_count call-site literals swapped",
        "stage03",
        [
            (f"{S3}/{CR}/src/chapters/gallery.rs", '            &line,\n            "Three zeros inside",', '            &line,\n            "BACKWARDS_PLACEHOLDER",'),
            (f"{S3}/{CR}/src/chapters/gallery.rs", '            &line,\n            "backwards twice",', '            &line,\n            "Three zeros inside",'),
            (f"{S3}/{CR}/src/chapters/gallery.rs", '"BACKWARDS_PLACEHOLDER"', '"backwards twice"'),
        ],
        "S3 gallery.rs: call-site literals give",
        extra_changed=[f"{CR}/src/chapters/gallery.rs"],
    )
    dog = "        let dog = DogWalk::build(&kit, stage, wide, out)?;\n        line.wait_sentence(stage, {})?;\n"
    semantic(scratch, "d10 S6 11_7 dog wait reverted to sentence 3", "stage03", [(f"{S3}/{CR}/src/chapters/counting.rs", dog.format(4), dog.format(3))], "S6 cue 11_7's hooks no longer match", extra_changed=[f"{CR}/src/chapters/counting.rs"])
    semantic(scratch, "d11 S1 11_7 hook on a phrase said three times", "stage03", [(f"{S3}/{CR}/src/chapters/counting.rs", 'line.wait_for(stage, "with z")?;', 'line.wait_for(stage, "z to the n")?;')], "S1 11_7", extra_changed=[f"{CR}/src/chapters/counting.rs"])
    semantic(
        scratch,
        "d12 S2 08_3 phrases swapped out of spoken order",
        "stage03",
        [
            (f"{S3}/{CR}/src/chapters/loop_sums.rs", 'line.until(stage, "As the steps shrink", 2.4)', 'line.until(stage, "whatever the size", 2.4)'),
            (f"{S3}/{CR}/src/chapters/loop_sums.rs", '        line.wait_for(stage, "whatever the size")?;', '        line.wait_for(stage, "As the steps shrink")?;'),
        ],
        "S2 08_3",
        extra_changed=[f"{CR}/src/chapters/loop_sums.rs"],
    )
    semantic(scratch, "d13 S4 something arrives between 15_1's voice end and its beat", "stage03", [(f"{S3}/{CR}/src/chapters/closing.rs", "        // the film's first picture, answered: it gets a moment before the wipe\n", "        fade_in_all(stage, &[ring], 0.2)?;\n")], "S4 closing.rs: between 15_1's voice end", extra_changed=[f"{CR}/src/chapters/closing.rs"])
    semantic(scratch, "d14 binder guard surfaced: 15_2's beat lengthened", "stage03", [(f"{S3}/{CR}/src/chapters/closing.rs", "pause(stage, 1.2)?;", "pause(stage, 2.5)?;")], "binder: closing.rs", extra_changed=[f"{CR}/src/chapters/closing.rs"], binder_should_pass=False)
    semantic(
        scratch,
        "d15 cross-check: 04_4 reading window 7.5 -> 8 s in table and storyboard",
        "stage03",
        [(f"{S3}/{CR}/src/chapters/mod.rs", '    ("04_4", 7.5),', '    ("04_4", 8.0),'), (f"{S3}/{CR}/docs/storyboard.md", "| read 7.5 s |", "| read 8 s |")],
        "cross-check: the map is not",
        extra_changed=[f"{CR}/src/chapters/mod.rs", f"{CR}/docs/storyboard.md"],
    )
    semantic(
        scratch,
        "d16 a second script change beside F3 (03_1, script and storyboard)",
        "stage04",
        [(f"{S4}/{CR}/script.tsv", "Its graph would need four dimensions.", "Its graph would need four axes."), (f"{S4}/{CR}/docs/storyboard.md", "Its graph would need four dimensions.", "Its graph would need four axes.")],
        "is neither stage03's",
    )
    semantic(scratch, "d17 F3 in the script, storyboard left at stage03's", "stage04", [(f"{S4}/{CR}/docs/storyboard.md", F3_NEW, F3_OLD)], "docs/storyboard.md is", binder_should_pass=False)
    semantic(scratch, "d18 an undeclared file changes in stage04 (a the_bump comment)", "stage04", [(f"{S4}/{CR}/src/chapters/the_bump.rs", "//! 01_the_bump: a perfectly smooth curve", "//! 01_the_bump: a perfectly smooth  curve")], "is not in the declared successor", extra_changed=[f"{CR}/src/chapters/the_bump.rs"])
    m = mirror(scratch, "d19-film-code")
    edit(m, f"{S4}/{CR}/src/chapters/counting.rs", 'line.wait_for(stage, "with z")?;', 'line.wait_for(stage, "with z")?; ')
    planted = sha(rb(os.path.join(m, S4, CR, "src/chapters/counting.rs")))
    forge(m, "stage04", declared_after=[(DECLARED[f"{CR}/src/chapters/counting.rs"][1], planted)])
    expect("d19 film code of a declared file changes, declared hash forged to match", wrapper(m, "stage04"), 1, "film code (everything before its one `#[cfg(test)] mod tests`) differs")
    m = mirror(scratch, "d19b-film-code-below-416")
    edit(m, f"{S4}/{CR}/src/chapters/draw.rs", "// a separate clock drives the ring, as the quiz countdowns do", "// a separate clock drives the ring,  as the quiz countdowns do")
    planted = sha(rb(os.path.join(m, S4, CR, "src/chapters/draw.rs")))
    forge(m, "stage04", declared_after=[(DECLARED[f"{CR}/src/chapters/draw.rs"][1], planted)])
    expect("d19b draw.rs film code below its `#[cfg(test)] pub fn contains` (line 416) changes, declared hash forged", wrapper(m, "stage04"), 1, "draw.rs's film code")
    m = mirror(scratch, "d20-strays-aside")
    aside = os.path.join(scratch, "mirrors", "d20-moved-aside")
    for rel in ("src/chapters/counting.rs.orig", "src/chapters/mod.rs.orig", "src/chapters/one_derivative.rs.orig", ".ee/hook-state/session_start_orient.last.json"):
        dst = os.path.join(aside, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        os.rename(os.path.join(m, S4, CR, rel), dst)
    forge(m, "stage04")
    code, text = wrapper(m, "stage04")
    a = json.loads(rb(os.path.join(m, "out", "strict-audit.json"))) if code == 0 else {}
    record(code == 0 and len(a.get("receipt", {}).get("f2_strays_moved_aside", [])) == 4, "D d20 F2 strays moved aside (renamed out, kept) still binds and is recorded", f"exit {code}")
    m = mirror(scratch, "d21-installed")
    shutil.copyfile(UNITS4, os.path.join(m, S4, CR, "narration_units.json"))
    shutil.copyfile(MAP4, os.path.join(m, S4, CR, "docs/narration-direction-map-oliver12.json"))
    forge(m, "stage04", extra_changed=[f"{CR}/narration_units.json", f"{CR}/docs/narration-direction-map-oliver12.json"])
    code, text = wrapper(m, "stage04")
    record(code == 0, "D d21 the bound plan installed in stage04 still binds (installed equals binding)", f"exit {code}")
    m = mirror(scratch, "d22-installed-wrong")
    shutil.copyfile(f"{OUT3}/{CR}/narration_units.json", os.path.join(m, S4, CR, "narration_units.json"))
    forge(m, "stage04", extra_changed=[f"{CR}/narration_units.json"])
    expect("d22 stage03's units installed in stage04", wrapper(m, "stage04"), 1, "is installed but is not this binding")
    m = mirror(scratch, "d23-out")
    expect("d23 OUT inside the stage04 tree", wrapper(m, "stage04", out=f"{S4}/bound"), 2, "refused: OUT")
    expect("d24 OUT inside another bundle", wrapper(m, "stage04", out=f"{P}/complex-stage03-cross-review01/bound"), 2, "another bundle")
    expect("d25 map version oliver11", wrapper(m, "stage04", extra=("--map-version", "oliver11")), 2, "already used")
    os.makedirs(os.path.join(m, "out-taken"))
    with open(os.path.join(m, "out-taken", "strict-audit.json"), "w") as h:
        h.write("{}\n")
    expect("d26 OUT holds a file with other bytes", wrapper(m, "stage04", out="out-taken"), 1, "exists in OUT with other bytes")
    expect("d27 --check on a stale OUT", wrapper(m, "stage04", out="out-taken", extra=("--check",)), 1, "STALE: ")


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: run_checks.py FRESH_SCRATCH")
    scratch = sys.argv[1]
    if os.path.exists(scratch):
        raise SystemExit("refused: scratch exists; pass a fresh path")
    os.makedirs(scratch)
    SCRATCH_NAMES.extend(sorted({os.path.realpath(scratch), os.path.abspath(scratch)}, key=len, reverse=True))
    section_a()
    section_b()
    section_c()
    section_d(scratch)
    failed = results.count(False)
    print(f"{len(results)} checks, {failed} failed")
    print(f"FAILURES: {'none' if not failed else failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
