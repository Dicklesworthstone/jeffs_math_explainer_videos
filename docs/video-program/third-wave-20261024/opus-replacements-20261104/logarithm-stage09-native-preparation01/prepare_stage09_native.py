#!/usr/bin/env python3
"""Logarithm stage09 native preparation (root's Mail 3882). Root reviews and runs every step.

Installs root's integrated stage09 candidate (script 559afa36: the two reviewed wording changes, 04_4
and 08_5) as a crate beside root's W engine, then writes one resource-guarded job: the native tests,
the binary and strict Clippy, with the shared binary preserved before the job and the new one copied
out after it. Nothing here compiles or renders, and nothing here edits canonical source, root's
stage08, the EL candidate, Beads or the TODO.

Engine: root's W stage, through W/engine-budget.toml, exactly as root's stage08 build: canonical
franken_manim a399d626 plus root's three geometry patches. W/engine is checked file for file against
the stage08 build's guard rows, so the only difference from that build is the stage09 crate. The
six-entry secondary-validation stack is qualified but not promoted; this helper refuses a W/engine
that carries any of the twelve files that stack changes, and it never promotes the canonical pin.

    python3 -I -B prepare_stage09_native.py check   --w W
    python3 -I -B prepare_stage09_native.py install --w W --into D
    python3 -I -B prepare_stage09_native.py script  --w W --into D --attempt 01
    python3 -I -B prepare_stage09_native.py verify  --w W --into D --attempt 01    (after root's run)

Run from the repository root. W and D are absolute; D must be outside the repository because the
installed manifest names W's absolute paths. --el, --white, --receipts and --target-root exist for
the checks' scratch copies; root leaves them at their defaults. Exit 0 pass, 1 refused, 2 usage.
"""
import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import sys
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve()
REPO = HERE.parents[5]
EL_REL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
WHITE_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-clarity-integration01"
RECEIPTS_REL = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
LOCK_CHECK_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-geometry-validation02/verify_patched_lock.py"
GUARD_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py"
GUARD_SOURCE_REL = ("narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
                    "run_bounded_phase_oliver_owner48g_target14g_20261102.py")
CRATE = "explainer_corpus/logarithm_mysteries"
STAGE = "logarithm-stage09"
FMN_GIT = "https://github.com/Dicklesworthstone/franken_manim"

