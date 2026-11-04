#!/usr/bin/env python3
"""F1-F5 at the source: what the chapter does at each spoken phrase.

Run from the repository root:
  python3 -I -B <stage>/checks/source_check.py <crate dir>
It should pass on the candidate and fail on the base (the negative control).
This reads text. It proves no compile, render or motion.
"""

import re
import sys
from pathlib import Path

crate = Path(sys.argv[1])
src = (crate / "src/chapters/both_promises.rs").read_text()
code = src.partition("#[cfg(test)]")[0]
failures = []


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def between(a, b, text=code):
    i = text.find(a)
    j = text.find(b, i + 1) if i >= 0 else -1
    return text[i:j] if i >= 0 and j >= 0 else ""


PHRASES = [
    "Could a passed tile ever be left out",
    "Suppose one were",
    "Go back to the last moment",
    "a tile that ends later than the missing one",
    "or to the very start if it never did",
    "Every tile handed out since then",
    "and the missing tile too",
    "had not been entered at that moment",
    "or the rule would have picked a tile ending sooner",
    "And every one of them ends by now",
    "So all those tiles fit inside the stretch",
    "one more of them than seats added",
    "But each town's row fits at most",
    "and the fractions add up to one",
    "a stretch never holds more whole tiles",
    "no passed tile is ever left out",
    "never hands out a tile before the marker enters it",
    "so it keeps the quota promise at every house size",
]

proof = between('say(&kit, stage, "12_8")', "// 12_10 one promise")
found = re.findall(r's\.wait_for\(stage, "([^"]+)"\)', proof)
print("phrases, 12_8 and 12_9")
check(found == PHRASES, f"exactly these {len(PHRASES)} beats in this order (found {len(found)})")


def beat(i):
    """The source between phrase i and the next (or the section's end)."""
    a = f's.wait_for(stage, "{PHRASES[i]}")'
    b = f's.wait_for(stage, "{PHRASES[i + 1]}")' if i + 1 < len(PHRASES) else "// 12_10 one promise"
    return between(a, b)


print("F1: hypothetical against actual")
past = beat(0)
check("fade_out_all(stage, &past_now" in past and "run_tracker(stage, m, to as f64" in past,
      "at 'Could': fills past now fade and the marker returns to now")
sup = beat(1)
check("fade_out_all(stage, &since_then" in sup, "at 'Suppose': the fills handed out after then fade")
i_fade, i_gone = sup.find("since_then"), sup.find("let gone")
check(0 <= i_fade < i_gone, "before any red ring: no moment shows a run with a hole")
check("TILE_Y[missing.town]" in sup and "suppose: a passed tile not handed out" in sup, "'?' on the missing tile, and the red line")
check("tile(&tiles, proof.missing.town, proof.missing.k)" in proof, "the missing tile comes from the model, not a literal")
check("(CEDAR, 8)" not in proof and "CEDAR, 8" not in proof, "no trace of the old Cedar 8")
check("solid: real up to then, hollow: only supposed" in beat(2), "the legend names the two kinds of tile")
later = beat(3)
check("ring(stage, &later, Palette::ink())" in later and "stretch.later.end" in later and "stretch.missing.end" in later,
      "the later tile is ringed, with its end and the missing tile's, from facts")
check("history[from_h] != proof.then_seats" in proof and "history[to_h] != proof.now_seats" in proof,
      "a guard ties the model's proof to the run the chapter drew")
check("(f.0, f.1) != (t.town, t.k)" in proof, "a guard ties the fading fills to the seats handed out after then")

back = beat(15)
check(back.find("fade_out_all(stage, &supposed") >= 0 and back.find("fade_out_all(stage, &supposed") < back.find("let mut back"),
      "at 'no passed tile': the supposition goes first, then the run returns")
check("for t in &proof.real_after" in back and "flat(town_color(t.town), 0.75)" in back,
      "the run returns as fresh fills of the real seats, in the fills' one shade")
check("the real run at {to} seats: {}" in back, "and says so, from the history")
check('stretch.now.{key}' in back, "the restored seats are checked against the facts")
check("tile_rect(stage, t, flat(Palette::faint(), 0.45))" in beat(16) and "t.start >= to_r" in beat(16),
      "at 'never hands out': veils on the tiles not entered at now")

