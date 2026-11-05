#!/usr/bin/env python3
"""Build the 19-package table and config template from the lab's real lock, and check them.

Reads only. Writes packages.tsv and engine-budget.config.toml.in beside this folder's parent, and
prints what it checked. Needs Python 3.11 or later for tomllib. Runs no Cargo.

    python3 -I -B manifest.py
"""
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE.parent
REPO = HERE.parents[5]
PIN = Path.home() / ".cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62"
CRATE = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/color-native-stage01/crate"
URL = "https://github.com/Dicklesworthstone/franken_manim"
REV = "a399d626c43ecc852a5c795288979f52b446bc5c"
SOURCE = f"git+{URL}?rev={REV}#{REV}"
ASKED = {
    "fmn", "fmn-anim", "fmn-cache", "fmn-codec", "fmn-config", "fmn-core", "fmn-dmath", "fmn-frame", "fmn-geom",
    "fmn-hash", "fmn-library", "fmn-mobject", "fmn-output", "fmn-platform", "fmn-render", "fmn-runtime", "fmn-scene",
    "fmn-tex", "fmn-text",
}


def toml(path):
    return tomllib.loads(path.read_text())


def tell(text):
    print(text.replace(str(Path.home()), "~").replace(str(REPO), "$REPO"))


# --- the lock
lock = toml(CRATE / "Cargo.lock")
ours = [p for p in lock["package"] if "franken_manim" in p.get("source", "")]
names = {p["name"] for p in ours}
assert names == ASKED and len(ours) == 19, sorted(names ^ ASKED)
assert {p["source"] for p in ours} == {SOURCE}, {p["source"] for p in ours}
assert not [p for p in lock["package"] if p["name"] in ASKED and p not in ours]
tell(f"lab lock: {len(lock['package'])} packages, exactly 19 from {URL} at {REV[:8]}, none duplicated by name")
assert not [p for p in lock["package"] if p["name"] == "block"]
tell("lab lock has no `block`, so the engine's own [patch.crates-io] entry has nothing to do here")

# --- the export
root = toml(PIN / "Cargo.toml")
workspace_version = root["workspace"]["package"]["version"]
members = root["workspace"].get("members", [])
rows = []
for package in sorted(ours, key=lambda p: p["name"]):
    name = package["name"]
    manifest = toml(PIN / "crates" / name / "Cargo.toml")
    declared = manifest["package"]["version"]
    version = workspace_version if isinstance(declared, dict) else declared
    assert manifest["package"]["name"] == name and version == package["version"], name
    assert any(m == f"crates/{name}" or m == "crates/*" for m in members), name
    rows.append((name, package["version"], package["source"], f"crates/{name}"))
tell(f"pinned checkout: every package is crates/<name>, a workspace member, at the lock's version {workspace_version}")

# --- why all of them
direct = {}
for label, path in (("the lab crate", CRATE / "Cargo.toml"), ("explainer_kit", REPO / "crates/explainer_kit/Cargo.toml"), ("explainer_score", REPO / "crates/explainer_score/Cargo.toml")):
    manifest = toml(path)
    found = sorted(d for table in ("dependencies", "build-dependencies") for d in manifest.get(table, {}) if d in ASKED)
    direct[label] = found
    tell(f"{label} names directly: {found}")
repo_deps = toml(REPO / "Cargo.toml")["workspace"]["dependencies"]
for name in ("fmn", "fmn-codec", "fmn-output"):
    assert repo_deps[name]["git"] == URL and repo_deps[name]["rev"] == REV, name
tell("the repo workspace pins fmn, fmn-codec and fmn-output to the same URL and rev")
tell("so patching only fmn or fmn-render would leave git fmn-output and fmn-codec beside their path copies: two of each type")

