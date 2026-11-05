"""Complex Frederick timing bridge, stage06: from ingested recognition and root's selected complete takes to the kit's
sentence timing, phrase marks and an elevenlabs-ui-receipt-v2 receipt (bead .39).

Run from the repository root with python3 -I -B. Every subcommand reads the pinned combined-repair-stage06 script,
units, direction map and chapter sources, and writes only into a NEW folder given by --out.

Stage06 is the repaired bridge (47069583) with stage06 pins and two lanes. Takes come from root's new stage06 lane
(--lane) and, only through root's verified stage05 -> stage06 transfer (--transfer), from the sealed stage05 lane:
a stage05 take enters when the transfer lists its cue, name and hash, and is reported as not transferred otherwise.
Every take keeps the request, receipts and recognition it was made and processed with; the stage05 ones also carry
the transfer they rest on, and the decode receipt binds the transfer's hash.

  report   Read-only on its inputs. The phrase anchors the stage05 chapter code waits on (cue, byte offsets, phrase,
           call sites), the direction dispositions implied by the map's visual holds, and, for every take that
           root's recognition lane has ingested, each sentence onset and phrase mark as the recognizer placed it,
           with its recognizer window, plus the questions that block a take until root answers them. Without
           --lane it reports the source side only.
  decode   Root's run. Converts each take root SELECTED to 24 kHz mono PCM16 with root's ffmpeg arguments (no cut,
           time or pitch change) and writes the elevenlabs-ui-receipt-v2 receipt that the canonical
           tools/coherent_narration.py reads (installed by root from elevenlabs-external-voice-repair01 plus F2).
  prepare  Root's run. Root's selection, root's answers and the decode receipt give the span manifest (whole
           takes, sentence ends at recognizer onsets or root's samples, no inserted silence), the map's visual-hold
           dispositions, a workspace with byte copies of the pinned assembler, its cache module and the stage05
           map, and the exact assemble command. The spans are dry-run through that assembler code first. Each
           selected take is first converted again into the new decode-verification/ folder and compared with its
           decoded PCM; those conversions are kept, and their hashes go in decode-verification.json.
  timings  Root's run, after root's assemble. Verifies the assembly with the same assembler code, checks every
           cue is its decoded take sample for sample, and that the retained conversions are unchanged; it
           mirrors the kit's report and phrase-timing checks, and
           writes phrase-timings.json (every script cue) or a chapter fragment, which the kit cannot load alone.

Recognizer times are estimates, never heard boundaries. A whisper-cpp word segment often starts where the previous
word ended, so its start can sit in the pause before the word. Every onset and mark therefore keeps its recognizer
window [segment start, segment end] and whether it abuts the previous word. Root may replace any onset or mark
with a sample in the answers file. Choosing takes is root's: decode and prepare need a selection with `by` and
`basis`. Nothing here reads Oliver timings, synthesizes, runs recognition, builds, renders or listens.

Exit codes: 0 done; 2 refused (every problem listed) or usage; 3 a pinned input moved.
"""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import subprocess
import sys
import wave

sys.dont_write_bytecode = True  # dependencies and the workspace assembler are imported; leave no __pycache__

SCHEMA = "complex-frederick-timing-bridge-stage06-v1"
REPO = Path.cwd()
HERE = Path(__file__).resolve()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
DEPENDENCIES = {  # role: (repository-relative path, sha256); a moved pin exits 3
    # The stage06 intake: its pins, its lane rules and its transfer check (which imports the sealed stage05 intake).
    "intake": (f"{P}/complex-frederick-stage06-reuse-root01/frederick_intake_stage06.py",
               "a28166cbfa05af38d3a3d3a6977a4345f2256220dce62f0c3c19703b2407b0f4"),
    "log_preparation": (f"{P}/logarithm-stage08-audio-preparation01/prepare_audio.py",
                        "3083d5cdfc98f18c404278155b9d432a0530e7b0eae0b0c09afca02e27c2d540"),
    # Canonical since root's Mail 3920: Bronze's elevenlabs-external-voice-repair01 patch (18c4e75e) plus WhiteSwan's
    # F2 legacy-voice fix; the only difference from 18c4e75e is the legacy Oliver branch, which Frederick never takes.
    "assembler": ("tools/coherent_narration.py",
                  "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5"),
    "assembler_cache": ("tools/narration_cache.py", "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0"),
    "kit": ("crates/explainer_kit/src/narration.rs", "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8"),
}
# sha256 of "name sha256\n" for every src/chapters/*.rs of the stage06 crate, sorted by name. Stage05's was a8e92ca2
# (25 files); set from root's integrated stage06 crate.
CHAPTER_SOURCES_SHA256 = "f710eaae60ddc438a98add17c0195aa78969e2501e8c1adaed21c16e75fd8f87"
RATE_OUT = 24000
FFMPEG_ARGS = ["-nostdin", "-n", "-v", "error"]  # root's decode arguments, as in logarithm-stage08
FFMPEG_OUT = ["-ac", "1", "-ar", "24000", "-c:a", "pcm_s16le"]
MIN_RUN = 3  # the log preparation's support rule: a run of at least this many agreeing words
RECEIPT_SCHEMA = "elevenlabs-ui-receipt-v2"
RECEIPT_SOURCE = "ElevenLabs UI, Frederick Surrey"
PROVIDER = "ElevenLabs"
SELECTION_SCHEMA = "complex-frederick-selection-v1"
ANSWERS_SCHEMA = "complex-frederick-answers-v1"
TEST_SPLIT = "\n#[cfg(test)]\nmod tests {"
HOOK = re.compile(r"([A-Za-z_]\w*)\s*\.\s*(wait_for|until|at)\s*\(")
# Hooks whose phrase is not a literal argument, each bound to the exact source line and the literals it reads.
INDIRECT = (
    {"file": "counting.rs", "hook": "line.wait_for(stage, NEEDS_SAID[i])?;",
     "define": r"const NEEDS_SAID: \[&str; 3\] = ", "kind": "array"},
    {"file": "reading_color.rs", "hook": "line.wait_for(stage, phrase)?;",
     "define": r"pub const NAMED: \[\(&str, f64\); 4\] = ", "kind": "array"},
    {"file": "gallery.rs", "hook": "line.wait_for(stage, phrase)?;", "define": r"let named = ", "kind": "array"},
    {"file": "the_bump.rs", "hook": "line.wait_for(stage, phrase)?;", "define": r"for \(deg, phrase\) in ",
     "kind": "array"},
    {"file": "gallery.rs", "hook": "let rt = line.until(stage, phrase, 2.5)?;", "define": r"Self::walk_and_count\(",
     "kind": "argument", "index": 3},
)
UNCERTAINTY = ("window_s is the recognizer's word segment [start, end]. The spoken onset was not measured. When "
               "abuts_previous_word is true the segment may begin in the pause before the word, so the onset can lie "
               "anywhere up to the segment end. A recognizer can also place a word late; no bound outside the window "
               "is claimed. sample_24k is round(start x 24000) on the decoded take's timeline.")
LIMITS = ("Recognizer estimates and source-bound bookkeeping only. No take selected, heard, measured, cut or "
          "accepted here; no fine sync, listening or voice approval. Oliver timings are never read.")


class Refused(Exception):
    """Exit 2: refused; every problem is listed."""


class PinMoved(Exception):
    """Exit 3: a pinned input moved; review before running again."""


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def dumps(value):
    return json.dumps(value, indent=1, ensure_ascii=False) + "\n"


def write_new(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "xb") as handle:
        handle.write(data if isinstance(data, bytes) else data.encode("utf-8"))


def rel(path):
    """Repository-relative when inside the repository, else absolute (scratch fixtures)."""
    path = Path(path).resolve()
    return str(path.relative_to(REPO.resolve())) if path.is_relative_to(REPO.resolve()) else str(path)


_loaded = {}


