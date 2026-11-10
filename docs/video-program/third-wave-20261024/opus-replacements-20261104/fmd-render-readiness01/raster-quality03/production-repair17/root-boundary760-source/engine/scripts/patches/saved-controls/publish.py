"""One-shot, hash-checked publication of the two saved control feature patches.

This is transfer/commit verification, not a native-test runner. The feature
patches were exercised previously on the f9e970fd native extension with the
three changed Python adapters; that was not a fresh final-tree wheel. The
large bridge file is patched in place so unrelated upstream tests survive.
"""
from pathlib import Path
import gzip
import hashlib
import subprocess

HERE = Path(__file__).resolve().parent
PREFIX = "crates/fmn-python/"
EVENTS = PREFIX + "python/fmn_python/control_events.py"
STEPS = [
    {
        "parts": [f"0001.patch.gz.{i:02}" for i in range(3)],
        "gzip": True,
        "sha256": "c4b9d10c4ba92d6d29dbe51ce5c17c231b4c50d8b7b107daeca1177a5622e518",
        "before": {
            PREFIX + "python/fmn_python/color_sliders.py": "df788bedb82a7675f97759ff9230adf8cf5897c9",
            EVENTS: "281df7712b346a5135a2a4f5f4dd51a89b42fc40",
            PREFIX + "python/fmn_python/tracker_lifecycle.py": "76737e860577ec27f726ea175c9d5007ad540681",
            PREFIX + "tests/tracker_lifecycle.py": "b5d8925f5e245b75e945584696dabea02d03b92e",
        },
        "after": {
            PREFIX + "python/fmn_python/color_sliders.py": "a383fc270732f9ea7292e6ec6df6e85e5a90354e",
            EVENTS: "e817e61263fd20c6a1377a56d382bccb7db30786",
            PREFIX + "python/fmn_python/tracker_lifecycle.py": "fcbf682f66ec440988010a6866ff1dcbc9470298",
            PREFIX + "tests/tracker_lifecycle.py": "5b0bd5b031792965e98f158a1adbcf4d76b146a4",
        },
        "message": """feat(python): construct live sliders and toggles through public native-backed shapes

Publish saved local feature b60e12a7dbada9e0195c2db8b43469c592b4f6ec unchanged.
Real RoundedRectangle/Circle/Line and Rectangle children are available before
cooperative tracker hooks. Forward public primitive options, accept finite
ranges outside the old interval, retain color-bank defaults, and replace the
unpickleable stock updater. Add sixteen tracker lifecycle cases.

Prior native evidence: 27/29 tracker methods, 11/11 interaction and 13/13 color
bank cases pass; three pre-adoption-view assertions in two tracker methods
fail on the old f9e970fd extension, including an unchanged baseline failure.
14 event and 20 bank modeled cases pass. Three changing PNGs equal independent
native primitives at 1/4/16 workers. No fresh final-tree native build, workspace
or certification claim. This publication verifies source hashes and syntax,
not new native acceptance. No PR, force push, gate weakening or bead closure.

Original-Local-Commit: b60e12a7dbada9e0195c2db8b43469c592b4f6ec
Relates-to: fm-c1up; fm-5wq.13""",
    },
    {
        "parts": [f"checkbox.{i:02}.patch" for i in range(4)],
        "gzip": False,
        "sha256": "a31abd12a03d85877f3b5136c1fdbe39ca6f7b5d849ae1bebfbeba131e0a33e5",
        "before": {
            EVENTS: "e817e61263fd20c6a1377a56d382bccb7db30786",
            PREFIX + "tests/bridge.py": "6e8e9277c4b080819a94eb510564569b0c6e5793",
            PREFIX + "tests/control_interaction.py": "300247f3df37673aa64add673add35b5906338fb",
            "docs/CONTROL_INTERACTION.md": "b58877536b8e665a335fbae151a422e4d713f93d",
        },
        "after": {
            EVENTS: "84140fafd1a548ee6f05bbb748b5fbc6d7ddec85",
            PREFIX + "tests/bridge.py": "e013d105072d56410b483afdb0982a24fb6b2bc9",
            PREFIX + "tests/control_interaction.py": "7803af03e547348de16cb92116c0c10d8b016611",
            "docs/CONTROL_INTERACTION.md": "3fa082943b9bf523984639140bb8b96ea52f5270",
        },
        "message": """feat(python): compose authorable checkbox marks from live native line families

Publish saved local feature 531c9dc293cf2325ec0456042705f7b19f85b014 with its
unchanged implementation, tests and docs. Preserve intervening bridge tests:
apply the saved checkbox hunks to the independently verified current preimage.
Use public Rectangle and two-Line VGroup composition, retain authored factory
objects, expose children to tracker hooks, and preserve the live box's camera
lock across toggles. Forward public primitive options with named diagnostics.

Prior evidence: 23/23 native interaction cases, 13/13 native color-bank cases,
both affected bridge sections and 14 event plus 20 bank modeled cases pass.
Three changing PNGs under a moved/scaled camera equal independent native line
compositions at 1/4/16 workers. The pre-feature adapter fails eight assertions
and three cases with errors. All original interaction tests remain unchanged.
The older f9e970fd extension still fails three pre-adoption-view assertions in
two tracker methods; no claim those pass or that newer native code was tested.
No fresh final-tree wheel, full-workspace or certification claim. Publication
checks exact source hashes and syntax, not additional native acceptance.
No PR, force push, new runtime dependency, removed test or bead closure.

Original-Local-Commit: 531c9dc293cf2325ec0456042705f7b19f85b014
Relates-to: fm-c1up; fm-5wq.13""",
    },
]


