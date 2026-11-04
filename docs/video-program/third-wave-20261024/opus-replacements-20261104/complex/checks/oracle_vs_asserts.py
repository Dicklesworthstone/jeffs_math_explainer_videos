"""Compare the oracle's printed facts with facts.rs keys and with every
literal value the Rust tests assert (facts.rs `assert_eq!(get("k"), "v")`,
chapter tests `f.get("k"), Some("v")`). Read-only."""

import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
oracle = {}
for line in Path(sys.argv[2]).read_text(encoding="utf-8").splitlines():
    if line.startswith("#") or "\t" not in line:
        continue
    k, v = line.split("\t", 1)
    oracle[k] = v

facts_src = (root / "src/facts.rs").read_text(encoding="utf-8")
keys = set(re.findall(r'f\.(?:add|pin)\(\s*"([^"]+)"', facts_src))
print("facts.rs keys:", len(keys), "oracle keys:", len(oracle))
print("in facts.rs, not oracle:", sorted(keys - set(oracle)))
print("in oracle, not facts.rs:", sorted(set(oracle) - keys))

asserts = []
for f in [root / "src/facts.rs", *sorted((root / "src/chapters").glob("*.rs"))]:
    src = f.read_text(encoding="utf-8")
    asserts += [(f.name, k, v) for k, v in re.findall(r'get\("([^"]+)"\),\s*"([^"]*)"', src)]
    asserts += [(f.name, k, v) for k, v in re.findall(r'get\("([^"]+)"\),\s*Some\("([^"]*)"\)', src)]
bad = [(n, k, v, oracle.get(k)) for n, k, v in asserts if oracle.get(k) != v]
print("literal asserts:", len(asserts), "disagreeing with the oracle:", bad)
sys.exit(1 if bad or keys != set(oracle) else 0)
