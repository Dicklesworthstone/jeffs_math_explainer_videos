#!/usr/bin/env python3
"""Attribute leaf hotspots to their nearest franken_manim caller.

    perf script -i run.data -F comm,tid,ip,sym | python3 attribute.py [LEAF_REGEX]

For every sample whose leaf frame matches LEAF_REGEX (default: page
zeroing, page-fault entry, memcpy/memset, kernel copy-in), report the first
`fmn_*` frame above it, so kernel and libc time is charged to the engine
code that caused it.
"""
import collections
import re
import sys

LEAF = re.compile(sys.argv[1] if len(sys.argv) > 1 else
                  r"kernel_init_pages|clear_page|irqentry|_copy_from_iter|memmove|memset|memcpy")
OWN = re.compile(r"fmn_[a-z_]+::|<fmn_|hoeffding::")


def samples(stream):
    block = []
    for line in stream:
        if line.strip():
            block.append(line.rstrip("\n"))
        elif block:
            yield block
            block = []
    if block:
        yield block


def sym(frame_line):
    parts = frame_line.strip().split(None, 1)
    s = parts[1] if len(parts) > 1 else parts[0]
    s = re.sub(r"\s+\(.*\)$", "", s)  # drop "(dso)"
    s = re.sub(r"::h[0-9a-f]{16}$", "", s)
    return s[:140]


total = 0
by_leaf = collections.Counter()
by_owner = collections.Counter()
for block in samples(sys.stdin):
    total += 1
    head = block[0].split()
    comm = head[0] if head else "?"
    frames = block[1:]
    if not frames:
        continue
    leaf = sym(frames[0])
    if not LEAF.search(leaf):
        continue
    owner = next((sym(f) for f in frames[1:] if OWN.search(f)), "<no fmn frame>")
    by_leaf[leaf.split("+")[0]] += 1
    by_owner[(comm, owner)] += 1

print(f"samples: {total}; matching leaf: {sum(by_leaf.values())}")
for (comm, owner), n in by_owner.most_common(15):
    print(f"{100 * n / total:6.2f}%  {comm:16} {owner}")
