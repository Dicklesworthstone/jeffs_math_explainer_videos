"""Source check of the 11_7 and 12_6 splits (Mail 1541): exact words, order, and every hook.

A text scan only: it reads the script and the two chapter sources. It says nothing about
compiling, timing, pictures or listening. Run from the repo root:

    python3 -I -B <stage>/checks/split_check.py
"""
import re
import sys
from pathlib import Path

STAGE = Path(sys.argv[0]).resolve().parent.parent
REL = "explainer_corpus/logarithm_mysteries"
ROOT = Path.cwd() / REL
NEW = STAGE / REL
OLD = STAGE / "before-splits" / REL


def read(path):
    return dict(line.split("\t", 1) for line in path.read_text(encoding="utf-8").splitlines() if line and not line.startswith("#"))


def sentences(text):
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


def scene(path):
    return path.read_text(encoding="utf-8").split("#[cfg(test)]")[0]


old_script, new_script = read(ROOT / "script.tsv"), read(NEW / "script.tsv")
assert " ".join(old_script.values()) == " ".join(new_script.values())
expected = []
for cue in old_script:
    expected += {"11_7": ["11_7", "11_20"], "12_6": ["12_6", "12_20", "12_21"]}.get(cue, [cue])
assert list(new_script) == expected
print(f"script: {len(old_script)} -> {len(new_script)} cues; every word identical, same order")
counts = {c: len(new_script[c].split()) for c in ("11_7", "11_20", "12_6", "12_20", "12_21")}
print("words per new line:", counts)
assert counts == {"11_7": 91, "11_20": 32, "12_6": 39, "12_20": 61, "12_21": 50}
assert max(len(t.split()) for c, t in new_script.items() if c[:2] in ("11", "12")) <= 100

# old (cue, sentence) -> new (cue, sentence)
MAP = {}
for old, parts in (("11_7", ("11_7", "11_20")), ("12_6", ("12_6", "12_20", "12_21"))):
    n = 0
    for part in parts:
        for j, sentence in enumerate(sentences(new_script[part])):
            assert sentences(old_script[old])[n] == sentence
            MAP[(old, n)] = (part, j)
            n += 1
    assert n == len(sentences(old_script[old]))
print(f"sentences: {len(MAP)} mapped, each the same exact words")

PATTERNS = [
    (r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', "until_sentence"),
    (r'sentence_time\(&kit, "(\d\d_\d+)", (\d+)\)', "sentence_time"),
    (r'n\.sentence_at\("(\d\d_\d+)", (\d+)\)', "sentence_at"),
    (r'sentence_len\(&kit\.sentence_starts\("(\d\d_\d+)"\), (\d+)\)', "sentence_len"),
    (r'sentence_starts\("(\d\d_\d+)"\), (\d+), [\d.]+, [\d.]+, [\d.]+\)', "room"),
    (r'until_share\(&kit, stage, "(\d\d_\d+)", (\d+),', "until_share"),
]
CONSTS = re.compile(r'^const ([A-Z_]+): &str = "([^"]+)";', re.M)
for name in ("unreachable.rs", "what_it_forgets.rs"):
    old_src, new_src = scene(OLD / "src/chapters" / name), scene(NEW / "src/chapters" / name)
    consts = dict(CONSTS.findall(new_src))
    print(name)
    for pattern, kind in PATTERNS:
        olds = [(c, int(j)) for c, j in re.findall(pattern, old_src)]
        news = [(c, int(j)) for c, j in re.findall(pattern, new_src)]
        survivors = []
        for hook in olds:
            target = MAP.get(hook, hook)
            # a hook at a new line's first sentence becomes that line's kit.say, and a hook
            # on the sentence after a line's end becomes the line's end
            if kind == "until_sentence" and target[1] == 0 and hook in MAP and target[0] != hook[0]:
                assert f'kit.say(stage, "{target[0]}")' in new_src, hook
                print(f"  {kind} {hook} -> kit.say {target[0]}")
                continue
            if kind in ("sentence_time", "sentence_at") and hook in MAP and target[1] == 0 and target[0] != hook[0]:
                prev = hook[0]
                assert f'line_end(&kit, "{prev}")' in new_src, hook
                print(f"  {kind} {hook} -> line_end {prev} (the sentence after it is now another line)")
                continue
            survivors.append(target)
            if target != hook:
                print(f"  {kind} {hook} -> {target}")
        assert sorted(survivors) == sorted(news), (kind, survivors, news)
    # every phrase hook names a line that has the phrase, once
    hooks = re.findall(r'(?:until_phrase\(&kit, stage, |phrase_time\(&kit, )"(\d\d_\d+)", ([A-Z_]+)\)', new_src)
    for cue, const in hooks:
        assert new_script[cue].count(consts[const]) == 1, (cue, const)
    print(f"  {len(hooks)} phrase hooks, each once in its line")
    says = re.findall(r'kit\.say\(stage, "(\d\d_\d+)"\)', new_src)
    chapter = says[0][:2]
    assert says == [c for c in new_script if c[:2] == chapter], says
    print(f"  kit.say order matches the script: {len(says)} lines")
    for cue in ("11_7", "12_6"):
        if cue[:2] == chapter:
            left = [j for _, j in re.findall(rf'"({cue})", (\d+)', new_src)]
            assert all(int(j) < len(sentences(new_script[cue])) for j in left), left

print("phrase bytes for phrase-timings, in the split script:")
for name in ("unreachable.rs", "what_it_forgets.rs"):
    new_src = scene(NEW / "src/chapters" / name)
    consts = dict(CONSTS.findall(new_src))
    seen = {}
    for cue, const in re.findall(r'(?:until_phrase\(&kit, stage, |phrase_time\(&kit, )"(\d\d_\d+)", ([A-Z_]+)\)', new_src):
        seen.setdefault(cue, []).append(const)
    for cue, list_ in seen.items():
        unique = list(dict.fromkeys(list_))
        print(
            f"  {cue}: "
            + ", ".join(f"{new_script[cue].encode().find(consts[c].encode())} {consts[c]!r}" for c in unique)
        )
print("PASS")