# Stage09: root's integration (receipt) of White's sealed bundle, installed into EL.
PIN = {
    "white_sums": "5fcb5ea628feb09888f5576cc78c4a05cb745180f68f7361cc6f601173cd2a7d",
    "root_receipt": "e16430ecc6728cde8325d761ee6d55b533dd1232025c15d6d4603f13e9e69282",
    "script": "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a",
    "units": "1132a5a452b339dd4d6fb340668c525b2e10dfe892d27035ce11957e80cafce9",
    "map16": "14e87893ea389ef2cdf827f0a7e59a8e2b65ac537b726d796ffceff5117e4787",
    "product_rule": "bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635",
    "cargo_toml": "a8c95209d3f1034a4fcfdb4418f7bf48d4afedf4e79c3730a84476e68dbc84d6",
    "cargo_lock": "50cd8811ba9df6ae9b6a55c9a13b52d722214c59de30257ad72fda66a56ded2a",
}
# The two wording changes: sha256 of the text column of each script row, the new words and the old.
WORDING = {
    "04_4": ("7e4e96121ff25f997a75bc882f70884bd61b3b039203e8df9c9d46fe545d2e83",
             "which is why base e is called natural.", "which is why this base is called natural."),
    "08_5": ("8e26bcb04dba0673057649b76a1fce2ce0375bcc1ff1a8fffb7d8379c13f4d82",
             "Taking half of the lost lap", "Halving the lost lap"),
}
# Superseded stage08 bytes: refused wherever stage09 bytes are required.
STAGE08 = {
    "script": "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e",
    "units": "0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c",
    "product_rule": "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db",
    "map15": "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4",
}
# The only files stage09 changes or adds against the stage08 sources root built from.
CHANGED = {"script.tsv", "narration_units.json", "src/chapters/product_rule.rs"}
ADDED = {"docs/narration-direction-map-oliver16.json"}
# Root's W stage as its stage08 build used it (W/logarithm-stage08-build01, exit 0, 02:39:03-02:39:44Z).
W_PIN = {
    "engine-budget.toml": "cdd28268804053230a1feb172f190d0010e09b3021e8b3ab0c69d9096feb9ed6",
    "Cargo.lock.pinned": "b14e1fac6dd2ddc7dd5fca9dec8feae2c626b72a4cbe55ac7a89d64b326ecc04",
    "Cargo.lock.view-pinned": "1c7e02438e0a6aaedf0fd360f0316dd6132309d23f51aeef88c2250bfa63e5a1",
    "logarithm-stage08-build01-inputs.json": "f730d69d99461107548be0b2cd5ab3945e563ceedd18005948efc937c7616fb0",
    "logarithm-stage08-source-binding01.json": "eb71f333ffba6ec99024215365d9d8ce0e62542b93e74ef5cc6f43ed574bb5e7",
    "logarithm-stage08-build01.sh": "f6ffffa1075a3f850d123564e3e9a77e6d2282691424a53b9729f6009dd49f57",
    "logarithm-stage08-original-Cargo.toml": "a8c95209d3f1034a4fcfdb4418f7bf48d4afedf4e79c3730a84476e68dbc84d6",
    "logarithm-stage08-original-Cargo.lock": "50cd8811ba9df6ae9b6a55c9a13b52d722214c59de30257ad72fda66a56ded2a",
    "logarithm-stage08/Cargo.toml": "dc41f2392d8ca586d0a42d59a0a3866e8f20932a4db4eb8b0e02140d177f0a47",
    "logarithm-stage08/Cargo.lock": "e0b265e844bdf2ec1f79ba3219eef5788cc2e9128b122283cb5e7fbf70fdb721",
}
TOOL_PIN = {
    LOCK_CHECK_REL: "1756506bfa371f79cab551689ef3d466416909c0fb521f72b354f4cc61600327",
    GUARD_REL: "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097",
    GUARD_SOURCE_REL: "819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da",
}
# The twelve engine files the secondary-validation stack changes, as they are without it (its
# manifests/unpatched.json, sha256 d4ee47be...); None means the stack adds the file.
S_UNPATCHED = {
    "crates/fmn-render/src/engine.rs": "85bdc6b721cf40fb836eaa07d324138ced6d8edd0e3e7919e37379b62f764d8c",
    "crates/fmn-render/src/lib.rs": "2d41dce6d1ca15daabcd04e8f39b36e563e58f342411b7903c0d98602bc5d014",
    "crates/fmn-render/src/retained.rs": "c2124bcc4514303dfd38a748a7bc0b0c93f68a559b21a447c9ec803d5725ffae",
    "crates/fmn-render/src/three_d.rs": "61ae25888f2f38602ad8a244f706600e11c520c96e0b56780bf5203200214451",
    "crates/fmn-render/src/vector_frame.rs": "81fbf8c03a0d93bd8e7476a5f5f441be45eabdac7f05ddf2d3138b5f004a5181",
    "crates/fmn-render/tests/camera_route_limits.rs": None,
    "crates/fmn-render/tests/frame_table_limits.rs": None,
    "crates/fmn/src/rendering.rs": "543fb1f5d1b90cc3ae95d0139bbab94702b8722f9ce816d7fb090a3f7e6efdc1",
    "crates/fmn/src/rendering/pipeline.rs": "634c22b57bd59d225e793a2201330729136ba162e42bdc9a146863c2d758573a",
    "crates/fmn/tests/native_bundle_pipeline.rs": "187a5883bf04d6185e95f3d7f11299ae4285668decacdca90d29203d9b925f60",
    "crates/fmn/tests/native_camera_pipeline.rs": "1885049346a6a0f9a1e8ecffb79fda1f949e734a4be1ab608311c51dd4d7618a",
    "crates/fmn/tests/native_render.rs": "d64416e4536cde9a0fbfa7e3e87ac488a36b0b9723c789bb304a68a818ac70cd",
}
ENGINE_FILES, VIEW_FILES = 4057, 68
# The shared binary root's stage08 build linked (02:39:28Z), reported, not required.
STAGE08_BINARY = "e92334010297395ac323d869a341d34ce583fc841deee90fee5cde145e819050"
TESTS = 241  # #[test] functions in the stage09 sources, none ignored; stage08's build ran 241
CHAPTERS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade", "04_undoing_growth",
            "05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule", "09_matrix_logs",
            "10_doing_nothing", "11_unreachable", "12_what_it_forgets"]
