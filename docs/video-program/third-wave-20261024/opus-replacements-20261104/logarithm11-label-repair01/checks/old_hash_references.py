"""Which files name the three hashes a rebind replaces. Reads only.

python3 -I -B old_hash_references.py REPO_ROOT   (run from anywhere)

Searches the crate, tools, crates, and the root's logarithm run folders and staging, in text-like files. Each root run
file is a dated record of what that run read, so it is listed to be understood, not edited.
"""
import sys
from pathlib import Path

HASHES = {
    "source unreachable.rs": "43c8dac362df7fac8d6c974b3c108a25dee677faac85d093bfdafc1564db6527",
    "map oliver12": "7dbb3e1dcc309582545b3ccf490e40c581d209c091460195b6b9f2955b2b3860",
    "units": "34747511327c7c8bfee51c7a04c1fc721199d11117cb840c3df1e3205864dc7d",
}
TOPS = [
    "explainer_corpus/logarithm_mysteries",
    "tools",
    "crates",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104",
]
SUFFIXES = {".json", ".py", ".sha256", ".txt", ".md", ".tsv", ".sh", ".rs", ".log", ""}
LIMIT = 8_000_000


def main():
    repo = Path(sys.argv[1]).resolve()
    found = {name: [] for name in HASHES}
    scanned = 0
    for top in TOPS:
        for path in sorted((repo / top).rglob("*")):
            if not path.is_file() or path.suffix not in SUFFIXES or "__pycache__" in path.parts:
                continue
            if "logarithm11-label-repair01" in path.parts:  # this folder quotes the hashes on purpose
                continue
            if path.stat().st_size > LIMIT:
                continue
            scanned += 1
            data = path.read_bytes()
            for name, value in HASHES.items():
                if value.encode() in data:
                    found[name].append(str(path.relative_to(repo)))
    print("files scanned:", scanned)
    root = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/"
    stage = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
    for name, paths in found.items():
        print(f"--- {name}: {len(paths)} files")
        for p in paths:
            print("  ", p.replace(root, "ROOT_RUN/").replace(stage, "STAGING/"))


if __name__ == "__main__":
    main()
