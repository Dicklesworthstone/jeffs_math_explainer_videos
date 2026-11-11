"""Scratch only: write root's proposed chapter-05 label correction as new files beside nothing in production.
Reads publication976/description-timeline982.txt and chapter-times982.json, replaces exactly one occurrence each
(the label line; the label and description_sha256 fields, as raw text so formatting is unchanged), creates the two
outputs exclusively in OUT, and prints their sha256 and a unified diff.
usage: python3 -I -B propose-ch05-label01.py PUBLICATION976 OUT"""
import difflib
import hashlib
import sys
from pathlib import Path

U, OUT = Path(sys.argv[1]), Path(sys.argv[2])
OLD, NEW = 'Why the whole series converges', 'Why the stepped approximations reach E'
sha = lambda b: hashlib.sha256(b).hexdigest()

d_raw = (U / 'description-timeline982.txt').read_bytes()
d_old = d_raw.decode()
assert d_old.count(f'9:31 {OLD}\n') == 1 and d_old.count(OLD) == 1
d_new = d_old.replace(f'9:31 {OLD}\n', f'9:31 {NEW}\n')

c_raw = (U / 'chapter-times982.json').read_bytes()
c_old = c_raw.decode()
old_dsha, new_dsha = sha(d_raw), sha(d_new.encode())
assert c_old.count(f'"label": "{OLD}"') == 1 and c_old.count(old_dsha) == 1, 'chapter-times does not bind the current description once'
c_new = c_old.replace(f'"label": "{OLD}"', f'"label": "{NEW}"').replace(old_dsha, new_dsha)

OUT.mkdir()
for name, text in [('description-timeline982-ch05-proposed01.txt', d_new), ('chapter-times982-ch05-proposed01.json', c_new)]:
    with (OUT / name).open('x', encoding='utf-8') as f:
        f.write(text)
print('description old', old_dsha, 'new', sha(d_new.encode()))
print('chapter-times old', sha(c_raw), 'new', sha(c_new.encode()))
for a, b, n in [(d_old, d_new, 'description-timeline982'), (c_old, c_new, 'chapter-times982')]:
    sys.stdout.writelines(difflib.unified_diff(a.splitlines(True), b.splitlines(True), n + ' (current)', n + ' (proposed01)', n=0))
