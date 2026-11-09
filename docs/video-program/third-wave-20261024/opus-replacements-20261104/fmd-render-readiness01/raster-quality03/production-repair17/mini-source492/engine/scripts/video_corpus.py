#!/usr/bin/env python3
"""VIDEO_CORPUS.lock tooling — fm-rqc (§15.3-15.4, R13).

The corpus gate's machinery: `VIDEO_CORPUS.lock` pins an exact scene
allowlist from the pinned `3b1b/videos` tree — scene names, module blob
hashes, era, per-scene attribution, and the asset manifest — under the
CC BY-NC-SA fixture policy (private gallery fixtures, public permissive
primitive corpus, per-scene attribution). The G4a criterion the lock
serves: the enumerated scenes run source-unedited through fmn-python
under the documented shims and pass structural assertions plus
Look-Gallery review — never pixel diffs.

Deterministic and stdlib-only, like scripts/harvest_tex_corpus.py: the
same pins reproduce the same lock byte-for-byte (sorted iteration
everywhere, no timestamps).

Subcommands
-----------
scan    Advisory census of the pinned videos tree: every Scene subclass,
        scored for self-containedness (imports, asset references, custom
        GLSL, TeX usage). Guides curation; commits nothing.
        `--self-contained` restricts to modules with no signals at all.
verify  Regenerate the lock in memory from the SEED tables plus the
        pinned checkout and byte-compare against the committed
        VIDEO_CORPUS.lock. Exit 0 clean; exit 1 drift; exit 2 when the
        gitignored checkout is absent (with the exact clone commands).
emit    Write VIDEO_CORPUS.lock. Refused unless VIDEO_CORPUS_UPDATE=1 —
        the blessing ritual mirrors the coverage ratchet's.

The pins are read from SUITE.lock `[reference]`; the checkout convention
is scripts/videos_ref (gitignored), exactly as G0-4 established.
"""

from __future__ import annotations

import ast
import hashlib
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LOCK_PATH = REPO / "VIDEO_CORPUS.lock"
VIDEOS_REF = REPO / "scripts" / "videos_ref"

# ---------------------------------------------------------------------------
# The curated seed allowlist (fm-rqc tranche 1).
#
# Selection rule (mechanical, from `scan`): modern-era modules whose only
# foreign import is the era shim `manim_imports_ext` (itself resolving to
# manimlib plus the in-tree `custom/` package), with no asset-file string
# literals and no custom-GLSL signals at module scope. Each entry names
# one Scene subclass the G4a harness drives source-unedited. TeX-bearing
# modules are deliberately included: a scene whose TeX outruns the
# current fmd-math tier is re-marked pending-with-named-constructs by the
# harness, feeding the W6 ratchet — that machinery needs seed exercise.
#
# status vocabulary (the lock's whole vocabulary):
#   allowlisted                    in the gallery; the harness must run it
#   pending-with-named-constructs  TeX outran the fmd-math tier; the named
#                                  constructs feed the W6 ratchet
#   excluded                       considered and rejected, reason recorded
#
# Absence from the lock is NO claim — only exclusion rows are deliberate
# rejections.
# ---------------------------------------------------------------------------
SEED_SCENES: tuple[tuple[str, str, str, str], ...] = (
    # (scene class, module path, era, curation note)
    ("Cubic", "_2022/quintic/roots_and_coefs.py", "2022", "root/coef planes via RootCoefScene.setup; TeX-bearing module"),
    ("QuadraticFormula", "_2022/quintic/roots_and_coefs.py", "2022", "root/coef planes, custom plane configs; TeX-bearing module"),
    ("WaveMachineDemo", "_2023/optics_puzzles/wave_machine.py", "2023", "3D wave kinematics; no TeX"),
    ("MaxProcess", "_2024/puzzles/max_rand.py", "2024", "probability process; TeX-bearing (19 call sites in module)"),
    ("GroverPreview", "_2025/colliding_blocks_v2/grover.py", "2025", "state-space preview; light TeX"),
    ("FlattenCone", "_2025/guest_videos/euclid.py", "2025", "surface development; no TeX"),
    ("SquareOnASphere", "_2025/guest_videos/euclid.py", "2025", "spherical geometry; no TeX"),
    ("BeamSplitter", "_2025/grover/polarization.py", "2025", "optics diagram; light TeX"),
)

