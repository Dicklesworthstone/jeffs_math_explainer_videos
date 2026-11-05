"""Scratch: per-finding patches from the frozen baseline, proved against the staged copies.

Each edit must match exactly once. Every order of the three patches must rebuild the staged
bytes, and each patch must apply alone. Nothing outside SP and OUT/patches is written.
Usage: lmr_patches01.py BASE STAGED OUT SP
"""
import filecmp
import itertools
import os
import shutil
import subprocess
import sys

BASE, STAGED, OUT, SP = sys.argv[1:5]
FILES = ["draw.rs", "complex_turns.rs", "lifting_loops.rs"]
REL = "src/chapters/"

PROBE_OLD = """    // a bare probe in the scene drives the tracker by the clock, so the bar
    // drains during the music's own gap rather than in a played animation
    let probe = stage.arena_mut().add(VMobject::new());"""
PROBE_NEW = """    // the pause line doubles as the probe: in the scene it drives the tracker by the
    // clock, so the bar drains during the music's own gap, and it fades with the bar
    let probe = pause_line(kit, stage, at)?;"""
AFTER_OLD = """    fade_out_all(stage, &[bar, probe], 0.25)
}
"""
AFTER_NEW = """    fade_out_all(stage, &[bar, probe], 0.25)
}

/// The line under every prediction bar, at the size of the fine print.
const PAUSE_WORDS: &str = "pause for more time";
const PAUSE_SIZE: f64 = 22.0;
/// From the bar's centre down to the line's centre.
const PAUSE_DROP: f64 = 0.27;

/// The pause line of a bar centred at `at`, detached.
fn pause_line(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let below = v(at[0], at[1] - PAUSE_DROP);
    kit.text(stage, PAUSE_WORDS, PAUSE_SIZE, Palette::dim(), below)
}
"""
TEST_OLD = """        assert_eq!(dx, 32.0 * 0.011);
        assert_eq!(w, 32.0 * 0.011 + 32.0 * 0.0095);
    }
}"""
TEST_NEW = """        assert_eq!(dx, 32.0 * 0.011);
        assert_eq!(w, 32.0 * 0.011 + 32.0 * 0.0095);
    }

    #[test]
    fn the_pause_line_stays_under_the_narrowest_bar() {
        probe_scene(|stage| {
            let kit = Kit::new()?;
            let line = pause_line(&kit, stage, ORIGIN)?;
            // the narrowest bar is 2.6 wide, so it runs ±1.3 about `at`
            let half = stage.get_width(line) / 2.0;
            assert!(half < 1.3 - 0.1, "half width {half}");
            // the bar's lower edge is at −0.035
            let top = stage.get_bounding_box_point(line, UP)[1];
            assert!(top < -0.035 - 0.1, "top {top}");
            Ok(())
        });
    }
}"""
GROUPS = {
    "O1-pause-line": [
        ("draw.rs", PROBE_OLD, PROBE_NEW),
        ("draw.rs", AFTER_OLD, AFTER_NEW),
        ("draw.rs", TEST_OLD, TEST_NEW),
        ("lifting_loops.rs",
         "predict(&kit, stage, v(Z.centre[0], 2.55), 2.6)?;",
         "predict(&kit, stage, v(read_x + 1.3, read_y - 1.4), 2.6)?;"),
    ],
    "O2-rounded-value": [
        ("complex_turns.rs",
         '&format!("real part ln {} = {}", glen, fact("complex.generic_ln_len")?),',
         '&format!("real part ln {} ≈ {}", glen, fact("complex.generic_ln_len")?),'),
    ],
    "O3-bar-off-the-axis": [
        ("complex_turns.rs",
         "predict(&kit, stage, v(OUT.centre[0], -2.75), 3.0)?;",
         "predict(&kit, stage, v(IN.centre[0] + 2.2, 1.85), 3.0)?;"),
    ],
}


def tree(name):
    path = os.path.join(SP, "lmr-trees", name)
    os.makedirs(os.path.join(path, REL))
    for f in FILES:
        shutil.copyfile(os.path.join(BASE, REL, f), os.path.join(path, REL, f))
    return path


os.makedirs(os.path.join(STAGED, REL))
for f in FILES:
    s = open(os.path.join(BASE, REL, f), encoding="utf-8", newline="").read()
    for group in GROUPS.values():
        for name, old, new in group:
            if name == f:
                assert s.count(old) == 1, (f, old[:50], s.count(old))
                s = s.replace(old, new)
    with open(os.path.join(STAGED, REL, f), "x", encoding="utf-8", newline="") as fh:
        fh.write(s)

patches = {}
os.makedirs(os.path.join(OUT, "patches"))
for name, edits in GROUPS.items():
    t = tree("edit-" + name)
    for f, old, new in edits:
        p = os.path.join(t, REL, f)
        s = open(p, encoding="utf-8").read()
        assert s.count(old) == 1, (name, f, old[:40], s.count(old))
        open(p, "w", encoding="utf-8").write(s.replace(old, new))
    text = ""
    for f in FILES:
        r = subprocess.run(
            ["diff", "-u", "--label", "a/" + REL + f, "--label", "b/" + REL + f,
             os.path.join(BASE, REL, f), os.path.join(t, REL, f)],
            capture_output=True, text=True)
        assert r.returncode in (0, 1), r.stderr
        text += r.stdout
    patch = os.path.join(OUT, "patches", name + ".patch")
    with open(patch, "x", encoding="utf-8") as fh:
        fh.write(text)
    patches[name] = os.path.abspath(patch)
    plus = sum(1 for x in text.splitlines() if x.startswith("+") and not x.startswith("+++"))
    minus = sum(1 for x in text.splitlines() if x.startswith("-") and not x.startswith("---"))
    print("%s: +%d -%d" % (name, plus, minus))


def apply(path, patch):
    return subprocess.run(["patch", "-p1", "-s", "--no-backup-if-mismatch", "-i", patch],
                          cwd=path, capture_output=True, text=True)


n = 0
for order in itertools.permutations(patches):
    n += 1
    t = tree("order-%d" % n)
    for name in order:
        r = apply(t, patches[name])
        assert r.returncode == 0, (order, name, r.stdout, r.stderr)
    for f in FILES:
        assert filecmp.cmp(os.path.join(t, REL, f), os.path.join(STAGED, REL, f), shallow=False), (order, f)
print("every order of the three patches rebuilds the staged bytes:", n, "orders")
for name, patch in patches.items():
    t = tree("alone-" + name)
    r = apply(t, patch)
    assert r.returncode == 0, (name, r.stdout, r.stderr)
    g = subprocess.run(["git", "apply", "--check", patch], cwd=tree("git-" + name), capture_output=True, text=True)
    print(name, "applies alone with patch -p1; git apply --check exit", g.returncode, g.stderr.strip()[:120])
