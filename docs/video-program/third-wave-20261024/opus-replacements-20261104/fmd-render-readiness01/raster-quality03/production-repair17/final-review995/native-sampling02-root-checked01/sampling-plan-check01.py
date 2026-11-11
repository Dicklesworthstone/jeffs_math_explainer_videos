"""Checks and frame lists for the combined-repair native sampling plan (read-only; never builds or renders).

  sources   PLAN SRC_ROOT                  P1: SRC_ROOT/src/chapters/{the_bump,loop_sums,mod,boundary}.rs hash to the
                                           plan's staged sources (exit 1 on any mismatch)
  timeline  PLAN CHAPTER RENDERED.tsv      P2: the rendered chapter timeline, with its line WAV directory replaced by
                                           the selected one, hashes to the selected timeline (exit 1 if not); on a
                                           mismatch prints the first differing non-path rows
  frames    PLAN CHAPTER [required|all]    local frame indices to extract, one per line
  select    PLAN CHAPTER [required|all]    the same frames as an ffmpeg select expression (eq(n,K)+...)
  expect    PLAN                           P3 expected frame counts per chapter
  self-test PLAN T01 T08 T09               run P2 on three timeline files (e.g. the local selected/preview ones)

usage: python3 -I -B sampling-plan-check01.py COMMAND PLAN.json ..."""
import hashlib
import json
import sys


def load(path):
    return json.load(open(path, encoding="utf-8"))


def chapter_row(plan, chapter):
    rows = [c for c in plan["chapters"] if c["chapter"] == chapter]
    if not rows:
        raise SystemExit(f"no chapter {chapter} in plan")
    return rows[0]


def sources(plan, root):
    bad = 0
    for rel, want in plan["sources"]["staged"].items():
        try:
            got = hashlib.sha256(open(f"{root.rstrip('/')}/{rel}", "rb").read()).hexdigest()
        except OSError as e:
            got = f"unreadable ({e})"
        ok = got == want
        bad += not ok
        print(f"{'OK ' if ok else 'BAD'} {rel} {got[:16]} want {want[:16]}")
    return 1 if bad else 0


def rewrite(text, new_dir):
    out = []
    for line in text.split("\n"):
        cols = line.split("\t")
        if cols[0] == "line":
            cols[-1] = new_dir + "/" + cols[-1].rsplit("/", 1)[1]
        out.append("\t".join(cols))
    return "\n".join(out)


def timeline(plan, chapter, path):
    row = chapter_row(plan, chapter)
    raw = open(path, "rb").read().decode("utf-8")
    got = hashlib.sha256(rewrite(raw, row["narration_dir_in_selected"]).encode("utf-8")).hexdigest()
    ok = got == row["selected_timeline_sha256"]
    print(f"{'OK ' if ok else 'BAD'} {chapter} timeline (narration dir rewritten) {got[:16]} want "
          f"{row['selected_timeline_sha256'][:16]}")
    if not ok:
        rows = [r.split("\t") for r in raw.split("\n")]
        frames = [r for r in rows if r[0] == "frames"]
        print("   frames row:", frames, "plan expects", row["frames"])
        for r in rows:
            if r[0] in ("line", "mark", "breath", "listen"):
                print("   ", "\t".join(r[:-1] if r[0] == "line" else r))
    return 0 if ok else 1


def frames(plan, chapter, tier):
    chapter_row(plan, chapter)
    return [s["local_frame"] for s in plan["samples"]
            if s["chapter"] == chapter and (tier == "all" or s["tier"] == "required")]


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    cmd, plan = sys.argv[1], load(sys.argv[2])
    args = sys.argv[3:]
    if cmd == "sources":
        return sources(plan, args[0])
    if cmd == "timeline":
        return timeline(plan, args[0], args[1])
    if cmd in ("frames", "select"):
        tier = args[1] if len(args) > 1 else "required"
        fs = frames(plan, args[0], tier)
        if cmd == "frames":
            print("\n".join(str(f) for f in fs))
        else:
            print("+".join(f"eq(n\\,{f})" for f in fs))
        return 0
    if cmd == "expect":
        for c in plan["chapters"]:
            print(c["chapter"], "frames", c["frames"], "master start", c["master_start_frame"])
        return 0
    if cmd == "self-test":
        rc = 0
        for chapter, path in zip(("01_the_bump", "08_loop_sums", "09_boundary"), args):
            rc |= timeline(plan, chapter, path)
        bogus = rewrite(open(args[2], encoding="utf-8").read(), "/x").replace("line\t09_9\t15577", "line\t09_9\t15578")
        got = hashlib.sha256(rewrite(bogus, chapter_row(plan, "09_boundary")["narration_dir_in_selected"])
                             .encode("utf-8")).hexdigest()
        caught = got != chapter_row(plan, "09_boundary")["selected_timeline_sha256"]
        print(f"{'OK ' if caught else 'BAD'} control: a timeline with 09_9 moved one frame is refused")
        return rc | (0 if caught else 1)
    print(__doc__)
    return 2


sys.exit(main())