# Scenes that were allowlisted in an earlier curation pass and had to be
# withdrawn: they stay in the lock as excluded scene rows so the reversal
# is on the record (R13), not silently disappeared.
SEED_EXCLUDED_SCENES: tuple[tuple[str, str, str, str], ...] = (
    ("FlowerSymmetries", "_2022/galois/groups.py", "2022", "loads an ImageMobject from an out-of-tree Dropbox path; unobtainable asset — the harness proved it, the extension heuristic had missed it"),
    ("PoolTableReflections", "_2023/standup_maths/pool.py", "2023", "loads ImageMobject('pool_table'), an extensionless asset-dir reference outside the tree; caught by the harness, now flagged by scan's constructor signal"),
)

# Modules considered for the seed and deliberately rejected. These are
# recorded (R13: reasons on the record), not merely omitted.
SEED_EXCLUSIONS: tuple[tuple[str, str], ...] = (
    ("_2022/borwein/main.py", "not seed-tier: 16 scenes, 73 TeX call sites; revisit for the full gallery once the structural harness is proven on the seed"),
    ("_2022/some2/announcement.py", "meta/announcement content; no mathematical-rendering value for the gallery"),
    ("_2025/colliding_blocks_v2/supplements.py", "38 supplement scenes of channel meta-content; not gallery material"),
)

# Era-level exclusions (R13). Pre-2020 trees predate the Reference-era
# manimlib API; running them source-unedited is not a current claim.
# Revisit trigger: per-scene era-shim documentation.
ERA_EXCLUSIONS: tuple[tuple[str, str], ...] = (
    ("_2015 _2016 _2017 _2018 _2019", "out-of-era: pre-Reference manimlib API surface; revisit per-scene with documented era shims"),
    ("_2020 _2021", "transition-era API; unreviewed for the seed; revisit per-scene before the full gallery"),
)

# The documented shims (§15.3): what the harness virtualizes so scene
# source stays unedited. The import shim's entry blob is hash-pinned;
# its `custom/**` closure is covered by the tree pin itself.
SHIMS: tuple[tuple[str, str, str], ...] = (
    # (name, mechanism, hashed file relative to videos_ref or "-")
    ("import-virtualization", "manimlib import surface served by fmn-python; the era shim manim_imports_ext plus its in-tree custom/** closure import-resolve inside the pinned tree", "manim_imports_ext.py"),
    ("asset-path-virtualization", "corpus asset references resolve through the AssetFetcher/cache capability, never the host filesystem; the seed allowlist requires no assets", "-"),
    ("fonts", "bundled OFL faces substitute for the era's system fonts (documented divergence; Look-Gallery reviewed)", "-"),
    ("kwarg-vintage", "era renames between a scene's vintage and the Reference pin are aliased narrowly and per-kwarg, never wholesale: set_stroke/set_backstroke background= (pre-2023 manimgl) aliases to the pin's behind= (R13 ruling 2026-08-10, fm-d3gt tranche 19: Cubic)", "-"),
)


def read_reference_pins() -> dict[str, str]:
    pins: dict[str, str] = {}
    in_ref = False
    for line in (REPO / "SUITE.lock").read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_ref = stripped == "[reference]"
            continue
        if not in_ref or not stripped or stripped.startswith("#"):
            continue
        fields = stripped.split("\t")
        if len(fields) >= 2:
            pins[fields[0]] = fields[1]
    return pins


def require_checkout() -> None:
    if not (VIDEOS_REF / ".git").exists():
        pins = read_reference_pins()
        sys.stderr.write(
            "videos checkout missing. Recreate it exactly as G0-4 did:\n"
            "  git clone https://github.com/3b1b/videos scripts/videos_ref\n"
            f"  git -C scripts/videos_ref checkout {pins['3b1b/videos']}\n"
        )
        raise SystemExit(2)


def blob_sha256(rel: str) -> str:
    return hashlib.sha256((VIDEOS_REF / rel).read_bytes()).hexdigest()