# every mobject the supposition created is gone once it fails
start = proof.find('s.wait_for(stage, "Suppose one were")')
end = proof.find('s.wait_for(stage, "no passed tile is ever left out")')
region = proof[start:end]
created = set()
for m in re.finditer(r"let (?:mut )?(\w+)(?:: [^=]+)? = (.*?);\n", region, re.S):
    name, rhs = m.group(1), m.group(2)
    if re.search(r"kit\.text|ring\(|stroke\(|chip\(|tile_rect\(|marker\(|arena_mut\(\)\.add", rhs):
        created.add(name)
for m in re.finditer(r"(\w+)\.push\((?:kit\.text|chip\(|ring\()", region):
    created.add(m.group(1))
goes = between("let supposed = [", "fade_out_all(stage, &supposed", proof)
earlier = " ".join(re.findall(r"fade_out_all\(stage, &\[([^\]]*)\]", region))
gone_later = {n for n in created if re.search(rf"\b{n}\b", goes)}
gone_early = {n for n in created if re.search(rf"\b{n}\b", earlier)}
left_over = created - gone_later - gone_early
check(not left_over and len(created) >= 15, f"{len(created)} supposed mobjects created, all faded (left: {sorted(left_over)})")
check("then_mark[0]" in goes, "the ink marker goes too")
check(not (gone_later & gone_early), f"and none is faded twice ({sorted(gone_later & gone_early)})")

print("F2: stroke-only quota boxes")
tail = code[code.find("// 12_10 one promise"):]
check("refill(" not in code, "no refill anywhere in the chapter (it filled the outlines)")
check("q.ceil(), Palette::green())?" in tail and "morph_all(stage, &green" in tail, "12_10 morphs into stroked green copies")
check(re.search(r"\brefill\b", code.split("impl SceneConstruct")[0]) is None, "and refill is no longer imported")

print("F3: rings match the labels")
fit = beat(10)
check("fade_in_lagged(stage, &fit" in fit and "t.town == missing.town && t.k == missing.k" in fit,
      "rings on the whole tiles inside; the missing one keeps its red ring")
check("fit.len() + 1 != fits" in fit, "a guard: hero rings plus the red one equal the count")
check("gone" in goes and "fit" in goes, "both kinds of ring leave together, so no moment shows 4 beside '= 5'")
check("flash_all(stage, &[band" not in proof, "the band no longer flashes at 1.25x (it hid what it framed)")

print("F4: dependencies at their phrases")
ne = beat(7)
check("before" in ne and "pulse_all(stage, &then_mark[..1]" in ne and "not entered at {from}" in ne,
      "'had not been entered': tiles started before then are veiled, then's line pulses, the wait is named")
so = beat(8)
check("pulse_all(stage, &[gone, later_ring]" in so and "later_veil" in so, "'ending sooner': the two tiles compared pulse")
now = beat(9)
check("pulse_all(stage, &mark[..1]" in now and "after" in now and "band" in now,
      "'ends by now': a beat now exists; tiles past now are veiled, now's line pulses, the band frames what is left")
check("fade_in_all(stage, &chips[seats_n..]" in beat(6), "'and the missing tile too': the eighth chip, red")
check("chips[seats_n..], None" in beat(11), "'one more of them': that chip pulses beside the count")
cannot = beat(14)
check("flash_all(stage, &[sum]" in cannot and "room" in cannot and "tiles cannot fit" in cannot,
      "'never holds more': a bracket gives room for seven, the eighth has none")
check(len(re.findall(r"\bflash_all\(", proof)) == 1, "the only 1.25x flash in the proof is on the sum, a text (F7 stays out of it)")
check("indicate(stage.arena_mut(), m, 1.0, color)" in code, "pulses recolor at scale 1.0: no tile grows")

print("F5: the prediction ring clears 2,000")
check("let q = think(&kit, stage, v(PREDICT_AT.0, PREDICT_AT.1))?;" in code, "12_1 calls think at its own spot, on one line for the plan's binding")
check("think_here" not in code, "think_here is gone from this chapter")

print("No change to what is spoken")
script_in_source = re.findall(r'say\(&kit, stage, "(12_\d+)"\)', code)
check(script_in_source == [f"12_{i}" for i in range(1, 11)], f"12_1 to 12_10 in order, once each ({len(script_in_source)})")

print()
if failures:
    print(f"FAILED {len(failures)}")
    sys.exit(1)
print("PASS (source text only: no compile, no render)")