FILM_BINARIES = ["debug/complex_analysis_visual", "debug/examples/plate_lab"]  # pinned only to show no relink


class Refused(Exception):
    pass


def need(condition, message):
    if not condition:
        raise Refused(message)


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def tree(root):
    """{relative path: sha256} of every file under root; links and non-files are refused."""
    root = Path(root)
    need(root.is_dir() and not root.is_symlink(), f"not a directory: {root}")
    found = {}
    for parent, dirs, names in os.walk(root, followlinks=False):
        for name in dirs + names:
            path = Path(parent) / name
            need(not path.is_symlink(), f"symbolic link: {path}")
            need(path.is_dir() or path.is_file(), f"not a regular file: {path}")
        for name in names:
            path = Path(parent) / name
            found[path.relative_to(root).as_posix()] = sha(path)
    return found


def pinned(path, digest, what):
    need(Path(path).is_file(), f"{what}: missing {path}")
    actual = sha(path)
    need(actual == digest, f"{what}: sha256 {actual[:12]}, expected {digest[:12]} ({path})")


class Paths:
    def __init__(self, args):
        self.w = Path(args.w)
        need(self.w.is_absolute() and self.w.is_dir(), "--w must be an existing absolute directory")
        self.el = Path(args.el) if args.el else REPO / EL_REL
        self.white = Path(args.white) if args.white else REPO / WHITE_REL
        self.receipts = Path(args.receipts) if args.receipts else REPO / RECEIPTS_REL
        self.target = Path(args.target_root) if args.target_root else REPO / "target"
        self.s9 = self.el / "combined-repair-stage09"
        self.candidate = self.s9 / "candidate"
        self.crate = self.candidate / CRATE
        self.into = None
        if getattr(args, "into", None):
            self.into = Path(args.into)
            need(self.into.is_absolute() and self.into.is_dir(), "--into must be an existing absolute directory")
            need(not self.into.resolve().is_relative_to(REPO.resolve()),
                 "--into is inside the repository: the stage names absolute scratch paths, keep it outside")
            for forbidden in (self.w / "engine", self.w / "repo-view", self.w / "logarithm-stage08", self.el):
                need(not self.into.resolve().is_relative_to(forbidden.resolve()), f"--into is inside {forbidden}")
            self.stage = self.into / STAGE

    def shared_binary(self):
        return self.target / "geometry-kit01" / "debug" / "logarithm_mysteries"


# ---------------------------------------------------------------- sources

def script_rows(text):
    rows = {}
    for line in text.splitlines():
        if line and not line.startswith("#"):
            fields = line.split("\t")
            need(len(fields) == 2 and fields[0] not in rows, f"script row malformed or repeated: {line[:40]}")
            rows[fields[0]] = fields[1]
    return rows