def dep(role):
    """A pinned dependency: a module for Python files, else its path."""
    if role not in _loaded:
        name, pin = DEPENDENCIES[role]
        path = REPO / name
        got = sha_file(path) if path.is_file() else "missing"
        if got != pin:
            raise PinMoved(f"{role} {name} is {got[:12]}, not the pinned {pin[:12]}")
        if path.suffix == ".py" and role != "assembler" and role != "assembler_cache":
            spec = importlib.util.spec_from_file_location(f"bridge_{role}", path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            _loaded[role] = module
        else:
            _loaded[role] = path
    return _loaded[role]


def wav_info(path):
    """((channels, width, rate), frames, pcm) of a PCM WAV."""
    try:
        with wave.open(str(path), "rb") as w:
            params = (w.getnchannels(), w.getsampwidth(), w.getframerate())
            frames = w.getnframes()
            pcm = w.readframes(frames)
    except (wave.Error, EOFError, OSError) as error:
        raise Refused(f"{Path(path).name}: not a readable PCM WAV ({error})") from None
    if len(pcm) != frames * params[0] * params[1]:
        raise Refused(f"{Path(path).name}: truncated sample data")
    return params, frames, pcm


# ----------------------------------------------------------------------------- stage05 source side

def lex_rust(src):
    """(masked source, [(start, end, value)]): comments blanked, string literals masked with NUL, char literals
    masked; offsets are unchanged so a masked match points at the original text."""
    out, lits, i, n = list(src), [], 0, len(src)

    def mask(a, b, fill):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = fill

    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            mask(i, j, " ")
            i = j
        elif src.startswith("/*", i):
            j = src.index("*/", i) + 2
            mask(i, j, " ")
            i = j
        elif c in "br" and re.match(r'b?r#*"', src[i:i + 40]) and (
                i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
            m = re.match(r'b?r(#*)"', src[i:])
            close = '"' + m.group(1)
            start = i + len(m.group(0))
            j = src.index(close, start)
            lits.append((i, j + len(close), src[start:j]))
            mask(i, j + len(close), "\x00")
            i = j + len(close)
        elif c == '"' or (src.startswith('b"', i) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_"))):
            j = i + (2 if c == "b" else 1)
            value = []
            while src[j] != '"':
                if src[j] == "\\":
                    e = src[j + 1]
                    if e == "\n":
                        j += 2
                        while src[j] in " \t\n\r":
                            j += 1
                        continue
                    if e == "u":
                        close = src.index("}", j)
                        value.append(chr(int(src[j + 3:close], 16)))
                        j = close + 1
                        continue
                    if e == "x":
                        value.append(chr(int(src[j + 2:j + 4], 16)))
                        j += 4
                        continue
                    value.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0", "\\": "\\", '"': '"', "'": "'"}[e])
                    j += 2
                else:
                    value.append(src[j])
                    j += 1
            lits.append((i, j + 1, "".join(value)))
            mask(i, j + 1, "\x00")
            i = j + 1
        elif c == "'":
            m = re.match(r"'(\\u\{[0-9a-fA-F]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'\n])'", src[i:])
            if m:
                mask(i, i + len(m.group(0)), "\x00")
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return "".join(out), lits


def closing(masked, at):
    """Index of the bracket closing the one at `at` (parentheses, brackets, braces)."""
    pairs, stack = {"(": ")", "[": "]", "{": "}"}, []
    for j in range(at, len(masked)):
        c = masked[j]
        if c in pairs:
            stack.append(pairs[c])
        elif c in ")]}":
            if not stack or stack.pop() != c:
                break
            if not stack:
                return j
    raise PinMoved("unbalanced brackets in a chapter source")


def top_level(masked, a, b):
    """Spans of the comma-separated items between a and b (exclusive), at bracket depth zero."""
    items, depth, start = [], 0, a
    for j in range(a, b):
        c = masked[j]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif c == "," and depth == 0:
            items.append((start, j))
            start = j + 1
    if masked[start:b].strip():
        items.append((start, b))
    return items


def literals_in(lits, a, b):
    return [value for s, e, value in lits if a <= s and e <= b]


def indirect_phrases(spec, masked, lits, name):
    hits = list(re.finditer(spec["define"], masked))
    if not hits or (spec["kind"] == "array" and len(hits) != 1):
        raise PinMoved(f"{name}: the indirect hook source `{spec['define']}` is no longer found exactly as bound")
    phrases = []
    for m in hits:
        if spec["kind"] == "array":
            at = m.end() + len(masked[m.end():]) - len(masked[m.end():].lstrip())
            if masked[at] != "[":
                raise PinMoved(f"{name}: `{spec['define']}` no longer starts an array")
            end = closing(masked, at)
            for a, b in top_level(masked, at + 1, end):
                found = literals_in(lits, a, b)
                if len(found) != 1:
                    raise PinMoved(f"{name}: an element of `{spec['define']}` does not hold exactly one string")
                phrases.append(found[0])
        else:
            at = m.end() - 1
            end = closing(masked, at)
            items = top_level(masked, at + 1, end)
            a, b = items[spec["index"]]
            arg = masked[a:b].strip()
            found = literals_in(lits, a, b)
            if len(found) != 1 or set(arg) != {"\x00"}:
                raise PinMoved(f"{name}: argument {spec['index']} of `{spec['define']}` is not a string literal")
            phrases.append(found[0])
    return phrases


def chapter_hooks(path):
    """Every phrase hook of one chapter file, outside its test module: [(hook, [phrases])], masked, literals."""
    name = path.name
    body = path.read_text(encoding="utf-8").split(TEST_SPLIT)[0]
    masked, lits = lex_rust(body)
    lit_at = {s: (e, value) for s, e, value in lits}
    lines = body.splitlines()
    found, used = [], set()
    for m in HOOK.finditer(masked):
        recv, call = m.group(1), m.group(2)
        if name == "draw.rs" and recv == "self":
            continue  # Spoken's own wait_for/until, which take the phrase from their callers
        line = body.count("\n", 0, m.start()) + 1
        rest = masked[m.end():]
        if call in ("wait_for", "until"):
            a = re.match(r"\s*stage\s*,\s*", rest)
            if not a:
                raise PinMoved(f"{name}:{line}: {recv}.{call}( does not take (stage, phrase, ...)")
            at = m.end() + a.end()
        else:
            at = m.end() + len(rest) - len(rest.lstrip())
        hook = {"file": name, "line": line, "pos": m.start(), "recv": recv, "call": call,
                "source": lines[line - 1].strip()}
        if at in lit_at:
            end, value = lit_at[at]
            if masked[end:].lstrip()[:1] not in (",", ")"):
                raise PinMoved(f"{name}:{line}: the phrase argument is not a plain string literal")
            found.append((dict(hook, via="literal argument"), [value]))
            continue
        specs = [k for k, spec in enumerate(INDIRECT) if spec["file"] == name and spec["hook"] == hook["source"]]
        if len(specs) != 1:
            raise PinMoved(f"{name}:{line}: unrecognised phrase hook `{hook['source']}`; extend INDIRECT after review")
        used.add(specs[0])
        spec = INDIRECT[specs[0]]
        found.append((dict(hook, via=f"indirect: {spec['define'].replace(chr(92), '')}"),
                      indirect_phrases(spec, masked, lits, name)))
    return found, masked, lit_at, used


def bind(script, hook, phrase, masked, lit_at):
    """(cue, how) or (None, problem) for one phrase at one hook."""
    fns = list(re.finditer(r"\bfn\s+[A-Za-z_]\w*", masked[:hook["pos"]]))
    start = fns[-1].start() if fns else 0
    say = re.compile(r"\blet\s+(?:mut\s+)?" + re.escape(hook["recv"])
                     + r"\s*=\s*say\(\s*&?kit\s*,\s*stage\s*,\s*(\x00+)\s*\)")
    says = list(say.finditer(masked, start, hook["pos"]))
    holders = [cue for cue, text in script.items() if phrase in text]
    if says:
        cue = lit_at[says[-1].start(1)][1]
        line = masked.count("\n", 0, says[-1].start()) + 1
        if cue in script and phrase in script[cue]:
            return cue, f"say(&kit, stage, \"{cue}\") at line {line} in the same function"
        return None, f"the line spoken here ({cue}, line {line}) has no phrase {phrase!r}, so the film's at() would fail"
    if len(holders) == 1:
        return holders[0], "receiver is not a local say(); the only script cue containing the phrase"
    return None, f"{len(holders)} script cues contain {phrase!r} and no local say() names one"


def film_sentence_offsets(text):
    """draw.rs sentence_offsets, mirrored: char offsets of sentence starts."""
    out = [0]
    for i in range(max(len(text) - 2, 0)):
        if text[i] in ".?!" and text[i + 1] == " ":
            out.append(i + 2)
    return out


def film_at(text, byte, starts, length):
    """draw.rs Spoken::at without a phrase mark: char share between sentence starts. starts = [(char, seconds)]."""
    i = len(text.encode("utf-8")[:byte].decode("utf-8"))
    total = max(len(text), 1)
    j = max([k for k, (at, _) in enumerate(starts) if at <= i], default=0)
    a, t0 = starts[j]
    b, t1 = starts[j + 1] if j + 1 < len(starts) else (total, length)
    return t0 + (t1 - t0) * ((i - a) / max(b - a, 1))


def film_starts(text, measured, length):
    """draw.rs say(): measured starts when their count equals the film's offsets, else char shares."""
    offsets = film_sentence_offsets(text)
    total = max(len(text), 1)
    return [(at, measured[j] if len(measured) == len(offsets) else length * at / total) for j, at in enumerate(offsets)]


def direction_hits(words, phrase, item):
    hits = [at + len(phrase) - 1 for at in range(len(words) - len(phrase) + 1)
            if phrase and words[at:at + len(phrase)] == phrase]
    return [at for at in hits if at == item["word_index"]] if "word_index" in item else hits


def map_dispositions(intake, script, crate, mapdata):
    """native_visual_hold dispositions for map overrides that are end-of-cue visual holds with source evidence."""
    tree = crate.parents[1]
    holds = {}
    for hold in mapdata.get("visual_holds", []):
        holds.setdefault(hold["cue"], []).append(hold)
    rows, problems = [], []
    for index, item in enumerate(mapdata["overrides"]):
        cue = item["cue"]
        words = intake.direction_words(script[cue])
        hits = direction_hits(words, intake.direction_words(item["after"]), item)
        found = holds.get(cue, [])
        if item.get("role") != "visual" or len(hits) != 1 or hits[0] != len(words) - 1 or len(found) != 1:
            problems.append({"cue": cue, "override_index": index,
                             "detail": "not one end-of-cue visual hold with one bound hold; root disposes it"})
            continue
        hold = found[0]
        path = tree / hold["path"]
        lines = path.read_text(encoding="utf-8").splitlines() if path.is_file() else []
        if not path.is_file() or sha_file(path) != hold["sha256"] or not (
                0 < hold["line"] <= len(lines) and hold["marker"] in lines[hold["line"] - 1]):
            problems.append({"cue": cue, "override_index": index,
                             "detail": f"hold evidence {hold['path']} line {hold['line']} no longer matches the map"})
            continue
        rows.append({"override_index": index, "cue": cue, "kind": "native_visual_hold",
                     "reason": (f"{hold['kind'].capitalize()} window after the complete cue, on the native picture "
                                f"timeline ({Path(hold['path']).name}:{hold['line']}), before {hold['next_cue']}. "
                                "No silence is inserted in the take. Derived from the stage05 map's visual_holds; "
                                "root may replace it."),
                     "evidence": {"path": rel(path), "sha256": hold["sha256"]},
                     "end_word_index": hits[0], "hold_kind": hold["kind"], "hold_line": hold["line"]})
    return rows, problems


def source_side(crate, check_sources=True):
    """Pinned script, units and map; phrase anchors from the chapter code; map dispositions."""
    intake = dep("intake")
    bron = dep("log_preparation")
    script, files = intake.check_binding(crate)
    units = load_json(files["units"])
    chapters = sorted((crate / "src/chapters").glob("*.rs"))
    listing = "".join(f"{p.name} {sha_file(p)}\n" for p in chapters)
    if check_sources and sha_bytes(listing.encode()) != CHAPTER_SOURCES_SHA256:
        raise PinMoved(f"chapter sources moved: {sha_bytes(listing.encode())[:12]}, not the pinned "
                       f"{CHAPTER_SOURCES_SHA256[:12]}")
    anchors, problems, hooks, used = {}, [], 0, set()
    for path in chapters:
        found, masked, lit_at, spec_used = chapter_hooks(path)
        used |= {(path.name, k) for k in spec_used}
        for hook, phrases in found:
            hooks += 1
            for phrase in phrases:
                cue, how = bind(script, hook, phrase, masked, lit_at)
                site = {"file": hook["file"], "line": hook["line"], "call": hook["call"], "via": hook["via"]}
                if cue is None:
                    problems.append(dict(site, phrase=phrase, detail=how))
                    continue
                raw = script[cue].encode("utf-8")
                byte = raw.find(phrase.encode("utf-8"))
                row = anchors.setdefault((cue, byte), {
                    "cue": cue, "byte": byte, "byte_end": byte + len(phrase.encode("utf-8")),
                    "char": len(raw[:byte].decode("utf-8")), "phrase": phrase,
                    "occurrences_in_cue": script[cue].count(phrase), "hooks": []})
                if phrase != row["phrase"]:
                    row.setdefault("other_phrases_at_this_byte", []).append(phrase)
                row["hooks"].append(dict(site, binding=how))
    expected = {(spec["file"], k) for k, spec in enumerate(INDIRECT)}
    if check_sources and used != expected:
        raise PinMoved(f"indirect hooks not found in the source: {sorted(expected - used)}")
    order = {cue: k for k, cue in enumerate(script)}
    rows = sorted(anchors.values(), key=lambda r: (order[r["cue"]], r["byte"]))
    for row in rows:
        spans = bron.kit_sentences(script[row["cue"]])
        k = max(n for n, (a, _b) in enumerate(spans) if a <= row["char"]) if row["char"] >= spans[0][0] else 0
        row["sentence"] = k
        row["at_sentence_start"] = row["char"] == spans[k][0]
    mapdata = load_json(files["map"])
    dispositions, disposition_problems = map_dispositions(intake, script, crate, mapdata)
    film = {cue: {"kit_sentences": len(bron.kit_sentences(text)), "film_offsets": len(film_sentence_offsets(text))}
            for cue, text in script.items()}
    side = {
        "schema": SCHEMA, "kind": "source",
        "pins": {"script_sha256": intake.PINS["script"], "units_sha256": intake.PINS["units"],
                 "map_sha256": intake.PINS["map"], "chapter_sources_sha256": sha_bytes(listing.encode()),
                 "dependencies": {role: {"path": p, "sha256": s} for role, (p, s) in DEPENDENCIES.items()}},
        "crate": rel(crate),
        "chapter_sources": listing.splitlines(),
        "hook_sites": hooks,
        "anchors": rows,
        "anchor_problems": problems,
        "dispositions": dispositions,
        "disposition_problems": disposition_problems,
        "film_ignores_measured_sentence_starts": sorted(c for c, f in film.items()
                                                        if f["kit_sentences"] != f["film_offsets"]),
        "unit_ids": {u["line_ids"][0]: u["id"] for u in units["units"]},
    }
    side["sha256"] = sha_bytes(json.dumps(side, sort_keys=True, ensure_ascii=False).encode("utf-8"))
    return side, script, files


# ----------------------------------------------------------------------------- recognition side

def observed(bron, segments):
    words = []
    for index, segment in enumerate(segments):
        for k, (word, _) in enumerate(bron.tokens(str(segment.get("text", "")))):
            words.append({"w": word, "start": float(segment["start_sec"]), "end": float(segment["end_sec"]),
                          "seg": index, "k": k})
    return words


def align(bron, text, segments):
    """The log preparation's alignment: difflib over its normalized tokens, equal and same-length replace runs."""
    import difflib
    canon = bron.tokens(text)
    obs = observed(bron, segments)
    matcher = difflib.SequenceMatcher(a=[w for w, _ in canon], b=[o["w"] for o in obs], autojunk=False)
    match, run, replaced, differences = {}, {}, {}, []
    for op, i1, i2, j1, j2 in matcher.get_opcodes():
        if op == "equal":
            for k in range(i2 - i1):
                match[i1 + k] = j1 + k
                run[i1 + k] = i2 - i1
            continue
        differences.append({"op": op, "script": " ".join(w for w, _ in canon[i1:i2]),
                            "recognized": " ".join(o["w"] for o in obs[j1:j2]), "script_token_index": i1})
        if op == "replace" and i2 - i1 == j2 - j1:
            for k in range(i2 - i1):
                replaced[i1 + k] = j1 + k
    return canon, obs, match, run, replaced, differences


def window(obs, j):
    o = obs[j]
    row = {"asr_start_s": round(o["start"], 6), "asr_end_s": round(o["end"], 6),
           "window_s": [round(o["start"], 6), round(o["end"], 6)], "segment_index": o["seg"]}
    if j > 0:
        row["previous_word_end_s"] = round(obs[j - 1]["end"], 6)
        row["abuts_previous_word"] = abs(obs[j - 1]["end"] - o["start"]) < 1e-6
    return row


def place(i, al, seconds, after):
    """The log preparation's rule for canonical token i (supported only in an agreeing run of at least MIN_RUN,
    first in its segment, in range and order), keeping the recognizer window."""
    canon, obs, match, run, replaced, _ = al
    row = {"word": canon[i][0]}
    if i in match:
        j = match[i]
        o = obs[j]
        row.update(window(obs, j), agreeing_run=run[i])
        if run[i] < MIN_RUN:
            row["status"] = "short_agreement"
        elif o["k"] != 0:
            row["status"] = "inside_a_segment"
        elif not after < o["start"] < seconds:
            row["status"] = "out_of_order_or_range"
        else:
            row["status"] = "supported"
            row["sample_24k"] = round(o["start"] * RATE_OUT)
        return row
    row["status"] = "no_agreeing_word"
    if i in replaced and i - 1 in match and i + 1 in match and run[i - 1] >= MIN_RUN and run[i + 1] >= MIN_RUN:
        j = replaced[i]
        o = obs[j]
        if o["k"] == 0 and after < o["start"] < seconds:
            row["candidate"] = dict(window(obs, j), heard=o["w"], sample_24k=round(o["start"] * RATE_OUT),
                                    why="the recognizer's one different word in this slot, between agreeing runs")
    return row


def take_points(bron, cue, name, text, segments, seconds, anchors):
    al = align(bron, text, segments)
    canon = al[0]
    sentences, after = [], 0.0
    for n, (a, _b) in enumerate(bron.kit_sentences(text)):
        if n == 0:
            sentences.append({"index": 0, "char": a, "status": "file_start", "sample_24k": 0,
                              "basis": "the complete take starts the cue"})
            continue
        i = next(ix for ix, (_, off) in enumerate(canon) if off >= a)
        row = dict(id=f"onset:{cue}:s{n}:{name}", index=n, char=a, **place(i, al, seconds, after))
        if row["status"] == "supported":
            after = row["asr_start_s"]
        sentences.append(row)
    marks, after = [], -1.0
    for anchor in anchors:
        pid = f"anchor:{cue}:b{anchor['byte']}:{name}"
        base = {"id": pid, "byte": anchor["byte"], "byte_end": anchor["byte_end"], "phrase": anchor["phrase"],
                "sentence": anchor["sentence"]}
        if anchor["at_sentence_start"] and anchor["sentence"] > 0:
            marks.append(dict(base, status="sentence_onset", same_as=sentences[anchor["sentence"]]["id"]))
            continue
        i = next((ix for ix, (_, off) in enumerate(canon) if off == anchor["char"]), None)
        if i is None:
            marks.append(dict(base, status="no_word_at_byte"))
            continue
        row = dict(base, **place(i, al, seconds, -1.0))
        if row["status"] == "supported":
            if row["asr_start_s"] <= after:
                row["status"] = "out_of_order_or_range"
                row.pop("sample_24k")
            else:
                after = row["asr_start_s"]
        marks.append(row)
    return sentences, marks, al[5]


def recognitions(lane, jobs, script, takes_dir, transfer=None):
    """Stage06: the new lane's takes, plus the sealed stage05 lane's takes that root's verified transfer lists."""
    takes, info = read_lane(lane, jobs, script, takes_dir)
    info = {"stage06": info, "stage05": None, "transfer": None}
    if transfer is None:
        return takes, info
    old, info["stage05"] = read_lane(REPO / transfer["lane"], jobs, script, takes_dir)
    listed = {(cue, t["name"]): t for cue, row in transfer["cues"].items() for t in row["takes"]}
    kept, left = [], []
    for take in old:
        held = listed.get((take["cue"], take["file"]))
        if held is None or (held["path"], held["sha256"], held["pcm_sha256"], held["ingest"], held["plan"]) != (
                take["path"], take["sha256"], take["pcm_sha256"], take["recognition"]["ingest"],
                take["recognition"]["plan"]):
            left.append(f"{take['cue']} {take['file']}")
            continue
        kept.append(dict(take, transfer={"path": transfer["path"], "sha256": transfer["sha256"]}))
    missing = sorted(f"{cue} {name}" for cue, name in set(listed) - {(t["cue"], t["file"]) for t in kept})
    if missing:
        raise Refused(f"the transfer lists stage05 takes the stage05 lane does not give as listed: {missing}")
    both = sorted({t["cue"] for t in kept} & {t["cue"] for t in takes})
    twins = sorted({t["sha256"] for t in kept} & {t["sha256"] for t in takes})
    if both or twins:
        raise Refused(f"cues with takes in both lanes {both}, or the same audio in both {twins}")
    info["transfer"] = {"path": transfer["path"], "sha256": transfer["sha256"], "cues": len(transfer["cues"]),
                        "takes": len(kept), "stage05_takes_not_transferred": left}
    order = {cue: k for k, cue in enumerate(script)}
    return sorted(kept + takes, key=lambda t: (order[t["cue"]], t["file"])), info


def read_lane(lane, jobs, script, takes_dir):
    """Every take one lane has ingested: its fw record re-read from the guard job and re-bound by hash."""
    intake = dep("intake")
    if lane is None or not Path(lane).is_dir():
        return [], {"lane": None if lane is None else rel(lane), "ingests": [], "state": "no recognition lane"}
    lane = Path(lane)
    state = intake.lane_state(lane)
    takes, ingests, seen = [], [], set()
    for ingest_path in sorted(lane.glob("ingest-[0-9][0-9]/ingest.json")):
        ing = load_json(ingest_path)
        folder = ingest_path.parent
        name = folder.name
        plan = state["plans"][ing["plan"]]
        problems = []
        if sha_file(folder / "transcripts.json") != ing["transcripts_sha256"]:
            problems.append("transcripts.json changed since ingest")
        if sha_file(lane / ing["plan"] / "frozen-inputs.json") != ing["frozen_inputs_sha256"]:
            problems.append("the plan's frozen-inputs.json changed since ingest")
        # plan.json (where each take names its request and downloads receipts) is read below, so bind it, and
        # every receipt it names, to the frozen inputs the guard hashed before and after recognition.
        frozen = load_json(lane / ing["plan"] / "frozen-inputs.json").get("files", [])
        pinned = [f for f in frozen if f.get("role") == "plan"]
        if len(pinned) != 1 or Path(pinned[0]["path"]).name != "plan.json" \
                or Path(pinned[0]["path"]).parent.name != ing["plan"] \
                or pinned[0]["sha256"] != sha_file(lane / ing["plan"] / "plan.json"):
            problems.append(f"{ing['plan']}/plan.json is not the plan its guard froze")
        frozen_receipts = {(Path(f["path"]).parent.name, Path(f["path"]).name): f["sha256"]
                           for f in frozen if f.get("role") == "receipt"}
        ingests.append({"ingest": name, "plan": ing["plan"], "job": ing["job"], "processed": len(ing["processed"]),
                        "failed": len(ing["failed"]), "run_complete": ing["run_complete"]})
        if not ing["processed"]:
            if problems:
                raise Refused(f"{name}: " + "; ".join(problems))
            continue
        job = Path(jobs) / ing["job"]
        log, receipt = job / "command.log", job / "resource-receipt.json"
        if not log.is_file() or sha_file(log) != ing["command_log_sha256"]:
            problems.append(f"job {ing['job']} command.log is missing under the jobs folder or changed since ingest")
        elif not receipt.is_file() or sha_file(receipt) != ing["resource_receipt_sha256"]:
            problems.append(f"job {ing['job']} resource-receipt.json changed since ingest")
        if problems:
            raise Refused(f"{name}: " + "; ".join(problems))
        records = {}
        for line in log.read_text(encoding="utf-8").splitlines():
            if line.startswith("{"):
                record = json.loads(line)
                records[record["input"]] = record
        rows = {row["sha256"]: row for row in load_json(folder / "transcripts.json")["rows"]}
        for brief in ing["processed"]:
            take = next((t for t in plan["takes"] if t["sha256"] == brief["sha256"] and t["name"] == brief["name"]),
                        None)
            if take is None or (take["cue"], take["name"]) in seen:
                raise Refused(f"{name}: processed {brief['name']} is not one take of {ing['plan']}")
            seen.add((take["cue"], take["name"]))
            record = records.get(take["path"])
            why = intake.record_problems(record, plan) if record else ["no fw record"]
            if why:
                raise Refused(f"{name}: {take['name']}: {'; '.join(why)}")
            segments = record["report"]["result"]["segments"]
            current = sha_bytes(script[take["cue"]].encode("utf-8")) == take["text_sha256"]
            row = rows.get(take["sha256"])
            if row is None or (current and intake.align(script[take["cue"]], segments) != row["alignment"]):
                raise Refused(f"{name}: transcripts.json no longer matches the job's fw record for {take['name']}")
            path = Path(take["path"])
            if not path.is_file() or sha_file(path) != take["sha256"]:
                raise Refused(f"{take['name']}: the WAV changed or is missing since the plan")
            for kind, held in sorted(take["receipts"].items()):
                if Path(held["name"]).name != held["name"] \
                        or frozen_receipts.get((path.parent.name, held["name"])) != held["sha256"]:
                    raise Refused(f"{name}: {take['name']}: its {kind} receipt {held['name']!r} is not a bare name in "
                                  "the take folder that the guard froze at this hash")
            request = Path(take["path"]).parent / take["receipts"]["request"]["name"]
            takes.append({"cue": take["cue"], "file": take["name"], "path": rel(path), "sha256": take["sha256"],
                          "pcm_sha256": take["pcm_sha256"], "frames": take["frames"], "rate": take["sample_rate"],
                          "seconds": take["frames"] / take["sample_rate"], "text_current": current,
                          "request": {"path": rel(request), "sha256": take["receipts"]["request"]["sha256"]},
                          "downloads": {"path": rel(path.parent / take["receipts"]["downloads"]["name"]),
                                        "sha256": take["receipts"]["downloads"]["sha256"]},
                          "recognition": {"lane": rel(lane), "ingest": name, "plan": ing["plan"], "job": ing["job"],
                                          "command_log_sha256": ing["command_log_sha256"],
                                          "record_index": record["index"]},
                          "segments": segments})
    order = {cue: k for k, cue in enumerate(script)}
    takes.sort(key=lambda t: (order[t["cue"]], t["file"]))
    return takes, {"lane": rel(lane), "ingests": ingests, "state": "read",
                   "open_plans": sorted(set(state["plans"]) - set(state["closed"]))}


def supplements(pairs, takes):
    """Root's targeted recognitions with another model (job folder + the frozen manifest its guard hashed), bound
    record by record to ingested lane takes. They are cross-checks and opt-in alternatives, never applied unasked."""
    intake = dep("intake")
    by_path = {t["path"]: t for t in takes}
    found, info = {}, []
    for job, manifest in pairs or []:
        job, manifest = Path(job), Path(manifest)
        receipt_path, log_path = job / "resource-receipt.json", job / "command.log"
        if not (receipt_path.is_file() and log_path.is_file() and manifest.is_file()):
            raise Refused(f"supplement {job.name}: needs resource-receipt.json, command.log and its manifest")
        receipt, manifest_sha = load_json(receipt_path), sha_file(manifest)
        if receipt.get("phase") != "asr" or not receipt.get("admitted") or receipt.get("preflight_only") \
                or receipt.get("exit") != 0 or "finished" not in receipt or "error" in receipt \
                or receipt.get("stopped_for_resource_limit") or receipt.get("inputs_sha256") != manifest_sha \
                or receipt.get("final_inputs_sha256") != manifest_sha:
            raise Refused(f"supplement {job.name}: not a completed guarded recognition of {manifest.name}")
        command = receipt.get("command") or []

        def option(flag):
            return command[command.index(flag) + 1] if flag in command[:-1] else None

        files = {row["path"]: row["sha256"] for row in load_json(manifest)["files"]}
        model, inputs = option("--model"), option("--inputs-from")
        if len(command) < 2 or command[1] != "transcribe" or option("--backend") != "whisper-cpp" \
                or option("--language") != "en" or option("--max-segment-length") != "1" \
                or not {"--no-diarize", "--split-on-word", "--json"} <= set(command) \
                or model not in files or inputs not in files or files.get(command[0]) != intake.TOOL_PINS["fw"]:
            raise Refused(f"supplement {job.name}: its command is not word-split whisper-cpp fw over a manifest-bound "
                          "model and input list")
        if sha_file(inputs) != files[inputs]:
            raise Refused(f"supplement {job.name}: its input list changed")
        listed = Path(inputs).read_text(encoding="utf-8").splitlines()
        records = {}
        for number, line in enumerate(log_path.read_text(encoding="utf-8").splitlines(), 1):
            if not line.startswith("{"):
                continue
            record = json.loads(line)
            index = record.get("index")
            if record.get("schema_version") != intake.FW_RECORD or record.get("total") != len(listed) \
                    or type(index) is not int or not 0 <= index < len(listed) or listed[index] != record.get("input") \
                    or record["input"] in records:
                raise Refused(f"supplement {job.name}: command.log line {number} is not one record of its input list")
            records[record["input"]] = record
        for path, record in records.items():
            request, result = record["report"]["request"], record["report"]["result"]
            params = request.get("backend_params") or {}
            tag = str((result.get("raw_output") or {}).get("model_version_tag", ""))
            take = by_path.get(rel(path))
            if record.get("status") != "ok" or request.get("model") != model or request.get("backend") != "whisper_cpp" \
                    or result.get("backend") != "whisper_cpp" or request.get("language") != "en" \
                    or request.get("diarize") is not False or params.get("split_on_word") is not True \
                    or (params.get("decoding") or {}).get("max_segment_length") != 1 \
                    or f"sha256:{files[model][:12]}" not in tag or not isinstance(result.get("segments"), list):
                raise Refused(f"supplement {job.name}: the record for {Path(path).name} is not an ok word-split record "
                              "of the manifest's model")
            if take is None or files.get(path) != take["sha256"] or sha_file(path) != take["sha256"]:
                raise Refused(f"supplement {job.name}: {Path(path).name} is not an unchanged ingested lane take")
            found.setdefault(take["path"], []).append({
                "job": job.name, "model": {"name": Path(model).name, "sha256": files[model]},
                "record_index": record["index"], "segments": result["segments"]})
        info.append({"job": rel(job), "manifest": rel(manifest), "manifest_sha256": manifest_sha,
                     "receipt_sha256": sha_file(receipt_path), "command_log_sha256": sha_file(log_path),
                     "model": {"name": Path(model).name, "sha256": files[model]}, "records": len(records)})
    return found, info


def cross_check(row, extra, job, model):
    """Attach another recognizer's placement of the same point to a primary row."""
    entry = {"job": job, "model": model["name"], "status": extra["status"]}
    if extra["status"] == "supported":
        entry.update(sample_24k=extra["sample_24k"], window_s=extra["window_s"],
                     abuts_previous_word=extra.get("abuts_previous_word"), agreeing_run=extra["agreeing_run"])
        if row.get("status") == "supported":
            entry["delta_s"] = round(extra["asr_start_s"] - row["asr_start_s"], 6)
    row.setdefault("supplementary", []).append(entry)


def build_report(crate, lane, jobs, takes_dir, extra=None, check_sources=True, transfer=None):
    bron = dep("log_preparation")
    side, script, files = source_side(crate, check_sources)
    moved = None
    if transfer is not None:
        moved = dep("intake").check_transfer(Path(transfer), script)
        moved["path"] = rel(transfer)
    takes, lane_info = recognitions(lane, jobs, script, takes_dir, moved)
    more, lane_info["supplements"] = supplements(extra, takes)
    by_cue = {}
    for anchor in side["anchors"]:
        by_cue.setdefault(anchor["cue"], []).append(anchor)
    entries, questions = [], []
    for t in takes:
        sentences, marks, differences = take_points(bron, t["cue"], t["file"], script[t["cue"]], t["segments"],
                                                    t["seconds"], by_cue.get(t["cue"], []))
        entry = {k: v for k, v in t.items() if k != "segments"}
        notes = []
        for sup in more.get(t["path"], []):
            s2, m2, d2 = take_points(bron, t["cue"], t["file"], script[t["cue"]], sup["segments"], t["seconds"],
                                     by_cue.get(t["cue"], []))
            for row, other in zip(sentences[1:] + marks, s2[1:] + m2):
                if row["status"] != "sentence_onset":
                    cross_check(row, other, sup["job"], sup["model"])
            notes.append({"job": sup["job"], "model": sup["model"], "record_index": sup["record_index"],
                          "differences": d2})
        entry.update(sentences=sentences, anchors=marks, differences=differences, supplementary_recognition=notes)
        entries.append(entry)
        if not t["text_current"]:
            continue

        def offers(row):
            return [f"use_supplementary:{x['job']}" for x in row.get("supplementary", []) if x["status"] == "supported"]

        for s in sentences[1:]:
            if s["status"] != "supported":
                questions.append({"id": s["id"], "kind": "sentence_onset", "cue": t["cue"], "files": [t["file"]],
                                  "blocking": True,
                                  "detail": f"Sentence {s['index']} of {t['cue']} starts with '{s['word']}'; the "
                                            f"recognizer gives no supported onset ({s['status']}).",
                                  "candidate": s.get("candidate"), "supplementary": s.get("supplementary"),
                                  "decisions": (["use_candidate"] if s.get("candidate") else []) + offers(s)
                                               + ["sample"]})
        for m in marks:
            if m["status"] not in ("supported", "sentence_onset"):
                questions.append({"id": m["id"], "kind": "phrase_anchor", "cue": t["cue"], "files": [t["file"]],
                                  "blocking": True,
                                  "detail": f"Anchor {m['phrase']!r} at byte {m['byte']} has no supported onset "
                                            f"({m['status']}); without a mark the film uses its char-share estimate.",
                                  "candidate": m.get("candidate"), "supplementary": m.get("supplementary"),
                                  "decisions": (["use_candidate"] if m.get("candidate") else []) + offers(m)
                                               + ["sample", "omit_mark"]})
        if differences or any(n["differences"] for n in notes):
            def said(diffs):
                return "; ".join(f"{d['op']} '{d['script']}' -> '{d['recognized']}'" for d in diffs) or "none"
            questions.append({
                "id": f"content:{t['cue']}:{t['file']}", "kind": "content", "cue": t["cue"], "files": [t["file"]],
                "blocking": True,
                "detail": f"Normalized recognizer disagreements with the script. Ingested recognition: "
                          f"{said(differences)}." + "".join(f" {n['model']['name']} ({n['job']}): "
                                                            f"{said(n['differences'])}." for n in notes)
                          + " A transcript is not listening. 'proceed' records root's decision that the difference "
                            "does not stop preparation; it states nothing heard.",
                "candidate": None, "decisions": ["proceed", "reject_take"]})
    for p in side["anchor_problems"]:
        questions.append({"id": f"source:{p['file']}:{p['line']}:{p['phrase']}", "kind": "source", "cue": None,
                          "files": [], "blocking": False, "detail": p["detail"], "candidate": None, "decisions": []})
    for p in side["disposition_problems"]:
        questions.append({"id": f"disposition:{p['cue']}:{p['override_index']}", "kind": "disposition",
                          "cue": p["cue"], "files": [], "blocking": True, "detail": p["detail"], "candidate": None,
                          "decisions": []})
    order = {cue: k for k, cue in enumerate(script)}
    questions.sort(key=lambda q: (order.get(q["cue"], -1), q["id"]))
    with_takes = sorted({t["cue"] for t in takes if t["text_current"]}, key=order.get)
    chapters = {}
    for cue in script:
        chapters.setdefault(cue[:2], []).append(cue)
    gaps = sorted(s["asr_end_s"] - s["asr_start_s"] for e in entries for s in e["sentences"][1:] + e["anchors"]
                  if s.get("status") == "supported" and s.get("abuts_previous_word"))
    deltas = sorted(abs(x["delta_s"]) for e in entries for s in e["sentences"][1:] + e["anchors"]
                    for x in s.get("supplementary", []) if "delta_s" in x)
    counts = {
        "script_cues": len(script), "hook_sites": side["hook_sites"], "phrase_anchors": len(side["anchors"]),
        "anchor_cues": len({a["cue"] for a in side["anchors"]}), "anchor_problems": len(side["anchor_problems"]),
        "dispositions": len(side["dispositions"]), "disposition_problems": len(side["disposition_problems"]),
        "takes": len(entries), "cues_with_current_takes": len(with_takes),
        "sentence_onsets": sum(len(e["sentences"]) - 1 for e in entries),
        "sentence_onsets_supported": sum(s["status"] == "supported" for e in entries for s in e["sentences"][1:]),
        "anchor_marks": sum(len(e["anchors"]) for e in entries),
        "anchor_marks_supported": sum(m["status"] == "supported" for e in entries for m in e["anchors"]),
        "anchor_marks_at_sentence_onsets": sum(m["status"] == "sentence_onset" for e in entries for m in e["anchors"]),
        "supported_points_abutting_previous_word": len(gaps),
        "abutting_window_s": {"median": round(gaps[len(gaps) // 2], 3) if gaps else None,
                              "max": round(gaps[-1], 3) if gaps else None},
        "supplementary_cross_checks": len(deltas),
        "supplementary_abs_delta_s": {"median": round(deltas[len(deltas) // 2], 3) if deltas else None,
                                      "max": round(deltas[-1], 3) if deltas else None},
        "questions": len(questions), "blocking_questions": sum(q["blocking"] for q in questions),
    }
    coverage = {"schema": SCHEMA, "kind": "coverage", "source_sha256": side["sha256"], "recognition": lane_info,
                "counts": counts, "cues_with_current_takes": with_takes,
                "cues_without_recognition": [c for c in script if c not in with_takes],
                "complete_chapters": sorted(ch for ch, cues in chapters.items() if all(c in with_takes for c in cues)),
                "film_ignores_measured_sentence_starts": side["film_ignores_measured_sentence_starts"],
                "uncertainty": UNCERTAINTY, "limits": LIMITS}
    files_out = {
        "source.json": dumps(side),
        "takes.json": dumps({"schema": SCHEMA, "kind": "takes", "source_sha256": side["sha256"],
                             "recognition": lane_info, "min_agreeing_run": MIN_RUN, "uncertainty": UNCERTAINTY,
                             "takes": entries, "limits": LIMITS}),
        "questions.json": dumps({"schema": SCHEMA, "kind": "questions", "source_sha256": side["sha256"],
                                 "claim": "A blocking question stops the files it names from being prepared until "
                                          "root answers it. Answers are root's statements; this tool checks their "
                                          "form only.", "questions": questions}),
        "answers-template.json": dumps({"schema": ANSWERS_SCHEMA, "source_sha256": side["sha256"],
                                        "answers": {q["id"]: {"decision": None, "by": None, "basis": None}
                                                    for q in questions if q["decisions"]}}),
        "coverage.json": dumps(coverage),
    }
    return {"side": side, "script": script, "files": files, "takes": {(e["cue"], e["file"]): e for e in entries},
            "questions": questions, "coverage": coverage, "out": files_out,
            "transfer": None if moved is None else {"path": moved["path"], "sha256": moved["sha256"]}}


def report_for(args):
    """The report on root's stage06 lane (if any) and its transfer (if any)."""
    return build_report(Path(args.crate), Path(args.lane) if args.lane else None, Path(args.jobs), Path(args.takes),
                        args.supplement, transfer=Path(args.transfer) if args.transfer else None)


# ----------------------------------------------------------------------------- selection, answers, points

def load_selection(path, report, chapter):
    sel = load_json(path)
    side, script = report["side"], report["script"]
    problems = []
    if sel.get("schema") != SELECTION_SCHEMA or sel.get("source_sha256") != side["sha256"] \
            or sel.get("script_sha256") != side["pins"]["script_sha256"]:
        problems.append(f"the selection is not {SELECTION_SCHEMA} for this stage06 source side ({side['sha256'][:12]})")
    if not str(sel.get("by") or "").strip() or not str(sel.get("basis") or "").strip():
        problems.append("the selection needs `by` and `basis`: choosing takes is root's decision")
    if sel.get("chapter") != chapter:
        problems.append(f"the selection's chapter {sel.get('chapter')!r} is not --chapter {chapter!r}")
    want = [c for c in script if chapter is None or c.startswith(chapter + "_")]
    if not want:
        problems.append(f"chapter {chapter} has no cues")
    elif list(sel.get("takes") or {}) != want:
        problems.append(f"the selection must name exactly these {len(want)} cues in script order: {want[0]} ... {want[-1]}")
    chosen = {}
    for cue, name in (sel.get("takes") or {}).items():
        take = report["takes"].get((cue, name))
        if take is None:
            problems.append(f"{cue}: {name} is not an ingested take of that cue in this lane")
        elif not take["text_current"]:
            problems.append(f"{cue}: {name} records other wording than the stage06 script")
        else:
            chosen[cue] = take
    if problems:
        raise Refused("\n".join(problems))
    return sel, chosen


def check_answers(path, report, chosen):
    data = load_json(path) if path else {"schema": ANSWERS_SCHEMA, "source_sha256": report["side"]["sha256"],
                                         "answers": {}}
    problems = []
    if data.get("schema") != ANSWERS_SCHEMA or data.get("source_sha256") != report["side"]["sha256"]:
        problems.append(f"the answers are not {ANSWERS_SCHEMA} for this stage06 source side")
    known = {q["id"]: q for q in report["questions"]}
    points = {}

    def offers(row):
        return [f"use_supplementary:{x['job']}" for x in row.get("supplementary", []) if x["status"] == "supported"]

    for take in report["takes"].values():
        for s in take["sentences"][1:]:
            points[s["id"]] = ["sample"] + offers(s)
        for m in take["anchors"]:
            if m["status"] != "sentence_onset":
                points[m["id"]] = ["sample", "omit_mark"] + offers(m)
    names = {t["file"] for t in chosen.values()}
    applied = {}
    for qid, answer in sorted((data.get("answers") or {}).items()):
        q = known.get(qid)
        if q is None and qid not in points:
            problems.append(f"answer to an unknown question or point {qid}")
            continue
        files = q["files"] if q else [qid.rsplit(":", 1)[1]]
        if not set(files) & names or (answer or {}).get("decision") is None:
            continue
        allowed = q["decisions"] if q else points[qid]
        if answer["decision"] not in allowed:
            problems.append(f"{qid}: decision {answer['decision']!r} is not one of {allowed}")
        elif not str(answer.get("by") or "").strip() or not str(answer.get("basis") or "").strip():
            problems.append(f"{qid}: an answer needs `by` and `basis`")
        elif answer["decision"] == "sample" and (type(answer.get("sample_24k")) is not int or answer["sample_24k"] <= 0):
            problems.append(f"{qid}: decision 'sample' needs a positive integer sample_24k")
        elif answer["decision"] == "reject_take":
            problems.append(f"{qid}: root rejected {', '.join(sorted(set(files) & names))}; select another take")
        else:
            applied[qid] = {"decision": answer["decision"], "by": answer["by"], "basis": answer["basis"],
                            **({"sample_24k": answer["sample_24k"]} if answer["decision"] == "sample" else {})}
    for q in report["questions"]:
        if q["blocking"] and q["cue"] in chosen and (not q["files"] or set(q["files"]) & names) \
                and q["id"] not in applied and not any(q["id"] in p for p in problems):
            problems.append(f"open question blocks the selection: {q['id']}: {q['detail']}")
    if problems:
        raise Refused("\n".join(problems))
    return applied


def chosen_point(row, a):
    """(sample_24k, basis, window_s, abuts) for one onset or mark under root's answer, or None if none applies."""
    decision = a.get("decision") or ""
    if decision == "sample":
        return a["sample_24k"], f"sample given by {a['by']}: {a['basis']}", None, None
    if decision == "use_candidate":
        c = row["candidate"]
        return (c["sample_24k"], f"recognizer candidate accepted by {a['by']}: {a['basis']}", c["window_s"],
                c.get("abuts_previous_word"))
    if decision.startswith("use_supplementary:"):
        x = next(x for x in row["supplementary"] if x["job"] == decision.split(":", 1)[1] and x["status"] == "supported")
        return (x["sample_24k"], f"{x['model']} onset ({x['job']}) accepted by {a['by']}: {a['basis']}", x["window_s"],
                x.get("abuts_previous_word"))
    if row["status"] == "supported":
        return row["sample_24k"], "recognizer onset", row["window_s"], row.get("abuts_previous_word")
    return None


def resolve(take, applied, frames24):
    """Sentence ends (24 kHz samples) and phrase marks for one selected take, each with its basis and window."""
    starts = []
    for s in take["sentences"][1:]:
        point = chosen_point(s, applied.get(s["id"], {}))
        if point is None:
            raise Refused(f"{s['id']}: no onset")
        starts.append({"sample_24k": point[0], "basis": point[1], "window_s": point[2], "abuts_previous_word": point[3],
                       "cross_checks": s.get("supplementary", [])})
    ends = [x["sample_24k"] for x in starts] + [frames24]
    if any(b <= a for a, b in zip([0] + ends, ends)):
        raise Refused(f"{take['cue']} {take['file']}: sentence ends are not strictly increasing inside the take: {ends}")
    marks = []
    for m in take["anchors"]:
        a = applied.get(m["id"], {})
        base = {"byte": m["byte"], "byte_end": m["byte_end"], "phrase": m["phrase"], "sentence": m["sentence"]}
        if m["status"] == "sentence_onset":
            onset = starts[m["sentence"] - 1]
            marks.append(dict(base, sample_24k=onset["sample_24k"], basis=f"sentence {m['sentence']} onset: "
                              f"{onset['basis']}", window_s=onset["window_s"],
                              abuts_previous_word=onset["abuts_previous_word"], cross_checks=onset["cross_checks"]))
            continue
        if a.get("decision") == "omit_mark":
            continue
        point = chosen_point(m, a)
        if point is None:
            raise Refused(f"{m['id']}: no mark")
        marks.append(dict(base, sample_24k=point[0], basis=point[1], window_s=point[2], abuts_previous_word=point[3],
                          cross_checks=m.get("supplementary", [])))
    previous, bounds = -1, [0] + ends
    for mark in marks:
        mark["seconds"] = mark["sample_24k"] / RATE_OUT
        if not previous <= mark["sample_24k"] < frames24:
            raise Refused(f"{take['cue']} {take['file']}: the mark at byte {mark['byte']} ({mark['sample_24k']}) breaks "
                          "the kit's order or range rule")
        k = mark["sentence"]
        if not bounds[k] <= mark["sample_24k"] < bounds[k + 1]:
            raise Refused(f"{take['cue']} {take['file']}: the mark at byte {mark['byte']} ({mark['sample_24k']}) lies "
                          f"outside its sentence {k}, samples [{bounds[k]}, {bounds[k + 1]})")
        previous = mark["sample_24k"]
    return starts, ends, marks


def expected_frames(take):
    """ceil(n/2) for a 48 kHz take with root's ffmpeg arguments (the log preparation's measured rule)."""
    if take["rate"] != 48000:
        raise Refused(f"{take['file']}: {take['rate']} Hz; Frederick takes are 48 kHz")
    return (take["frames"] + 1) // 2


# ----------------------------------------------------------------------------- decode

def make_out(out):
    out = Path(out)
    if out.exists():
        raise Refused(f"--out {out} exists; give a new folder (nothing is overwritten)")
    for owner in (REPO / dep("intake").TAKES, REPO / dep("intake").CRATE, HERE.parent):
        if out.resolve().is_relative_to(owner.resolve()):
            raise Refused(f"--out must not be inside {rel(owner)}")
    out.mkdir(parents=True)
    return out


def cmd_decode(args):
    report = report_for(args)
    sel, chosen = load_selection(args.selection, report, args.chapter)
    out = make_out(args.out)
    version = subprocess.run([args.ffmpeg, "-version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    intake = dep("intake")
    records, problems = [], []
    for cue, take in chosen.items():
        request = Path(take["request"]["path"])
        data = load_json(request)
        if sha_file(request) != take["request"]["sha256"] or data.get("cue") != cue \
                or data.get("text") != report["script"][cue] or data.get("voice") != intake.VOICE:
            problems.append(f"{cue}: the request {request.name} changed or does not name this cue, text and voice")
            continue
        downloads = Path(take["downloads"]["path"])
        listing = load_json(downloads) if downloads.is_file() else {}
        if not downloads.is_file() or sha_file(downloads) != take["downloads"]["sha256"] \
                or listing.get("cue") != cue or listing.get("request") != request.name \
                or not any(isinstance(t, dict) and Path(str(t.get("path", ""))).name == take["file"]
                           and t.get("sha256") == take["sha256"] for t in listing.get("takes") or []):
            problems.append(f"{cue}: the downloads receipt {downloads.name} changed or does not list {take['file']} "
                            f"from {request.name}")
            continue
        dst = out / "decoded" / f"{cue}.wav"
        dst.parent.mkdir(exist_ok=True)
        subprocess.run([args.ffmpeg, *FFMPEG_ARGS, "-i", take["path"], *FFMPEG_OUT, str(dst)], check=True, timeout=300)
        params, frames, _pcm = wav_info(dst)
        want = expected_frames(take)
        if params != (1, 2, RATE_OUT) or frames != want:
            problems.append(f"{cue}: decoded {params} with {frames} frames; expected mono PCM16 24 kHz with {want} "
                            "(kept, not usable)")
        records.append({"cue": cue, "text": report["script"][cue], "voice": intake.VOICE,
                        "request": take["request"]["path"], "request_sha256": take["request"]["sha256"],
                        "source": take["path"], "source_sha256": take["sha256"],
                        "wav": rel(dst), "wav_sha256": sha_file(dst), "samples": frames, "rate": RATE_OUT,
                        "source_pcm_sha256": take["pcm_sha256"], "source_frames": take["frames"],
                        "source_rate": take["rate"], "downloads_receipt": take["downloads"],
                        "recognition": take["recognition"]})
    receipt = {"schema": RECEIPT_SCHEMA, "source": RECEIPT_SOURCE, "provider": PROVIDER, "voice": intake.VOICE,
               "records": records,
               "bridge": {"schema": SCHEMA, "tool": rel(HERE), "tool_sha256": sha_file(HERE),
                          "source_sha256": report["side"]["sha256"], "selection_sha256": sha_file(args.selection),
                          "selection": {"by": sel["by"], "basis": sel["basis"]}, "chapter": args.chapter,
                          "transfer": report["transfer"]},
               "conversion": "Complete take to mono 24 kHz PCM16 by ffmpeg; no cut, gain, time or pitch processing.",
               "ffmpeg": {"version": version, "arguments": FFMPEG_ARGS + ["-i", "SOURCE"] + FFMPEG_OUT + ["TARGET"]},
               "claim": "Format conversion only. The request records the UI voice setting, not the speaker heard. "
                        "No take accepted, heard or timed here."}
    write_new(out / "decode-receipt.json", dumps(receipt))
    if problems:
        raise Refused("\n".join(problems))
    print(f"decoded {len(records)} complete takes into {rel(out)}; receipt {rel(out / 'decode-receipt.json')}")
    return 0


# ----------------------------------------------------------------------------- prepare

def import_workspace(tool, tag):
    spec = importlib.util.spec_from_file_location(f"workspace_assembler_{tag}", tool)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def workspace_copies(files):
    intake = dep("intake")
    units = load_json(files["units"])
    contract_map = units["direction_contract"]["map"]["path"]
    if contract_map != f"explainer_corpus/{intake.TOPIC}/{intake.MAP_REL}":
        raise PinMoved(f"the units' direction contract names {contract_map}")
    return ((dep("assembler"), "tools/coherent_narration.py", DEPENDENCIES["assembler"][1]),
            (dep("assembler_cache"), "tools/narration_cache.py", DEPENDENCIES["assembler_cache"][1]),
            (files["map"], contract_map, intake.PINS["map"]))


def dry_run(tool, files, spans_path, chapter):
    """The spans through the same assembler code root will run, writing nothing."""
    mod = import_workspace(tool, "dry")
    script, units = mod.read_units(Path(files["script"]), Path(files["units"]), True)
    manifest = json.loads(Path(spans_path).read_text(encoding="utf-8"))
    selected = [u for u in units if not chapter or u["line_ids"][0].startswith(chapter + "_")]
    if [r["id"] for r in manifest["units"]] != [u["id"] for u in selected]:
        raise ValueError("span manifest omits, repeats or reorders selected units")
    mod.check_boundary_review(manifest.get("boundary_review"))
    voice = mod.assembly_voice(manifest["units"])
    directions = mod.review_directions(mod.direction_contract(Path(files["script"]),
                                                              json.loads(Path(files["units"]).read_text())),
                                       selected, manifest)
    lines = {}
    for unit, record in zip(selected, manifest["units"]):
        for cue, row, _pcm in mod.unit_spans(unit, script, record):
            lines.setdefault(cue, []).append(row)
    mod.check_dispositions(directions, lines)
    return voice, lines


def check_decoded(receipt, receipt_path, chosen, report, selection, chapter):
    intake = dep("intake")
    problems = []
    bridge = receipt.get("bridge") or {}
    if (receipt.get("schema"), receipt.get("source"), receipt.get("provider"), receipt.get("voice")) != (
            RECEIPT_SCHEMA, RECEIPT_SOURCE, PROVIDER, intake.VOICE) \
            or bridge.get("source_sha256") != report["side"]["sha256"] or bridge.get("transfer") != report["transfer"] \
            or bridge.get("selection_sha256") != sha_file(selection) or bridge.get("chapter") != chapter \
            or [r.get("cue") for r in receipt.get("records", [])] != list(chosen):
        problems.append(f"{rel(receipt_path)} is not this tool's Frederick v2 decode of this selection")
        return problems
    for row in receipt["records"]:
        take = chosen[row["cue"]]
        wav = Path(row["wav"])
        if not wav.is_file():
            problems.append(f"{row['cue']}: decoded WAV {row['wav']} is missing")
            continue
        params, frames, _ = wav_info(wav)
        if (row["voice"], row["text"], row["source"], row["source_sha256"], row["request"], row["request_sha256"]) != (
                intake.VOICE, report["script"][row["cue"]], take["path"], take["sha256"], take["request"]["path"],
                take["request"]["sha256"]) or sha_file(wav) != row["wav_sha256"] or params != (1, 2, RATE_OUT) \
                or frames != row["samples"] or frames != expected_frames(take) or row["rate"] != RATE_OUT:
            problems.append(f"{row['cue']}: the decoded file, its receipt row and the selected take disagree")
    return problems


def retain_conversions(out, receipt, chosen, ffmpeg):
    """The decode receipt's rows are the decode's own statement, so prepare converts every selected take again, with
    the decode arguments, into OUT/decode-verification/ (a new folder: it refuses one that exists) and compares the
    PCM. Every conversion is kept as evidence, matching or not, and OUT/decode-verification.json records its hash."""
    folder = out / "decode-verification"
    folder.mkdir()
    version = subprocess.run([ffmpeg, "-version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    files, problems = [], []
    for row in receipt["records"]:
        take, again = chosen[row["cue"]], folder / f"{row['cue']}.wav"
        subprocess.run([ffmpeg, *FFMPEG_ARGS, "-i", take["path"], *FFMPEG_OUT, str(again)], check=True, timeout=300)
        pcm = wav_info(again)[2]
        same = pcm == wav_info(Path(row["wav"]))[2]
        files.append({"cue": row["cue"], "source": take["path"], "source_sha256": take["sha256"], "wav": rel(again),
                      "sha256": sha_file(again), "pcm_sha256": sha_bytes(pcm), "decoded": row["wav"],
                      "decoded_sha256": row["wav_sha256"], "matches_decoded": same})
        if not same:
            problems.append(f"{row['cue']}: the decoded PCM is not this take's conversion with the decode arguments")
    record = {"schema": SCHEMA, "kind": "decode-verification", "folder": rel(folder),
              "ffmpeg": {"version": version, "arguments": FFMPEG_ARGS + ["-i", "SOURCE"] + FFMPEG_OUT + ["TARGET"]},
              "files": files, "kept": "every conversion stays in this folder as evidence; nothing is removed"}
    write_new(out / "decode-verification.json", dumps(record))
    if problems:
        write_new(out / "REFUSED.txt", "The decoded PCM is not the conversion of its selected take:\n"
                  + "".join(f"  {p}\n" for p in problems)
                  + "The conversions in decode-verification/ and their hashes in decode-verification.json are kept "
                    "as evidence. Nothing was prepared; decode again and prepare into a new folder.\n")
        raise Refused("\n".join(problems) + f"\nsee {rel(out / 'REFUSED.txt')}")
    return {"path": rel(out / "decode-verification.json"), "sha256": sha_file(out / "decode-verification.json"),
            "files": len(files)}


def cmd_prepare(args):
    report = report_for(args)
    sel, chosen = load_selection(args.selection, report, args.chapter)
    applied = check_answers(args.answers, report, chosen)
    receipt_path = Path(args.decoded) / "decode-receipt.json"
    if not receipt_path.is_file():
        raise Refused(f"no decode receipt in {args.decoded}")
    receipt = load_json(receipt_path)
    problems = check_decoded(receipt, receipt_path, chosen, report, args.selection, args.chapter)
    if problems:
        raise Refused("\n".join(problems))
    rows = {r["cue"]: r for r in receipt["records"]}
    side, script, files = report["side"], report["script"], report["files"]
    units, plan = [], {}
    for cue, take in chosen.items():
        row = rows[cue]
        try:
            starts, ends, marks = resolve(take, applied, row["samples"])
        except Refused as error:
            problems.append(str(error))
            continue
        units.append({"id": side["unit_ids"][cue], "wav": row["wav"], "sha256": row["wav_sha256"],
                      "generation_receipt": rel(receipt_path), "sentence_ends": ends, "gap_after_samples": [0] * len(ends)})
        length = row["samples"] / RATE_OUT
        measured = [0.0] + [s["sample_24k"] / RATE_OUT for s in starts]
        film = film_starts(script[cue], measured, length)
        for mark in marks:
            mark["film_without_mark_s"] = round(film_at(script[cue], mark["byte"], film, length), 6)
        plan[cue] = {"take": take["file"], "take_sha256": take["sha256"], "recognition": take["recognition"],
                     "samples_24k": row["samples"],
                     "sentence_starts_24k": [{"sample_24k": 0, "basis": "start of the complete take"}] + starts,
                     "film_uses_measured_sentence_starts": cue not in side["film_ignores_measured_sentence_starts"],
                     "marks": marks}
    blocked = [p for p in side["disposition_problems"] if p["cue"] in chosen]
    problems += [f"{p['cue']}: override {p['override_index']}: {p['detail']}" for p in blocked]
    if problems:
        raise Refused("\n".join(problems))
    dispositions = [{k: d[k] for k in ("override_index", "cue", "kind", "reason", "evidence")}
                    for d in side["dispositions"] if d["cue"] in chosen]
    out = make_out(args.out)
    verification = retain_conversions(out, receipt, chosen, args.ffmpeg)
    basis = ("Complete ElevenLabs Frederick Surrey takes, converted to 24 kHz only. Sentence ends are recognizer word "
             "onsets (estimates; an onset can sit in the pause before its word) except where root's answers give "
             "samples. No samples removed, no silence inserted, no gain, time or pitch processing.")
    spans = {"schema_version": 1, "units_sha256": side["pins"]["units_sha256"], "units": units,
             "direction_dispositions": dispositions,
             "boundary_review": {"reviewer": args.reviewer, "basis": basis, "listening": "unreviewed"}}
    write_new(out / "spans.json", json.dumps(spans, indent=2) + "\n")
    for src, dst, pin in workspace_copies(files):
        data = Path(src).read_bytes()
        if sha_bytes(data) != pin:
            raise PinMoved(f"{rel(src)} changed during prepare")
        write_new(out / "workspace" / dst, data)
    tool = out / "workspace/tools/coherent_narration.py"
    try:
        voice, lines = dry_run(tool, files, out / "spans.json", args.chapter)
    except (ValueError, KeyError, TypeError, OSError, wave.Error) as error:
        write_new(out / "REFUSED.txt", f"The assembler's own checks refused these spans: {type(error).__name__}: "
                  f"{error}\nNothing was assembled. Keep this folder as evidence and prepare again into a new one.\n")
        raise Refused(f"the assembler's own checks refused these spans ({type(error).__name__}: {error}); "
                      f"see {rel(out / 'REFUSED.txt')}") from None
    chapter_args = ["--chapter", args.chapter] if args.chapter else []
    assemble = ["python3", "-I", "-B", rel(tool), "assemble", "--script", rel(files["script"]), "--units",
                rel(files["units"]), "--require-directions", *chapter_args, "--spans", rel(out / "spans.json"),
                "--out", rel(out) + "-narration"]
    timings = ["python3", "-I", "-B", rel(HERE), "timings", "--prepared", rel(out), "--narration",
               rel(out) + "-narration", "--out", rel(out) + "-timings"]
    write_new(out / "assemble-command.json", dumps({
        "run_from": "repository root", "argv": assemble, "then": timings,
        "why_a_workspace": ("The units' direction contract names its map repository-relative, and coherent_narration.py "
                            "resolves it against its own tree (the tool's parent folder), where the canonical tree has "
                            "no oliver12 map. The workspace holds byte copies of the pinned canonical "
                            "tools/coherent_narration.py, tools/narration_cache.py and the stage05 map; every other "
                            "path resolves from the repository root."),
        "dry_run": f"read_units, boundary review, assembly_voice ({voice}), direction dispositions and unit_spans "
                   f"accepted {len(lines)} cues; nothing written"}))
    write_new(out / "phrase-marks-plan.json", dumps({
        "claim": "Not a phrase-timings.json: the assembled WAV hashes do not exist yet. `timings` writes the real file "
                 "after assembly.", "uncertainty": UNCERTAINTY, "cues": plan}))
    write_new(out / "prepare-receipt.json", dumps({
        "schema": SCHEMA, "kind": "prepare", "tool": rel(HERE), "tool_sha256": sha_file(HERE),
        "source_sha256": side["sha256"], "pins": side["pins"], "chapter": args.chapter,
        "selection": {"path": rel(args.selection), "sha256": sha_file(args.selection), "by": sel["by"],
                      "basis": sel["basis"], "takes": sel["takes"]},
        "answers": {"path": rel(args.answers) if args.answers else None,
                    "sha256": sha_file(args.answers) if args.answers else None, "applied": applied},
        "decode_receipt": {"path": rel(receipt_path), "sha256": sha_file(receipt_path)},
        "decode_verification": verification,
        "spans_sha256": sha_file(out / "spans.json"), "voice": voice,
        "phrase_marks_plan_sha256": sha_file(out / "phrase-marks-plan.json"),
        "recognition": report["coverage"]["recognition"],
        "claim": "Source-bound preparation. Selection and answers are root's; nothing here was heard or accepted."}))
    print(f"prepared {len(units)} units into {rel(out)}; dry run accepted {len(lines)} cues; next, from the "
          f"repository root: {' '.join(assemble)}")
    return 0


# ----------------------------------------------------------------------------- timings

def cmd_timings(args):
    bron = dep("log_preparation")
    intake = dep("intake")
    prepared = Path(args.prepared)
    receipt = load_json(prepared / "prepare-receipt.json")
    plan = load_json(prepared / "phrase-marks-plan.json")["cues"]
    spans_path = prepared / "spans.json"
    side, script, files = source_side(Path(args.crate))
    if receipt.get("schema") != SCHEMA or receipt["source_sha256"] != side["sha256"]:
        raise Refused("the prepared folder was made for another stage06 source side; timings never carry over")
    if sha_file(spans_path) != receipt["spans_sha256"]:
        raise Refused("spans.json changed since prepare")
    if sha_file(prepared / "phrase-marks-plan.json") != receipt.get("phrase_marks_plan_sha256"):
        raise Refused("phrase-marks-plan.json changed since prepare (its marks become phrase-timings.json)")
    held = receipt.get("decode_verification") or {}
    verified = prepared / "decode-verification.json"
    if not verified.is_file() or sha_file(verified) != held.get("sha256"):
        raise Refused("decode-verification.json is missing or changed since prepare")
    for kept in load_json(verified)["files"]:
        again = prepared / "decode-verification" / f"{kept['cue']}.wav"
        if not kept["matches_decoded"] or not again.is_file() or sha_file(again) != kept["sha256"]:
            raise Refused(f"{kept['cue']}: the retained conversion {rel(again)} is missing or changed since prepare")
    for _src, dst, pin in workspace_copies(files):
        if sha_file(prepared / "workspace" / dst) != pin:
            raise Refused(f"workspace copy {dst} changed")
    chapter = receipt["chapter"]
    narration = Path(args.narration)
    mod = import_workspace(prepared / "workspace/tools/coherent_narration.py", "verify")
    try:
        mod.verify(narration, Path(files["script"]), Path(files["units"]), require_directions=True, chapter=chapter)
    except (ValueError, KeyError, TypeError, OSError, wave.Error) as error:
        raise Refused(f"coherent_narration verify failed: {type(error).__name__}: {error}") from None
    assembly = load_json(narration / "assembly.json")
    spans = load_json(spans_path)
    if assembly["manifest"] != spans:
        raise Refused("the assembly was not made from this spans.json")
    report = load_json(narration / "report.json")
    rate = report["meta"]["rate"]
    cues = [c for c in script if c in plan]
    decoded = {cue: unit for cue, unit in zip(cues, spans["units"])}
    problems, entries = [], {}
    for cue in cues:
        rows = report["lines"].get(cue)
        spans_k = bron.kit_sentences(script[cue])
        if rows is None or len(rows) != len(spans_k):
            problems.append(f"{cue}: report has {None if rows is None else len(rows)} sentences, the kit counts "
                            f"{len(spans_k)}")
            continue
        for k, ((a, z), row) in enumerate(zip(spans_k, rows)):
            if bron.kit_normalize(script[cue][a:z].strip()) != bron.kit_normalize(row["sentence"]):
                problems.append(f"{cue} sentence {k}: report text differs from the script")
        params, frames, pcm = wav_info(narration / f"{cue}.wav")
        _p, _f, pcm_src = wav_info(decoded[cue]["wav"])
        if pcm != pcm_src:
            problems.append(f"{cue}: the assembled PCM is not the decoded take sample for sample")
        total = sum(r["out_samples"] + r["gap_after_samples"] for r in rows)
        if abs(total / rate - frames / params[2]) >= 0.05:
            problems.append(f"{cue}: the report accounts for {total / rate:.3f} s, the WAV is {frames / params[2]:.3f} s")
        onsets, at = [], 0
        for r in rows:
            onsets.append(at)
            at += r["out_samples"] + r["gap_after_samples"]
        planned = [s["sample_24k"] for s in plan[cue]["sentence_starts_24k"]]
        if onsets != planned:
            problems.append(f"{cue}: report sentence onsets {onsets} differ from the prepared starts {planned}")
        marks, previous_t, previous_b, bounds = [], 0.0, -1, planned + [frames]
        for m in plan[cue]["marks"]:
            seconds = m["sample_24k"] / RATE_OUT
            k = m.get("sentence")
            if not (0 <= seconds < frames / params[2]) or seconds < previous_t or m["byte"] <= previous_b \
                    or m["byte"] >= len(script[cue].encode("utf-8")):
                problems.append(f"{cue}: the mark at byte {m['byte']} ({seconds} s) breaks the kit's order or range rule")
            elif type(k) is not int or not 0 <= k < len(planned) or not bounds[k] <= m["sample_24k"] < bounds[k + 1]:
                problems.append(f"{cue}: the mark at byte {m['byte']} ({m['sample_24k']}) lies outside its sentence {k}")
            previous_t, previous_b = seconds, m["byte"]
            marks.append({"byte": m["byte"], "seconds": seconds, "sample_24k": m["sample_24k"], "phrase": m["phrase"],
                          "basis": m["basis"], "window_s": m["window_s"],
                          "abuts_previous_word": m.get("abuts_previous_word"), "cross_checks": m.get("cross_checks", []),
                          "film_without_mark_s": m["film_without_mark_s"]})
        entries[cue] = {"wav_sha256": sha_file(narration / f"{cue}.wav"),
                        "text_sha256": sha_bytes(script[cue].encode("utf-8")), "marks": marks}
    if problems:
        raise Refused("\n".join(problems))
    complete = cues == list(script)
    name = "phrase-timings.json" if complete else f"phrase-timings-fragment-{chapter}.json"
    out = make_out(args.out)
    timings = {"basis": "recognizer word onsets (whisper-cpp word segments) unless a mark's basis names root's "
                        "sample; not heard boundaries", "uncertainty": UNCERTAINTY, "voice": intake.VOICE,
               "schema": SCHEMA, "script_sha256": side["pins"]["script_sha256"], "source_sha256": side["sha256"],
               "prepare_receipt_sha256": sha_file(prepared / "prepare-receipt.json"),
               **({} if complete else {"fragment": f"chapter {chapter} only; the kit needs every script cue, so a "
                                                   "complete map must be made before installing"}),
               "cues": entries}
    write_new(out / name, dumps(timings))
    write_new(out / "timings-receipt.json", dumps({
        "schema": SCHEMA, "kind": "timings", "tool": rel(HERE), "tool_sha256": sha_file(HERE),
        "verified": "workspace coherent_narration.verify with --require-directions", "chapter": chapter,
        "assembly_sha256": sha_file(narration / "assembly.json"), "report_sha256": sha_file(narration / "report.json"),
        "cues": len(entries), "marks": sum(len(e["marks"]) for e in entries.values()), "file": name,
        "install": f"copy {name} to {rel(narration)}/phrase-timings.json without overwriting" if complete
        else "not installable alone",
        "claim": "Technical checks and recognizer estimates; no listening, fine sync or render."}))
    print(f"{name} for {len(entries)} cues ({sum(len(e['marks']) for e in entries.values())} marks) in {rel(out)}")
    return 0


# ----------------------------------------------------------------------------- report and main

def cmd_report(args):
    report = report_for(args)
    out = make_out(args.out)
    for name, data in report["out"].items():
        write_new(out / name, data)
    c = report["coverage"]["counts"]
    recognition = report["coverage"]["recognition"]
    states = ", ".join(f"{name}={recognition[name]['state']}" for name in ("stage05", "stage06")
                       if recognition.get(name) is not None)
    print(f"source: {c['hook_sites']} hook sites, {c['phrase_anchors']} anchors in {c['anchor_cues']} cues, "
          f"{c['anchor_problems']} anchor problems, {c['dispositions']} dispositions; recognition "
          f"{states}: {c['takes']} takes, sentence onsets "
          f"{c['sentence_onsets_supported']}/{c['sentence_onsets']} supported, anchor marks "
          f"{c['anchor_marks_supported']}/{c['anchor_marks']} supported (+{c['anchor_marks_at_sentence_onsets']} at "
          f"sentence onsets); {c['questions']} questions, {c['blocking_questions']} blocking; written to {rel(out)}")
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                     formatter_class=argparse.RawDescriptionHelpFormatter, epilog=__doc__)
    parser.add_argument("command", choices=("report", "decode", "prepare", "timings"))
    parser.add_argument("--crate", help="stage06 crate (pinned; drift exits 3)")
    parser.add_argument("--takes", help="retained Frederick take folder")
    parser.add_argument("--lane", help="root's stage06 recognition lane (frederick_intake_stage06.py plan/ingest folders)")
    parser.add_argument("--transfer", help="root's stage05 -> stage06 transfer.json (frederick_intake_stage06.py transfer)")
    parser.add_argument("--jobs", help="the guard's job folder holding each ingest's job")
    parser.add_argument("--out", help="a NEW folder")
    parser.add_argument("--selection", help=f"root's {SELECTION_SCHEMA} file")
    parser.add_argument("--answers", help=f"root's {ANSWERS_SCHEMA} file")
    parser.add_argument("--decoded", help="the decode --out folder")
    parser.add_argument("--reviewer", help="who records the boundary review in the spans")
    parser.add_argument("--prepared", help="the prepare --out folder")
    parser.add_argument("--narration", help="root's assemble --out folder")
    parser.add_argument("--chapter", help="two-digit chapter; omitted means every script cue")
    parser.add_argument("--ffmpeg", default="ffmpeg")
    parser.add_argument("--supplement", nargs=2, action="append", metavar=("JOB", "MANIFEST"),
                        help="root's targeted recognition job and the manifest its guard hashed (repeatable)")
    args = parser.parse_args(argv)
    try:
        intake = dep("intake")
        args.crate = args.crate or str(REPO / intake.CRATE)
        args.takes = args.takes or str(REPO / intake.TAKES)
        args.jobs = args.jobs or str(REPO / intake.WORK)
        need = {"report": ("out",), "decode": ("selection", "out"),
                "prepare": ("selection", "decoded", "reviewer", "out"),
                "timings": ("prepared", "narration", "out")}[args.command]
        missing = [f"--{n}" for n in need if not getattr(args, n)]
        if args.command in ("decode", "prepare") and not (args.lane or args.transfer):
            missing.append("--lane or --transfer")
        if missing:
            parser.error(f"{args.command} needs {', '.join(missing)}")
        if args.chapter is not None and not re.fullmatch(r"\d\d", args.chapter):
            parser.error("--chapter is two digits")
        if not (REPO / "tools/coherent_narration.py").is_file():
            raise Refused("run from the repository root")
        return {"report": cmd_report, "decode": cmd_decode, "prepare": cmd_prepare, "timings": cmd_timings}[
            args.command](args)
    except Refused as error:
        print(f"REFUSED:\n{error}", file=sys.stderr)
        return 2
    except PinMoved as error:
        print(f"PIN MOVED: {error}", file=sys.stderr)
        return 3
    except Exception as error:  # the intake's own refusals and pin moves keep their meaning
        kind = type(error).__name__
        if kind == "Refused":
            print(f"REFUSED:\n{error}", file=sys.stderr)
            return 2
        if kind == "PinMoved":
            print(f"PIN MOVED: {error}", file=sys.stderr)
            return 3
        raise


if __name__ == "__main__":
    sys.exit(main())
