"""Run the pure helpers that the real-Stage tests and the lab rely on, lifted from the candidate's source. No fmn needed.

    python3 -I -B .../complex-color-repair03/checks/pure_helpers.py SCRATCH_DIR

draw.rs's share_of and plate_lab.rs's same_colours decide whether a lane still holds its colours. Their tests sit in
files that need the engine, which cannot link here. This cuts the two functions out of the candidate by name, compiles
them on their own with rustc --test, and asks each to pass good lanes and fail planted ones. It proves those two
functions alone: the scenes around them have not run.
"""
import pathlib
import subprocess
import sys

BUNDLE = pathlib.Path(__file__).resolve().parent.parent
CRATE = BUNDLE / "candidate/explainer_corpus/complex_analysis_visual"
scratch = pathlib.Path(sys.argv[1]).resolve()
scratch.mkdir(parents=True, exist_ok=True)


def cut(text, head):
    """The function starting at `head`, through the closing brace at its own indent."""
    start = text.index(head)
    indent = text[text.rfind("\n", 0, start) + 1 : start]
    end = text.index(f"\n{indent}}}\n", start) + len(indent) + 3
    return text[start:end]


share = cut((CRATE / "src/chapters/draw.rs").read_text(), "fn share_of(")
same = cut((CRATE / "examples/plate_lab.rs").read_text(), "fn same_colours(")
source = share + "\n" + same + r'''
#[test]
fn share_of_reads_a_clean_fade() {
    let full: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 1.0], [0.7, 0.2, 0.1, 1.0]].concat();
    assert_eq!(share_of(&full, &full), Some(1.0));
    let half: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.5]].concat();
    assert_eq!(share_of(&full, &half), Some(0.5));
}

#[test]
fn share_of_fails_every_planted_loss() {
    let full: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 1.0], [0.7, 0.2, 0.1, 1.0]].concat();
    let flat: Vec<f32> = [[0.4, 0.5, 0.6, 0.0], [0.4, 0.5, 0.6, 1.0], [0.4, 0.5, 0.6, 1.0]].concat();
    assert_eq!(share_of(&full, &flat), None, "flattened");
    let uneven: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.6]].concat();
    assert_eq!(share_of(&full, &uneven), None, "uneven alpha");
    let leak: Vec<f32> = [[0.1, 0.2, 0.3, 0.1], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.5]].concat();
    assert_eq!(share_of(&full, &leak), None, "clear lead-in painted");
    assert_eq!(share_of(&full, &full[..8]), None, "wrong length");
}

#[test]
fn same_colours_reads_and_fails() {
    let a = vec![vec![0.1, 0.2, 0.3, 0.0, 0.4, 0.5, 0.6, 1.0]];
    assert!(same_colours(&a, &a, None));
    let gone = vec![vec![0.1, 0.2, 0.3, 0.0, 0.4, 0.5, 0.6, 0.0]];
    assert!(!same_colours(&a, &gone, None));
    assert!(same_colours(&a, &gone, Some(0.0)));
    let flat = vec![vec![0.4, 0.5, 0.6, 0.0, 0.4, 0.5, 0.6, 1.0]];
    assert!(!same_colours(&a, &flat, None));
    assert!(!same_colours(&a, &flat, Some(1.0)));
}
'''
rs = scratch / "pure_helpers.rs"
rs.write_text(source)
binary = scratch / "pure_helpers"
for command in (["rustc", "--edition", "2024", "--test", "-A", "dead_code", str(rs), "-o", str(binary)], [str(binary)]):
    r = subprocess.run(command, capture_output=True, text=True)
    print((r.stdout + r.stderr).replace(str(scratch), "<scratch>"))
    if r.returncode:
        sys.exit(1)