def check_sources(p):
    """Every stage09 source byte, bound to White's seal, root's receipt and the reviewed wording."""
    facts = {}
    pinned(p.white / "SHA256SUMS", PIN["white_sums"], "White's sealed bundle")
    sealed = {}
    for line in (p.white / "SHA256SUMS").read_text().splitlines():
        digest, _, rel = line.partition("  ")
        if rel.startswith("stage09/"):
            sealed[rel[len("stage09/"):]] = digest
    need(len(sealed) == 58, f"White's seal lists {len(sealed)} stage09 files, expected 58")
    installed = {f"candidate/{rel}": h for rel, h in tree(p.candidate).items()}
    installed.update({f"bound-plan/{rel}": h for rel, h in tree(p.s9 / "bound-plan").items()})
    need(set(installed) == set(sealed), "EL stage09 candidate/bound-plan files differ from White's seal: "
         f"extra {sorted(set(installed) - set(sealed))[:3]}, missing {sorted(set(sealed) - set(installed))[:3]}")
    moved = sorted(rel for rel in sealed if installed[rel] != sealed[rel])
    need(not moved, f"EL stage09 bytes differ from White's seal: {moved[:3]}")
    crate = {rel[len(f"candidate/{CRATE}/"):]: h for rel, h in installed.items() if rel.startswith(f"candidate/{CRATE}/")}
    need(len(crate) == 48 and len(installed) == 58, f"stage09 crate has {len(crate)} files, expected 48")
    facts["sealed_files"] = 58

    pinned(p.s9 / "root-integration-receipt01.json", PIN["root_receipt"], "root's integration receipt")
    receipt = json.loads((p.s9 / "root-integration-receipt01.json").read_text())
    for key, value in (("sealed_manifest_sha256", PIN["white_sums"]), ("script_sha256", PIN["script"]),
                       ("changed_cues", ["04_4", "08_5"]), ("unchanged_cues", 78), ("installed_files", 58),
                       ("install_verify_exit", 0), ("root_replay_exit", 0), ("voice", "ElevenLabs Oliver")):
        need(receipt.get(key) == value, f"root's receipt {key} is {receipt.get(key)!r}, expected {value!r}")
    facts["receipt_open_gates"] = {k: receipt.get(k) for k in
                                   ("native_test", "new_recordings", "listening", "fine_sync", "independent_final_av")}

    need(crate["script.tsv"] != STAGE08["script"], "the superseded stage08 script b89eab60 is installed")
    need(crate["script.tsv"] == PIN["script"], f"script.tsv {crate['script.tsv'][:12]}, expected 559afa36")
    text = (p.crate / "script.tsv").read_text(encoding="utf-8")
    rows = script_rows(text)
    need(len(rows) == 80, f"script has {len(rows)} cues, expected 80")
    for cue, (digest, new, old) in WORDING.items():
        need(hashlib.sha256(rows[cue].encode()).hexdigest() == digest, f"{cue} text is not the reviewed wording")
        need(rows[cue].count(new) == 1, f"{cue} lacks {new!r}")
        need(old not in text, f"stage08 wording {old!r} is still in the script")
    facts["wording"] = {cue: digest for cue, (digest, _, _) in WORDING.items()}

    need(crate["narration_units.json"] != STAGE08["units"], "the superseded stage08 units 0e27709b are installed")
    need(crate["narration_units.json"] == PIN["units"], "narration_units.json is not root's stage09 units")
    units = json.loads((p.crate / "narration_units.json").read_text())
    contract = units["direction_contract"]
    need(units["script_sha256"] == PIN["script"] and contract["script_sha256"] == PIN["script"],
         "units are not bound to script 559afa36")
    need(contract["map"] == {"path": f"{CRATE}/docs/narration-direction-map-oliver16.json", "sha256": PIN["map16"]},
         f"units bind map {contract['map']}, expected oliver16 14e87893")
    need("oliver15" not in (p.crate / "narration_units.json").read_text(), "units still name the stage08 map oliver15")
    need(len(units["units"]) == 80, f"{len(units['units'])} units, expected 80")
    by_cue = {u["line_ids"][0]: u for u in units["units"] if len(u["line_ids"]) == 1}
    for cue in WORDING:
        need(by_cue[cue]["text"] == rows[cue], f"unit for {cue} does not carry the script's stage09 text")
    need(crate["docs/narration-direction-map-oliver16.json"] == PIN["map16"], "oliver16 map bytes moved")
    need(crate.get("docs/narration-direction-map-oliver15.json") == STAGE08["map15"],
         "the historical oliver15 map is not the stage08 bytes")
    need(crate["src/chapters/product_rule.rs"] == PIN["product_rule"]
         and crate["src/chapters/product_rule.rs"] != STAGE08["product_rule"],
         "product_rule.rs is not the stage09 bytes (its test finds the new 08_5 words)")
    need(crate["Cargo.toml"] == PIN["cargo_toml"] and crate["Cargo.lock"] == PIN["cargo_lock"],
         "candidate Cargo.toml/Cargo.lock are not the stage08 originals")
    facts["crate"] = crate
    return facts


# ---------------------------------------------------------------- engine

