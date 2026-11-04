#!/usr/bin/env python3
"""Which of the final 80 cues already have an exact Oliver recording here? Run from the repo root:

    python3 -I -B oliver_inventory.py --out-tsv oliver-cue-inventory.tsv --out-json oliver-cue-inventory.json

Reads only. Walks narration_lab for every JSON file up to 40 MB and looks for each cue's exact text,
its text hash, a normalized form and each exact sentence. A hit counts as a recording only when the
same JSON object also holds a non-empty WAV or audio field. Everything else is a plan row. The 108
real "ElevenLabs UI, Oliver" receipts use the fields wav and wav_sha256, which are in AUDIO_KEYS; a
record under some other field name would be missed.

It cannot see the provider's account. "Not local" does not mean "not in provider history":
requests.json says that check is still owed, per cue, before any synthesis. No browser, no provider
call, no synthesis here. Narration is ElevenLabs Oliver.
"""
import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
PLAN = LANE / "combined-repair-stage06/bound-plan"
SCRIPT = LANE / "combined-repair-stage06/explainer_corpus/logarithm_mysteries/script.tsv"
REQUESTS = LANE / "narration-request-final80-01/requests.json"
HISTORY = LANE / "narration-request-final80-01/history-observation01.json"
MAP = PLAN / "explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver13.json"
LIMIT = 40 * 1024 * 1024
AUDIO_KEYS = {"wav", "wav_path", "wav_sha256", "mp3", "mp3_path", "audio", "audio_path", "output_wav", "take", "take_path", "source_wav", "source_wav_sha256", "decoded_wav", "download"}
SENTENCE = re.compile(r".*?[.?!](?= |$)|.+$")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def norm(text):
    return " ".join(re.findall(r"[a-z0-9']+", text.lower().replace("’", "'")))


def read_script(path):
    rows = {}
    for row in Path(path).read_text(encoding="utf-8").split("\n"):
        if row and not row.startswith("#"):
            cue, text = row.split("\t")
            rows[cue] = text
    return rows


def walk(value, owner=None):
    """Every string, with the dict that directly holds it."""
    if isinstance(value, dict):
        for key, item in value.items():
            if isinstance(item, str):
                yield key, item, value
            else:
                yield from walk(item, value)
    elif isinstance(value, list):
        for item in value:
            if isinstance(item, str):
                yield None, item, owner
            else:
                yield from walk(item, owner)