def render_lock() -> str:
    """The complete lock, deterministically, from the tables + checkout."""
    pins = read_reference_pins()
    out: list[str] = []
    w = out.append
    w("# VIDEO_CORPUS.lock — the corpus gate's pinned scene allowlist (fm-rqc, §15.3-15.4)")
    w("#")
    w("# The G4a criterion this lock serves: every allowlisted scene runs")
    w("# SOURCE-UNEDITED through the fmn-python portal under the [shims]")
    w("# documented below, and passes structural assertions (object counts,")
    w("# timings, bounding envelopes) plus Look-Gallery review. Pixel diffs")
    w("# are deleted by design (§4).")
    w("#")
    w("# Fixture policy (CC BY-NC-SA): the 3b1b/videos tree is © Grant")
    w("# Sanderson, CC BY-NC-SA; gallery fixtures derived from it stay")
    w("# private; the public corpus is the permissive primitive set. Every")
    w("# scene row carries its in-tree provenance as attribution.")
    w("#")
    w("# Absence from this lock is NO claim. Exclusion rows are deliberate,")
    w("# reasoned rejections (R13). Regenerate only via:")
    w("#   VIDEO_CORPUS_UPDATE=1 python3 scripts/video_corpus.py emit")
    w("# Verify against the pinned checkout via:")
    w("#   python3 scripts/video_corpus.py verify")
    w("")
    w("[pins]")
    w("# Mirrors SUITE.lock [reference]; verify asserts the equality.")
    for name in ("3b1b/videos", "3b1b/manim"):
        w(f"{name}\t{pins[name]}")
    w("")
    w("[shims]")
    w("# name\tmechanism\tpinned-entry-blob (sha256 at the tree pin, or -)")
    for name, mechanism, hashed in SHIMS:
        digest = blob_sha256(hashed) if hashed != "-" else "-"
        w(f"{name}\t{mechanism}\t{digest}")
    w("")
    w("[scenes]")
    w("# scene\tmodule\tmodule-sha256\tera\tstatus\tattribution\tnote")
    statused = [(scene, module, era, "allowlisted", note) for scene, module, era, note in SEED_SCENES] + [
        (scene, module, era, "excluded", note)
        for scene, module, era, note in SEED_EXCLUDED_SCENES
    ]
    for scene, module, era, status, note in sorted(statused):
        attribution = f"3b1b/videos@{pins['3b1b/videos'][:12]} {module}"
        w(
            f"{scene}\t{module}\t{blob_sha256(module)}\t{era}\t{status}\t{attribution}\t{note}"
        )
    w("")
    w("[assets]")
    w("# path\tsha256\trequired-by — the seed allowlist requires none.")
    w("")
    w("[exclusions]")
    w("# module-or-era-set\treason")
    for module, reason in sorted(SEED_EXCLUSIONS):
        w(f"{module}\t{reason}")
    for eras, reason in ERA_EXCLUSIONS:
        w(f"{eras}\t{reason}")
    w("")
    return "\n".join(out)


# ---------------------------------------------------------------------------
# scan — the advisory census
# ---------------------------------------------------------------------------

ASSET_SUFFIXES = (
    ".png",
    ".jpg",
    ".jpeg",
    ".svg",
    ".gif",
    ".mp3",
    ".wav",
    ".mp4",
    ".obj",
    ".glb",
)
TEX_CONSTRUCTORS = {"Tex", "TexText", "OldTex", "OldTexText", "SingleStringTex"}
# Constructors that resolve extensionless names through asset directories —
# an asset dependency even when no literal carries a file suffix.
ASSET_CONSTRUCTORS = {"ImageMobject", "SVGMobject", "Sound"}

STDLIB_OK = {
    "manimlib",
    "numpy",
    "np",
    "math",
    "itertools",
    "functools",
    "random",
    "typing",
    "fractions",
    "collections",
    "copy",
    "operator",
    "os",
    "sys",
    "re",
    "string",
    "colorsys",
}


class ModuleFacts(ast.NodeVisitor):
    """One pass over a videos-tree module: scenes, imports, asset signals."""

    def __init__(self) -> None:
        self.scene_classes: list[str] = []
        self.imports: set[str] = set()
        self.asset_literals: set[str] = set()
        self.glsl_signals: set[str] = set()
        self.tex_calls = 0

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        for base in node.bases:
            name = base.id if isinstance(base, ast.Name) else (
                base.attr if isinstance(base, ast.Attribute) else ""
            )
            if name.endswith("Scene"):
                self.scene_classes.append(node.name)
                break
        self.generic_visit(node)

    def visit_Import(self, node: ast.Import) -> None:
        for alias in node.names:
            self.imports.add(alias.name.split(".")[0])

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        root = (node.module or "").split(".")[0]
        if node.level:
            root = f".{root}" if root else "."
        self.imports.add(root)

    def visit_Call(self, node: ast.Call) -> None:
        callee = node.func.id if isinstance(node.func, ast.Name) else (
            node.func.attr if isinstance(node.func, ast.Attribute) else ""
        )
        if callee in TEX_CONSTRUCTORS:
            self.tex_calls += 1
        if callee in ASSET_CONSTRUCTORS:
            self.asset_literals.add(f"call:{callee}")
        self.generic_visit(node)

    def visit_Constant(self, node: ast.Constant) -> None:
        if isinstance(node.value, str) and node.value.lower().endswith(ASSET_SUFFIXES):
            self.asset_literals.add(node.value)

    def visit_Attribute(self, node: ast.Attribute) -> None:
        if node.attr in ("shader_folder", "set_color_by_code"):
            self.glsl_signals.add(node.attr)
        self.generic_visit(node)