def check_engine(p, crate):
    """Root's W exactly as its stage08 build used it, and none of the secondary stack."""
    w = p.w
    for rel, digest in W_PIN.items():
        pinned(w / rel, digest, f"W/{rel}")
    for rel, digest in TOOL_PIN.items():
        pinned(REPO / rel, digest, rel)

    budget = tomllib.loads((w / "engine-budget.toml").read_text())
    patches = budget["patch"][FMN_GIT]
    need(len(budget) == 1 and len(budget["patch"]) == 1, "engine-budget.toml holds more than the franken_manim patch")
    need(len(patches) == 19, f"engine-budget.toml patches {len(patches)} crates, expected 19")
    for name, entry in patches.items():
        need(entry == {"path": f"{w}/engine/crates/{name}"}, f"{name} is not W's engine copy: {entry}")
    lock = tomllib.loads((p.crate / "Cargo.lock").read_text())["package"]
    need({r["name"] for r in lock if FMN_GIT in r.get("source", "")} == set(patches),
         "the lock's franken_manim packages and the budget's patches differ")

    rows = json.loads((w / "logarithm-stage08-build01-inputs.json").read_text())["files"]
    for sub, count in (("engine", ENGINE_FILES), ("repo-view", VIEW_FILES)):
        prefix = f"{w}/{sub}/"
        recorded = {r["path"][len(prefix):]: r["sha256"] for r in rows if r["path"].startswith(prefix)}
        now = tree(w / sub)
        need(len(recorded) == count and set(now) == set(recorded),
             f"W/{sub}: {len(now)} files now, {len(recorded)} in stage08's guard rows; "
             f"added {sorted(set(now) - set(recorded))[:3]}, gone {sorted(set(recorded) - set(now))[:3]}")
        moved = sorted(rel for rel in now if now[rel] != recorded[rel])
        need(not moved, f"W/{sub} changed since the stage08 build: {moved[:3]}")
    for rel, digest in S_UNPATCHED.items():
        path = w / "engine" / rel
        if digest is None:
            need(not path.exists(), f"W/engine carries the secondary stack's {rel}")
        else:
            need(path.is_file() and sha(path) == digest, f"W/engine {rel} is not the unpatched file (secondary stack?)")

    binding = json.loads((w / "logarithm-stage08-source-binding01.json").read_text())
    stage08 = {r["path"]: r for r in binding["files"]}
    need(set(binding["changes"]) == {"Cargo.toml", "Cargo.lock"} and len(stage08) == 47, "stage08 binding shape moved")
    need(tree(w / "logarithm-stage08") == {rel: r["stage_sha256"] for rel, r in stage08.items()},
         "W/logarithm-stage08 no longer matches root's stage08 binding")
    need(set(crate) - set(stage08) == ADDED and not set(stage08) - set(crate),
         f"stage09 files beyond stage08's plus the oliver16 map: {sorted(set(crate) ^ set(stage08))}")
    changed = {rel for rel in stage08 if stage08[rel]["source_sha256"] != crate[rel]}
    need(changed == CHANGED, f"stage09 changes {sorted(changed)} against stage08's sources, expected {sorted(CHANGED)}")
    return {"engine_files": ENGINE_FILES, "repo_view_files": VIEW_FILES, "patched_crates": 19,
            "stage09_vs_stage08": sorted(changed | ADDED)}


def derive(p):
    """Cargo.toml and Cargo.lock as root's stage08 copy has them, from the stage09 candidate's originals."""
    text = (p.crate / "Cargo.toml").read_text()
    out = text
    for name in ("explainer_kit", "explainer_score"):
        old = f'{name} = {{ path = "../../crates/{name}" }}\n'
        need(text.count(old) == 1, f"Cargo.toml lacks the single {name} path line")
        need((p.w / "repo-view/crates" / name / "Cargo.toml").is_file(), f"W/repo-view lacks {name}")
        out = out.replace(old, f'{name} = {{ path = "{p.w}/repo-view/crates/{name}" }}\n')
    head, *parts = (p.crate / "Cargo.lock").read_text().split("[[package]]\n")
    lock = head + "".join("[[package]]\n" + "".join(
        line for line in part.splitlines(keepends=True)
        if not (line.startswith("source = ") and "franken_manim" in line)) for part in parts)
    toml_b, lock_b = out.encode(), lock.encode()
    need(toml_b == (p.w / "logarithm-stage08/Cargo.toml").read_bytes(),
         "derived Cargo.toml differs from root's W/logarithm-stage08 copy")
    need(lock_b == (p.w / "logarithm-stage08/Cargo.lock").read_bytes(),
         "derived Cargo.lock differs from root's W/logarithm-stage08 copy")
    return toml_b, lock_b


