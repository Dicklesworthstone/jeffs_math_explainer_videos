#!/usr/bin/env python3
"""Bind the final 80-cue logarithm narration plan to combined-repair-stage07/candidate, revision 3.

    python3 -I -B <this file> [--check] [--out DIR] [--final-out REPO_RELATIVE_DIR]
    python3 -I -B <this file> --print-pins

Run from the repo root. Revision of logarithm-stage07-binder02's binder, which stays sealed. It
binds LavenderPike's chapter 05-08 teaching repair (logarithm05-08-teaching-repair01), which root
integrated into the candidate with integration-partial07.json: four chapter files, eight phrase
anchors read through local until_phrase helpers, three new word-sync waits, and seven holds whose
file hash changes, one of them (05_5) on moved lines. Every added row must equal the repair's sealed
hold-table.json, binder-rows.json and phrase-anchors-proposed.json, and the receipt must name the
sealed patch and every before and after hash. The map takes a new name, oliver15, because its hold
hashes differ from the oliver14 that binder02 bound and root wrote to stage07/bound-plan.

Reads stage07's candidate bytes (script b89eab60, with the 01_5 rounding wording), the kit, the
tools, root's request files, stage06's bound plan and binder02's plan (both for comparison only).
Writes only under --out, by default bound-plan next to this file, and refuses canonical and the
candidate as a target:

  explainer_corpus/logarithm_mysteries/narration_units.json
  explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver15.json
  hold-audit.json                          the 13 predictions, two readings, every other wait
  phrase-anchors.json                      every exact-byte phrase anchor; no seconds
  phrase-timings-unmeasured-template.json  all 80 cues, wav and seconds null, so it cannot load
  direction-dispositions-proposed.json     15 native_visual_hold rows for root's span manifest
  cue-delta.json                           each final cue against root's request text
  stage07-delta.json                       holds, waits, hooks and anchors against stage06
  increment-delta.json                     every change in the plan against binder02's
  inputs.sha256                            every input's bytes, for shasum -a 256 -c

STALE INPUTS ARE REFUSED. Every file this reads is pinned below, and the candidate's chapter
folder is pinned by listing. Any difference stops the run with exit 3 before anything is
computed or written. To rebind on purpose, run --print-pins, read the difference and edit PINS.

This is a text audit of Rust source. Nothing is compiled, rendered, synthesized, measured or
heard. Only chapters registered in mod.rs are read as film. A wait or phrase hook this file does
not classify stops the run, so a classification can never go stale silently. An existing output
that differs is refused, never overwritten. Nothing here depends on the clock, and nothing reads
the installed canonical crate, so a rerun on the same inputs writes the same bytes.

Narration is ElevenLabs Oliver. No Michael, no FrankenTTS.
"""
import difflib
import hashlib
import importlib.util
import json
import re
import sys
from collections import Counter
from pathlib import Path

TOPIC = "logarithm_mysteries"
REL = f"explainer_corpus/{TOPIC}"
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE06_ROOT = LANE / "combined-repair-stage06"
STAGE06 = STAGE06_ROOT / REL
STAGE06_CH = STAGE06 / "src/chapters"
STAGE06_PLAN = STAGE06_ROOT / "bound-plan"
STAGE06_MAP_NAME = "narration-direction-map-oliver13.json"
STAGE07_ROOT = LANE / "combined-repair-stage07"
STAGE = STAGE07_ROOT / "candidate" / REL
CH = STAGE / "src/chapters"
OWN_FOLDER = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder03"
FINAL_OUT = f"{OWN_FOLDER}/bound-plan"
# Canonical holds a stage06 oliver13; binder02's oliver14 is in stage07/bound-plan with the chapter 05-08
# hashes from before the teaching repair. This map binds other bytes, so it needs a new name.
MAP_NAME = "narration-direction-map-oliver15.json"
STALE_MAP = "narration-direction-map-oliver11.json"
EXPECTED_SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
ROUNDING = {
    "cue": "01_5",
    "before_script": STAGE07_ROOT / "before-rounding-wording01/script.tsv",
    "before_sha256": "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84",
    "old": "its true position is one point five eight five steps",
    "new": "its position is about one point five eight five steps",
    "request": LANE / "narration-request-rounding01/01_5.txt",
    "request_sha256": "f0a6d133ce8d07e4fcb9242c0e7a70966611ce1baca1501d3b0242c58b9e9b92",
    "review": LANE / "rounding-wording-repair01/review.json",
}
FINAL80_REQUESTS = LANE / "narration-request-final80-01/requests.json"
CURRENT_REQUESTS = LANE / "narration-request-final80-02/requests.json"
OPUS = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
WHITE_MARKERS = OPUS / "logarithm-opening-oliver-timing01/source-markers.json"
BINDER02_PLAN = OPUS / "logarithm-stage07-binder02/bound-plan"
TEACHING = OPUS / "logarithm05-08-teaching-repair01"
TEACHING_FILES = {
    "patch": TEACHING / "logarithm05-08-teaching.patch",
    "holds": TEACHING / "hold-table.json",
    "rows": TEACHING / "binder-rows.json",
    "anchors": TEACHING / "phrase-anchors-proposed.json",
    "sums": TEACHING / "SHA256SUMS",
}
TEACHING_RECEIPT = "integration-partial07.json"
TEACHING_CHAPTERS = ("complex_turns.rs", "lifting_loops.rs", "branch_cut.rs", "product_rule.rs")
WHITE_ANCHORS = {("03_3", "forgets")}
TEACHING_ANCHORS = {
    ("05_20", "the logarithm of two"), ("05_20", "a half pi"), ("05_20", "the length is two"), ("05_20", "land on two i"),
    ("06_6", "passes minus one"), ("06_6", "leaps from pi to minus pi"),
    ("08_5", "leaves a half turn"), ("08_5", "a factor of minus one"),
}
ADDED_ANCHORS = WHITE_ANCHORS | TEACHING_ANCHORS  # anchors stage06 lacks
# Which integration receipt names which stage07 file; the file's current hash must appear in it.
RECEIPTS = {
    "integration-partial01.json": ("src/chapters/matrix_logs.rs", "src/chapters/doing_nothing.rs"),
    "integration-partial02.json": ("script.tsv",),
    "integration-partial03.json": ("src/chapters/undoing_growth.rs", "src/chapters/unreachable.rs"),
    "integration-partial04.json": ("src/chapters/ratio_ruler.rs", "src/chapters/slide_rule.rs"),
    "integration-partial05.json": ("src/chapters/what_it_forgets.rs",),
    "integration-partial06.json": ("src/chapters/folded_decade.rs",),
    TEACHING_RECEIPT: tuple(f"src/chapters/{name}" for name in TEACHING_CHAPTERS),
}
# Every other crate file this reads, at the same path in stage06 and stage07. Stage06's bytes are
# read for the comparison; its oracle table still serves because oracle.py and facts.rs are equal.
SAME_NAME_FILES = ("script.tsv", "src/main.rs", "src/facts.rs", "oracle.py", "docs/storyboard.md", f"docs/{STALE_MAP}", "narration_units.json")
STAGE06_PLAN_FILES = (
    f"{REL}/narration_units.json",
    f"{REL}/docs/{STAGE06_MAP_NAME}",
    "hold-audit.json",
    "phrase-anchors.json",
    "phrase-timings-unmeasured-template.json",
    "direction-dispositions-proposed.json",
    "cue-delta.json",
    "inputs.sha256",
)
BINDER02_PLAN_FILES = (
    f"{REL}/narration_units.json",
    f"{REL}/docs/narration-direction-map-oliver14.json",
    "hold-audit.json",
    "phrase-anchors.json",
    "phrase-timings-unmeasured-template.json",
    "direction-dispositions-proposed.json",
    "cue-delta.json",
    "stage07-delta.json",
    "inputs.sha256",
)
SHARED_FILES = (
    "crates/explainer_kit/src/kit.rs",
    "crates/explainer_kit/src/narration.rs",
    "crates/explainer_kit/src/captions.rs",
    "crates/explainer_score/src/score.rs",
    "tools/coherent_narration.py",
    "tools/narration_cache.py",
    "tools/narration_delivery.py",
    "tools/final_check.py",
)
OBSOLETE = [("00_1", "00_2"), ("01_1", "01_2"), ("03_1", "03_2"), ("04_1", "04_2")]
PREDICTIONS = 13