ws_deps = root["workspace"]["dependencies"]
edges, optional_edges, artifacts = {}, {}, []
for name in ASKED:
    manifest = toml(PIN / "crates" / name / "Cargo.toml")
    tables = [manifest.get("dependencies", {}), manifest.get("build-dependencies", {})]
    for target in manifest.get("target", {}).values():
        tables += [target.get("dependencies", {}), target.get("build-dependencies", {})]
    required, optional = set(), set()
    for table in tables:
        for dep, spec in table.items():
            spec = spec if isinstance(spec, dict) else {}
            resolved = ws_deps.get(dep, {}) if spec.get("workspace") else spec
            resolved = resolved if isinstance(resolved, dict) else {}
            if "artifact" in spec or "artifact" in resolved:
                artifacts.append((name, dep))
            if "path" in resolved:
                (optional if spec.get("optional") else required).add(Path(resolved["path"]).name)
    edges[name], optional_edges[name] = required, optional
closure, stack = set(), ["fmn", "fmn-codec", "fmn-output"]
while stack:
    name = stack.pop()
    if name not in closure:
        closure.add(name)
        stack += sorted(edges.get(name, ()))
outside = sorted(closure - ASKED)
unreached = sorted(ASKED - closure)
sibling_optional = sorted({d for n in ASKED for d in optional_edges[n]} - ASKED)
assert not outside, outside
tell(f"required path dependencies from those three roots: {len(closure)} packages, none outside the 19")
tell(f"of the 19, reached only through features or targets this walk does not follow: {unreached or 'none'}")
tell(f"optional sibling crates outside the 19, unactivated in the lock: {sibling_optional or 'none'}")
assert not artifacts, artifacts
tell("none of the 19 uses an `artifact =` dependency, so the lab does not need bindeps; fmn-conformance alone does, in the export")

# --- config and toolchain
ours_cfg, repo_cfg = toml(PIN / ".cargo/config.toml"), toml(REPO / ".cargo/config.toml")
assert ours_cfg["build"]["rustflags"] == repo_cfg["build"]["rustflags"] == ["-Z", "threads=4"]
assert ours_cfg["build"]["fingerprint"] == repo_cfg["build"]["fingerprint"] == "content"
assert ours_cfg["unstable"]["checksum-freshness"] and repo_cfg["unstable"]["checksum-freshness"]
assert ours_cfg["unstable"].get("bindeps") is True and "bindeps" not in repo_cfg["unstable"]
tell("engine and repo .cargo/config.toml agree on rustflags, fingerprint, checksum-freshness; only the engine sets bindeps")
ours_tc, repo_tc = toml(PIN / "rust-toolchain.toml"), toml(REPO / "rust-toolchain.toml")
assert ours_tc["toolchain"]["channel"] == repo_tc["toolchain"]["channel"] == "nightly-2026-08-31"
tell("both pin nightly-2026-08-31")
home_cfg = Path.home() / ".cargo/config.toml"
if home_cfg.exists():
    home = toml(home_cfg)
    tell(f"~/.cargo/config.toml sets rustflags: {bool(home.get('build', {}).get('rustflags'))}")

# --- would the engine's tests resolve offline
engine = toml(PIN / "Cargo.lock")
by_name = {}
for package in engine["package"]:
    by_name.setdefault(package["name"], []).append(package)


def resolve(text):
    parts = text.split(" ")
    options = by_name[parts[0]]
    if len(parts) == 1:
        assert len(options) == 1, text
        return options[0]
    return next(p for p in options if p["version"] == parts[1])


seen, stack = {}, [resolve("fmn"), resolve("fmn-render")]
while stack:
    package = stack.pop()
    key = (package["name"], package["version"])
    if key not in seen:
        seen[key] = package
        stack += [resolve(d) for d in package.get("dependencies", [])]
cache = list((Path.home() / ".cargo/registry/cache").glob("*"))
missing, third_party, git_sources = [], 0, set()
for (name, version), package in sorted(seen.items()):
    source = package.get("source", "")
    if source.startswith("registry+"):
        third_party += 1
        if not any((folder / f"{name}-{version}.crate").exists() for folder in cache):
            missing.append(f"{name} {version}")
    elif source.startswith("git+"):
        git_sources.add(source.split("#")[0])
