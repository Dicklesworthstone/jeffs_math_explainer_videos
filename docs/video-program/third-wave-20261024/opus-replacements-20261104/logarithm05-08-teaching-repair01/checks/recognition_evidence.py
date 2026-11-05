"""What root's guarded recognition run says about the 05_20 hooks, on both retained takes.

    python3 -I -B recognition_evidence.py <command.log> <takes.json> <script.tsv> <staged-complex_turns.rs>

Reads the four files, writes nothing. It takes the word rows of both 05_20 takes and compares them with
the staged hooks, under the two timings the film can run on:

  marks     a word mark at each phrase's byte (root's aligner, later); the rows stand in for those marks
  fallback  no mark: the phrase's character share of its measured sentence, as phrase_seconds computes it

The source holds no take time: this reads the times from the log and the phrases from the Rust. Nothing
here selects a take. The rows are one model's provisional word timing, and nobody has heard these takes
for this check, so they support a design and accept nothing. The sentence's start is taken from its
first word's row, which stands in for the narrator's measured start. Standard library.
"""
import hashlib
import json
import pathlib
import re
import sys

LOG, TAKES, SCRIPT, SOURCE = (pathlib.Path(a) for a in sys.argv[1:5])
LOG_SHA = "8d20c0079989fd2d652a9e39c49301d6eb9092d9c959669f7696c1aa60557333"
TAKES_SHA = "28983ebf376f2b269c5d54be51aac164616dc3d76cae478216c9a4e394c99091"
CUE = "05_20"
failures = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failures.append(name)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


check("the log is the bytes root named", sha(LOG) == LOG_SHA, LOG_SHA[:16])
check("the take list is the bytes root named", sha(TAKES) == TAKES_SHA, TAKES_SHA[:16])

takes = [row for row in json.loads(TAKES.read_text(encoding="utf-8"))["takes"] if row["cue"] == CUE]
names = [row["wav"].rsplit("/", 1)[-1] for row in takes]
script = dict(line.rstrip("\n").split("\t", 1) for line in SCRIPT.read_text(encoding="utf-8").splitlines() if "\t" in line and not line.startswith("#"))
text = script[CUE]
check("two takes are bound to this cue, on the script's words", len(takes) == 2 and all(row["text"] == text for row in takes), ", ".join(names))
source = SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]\nmod tests")[0]
consts = dict(re.findall(r'const ([A-Z_]+): &str =\s*"([^"\\]*)";', source))
HOOKS = re.findall(r'until_phrase\(&kit, stage, "05_20", ([A-Z_]+)\)', source)
phrases = [consts[name] for name in HOOKS]
check("the film's hooks are read from its source", HOOKS == ["TAKE_X_PHRASE", "TAKE_Y_PHRASE", "LENGTH_PHRASE", "LAND_PHRASE"], ", ".join(phrases))
check("the film holds none of the recognized times", not re.search(r"\b(23\.1|22\.92|23\.7|23\.56|24\.2|24\.08)", source))

rows = {}
for line in LOG.read_text(encoding="utf-8").splitlines():
    if not line.startswith("{"):
        continue
    record = json.loads(line)
    name = record.get("input", "").rsplit("/", 1)[-1]
    if name in names:
        check(f"{name}: status ok", record["status"] == "ok")
        rows[name] = record["report"]["result"]["segments"]
check("both takes have word rows", sorted(rows) == sorted(names))


def phrase_share(phrase, t0, t1):
    byte = text.find(phrase)
    opens = [0] + [i + 2 for i in range(len(text) - 1) if text[i] in ".?!" and text[i + 1] == " "]
    j = max(i for i, o in enumerate(opens) if o <= byte)
    close = opens[j + 1] if j + 1 < len(opens) else len(text)
    return t0 + (t1 - t0) * (byte - opens[j]) / len(text[opens[j]:close].rstrip())


def word_rows(segments):
    # the script's words and the rows run side by side up to "two i", which this model writes "2i"
    return [re.sub(r"[^a-z0-9]", "", s["text"].lower()) for s in segments]


