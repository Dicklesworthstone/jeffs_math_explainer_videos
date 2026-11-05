"""The focused Rust tests' source conditions, run in Python on the baseline, the staged files and plants.

    python3 -I -B source_conditions.py <baseline-chapters> <staged-chapters> <script.tsv>

The Rust tests are the real gates and only root can run them. This runs their text conditions without
Cargo, so that a typo in an order list shows now. Each row must pass on the staged bytes and fail on
the baseline, and each plant must make a passing row fail. Standard library.
"""
import pathlib
import re
import sys

BASE, STAGED, SCRIPT = (pathlib.Path(a) for a in sys.argv[1:4])
script = dict(line.rstrip("\n").split("\t", 1) for line in SCRIPT.read_text(encoding="utf-8").splitlines() if "\t" in line and not line.startswith("#"))
failures = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failures.append(name)


def film(text):
    return text.split("#[cfg(test)]\nmod tests")[0]


def in_order(text, marks):
    at, last = [], -1
    for mark in marks:
        where = text.find(mark)
        if where < 0 or where <= last:
            return False, mark
        at.append(where)
        last = where
    return True, ""


def phrase_rule(text, cue, phrases, open_with, close_with):
    words = script[cue]
    open_at = words.find(open_with) if open_with else words.rfind(". ")
    close_at = words.find(close_with) if close_with else len(words)
    after = open_at
    for phrase in phrases:
        byte = words.find(phrase)
        if words.count(phrase) != 1 or byte <= after or byte >= close_at or words[byte - 1] != " ":
            return False
        if f'const {const_of(text, phrase)}: &str = "{phrase}";' not in text:
            return False
        after = byte
    return True


def const_of(text, phrase):
    found = re.search(r'const ([A-Z_]+): &str = "' + re.escape(phrase) + '";', text)
    return found.group(1) if found else "NONE"


RULES = {
    "C1 complex_turns.rs": ("complex_turns.rs", [
        'kit.until_sentence(stage, "05_20", 4)',
        "fade_out_all(stage, &[lengths, angles, times_i], 0.4)",
        'until_phrase(&kit, stage, "05_20", TAKE_X_PHRASE)',
        "fade_in_all(stage, &[example_x], 0.4)",
        'until_phrase(&kit, stage, "05_20", TAKE_Y_PHRASE)',
        "swap(stage, &[example_x], &[example_xy], 0.3)",
        'until_phrase(&kit, stage, "05_20", LENGTH_PHRASE)',
        "fade_in_all(stage, &[example_word], 0.4)",
        'until_phrase(&kit, stage, "05_20", LAND_PHRASE)',
        "swap(stage, &[example_xy], &[example], 0.3)",
        "fade_in_all(stage, &[two_i_arrow, two_i_word], 0.5)",
        'kit.say(stage, "05_3")',
    ], "05_20", ["the logarithm of two", "a half pi", "the length is two", "land on two i"], None, None),
    "L1 lifting_loops.rs": ("lifting_loops.rs", [
        "tween(turn, 0.45, 2.6)",
        'kit.until_sentence(stage, "06_6", 3)',
        'until_phrase(&kit, stage, "06_6", PASSES_PHRASE)',
        "fade_in_all(stage, &[minus_one], 0.3)",
        "tween(turn, 0.6, 1.6)",
        'until_phrase(&kit, stage, "06_6", LEAPS_PHRASE)',
        "jump_inset(&kit, stage, &jump_words()?)",
        "fade_in_all(stage, &drop, 0.4)",
        "tween(turn, 0.97, 1.2)",
        'kit.until_sentence(stage, "06_6", 4)',
    ], "06_6", ["passes minus one", "leaps from pi to minus pi"], "The gap sits", "So that candidate"),
    "R10 product_rule.rs": ("product_rule.rs", [
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
    ], "08_5", ["leaves a half turn", "a factor of minus one"], "Halving the lost lap", "The root of a product"),
}

print("== source order and phrases, as the Rust tests state them")
for label, (name, marks, cue, phrases, open_with, close_with) in RULES.items():
    staged = (STAGED / name).read_text(encoding="utf-8")
    base = (BASE / name).read_text(encoding="utf-8")
    ok, missing = in_order(film(staged), marks)
    check(f"{label}: the staged film's order", ok, missing)
    check(f"{label}: its phrases, once each, each starting a word, in spoken order, in the right sentence", phrase_rule(staged, cue, phrases, open_with, close_with))
    bad, _ = in_order(film(base), marks)
    check(f"{label}: positive control, the baseline fails the same order", not bad)

