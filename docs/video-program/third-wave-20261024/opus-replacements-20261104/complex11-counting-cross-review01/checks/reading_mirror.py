"""The cue's timeline at several speech rates, and a mirror of mod.rs's two reading-window tests.

The timeline models Spoken's fallback: sentence starts share the cue by characters, and a phrase
shares its sentence by characters. It follows the staged order of 11_7's calls. It is an estimate
with no take behind it: real sentence lengths and word times come only from an aligned Oliver take.
Its control is WhiteSwan's own table, whose sentence starts it is given.

Usage: python3 -I -B reading_mirror.py SCRIPT_TSV COUNTING_RS CORRECTED_RS MOD_RS OLIVER_CHARS_PER_S OLIVER_MIN OLIVER_MAX
"""

import re
import sys

script_path, counting_path, corrected_path, mod_path = sys.argv[1:5]
OLIVER, OLIVER_MIN, OLIVER_MAX = (float(a) for a in sys.argv[5:8])
script = dict(row.rstrip("\n").split("\t", 1) for row in open(script_path, encoding="utf8") if "\t" in row)
said = script["11_7"]
counting = open(counting_path, encoding="utf8").read()
corrected = open(corrected_path, encoding="utf8").read()
mod = open(mod_path, encoding="utf8").read()
failed = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failed.append(name)


# ------------------------------------------------------------------ the sentences and phrases
offsets = [0] + [m.end() for m in re.finditer(r"[.?!] ", said)]
sentences = [said[a:b].strip() for a, b in zip(offsets, offsets[1:] + [len(said)])]
words = len(said.split())
print(f"11_7 has {len(sentences)} sentences, {len(said)} characters, {words} words")
check("the cue stays under the request limit of 100 words", words <= 100, str(words))
for k, opening in ((1, "Divide"), (2, "Now it starts"), (3, "On a huge circle"), (4, "Picture"), (6, "So there are")):
    check(f"sentence {k} opens with {opening!r}", sentences[k].startswith(opening))
for k, phrase in ((0, "complex polynomial"), (1, "which leaves")):
    check(f"{phrase!r} is in sentence {k} and nowhere else", said.count(phrase) == 1 and phrase in sentences[k])
keys = re.findall(r'line\.(?:wait_for|until)\(stage, "([^"]+)"', counting[counting.index('say(&kit, stage, "11_7")') : counting.index('say(&kit, stage, "11_8")')])
print("phrase keys in the cue:", keys)
check("the cue uses only those two phrase keys", sorted(keys) == ["complex polynomial", "which leaves"])


def timeline(starts, on_words=False):
    """Seconds from the cue's start. `starts` are sentence starts. `on_words` is C2's key."""
    def phrase(text):
        byte = said.index(text)
        j = max(k for k, o in enumerate(offsets) if o <= byte)
        return starts[j] + (starts[j + 1] - starts[j]) * (byte - offsets[j]) / (offsets[j + 1] - offsets[j])

    now = 0.3 + 0.4 + 0.4  # gone, the painter's fade, the claim
    now = max(now, phrase("complex polynomial"))
    now += 0.3  # the row f(z)
    before_in = now
    out_rt = max(0.7, min(1.4, starts[1] - now))
    now += out_rt
    turned = now
    now = max(now, starts[1])
    hold = now - turned
    now += 0.3  # every output
    every_in = now
    back_rt = max(phrase("which leaves") - now, 1.2)
    now += back_rt
    back_end = now
    now += 0.2  # the old rows go
    now += 0.3  # the results come
    row_in = now
    now = max(now, phrase("with z") if on_words else starts[2])
    row_clean = now - row_in
    now += 0.3 + 0.4  # the row goes, degree_n comes
    words_at = phrase("z to the n")
    return {
        "out": out_rt,
        "hold on f": hold,
        "back": back_rt,
        "f row clean": back_end - before_in,
        "every clean": back_end - every_in,
        "result row clean": row_clean,
        "z^n in, against 'z to the n'": now - words_at,
        "results + card until the wide plane": starts[3] - row_in,
    }


def starts_at(rate):
    return [o / rate for o in offsets] + [len(said) / rate]


print("\nthe control: WhiteSwan's table gives these sentence starts and turn times")
for name, given, want in (
    ("old scratch rate", [0.0, 5.9, 11.4, 16.1], (1.40, 2.47)),
    ("15% faster", [0.0, 5.0, 9.7, 13.7], (1.40, 2.05)),
    ("30% faster", [0.0, 4.2, 8.0, 11.3], (1.36, 1.64)),
):
    row = timeline(given + [given[-1] + 30.0])
    agree = abs(row["out"] - want[0]) < 0.03 and abs(row["back"] - want[1]) < 0.06
    check(f"{name}: turn out {row['out']:.2f} / back {row['back']:.2f}, theirs {want[0]} / {want[1]}", agree)

print("\nat rates, staged order, then with C2. Seconds; rates are characters a second:")
order = ["out", "hold on f", "back", "f row clean", "every clean", "result row clean", "z^n in, against 'z to the n'", "results + card until the wide plane"]
print("rate\tvariant\t" + "\t".join(order))
rows = {}
for label, rate in (("scratch 12.7", 12.7), (f"Oliver slowest cue {OLIVER_MIN:.2f}", OLIVER_MIN), (f"Oliver overall {OLIVER:.2f}", OLIVER), ("15 flat", 15.0), (f"Oliver fastest cue {OLIVER_MAX:.2f}", OLIVER_MAX), ("16.5 (30% over scratch)", 16.5)):
    for variant, on_words in (("staged", False), ("C2", True)):
        row = timeline(starts_at(rate), on_words)
        rows[(label, variant)] = row
        print(f"{label}\t{variant}\t" + "\t".join(f"{row[k]:.2f}" for k in order))

