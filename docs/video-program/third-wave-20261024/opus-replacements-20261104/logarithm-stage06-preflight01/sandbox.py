"""A disposable copy of just the files the preflight reads, for tests and the rehearsal.

Everything happens under a temp folder. The real tree is only read.
"""
import hashlib
import importlib.util
import json
import shutil
import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent


def load_local(name):
    spec = importlib.util.spec_from_file_location(f"stage06_{name}", HERE / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


pf = load_local("preflight")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def build_base(repo, dst):
    """Copy the tree the preflight needs: tools, canonical crate, stage06, request text and receipts."""
    repo, dst = Path(repo).resolve(), Path(dst)
    rels = set()

    def tree(rel):
        for p in (repo / rel).rglob("*"):
            if p.is_file() and "target" not in p.parts and "__pycache__" not in p.parts:
                rels.add(p.relative_to(repo))

    tree(pf.REL)
    tree(pf.STAGE)
    tree(pf.LANE / "narration-request-final80-01")
    tree(pf.LANE / "narration-request-text01")
    for p in (repo / "tools").glob("*.py"):
        rels.add(p.relative_to(repo))
    rels.update({pf.BINDING, pf.PREVIEW, Path("crates/explainer_kit/src/narration.rs")})
    for pair in pf.PATCH_FILES.values():
        rels.update(pair)
    plan = repo / pf.STAGE / "bound-plan"
    requests = json.loads((repo / pf.LANE / "narration-request-final80-01/requests.json").read_bytes())
    rels.add(Path(requests["script_path"]))
    for line in (plan / "inputs.sha256").read_text().splitlines():
        rels.add(Path(line.split("  ", 1)[1]))
    audit = json.loads((plan / "hold-audit.json").read_bytes())
    plan_map = json.loads((plan / pf.REL / "docs" / pf.MAP_NAME).read_bytes())
    for row in audit["helpers"] + audit["consumers"] + plan_map["shared_helpers"]:
        path = Path(row["path"])
        if not path.is_relative_to(pf.REL):
            rels.add(path)
    for rel in sorted(rels):
        out = dst / rel
        out.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / rel, out)
    return len(rels)


class Box:
    """One mutated copy. edit() rebinds hashes so a semantic check, not a stale hash, does the catching."""

    def __init__(self, base, work):
        self.root = Path(work)
        shutil.copytree(base, self.root)
        self.stage = self.root / pf.STAGE
        self.plan = self.stage / "bound-plan"
        self.crate = self.stage / pf.REL
        self.chapters = self.crate / "src/chapters"
        self.canon = self.root / pf.REL
        self.units = self.plan / pf.REL / "narration_units.json"
        self.map = self.plan / pf.REL / "docs" / pf.MAP_NAME
        self.requests = self.root / pf.LANE / "narration-request-final80-01/requests.json"

    def bound_files(self):
        names = ["hold-audit.json", "phrase-anchors.json", "phrase-timings-unmeasured-template.json", "direction-dispositions-proposed.json", "cue-delta.json", "inputs.sha256"]
        return [self.plan / n for n in names] + [self.units, self.map, self.requests]

    def swap(self, old, new):
        """Replace a hash everywhere the plan binds it, then follow what that changed."""
        if old == new:
            return
        changed = []
        for path in self.bound_files():
            before = path.read_bytes()
            after = before.replace(old.encode(), new.encode())
            if after != before:
                path.write_bytes(after)
                changed.append((sha(before), sha(after)))
        for a, b in changed:
            self.swap(a, b)

    def write(self, path, data, rebind=True):
        path = Path(path)
        old = sha(path.read_bytes())
        path.write_bytes(data)
        if rebind:
            self.swap(old, sha(data))

    def sub(self, path, old, new, count=1, rebind=True):
        """Exact text replacement that refuses to silently miss."""
        text = Path(path).read_text(encoding="utf-8")
        if text.count(old) != count:
            raise AssertionError(f"{path.name}: {old!r} occurs {text.count(old)} times, expected {count}")
        self.write(path, text.replace(old, new).encode("utf-8"), rebind)

    def chapter(self, name, old, new, canon=False):
        """Edit a staged chapter. canon=True makes the same edit in canonical, so the delta check stays quiet."""
        self.sub(self.chapters / name, old, new)
        if canon:
            Path(self.canon / "src/chapters" / name).write_bytes((self.chapters / name).read_bytes())

    def append(self, name, extra, canon=False):
        path = self.chapters / name
        self.write(path, path.read_bytes() + extra.encode("utf-8"))
        if canon:
            Path(self.canon / "src/chapters" / name).write_bytes(path.read_bytes())

    def edit_json(self, path, change, rebind=True):
        data = json.loads(Path(path).read_bytes())
        change(data)
        self.write(path, (json.dumps(data, indent=2, ensure_ascii=False) + "\n").encode("utf-8"), rebind)