def scan(argv: list[str]) -> int:
    require_checkout()
    rows: list[tuple[str, str, str, str]] = []
    for path in sorted(VIDEOS_REF.rglob("*.py")):
        rel = path.relative_to(VIDEOS_REF).as_posix()
        if rel.startswith("."):
            continue
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        facts = ModuleFacts()
        facts.visit(tree)
        if not facts.scene_classes:
            continue
        foreign = sorted(
            i for i in facts.imports if i not in STDLIB_OK and not i.startswith(".")
        ) + sorted(i for i in facts.imports if i.startswith("."))
        flags: list[str] = []
        if foreign:
            flags.append("imports:" + ",".join(foreign))
        if facts.asset_literals:
            flags.append(f"assets:{len(facts.asset_literals)}")
        if facts.glsl_signals:
            flags.append("glsl")
        if facts.tex_calls:
            flags.append(f"tex:{facts.tex_calls}")
        clean = not foreign and not facts.asset_literals and not facts.glsl_signals
        verdict = "self-contained" if clean else "entangled"
        rows.append((verdict, rel, ";".join(facts.scene_classes), " ".join(flags) or "-"))
    only_clean = "--self-contained" in argv
    for verdict, rel, scenes, flags in rows:
        if only_clean and verdict != "self-contained":
            continue
        sys.stdout.write(f"{verdict}\t{rel}\t{scenes}\t{flags}\n")
    return 0


# ---------------------------------------------------------------------------
# verify / emit
# ---------------------------------------------------------------------------


def verify() -> int:
    require_checkout()
    expected = render_lock()
    if not LOCK_PATH.exists():
        sys.stderr.write("VIDEO_CORPUS.lock missing; run the emit ritual\n")
        return 1
    actual = LOCK_PATH.read_text(encoding="utf-8")
    if actual == expected:
        scene_count = sum(
            1
            for line in actual.splitlines()
            if line and not line.startswith(("#", "["))
            and "\tallowlisted\t" in line
        )
        sys.stdout.write(
            f"OK: VIDEO_CORPUS.lock reproduces byte-for-byte at the pins "
            f"({scene_count} allowlisted scenes)\n"
        )
        return 0
    exp_lines = expected.splitlines()
    act_lines = actual.splitlines()
    for index, (exp, act) in enumerate(zip(exp_lines, act_lines), start=1):
        if exp != act:
            sys.stderr.write(
                f"drift at line {index}:\n  committed: {act}\n  computed:  {exp}\n"
            )
            break
    else:
        sys.stderr.write(
            f"drift: line-count mismatch (committed {len(act_lines)}, "
            f"computed {len(exp_lines)})\n"
        )
    return 1


def emit() -> int:
    if os.environ.get("VIDEO_CORPUS_UPDATE") != "1":
        sys.stderr.write(
            "refusing to rewrite VIDEO_CORPUS.lock without VIDEO_CORPUS_UPDATE=1\n"
        )
        return 2
    require_checkout()
    LOCK_PATH.write_text(render_lock(), encoding="utf-8")
    sys.stdout.write(f"wrote {LOCK_PATH.relative_to(REPO)}\n")
    return 0


def main() -> int:
    if len(sys.argv) < 2 or sys.argv[1] not in ("scan", "verify", "emit"):
        sys.stderr.write(__doc__ or "")
        return 2
    if sys.argv[1] == "scan":
        return scan(sys.argv[2:])
    if sys.argv[1] == "verify":
        return verify()
    return emit()


if __name__ == "__main__":
    raise SystemExit(main())
