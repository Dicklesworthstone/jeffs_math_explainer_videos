#!/usr/bin/env python3
"""Bind the final 80-cue logarithm narration plan to combined-repair-stage04 (Mail 1599).

    python3 -I -B <this file> [--check] [--out DIR]

Run from the repo root. Reads stage04's bytes, the kit, the tools and root's frozen request
inventory. Writes only under this file's folder (or --out, for a trial):

  explainer_corpus/logarithm_mysteries/narration_units.json
  explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver12.json
  hold-audit.json                          the 13 predictions, two readings, every other wait
  phrase-anchors.json                      every exact-byte phrase anchor; no seconds
  phrase-timings-unmeasured-template.json  all 80 cues, wav and seconds null, so it cannot load
  direction-dispositions-proposed.json     15 native_visual_hold rows for root's span manifest
  cue-delta.json                           each final cue against root's 75-cue request text
  inputs.sha256                            every input's bytes, for shasum -a 256 -c

This is a text audit of Rust source. Nothing is compiled, rendered, synthesized, measured or
heard. Only chapters registered in mod.rs are read. A wait or phrase hook this file does not
classify stops the run, so a classification can never go stale silently. An existing output
that differs is refused, never overwritten. Nothing here depends on the clock, and nothing
reads the installed canonical crate, so a rerun on the same inputs writes the same bytes.

Narration is ElevenLabs Oliver. No Michael, no FrankenTTS.
"""
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

TOPIC = "logarithm_mysteries"
REL = f"explainer_corpus/{TOPIC}"
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE_ROOT = LANE / "combined-repair-stage04"
STAGE = STAGE_ROOT / REL
CH = STAGE / "src/chapters"
FINAL_OUT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final80-bound-plan01"
MAP_NAME = "narration-direction-map-oliver12.json"
STALE_MAP = "narration-direction-map-oliver11.json"
EXPECTED_SCRIPT = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"
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
PHRASE_MARK_USES = {"slide_rule.rs": 1, "undoing_growth.rs": 1, "doing_nothing.rs": 1, "unreachable.rs": 1, "what_it_forgets.rs": 1}
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
    "slide_rule.rs": "Without a mark at this byte: the phrase's character share of the cue's first sentence (slide_rule.rs phrase_time). Provisional.",
    "undoing_growth.rs": "Without a mark at this byte: the phrase's character share between the onsets of sentences 3 and 4 (undoing_growth.rs climb_onset). Provisional.",
    "doing_nothing.rs": "Without a mark at this byte: the phrase's byte share of its measured sentence (doing_nothing.rs phrase_seconds). Provisional.",
    "unreachable.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (unreachable.rs phrase_seconds). Provisional.",
    "what_it_forgets.rs": "Without a mark at this byte: the phrase's character share of its measured sentence (what_it_forgets.rs phrase_seconds). Provisional.",
}
# Every wait that is not predict, a reading window or a chapter's finish: (file, line) -> class, note.
SITES = {
    ("ratio_ruler.rs", 181): ("picture_pacing", "Paces the note dots of the opening, 0.18 s each, in a loop. No hold for a viewer."),
    ("ratio_ruler.rs", 509): ("helper_tail", "Last line of preview(), called for 00_6 directly before kit.finish, which holds to the same point. No added window."),
    ("slide_rule.rs", 102): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("undoing_growth.rs", 348): ("word_sync", "Waits to the onset of \"it only ever climbs\", inside 04_2."),
    ("matrix_logs.rs", 206): ("picture_pacing", "Shows each named matrix for 0.9 s, in a loop inside 09_1. Whether a pass outlasts its sentence is not fixed by source."),
    ("doing_nothing.rs", 215): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("doing_nothing.rs", 598): ("reading_voice_end", "10_4's reading hold starts its arithmetic here. See reading_holds."),
    ("unreachable.rs", 288): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("unreachable.rs", 1555): ("reading_voice_end", "11_20's reading clock starts when this returns. See reading_holds."),
    ("what_it_forgets.rs", 554): ("word_sync_helper", "Inside until_share: waits to a share of a sentence, inside the voice."),
    ("what_it_forgets.rs", 632): ("word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."),
    ("what_it_forgets.rs", 1143): ("picture_pacing", "ARRIVAL_HOLD, 0.5 s on an arrival, then until_phrase, inside 12_6."),
}
READING_MARKERS = {
    "doing_nothing.rs": "kit.music_breath(stage, FINE_PRINT_S - read_for)?;",
    "unreachable.rs": "kit.music_breath(stage, rule_read + row_reads.iter().sum::<f64>())?;",
}
PREDICTION_REASON = "Source-bound native prediction hold follows complete cue; full recordings must be preserved."
READING_REASON = "Source-bound native reading hold follows complete cue; full recordings must be preserved."
PREDICTION_BASIS = "Direct registered scene cue interval and native hold call. Source only; not audible or picture acceptance."
UNITS_SCOPE = (
    "Candidate full-cue source plan for ElevenLabs Oliver, combined-repair-stage04: 80 units, one per cue, "
    "at most 100 words. It says nothing about whether a recording of any cue exists; unchanged wording is "
    "not evidence of a usable take. No synthesis, take selection, listening, timing or independent "
    "acceptance. No Michael or FrankenTTS."
)
MAP_SCOPE = (
    "Candidate final 80-cue map, combined-repair-stage04: 13 prediction holds and 2 reading holds, each "
    "bound to staged source. Replaces oliver11's four stale pairs, which name the retired seven-chapter "
    "draft. Hashes are the proposed staged bytes, not installed canonical files. Authored coverage only: "
    "no silence is added to any take, and no take, measured timing, listening or independent acceptance "
    "is claimed. ElevenLabs Oliver; no Michael or FrankenTTS."
)


