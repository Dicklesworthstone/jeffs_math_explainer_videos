"""Read-only scope check: candidate03 leaves everything before `fn previews(` byte-identical to
build268 (so construct(), every wait_for/fade/pause/hold and the 01_6 chunking code), and
previews() still pushes the same mobs in the same order.

usage: python3 -I -B scope-check01.py BASE.rs CANDIDATE.rs
"""
import re
import sys

base, cand = (open(p, encoding="utf-8").read() for p in sys.argv[1:3])
head_b, head_c = base.split("fn previews(")[0], cand.split("fn previews(")[0]
print("text before fn previews( identical:", head_b == head_c, len(head_b), "chars")


def previews_body(s):
    body = s.split("fn previews(")[1]
    return body[:body.index("\n}\n")]


def mob_ops(body):
    return re.findall(r"out\.(push|extend|insert)\(", body)


ob, oc = mob_ops(previews_body(base)), mob_ops(previews_body(cand))
print("previews() out ops base", ob)
print("previews() out ops cand", oc)
print("same op sequence:", ob == oc)
anim = re.compile(r"fade_in_all|fade_out_all|stage\.play|wait_for|wait_until|pause\(|kit\.hold|music_breath|tween")
print("animation/timing calls in candidate previews():", anim.findall(previews_body(cand)))
tail_c = cand.split("fn previews(")[1].split("\n}\n", 1)[1]
print("animation/timing calls added after previews() (helper + consts + tests):",
      anim.findall(tail_c.split("#[cfg(test)]")[0]))
print("narration phrase literals in the diff region:", re.findall(r'say\(|"0\d_\d"', tail_c))
