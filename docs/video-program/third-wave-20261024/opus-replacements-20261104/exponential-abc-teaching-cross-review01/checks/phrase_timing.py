"""until_clause against the corpus phrase rule: same draft timing, different behaviour with marks and with drift.

Usage from the repo root:  python3 -I -B phrase_timing.py SCRATCH_DIR
The new rule is real Rust: `phrase_seconds` and its test module are cut from candidate/phrase.rs, and `sentences` and
`estimated_starts` from the Kit, then compiled with rustc (std only) into SCRATCH_DIR. The old rule is a Python
transcription of the arithmetic in until_clause, with its constants read out of the frozen source.
Section F measures word marks on one other film's Oliver takes. Those marks are recognition estimates, not heard
boundaries. Nothing here is a listening or picture claim. Nothing is written in the repo.
"""
import hashlib
import json
import os
import re
import statistics
import subprocess
import sys

R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R + "exponential-abc-teaching-repair01/candidate/"
O = R + "exponential-abc-teaching-cross-review01/"
GOLAY = "narration_lab/third-wave-20261024/root/elevenlabs-golay-20261028/narration/"
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def cut(source, signature):
    at = source.index(signature)
    start = source.rfind("\n", 0, at) + 1
    depth = 0
    for i in range(at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    raise ValueError(signature)


def slices():
    rows = {}
    for name in ("series01", "product01"):
        for line in open(C + name + "/script-slice.tsv", encoding="utf-8").read().splitlines():
            key, text = line.split("\t", 1)
            rows[key] = text
    return rows


kit = open("crates/explainer_kit/src/narration.rs", encoding="utf-8").read()
phrase = open(O + "candidate/phrase.rs", encoding="utf-8").read()
WPS = float(re.search(r"const SCRATCH_WPS: f64 = ([\d.]+);", kit).group(1))
harness = (
    "#![allow(dead_code)]\nuse std::collections::HashMap;\nuse std::io::BufRead;\n"
    + cut(kit, "fn sentences(") + "\n" + cut(kit, "fn estimated_starts(") + "\n" + cut(phrase, "fn phrase_seconds(") + "\n"
    + r"""
fn nums(s: &str) -> Vec<f64> { s.split(',').filter(|x| !x.is_empty()).map(|x| x.parse().unwrap()).collect() }
fn main() {
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let f: Vec<&str> = line.split('\t').collect();
        if f[0] == "est" {
            let v: Vec<String> = estimated_starts(f[2], f[1].parse().unwrap()).iter().map(|x| format!("{x:.17e}")).collect();
            println!("{}", v.join(","));
        } else {
            let mut marks = HashMap::new();
            for pair in f[2].split(',').filter(|x| !x.is_empty()) {
                let (b, s) = pair.split_once(':').unwrap();
                marks.insert(b.parse::<usize>().unwrap(), s.parse::<f64>().unwrap());
            }
            match phrase_seconds(f[5], f[1], &marks, &nums(f[3]), f[4].parse().unwrap()) {
                Ok(x) => println!("OK {x:.17e}"),
                Err(e) => println!("ERR {e}"),
            }
        }
    }
}
"""
)
path = os.path.join(S, "phrase_cut.rs")
with open(path, "w", encoding="utf-8") as out:
    out.write(harness)
binary = os.path.join(S, "phrase_cut")
r = subprocess.run(["rustc", "--edition", "2024", "-o", binary, path], capture_output=True, text=True)
ck(r.returncode == 0, "phrase_seconds and the Kit's sentence functions compile, cut verbatim", r.stderr[-600:].replace(S, "<scratch>") if r.returncode else "")


def ask(rows):
    lines = "\n".join("\t".join(row) for row in rows) + "\n"
    return subprocess.run([binary], input=lines, capture_output=True, text=True, check=True).stdout.splitlines()


def new_rule(text, words, starts, length, marks=None):
    mark_text = ",".join(f"{b}:{s!r}" for b, s in (marks or {}).items())
    row = ["phrase", words, mark_text, ",".join(repr(x) for x in starts), repr(length), text]
    answer = ask([row])[0]
    return float(answer[3:]) if answer.startswith("OK ") else answer


def old_rule(starts, length, j, sentence, clause):
    """The arithmetic of until_clause, seconds from the line start."""
    start = starts[j]
    end = starts[j + 1] if j + 1 < len(starts) else starts[0] + length
    byte = sentence.encode().find(clause.encode())
    return start + (end - start) * byte / len(sentence.encode())


def scratch(text):
    speech = len(text.split()) / WPS
    starts = [float(x) for x in ask([["est", repr(speech), text]])[0].split(",")]
    return starts, speech + 0.4


script = slices()
factorials = open(C + "series01/factorials.rs", encoding="utf-8").read()
grid = open(C + "product01/product_grid.rs", encoding="utf-8").read()


def const(source, name):
    return re.search(rf'const {name}: &str =\s*"([^"]*)";', source).group(1)


CALLS = [
    ("02_4", 4, const(factorials, "OBLIGATIONS_SENTENCE"), "and that differentiating"),
    ("02_6", 3, const(factorials, "DEBTS_SENTENCE"), "why adding"),
    ("02_6", 3, const(factorials, "DEBTS_SENTENCE"), "why the stepped"),
    ("02_6", 3, const(factorials, "DEBTS_SENTENCE"), "why it is"),
    ("03_2", 0, const(grid, "ROWS_SENTENCE"), "collect them"),
    ("03_7", 3, const(grid, "LADDER_SENTENCE"), "reciprocals"),
    ("03_7", 3, const(grid, "LADDER_SENTENCE"), "continuity"),
]

print(f"--- A. no marks: the two rules agree (scratch voice at {WPS} words/s, then invented uneven onsets)")
worst = 0.0
for cue, j, sentence, clause in CALLS:
    text = script[cue]
    ck(sentence in text, f"{cue}: the constant is in the slice")
    ck(text.count(clause) == 1, f"{cue}: {clause!r} occurs once in the whole line")
    starts, length = scratch(text)
    old, new = old_rule(starts, length, j, sentence, clause), new_rule(text, clause, starts, length)
    worst = max(worst, abs(old - new))
    print(f"    {cue} s{j} {clause!r}: until_clause {old:.3f} s, until_phrase {new:.3f} s")
    uneven = [starts[0]] + [starts[i] + 0.37 * i + 0.11 * (i % 2) for i in range(1, len(starts))]
    far = length + 0.37 * len(starts)
    old2, new2 = old_rule(uneven, far, j, sentence, clause), new_rule(text, clause, uneven, far)
    worst = max(worst, abs(old2 - new2))
ck(worst < 1e-9, "the two rules give the same time at every call site", f"largest difference {worst:.2e} s")
ck("collecting the" in script["03_2"] and script["03_2"].find("collect them") > script["03_2"].find("collecting the"), "'collect them' does not match 'collecting the'")

print("--- B. a planted word mark")
for cue, j, sentence, clause in CALLS:
    text = script[cue]
    starts, length = scratch(text)
    byte = text.encode().find(clause.encode())
    share = new_rule(text, clause, starts, length)
    planted = share + 0.7
    got = new_rule(text, clause, starts, length, {byte: planted})
    missed = new_rule(text, clause, starts, length, {byte + 1: planted})
    old = old_rule(starts, length, j, sentence, clause)
    ck(abs(got - planted) < 1e-12 and abs(missed - share) < 1e-12 and abs(old - share) < 1e-9, f"{cue} {clause!r}: until_phrase takes the mark and ignores one on another byte; until_clause stays {planted - old:+.2f} s off it")
ck("phrase_marks" not in cut(factorials, "fn until_clause(") and "phrase_marks" not in cut(grid, "fn until_clause("), "neither until_clause reads phrase_marks")

print("--- C. drift: what happens when the script moves and the code does not")
cue, j, sentence, clause = CALLS[5]
text = script[cue]
starts, length = scratch(text)
reworded = text.replace("reciprocals give negatives", "division gives negatives")
starts2, length2 = scratch(reworded)
result = new_rule(reworded, clause, starts2, length2)
old = old_rule(starts2, length2, j, sentence, clause)
ck(isinstance(result, str) and result.startswith("ERR"), "the phrase is reworded away: until_phrase refuses", str(result))
print(f"    until_clause on the same script still returns {old:.3f} s from the stale constant, with no error")
split_low = script["02_6"].replace("We still owe three explanations: why", "We still owe three explanations. why")
assert split_low != script["02_6"]
cue, j, sentence, clause = CALLS[2]
starts3, length3 = scratch(split_low)
wrong = old_rule(starts3, length3, j, sentence, clause)
right = new_rule(split_low, clause, starts3, length3)
ck(isinstance(right, float) and abs(wrong - right) > 1.0, f"a colon becomes a full stop in 02_6: until_clause's fixed s{j} lands {wrong - right:+.2f} s from the phrase; until_phrase follows the text")
result = new_rule(script["03_7"], "continuity", [0.0, 1.0], 9.0)
ck(isinstance(result, str) and result.startswith("ERR"), "a narrator with a different sentence count: until_phrase refuses", str(result))
canonical = dict(line.split("\t", 1) for line in open("explainer_corpus/exponential_magic/script.tsv", encoding="utf-8").read().splitlines() if "\t" in line)
if "03_2" in canonical:
    starts4, length4 = scratch(canonical["03_2"])
    result = new_rule(canonical["03_2"], "collect them", starts4, length4)
    ck(isinstance(result, str) and result.startswith("ERR"), "the unmerged canonical script's 03_2 is another line: until_phrase refuses", str(result))
    print(f"    until_clause against that line: {old_rule(starts4, length4, 0, const(grid, 'ROWS_SENTENCE'), 'collect them'):.3f} s, no error")

print("--- D. the module's own tests, cut with phrase_seconds and run")
test_mod = phrase[phrase.index("#[cfg(test)]") :]
test_src = cut(phrase, "fn phrase_seconds(") + "\n" + test_mod
test_src = "use std::collections::HashMap;\n" + test_src
path = os.path.join(S, "phrase_tests.rs")
with open(path, "w", encoding="utf-8") as out:
    out.write(test_src)
binary_t = os.path.join(S, "phrase_tests")
r = subprocess.run(["rustc", "--edition", "2024", "--test", "-o", binary_t, path], capture_output=True, text=True)
ck(r.returncode == 0, "the tests compile", r.stderr[-600:].replace(S, "<scratch>") if r.returncode else "")
r = subprocess.run([binary_t], capture_output=True, text=True)
for line in r.stdout.splitlines():
    if line.startswith("test "):
        print("    " + line)
ck(r.returncode == 0, "the tests pass")
mutated = test_src.replace("if let Some(&seconds) = marks.get(&byte) {", "if let Some(&seconds) = marks.get(&(byte + 1)) {")
assert mutated != test_src
path = os.path.join(S, "phrase_tests_mut.rs")
with open(path, "w", encoding="utf-8") as out:
    out.write(mutated)
subprocess.run(["rustc", "--edition", "2024", "--test", "-A", "warnings", "-o", binary_t + "_mut", path], check=True, capture_output=True)
ck(subprocess.run([binary_t + "_mut"], capture_output=True, text=True).returncode != 0, "control: a mark looked up on the wrong byte fails the tests")

print("--- E. draft timing of the 03_7 wording option")
option = script["03_7"].replace("Now we can name it. Call E of one by the letter e.", "Now we can name a number. Call E of one by the small letter e.")
ck(option != script["03_7"], "the option differs from the slice")
s_old, l_old = scratch(script["03_7"])
s_new, l_new = scratch(option)
ck(len(s_old) == len(s_new) == 5, "still five sentences, so every until_sentence index stands")
print(f"    line length at scratch rate {l_old:.2f} s -> {l_new:.2f} s; sentence 3 opens {s_old[3]:.2f} s -> {s_new[3]:.2f} s")
for clause in ("reciprocals", "continuity"):
    ck(option.count(clause) == 1 and isinstance(new_rule(option, clause, s_new, l_new), float), f"{clause!r} still resolves")

print("--- F. how far a character share sits from word marks, on one other film's Oliver takes")
timings = json.load(open(GOLAY + "phrase-timings.json", encoding="utf-8"))
report = json.load(open(GOLAY + "report.json", encoding="utf-8"))
script_bytes = open("explainer_corpus/golay_echoes/script.tsv", "rb").read()
ck(hashlib.sha256(script_bytes).hexdigest() == timings["script_sha256"], "the golay script is the one the marks were made from")
print("    basis as the file states it:", timings["basis"])
units = open(GOLAY + "external-units.json", encoding="utf-8").read()
print("    unit ids naming Oliver in external-units.json:", len(re.findall(r'"id": "oliver_', units)))
texts = {}
for line in script_bytes.decode().splitlines():
    if "\t" in line:
        key, value = line.split("\t", 1)
        texts[key] = value
rate = report["meta"]["rate"]
errors, after_pause, skipped = [], [], 0
for cue, entry in sorted(timings["cues"].items()):
    text = texts.get(cue)
    if text is None or hashlib.sha256(text.encode()).hexdigest() != entry["text_sha256"]:
        skipped += 1
        continue
    t, starts = 0, []
    for sentence in report["lines"][cue]:
        starts.append(t / rate)
        t += sentence["out_samples"] + sentence["gap_after_samples"]
    length = t / rate
    opens = [0] + [i + 2 for i in range(len(text) - 1) if text[i] in ".?!" and text[i + 1] == " "]
    if len(opens) != len(starts):
        skipped += 1
        continue
    byte_text = text.encode()
    for mark in entry["marks"]:
        byte = mark["byte"]
        if byte in opens or byte == 0:
            continue
        j = max(i for i, at in enumerate(opens) if at <= byte)
        close = opens[j + 1] if j + 1 < len(opens) else len(byte_text)
        t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
        before = len(byte_text[opens[j] : byte].decode())
        whole = len(byte_text[opens[j] : close].decode().rstrip())
        error = t0 + (t1 - t0) * before / whole - mark["seconds"]
        errors.append(error)
        if byte_text[max(0, byte - 2) : byte] in (b", ", b": "):
            after_pause.append(error)


def spread(values):
    size = sorted(abs(v) for v in values)
    p = lambda q: size[min(len(size) - 1, int(q * len(size)))]
    return (f"n {len(size)}; absolute median {statistics.median(size):.2f} s, p90 {p(0.9):.2f}, p95 {p(0.95):.2f}, max {size[-1]:.2f}; "
            f"at or over 0.25 s {sum(v >= 0.25 for v in size) / len(size):.0%}, at or over 0.5 s {sum(v >= 0.5 for v in size) / len(size):.0%}; "
            f"signed mean {statistics.fmean(values):+.2f} s")


print(f"    cues used {len(timings['cues']) - skipped} of {len(timings['cues'])}")
print("    every word inside a sentence:", spread(errors))
print("    words right after a comma or colon:", spread(after_pause))

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