def check_all(p):
    facts = check_sources(p)
    facts["engine"] = check_engine(p, facts["crate"])
    toml_b, lock_b = derive(p)
    facts["stage_cargo"] = {"Cargo.toml": hashlib.sha256(toml_b).hexdigest(),
                            "Cargo.lock": hashlib.sha256(lock_b).hexdigest()}
    return facts, toml_b, lock_b


# ---------------------------------------------------------------- install

def outputs(p):
    return {"stage": p.stage, "toml": p.into / f"{STAGE}-original-Cargo.toml",
            "lock": p.into / f"{STAGE}-original-Cargo.lock", "binding": p.into / f"{STAGE}-source-binding01.json"}


def install(p):
    facts, toml_b, lock_b = check_all(p)
    out = outputs(p)
    for path in out.values():
        need(not path.exists() and not path.is_symlink(), f"refusing to overwrite {path}")
    out["stage"].mkdir()
    rows = []
    for rel, source_sha in sorted(facts["crate"].items()):
        dest = out["stage"] / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        if rel in ("Cargo.toml", "Cargo.lock"):
            with dest.open("xb") as handle:
                handle.write(toml_b if rel == "Cargo.toml" else lock_b)
            shutil.copystat(p.crate / rel, dest)
        else:
            shutil.copy2(p.crate / rel, dest)
        rows.append({"path": rel, "source_sha256": source_sha, "stage_sha256": sha(dest)})
    for key, rel in (("toml", "Cargo.toml"), ("lock", "Cargo.lock")):
        shutil.copy2(p.crate / rel, out[key])
    binding = {
        "schema": "logarithm-stage09-native-source-binding-v1",
        "helper_sha256": sha(HERE),
        "source": str(p.crate),
        "stage": str(out["stage"]),
        "changes": ["Cargo.toml", "Cargo.lock"],
        "change_rule": "explainer_kit/explainer_score paths point at W/repo-view; the 19 franken_manim lock "
                       "packages lose their git source line, as root's stage08 copy",
        "engine": {"w": str(p.w), "config": str(p.w / "engine-budget.toml"),
                   "config_sha256": W_PIN["engine-budget.toml"], "same_as": "root's logarithm-stage08-build01"},
        "script_sha256": PIN["script"],
        "units_sha256": PIN["units"],
        "direction_map": {"path": f"{CRATE}/docs/narration-direction-map-oliver16.json", "sha256": PIN["map16"]},
        "wording": facts["wording"],
        "stage08_refused": STAGE08,
        "files": rows,
    }
    with out["binding"].open("x") as handle:
        handle.write(json.dumps(binding, indent=2) + "\n")
    verify_stage(p, facts)
    return {"installed": len(rows), "stage": str(out["stage"]), "binding_sha256": sha(out["binding"]),
            "stage_cargo": facts["stage_cargo"]}


def verify_stage(p, facts=None):
    if facts is None:
        facts, toml_b, lock_b = check_all(p)
    else:
        toml_b, lock_b = derive(p)
    out = outputs(p)
    now = tree(out["stage"])
    need(set(now) == set(facts["crate"]), f"installed stage files differ: {sorted(set(now) ^ set(facts['crate']))[:3]}")
    for rel, digest in facts["crate"].items():
        expected = {"Cargo.toml": hashlib.sha256(toml_b).hexdigest(),
                    "Cargo.lock": hashlib.sha256(lock_b).hexdigest()}.get(rel, digest)
        need(now[rel] == expected, f"installed {rel} is not the bound bytes")
    pinned(out["toml"], PIN["cargo_toml"], "stage09 original Cargo.toml")
    pinned(out["lock"], PIN["cargo_lock"], "stage09 original Cargo.lock")
    binding = json.loads(out["binding"].read_text())
    need(binding["helper_sha256"] == sha(HERE), "the binding was written by another version of this helper")
    need({r["path"]: r["stage_sha256"] for r in binding["files"]} == now, "the binding's stage rows differ from the stage")
    need(binding["script_sha256"] == PIN["script"] and binding["engine"]["w"] == str(p.w), "binding names other bytes or engine")
    return facts


# ---------------------------------------------------------------- guarded job

def job_paths(p, attempt):
    job = f"{STAGE}-build{attempt}"
    return {"job": job, "script": p.into / f"{job}.sh", "inputs": p.into / f"{job}-inputs.json",
            "before": p.into / f"before-{job}-binary", "after": p.into / f"{job}-binary",
            "sums": p.into / f"{job}-binaries.sha256", "receipt": p.receipts / job}