SPOKEN ={"the logarithm of two": "the", "a half pi": "a", "the length is two": "the", "land on two i": "land"}
WORD_INDEX = {phrase: len(text[: text.find(phrase)].split()) for phrase in phrases}
# (what the film does before the next hook, seconds): the schedule in complex_turns.rs after sentence 4
PLAYS = {"the logarithm of two": [0.4], "a half pi": [0.3], "the length is two": [0.4], "land on two i": [0.3, 0.5]}
FIRST_FADE = 0.4

for name in names:
    segments = rows[name]
    plain = word_rows(segments)
    sentence =len(text[: text.find("For instance")].split())
    check(f"{name}: the sentence's first word lines up", plain[sentence] == "for")
    voice_end = segments[-1]["end_sec"]
    t0 = segments[sentence]["start_sec"]
    print(f"\n{name}: sentence starts {t0:.2f}, voice ends {voice_end:.2f}")
    print("  hook                   word row  fallback  fallback minus row")
    marks, shares = {}, {}
    for phrase in phrases:
        at = WORD_INDEX[phrase]
        check(f"{name}: {phrase!r} lines up with its word row", plain[at] == SPOKEN[phrase], plain[at])
        marks[phrase] = segments[at]["start_sec"]
        shares[phrase] = phrase_share(phrase, t0, voice_end)
        print(f"  {phrase:22} {marks[phrase]:8.2f}  {shares[phrase]:8.2f}  {shares[phrase] - marks[phrase]:+.2f}")
    named = plain.index("2i")
    print(f"  the word row for \"two i\", which this model writes 2i: {segments[named]['start_sec']:.2f} to {segments[named]['end_sec']:.2f}")
    for label, times in (("marks", marks), ("fallback", shares)):
        now = t0 + FIRST_FADE
        print(f"  {label}: settled pictures")
        for phrase in phrases:
            idle = times[phrase] - now
            start = max(now, times[phrase])
            now = start + sum(PLAYS[phrase])
            print(f"    {phrase:22} starts {start:6.2f}  settled {now:6.2f}  the picture before it settled {idle:.2f} s ahead of this hook")
            check(f"{name}, {label}: the picture before {phrase!r} has settled by then", idle >= 0.0)
        check(f"{name}, {label}: the last picture settles inside the voice", now < voice_end, f"{voice_end - now:.2f} s left")
        if label == "marks":
            swap_settled = times["land on two i"] + PLAYS["land on two i"][0]
            lead = segments[named]["start_sec"] - swap_settled
            print(f"    the answer has settled {lead:+.2f} s before the row for \"two i\" starts, inside \"land on two i\"")
            check(f"{name}: the answer leads its row by under half a second, and starts on \"land\"", lead < 0.5)

    # the two hooks not taken: the first staging's "so we land on two i", and "two i" itself
    so_row = segments[WORD_INDEX["land on two i"] - 2]["start_sec"]
    check(f"{name}: the row two words before \"land\" is \"so\"", plain[WORD_INDEX["land on two i"] - 2] == "so")
    early = so_row + PLAYS["land on two i"][0]
    print(f"  on \"so we land on two i\" the answer would settle at {early:.2f}: {marks['land on two i'] - early:.2f} s before \"land\", {segments[named]['start_sec'] - early:.2f} s before \"two i\"")
    check(f"{name}: hooked on \"so we land\", the answer would settle before \"land\" is said", early < marks["land on two i"])
    late = phrase_share("two i", t0, voice_end) + sum(PLAYS["land on two i"])
    print(f"  hooked on \"two i\" with the fallback, the last picture would settle at {late:.2f}, voice ends {voice_end:.2f}")
    check(f"{name}: hooked on \"two i\", the fallback's last picture would outlast the voice", late > voice_end)

print("\nFAILURES: " + (", ".join(failures) if failures else "none"))
sys.exit(1 if failures else 0)
