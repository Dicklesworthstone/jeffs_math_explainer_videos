"""Write probe-plate-lab.patch: the plan changes the bounded 4K probe needs, on top of patch B's examples/plate_lab.rs.

    python3 -I -B .../patches/make_probe_patch.py SOURCE NEW_OUT     (from the repo root)

SOURCE is the lab's plate_lab.rs, and must hash to B's bytes. NEW_OUT is the edited copy, in scratch. The patch is the diff
of the two with the corpus paths in its header. Only the lab changes: no film source, no engine limit.
"""
import difflib
import hashlib
import pathlib
import sys

B_BYTES = "37d9d3386c24ae1d1ad19edef7735cd8c8af49453643bcb8ae674db1bf764f9e"
TARGET = "explainer_corpus/complex_analysis_visual/examples/plate_lab.rs"
EDITS = [
    # two more of the film's plates
    (
        """fn residue_bump_plane() -> Plane {""",
        """fn lacunary_plane() -> Plane {
    Plane::new(v(-3.0, -0.35), 5.4, (-1.05, 1.05), (-1.05, 1.05))
}
fn painted_plane() -> Plane {
    Plane::new(v(-2.8, -0.25), 5.4, (-2.0, 2.0), (-2.0, 2.0))
}
fn residue_bump_plane() -> Plane {""",
    ),
    (
        """const SUBJECTS: [Subject; 9] = [""",
        """const SUBJECTS: [Subject; 11] = [""",
    ),
    (
        """    Subject {
        key: "openbump",
        func: Func::Bump,
        plane: opening_bump_plane,
        enhance: 1.0,
    },
];""",
        """    Subject {
        key: "openbump",
        func: Func::Bump,
        plane: opening_bump_plane,
        enhance: 1.0,
    },
    Subject {
        key: "lacunary",
        func: Func::Lacunary,
        plane: lacunary_plane,
        enhance: 1.0,
    },
    Subject {
        key: "abs2",
        func: Func::AbsSquare,
        plane: painted_plane,
        enhance: 1.0,
    },
];""",
    ),
    # the one plate where averaging a cell looked closer to truth
    (
        """const CANDIDATES: [Candidate; 11] = [""",
        """const CANDIDATES: [Candidate; 12] = [""",
    ),
    (
        """    candidate("cells2", PLATE_CELLS2),
""",
        """    candidate("cells2", PLATE_CELLS2),
    candidate(
        "cells2_p3",
        PlateStyle {
            samples: 2,
            ..PLATE_CELLS2
        },
    ),
""",
    ),
    # the plan: cells4 cannot run, and the overlaps are decided
    (
        """    add(Kind::Still, &["z2", "sinrecip"], &["cells4"]);
""",
        """    add(Kind::Still, &["lacunary"], &["now", "tile", "rows2", "cells2"]);
    add(Kind::Still, &["abs2"], &["now", "cells2"]);
    add(Kind::Still, &["sinrecip"], &["cells2_p3"]);
""",
    ),
    (
        """    add(Kind::Fade, &["z"], &["now", "overlap", "overlap06", "tile", "blend"]);
""",
        # blend's fade already matched tile's at 720p; the fallback rows2 has never faded
        """    add(Kind::Fade, &["z"], &["now", "tile", "rows2", "cells2"]);
""",
    ),
    (
        """    add(Kind::Repaint, &["z2"], &["now", "blend", "rows2"]);
""",
        """    add(Kind::Repaint, &["z2"], &["now", "blend", "rows2", "cells2"]);
""",
    ),
    (
        """        assert_eq!(total, 67 + 77 + 9);
""",
        """        assert_eq!(total, 72 + 70 + 12);
""",
    ),
    (
        """        assert!(continuing.contains("Plane::new(v(-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5))"));
""",
        """        assert!(continuing.contains("Plane::new(v(-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5))"));
        assert!(continuing.contains("Plane::new(v(-3.0, -0.35), 5.4, (-1.05, 1.05), (-1.05, 1.05))"));
        let one_point = include_str!("../src/chapters/one_point.rs");
        assert!(one_point.contains("Plane::new(v(-2.8, -0.25), 5.4, (-2.0, 2.0), (-2.0, 2.0))"));
""",
    ),
]


def main(source, new_out):
    text = pathlib.Path(source).read_text(encoding="utf-8")
    digest = hashlib.sha256(text.encode()).hexdigest()
    if digest != B_BYTES:
        sys.exit(f"source is {digest}, not patch B's bytes")
    edited = text
    for old, new in EDITS:
        if edited.count(old) != 1:
            sys.exit(f"{edited.count(old)} matches for: {old[:70]}")
        edited = edited.replace(old, new)
    pathlib.Path(new_out).write_text(edited, encoding="utf-8")
    diff = difflib.unified_diff(text.splitlines(keepends=True), edited.splitlines(keepends=True), f"a/{TARGET}", f"b/{TARGET}", n=3)
    out = pathlib.Path(__file__).resolve().parent / "probe-plate-lab.patch"
    out.write_text("".join(diff), encoding="utf-8")
    print(f"{out.name}: {sum(1 for _ in out.open())} lines, sha256 {hashlib.sha256(out.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