def render_script(p, j, before_sha):
    q = shlex.quote
    if before_sha is None:
        preserve = ['[ ! -e "$B" ] || refuse "a shared binary appeared after this script was written"']
    else:
        preserve = [f'[ -f "$B" ] && [ "$(sha "$B")" = {before_sha} ] || refuse "the shared binary is not the one this script preserves"',
                    'cp -p "$B" "$BEFORE"',
                    f'[ "$(sha "$BEFORE")" = {before_sha} ] || refuse "the preserved copy differs"']
    lines = [
        f"# {j['job']}: logarithm stage09 native tests, binary and strict Clippy; written by prepare_stage09_native.py",
        f"# (helper sha256 {sha(HERE)}). Engine: root's W stage through W/engine-budget.toml, as logarithm-stage08-build01.",
        "# Launch only through the guard (it sets CARGO_TARGET_DIR); every path below is pinned by the job's inputs.",
        "set -eu",
        'refuse() { echo "REFUSED: $1" >&2; exit 1; }',
        "sha() { shasum -a 256 \"$1\" | cut -d ' ' -f 1; }",
        f"W={q(str(p.w))}",
        f"D={q(str(p.into))}",
        f"R={q(str(REPO))}",
        f"T={q(str(p.target))}",
        '[ "${CARGO_TARGET_DIR:-}" = "$T" ] || refuse "CARGO_TARGET_DIR is not the guard\'s target"',
        "export CARGO_INCREMENTAL=0",
        "export RCH_SHIM_LOCAL_IDE=1",
        'export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"',
        'cd "$R"',
        f'M="$D/{STAGE}/Cargo.toml"',
        'B="$CARGO_TARGET_DIR/debug/logarithm_mysteries"',
        f'BEFORE="$D/{j["before"].name}"',
        f'AFTER="$D/{j["after"].name}"',
        f'SUMS="$D/{j["sums"].name}"',
        'for f in "$BEFORE" "$AFTER" "$SUMS"; do [ ! -e "$f" ] || refuse "$f exists"; done',
        *preserve,
        f'python3 -I -B "$R/{LOCK_CHECK_REL}" "$D/{STAGE}-original-Cargo.lock" "$D/{STAGE}/Cargo.lock"',
        'cargo test --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets',
        'cargo build --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --bin logarithm_mysteries',
        'cp -p "$B" "$AFTER"',
        'cargo clippy --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets --no-deps -- -D warnings',
        'cmp "$B" "$AFTER" || refuse "Clippy changed the shared binary"',
        f'(cd "$D" && shasum -a 256 {"" if before_sha is None else q(j["before"].name) + " "}{q(j["after"].name)}) > "$SUMS"',
        '"$AFTER" chapters',
    ]
    return "\n".join(lines) + "\n"


def input_rows(p, j, facts):
    paths = [j["script"], *(v for k, v in outputs(p).items() if k != "stage")]
    paths += [p.stage / rel for rel in sorted(facts["crate"])]
    for sub in ("engine", "repo-view", "logarithm-stage08"):
        paths += [p.w / sub / rel for rel in sorted(tree(p.w / sub))]
    paths += [p.w / rel for rel in W_PIN if "/" not in rel]
    paths += [p.candidate / rel for rel in sorted(tree(p.candidate))]
    paths += [p.s9 / "bound-plan" / rel for rel in sorted(tree(p.s9 / "bound-plan"))]
    paths += [p.s9 / "root-integration-receipt01.json", p.white / "SHA256SUMS", HERE]
    paths += [REPO / rel for rel in TOOL_PIN]
    paths += [p.target / "geometry-kit01" / rel for rel in FILM_BINARIES if (p.target / "geometry-kit01" / rel).is_file()]
    need(len(set(paths)) == len(paths), "an input row repeats")
    return [{"path": str(path), "sha256": sha(path)} for path in paths]