SAY = re.compile(r'\bkit\.say\(stage, "(\d\d_\d+)"\)')
ANY_SAY = re.compile(r"\.say\(")
TIMING = re.compile(
    r"\b(predict_pause|predict|kit\.hold|kit\.music_breath|kit\.listen|kit\.sonify|kit\.mark|kit\.finish|"
    r"stage\.wait|wait_frames|wait_until)\("
)
HOOKS = [
    (re.compile(r'until_phrase\(&kit, stage, "(\d\d_\d+)", ([A-Z_]+|"[^"]*")\)'), "until_phrase"),
    (re.compile(r'until_phrase\(stage, &kit, "(\d\d_\d+)", ([A-Z_]+|"[^"]*")\)'), "until_phrase"),
    (re.compile(r'phrase_time\(&kit, "(\d\d_\d+)", ([A-Z_]+|"[^"]*")\)'), "phrase_time"),
]
HOOK_HELPER_LINE = re.compile(r"\bfn (until_phrase|phrase_time)\(|phrase_time\(kit, (id|cue), phrase\)")
PHRASE_MARK_USES = {"complex_turns.rs": 1, "lifting_loops.rs": 1, "product_rule.rs": 1, "folded_decade.rs": 1, "slide_rule.rs": 1, "undoing_growth.rs": 1, "doing_nothing.rs": 1, "unreachable.rs": 1, "what_it_forgets.rs": 1}
SENTENCE_HOOKS = [
    (re.compile(r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)'), "until_sentence"),
    (re.compile(r'sentence_time\(&kit, "(\d\d_\d+)", (\d+)\)'), "sentence_time"),
    (re.compile(r'\.sentence_at\("(\d\d_\d+)", (\d+)\)'), "sentence_at"),
    (re.compile(r'sentence_len\(&kit\.sentence_starts\("(\d\d_\d+)"\), (\d+)\)'), "sentence_len"),
    (re.compile(r'room\(&kit\.sentence_starts\("(\d\d_\d+)"\), (\d+),'), "room"),
    (re.compile(r'until_share\(&kit, stage, "(\d\d_\d+)", (\d+),'), "until_share"),
]
NON_LITERAL_SENTENCE = re.compile(r'kit\.until_sentence\(stage, "(\d\d_\d+)", [^0-9"]')

FALLBACK = {
    "complex_turns.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (complex_turns.rs phrase_seconds). Provisional.",
    "lifting_loops.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (lifting_loops.rs phrase_seconds). Provisional.",
    "product_rule.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (product_rule.rs phrase_seconds). Provisional.",
    "folded_decade.rs": "Without a mark at this byte: DIGITS_SHARE of the span from sentence 0 to sentence 1 (folded_decade.rs lay_beats). Provisional.",
    "slide_rule.rs": "Without a mark at this byte: the phrase's character share of the cue's first sentence (slide_rule.rs phrase_time). Provisional.",
    "undoing_growth.rs": "Without a mark at this byte: the phrase's character share between the onsets of sentences 3 and 4 (undoing_growth.rs climb_onset). Provisional.",
    "doing_nothing.rs": "Without a mark at this byte: the phrase's byte share of its measured sentence (doing_nothing.rs phrase_seconds). Provisional.",
    "unreachable.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (unreachable.rs phrase_seconds). Provisional.",
    "what_it_forgets.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (what_it_forgets.rs phrase_seconds). Provisional.",
}
# Every wait that is not predict, a reading window or a chapter's finish: (file, line) -> class, note.
SITES = {
    ("ratio_ruler.rs", 210): ("picture_pacing", "Paces the note dots of the opening, 0.18 s each, in a loop, now inside 00_2. No hold for a viewer."),
    ("ratio_ruler.rs", 509): ("helper_tail", "Last line of preview(), called for 00_6 directly before kit.finish, which holds to the same point. No added window."),
    ("ratio_ruler.rs", 555): ("word_sync_helper", "Inside read_out: waits to a character share of 00_1's second sentence, inside the voice. The census lists it under 00_6 because the helper follows preview() in the file; it runs during 00_1, called at :203. An estimate from source, not a measured time."),
    ("slide_rule.rs", 102): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("undoing_growth.rs", 351): ("word_sync", "Waits to the onset of \"it only ever climbs\", inside 04_2."),
    ("complex_turns.rs", 845): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("lifting_loops.rs", 769): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("product_rule.rs", 606): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("matrix_logs.rs", 206): ("picture_pacing", "Shows each named matrix for 0.9 s, in a loop inside 09_1. Whether a pass outlasts its sentence is not fixed by source."),
    ("doing_nothing.rs", 215): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("doing_nothing.rs", 598): ("reading_voice_end", "10_4's reading hold starts its arithmetic here. See reading_holds."),
    ("unreachable.rs", 288): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("unreachable.rs", 1555): ("reading_voice_end", "11_20's reading clock starts when this returns. See reading_holds."),
    ("what_it_forgets.rs", 556): ("word_sync_helper", "Inside until_share: waits to a share of a sentence, inside the voice."),
    ("what_it_forgets.rs", 634): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("what_it_forgets.rs", 1145): ("picture_pacing", "ARRIVAL_HOLD, 0.5 s on an arrival, then until_phrase, inside 12_6."),
}
READING_MARKERS = {
    "doing_nothing.rs": "kit.music_breath(stage, FINE_PRINT_S - read_for)?;",
    "unreachable.rs": "kit.music_breath(stage, rule_read + row_reads.iter().sum::<f64>())?;",
}
CH_LISTING = (
    "bookends.rs", "branch_cut.rs", "building_ruler.rs", "complex_turns.rs", "doing_nothing.rs", "doing_nothing.rs.orig",
    "draw.rs", "folded_decade.rs", "lifting_loops.rs", "matrix_log.rs", "matrix_logs.rs", "mod.rs", "product_law.rs",
    "product_rule.rs", "ratio_ruler.rs", "real_inverse.rs", "slide_rule.rs", "thumbnail.rs", "undoing_growth.rs",
    "unreachable.rs", "what_it_forgets.rs", "winding.rs",
)
# PINS-BEGIN
PINS = {
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/script.tsv": "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/script.tsv": "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/main.rs": "b45e720b6ce9d63a933946d0b12fb978044512b9ff10309fa7ba3a9ef52f54e1",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/main.rs": "b45e720b6ce9d63a933946d0b12fb978044512b9ff10309fa7ba3a9ef52f54e1",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/facts.rs": "8901e74b0afdc6ea9a2c17b9508a51631e7fd7888df643652bac40c505874a5e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/facts.rs": "8901e74b0afdc6ea9a2c17b9508a51631e7fd7888df643652bac40c505874a5e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/oracle.py": "3bf99af6e88bdfc363fc42adf20f346ecb9bd6f161bebe2fcdc341fad02cbcce",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/oracle.py": "3bf99af6e88bdfc363fc42adf20f346ecb9bd6f161bebe2fcdc341fad02cbcce",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/docs/storyboard.md": "7db2b8edeed482f51c9a99a8e93ba6efc51df3337cccbbde8dc51fefcab772e1",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/docs/storyboard.md": "7db2b8edeed482f51c9a99a8e93ba6efc51df3337cccbbde8dc51fefcab772e1",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver11.json": "c73bec82a4461342ef24247539899ec5d163de9cd8bce577a5746378786d92fd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver11.json": "c73bec82a4461342ef24247539899ec5d163de9cd8bce577a5746378786d92fd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/narration_units.json": "7eb65d0149f890d33d005d34d9d0488bb03cd7a734871adb43e79fd170f068a3",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/narration_units.json": "7eb65d0149f890d33d005d34d9d0488bb03cd7a734871adb43e79fd170f068a3",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/bookends.rs": "f5a5a48e6402b5f82022e1c2b302d526e7ff0491c9d73357811b0a09bb035555",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/bookends.rs": "f5a5a48e6402b5f82022e1c2b302d526e7ff0491c9d73357811b0a09bb035555",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/branch_cut.rs": "c1159c3727a1f1d83351d2c837027c563455df6ebc2de32a2db5973d56cbc401",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/branch_cut.rs": "193c4733de9f9081f5671700a6710dd76e989b41ab268824bbb0756313fbdbbd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/building_ruler.rs": "ae125e5ec9ec9bd6df073d90c9a9b77f80a9c202c04ad6803ea6ef3b8a3f903a",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/building_ruler.rs": "ae125e5ec9ec9bd6df073d90c9a9b77f80a9c202c04ad6803ea6ef3b8a3f903a",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/complex_turns.rs": "93f663d372ab79a77139287dc38b94c148909f7ea4a6ba549884c91f342eb19a",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/complex_turns.rs": "2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/doing_nothing.rs": "7f445be212c586093d2092113b086d809dcc189619985baf1c8f0c407c709b47",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/doing_nothing.rs": "eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/doing_nothing.rs.orig": "c526e62d34a42bc3d08e339a5fdbb1df51f8213ac2440207b58a829dcb5164de",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/doing_nothing.rs.orig": "c526e62d34a42bc3d08e339a5fdbb1df51f8213ac2440207b58a829dcb5164de",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/draw.rs": "2a4e84504bab636433cd419b2c9990986d7c549bf05aa3432d33bd5a9b60fe28",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/draw.rs": "2a4e84504bab636433cd419b2c9990986d7c549bf05aa3432d33bd5a9b60fe28",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/folded_decade.rs": "acfd77f46c2378c98ec77d1b5e7d0f2cb9c15f2bc3d209d80ba690c8fcc26e5e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/folded_decade.rs": "acfc7f9c198b3d3d0f99e1b160ec6c2d4e22a4ce28aeb4263deb29df9efa670d",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/lifting_loops.rs": "01885b9f558c36bc8af425bfdf427e4b30d74d575819121e52988b5d06db7346",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/lifting_loops.rs": "ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/matrix_log.rs": "d6ee8a2b63b4bb69b684fbefdec0633ac41fd02b20d758af1f24edf8b89d2e12",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/matrix_log.rs": "d6ee8a2b63b4bb69b684fbefdec0633ac41fd02b20d758af1f24edf8b89d2e12",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/matrix_logs.rs": "7df6f8681099c76dcd1f32494e2edcbb07b427c54178d5c557d5801052f481b0",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/matrix_logs.rs": "c82b2a4d10f3c5fda758ad110b6297efc5512a643bfc26f4f05e4532c6ebf2cd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/mod.rs": "ec18cd537b8c81bd4a1341d752c8b632a4355f083281776863d33c3a03a8ccfa",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/mod.rs": "ec18cd537b8c81bd4a1341d752c8b632a4355f083281776863d33c3a03a8ccfa",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/product_law.rs": "21134bccc57bcc5b324666bf1ffbfd31f294d9ad496cf6397ff52a11eeafada6",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/product_law.rs": "21134bccc57bcc5b324666bf1ffbfd31f294d9ad496cf6397ff52a11eeafada6",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/product_rule.rs": "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/product_rule.rs": "e9019c828710207b806175a040aab936ee558b102d3cb55328078d42d897f7e0",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/ratio_ruler.rs": "198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/ratio_ruler.rs": "92cf028b124f8a3732218cbe31505e5351f86fea5bad180c0d3c97240e077bfd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/real_inverse.rs": "f3e18f94f7d692ae74ceedfda56dd4db10eb4fbd04dc2a3227d835af41d81e51",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/real_inverse.rs": "f3e18f94f7d692ae74ceedfda56dd4db10eb4fbd04dc2a3227d835af41d81e51",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/slide_rule.rs": "68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/slide_rule.rs": "eea4d01d96fd4853ff541e0f55264178a09428079200dd9397b148a3976ab7e4",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/thumbnail.rs": "5f6072695559af35555711df1b2ba7ad10f7ecaafb8ba902380db29143e83efb",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/thumbnail.rs": "5f6072695559af35555711df1b2ba7ad10f7ecaafb8ba902380db29143e83efb",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/undoing_growth.rs": "4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/undoing_growth.rs": "754ea3125c2f7e89440807772713fe57584c96937be3ec0ac8c52d86645cf323",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs": "c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs": "a676dac78580ef73f0e658a21c0395cfe0505fe631bd6aa0d71b1f3ec05a86ee",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/what_it_forgets.rs": "6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/what_it_forgets.rs": "e938b750ce579faed820ae7ddd0d289819f7148bdfb991dc99f24d35d6f8491b",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/src/chapters/winding.rs": "860291e51fdbc83095786e0528080cb9eba4ccc618fa00b963197c59019c444b",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/explainer_corpus/logarithm_mysteries/src/chapters/winding.rs": "860291e51fdbc83095786e0528080cb9eba4ccc618fa00b963197c59019c444b",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial01.json": "d0f48615f24d497fd1b39ea9c011ee28b134e2d753002861c31ee5b706d333b8",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial02.json": "5be0e08a15fa483f9e298913731f0f47e93aac364aec66356c270672b4eb44b9",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial03.json": "ae98c13b82310aeffecd6544542e07ae0eaeb6938763f30049e44d1ed73c5bbe",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial04.json": "13093614c086c2137aceebfaad9ed9cdf24603115c9a7411566d541ff93ca15f",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial05.json": "6a437e6e52fd35bd9580ad2890c221a2b01a26c2c8954d8a16fc5dddcac3cfe0",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial06.json": "52928856607173bd1e535a332ebc37a4a214457803e3479cc41e425d4c8408a5",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/integration-partial07.json": "4b1751d5bc3170f6b3c779616b1dd4c8c8f190ec678b2ebd5337cc6a5c058db3",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/oracle04.tsv": "f5c028cea6ca71e3a3519a430af484079d0c7a445d0b771a9c3b00d754258540",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-rounding-wording01/script.tsv": "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/narration-request-rounding01/01_5.txt": "f0a6d133ce8d07e4fcb9242c0e7a70966611ce1baca1501d3b0242c58b9e9b92",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/rounding-wording-repair01/review.json": "f29cc06f8990c99dd356f3b0f5123e29c26096b02c87c7d6938c3b024ea877fb",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/narration-request-final80-01/requests.json": "f93a6f1e13808249711901282cfce89e9256ffd2749fbed439cd6a4686fc3069",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/narration-request-inventory01.json": "d4f78c943703c04112ff2b96088d563a2f467c0defd984b31cc46e7e88243d9f",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/narration-request-plan01.json": "71c5a5111b1a0eec44531403230ccce7acaa44fa29167ad138cd4602e2d5a5aa",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/narration-request-final80-02/requests.json": "c533f4ade3929518a68ecdf7f7b7027c22cbf59480e599c4b5982fd25e1e33be",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-oliver-timing01/source-markers.json": "4b523a5ec39ed7a636b4e3480ed60991091719e8628450c113887b1cfa9fdeb3",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/explainer_corpus/logarithm_mysteries/narration_units.json": "d594089297ed09534bf4a291f1ddb33d10402da3d60a2aeee9f7175a4381aee2",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver13.json": "56f6ad64344514005a5ac385eb7826fa276346fb6c449acf54bf3de191da7e27",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/hold-audit.json": "610e9f49db586cc9c307a9a16fd0d9d9ae7d70d7b29769f56eed01f1b238bf14",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/phrase-anchors.json": "01b7d1a54d08062e1b45d725d990e5684c8a92da4ffc495475bc6f4adac60d76",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/phrase-timings-unmeasured-template.json": "141a635e86e430711b61a527cf5e05763bddb5ed6ad9e11af7126a6b6415fd5a",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/direction-dispositions-proposed.json": "5239cb21f91543c16dbf647324ec67f524915f1f64c8ba3d08c9b5049d681049",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/cue-delta.json": "125a049a035dc3f7db59e8fc1931591358e625b5231d64e3ba9ad19e3275d988",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/inputs.sha256": "b4926a8f35044f8ba7b686c266bd2a6695a7b7a17bf73c1cab34f31e98bb418d",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/explainer_corpus/logarithm_mysteries/narration_units.json": "61a1f622d083a700e3749bb6a4b92c61625231033ab5aef6dcfacc16c71d7b8f",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver14.json": "6a326cdee5ab9e8b2055bcb2aa77ffeb16c493df16f4d0e464e448175a97cf66",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/hold-audit.json": "15aa3a4b3b7931090e15467b92257af2dac10adc51373f79fd9b52a3fa572560",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/phrase-anchors.json": "603ba46d1b2893a8f4428ebb5f655d2785416d28cf4629b824e50e3dc82e1c15",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/phrase-timings-unmeasured-template.json": "901392b45e57c559b9159ea19ed3abe240eab0bc9f088adf448ba2c2b7febb1b",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/direction-dispositions-proposed.json": "8b4fe095a7965ce6f834acecd350da62f114bb19be3f0900617b35f8a27b4baa",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/cue-delta.json": "2263bb51a62e41972cd12adca21660b67b515f2fda9781291e6761a4bb56cdac",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/stage07-delta.json": "08f106ac727bbae2512d740d7cd4cda292a9d76e721dc422f73fc58c4c07535a",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder02/bound-plan/inputs.sha256": "cafb258539f9c3d2ea4ad6a438c2afb662945b24bf4368837d5393b20e8eca3c",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm05-08-teaching-repair01/logarithm05-08-teaching.patch": "1aafaaa84671c95e5a49340882efbe32aaea2ab7b74ba4ab149a1636dd90db34",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm05-08-teaching-repair01/hold-table.json": "24c7ac0c693d43a91c6ff68e86eba63cdf1683d9908d36050781ecabeb4bddf0",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm05-08-teaching-repair01/binder-rows.json": "8492714dfcc8ad9c8e4a2dd2c704f546eb59ad324b0de0e250b8e6385543e574",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm05-08-teaching-repair01/phrase-anchors-proposed.json": "4c180bcdee55d433d6bf5298d3cc6d2a71ac515d8fc07c22734e64ea2595726b",
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm05-08-teaching-repair01/SHA256SUMS": "f4e2589d4dfd0c57a014b1b2119f58b5915579a3ee126f071fdb5784cafcaf6f",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-teaching05-08-01/complex_turns.rs": "2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-teaching05-08-01/lifting_loops.rs": "ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-teaching05-08-01/branch_cut.rs": "193c4733de9f9081f5671700a6710dd76e989b41ab268824bbb0756313fbdbbd",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-teaching05-08-01/product_rule.rs": "e9019c828710207b806175a040aab936ee558b102d3cb55328078d42d897f7e0",
    "crates/explainer_kit/src/kit.rs": "ef035faf236498e7485fb60a66d862b383e1a5040fa17962e1633596ed99e106",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    "crates/explainer_kit/src/captions.rs": "28e1aa51370b8c3b2fa1f711a388d9233909503684dc4c1d77a2d5f1043b26bd",
    "crates/explainer_score/src/score.rs": "8ba3f501a487b981e65d10b92c332ca7d69c36c3326fb4443309d81068ef5592",
    "tools/coherent_narration.py": "e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0",
    "tools/narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    "tools/narration_delivery.py": "c91914b89bed9e7aca71a649d403fa591bf40ec135f9c3d30f9d5e45c88422ba",
    "tools/final_check.py": "5996634981eef3c2e1dd6780c30f785fc33e1200beaf17afdb5a15cf42bba26a",
}
# PINS-END
PREDICTION_REASON = "Source-bound native prediction hold follows complete cue; full recordings must be preserved."
READING_REASON = "Source-bound native reading hold follows complete cue; full recordings must be preserved."
PREDICTION_BASIS = "Direct registered scene cue interval and native hold call. Source only; not audible or picture acceptance."
UNITS_SCOPE = (
    "Candidate full-cue source plan for ElevenLabs Oliver, combined-repair-stage07/candidate, script b89eab60: "
    "80 units, one per cue, at most 100 words. 01_5's wording changed, so no take of its old text can bind. It "
    "says nothing about whether a recording of any other cue exists; unchanged wording is not evidence of a "
    "usable take. No synthesis, take selection, listening, timing or independent acceptance. No Michael or "
    "FrankenTTS."
)
MAP_SCOPE = (
    "Candidate final 80-cue map, combined-repair-stage07/candidate: 13 prediction holds and 2 reading holds, "
    "each bound to staged source, after the chapter 05-08 teaching repair. Proposed successor to oliver14, "
    "which binds the same script and holds with the chapter 05-08 bytes from before that repair, and to "
    "stage06's oliver13, which binds script f201d2dc; like them, replaces oliver11's four stale pairs, which name the retired "
    "seven-chapter draft. Hashes are the proposed staged bytes, not installed canonical files. Authored "
    "coverage only: no silence is added to any take, and no take, measured timing, listening or independent "
    "acceptance is claimed. ElevenLabs Oliver; no Michael or FrankenTTS."
)


def stop(message):
    raise SystemExit(f"bind_final80: {message}")


def pin_paths():
    """Every file the binder reads apart from the request texts, which root's own files pin."""
    rows = []
    for name in SAME_NAME_FILES:
        rows += [STAGE / name, STAGE06 / name]
    for name in CH_LISTING:
        rows += [CH / name, STAGE06_CH / name]
    rows += [STAGE07_ROOT / r for r in RECEIPTS]
    rows += [STAGE06_ROOT / "oracle04.tsv", ROUNDING["before_script"], ROUNDING["request"], ROUNDING["review"], FINAL80_REQUESTS]
    rows += [LANE / "narration-request-inventory01.json", LANE / "narration-request-plan01.json"]
    rows += [CURRENT_REQUESTS, WHITE_MARKERS]
    rows += [STAGE06_PLAN / p for p in STAGE06_PLAN_FILES]
    rows += [BINDER02_PLAN / p for p in BINDER02_PLAN_FILES]
    rows += list(TEACHING_FILES.values())
    rows += [STAGE07_ROOT / "before-teaching05-08-01" / name for name in TEACHING_CHAPTERS]
    rows += [Path(p) for p in SHARED_FILES]
    return [str(p) for p in rows]


def refuse_stale_inputs():
    """Exit 3 before anything is computed if any pinned input differs."""
    problems = []
    wanted = pin_paths()
    if set(wanted) != set(PINS):
        problems.append(f"the pin table does not name exactly this binder's inputs: {sorted(set(wanted) ^ set(PINS))}")
    for path, want in PINS.items():
        p = Path(path)
        if not p.is_file():
            problems.append(f"MISSING  {path}")
        elif file_sha(p) != want:
            problems.append(f"CHANGED  {path}\n         pinned {want}\n         found  {file_sha(p)}")
    listed = sorted(p.name for p in CH.iterdir()) if CH.is_dir() else []
    if listed != sorted(CH_LISTING):
        problems.append(f"LISTING  {CH}: added {sorted(set(listed) - set(CH_LISTING))}, gone {sorted(set(CH_LISTING) - set(listed))}")
    if problems:
        print("bind_final80: STALE INPUT, refused; nothing computed or written.", file=sys.stderr)
        for row in problems:
            print(row, file=sys.stderr)
        sys.exit(3)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    return sha(Path(path).read_bytes())


def load_coherent():
    spec = importlib.util.spec_from_file_location("coherent_narration", "tools/coherent_narration.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source(path):
    """(lines, scene text): the scene text stops at the tests, and keeps the file's line numbers."""
    text = Path(path).read_text(encoding="utf-8")
    at = text.find("#[cfg(test)]")
    if at >= 0 and not text.startswith("#[cfg(test)]\nmod tests", at):
        stop(f"{path}: a cfg(test) item before the tests module")
    scene = text if at < 0 else text[:at]
    return text.split("\n"), scene


def registered():
    src = (CH / "mod.rs").read_text(encoding="utf-8")
    pairs = re.findall(r'key: "(\d\d_\w+)",\s*make: \|kit\| Box::new\((\w+)::', src)
    declared = set(re.findall(r"^(?:pub )?mod (\w+);", src, re.M))
    if any(m not in declared for _, m in pairs) or len({m for _, m in pairs}) != len(pairs):
        stop("the registry does not match the declared modules")
    if sorted(m for _, m in pairs) != sorted(d for d in declared if d not in ("draw", "thumbnail")):
        stop("mod.rs declares a chapter the registry does not use")
    return [(key, f"{module}.rs") for key, module in pairs]


def one_line(lines, needle, display):
    hits = [n for n, line in enumerate(lines, 1) if needle in line]
    if len(hits) != 1:
        stop(f"{display}: {needle!r} is on {len(hits)} lines")
    return hits[0]


def evidence_row(display, actual, needle):
    data = Path(actual).read_bytes()
    lines = data.decode("utf-8").split("\n")
    n = one_line(lines, needle, display)
    return {"path": display, "sha256": sha(data), "line": n, "marker": lines[n - 1].strip()}


def constant(pattern, text, display):
    hits = re.findall(pattern, text, re.M)
    if len(hits) != 1:
        stop(f"{display}: {pattern!r} matched {len(hits)} times")
    return hits[0]


def census(chapters, cues):
    """Every say and timing call in the registered chapters' scene code, by file and line."""
    says, sites = [], []
    for name in chapters:
        lines, scene = source(CH / name)
        current = None
        for n, line in enumerate(scene.split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            m = SAY.search(line)
            if m:
                current = m.group(1)
                says.append({"file": name, "line": n, "cue": current})
            elif ANY_SAY.search(line):
                stop(f"{name}:{n}: a say this audit cannot read: {line.strip()}")
            t = TIMING.search(line)
            if t and not re.search(r"\bfn (predict|wait_until)\(", line):
                sites.append({"file": name, "line": n, "call": t.group(1), "marker": line.strip(), "cue": current})
    if [s["cue"] for s in says] != cues:
        stop("registered chapters do not say every script cue once, in order")
    return says, sites


def sentence_opens(text):
    """Byte offsets where sentences open, as phrase_seconds and doing_nothing's sentence_offsets count them."""
    raw = text.encode("utf-8")
    opens = [0]
    for i in range(len(raw) - 1):
        if raw[i] in b".?!" and raw[i + 1] == 0x20:
            opens.append(i + 2)
    return opens


def after_marker(lines, marker_line, next_say_line):
    """What the source does between a hold's marker and the next cue's say."""
    rows = []
    if next_say_line is None:
        return rows
    for n in range(marker_line + 1, next_say_line):
        line = lines[n - 1].strip()
        if line and not line.startswith("//"):
            rows.append({"line": n, "code": line})
    return rows


def reading_11_20():
    """The 17.73 s, from the literals in unreachable.rs and the fact behind the mixed row."""
    display = f"{REL}/src/chapters/unreachable.rs"
    _, scene = source(CH / "unreachable.rs")
    block = re.search(r"^const THEOREM: \[&str; 4\] = \[\n(.*?)^\];", scene, re.M | re.S)
    if not block:
        stop("unreachable.rs: THEOREM not found")
    theorem = re.findall(r'"([^"\\]*)"', block.group(1))
    rate = float(constant(r"^const READING_RATE: f64 = ([0-9.]+);", scene, display))
    body = re.search(r"^fn block_examples\(\).*?^}", scene, re.M | re.S)
    if not body:
        stop("unreachable.rs: block_examples not found")
    verdicts = re.findall(r'"([^"\\]*(?:real log)[^"\\]*)"', body.group(0))
    expected = ["{} paired, {} positive: real log", "matching pair: real log", "no partners: no real log", "one size-2 block: no real log"]
    if sorted(verdicts) != sorted(expected):
        stop(f"unreachable.rs: the row verdicts changed: {verdicts}")
    row = next(line for line in (STAGE06_ROOT / "oracle04.tsv").read_text(encoding="utf-8").splitlines() if line.startswith("matrix.blocks_mixed\t"))
    values = [float(x) for x in row.split("\t")[1].split(",")]
    if any(v != int(v) for v in values):
        stop("matrix.blocks_mixed is no longer whole numbers")
    shown = lambda v: ("−" if v < 0 else "") + str(abs(int(v)))
    said = "{} paired, {} positive: real log".format(shown(values[0]), shown(values[2]))
    rows = ["matching pair: real log", "no partners: no real log", said, "one size-2 block: no real log"]
    rule_chars, row_chars = sum(len(t) for t in theorem), sum(len(t) for t in rows)
    return {
        "rule_lines": theorem,
        "rule_chars": rule_chars,
        "row_verdicts": rows,
        "row_chars": row_chars,
        "characters_per_second": rate,
        "rule_seconds": round(rule_chars / rate, 4),
        "row_seconds": round(row_chars / rate, 4),
        "window_seconds": round((rule_chars + row_chars) / rate, 4),
        "matrix_blocks_mixed": row.split("\t")[1],
    }


def reading_10_4():
    display = f"{REL}/src/chapters/doing_nothing.rs"
    _, scene = source(CH / "doing_nothing.rs")
    textbook = constant(r'^const TEXTBOOK: &str = "([^"\\]*)";', scene, display)
    cited = constant(r'^const SOURCE: &str = "([^"\\]*)";', scene, display)
    floor = float(constant(r"^const FINE_PRINT_S: f64 = ([0-9.]+);", scene, display))
    # Since the chapter 10 repair SOURCE carries the prefix itself; count it once.
    prefix = "precisely: "
    if not cited.startswith(prefix):
        stop("doing_nothing.rs: SOURCE no longer opens with its prefix")
    chars = len(textbook) + len(cited)
    return {
        "fine_print": {"textbook": textbook, "source": cited[len(prefix):], "prefix": prefix},
        "source_const_includes_prefix": True,
        "fine_print_chars": chars,
        "floor_seconds_from_shown": floor,
        "characters_per_second_at_floor": round(chars / floor, 3),
    }


def sentence_census(chapter_dir, names, texts, coherent):
    """Every literal sentence read, and every line that reads sentences some other way, in scene code."""
    rows, other = [], []
    for name in names:
        _, scene = source(Path(chapter_dir) / name)
        current = None
        for n, line in enumerate(scene.split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            m = SAY.search(line)
            if m:
                current = m.group(1)
            hit = False
            for pattern, kind in SENTENCE_HOOKS:
                for cue, j in pattern.findall(line):
                    hit = True
                    if not 0 <= int(j) < len(coherent.sentences(texts[cue])):
                        stop(f"{name}:{n}: {kind} {cue} sentence {j} does not exist")
                    rows.append({"file": name, "line": n, "hook": kind, "cue": cue, "sentence": int(j), "current": current})
            if not hit and re.search(r"\b(sentence_at|sentence_time|until_sentence|until_share|sentence_starts|sentence_len)\(", line) and not re.search(r"\bfn \w+\(", line):
                other.append({"file": name, "line": n, "code": line.strip(), "current": current})
    return rows, other


def opening_read_out(texts, lines):
    """00_1's printed numbers: F1's character shares, as source estimates. Never marks or times."""
    name = "ratio_ruler.rs"
    _, scene = source(CH / name)
    block = re.search(r"^const READ_PHRASES: \[&str; 6\] = \[\n(.*?)^\];", scene, re.M | re.S)
    if not block:
        stop(f"{name}: READ_PHRASES not found")
    phrases = re.findall(r'"([^"\\]*)"', block.group(1))
    if len(phrases) != 6:
        stop(f"{name}: {len(phrases)} read phrases")
    text = texts["00_1"]
    if ". " not in text:
        stop("00_1 has no second sentence")
    sentence = text[text.index(". ") + 2 :]
    rows = []
    for p in phrases:
        if sentence.count(p) != 1:
            stop(f"00_1: {p!r} occurs {sentence.count(p)} times, and read_shares would fall back or mislead")
        rows.append({"phrase": p, "char_offset": sentence.index(p), "share_of_rest_of_cue": round(sentence.index(p) / len(sentence), 4), "measured_seconds": None})
    if [r["char_offset"] for r in rows] != sorted(r["char_offset"] for r in rows):
        stop("00_1: the read phrases are out of spoken order")
    return {
        "status": "CURRENT BASELINE. Character-share estimates from source; not measured, not marks, not phrase hooks.",
        "note": "ratio_ruler.rs read_shares and read_out, as integrated. Deliberately not until_phrase, so they read no entry of phrase-timings.json, and a mark for these words would change nothing until the source reads it. White's audit, logarithm-opening-oliver-timing01, compared these with two retained takes per cue and proposes no source change. Neither take is selected and nothing was heard. This plan proposes no hook and no anchor for these words.",
        "cue": "00_1",
        "sentence_index": 1,
        "text_measured": sentence,
        "characters": len(sentence),
        "source_lines": {
            "call": one_line(lines, "read_out(stage, &kit, &labels)?;", name),
            "script_const": one_line(lines, 'const SCRIPT: &str = include_str!("../../script.tsv");', name),
            "read_phrases": one_line(lines, "const READ_PHRASES: [&str; 6] = [", name),
            "read_fade": one_line(lines, "const READ_FADE: f64", name),
            "read_shares": one_line(lines, "fn read_shares(sentence: &str) -> Option<Vec<f64>> {", name),
            "read_out": one_line(lines, "fn read_out(stage: &mut Stage<'_>, kit: &Kit, words: &[Mob]) -> fmn::Result<()> {", name),
            "sentence_at": one_line(lines, 'let s1 = n.sentence_at("00_1", 1)?;', name),
            "wait": one_line(lines, "super::draw::wait_until(stage, at)?;", name),
        },
        "fallback_without_narrator": "One every 0.9 s (read_out); a draft-only path, not a timing claim.",
        "rows": rows,
    }


def white_markers(anchor_rows, script_sha, file_hash):
    """The added anchor must be exactly what White's sealed source-markers.json proposed."""
    doc = json.loads(WHITE_MARKERS.read_text(encoding="utf-8"))
    rows = doc["phrase_anchor_delta"]
    if len(rows) != 1 or {(r["cue"], r["phrase"]) for r in rows} != WHITE_ANCHORS:
        stop("source-markers.json no longer proposes exactly the one anchor")
    want = rows[0]
    got = next((r for r in anchor_rows if (r["cue"], r["phrase"]) == (want["cue"], want["phrase"])), None)
    if got is None:
        stop(f"03_3: no anchor on the phrase source-markers.json says, {want['phrase']!r}")
    for field in ("byte", "byte_end", "const", "sentence_index", "cue_text_sha256", "hook_sites", "measured_seconds"):
        if got[field] != want[field]:
            stop(f"03_3: {field} is {got[field]!r}, source-markers.json says {want[field]!r}")
    if doc["baseline"]["stage07_script_sha256"] != script_sha:
        stop("source-markers.json binds another script")
    source_row = doc["sources"][0]
    if len(doc["sources"]) != 1 or not source_row["path"].endswith("/folded_decade.rs") or source_row["candidate_sha256"] != file_hash["folded_decade.rs"]:
        stop("source-markers.json names other source bytes")
    for field in ("cue_text_delta", "map13_hold_delta", "binder_sites_delta", "sentence_hook_delta"):
        if doc[field]:
            stop(f"source-markers.json now proposes a change in {field}")
    if doc["phrase_mark_uses_delta"] != {"folded_decade.rs": 1} or doc["fallback_delta"] != {"folded_decade.rs": FALLBACK["folded_decade.rs"]}:
        stop("the binder's folded_decade.rs rows differ from source-markers.json")
    if doc["template_mark_delta"] != {want["cue"]: [{"byte": want["byte"], "seconds": None, "phrase": want["phrase"]}]}:
        stop("the template row differs from source-markers.json")
    return {"path": str(WHITE_MARKERS), "sha256": file_sha(WHITE_MARKERS)}


def teaching_rows(anchor_rows, hold_rows, file_hash):
    """Every added row must equal the teaching repair's sealed files, and the receipt must be root's for its patch."""
    sums = dict(reversed(line.split("  ", 1)) for line in TEACHING_FILES["sums"].read_text(encoding="utf-8").splitlines() if line.strip())
    for key in ("patch", "holds", "rows", "anchors"):
        rel = TEACHING_FILES[key].relative_to(TEACHING).as_posix()
        if sums.get(rel) != file_sha(TEACHING_FILES[key]):
            stop(f"{rel} is not the bytes the repair's SHA256SUMS seals")
    receipt = json.loads((STAGE07_ROOT / TEACHING_RECEIPT).read_text(encoding="utf-8"))
    if receipt.get("patch_sha256") != file_sha(TEACHING_FILES["patch"]):
        stop(f"{TEACHING_RECEIPT} names another patch")
    changed = {Path(r["path"]).name: r for r in receipt.get("changed", [])}
    if sorted(changed) != sorted(TEACHING_CHAPTERS) or any(not r["path"].startswith(f"{REL}/src/chapters/") for r in changed.values()):
        stop(f"{TEACHING_RECEIPT} does not name exactly the four repaired chapters")
    holds = json.loads(TEACHING_FILES["holds"].read_text(encoding="utf-8"))
    for name in TEACHING_CHAPTERS:
        before = STAGE07_ROOT / "before-teaching05-08-01" / name
        per = holds["per_file"][name]["sha256"]
        if changed[name]["after"] != file_hash[name] or per["new"] != file_hash[name] or sums.get(f"{REL}/src/chapters/{name}") != file_hash[name]:
            stop(f"{name}: the candidate's bytes are not the repair's sealed after-bytes")
        if changed[name]["before"] != file_sha(before) or per["old"] != file_sha(before) or file_sha(STAGE06_CH / name) != file_sha(before):
            stop(f"{name}: root's preserved original is not the repair's before-bytes")
    for flag in ("native_validation", "picture_acceptance", "listening"):
        if receipt.get(flag) is not False:
            stop(f"{TEACHING_RECEIPT} claims {flag}")
    rows = json.loads(TEACHING_FILES["rows"].read_text(encoding="utf-8"))
    if rows["phrase_mark_uses"] != {n: PHRASE_MARK_USES[n] for n in rows["phrase_mark_uses"]} or sorted(rows["phrase_mark_uses"]) != sorted(set(TEACHING_CHAPTERS) - {"branch_cut.rs"}):
        stop("PHRASE_MARK_USES differs from the repair's binder-rows.json")
    if rows["fallback"] != {n: FALLBACK[n] for n in rows["fallback"]}:
        stop("FALLBACK differs from the repair's binder-rows.json")
    if {(n, line): kind for n, line, kind, _ in rows["sites"]} != {k: v[0] for k, v in SITES.items() if k[0] in TEACHING_CHAPTERS}:
        stop("the SITES rows for the repaired chapters differ from the repair's binder-rows.json")
    if {tuple(r) for r in rows["added_anchors"]} != TEACHING_ANCHORS:
        stop("the added anchors differ from the repair's binder-rows.json")
    proposed = json.loads(TEACHING_FILES["anchors"].read_text(encoding="utf-8"))["anchors"]
    if {(r["cue"], r["phrase"]) for r in proposed} != TEACHING_ANCHORS or len(proposed) != len(TEACHING_ANCHORS):
        stop("phrase-anchors-proposed.json no longer proposes exactly the eight anchors")
    for want in proposed:
        got = next((r for r in anchor_rows if (r["cue"], r["phrase"]) == (want["cue"], want["phrase"])), None)
        if got is None:
            stop(f"{want['cue']}: no anchor on the phrase the repair proposes, {want['phrase']!r}")
        for field in want:
            if got[field] != want[field]:
                stop(f"{want['cue']} {want['phrase']!r}: {field} is {got[field]!r}, the repair says {want[field]!r}")
    by_cue = {r["cue"]: r for r in hold_rows}
    for want in holds["holds"]:
        got = by_cue.get(want["cue"])
        new = want["new"]
        if got is None or (got["file"], got["next_cue"], got["marker"], got["source_sha256"], got["say_line"], got["marker_line"], got["next_say_line"], got["interval_sha256"]) != (want["file"], want["next_cue"], want["marker"], new["sha256"], new["say_line"], new["marker_line"], new["next_say_line"], new["interval_sha256"]):
            stop(f"{want['cue']}: the hold differs from the repair's hold-table.json")
    if sorted(r["cue"] for r in hold_rows if r["file"] in TEACHING_CHAPTERS) != sorted(h["cue"] for h in holds["holds"]):
        stop("the holds in the repaired chapters are not the repair's seven")
    return {
        "receipt": {"path": str(STAGE07_ROOT / TEACHING_RECEIPT), "sha256": file_sha(STAGE07_ROOT / TEACHING_RECEIPT)},
        "patch": {"path": str(TEACHING_FILES["patch"]), "sha256": file_sha(TEACHING_FILES["patch"])},
        "sealed_rows": {k: {"path": str(TEACHING_FILES[k]), "sha256": file_sha(TEACHING_FILES[k])} for k in ("holds", "rows", "anchors")},
    }


def json_changes(old, new, path="$"):
    """Every difference between two JSON values, by path."""
    if isinstance(old, dict) and isinstance(new, dict):
        rows = []
        for key in sorted(set(old) | set(new)):
            if key not in old:
                rows.append({"path": f"{path}.{key}", "added": new[key]})
            elif key not in new:
                rows.append({"path": f"{path}.{key}", "removed": old[key]})
            else:
                rows += json_changes(old[key], new[key], f"{path}.{key}")
        return rows
    if isinstance(old, list) and isinstance(new, list):
        if len(old) == len(new):
            return [row for i, (a, b) in enumerate(zip(old, new)) for row in json_changes(a, b, f"{path}[{i}]")]
        return [{"path": path, "added": [x for x in new if x not in old], "removed": [x for x in old if x not in new]}]
    return [] if old == new else [{"path": path, "old": old, "new": new}]


def scope_by_hash(change):
    """The map's scope note is recorded here by hash. Its text names the stale oliver11 map, and
    RubyRaven's STALE.oliver11_only_as_history census allows that name only in the map and in
    inputs.sha256. Each plan's map holds the text itself."""
    if change["path"] != "$.scope":
        return change
    return {"path": "$.scope", "old_sha256": sha(change["old"].encode("utf-8")), "new_sha256": sha(change["new"].encode("utf-8")), "text": "recorded by hash; each plan's map holds its scope note"}


def increment_compare(outputs):
    """Every change in the plan against binder02's sealed plan. The request rows must not change."""
    rows = []
    for name in BINDER02_PLAN_FILES:
        old_text = (BINDER02_PLAN / name).read_text(encoding="utf-8")
        new_name = name.replace("narration-direction-map-oliver14.json", MAP_NAME)
        new_text = outputs[new_name]
        row = {"file": new_name, "binder02_file": name, "binder02_sha256": sha(old_text.encode("utf-8")), "binder03_sha256": sha(new_text.encode("utf-8")), "identical": old_text == new_text}
        if name == "inputs.sha256":
            a, b = old_text.splitlines(), new_text.splitlines()
            row["lines_added"] = [x for x in b if x not in a]
            row["lines_removed"] = [x for x in a if x not in b]
        elif not row["identical"]:
            row["changes"] = [scope_by_hash(c) for c in json_changes(json.loads(old_text), json.loads(new_text))]
        if name == "cue-delta.json" and not row["identical"]:
            stop("the cue delta changed, but the teaching repair changes no cue and no request")
        rows.append(row)
    return {
        "schema_version": 1,
        "claim": "Text comparison of two plans. Nothing compiled, rendered, measured or heard. No take is selected; selection for all 80 cues remains pending.",
        "from": str(BINDER02_PLAN),
        "added": [
            "LavenderPike's chapter 05-08 teaching repair, integrated by root with integration-partial07.json",
            "eight phrase anchors (05_20, 06_6, 08_5), read through until_phrase; three word-sync waits",
            "seven holds rebound to the repaired files' hashes; 05_5's say and marker lines move 13 lines",
            "the map under a new name, oliver15",
        ],
        "map_name": {"binder02": "narration-direction-map-oliver14.json", "binder03": MAP_NAME},
        "files": rows,
    }


def stage06_compare(texts, old_texts, names, file_hash, coherent, audit_doc, phrase_doc, map_doc, sentence_new, sentence_other, script_sha, file_lines, map_sha):
    """What changed against stage06's bound plan. Stage06's own values are recomputed from its bytes first."""
    old_audit = json.loads((STAGE06_PLAN / "hold-audit.json").read_text(encoding="utf-8"))
    old_phrase = json.loads((STAGE06_PLAN / "phrase-anchors.json").read_text(encoding="utf-8"))
    old_map_path = STAGE06_PLAN / REL / "docs" / STAGE06_MAP_NAME
    old_map = json.loads(old_map_path.read_text(encoding="utf-8"))
    old_lines = {n: source(STAGE06_CH / n)[0] for n in names}
    old_hash = {n: file_sha(STAGE06_CH / n) for n in names}

    # ---- holds
    new_rows = {r["cue"]: r for r in audit_doc["prediction_holds"] + audit_doc["reading_holds"]}
    old_rows = {r["cue"]: r for r in old_audit["prediction_holds"] + old_audit["reading_holds"]}
    new_visual = {r["cue"]: r for r in map_doc["visual_holds"]}
    old_visual = {r["cue"]: r for r in old_map["visual_holds"]}
    if set(new_rows) != set(old_rows) or set(new_visual) != set(old_visual) or set(new_rows) != set(new_visual):
        stop(f"the held cues changed: {sorted(set(new_rows) ^ set(old_rows))}")
    holds = []
    for cue in [r["cue"] for r in map_doc["visual_holds"]]:
        a, b = old_rows[cue], new_rows[cue]
        name = b["file"]
        if a["file"] != name or a["next_cue"] != b["next_cue"]:
            stop(f"{cue}: the hold moved to another file or next cue")
        old_interval = old_lines[name][a["say_line"] - 1 : a["marker_line"]]
        new_interval = file_lines[name][b["say_line"] - 1 : b["marker_line"]]
        if sha("\n".join(old_interval).encode("utf-8")) != a["interval_sha256"] or a["interval_sha256"] != old_visual[cue]["interval_sha256"]:
            stop(f"{cue}: the interval formula does not reproduce stage06's own value")
        if old_hash[name] != a["source_sha256"]:
            stop(f"{cue}: stage06's bytes are not what its plan names")
        holds.append({
            "cue": cue,
            "next_cue": b["next_cue"],
            "kind": new_visual[cue]["kind"],
            "file": name,
            "file_sha256": {"stage06": a["source_sha256"], "stage07": b["source_sha256"], "changed": a["source_sha256"] != b["source_sha256"]},
            "say_line": {"stage06": a["say_line"], "stage07": b["say_line"], "moved": a["say_line"] != b["say_line"]},
            "marker_line": {"stage06": a["marker_line"], "stage07": b["marker_line"], "moved": a["marker_line"] != b["marker_line"]},
            "marker": {"stage06": a["marker"], "stage07": b["marker"], "changed": a["marker"] != b["marker"]},
            "next_say_line": {"stage06": a["next_say_line"], "stage07": b["next_say_line"]},
            "interval_sha256": {"stage06": a["interval_sha256"], "stage07": b["interval_sha256"], "changed": a["interval_sha256"] != b["interval_sha256"]},
            "interval_lines": {"stage06": len(old_interval), "stage07": len(new_interval)},
            "interval_diff": [d for d in difflib.unified_diff(old_interval, new_interval, lineterm="", n=0) if not d.startswith(("---", "+++"))],
            "cue_text_changed": a["cue_text_sha256"] != b["cue_text_sha256"],
            "after_marker_code_changed": [r["code"] for r in a["code_between_marker_and_next_say"]] != [r["code"] for r in b["code_between_marker_and_next_say"]],
        })
    # ---- other waits, paired by file and marker, in order
    pools = {}
    for w in old_audit["other_waits"]:
        pools.setdefault((w["file"], w["marker"]), []).append(w)
    waits, added = [], []
    for w in audit_doc["other_waits"]:
        pool = pools.get((w["file"], w["marker"]), [])
        if not pool:
            added.append({k: w[k] for k in ("file", "line", "cue", "class", "marker")})
            continue
        o = pool.pop(0)
        waits.append({
            "file": w["file"], "marker": w["marker"], "class": w["class"],
            "line": {"stage06": o["line"], "stage07": w["line"], "moved": o["line"] != w["line"]},
            "census_cue": {"stage06": o["cue"], "stage07": w["cue"], "changed": o["cue"] != w["cue"]},
            "class_changed": o["class"] != w["class"],
        })
    removed = [{k: o[k] for k in ("file", "line", "cue", "class", "marker")} for pool in pools.values() for o in pool]
    # ---- finishes
    old_finish = {f["file"]: f for f in old_audit["chapter_finishes"]}
    finishes = [
        {"file": f["file"], "cue": f["cue"], "line": {"stage06": old_finish[f["file"]]["line"], "stage07": f["line"]}}
        for f in audit_doc["chapter_finishes"] if old_finish[f["file"]]["line"] != f["line"]
    ]
    # ---- sentence hooks
    sentence_old, other_old = sentence_census(STAGE06_CH, names, old_texts, coherent)
    if len(sentence_old) != old_audit["counts"]["literal_sentence_hooks"]:
        stop("the census does not reproduce stage06's count of literal sentence hooks")
    if len(sentence_new) != audit_doc["counts"]["literal_sentence_hooks"]:
        stop("the census does not reproduce this run's count of literal sentence hooks")
    key = lambda r: (r["file"], r["hook"], r["cue"], r["sentence"], r["current"])
    old_c, new_c = Counter(map(key, sentence_old)), Counter(map(key, sentence_new))
    fields = ("file", "hook", "cue", "sentence", "current")
    shown = lambda c, rows: sorted(({**dict(zip(fields, k)), "times": v, "lines": sorted(r["line"] for r in rows if key(r) == k)} for k, v in c.items()), key=lambda r: (r["file"], r["lines"]))
    per_file = {
        n: {
            "stage06": sum(1 for r in sentence_old if r["file"] == n),
            "stage07": sum(1 for r in sentence_new if r["file"] == n),
            "lines_moved": [r["line"] for r in sentence_new if r["file"] == n] != [r["line"] for r in sentence_old if r["file"] == n],
        }
        for n in names
    }
    # ---- phrase anchors
    old_a = {(r["cue"], r["phrase"]): r for r in old_phrase["anchors"]}
    new_a = {(r["cue"], r["phrase"]): r for r in phrase_doc["anchors"]}
    if set(old_a) - set(new_a) or set(new_a) - set(old_a) != ADDED_ANCHORS:
        stop(f"the phrase anchors changed: {sorted(set(old_a) ^ set(new_a))}")
    anchor_rows, bytes_changed = [], []
    for k, b in new_a.items():
        if k in ADDED_ANCHORS:
            continue
        a = old_a[k]
        for f in ("byte", "byte_end", "sentence_index", "word_index", "const"):
            if a[f] != b[f]:
                bytes_changed.append({"cue": k[0], "phrase": k[1], "field": f, "stage06": a[f], "stage07": b[f]})
        anchor_rows.append({
            "cue": k[0], "phrase": k[1], "byte": b["byte"],
            "source_path": b["source_path"],
            "source_sha256": {"stage06": a["source_sha256"], "stage07": b["source_sha256"], "changed": a["source_sha256"] != b["source_sha256"]},
            "cue_text_changed": a["cue_text_sha256"] != b["cue_text_sha256"],
            "hook_lines": {"stage06": [s["line"] for s in a["hook_sites"]], "stage07": [s["line"] for s in b["hook_sites"]]},
        })
    old_share = old_phrase["sentence_share_hooks"]["rows"]
    new_share = phrase_doc["sentence_share_hooks"]["rows"]
    if [(r["cue"], r["sentence_index"], r["const"], r["share_of_sentence"]) for r in old_share] != [(r["cue"], r["sentence_index"], r["const"], r["share_of_sentence"]) for r in new_share]:
        stop("the sentence-share hooks changed")
    # ---- the helpers and consumers the holds rely on
    evidence = lambda doc: {(r["path"], r["marker"]): (r["sha256"], r["line"]) for r in doc["helpers"] + doc["consumers"]}
    old_e, new_e = evidence(old_audit), evidence(audit_doc)
    if set(old_e) != set(new_e):
        stop(f"the helper and consumer rows changed: {sorted(set(old_e) ^ set(new_e))}")
    evidence_changed = [
        {"path": k[0], "marker": k[1], "stage06": {"sha256": old_e[k][0], "line": old_e[k][1]}, "stage07": {"sha256": new_e[k][0], "line": new_e[k][1]}}
        for k in sorted(new_e) if old_e[k] != new_e[k]
    ]
    # ---- files
    chapter_files = [
        {"file": n, "stage06": old_hash[n], "stage07": file_hash[n], "changed": old_hash[n] != file_hash[n]}
        for n in names
    ]
    return {
        "schema_version": 1,
        "claim": "Text comparison of two sets of bytes and two bound plans. Stage06's holds, intervals and hook count are recomputed from its bytes before they are compared. Nothing compiled, rendered, measured or heard.",
        "from": {"plan": str(STAGE06_PLAN), "script_sha256": ROUNDING["before_sha256"], "map": STAGE06_MAP_NAME, "map_sha256": file_sha(old_map_path)},
        "to": {"crate": str(STAGE), "script_sha256": script_sha, "map": MAP_NAME, "map_sha256": map_sha},
        "wording": {
            "cue": ROUNDING["cue"],
            "old": ROUNDING["old"],
            "new": ROUNDING["new"],
            "text_sha256": {"stage06": sha(old_texts[ROUNDING["cue"]].encode("utf-8")), "stage07": sha(texts[ROUNDING["cue"]].encode("utf-8"))},
            "words": {"stage06": len(old_texts[ROUNDING["cue"]].split()), "stage07": len(texts[ROUNDING["cue"]].split())},
            "sentences": {"stage06": len(coherent.sentences(old_texts[ROUNDING["cue"]])), "stage07": len(coherent.sentences(texts[ROUNDING["cue"]]))},
            "is_a_held_cue": ROUNDING["cue"] in new_rows,
            "phrase_anchors_on_cue": sum(1 for k in new_a if k[0] == ROUNDING["cue"]),
            "sentence_reads_on_cue": [{k: r[k] for k in ("file", "line", "hook", "sentence")} for r in sentence_new if r["cue"] == ROUNDING["cue"]],
            "take": "Root's review of this wording records the cue as NOT generated. A take of the old text cannot bind the new text hash.",
        },
        "chapter_files": chapter_files,
        "holds": holds,
        "hold_summary": {
            "rows": len(holds),
            "file_hash_changed": [h["cue"] for h in holds if h["file_sha256"]["changed"]],
            "marker_line_moved": [h["cue"] for h in holds if h["marker_line"]["moved"]],
            "say_line_moved": [h["cue"] for h in holds if h["say_line"]["moved"]],
            "marker_text_changed": [h["cue"] for h in holds if h["marker"]["changed"]],
            "interval_changed": [h["cue"] for h in holds if h["interval_sha256"]["changed"]],
            "unchanged_entirely": [h["cue"] for h in holds if not h["file_sha256"]["changed"]],
        },
        "other_waits": {
            "paired": waits,
            "added": added,
            "removed": removed,
            "moved_or_recued": [w for w in waits if w["line"]["moved"] or w["census_cue"]["changed"]],
        },
        "chapter_finishes_moved": finishes,
        "helper_and_consumer_rows": {"count": len(new_e), "changed_since_stage06": evidence_changed},
        "sentence_hooks": {
            "literal": {"stage06": len(sentence_old), "stage07": len(sentence_new)},
            "per_file": per_file,
            "added": shown(new_c - old_c, sentence_new),
            "removed": shown(old_c - new_c, sentence_old),
            "other_sentence_reads": {"stage06": other_old, "stage07": sentence_other},
            "cue_differs_from_current": {"stage06": old_audit["sentence_hooks"]["cue_differs_from_current"], "stage07": audit_doc["sentence_hooks"]["cue_differs_from_current"]},
            "non_literal": {"stage06": old_audit["sentence_hooks"]["non_literal"], "stage07": audit_doc["sentence_hooks"]["non_literal"]},
        },
        "phrase_anchors": {
            "count": {"stage06": len(old_a), "stage07": len(new_a)},
            "added": [{f: new_a[k][f] for f in ("cue", "phrase", "byte", "byte_end", "const", "sentence_index", "word_index", "source_path", "source_sha256", "cue_text_sha256", "hook_sites", "measured_seconds")} for k in sorted(ADDED_ANCHORS)],
            "byte_or_text_position_changed": bytes_changed,
            "hook_lines_moved": [r for r in anchor_rows if r["hook_lines"]["stage06"] != r["hook_lines"]["stage07"]],
            "source_hash_changed": sum(1 for r in anchor_rows if r["source_sha256"]["changed"]),
            "rows": anchor_rows,
        },
        "sentence_share_hooks": {
            "unchanged_in_cue_sentence_const_and_share": True,
            "lines": [{"cue": r["cue"], "const": r["const"], "stage06": o["line"], "stage07": r["line"]} for o, r in zip(old_share, new_share)],
        },
        "stale_pairs": {
            "note": "other_waits_in_final_interval uses the census's line-order cue. read_out's waits at ratio_ruler.rs:555 run during 00_1 but are inside the voice, so they do not make 00_1 -> 00_2 a hold.",
            "stage06": [{"cue": r["cue"], "other_waits_in_final_interval": r["other_waits_in_final_interval"]} for r in old_audit["removed_pairs"]],
            "stage07": [{"cue": r["cue"], "other_waits_in_final_interval": r["other_waits_in_final_interval"]} for r in audit_doc["removed_pairs"]],
        },
    }


def storyboard_tables():
    text = (STAGE / "docs/storyboard.md").read_text(encoding="utf-8")

    def rows(heading):
        section = text.split(heading, 1)[1].split("\n## ", 1)[0]
        return [[c.strip().strip("*") for c in line.strip("|").split("|")] for line in section.split("\n") if re.match(r"\| \**\d\d_\d+\** \|", line)]

    return rows("## Prediction holds"), rows("## Authored reading holds"), text


def build(final_out=FINAL_OUT):
    coherent = load_coherent()
    script_path = STAGE / "script.tsv"
    script_bytes = script_path.read_bytes()
    script_sha = sha(script_bytes)
    if script_sha != EXPECTED_SCRIPT:
        stop(f"stage07's script is {script_sha}, not the 80 cues with the 01_5 rounding wording")
    texts = coherent.read_script(script_path)
    cues = list(texts)
    if len(cues) != 80:
        stop(f"{len(cues)} cues, not 80")
    if (STAGE / "docs" / MAP_NAME).exists():
        stop(f"stage07's candidate already has a {MAP_NAME}")
    reg = registered()
    names = [n for _, n in reg]
    says, sites = census(names, cues)
    say_at = {s["cue"]: s for s in says}
    file_lines = {n: source(CH / n)[0] for n in names}
    file_hash = {n: file_sha(CH / n) for n in names}
    text_hash = {c: sha(t.encode("utf-8")) for c, t in texts.items()}

    # ---------------------------------------------------------------- helpers and consumers
    draw_path = CH / "draw.rs"
    draw_text = draw_path.read_text(encoding="utf-8")
    draw_display = f"{REL}/src/chapters/draw.rs"
    predict_body = re.search(r"^pub fn predict\(.*?^}", draw_text, re.M | re.S)
    if not predict_body:
        stop("draw.rs: predict not found")
    body = predict_body.group(0)
    order = ["kit.hold(stage)?;", 'kit.mark(stage, "predict");', "kit.music_breath(stage, THINK)?;", "fade_out_all(stage, &[bar, probe], "]
    places = [body.find(x) for x in order]
    if -1 in places or places != sorted(places):
        stop("draw::predict no longer holds, marks, waits and fades in that order")
    think = float(constant(r"^pub const THINK: f64 = ([0-9.]+);", draw_text, "draw.rs"))
    fade = float(constant(r"fade_out_all\(stage, &\[bar, probe\], ([0-9.]+)\)", body, "draw.rs"))
    narration_text = Path("crates/explainer_kit/src/narration.rs").read_text(encoding="utf-8")
    breath = float(constant(r"^pub const BREATH: f64 = ([0-9.]+);", narration_text, "narration.rs"))
    main_text = (STAGE / "src/main.rs").read_text(encoding="utf-8")
    if "line_gaps: explainer_kit::narration::LineGaps::Constant" not in main_text:
        stop("main.rs no longer uses LineGaps::Constant")
    floor = round(breath + think + fade, 3)
    kit = "crates/explainer_kit/src/kit.rs"
    nar = "crates/explainer_kit/src/narration.rs"
    helpers = [
        evidence_row(draw_display, draw_path, "pub const THINK: f64"),
        evidence_row(draw_display, draw_path, "pub fn predict(kit: &Kit"),
        evidence_row(draw_display, draw_path, 'kit.mark(stage, "predict");'),
        evidence_row(draw_display, draw_path, "kit.music_breath(stage, THINK)?;"),
        evidence_row(draw_display, draw_path, "fade_out_all(stage, &[bar, probe], "),
        evidence_row(f"{REL}/src/main.rs", STAGE / "src/main.rs", "LineGaps::Constant"),
        evidence_row(kit, kit, "pub fn hold(&self, stage: &mut Stage<'_>)"),
        evidence_row(kit, kit, "pub fn mark(&self, stage: &Stage<'_>, label: &str)"),
        evidence_row(kit, kit, "pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64)"),
        evidence_row(nar, nar, "pub const BREATH: f64"),
        evidence_row(nar, nar, "pub fn hold(&self, stage: &mut Stage<'_>)"),
        evidence_row(nar, nar, "pub fn mark(&self, stage: &Stage<'_>, label: &str)"),
        evidence_row(nar, nar, "pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64)"),
        evidence_row(nar, nar, "fn window(&self, stage: &mut Stage<'_>, seconds: f64, label: Option<&str>)"),
        evidence_row(nar, nar, 'let timing_path = dir.join("phrase-timings.json");'),
    ]
    consumers = [
        evidence_row("tools/narration_delivery.py", "tools/narration_delivery.py", "def prediction_end(line, following, fps, predicts, outputs, declared, offset):"),
        evidence_row("tools/narration_delivery.py", "tools/narration_delivery.py", "if h.get('video') == topic.name and h.get('kind') == 'prediction')"),
        evidence_row("crates/explainer_kit/src/captions.rs", "crates/explainer_kit/src/captions.rs", "fn prediction_end(chapter: &ChapterTimeline, line_index: usize, cue_duration: f64)"),
        evidence_row("tools/final_check.py", "tools/final_check.py", 'held = (i in hold_ends and abs(take[-1]["end"] - hold_ends[i]) < 1e-6'),
        evidence_row("crates/explainer_score/src/score.rs", "crates/explainer_score/src/score.rs", "pub fn from_label(label: &str) -> Self {"),
        evidence_row("tools/coherent_narration.py", "tools/coherent_narration.py", "def direction_contract(script_path, plan):"),
        evidence_row("tools/coherent_narration.py", "tools/coherent_narration.py", "def check_dispositions(directions, lines):"),
    ]

    # ---------------------------------------------------------------- classify every wait
    predict_sites = [s for s in sites if s["call"] == "predict"]
    breath_sites = [s for s in sites if s["call"] == "kit.music_breath"]
    other_waits, finishes = [], []
    for s in sites:
        if s["call"] in ("predict", "kit.music_breath"):
            continue
        if s["call"] == "kit.finish":
            finishes.append({k: s[k] for k in ("file", "line", "cue", "marker")})
            continue
        key = (s["file"], s["line"])
        if key not in SITES:
            stop(f"{key}: a wait this audit has not classified: {s['marker']}")
        kind, note = SITES[key]
        other_waits.append({**{k: s[k] for k in ("file", "line", "cue", "marker")}, "class": kind, "note": note})
    seen = {(s["file"], s["line"]) for s in sites}
    if set(SITES) - seen:
        stop(f"classified waits that are not in the source: {sorted(set(SITES) - seen)}")
    if len(predict_sites) != PREDICTIONS:
        stop(f"{len(predict_sites)} predict calls, not {PREDICTIONS}")
    if sorted((s["file"], s["marker"]) for s in breath_sites) != sorted(READING_MARKERS.items()):
        stop(f"the reading windows changed: {breath_sites}")

    # ---------------------------------------------------------------- hold rows
    def bound(site, kind):
        cue, name, line = site["cue"], site["file"], site["line"]
        lines = file_lines[name]
        nxt = cues[cues.index(cue) + 1]
        same_chapter = nxt[:2] == cue[:2]
        start = say_at[cue]
        if start["file"] != name or not start["line"] < line:
            stop(f"{name}:{line}: the hold is not after its cue's say")
        next_say = None
        if same_chapter:
            next_say = say_at[nxt]["line"]
            if say_at[nxt]["file"] != name or not line < next_say:
                stop(f"{name}:{line}: the hold is not before {nxt}'s say")
        between_says = [s["cue"] for s in says if s["file"] == name and start["line"] < s["line"] < line]
        if between_says:
            stop(f"{name}:{line}: {between_says} said between {cue} and its hold")
        interval = "\n".join(lines[start["line"] - 1 : line])
        row = {
            "video": TOPIC,
            "cue": cue,
            "next_cue": nxt,
            "kind": kind,
            "path": f"{REL}/src/chapters/{name}",
            "sha256": file_hash[name],
            "line": line,
            "marker": site["marker"],
            "interval_sha256": sha(interval.encode("utf-8")),
        }
        audit = {
            "cue": cue,
            "next_cue": nxt,
            "next_cue_crosses_chapter": not same_chapter,
            "file": name,
            "source_sha256": file_hash[name],
            "say_line": start["line"],
            "marker_line": line,
            "marker": site["marker"],
            "next_say_line": next_say,
            "interval_sha256": row["interval_sha256"],
            "cue_text": texts[cue],
            "cue_text_sha256": text_hash[cue],
            "cue_words": len(texts[cue].split()),
            "final_sentence": coherent.sentences(texts[cue])[-1],
            "code_between_marker_and_next_say": after_marker(lines, line, next_say),
        }
        return row, audit

    story_predictions, story_readings, story_text = storyboard_tables()
    story_answered = {r[0]: r[2] for r in story_predictions}
    visual, prediction_audit = [], []
    for s in sorted(predict_sites, key=lambda s: cues.index(s["cue"])):
        row, audit = bound(s, "prediction")
        if audit["next_cue_crosses_chapter"]:
            stop(f"{s['cue']}: a prediction at a chapter's end")
        if audit["code_between_marker_and_next_say"]:
            stop(f"{s['cue']}: code between predict and the next say: {audit['code_between_marker_and_next_say']}")
        row["basis"] = PREDICTION_BASIS
        row["reason"] = PREDICTION_REASON
        row["evidence"] = {"path": row["path"], "sha256": row["sha256"]}
        visual.append(row)
        audit.update(
            final_sentence_is_question=audit["final_sentence"].endswith("?"),
            helper="draw::predict",
            source_floor_after_voice_s=floor,
            floor_composition={"breath_s": breath, "think_s": think, "bar_fade_s": fade},
            floor_note="Voice end plus breath, the mark, THINK of music, the bar's fade, then the next say, up to frame rounding. A floor: a picture that outlasts the voice delays kit.hold, which source alone cannot rule out. None of it is silence in any take.",
            storyboard_answered_in=story_answered.get(s["cue"]),
        )
        prediction_audit.append(audit)
    if [r["cue"] for r in visual] != [r[0] for r in story_predictions]:
        stop("the storyboard's prediction table differs from the source's predict calls")

    # ---------------------------------------------------------------- reading holds
    reading_audit = []
    for s in sorted(breath_sites, key=lambda s: cues.index(s["cue"])):
        row, audit = bound(s, "reading")
        name = s["file"]
        lines = file_lines[name]
        if name == "doing_nothing.rs":
            detail = reading_10_4()
            evidence = {
                "shown_from": one_line(lines, "let shown_from = stage.scene().time().to_f64();", name),
                "hold": 598,
                "read_for": one_line(lines, "let read_for = stage.scene().time().to_f64() - shown_from;", name),
                "condition": one_line(lines, "if read_for < FINE_PRINT_S {", name),
                "window": s["line"],
                "finish": finishes[[f["file"] for f in finishes].index(name)]["line"],
                "floor_const": one_line(lines, "const FINE_PRINT_S: f64", name),
            }
            if lines[evidence["hold"] - 1].strip() != "kit.hold(stage)?;":
                stop("doing_nothing.rs: the reading's kit.hold moved")
            audit.update(
                source_floor_after_voice_s=breath,
                floor_composition={"breath_s": breath, "window_s": None},
                window_seconds=None,
                window_formula="FINE_PRINT_S - read_for, only if read_for < FINE_PRINT_S",
                conditional=True,
                note="Conditional, and the chapter's last hold. read_for runs from when the fine print has faded in to voice end plus breath, so the window's length is a property of the take, and it can be zero. No number is claimed until a take is measured. The next cue is the first line of chapter 11, across a chapter join. No predict mark, so no caption is extended.",
                line_evidence=evidence,
                **detail,
            )
            row["basis"] = "Direct registered scene cue interval and native conditional reading hold. Source only; the window's length depends on the take, and it can be zero. Not audible or picture acceptance."
        else:
            detail = reading_11_20()
            evidence = {
                "theorem_const": one_line(lines, "const THEOREM: [&str; 4] = [", name),
                "rate_const": one_line(lines, "const READING_RATE: f64", name),
                "rule_seconds": one_line(lines, "fn rule_seconds() -> f64 {", name),
                "block_examples": one_line(lines, "fn block_examples() -> fmn::Result<[BlockRow; 4]> {", name),
                "hold": 1555,
                "rule_read": one_line(lines, "let rule_read = rule_seconds();", name),
                "row_reads": one_line(lines, "let row_reads = block_examples()?.map(|(_, verdict)| read_seconds(&verdict));", name),
                "window": s["line"],
                "next_say": audit["next_say_line"],
            }
            if lines[evidence["hold"] - 1].strip() != "kit.hold(stage)?;":
                stop("unreachable.rs: the reading's kit.hold moved")
            if detail["rule_chars"] != 159 or detail["row_chars"] != 107 or detail["window_seconds"] != 17.7333:
                stop(f"11_20's reading is not the 159 + 107 characters of Mail 1599: {detail}")
            for needle in ("159 characters", "107 characters", "17.73 s"):
                if needle not in story_text:
                    stop(f"the storyboard no longer says {needle}")
            audit.update(
                conditional=False,
                source_floor_after_voice_s=round(breath + detail["window_seconds"], 4),
                floor_composition={"breath_s": breath, "window_s": detail["window_seconds"]},
                window_formula="(rule_chars + row_chars) / READING_RATE",
                note="Unconditional. The clock starts at voice end plus breath; the next cue's say waits for the window. A music window, not silence in any take. Root may shorten the row walk after listening; the rule's 10.6 s is the floor. No predict mark, so no caption is extended.",
                line_evidence=evidence,
                **detail,
            )
            row["basis"] = "Direct registered scene cue interval and native reading hold; 159 + 107 characters at 15 a second, from the source's literals. Source only; not audible or picture acceptance."
        row["reason"] = READING_REASON
        row["evidence"] = {"path": row["path"], "sha256": row["sha256"]}
        visual.append(row)
        reading_audit.append(audit)
    visual.sort(key=lambda r: cues.index(r["cue"]))

    overrides = []
    for r in visual:
        overrides.append({
            "video": TOPIC,
            "cue": r["cue"],
            "after": texts[r["cue"]],
            "role": "visual",
            "reason": PREDICTION_REASON if r["kind"] == "prediction" else READING_REASON,
        })
    shared = sorted({(h["path"], h["sha256"]) for h in helpers})
    direction_map = {
        "schema_version": 1,
        "scripts": {TOPIC: script_sha},
        "overrides": overrides,
        "visual_holds": visual,
        "hold_kinds": {"prediction": len(prediction_audit), "reading": len(reading_audit)},
        "shared_helpers": [{"path": p, "sha256": h} for p, h in shared],
        "scope": MAP_SCOPE,
    }
    map_text = json.dumps(direction_map, indent=2, ensure_ascii=False) + "\n"
    units = {
        "schema_version": 1,
        "script_sha256": script_sha,
        "direction_contract": {
            "schema_version": 1,
            "topic": TOPIC,
            "script_sha256": script_sha,
            "map": {"path": f"{REL}/docs/{MAP_NAME}", "sha256": sha(map_text.encode("utf-8"))},
        },
        "scope": UNITS_SCOPE,
        "units": [{"id": f"oliver_{c}", "line_ids": [c], "text": t} for c, t in texts.items()],
    }
    for u in units["units"]:
        if not 1 <= len(u["text"].split()) <= 100:
            stop(f"{u['id']}: outside the coherent-unit word bound")
    units_text = json.dumps(units, indent=2, ensure_ascii=False) + "\n"

    # ---------------------------------------------------------------- the four stale pairs
    stale = json.loads((STAGE / "docs" / STALE_MAP).read_text(encoding="utf-8"))
    if sorted((h["cue"], h["next_cue"]) for h in stale["visual_holds"]) != sorted(OBSOLETE):
        stop(f"{STALE_MAP} does not hold exactly the four pairs Mail 1599 names")
    held = {(r["cue"], r["next_cue"]) for r in visual}
    removed = []
    for h in stale["visual_holds"]:
        pair = (h["cue"], h["next_cue"])
        if pair in held:
            stop(f"{pair} is a real hold in the final source")
        over = next(o for o in stale["overrides"] if o["cue"] == h["cue"])
        old_file = h["path"].rsplit("/", 1)[1]
        old_path = CH / old_file
        in_interval = [s for s in sites if s["cue"] == h["cue"] and s["file"] == say_at[h["cue"]]["file"]]
        if any(s["call"] in ("predict", "kit.music_breath") for s in in_interval):
            stop(f"{pair}: the final source does hold here")
        removed.append({
            "cue": h["cue"],
            "next_cue": h["next_cue"],
            "old_kind": h["kind"],
            "old_path": h["path"],
            "old_line": h["line"],
            "old_marker": h["marker"],
            "old_file_registered": old_file in names,
            "old_file_bytes_match_row": old_path.is_file() and file_sha(old_path) == h["sha256"],
            "old_marker_still_at_old_line": old_path.is_file() and source(old_path)[0][h["line"] - 1].strip() == h["marker"],
            "old_after_is_final_cue_text": over["after"] == texts[h["cue"]],
            "old_after_in_final_cue": " ".join(coherent.direction_words(over["after"])) in " ".join(coherent.direction_words(texts[h["cue"]])),
            "final_cue_says_at": say_at[h["cue"]],
            "final_cue_text": texts[h["cue"]],
            "windows_in_final_interval": [],
            "other_waits_in_final_interval": [{k: w[k] for k in ("file", "line", "class")} for w in other_waits if w["cue"] == h["cue"]],
        })

    # ---------------------------------------------------------------- sentence and phrase hooks
    for n in sentence_counts_problem(texts, coherent):
        stop(n)
    sentence_rows = {"literal": 0, "non_literal": [], "cue_differs_from_current": []}
    anchors = {}
    if re.search(r"until_phrase|phrase_time|phrase_marks", draw_text):
        stop("draw.rs has a phrase hook this audit does not read")
    for name in names:
        lines, scene = source(CH / name)
        consts = dict(re.findall(r'const ([A-Z_]+): &str =\s*"([^"\\]*)";', scene))
        uses = len(re.findall(r"phrase_marks\(", scene))
        if uses != PHRASE_MARK_USES.get(name, 0):
            stop(f"{name}: {uses} phrase_marks uses; {PHRASE_MARK_USES.get(name, 0)} expected")
        current = None
        for n, line in enumerate(scene.split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            m = SAY.search(line)
            if m:
                current = m.group(1)
            found = False
            for pattern, call in HOOKS:
                h = pattern.search(line)
                if h:
                    found = True
                    arg = h.group(2)
                    const = None if arg.startswith('"') else arg
                    phrase = arg.strip('"') if const is None else consts[const]
                    if h.group(1) != current:
                        stop(f"{name}:{n}: a hook on {h.group(1)} while {current} is the current cue")
                    add_anchor(anchors, h.group(1), phrase, const, name, n, call)
            if ("until_phrase(" in line or "phrase_time(" in line) and not found and not HOOK_HELPER_LINE.search(line):
                stop(f"{name}:{n}: a phrase hook this audit cannot read: {line.strip()}")
            for pattern, kind in SENTENCE_HOOKS:
                for cue, j in pattern.findall(line):
                    if not 0 <= int(j) < len(coherent.sentences(texts[cue])):
                        stop(f"{name}:{n}: {kind} {cue} sentence {j} does not exist")
                    sentence_rows["literal"] += 1
                    if cue != current:
                        sentence_rows["cue_differs_from_current"].append({"file": name, "line": n, "hook": kind, "cue": cue, "current": current})
            q = NON_LITERAL_SENTENCE.search(line)
            if q:
                sentence_rows["non_literal"].append({"file": name, "line": n, "cue": q.group(1), "code": line.strip()})
    share_hooks = []
    for name in names:
        _, scene = source(CH / name)
        floats = dict(re.findall(r"^const ([A-Z_]+): f64 = ([0-9.]+);", scene, re.M))
        for n, line in enumerate(scene.split("\n"), 1):
            h = re.search(r'until_share\(&kit, stage, "(\d\d_\d+)", (\d+), \w+, ([A-Z_]+)\)', line)
            if h:
                share_hooks.append({"cue": h.group(1), "sentence_index": int(h.group(2)), "const": h.group(3), "share_of_sentence": float(floats[h.group(3)]), "file": name, "line": n})
    if len(share_hooks) != len(re.findall(r"\buntil_share\(&kit", source(CH / "what_it_forgets.rs")[1])):
        stop("an until_share this audit cannot read")
    census_rows, sentence_other = sentence_census(CH, names, texts, coherent)
    # 04_2's climb reads the mark directly
    undoing, undoing_scene = source(CH / "undoing_growth.rs")
    climb = constant(r'^const CLIMB_PHRASE: &str = "([^"\\]*)";', undoing_scene, "undoing_growth.rs")
    add_anchor(anchors, "04_2", climb, "CLIMB_PHRASE", "undoing_growth.rs", one_line(undoing, 'climb_onset(text, said, s3, s4, &n.phrase_marks("04_2"))', "undoing_growth.rs"), "climb_onset")
    add_anchor(anchors, "04_2", climb, "CLIMB_PHRASE", "undoing_growth.rs", one_line(undoing, "super::draw::wait_until(stage, onset)?;", "undoing_growth.rs"), "wait_until")
    # 03_3's digits read their mark the same way, with the character share as the fallback
    folded, folded_scene = source(CH / "folded_decade.rs")
    forgets = constant(r'^const FORGETS_PHRASE: &str = "([^"\\]*)";', folded_scene, "folded_decade.rs")
    add_anchor(anchors, "03_3", forgets, "FORGETS_PHRASE", "folded_decade.rs", one_line(folded, 'phrase_marks("03_3")', "folded_decade.rs"), "forgets_mark")
    add_anchor(anchors, "03_3", forgets, "FORGETS_PHRASE", "folded_decade.rs", one_line(folded, "spoken, forgets_mark(&kit));", "folded_decade.rs"), "lay_beats")

    # ---------------------------------------------------------------- baseline: root's frozen 75-cue request text
    inventory = json.loads((LANE / "narration-request-inventory01.json").read_text(encoding="utf-8"))
    request_plan = json.loads((LANE / "narration-request-plan01.json").read_text(encoding="utf-8"))
    baseline = {}
    for row in inventory["cues"]:
        data = Path(row["text_path"]).read_bytes()
        if sha(data) != row["text_sha256"]:
            stop(f"{row['text_path']}: root's request text changed")
        baseline[row["cue"]] = data.decode("utf-8")
    # Stage06's 80 cues are the 75-cue request's words in order; stage07 differs from them in one
    # phrase of one cue. The walk reads stage06's text, and the rewording is checked on its own.
    before_path = ROUNDING["before_script"]
    if file_sha(before_path) != ROUNDING["before_sha256"]:
        stop(f"{before_path}: not the script before the rounding wording")
    old_texts = coherent.read_script(before_path)
    if list(old_texts) != cues:
        stop("the rounding wording changed the cue list")
    if [c for c in cues if old_texts[c] != texts[c]] != [ROUNDING["cue"]]:
        stop("the script differs from stage06's in more than the 01_5 rounding wording")
    reworded = old_texts[ROUNDING["cue"]]
    if reworded.count(ROUNDING["old"]) != 1 or reworded.replace(ROUNDING["old"], ROUNDING["new"], 1) != texts[ROUNDING["cue"]]:
        stop("01_5 is not the reviewed rounding wording")
    rounding_bytes = ROUNDING["request"].read_bytes()
    if sha(rounding_bytes) != ROUNDING["request_sha256"] or rounding_bytes != texts[ROUNDING["cue"]].encode("utf-8"):
        stop("the rounding request is not 01_5's exact text")
    if " ".join(baseline.values()) != " ".join(old_texts.values()):
        stop("stage06's 80 cues are not the 75-cue script's words in order")
    final80 = json.loads(FINAL80_REQUESTS.read_text(encoding="utf-8"))
    if final80["script_sha256"] != ROUNDING["before_sha256"] or len(final80["requests"]) != len(cues):
        stop("the final-80 request list does not bind stage06's script")
    for row in final80["requests"]:
        data = Path(row["text_path"]).read_bytes()
        if sha(data) != row["text_sha256"] or data != old_texts[row["cue"]].encode("utf-8") or row["parts"] != 1:
            stop(f"{row['text_path']}: not the final-80 text of {row['cue']}")
    # Root's list for the current script. Its submission state is copied as recorded, never judged.
    requests_now = json.loads(CURRENT_REQUESTS.read_text(encoding="utf-8"))
    if requests_now["script_sha256"] != script_sha or file_sha(requests_now["script_path"]) != script_sha:
        stop("the current request list does not bind this script")
    if requests_now["prior_manifest_sha256"] != file_sha(FINAL80_REQUESTS) or requests_now["supersedes"] != str(FINAL80_REQUESTS):
        stop("the current request list does not name the list it supersedes")
    if [r["cue"] for r in requests_now["requests"]] != cues:
        stop("the current request list is not one row per cue, in order")
    if "Oliver" not in requests_now["voice"]:
        stop("the current request list is not for ElevenLabs Oliver")
    current_request = {}
    for row in requests_now["requests"]:
        data = Path(row["text_path"]).read_bytes()
        if row["id"] != row["cue"] or row["parts"] != 1 or sha(data) != row["text_sha256"] or row["file_sha256"] != row["text_sha256"] or data != texts[row["cue"]].encode("utf-8"):
            stop(f"{row['text_path']}: not the current text of {row['cue']}")
        expected_folder = ROUNDING["request"] if row["cue"] == ROUNDING["cue"] else FINAL80_REQUESTS.parent / f"{row['cue']}.txt"
        if row["text_path"] != str(expected_folder):
            stop(f"{row['cue']}: the current request list names {row['text_path']}, not {expected_folder}")
        current_request[row["cue"]] = {
            "path": row["text_path"],
            "sha256": row["text_sha256"],
            "submitted": row["submitted"],
            "submission_state": row["submission_state"],
            "retained_pair_receipt": row.get("retained_pair_receipt"),
        }
    requests = {}
    for row in request_plan["requests"]:
        data = Path(row["text_path"]).read_bytes()
        if sha(data) != row["text_sha256"]:
            stop(f"{row['text_path']}: root's request text changed")
        requests.setdefault(data.decode("utf-8"), []).append(row)
    delta, offset_in = [], {}
    base_cues = list(baseline)
    base_i, used = 0, 0
    for cue, text in texts.items():
        old_text = old_texts[cue]
        base = base_cues[base_i]
        at = baseline[base].find(old_text, used)
        if at != used:
            stop(f"{cue} is not the next text of {base}")
        offset_in[cue] = (base, at)
        match = requests.get(old_text, [])
        if len(match) != 1:
            stop(f"{cue}: {len(match)} request texts with its stage06 bytes")
        if sha(Path(current_request[cue]["path"]).read_bytes()) != text_hash[cue]:
            stop(f"{cue}: no current request file with its exact bytes")
        delta.append({
            "cue": cue,
            "words": len(text.split()),
            "text_sha256": text_hash[cue],
            "baseline_cue": base,
            "baseline_byte": at,
            "text_changed_from_baseline": text != baseline[base],
            "cue_split_from_baseline": old_text != baseline[base],
            "reworded_since_stage06": text != old_text,
            "request_id_with_exact_bytes": None if text != old_text else match[0]["id"],
            "current_request": current_request[cue],
        })
        used = at + len(old_text.encode("utf-8")) + 1
        if used > len(baseline[base].encode("utf-8")):
            base_i, used = base_i + 1, 0

    # ---------------------------------------------------------------- finish the anchors
    anchor_rows = []
    for (cue, phrase), a in anchors.items():
        text = texts[cue]
        raw, wanted = text.encode("utf-8"), phrase.encode("utf-8")
        if raw.count(wanted) != 1:
            stop(f"{cue}: {phrase!r} occurs {raw.count(wanted)} times")
        byte = raw.find(wanted)
        if byte and raw[byte - 1] != 0x20:
            stop(f"{cue}: {phrase!r} does not begin a word")
        opens = sentence_opens(text)
        j = max(i for i, at in enumerate(opens) if at <= byte)
        word_index = len(coherent.direction_words(raw[:byte].decode("utf-8")))
        words = coherent.direction_words(text)
        if words[word_index : word_index + len(coherent.direction_words(phrase))] != coherent.direction_words(phrase):
            stop(f"{cue}: {phrase!r} is not at word {word_index}")
        files = sorted({s["file"] for s in a["sites"]})
        if len(files) != 1 or say_at[cue]["file"] != files[0]:
            stop(f"{cue}: a hook outside its chapter")
        name = files[0]
        if name == "slide_rule.rs" and j != 0:
            stop(f"{cue}: slide_rule's fallback reads the first sentence only")
        if name == "undoing_growth.rs" and j != 3:
            stop(f"{cue}: the climb's fallback reads sentences 3 to 4")
        if name == "folded_decade.rs" and j != 0:
            stop(f"{cue}: lay_beats reads a mark inside the first sentence only")
        base, base_at = offset_in[cue]
        if texts[cue] != old_texts[cue]:
            stop(f"{cue}: a phrase anchor on the reworded cue needs its bytes re-read")
        changed = old_texts[cue] != baseline[base]
        anchor_rows.append({
            "cue": cue,
            "byte": byte,
            "byte_end": byte + len(wanted),
            "phrase": phrase,
            "const": a["const"],
            "sentence_index": j,
            "word_index": word_index,
            "hook_sites": a["sites"],
            "source_path": f"{REL}/src/chapters/{name}",
            "source_sha256": file_hash[name],
            "cue_text_sha256": text_hash[cue],
            "baseline_cue": base,
            "baseline_byte": base_at + byte,
            "affected_by_cue_split": changed,
            "byte_or_cue_moved": base != cue or base_at != 0,
            "fallback_without_mark": FALLBACK[name],
            "measured_seconds": None,
        })
    anchor_rows.sort(key=lambda r: (cues.index(r["cue"]), r["byte"]))
    white = white_markers(anchor_rows, script_sha, file_hash)
    teaching = teaching_rows(anchor_rows, prediction_audit, file_hash)
    literal_readers = set()
    for name in names:
        literal_readers |= {(name, cue) for cue in re.findall(r'phrase_marks\("(\d\d_\d+)"\)', source(CH / name)[1])}
    if literal_readers != {("folded_decade.rs", "03_3"), ("undoing_growth.rs", "04_2")}:
        stop(f"literal phrase_marks readers changed: {sorted(literal_readers)}")
    for cue in {r["cue"] for r in anchor_rows}:
        order_check = [(min(s["line"] for s in r["hook_sites"]), r["byte"]) for r in anchor_rows if r["cue"] == cue]
        order_check.sort()
        if [b for _, b in order_check] != sorted(b for _, b in order_check):
            stop(f"{cue}: the source reaches its phrases out of spoken order")
    affected = [r for r in anchor_rows if r["affected_by_cue_split"]]
    phrase_doc = {
        "schema_version": 1,
        "topic": TOPIC,
        "script_sha256": script_sha,
        "claim": "Exact-byte source inventory. Every measured_seconds is null: no take has been measured, and no word timing is invented or estimated here.",
        "byte_rule": "UTF-8 bytes of the cue's text in script.tsv, where the source's text.find(phrase) lands. The kit's loader needs a mark at exactly this byte (narration.rs reads phrase-timings.json; the chapters look marks up by byte).",
        "wording_since_stage06": "01_5 was reworded and carries no anchor, so no anchor's byte or phrase is affected by it.",
        "affected_means": "affected_by_cue_split: the cue's text differs from root's 75-cue request text, because the cue was split. Its text_sha256 is new, and any timing bound to the old text cannot load. byte_or_cue_moved: the anchor also has a new cue id or byte, as it does in 11_20 and 12_21. The bytes in 11_7 and 12_6 are unchanged, because those cues keep the front of the old cue, though their hashes are new.",
        "unaffected_note": "Every other anchor keeps its cue and byte. All of them still need measured marks, because a mark binds a WAV and no take is bound.",
        "baseline": {"inventory": str(LANE / "narration-request-inventory01.json"), "script_sha256": inventory["script_sha256"]},
        "counts": {
            "anchors": len(anchor_rows),
            "cues_with_anchors": len({r["cue"] for r in anchor_rows}),
            "affected_by_cue_split": len(affected),
            "byte_or_cue_moved": sum(1 for r in anchor_rows if r["byte_or_cue_moved"]),
            "measured": 0,
        },
        "by_chapter_file": {name: sum(1 for r in anchor_rows if r["source_path"].endswith("/" + name)) for name in sorted({r["source_path"].rsplit("/", 1)[1] for r in anchor_rows})},
        "anchors": anchor_rows,
        "direct_mark_readers": {
            "note": "Source that names its cue in a literal phrase_marks call, not through until_phrase or a helper taking the cue as a variable. Each falls back to a character share without a mark. Every measured_seconds is null.",
            "rows": [
                {
                    "cue": r["cue"],
                    "phrase": r["phrase"],
                    "byte": r["byte"],
                    "const": r["const"],
                    "file": r["source_path"].rsplit("/", 1)[1],
                    "source_sha256": r["source_sha256"],
                    "hook_sites": r["hook_sites"],
                    "fallback_without_mark": r["fallback_without_mark"],
                    "evidence": white if r["cue"] == "03_3" else None,
                    "measured_seconds": None,
                }
                for r in anchor_rows if r["cue"] in ("04_2", "03_3")
            ],
        },
        "opening_read_out_estimates": opening_read_out(texts, file_lines["ratio_ruler.rs"]),
        "sentence_share_hooks": {
            "note": "Not byte anchors, and they read no word mark: each waits for a share of a measured sentence. Listed because they sit in split cues; each names its sentence in the new cue.",
            "rows": share_hooks,
        },
    }
    phrase_text = json.dumps(phrase_doc, indent=2, ensure_ascii=False) + "\n"
    template = {
        "claim": "NOT a phrase-timings.json. Every wav_sha256 and seconds is null, so the kit's loader refuses it (narration.rs: \"phrase timing WAV differs\", \"bad time\"). Root's aligner fills it from measured takes; nothing here is a timing.",
        "script_sha256": script_sha,
        "cues": {
            c: {
                "text_sha256": text_hash[c],
                "wav_sha256": None,
                "marks": [{"byte": r["byte"], "seconds": None, "phrase": r["phrase"]} for r in anchor_rows if r["cue"] == c],
            }
            for c in cues
        },
    }
    template_text = json.dumps(template, indent=2, ensure_ascii=False) + "\n"
    delta_doc = {
        "schema_version": 1,
        "claim": "Text comparison with root's frozen request files. It says nothing about whether any recording exists. Root's inventory found no exact local record for the 75-cue text and requires a provider history check before new synthesis. The final-80 request list's own state is reported as that file records it.",
        "request_id_note": "request_id_with_exact_bytes names narration-request-plan01's request, and is null for 01_5, whose new text is not in that plan. current_request is the row of root's current list: the file with the cue's exact current bytes and the submission state as that list records it.",
        "current_request_list": {
            "path": str(CURRENT_REQUESTS),
            "sha256": file_sha(CURRENT_REQUESTS),
            "created_at": requests_now["created_at"],
            "script_sha256": requests_now["script_sha256"],
            "supersedes": requests_now["supersedes"],
            "by_submission_state": dict(sorted(Counter(r["submission_state"] for r in requests_now["requests"]).items())),
            "submitted_true": sum(1 for r in requests_now["requests"] if r["submitted"] is True),
            "provider_history_checked_false": sum(1 for r in requests_now["requests"] if r["provider_history_checked"] is False),
            "claim": "Copied from root's list. Retained or submitted is not selected: no take is selected here, and selection for all 80 cues remains pending. Nothing here authorizes synthesis.",
        },
        "baseline_script_sha256": inventory["script_sha256"],
        "request_plan": str(LANE / "narration-request-plan01.json"),
        "request_plan_binds": request_plan["script_sha256"],
        "root_inventory_counts": inventory["counts"],
        "counts": {
            "cues": len(delta),
            "unchanged_from_baseline": sum(1 for d in delta if not d["text_changed_from_baseline"]),
            "split_cues": sum(1 for d in delta if d["cue_split_from_baseline"]),
            "reworded_since_stage06": sum(1 for d in delta if d["reworded_since_stage06"]),
            "with_exact_request_bytes": sum(1 for d in delta if d["request_id_with_exact_bytes"]),
            "with_current_request_file": sum(1 for d in delta if d["current_request"]),
        },
        "cues": delta,
    }
    delta_text = json.dumps(delta_doc, indent=2, ensure_ascii=False) + "\n"

    # ---------------------------------------------------------------- the audit
    boundary = {}
    for r in prediction_audit:
        boundary[r["cue"]] = "prediction"
    for r in reading_audit:
        boundary[r["cue"]] = "reading"
    for f in finishes:
        boundary.setdefault(f["cue"], "chapter_end")
    cue_rows = []
    for i, cue in enumerate(cues):
        nxt = cues[i + 1] if i + 1 < len(cues) else None
        cue_rows.append({
            "cue": cue,
            "unit": f"oliver_{cue}",
            "words": len(texts[cue].split()),
            "file": say_at[cue]["file"],
            "line": say_at[cue]["line"],
            "next_cue": nxt,
            "next_cue_crosses_chapter": nxt is not None and nxt[:2] != cue[:2],
            "boundary": boundary.get(cue, "say_breath"),
        })
    used = set(names) | {"draw.rs", "mod.rs", "thumbnail.rs"}
    ignored = [{"path": f"{REL}/src/chapters/{p.name}", "sha256": file_sha(p)} for p in sorted(CH.iterdir()) if p.name not in used]
    story_reading_cues = [r[0] for r in story_readings]
    audit_doc = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": "Text audit of Rust source at combined-repair-stage07/candidate. Uncompiled here, unrendered, unmeasured, unheard. Author/source binding, not independent AV or listening acceptance.",
        "open_for_root": ["native build and tests of these exact bytes", "selecting a take for all 80 cues", "measured phrase-timings.json, including 03_3's mark and the eight 05-08 marks", "Oliver sync, the reading windows on real lengths, listening", "independent AV acceptance"],
        "no_invented_silence": "No row adds silence to a take. Every hold is a native visual hold after a complete cue; the proposed dispositions are all native_visual_hold. Floors below are the source's own constants, not measurements.",
        "narration": "ElevenLabs Oliver. No Michael or FrankenTTS.",
        "script_sha256": script_sha,
        "map": f"{REL}/docs/{MAP_NAME}",
        "kit": {"line_gaps": "Constant", "breath_s": breath, "think_s": think, "predict_bar_fade_s": fade},
        "counts": {
            "cues": len(cues),
            "units": len(cues),
            "registered_chapters": len(names),
            "words_total": sum(len(t.split()) for t in texts.values()),
            "words_max": max(len(t.split()) for t in texts.values()),
            "prediction_holds": len(prediction_audit),
            "reading_holds": len(reading_audit),
            "other_waits": len(other_waits),
            "chapter_finishes": len(finishes),
            "literal_sentence_hooks": sentence_rows["literal"],
            "phrase_anchors": len(anchor_rows),
            "predictions_not_ending_in_a_question": [r["cue"] for r in prediction_audit if not r["final_sentence_is_question"]],
        },
        "prediction_holds": prediction_audit,
        "reading_holds": reading_audit,
        "removed_pairs": removed,
        "other_waits": other_waits,
        "chapter_finishes": finishes,
        "sentence_hooks": sentence_rows,
        "storyboard": {
            "prediction_table_matches_source": True,
            "reading_table_cues": story_reading_cues,
            "source_reading_cues": [r["cue"] for r in reading_audit],
            "reading_windows_missing_from_table": [r["cue"] for r in reading_audit if r["cue"] not in story_reading_cues],
        },
        "helpers": helpers,
        "consumers": consumers,
        "registered_chapters": [{"key": k, "file": n, "sha256": file_hash[n]} for k, n in reg],
        "teaching_repair_05_08": {
            "note": "LavenderPike's chapter 05-08 teaching repair, integrated by root. Its seven holds, three word-sync waits and eight anchors above equal its sealed rows; the receipt names its patch and every before and after hash. Source binding only, not native, picture or listening acceptance.",
            **teaching,
        },
        "unregistered_files_ignored": ignored,
        "cues": cue_rows,
    }
    audit_text = json.dumps(audit_doc, indent=2, ensure_ascii=False) + "\n"
    audit_sha = sha(audit_text.encode("utf-8"))
    dispositions = {
        "claim": "Proposed direction_dispositions for root's span manifest, one per direction: every hold stays native, and no silence is inserted. Each row's evidence is hold-audit.json. A proposal; assembly is root's, and no take exists to apply it to.",
        "map_sha256": sha(map_text.encode("utf-8")),
        "direction_dispositions": [
            {
                "override_index": i,
                "cue": r["cue"],
                "kind": "native_visual_hold",
                "reason": (
                    f"{'draw::predict' if r['kind'] == 'prediction' else 'kit.music_breath'} at "
                    f"{r['path'].rsplit('/', 1)[1]}:{r['line']} holds after the complete cue, in the picture and music. "
                    "The take's samples are not changed and no silence is inserted."
                ),
                "evidence": {"path": f"{final_out}/hold-audit.json", "sha256": audit_sha},
            }
            for i, r in enumerate(visual)
        ],
    }
    disposition_text = json.dumps(dispositions, indent=2, ensure_ascii=False) + "\n"

    # ---------------------------------------------------------------- the comparison with stage06
    for name in SAME_NAME_FILES:
        if name in ("src/facts.rs", "oracle.py") and file_sha(STAGE / name) != file_sha(STAGE06 / name):
            stop(f"{name} differs from stage06's, so oracle04.tsv no longer serves")
    named = {name for names_ in RECEIPTS.values() for name in names_}
    changed_files = {name for name in SAME_NAME_FILES if file_sha(STAGE / name) != file_sha(STAGE06 / name)}
    changed_files |= {f"src/chapters/{name}" for name in CH_LISTING if file_sha(CH / name) != file_sha(STAGE06_CH / name)}
    if changed_files != named:
        stop(f"the crate's differences from stage06 are not the integration receipts' files: {sorted(changed_files ^ named)}")
    for receipt, files in RECEIPTS.items():
        body = (STAGE07_ROOT / receipt).read_text(encoding="utf-8")
        for name in files:
            if file_sha(STAGE / name) not in body:
                stop(f"{receipt} does not name the current hash of {name}")
    compare_doc = stage06_compare(texts, old_texts, names, file_hash, coherent, audit_doc, phrase_doc, direction_map, census_rows, sentence_other, script_sha, file_lines, sha(map_text.encode("utf-8")))
    compare_text = json.dumps(compare_doc, indent=2, ensure_ascii=False) + "\n"

    # ---------------------------------------------------------------- inputs
    inputs = [script_path, STAGE / "src/main.rs", STAGE / "src/facts.rs", STAGE / "oracle.py", STAGE / "docs/storyboard.md", STAGE / "docs" / STALE_MAP, STAGE / "narration_units.json", STAGE06_ROOT / "oracle04.tsv"]
    inputs += [STAGE07_ROOT / r for r in RECEIPTS]
    inputs += [ROUNDING["before_script"], ROUNDING["request"], ROUNDING["review"], FINAL80_REQUESTS, CURRENT_REQUESTS, WHITE_MARKERS]
    inputs += [Path(r["text_path"]) for r in final80["requests"]]
    inputs += [STAGE06_PLAN / p for p in STAGE06_PLAN_FILES]
    inputs += [BINDER02_PLAN / p for p in BINDER02_PLAN_FILES]
    inputs += list(TEACHING_FILES.values())
    inputs += [STAGE07_ROOT / "before-teaching05-08-01" / name for name in TEACHING_CHAPTERS]
    inputs += [STAGE06_CH / n for n in names]
    inputs += [CH / "mod.rs", CH / "draw.rs"] + [CH / n for n in names]
    inputs += [Path(p) for p in (kit, nar, "crates/explainer_kit/src/captions.rs", "crates/explainer_score/src/score.rs", "tools/coherent_narration.py", "tools/narration_cache.py", "tools/narration_delivery.py", "tools/final_check.py")]
    inputs += [LANE / "narration-request-inventory01.json", LANE / "narration-request-plan01.json"]
    inputs += sorted({Path(r["text_path"]) for r in inventory["cues"]} | {Path(r["text_path"]) for r in request_plan["requests"]})
    inputs_text = "".join(f"{file_sha(p)}  {p}\n" for p in inputs)
    outputs = {
        f"{REL}/narration_units.json": units_text,
        f"{REL}/docs/{MAP_NAME}": map_text,
        "hold-audit.json": audit_text,
        "phrase-anchors.json": phrase_text,
        "phrase-timings-unmeasured-template.json": template_text,
        "direction-dispositions-proposed.json": disposition_text,
        "cue-delta.json": delta_text,
        "stage07-delta.json": compare_text,
        "inputs.sha256": inputs_text,
    }
    outputs["increment-delta.json"] = json.dumps(increment_compare(outputs), indent=2, ensure_ascii=False) + "\n"
    return outputs, audit_doc, phrase_doc, delta_doc, compare_doc


def add_anchor(anchors, cue, phrase, const, name, line, call):
    row = anchors.setdefault((cue, phrase), {"const": const, "sites": []})
    row["sites"].append({"file": name, "line": line, "call": call})


def sentence_counts_problem(texts, coherent):
    """A line whose two sentence rules disagree would stop the render at phrase_seconds."""
    for cue, text in texts.items():
        if len(sentence_opens(text)) != len(coherent.sentences(text)):
            yield f"{cue}: phrase_seconds and the sentence contract count different sentences"


def main(argv):
    args = list(argv)
    check = "--check" in args
    args = [a for a in args if a != "--check"]
    if not Path("tools/coherent_narration.py").is_file():
        stop("run from the repo root")
    if args == ["--print-pins"]:
        print("PINS = {")
        for path in pin_paths():
            print(f'    "{path}": "{file_sha(path)}",')
        print("}")
        return 0
    out = Path(__file__).resolve().parent / "bound-plan"
    final_out = FINAL_OUT
    while args:
        if args[0] == "--out" and len(args) > 1:
            out = Path(args[1])
        elif args[0] == "--final-out" and len(args) > 1:
            final_out = args[1].rstrip("/")
            if Path(final_out).is_absolute() or ".." in Path(final_out).parts:
                stop("--final-out must be repository-relative")
        else:
            stop(f"unknown arguments: {args}")
        args = args[2:]
    resolved = out.resolve()
    for guarded in (Path(REL), STAGE07_ROOT / "candidate"):
        if resolved == guarded.resolve() or guarded.resolve() in resolved.parents:
            stop(f"{out}: refused; this binder never writes in canonical or in the candidate")
    refuse_stale_inputs()
    print(f"inputs: {len(PINS)} pinned files and the chapter listing match")
    outputs, audit, phrases, delta, compare = build(final_out)
    bad = 0
    for rel, text in outputs.items():
        path = out / rel
        same = path.is_file() and path.read_text(encoding="utf-8") == text
        if not same and path.exists():
            print(f"DIFFERS, not overwritten: {rel}")
            bad += 1
        elif not same and check:
            print(f"MISSING: {rel}")
            bad += 1
        elif not same:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        print(f"{sha(text.encode('utf-8'))}  {rel}")
    c = audit["counts"]
    print(f"{c['cues']} cues, {c['units']} units, {c['words_total']} words, max {c['words_max']}; {c['registered_chapters']} registered chapters")
    print(f"{c['prediction_holds']} prediction holds, {c['reading_holds']} reading holds, {c['other_waits']} other waits classified, {c['chapter_finishes']} chapter finishes")
    print(f"predictions whose cue does not end in a question: {c['predictions_not_ending_in_a_question']}")
    print(f"removed pairs: {[r['cue'] + '->' + r['next_cue'] for r in audit['removed_pairs']]}")
    print(f"reading windows missing from the storyboard's table: {audit['storyboard']['reading_windows_missing_from_table']}")
    print(f"sentence hooks: {c['literal_sentence_hooks']} literal, {len(audit['sentence_hooks']['non_literal'])} non-literal, {len(audit['sentence_hooks']['cue_differs_from_current'])} naming another cue")
    p = phrases["counts"]
    print(f"phrase anchors: {p['anchors']} in {p['cues_with_anchors']} cues; {p['affected_by_cue_split']} in split cues, {p['byte_or_cue_moved']} with a new cue or byte; 0 measured")
    print(f"cue delta: {delta['counts']}")
    s = compare["hold_summary"]
    print(f"against stage06: {s['rows']} holds; file hash changed {s['file_hash_changed']}; marker line moved {s['marker_line_moved']}; interval changed {s['interval_changed']}; marker text changed {s['marker_text_changed']}")
    w = compare["other_waits"]
    print(f"other waits: added {[(r['file'], r['line']) for r in w['added']]}; removed {[(r['file'], r['line']) for r in w['removed']]}; moved {[(r['file'], r['line']['stage06'], r['line']['stage07']) for r in w['moved_or_recued']]}")
    h = compare["sentence_hooks"]
    print(f"literal sentence hooks: {h['literal']}; added {[(r['file'], r['hook'], r['cue'], r['sentence'], r['times']) for r in h['added']]}; removed {[(r['file'], r['hook'], r['cue'], r['sentence'], r['times']) for r in h['removed']]}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
