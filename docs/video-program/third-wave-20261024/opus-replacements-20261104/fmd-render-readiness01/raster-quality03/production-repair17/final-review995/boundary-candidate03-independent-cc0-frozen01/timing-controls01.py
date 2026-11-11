"""timing-controls01.py: two positive controls for CC1's timing-skeleton01.py on candidate03.

Writes NEW variants (mode 'x'), each with one change in the film part only (the text before
"\\n#[cfg(test)]\\nmod tests {"; candidate03's tests quote some of these lines):
  T1  the shrink's bar fade, which now also carries ca_out, rho_out and the leader: 0.3 -> 0.35
  T2  one 09_1 cue phrase: "Only now do we shrink the small circle" -> "Only now we shrink the small circle"

usage: python3 -I -B timing-controls01.py CANDIDATE03_RS OUT_DIR
"""
import os
import sys

src = open(sys.argv[1], encoding="utf-8").read()
film, mark, tests = src.partition("\n#[cfg(test)]\nmod tests {")
assert mark
EDITS = {
    "T1-bar-fade-035": ("fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;",
                        "fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.35)?;"),
    "T2-cue-phrase": ('line.wait_for(stage, "Only now do we shrink the small circle")?;',
                      'line.wait_for(stage, "Only now we shrink the small circle")?;'),
}
for name, (old, new) in EDITS.items():
    assert film.count(old) == 1, name
    path = os.path.join(sys.argv[2], name + "-boundary.rs")
    with open(path, "x", encoding="utf-8", newline="") as f:
        f.write(film.replace(old, new) + mark + tests)
    print("wrote", os.path.basename(path))