printed = "p(z) = f(z) ÷ 2i = z⁵ − 3z + 1"
need = len(printed) / 15.0
print(f"\nthe result row prints {len(printed)} characters: {need:.2f} s at the program's 15 a second, as a lower bound")
overall = rows[(f"Oliver overall {OLIVER:.2f}", "staged")]["result row clean"]
fastest = rows[(f"Oliver fastest cue {OLIVER_MAX:.2f}", "staged")]["result row clean"]
print(f"staged: {overall:.2f} s at Oliver's overall rate on another film, {fastest:.2f} s at that film's fastest cue")
check("with C2 the row reaches its bound at every rate above", all(r["result row clean"] >= need - 1e-9 for (lab, v), r in rows.items() if v == "C2"))
check("'with z' is in sentence 2 and nowhere else", said.count("with z") == 1 and "with z" in sentences[2])
check("the first 'z to the n' is sentence 2's", offsets[2] < said.index("z to the n") < offsets[3])
cue_of = lambda text: text[text.index('say(&kit, stage, "11_7")') : text.index('say(&kit, stage, "11_8")')]  # noqa: E731
check("C2 waits on the words and no longer on sentence 2's start", 'line.wait_for(stage, "with z")?;' in cue_of(corrected) and "line.wait_sentence(stage, 2)?;" not in cue_of(corrected))

# ------------------------------------------------------------------ a mirror of the two mod.rs tests
BREATH, COMES_IN, RULE = 0.6, 0.4, 15.0
windows = dict(re.findall(r'\("(\d\d_\d+)", ([\d.]+)\)', mod[mod.index("pub const READING_WINDOWS") : mod.index("/// Glyph supplements")]))
squashed = re.search(r'pub const SQUASHED: &str = "([^"]+)";', counting).group(1)
floor, ceiling = len(squashed) / RULE, 2 * len(squashed) / RULE
clean = BREATH - COMES_IN + float(windows["11_6"])
print(f"\n11_6's card: {len(squashed)} characters, {clean:.1f} s clean, floor {floor:.2f}, ceiling {ceiling:.2f}")
check("reading_windows_leave_time_for_what_the_voice_leaves_out, mirrored, 11_6", floor <= clean <= ceiling)


def card_test(source, cue, next_cue, card):
    """each_card_outlasts_its_reading_window, for a cue with a next cue in the chapter."""
    window, any_say = "reading_window(&kit, stage, &line)?;", "let line = say(&kit, stage, "
    here = source.find(f'{any_say}"{cue}")?;')
    shown = here + source[here:].find(f"let {card} = ")
    held = here + source[here:].find(window)
    if not shown < held:
        return f"{cue}: out of order"
    if "wait_until(stage, line.end())?;" not in source[here:shown]:
        return f"{cue}: comes under the voice"
    before = source[shown:held]
    if "fade_out" in before or "remove_from_scene" in before:
        return f"{cue}: cleared early"
    then = here + source[here:].find(f'{any_say}"{next_cue}")?;')
    if source[here + 1 : then].count(any_say):
        return f"{cue}: {next_cue} is not next"
    if source[held + len(window) : then].strip():
        return f"{cue}: {next_cue} waits on more"
    rest = source[then + 1 :]
    cleared_by = rest[: rest.find(any_say)]
    if "fade_out_all" not in cleared_by or not re.search(rf"(?<![\w]){card}(?![\w])", cleared_by):
        return f"{cue}: {next_cue} does not clear {card}"
    return None


for name, source in (("staged", counting), ("corrected", corrected)):
    problem = card_test(source, "11_6", "11_7", "squashed")
    check(f"each_card_outlasts_its_reading_window, mirrored, 11_6, {name}", problem is None, problem or "")
planted = counting.replace('reading_window(&kit, stage, &line)?;\n        let line = say(&kit, stage, "11_7")?;', 'reading_window(&kit, stage, &line)?;\n        pause(stage, 1.0)?;\n        let line = say(&kit, stage, "11_7")?;')
check("planted, caught: something between the window and the next cue", planted != counting and card_test(planted, "11_6", "11_7", "squashed") is not None)
planted = counting.replace("gone.extend([reference, follows, leash, squashed, unit]);", "gone.extend([reference, follows, leash, unit]);")
check("planted, caught: the card is never cleared", planted != counting and card_test(planted, "11_6", "11_7", "squashed") is not None)

for row in ("EVERY_OUTPUT", "ZERO_KEPT", "SAME_ROOTS", "BEFORE_DIVIDING_TEX", "AFTER_DIVIDING_TEX"):
    check(f"counting::{row} is registered once", mod.count(f"counting::{row},") == 1)
listed = re.findall(r"pub const (?:TEXT|TEX|MONO): &\[&str\] = &\[(.*?)\n\];", mod, re.S)
entries = [e.strip() for block in listed for e in block.split("\n") if e.strip().startswith(("counting::", '"'))]
check("no_supplement_is_empty_or_duplicated, mirrored on the rows' names", len(entries) == len(set(entries)), f"{len(entries)} rows")

print(f"\n{'all passed' if not failed else 'FAILED: ' + '; '.join(failed)}")
sys.exit(1 if failed else 0)