tell(f"engine lock, everything reachable from fmn and fmn-render by lock edges (an upper bound: it counts every member's dev-dependencies and every target): {len(seen)} packages, {third_party} from the registry")
tell(f"  missing from the local registry cache: {len(missing)}")


# The same walk, narrowed: dev-dependencies count only for the two crates under test, and a target
# table counts only if it can apply on this host. Third-party edges stay as the lock has them, which
# still overstates anything behind an unused feature.
def host_applies(cfg):
    text = cfg.replace(" ", "")
    if any(word in text for word in ('target_os="windows"', "windows", 'target_os="linux"', "wasm", 'target_os="android"', 'target_os="ios"')) and "not(" not in text and "any(" not in text:
        return False
    return True


members = {p.name for p in (PIN / "crates").iterdir() if (p / "Cargo.toml").is_file()}


def member_deps(name):
    manifest = toml(PIN / "crates" / name / "Cargo.toml")
    tables = [manifest.get("dependencies", {}), manifest.get("build-dependencies", {})]
    for cfg, target in manifest.get("target", {}).items():
        if host_applies(cfg):
            tables += [target.get("dependencies", {}), target.get("build-dependencies", {})]
    if name in ("fmn", "fmn-render"):
        tables.append(manifest.get("dev-dependencies", {}))
    out = set()
    for table in tables:
        for dep, spec in table.items():
            if isinstance(spec, dict) and spec.get("optional"):
                continue
            resolved = ws_deps.get(dep, {}) if isinstance(spec, dict) and spec.get("workspace") else spec
            package = resolved.get("package", dep) if isinstance(resolved, dict) else dep
            out.add(Path(resolved["path"]).name if isinstance(resolved, dict) and "path" in resolved else package)
    return out


narrow, parents, stack = {}, {}, ["fmn", "fmn-render"]
while stack:
    name = stack.pop()
    if name in narrow:
        continue
    if name in members:
        narrow[name] = None
        for dep in sorted(member_deps(name)):
            if dep in by_name and dep not in narrow:
                parents.setdefault(dep, name)
                stack.append(dep)
    else:
        for package in by_name.get(name, []):
            narrow[name] = package
            for dep in package.get("dependencies", []):
                child = dep.split(" ")[0]
                if child not in narrow:
                    parents.setdefault(child, name)
                    stack.append(child)


def route(name):
    steps = [name]
    while steps[-1] in parents and len(steps) < 40:
        steps.append(parents[steps[-1]])
    return " <- ".join(steps)


lab_versions = {}
for package in lock["package"]:
    lab_versions.setdefault(package["name"], []).append(package["version"])
sources = list((Path.home() / ".cargo/registry/src").glob("*"))
needed_missing = []
for item in missing:
    name, version = item.split(" ")
    unpacked = any((folder / f"{name}-{version}").is_dir() for folder in sources)
    reached = name in narrow
    if reached and not unpacked:
        needed_missing.append(item)
    tell(f"    {item}: unpacked source present {unpacked}; lab lock has {lab_versions.get(name, [])}; narrowed walk reaches it: {reached}" + (f"\n      via {route(name)}" if reached else ""))
tell(f"  in the narrowed walk and on disk in neither form: {needed_missing or 'none'}")
tell(f"  git sources reachable: {sorted(git_sources) or 'none'}")
db = Path.home() / ".cargo/git/db"
for source in sorted(git_sources):
    stem = source.split("?")[0].rsplit("/", 1)[-1]
    tell(f"    {stem}: local git db present: {any(db.glob(stem + '-*'))}")

# --- outputs
table = "package\tlock_version\tlock_source\texport_path\n" + "".join("\t".join(r) + "\n" for r in rows)
(OUT / "packages.tsv").write_text(table)
template = ["# Every package of the lab's lock from franken_manim, as path copies of one patched export.\n"]
template.append(f'[patch."{URL}"]\n')
template += [f'{r[0]} = {{ path = "@EXPORT@/{r[3]}" }}\n' for r in rows]
(OUT / "engine-budget.config.toml.in").write_text("".join(template))
tell(f"wrote packages.tsv ({len(rows)} rows) and engine-budget.config.toml.in")
