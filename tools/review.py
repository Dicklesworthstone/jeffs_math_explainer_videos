"""Assemble a persona review packet (plan §5.2):

    python3 -I tools/review.py <slug> storyboard|script|draft
    ./x <slug> review storyboard|script|draft

writes docs/reviews/<slug>/R<n>-packet.md: the stage's instructions, the
comprehension check, the findings format and the credit rules, then the
stage's artifacts inline, so a fresh reviewer needs nothing else. Each
reviewer also gets exactly one persona prompt from tools/review_prompts/.
Dispositions go in docs/reviews/<slug>/R<n>.md (fixed: commit / answered:
one sentence / deferred: bead id). Packets are regenerated, not committed.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PROMPTS = ROOT / "tools/review_prompts"
STAGES = {
    # stage: (round, reviewers, what to read)
    "storyboard": ("R1", ["maya_14", "dev_28", "ruth_45", "engine_feasibility", "creative_director"],
                   ["brief", "storyboard", "fact-sheet"]),
    "script": ("R2", ["maya_14", "dev_28", "ruth_45 (with ruth_audit)"], ["brief", "script", "storyboard", "fact-sheet"]),
    "draft": ("R3", ["maya_14", "dev_28", "ruth_45", "the owner"], ["script", "sheets"]),
}


def artifact(slug, name):
    """The artifact's path: the crate's docs/ once scaffolded, else docs/'s staging copy."""
    crate = ROOT / "explainer_corpus" / slug
    candidates = {
        "brief": [crate / "docs/brief.md", ROOT / f"docs/briefs/{slug}.md"],
        "storyboard": [crate / "docs/storyboard.md", ROOT / f"docs/storyboards/{slug}.md"],
        "fact-sheet": [crate / "docs/fact-sheet.md", ROOT / f"docs/facts/{slug}/fact-sheet.md"],
        "script": [crate / "script.tsv"],
    }[name]
    for p in candidates:
        if p.is_file():
            return p
    sys.exit(f"review: no {name} for {slug} (looked in {', '.join(str(c.relative_to(ROOT)) for c in candidates)})")


COMMON = """\
## What to do

1. Read your persona prompt (named in your instructions; all of them are in
   `tools/review_prompts/`) and become that reviewer.
2. Read every artifact below, in order, as that reviewer.
3. Write your review in exactly this shape:

   - **Comprehension check:** for every chapter, the one sentence you would
     say to a friend about what just happened and why it mattered. A wrong
     or empty sentence is a blocker for that chapter, whoever is "right".
     (Engine and creative reviewers: one sentence on the chapter's key
     picture instead.)
   - **Rubric:** per chapter, 1-5 for clarity, correctness, motivation,
     pacing, delight (persona reviewers only).
   - **Findings**, one table:

     `| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |`

     severity is one of blocker / major / minor / delight. Cite line ids
     (`04_3`). Every finding names a concrete fix.

## Credit rules

- A review is evidence that the artifacts were read, never by itself
  evidence that they are correct.
- Say what you checked and how. A number you did not recompute is
  "unchecked", never "fine". A checked "nothing to report" is a valid result.
- No flattery and no filler. Findings that would change the video come first.
- Do not edit any file in the repository. Return the review as your answer.
- When the three viewers pull apart: correctness (Ruth) is non-negotiable;
  then accessibility (Maya) decides pacing and vocabulary; then precision for
  practice (Dev) decides which mechanisms and costs make the cut. Most
  conflicts dissolve with a better example rather than a compromise.
"""

STAGE_NOTES = {
    "storyboard": """\
## This stage: R1, the storyboard

You are reviewing a storyboard: per chapter, one row per narration line with
its draft words (what you will hear), the picture (what you will see), the
features, and holds or listen windows (music alone; a sonification that
teaches). ⟨predict⟩ marks a prediction pause: the question is held on screen
for 1.5-3 s before the answer. Draft words are a draft: judge what they say
and show, not their polish (P3 writes the script for the ear). The fact sheet
holds every number with its independent check; the brief holds the video's
promises (takeaway, hook, pictures, vocabulary, notation, colour roles).
""",
    "script": """\
## This stage: R2, the script

You are reviewing the final script before narration is generated: every line
is spoken exactly as written, one TTS call per sentence. Check it against the
fact sheet (every number) and the storyboard (every persona note must reach
the narration, a card, a lower-third or the description, or be dropped with a
reason). Ruth also runs `tools/review_prompts/ruth_audit.md`.
""",
    "draft": """\
## This stage: R3, the narrated draft

You are reviewing the narrated 540p draft through its contact sheets and cue
sheets (images: open each one), with the script. Judge what the viewer sees
while each line is spoken: does the picture match the words, is anything
unreadable, does anything move while the viewer must read?
""",
}


def build(slug, stage):
    if stage not in STAGES:
        sys.exit(f"review: stage is one of {', '.join(STAGES)}")
    rnd, reviewers, reads = STAGES[stage]
    out = [f"# Review packet {rnd} ({stage}): {slug}\n",
           f"Generated by `tools/review.py {slug} {stage}`. Reviewers: {', '.join(reviewers)}.\n",
           STAGE_NOTES[stage], COMMON]
    for name in reads:
        if name == "sheets":
            render = ROOT / "explainer_corpus" / slug / "renders" / "narrated"
            sheets = sorted(render.glob("*.png")) if render.is_dir() else []
            if not sheets:
                sys.exit(f"review: no contact or cue sheets in {render.relative_to(ROOT)} (run the narrated draft first)")
            out.append("## Sheets (open each image)\n\n" + "\n".join(f"- `{p.relative_to(ROOT)}`" for p in sheets) + "\n")
            continue
        path = artifact(slug, name)
        text = path.read_text()
        if name == "script":
            text = "```\n" + text + "```"
        else:
            # artifacts keep their own headings one level down
            text = re.sub(r"^(#+) ", lambda m: "#" + m.group(1) + " ", text, flags=re.M)
        out.append(f"## Artifact: {name} (`{path.relative_to(ROOT)}`)\n\n{text}\n")
    memo = ROOT / f"docs/research/{slug}/MEMO.md"
    if memo.is_file():
        out.append(f"## Background (read only if you need it)\n\n- the research memo: `{memo.relative_to(ROOT)}`\n"
                   f"- the plan's section for this video: `docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md`\n")
    dest = ROOT / "docs/reviews" / slug / f"{rnd}-packet.md"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text("\n".join(out))
    return dest


def main(argv):
    if len(argv) != 2:
        sys.exit(__doc__)
    missing = [r for r in ("maya_14", "dev_28", "ruth_45", "ruth_audit", "engine_feasibility", "creative_director")
               if not (PROMPTS / f"{r}.md").is_file()]
    if missing:
        sys.exit(f"review: missing prompts {missing} in {PROMPTS.relative_to(ROOT)}")
    dest = build(*argv)
    print(f"{dest.relative_to(ROOT)} ({dest.stat().st_size // 1024} KB)")


if __name__ == "__main__":
    main(sys.argv[1:])
