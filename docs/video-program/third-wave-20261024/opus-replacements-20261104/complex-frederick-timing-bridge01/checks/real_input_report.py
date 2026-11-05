"""Real-input report of the timing bridge on root's recognition. From the repository root:

    python3 -I -B .../complex-frederick-timing-bridge01/checks/real_input_report.py FRESH_SCRATCH

Runs the bridge's report on root's lane EC/frederick-stage05-recognition01 (every ingest in it), with root's targeted
small.en job complex-frederick-targeted-small01 as the supplement, twice (the outputs must be byte-equal). Re-places
every point with the logarithm stage08 helper's own points_for and requires the same statuses and samples. Then
prints, per ingested take, each sentence onset and phrase mark with its byte offsets, sample, recognizer window,
abutment and second-model cross-check, the questions, and the intake's read-only status of the take folder (takes
that are not yet recognized). Reads root's lane, jobs, takes and the stage05 crate; writes only under FRESH_SCRATCH.
No decode, assembly, recognition, synthesis, build, render or listening.
"""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
TOOL = HERE.parent / "frederick_timing_bridge.py"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
WORK = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
LANE = f"{EC}/frederick-stage05-recognition01"
SUPPLEMENT = (f"{WORK}/complex-frederick-targeted-small01", f"{EC}/frederick-targeted-small01/inputs.json")
NAMES = ("source.json", "takes.json", "questions.json", "answers-template.json", "coverage.json")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def fmt(row):
    if row["status"] in ("supported",):
        text = (f"sample {row['sample_24k']} = {row['sample_24k'] / 24000:.6f} s, window {row['window_s']}, "
                f"abuts {row.get('abuts_previous_word')}, run {row['agreeing_run']}")
    elif row["status"] == "sentence_onset":
        text = f"= sentence {row['sentence']} onset"
    else:
        text = row["status"] + (f", candidate {row['candidate']['sample_24k']} ('{row['candidate']['heard']}')"
                                if row.get("candidate") else "")
    for x in row.get("supplementary", []):
        text += (f"; {x['model']}: {x['status']}" + (f" {x['sample_24k']} window {x['window_s']}" if "sample_24k" in x
                                                    else "") + (f" delta {x['delta_s']:+.3f} s" if "delta_s" in x else ""))
    return text


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    scratch.mkdir(parents=True)
    br = load("frederick_timing_bridge", TOOL)
    bron = br.dep("log_preparation")
    intake = br.dep("intake")
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    runs = []
    for tag in "ab":
        r = subprocess.run([sys.executable, "-I", "-B", str(TOOL), "report", "--lane", LANE, "--supplement", *SUPPLEMENT,
                            "--out", str(scratch / f"report-{tag}")], cwd=REPO, capture_output=True, text=True)
        runs.append(r)
    same = all(r.returncode == 0 for r in runs) and all(
        (scratch / "report-a" / n).read_bytes() == (scratch / "report-b" / n).read_bytes() for n in NAMES)
    check("report-twice", same, runs[0].stdout.strip().split("; written to")[0] if same else runs[0].stderr[-600:])
    if not same:
        print(f"REPORT: FAIL ({sum(results)}/{len(results)})")
        return 1
    out = scratch / "report-a"
    side = json.loads((out / "source.json").read_text(encoding="utf-8"))
    takes = json.loads((out / "takes.json").read_text(encoding="utf-8"))
    questions = json.loads((out / "questions.json").read_text(encoding="utf-8"))["questions"]
    coverage = json.loads((out / "coverage.json").read_text(encoding="utf-8"))
    rec = takes["recognition"]
    check("recognition-bound", rec["state"] == "read" and not rec["open_plans"] and rec["supplements"][0]["records"] == 2,
          "; ".join(f"{i['ingest']} ({i['plan']}, job {i['job']}): {i['processed']} processed, {i['failed']} failed"
                    for i in rec["ingests"])
          + f"; supplement {Path(rec['supplements'][0]['job']).name}: {rec['supplements'][0]['model']['name']} "
            f"{rec['supplements'][0]['model']['sha256'][:12]}, {rec['supplements'][0]['records']} records, manifest "
            f"{rec['supplements'][0]['manifest_sha256'][:12]}")

    # The log preparation's own placement, on the same fw records, agrees point for point.
    script, _files = intake.check_binding(REPO / intake.CRATE)
    raw, _info = br.recognitions(Path(LANE), REPO / WORK, script, REPO / intake.TAKES)
    by_take = {(t["cue"], t["file"]): t for t in takes["takes"]}
    differ, compared = [], 0
    for t in raw:
        mine = by_take[(t["cue"], t["file"])]
        anchors = [a for a in side["anchors"] if a["cue"] == t["cue"]]
        b_sent, b_marks = bron.points_for(t["cue"], t["file"], script[t["cue"]], {"segments": t["segments"]},
                                          t["seconds"], anchors)
        for a, b in zip(mine["sentences"][1:], b_sent[1:]):
            compared += 1
            if (a["status"], a.get("sample_24k")) != (b["status"], b.get("sample_24k")):
                differ.append(a["id"])
        for a, b in zip(mine["anchors"], b_marks):
            compared += 1
            if a["status"] == "sentence_onset":
                onset = mine["sentences"][a["sentence"]]
                if b["status"] == "supported" and onset.get("sample_24k") != b["sample_24k"]:
                    differ.append(a["id"])
            elif (a["status"], a.get("sample_24k")) != (b["status"], b.get("sample_24k")):
                differ.append(a["id"])
    check("log-preparation-agreement", not differ and compared == coverage["counts"]["sentence_onsets"]
          + coverage["counts"]["anchor_marks"],
          f"{compared} points re-placed by logarithm-stage08 prepare_audio.points_for: same status and sample; "
          f"differing {differ}")
    c = coverage["counts"]
    check("coverage", c["takes"] == len(raw) and not c["anchor_problems"] and not c["disposition_problems"],
          f"{c['takes']} takes of {c['cues_with_current_takes']} cues; sentence onsets {c['sentence_onsets_supported']}/"
          f"{c['sentence_onsets']} supported; anchor marks {c['anchor_marks_supported']}/{c['anchor_marks']} supported "
          f"+ {c['anchor_marks_at_sentence_onsets']} at sentence onsets; {c['supported_points_abutting_previous_word']} "
          f"supported points abut the previous word (window median {c['abutting_window_s']['median']} s, max "
          f"{c['abutting_window_s']['max']} s); {c['supplementary_cross_checks']} second-model cross-checks (|delta| "
          f"median {c['supplementary_abs_delta_s']['median']} s, max {c['supplementary_abs_delta_s']['max']} s); "
          f"complete chapters {coverage['complete_chapters']}")
    print(f"source side {side['sha256']}: {side['hook_sites']} hook sites, {len(side['anchors'])} anchors, "
          f"{len(side['dispositions'])} dispositions; selections and answers bind this hash")
    print("PER TAKE (24 kHz samples on the decoded timeline; recognizer estimates, not heard boundaries)")
    for t in takes["takes"]:
        print(f"{t['cue']} {t['file']}: {t['frames']} frames at {t['rate']} Hz ({t['seconds']:.3f} s), "
              f"{t['recognition']['ingest']} record {t['recognition']['record_index']}, sha {t['sha256'][:12]}")
        for s in t["sentences"][1:]:
            print(f"  s{s['index']} char {s['char']} '{s['word']}': {fmt(s)}")
        for m in t["anchors"]:
            print(f"  b{m['byte']}-{m['byte_end']} {m['phrase']!r}: {fmt(m)}")
        if t["differences"]:
            print("  differences: " + "; ".join(f"{d['op']} '{d['script']}' -> '{d['recognized']}'"
                                                for d in t["differences"]))
    print(f"QUESTIONS ({len(questions)}, {sum(q['blocking'] for q in questions)} blocking)")
    for q in questions:
        print(f"  {q['id']} [{', '.join(q['decisions'])}]")
    status = subprocess.run([sys.executable, "-I", "-B", str(REPO / br.DEPENDENCIES["intake"][0]), "status", "--lane",
                             LANE], cwd=REPO, capture_output=True, text=True)
    st = json.loads(status.stdout) if status.returncode == 0 else {}
    waiting = sorted({w.split()[0] for w in st.get("eligible_wavs", [])})
    check("intake-status-read-only", status.returncode == 0 and st.get("plan") == "status only; nothing written",
          f"take folder now: {len(st.get('eligible_wavs', []))} WAVs of {len(waiting)} cues eligible for the next "
          f"plan ({', '.join(waiting)}), {len(st.get('excluded_wavs', []))} already processed, refused "
          f"{len(st.get('refused', []))}, awaiting download {st.get('awaiting_download')}")
    print(f"REPORT: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