def scan(root, cues):
    by_text = {t: c for c, t in cues.items()}
    by_hash = {sha(t.encode()): c for c, t in cues.items()}
    by_norm = {norm(t): c for c, t in cues.items()}
    by_sentence = {}
    for cue, text in cues.items():
        for piece in SENTENCE.findall(text):
            # One-word sentences such as "No." occur everywhere and prove nothing.
            if len(piece.split()) >= 4:
                by_sentence.setdefault(piece.strip(), set()).add(cue)
    stats = {"json_files_read": 0, "json_files_over_40mb": 0, "json_files_unparseable": 0, "bytes_read": 0}
    takes, plan_hits, normal_hits, sentence_takes = {}, {}, {}, {}
    for base, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d not in (".git", "target", ".venv", ".venv-cosyvoice3", "node_modules", "__pycache__")]
        for name in names:
            if not name.endswith(".json"):
                continue
            path = Path(base) / name
            try:
                size = path.stat().st_size
            except OSError:
                continue
            if size > LIMIT:
                stats["json_files_over_40mb"] += 1
                continue
            stats["json_files_read"] += 1
            stats["bytes_read"] += size
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                stats["json_files_unparseable"] += 1
                continue
            for key, value, holder in walk(data):
                if len(value) < 3:
                    continue
                # The key must hold a value: the unmeasured template sits a text hash beside a null wav_sha256.
                audio = isinstance(holder, dict) and any(holder.get(k) for k in AUDIO_KEYS)
                cue = by_text.get(value) or (by_hash.get(value) if len(value) == 64 else None)
                if cue:
                    target = takes if audio else plan_hits
                    target.setdefault(cue, []).append(path.as_posix())
                    continue
                folded = norm(value)
                if folded in by_norm:
                    normal_hits.setdefault(by_norm[folded], []).append(path.as_posix())
                elif value in by_sentence and audio:
                    for hit in by_sentence[value]:
                        sentence_takes.setdefault(hit, []).append(path.as_posix())
    return stats, takes, plan_hits, normal_hits, sentence_takes


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--out-tsv", required=True)
    parser.add_argument("--out-json", required=True)
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    os.chdir(repo)
    cues = read_script(SCRIPT)
    requests = json.loads(REQUESTS.read_bytes())
    plan_map = json.loads(MAP.read_bytes())
    anchors = json.loads((PLAN / "phrase-anchors.json").read_bytes())["anchors"]
    delta = {d["cue"]: d for d in json.loads((PLAN / "cue-delta.json").read_bytes())["cues"]}
    observed = json.loads(HISTORY.read_bytes())
    stats, takes, plan_hits, normal_hits, sentence_takes = scan(Path("narration_lab"), cues)
    holds = {v["cue"]: v["kind"] for v in plan_map["visual_holds"]}
    marks = {}
    for row in anchors:
        marks.setdefault(row["cue"], []).append(row["byte"])
    seen_in_history = observed_cues(observed)
    rows = []
    for req in requests["requests"]:
        cue, text = req["cue"], cues[req["cue"]]
        data = (repo / req["text_path"]).read_bytes()
        rows.append({
            "cue": cue, "words": len(text.split()), "chars": len(text), "sentences": len([s for s in SENTENCE.findall(text) if s.strip()]),
            "text_sha256": sha(text.encode()), "request_file": req["text_path"], "request_file_exact": data == text.encode(),
            "older_request_id": req["prior_request_id"], "split_cue": delta[cue]["text_changed_from_baseline"],
            "provider_history": seen_in_history.get(cue, "not checked"), "submitted": req["submitted"],
            "exact_local_recordings": len(set(takes.get(cue, []))), "recording_files": sorted(set(takes.get(cue, []))),
            "plan_rows_only": len(set(plan_hits.get(cue, []))), "normalized_hits": len(set(normal_hits.get(cue, []))),
            "recordings_holding_an_exact_sentence": len(set(sentence_takes.get(cue, []))),
            "hold": holds.get(cue, ""), "phrase_anchor_bytes": marks.get(cue, []),
            "status": "EXACT LOCAL RECORDING: verify before reuse" if takes.get(cue) else "MISSING: needs an exact Oliver take",
        })
    Path(args.out_json).write_text(json.dumps({
        "claim": "Local files only. Not listening, and not a statement about the provider account: history is owed per cue before synthesis.",
        "voice": requests["voice"], "script_sha256": requests["script_sha256"], "scan": stats, "required_before_synthesis": requests["required_before_synthesis"],
        "totals": {"cues": len(rows), "words": sum(r["words"] for r in rows), "chars": sum(r["chars"] for r in rows), "sentences": sum(r["sentences"] for r in rows),
                   "exact_local_recordings": sum(1 for r in rows if r["exact_local_recordings"]), "missing": sum(1 for r in rows if not r["exact_local_recordings"]),
                   "provider_history_observed": sum(1 for r in rows if r["provider_history"] != "not checked"), "request_files_exact": sum(1 for r in rows if r["request_file_exact"])},
        "cues": rows}, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    columns = ["cue", "words", "chars", "sentences", "status", "exact_local_recordings", "recordings_holding_an_exact_sentence", "provider_history", "submitted", "older_request_id", "split_cue", "hold", "phrase_anchor_bytes", "request_file_exact", "text_sha256"]
    with open(args.out_tsv, "w", encoding="utf-8") as out:
        out.write("\t".join(columns) + "\n")
        for row in rows:
            out.write("\t".join(",".join(map(str, row[c])) if isinstance(row[c], list) else str(row[c]) for c in columns) + "\n")
    print(json.dumps(json.loads(Path(args.out_json).read_text())["totals"]), json.dumps(stats))
    return 0


def observed_cues(observed):
    """history-observation01.json states what was seen, per cue, in the provider's page."""
    found = {}

    def visit(value):
        if isinstance(value, dict):
            cue = value.get("cue") or value.get("id")
            if isinstance(cue, str) and re.fullmatch(r"\d\d_\d+", cue):
                found[cue] = "; ".join(str(value[k]) for k in value if k not in ("cue", "id", "text"))[:200] or "observed"
            for item in value.values():
                visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(observed)
    return found


if __name__ == "__main__":
    sys.exit(main())