print("\n== the other conditions")
c = (STAGED / "complex_turns.rs").read_text(encoding="utf-8")
check("C1: a swapped-out stage is never in a later fade list", "example_x," not in film(c) and "example_xy," not in film(c))
check("C1: the whole line is the one faded with the old list", "m1_dot, example, example_word, two_i_arrow, two_i_word" in film(c))
check("C1: the stages hold no answer", re.search(r'example_part\(&kit, stage, &format!\(r"[^"]*="', film(c)) is None)
check("C1: the whole line's string is unchanged", r'&format!(r"e^{{\ln {len} + i\pi/2}} = {len}i"),' in film(c))
check("C2: the clamp is gone and the arrow's literal is the constant", "y.min(IN_Y))" not in film(c) and "strip_top[1] + GOES_ON_TIP" in film(c) and "strip_top[1] + 0.48" not in film(c))
check("C2: no comment says the dot waits", "dot waits" not in c.replace("no dot waits", ""))
b = (STAGED / "branch_cut.rs").read_text(encoding="utf-8")
check("B8: the old constants are gone, and the stroke is one constant", "SPIRAL_K" not in b and "SPIRAL_START" not in b and "spiral_cut(), red, SPIRAL_W" in film(b))
p = (STAGED / "product_rule.rs").read_text(encoding="utf-8")
check("R10: the whole line's string is unchanged, and the first stage is its start", r'const HALVED: &str = r"\tfrac{1}{2}(2\pi i) = \pi i, \quad e^{\pi i} = -1";' in p and r'const HALVED_LAP: &str = r"\tfrac{1}{2}(2\pi i) = \pi i";' in p)
base_p = (BASE / "product_rule.rs").read_text(encoding="utf-8")
check("R10: its placement is unchanged", "Z.p(C::new(0.0, -1.55))" in film(p) and "Z.p(C::new(0.0, -1.55))" in film(base_p))
check("R10: every other row of the explanation stays", all(s in film(p) for s in [
    r"\sqrt{z} = e^{\frac{1}{2}\operatorname{Log} z}",
    r"\sqrt{{-1}}\,\sqrt{{-1}} = e^{{i\pi/2}} e^{{i\pi/2}} = e^{{i\pi}} = {}",
    r"\sqrt{{1}} = e^{{\frac{{1}}{{2}} \operatorname{{Log}} 1}} = e^{{0}} = {}",
    r"\operatorname{Log} 1 = 0,\ \text{not } 2\pi i",
    r"\sqrt{zw} = \pm \sqrt{z}\,\sqrt{w}",
]))
for name in RULES.values():
    text = (STAGED / name[0]).read_text(encoding="utf-8")
    check(f"{name[0]}: one phrase_marks use, one wait_frames call, one source of the script", film(text).count("phrase_marks(") == 1 and film(text).count("wait_frames(") == 1 and film(text).count("include_str!") == 1)

print("\n== plants: a defect in the staged text must fail a row")
PLANTS = [
    ("C1: the answer before its phrase", "complex_turns.rs", "swap(stage, &[example_xy], &[example], 0.3)?;\n", "", 'until_phrase(&kit, stage, "05_20", LENGTH_PHRASE)?;\n', "swap(stage, &[example_xy], &[example], 0.3)?;\n"),
    ("L1: the inset before its phrase", "lifting_loops.rs", 'until_phrase(&kit, stage, "06_6", LEAPS_PHRASE)?;\n', "", "fade_in_all(stage, &drop, 0.4)?;\n", 'until_phrase(&kit, stage, "06_6", LEAPS_PHRASE)?;\n'),
    ("R10: the sign before its phrase", "product_rule.rs", 'until_phrase(&kit, stage, "08_5", FACTOR_PHRASE)?;\n', "", "swap(stage, &[halving], &[halved], 0.3)?;\n", 'until_phrase(&kit, stage, "08_5", FACTOR_PHRASE)?;\n'),
]
for label, name, cut, repl, after, add in PLANTS:
    text = (STAGED / name).read_text(encoding="utf-8")
    assert text.count(cut) >= 1 and text.count(after) >= 1, label
    planted = text.replace(cut, repl, 1).replace(after, after + add, 1)
    rule = next(v for v in RULES.values() if v[0] == name)
    ok, missing = in_order(film(planted), rule[1])
    check(f"plant {label} is caught", not ok, missing)
planted = c.replace('LAND_PHRASE: &str = "land on two i"', 'LAND_PHRASE: &str = "the angle"')
rule = RULES["C1 complex_turns.rs"]
check("plant a phrase that is not in the sentence is caught", not phrase_rule(planted, "05_20", ["the angle"], None, None) or script["05_20"].count("the angle") != 1)

print("\nFAILURES: " + (", ".join(failures) if failures else "none"))
sys.exit(1 if failures else 0)