def write_script(p, attempt):
    facts = verify_stage(p)
    j = job_paths(p, attempt)
    for key in ("script", "inputs", "before", "after", "sums", "receipt"):
        need(not j[key].exists() and not j[key].is_symlink(), f"{j[key]} exists: use a new attempt number")
    binary = p.shared_binary()
    before_sha = sha(binary) if binary.is_file() else None
    need(before_sha is not None or not binary.exists(), f"{binary} is not a regular file")
    with j["script"].open("x") as handle:
        handle.write(render_script(p, j, before_sha))
    rows = input_rows(p, j, facts)
    with j["inputs"].open("x") as handle:
        handle.write(json.dumps({"files": rows}, indent=2) + "\n")
    launch = (f"python3 -I -B {GUARD_REL} --phase native --job {j['job']} "
              f"--inputs {j['inputs']} -- sh {j['script']}")
    return {"job": j["job"], "script_sha256": sha(j["script"]), "inputs_sha256": sha(j["inputs"]),
            "input_rows": len(rows), "preserves": before_sha or "absent (nothing to preserve)",
            "preserves_stage08_binary": before_sha == STAGE08_BINARY, "launch_from_repo_root": launch}


def verify_run(p, attempt):
    facts = verify_stage(p)
    j = job_paths(p, attempt)
    receipt = json.loads((j["receipt"] / "resource-receipt.json").read_text())
    need(receipt.get("phase") == "native" and receipt.get("admitted") is True, "the job was not admitted as native")
    need(receipt.get("command") == ["sh", str(j["script"])], f"the receipt ran {receipt.get('command')}")
    need(receipt.get("exit") == 0 and not receipt.get("stopped_for_resource_limit") and "error" not in receipt,
         f"the job did not pass: exit {receipt.get('exit')}, error {receipt.get('error')}")
    need(receipt.get("inputs_sha256") == receipt.get("final_inputs_sha256") == sha(j["inputs"]),
         "the guard's input hashes do not match this attempt's inputs file")
    need(json.loads(j["inputs"].read_text())["files"][0]["sha256"] == sha(j["script"]), "the script moved after launch")
    log = (j["receipt"] / "command.log").read_text(errors="replace")
    need("PASS: the 19 packages are path copies, every other record is identical" in log, "lock check did not pass")
    results = re.findall(r"^test result: .*$", log, re.M)
    need(len(results) == 1 and results[0].startswith(
        f"test result: ok. {TESTS} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"),
        f"test results: {results}")
    tail = [line for line in log.splitlines() if line.strip()][-len(CHAPTERS):]
    need(tail == CHAPTERS, f"the chapters listing ends {tail[-3:]}")
    sums = dict(reversed(line.split("  ", 1)) for line in j["sums"].read_text().splitlines())
    need(sums.get(j["after"].name) == sha(j["after"]), "the stage09 binary copy moved")
    if j["before"].exists():
        need(sums.get(j["before"].name) == sha(j["before"]), "the preserved binary moved")
    return {"job": j["job"], "exit": 0, "tests": f"{TESTS} passed", "clippy": "-D warnings, exit 0 (set -e)",
            "chapters": len(CHAPTERS), "stage09_binary": sums[j["after"].name],
            "preserved_binary": sums.get(j["before"].name, "none"),
            "shared_binary_now_equals_stage09_copy": p.shared_binary().is_file() and sha(p.shared_binary()) == sha(j["after"]),
            "not_claimed": "no render, scene review, listening, fine sync or AV acceptance"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["check", "install", "script", "verify"])
    parser.add_argument("--w", required=True)
    parser.add_argument("--into")
    parser.add_argument("--attempt")
    for hidden in ("--el", "--white", "--receipts", "--target-root"):
        parser.add_argument(hidden, help="checks only: a scratch copy in place of the default")
    args = parser.parse_args(argv)
    if args.command != "check" and not args.into:
        parser.error(f"{args.command} needs --into")
    if args.command in ("script", "verify") and not re.fullmatch(r"\d\d", args.attempt or ""):
        parser.error(f"{args.command} needs --attempt NN")
    try:
        p = Paths(args)
        if args.command == "check":
            facts, _, _ = check_all(p)
            facts.pop("crate")
            result = {"check": "PASS", **facts}
        elif args.command == "install":
            result = {"install": "PASS", **install(p)}
        elif args.command == "script":
            result = {"script": "WRITTEN", **write_script(p, args.attempt)}
        else:
            result = {"verify": "PASS", **verify_run(p, args.attempt)}
    except Refused as refusal:
        print(f"REFUSED: {refusal}")
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