def git(*args, data=None, check=True):
    return subprocess.run(["git", *args], input=data, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=check)


def text(*args):
    return git(*args).stdout.decode().strip()


def matches(expected):
    return all(Path(path).is_file() and not Path(path).is_symlink()
               and text("hash-object", "--no-filters", path) == value
               for path, value in expected.items())


def verify(expected):
    if not matches(expected):
        raise RuntimeError("Source changed: refusing to publish unverified bytes")


def clean():
    if text("status", "--porcelain"):
        raise RuntimeError("Publication requires a clean worktree and index")


def main():
    clean()
    git("fetch", "origin", "main")
    git("merge", "--ff-only", "origin/main")
    final = {path: sha for step in STEPS for path, sha in step["after"].items()}
    if matches(final):
        print("Both exact feature states are already published; no write")
        return
    for step in STEPS:
        clean()
        if matches(step["after"]):
            print("Feature already present; continuing")
            continue
        verify(step["before"])
        payload = b"".join((HERE / name).read_bytes() for name in step["parts"])
        if step["gzip"]:
            payload = gzip.decompress(payload)
        if hashlib.sha256(payload).hexdigest() != step["sha256"]:
            raise RuntimeError("Patch checksum mismatch; refusing to apply")
        git("apply", "--check", "-", data=payload)
        git("apply", "-", data=payload)
        verify(step["after"])
        changed = set(text("diff", "--name-only").splitlines())
        if changed != set(step["after"]):
            raise RuntimeError("Patch changed paths outside its reviewed scope")
        git("diff", "--check")
        for path in changed:
            if path.endswith(".py"):
                compile(Path(path).read_bytes(), path, "exec")
        git("add", "--", *sorted(changed))
        if set(text("diff", "--cached", "--name-only").splitlines()) != changed:
            raise RuntimeError("Unexpected staged paths")
        for path, sha in step["after"].items():
            if text("rev-parse", ":" + path) != sha:
                raise RuntimeError("Staging changed the validated bytes")
        git("-c", "user.name=github-actions[bot]", "-c",
            "user.email=41898282+github-actions[bot]@users.noreply.github.com",
            "commit", "-m", step["message"])
        for attempt in range(3):
            verify(step["after"])
            result = git("push", "origin", "HEAD:refs/heads/main", check=False)
            if result.returncode == 0:
                print("Published", text("rev-parse", "HEAD"), flush=True)
                break
            if attempt == 2:
                raise RuntimeError(result.stderr.decode())
            # Only our unpushed commit is rebased. Conflicts abort; no force.
            git("fetch", "origin", "main")
            git("rebase", "origin/main")
            verify(step["after"])
        clean()
    verify(final)
    git("fetch", "origin", "main")
    for path, sha in final.items():
        if text("rev-parse", "origin/main:" + path) != sha:
            raise RuntimeError("Published branch changed a feature file")
    print("Verified both commits on origin/main", flush=True)


if __name__ == "__main__":
    main()
