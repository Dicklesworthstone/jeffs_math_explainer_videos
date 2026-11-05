"""The retired stage09 film's real text records, read-only, as verify_final_picture.py's own parsers read them.

Used only as the real positive control (run_checks.py); anchors.json is byte-identical to the sealed verifier's. Paths are relative to the repository root, which must be the working directory. Stage09
hold rows come from oliver16, the map that film was bound to. Nothing here reads a picture or a sound.
"""
import importlib.util
import json
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HELPER = Path(P) / "logarithm-remote-final-verifier01/verify_remote_final.py"
LANE = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
EL = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
S = EL / "final-picture-repair01"
CRATE = S / "workspace/explainer_corpus/logarithm_mysteries"
VERIFICATION = LANE / "logarithm-stage09-final-verify02/verification.json"
VERIFICATION_SHA256 = "22fb77188d6e9d86a0c9499e7a81db782d4b29830d1ddb4be40fe4dd3003fd62"
WORK = LANE / "logarithm-stage09-render-work02"
NARRATION = EL / "production-audio-stage09-01/prepared02/narration"
OLD_MAP = CRATE / "docs/narration-direction-map-oliver16.json"
REVIEW_SEALS = {
    "pane167": "c67b5b8f7b9c31dcf54061b1e7e8236046cfc084b68d06c46bc594aa96c2ebfe",
    "pane168": "7bfd5f08573af2aff4f82b77180f846ec43245f9aef1577e4bdc5a702dc8ee88",
    "pane169": "41501db0878d567659e11ef3ae43372d9aff799e3c589b41bd181f2b483ad854",
}


def helper(name="verify_remote_final_for_checks"):
    spec = importlib.util.spec_from_file_location(name, HELPER.resolve())
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load(V):
    assert V.sha(VERIFICATION) == VERIFICATION_SHA256, "the stage09 verification record moved"
    v = json.loads(VERIFICATION.read_text())
    script = V.read_script(CRATE / "script.tsv")
    report = json.loads((NARRATION / "report.json").read_text())
    timelines = {x["chapter"]: V.parse_timeline((WORK / f"{x['chapter']}.timeline.tsv").read_text(), x["chapter"]) for x in v["chapters"]}
    holds = {h["cue"]: h for h in json.loads(OLD_MAP.read_text())["visual_holds"]}
    marks = V.check_phrases(json.loads((NARRATION / "phrase-timings.json").read_text()), script, NARRATION)
    layouts = {ch: V.layout_anchors((WORK / f"{ch}.layout.tsv").read_text(), f"{ch}.layout.tsv") for ch in V.CHAPTERS}
    return {"v": v, "chapters": v["chapters"], "placements": v["placements"], "timelines": timelines, "script": script,
            "report": report, "onsets": V.sentence_onsets(report), "holds": holds, "marks": marks, "layouts": layouts}