def stop(message):
    raise SystemExit(f"bind_final80: {message}")


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
    row = next(line for line in (STAGE_ROOT / "oracle04.tsv").read_text(encoding="utf-8").splitlines() if line.startswith("matrix.blocks_mixed\t"))
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
    chars = len(textbook) + len("precisely: ") + len(cited)
    return {
        "fine_print": {"textbook": textbook, "source": cited, "prefix": "precisely: "},
        "fine_print_chars": chars,
        "floor_seconds_from_shown": floor,
        "characters_per_second_at_floor": round(chars / floor, 3),
    }


def storyboard_tables():
    text = (STAGE / "docs/storyboard.md").read_text(encoding="utf-8")

    def rows(heading):
        section = text.split(heading, 1)[1].split("\n## ", 1)[0]
        return [[c.strip().strip("*") for c in line.strip("|").split("|")] for line in section.split("\n") if re.match(r"\| \**\d\d_\d+\** \|", line)]

    return rows("## Prediction holds"), rows("## Authored reading holds"), text


def build():
    coherent = load_coherent()
    script_path = STAGE / "script.tsv"
    script_bytes = script_path.read_bytes()
    script_sha = sha(script_bytes)
    if script_sha != EXPECTED_SCRIPT:
        stop(f"stage04's script is {script_sha}, not the merged final 80")
    texts = coherent.read_script(script_path)
    cues = list(texts)
    if len(cues) != 80:
        stop(f"{len(cues)} cues, not 80")
    if (STAGE / "docs" / MAP_NAME).exists():
        stop(f"stage04 already has a {MAP_NAME}")
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
    # 04_2's climb reads the mark directly
    undoing, undoing_scene = source(CH / "undoing_growth.rs")
    climb = constant(r'^const CLIMB_PHRASE: &str = "([^"\\]*)";', undoing_scene, "undoing_growth.rs")
    add_anchor(anchors, "04_2", climb, "CLIMB_PHRASE", "undoing_growth.rs", one_line(undoing, 'climb_onset(text, said, s3, s4, &n.phrase_marks("04_2"))', "undoing_growth.rs"), "climb_onset")
    add_anchor(anchors, "04_2", climb, "CLIMB_PHRASE", "undoing_growth.rs", one_line(undoing, "super::draw::wait_until(stage, onset)?;", "undoing_growth.rs"), "wait_until")

    # ---------------------------------------------------------------- baseline: root's frozen 75-cue request text
    inventory = json.loads((LANE / "narration-request-inventory01.json").read_text(encoding="utf-8"))
    request_plan = json.loads((LANE / "narration-request-plan01.json").read_text(encoding="utf-8"))
    baseline = {}
    for row in inventory["cues"]:
        data = Path(row["text_path"]).read_bytes()
        if sha(data) != row["text_sha256"]:
            stop(f"{row['text_path']}: root's request text changed")
        baseline[row["cue"]] = data.decode("utf-8")
    if " ".join(baseline.values()) != " ".join(texts.values()):
        stop("the final 80 cues are not the 75-cue script's words in order")
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
        base = base_cues[base_i]
        at = baseline[base].find(text, used)
        if at != used:
            stop(f"{cue} is not the next text of {base}")
        offset_in[cue] = (base, at)
        match = requests.get(text, [])
        if len(match) != 1:
            stop(f"{cue}: {len(match)} request texts with its exact bytes")
        delta.append({
            "cue": cue,
            "words": len(text.split()),
            "text_sha256": text_hash[cue],
            "baseline_cue": base,
            "baseline_byte": at,
            "text_changed_from_baseline": text != baseline[base],
            "request_id_with_exact_bytes": match[0]["id"],
        })
        used = at + len(text.encode("utf-8")) + 1
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
        base, base_at = offset_in[cue]
        changed = texts[cue] != baseline[base]
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
        "claim": "Text comparison with root's frozen request files. It says nothing about whether any recording exists. Root's inventory found no exact local record for the 75-cue text and requires a provider history check before new synthesis.",
        "baseline_script_sha256": inventory["script_sha256"],
        "request_plan": str(LANE / "narration-request-plan01.json"),
        "request_plan_binds": request_plan["script_sha256"],
        "root_inventory_counts": inventory["counts"],
        "counts": {
            "cues": len(delta),
            "unchanged_from_baseline": sum(1 for d in delta if not d["text_changed_from_baseline"]),
            "split_cues": sum(1 for d in delta if d["text_changed_from_baseline"]),
            "with_exact_request_bytes": sum(1 for d in delta if d["request_id_with_exact_bytes"]),
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
        "claim": "Text audit of Rust source at combined-repair-stage04. Uncompiled here, unrendered, unmeasured, unheard. Author/source binding, not independent AV or listening acceptance.",
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
                "evidence": {"path": f"{FINAL_OUT}/hold-audit.json", "sha256": audit_sha},
            }
            for i, r in enumerate(visual)
        ],
    }
    disposition_text = json.dumps(dispositions, indent=2, ensure_ascii=False) + "\n"

    # ---------------------------------------------------------------- inputs
    inputs = [script_path, STAGE / "src/main.rs", STAGE / "docs/storyboard.md", STAGE / "docs" / STALE_MAP, STAGE / "narration_units.json", STAGE_ROOT / "oracle04.tsv", STAGE_ROOT / "assembly04.json"]
    inputs += [CH / "mod.rs", CH / "draw.rs"] + [CH / n for n in names]
    inputs += [Path(p) for p in (kit, nar, "crates/explainer_kit/src/captions.rs", "crates/explainer_score/src/score.rs", "tools/coherent_narration.py", "tools/narration_cache.py", "tools/narration_delivery.py", "tools/final_check.py")]
    inputs += [LANE / "narration-request-inventory01.json", LANE / "narration-request-plan01.json"]
    inputs += sorted({Path(r["text_path"]) for r in inventory["cues"]} | {Path(r["text_path"]) for r in request_plan["requests"]})
    inputs_text = "".join(f"{file_sha(p)}  {p}\n" for p in inputs)
    return {
        f"{REL}/narration_units.json": units_text,
        f"{REL}/docs/{MAP_NAME}": map_text,
        "hold-audit.json": audit_text,
        "phrase-anchors.json": phrase_text,
        "phrase-timings-unmeasured-template.json": template_text,
        "direction-dispositions-proposed.json": disposition_text,
        "cue-delta.json": delta_text,
        "inputs.sha256": inputs_text,
    }, audit_doc, phrase_doc, delta_doc


def add_anchor(anchors, cue, phrase, const, name, line, call):
    row = anchors.setdefault((cue, phrase), {"const": const, "sites": []})
    row["sites"].append({"file": name, "line": line, "call": call})


def sentence_counts_problem(texts, coherent):
    """A line whose two sentence rules disagree would stop the render at phrase_seconds."""
    for cue, text in texts.items():
        if len(sentence_opens(text)) != len(coherent.sentences(text)):
            yield f"{cue}: phrase_seconds and the sentence contract count different sentences"


def main(argv):
    check = "--check" in argv
    args = [a for a in argv if a != "--check"]
    out = Path(__file__).resolve().parent
    if args[:1] == ["--out"]:
        out = Path(args[1])
    elif args:
        stop(f"unknown arguments: {args}")
    if not Path("tools/coherent_narration.py").is_file():
        stop("run from the repo root")
    outputs, audit, phrases, delta = build()
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
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
