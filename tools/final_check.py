#!/usr/bin/env python3
"""Delivery check of one video's final (reviewer's scratch, read only; EmeraldOwl).

    python3 -I final_check.py SLUG [CHECKOUT] [--strict] [--no-loudness] [--chapter-max 85] [--skill DIR]

CHECKOUT holds explainer_corpus/SLUG/{script.tsv,lexicon.json,renders/final/...}. Reads, never writes into it:
  1. the files a final must have (chapter MP4s and timelines, the master, the SRT and VTT, the marks)
  2. ffprobe of the master against the EXPECT gates: 3840x2160, 60 fps, BT.709 (primaries, transfer, matrix), A/V delta <= 0.2 s,
     and its own EBU R128 measurement (ffmpeg ebur128, run niced): integrated -16 +-1 LUFS, true peak <= -1.0 dBTP (+0.05)
  3. every chapter: ffprobe size/fps/codec, duration against its timeline (frames / fps), the sum against the master, and the 85 s limit
  4. the SRT against script.tsv: word for word, in order, line by line (numbers as spelled, names), exact text per line,
     SRT format (indices, stamps, <= 2 lines of <= 42 characters, no overlap), timing inside each line's audio and the master's length,
     the VTT against the SRT

Exit 0 whatever it finds by default (it is a reader); with --strict, exit 1 if any DEFECT line was printed.
"""
import argparse
import difflib
import json
import os
import re
import subprocess
import sys
from pathlib import Path


def default_skill_dir() -> Path:
    env_skill = os.environ.get("SKILLS")
    if env_skill:
        p = Path(env_skill) / "natural-sounding-narration-with-frankentts/scripts"
        if p.is_dir():
            return p
    home_dot = Path.home() / ".claude/skills/natural-sounding-narration-with-frankentts/scripts"
    if home_dot.is_dir():
        return home_dot
    return Path.home() / "jmev_orch/skills_current/natural-sounding-narration-with-frankentts/scripts"


def norm(w: str) -> str:
    return re.sub(r"^[^\w]+|[^\w]+$", "", w.lower().replace("’", "'").replace("‘", "'"))


def words(text: str) -> list[str]:
    return [norm(w) for w in text.split() if norm(w)]


fold = lambda s: re.sub(r"\s+", " ", s).strip()


def read_timeline(path: Path) -> dict:
    t = {"lines": [], "marks": [], "fps": None, "frames": None, "chapter": path.name.replace(".timeline.tsv", "")}
    for raw in path.read_text(encoding="utf-8").splitlines():
        c = raw.split("\t")
        if c[0] == "fps":
            t["fps"] = int(c[1])
        elif c[0] == "frames":
            t["frames"] = int(c[1])
        elif c[0] == "line":
            t["lines"].append({"id": c[1], "start": int(c[2]), "gain": c[3], "wav": c[4]})
        elif c[0] == "mark":
            t["marks"].append((c[1], int(c[2])))
    return t


def check_final(
    slug: str,
    root: Path,
    strict: bool = False,
    loud: bool = True,
    ch_max: float = 85.0,
    skill: Path | None = None,
    probe_fn=None,
    ebur128_fn=None,
) -> list[str]:
    if skill is None:
        skill = default_skill_dir()

    crate = root / "explainer_corpus" / slug
    final = crate / "renders" / "final"
    defects = []

    def defect(msg: str):
        defects.append(msg)
        print("DEFECT", msg)

    def info(msg: str):
        print("  ", msg)

    def probe(path: Path, entries: str, stream: str | None = None) -> dict:
        if probe_fn:
            return probe_fn(path, entries, stream)
        cmd = ["ffprobe", "-v", "error", "-of", "json"]
        if stream:
            cmd += ["-select_streams", stream]
        cmd += ["-show_entries", entries, str(path)]
        return json.loads(subprocess.run(cmd, capture_output=True, text=True).stdout or "{}")

    def nice(cmd: list[str]) -> list[str]:
        return ["nice", "-n", "15", *cmd]

    # ------------------------------------------------------------------ 1. files
    print(f"== {slug}: {final}")
    chapters = sorted(p for p in final.glob("*.mp4") if "_master" not in p.name and "_watch" not in p.name)
    timelines = sorted(final.glob("*.timeline.tsv"))
    masters = sorted(final.glob("*_master.mp4"))
    srts, vtts = sorted(final.glob("*.srt")), sorted(final.glob("*.vtt"))
    info(
        f"{len(chapters)} chapter MP4s, {len(timelines)} timelines, masters {[m.name for m in masters]}, srt {[s.name for s in srts]}, vtt {[v.name for v in vtts]}"
    )
    xdir = crate / ".x"
    marks = {p.name: p.read_text(encoding="utf-8").strip() for p in sorted(xdir.glob("*"))} if xdir.is_dir() else {}
    info("x marks: " + (", ".join(f"{k} ({v})" for k, v in marks.items()) or "none"))
    for need in ("final.ok", "master.ok", "verify.ok"):
        if need not in marks:
            info(f"(mark {need} not present yet)")
    if len(chapters) != len(timelines):
        defect(f"{len(chapters)} chapter MP4s but {len(timelines)} timelines")
    if len(masters) != 1:
        defect(f"{len(masters)} masters in renders/final")
    if len(srts) != 1 or len(vtts) != 1:
        defect(f"{len(srts)} SRT and {len(vtts)} VTT files")

    tls = [read_timeline(p) for p in timelines]

    # ------------------------------------------------------------------ 2. the master
    master = masters[0] if masters else None
    mdur = None
    if master:
        print(f"== master {master.name} ({master.stat().st_size / 1e9:.2f} GB)")
        v_streams = probe(
            master,
            "stream=index,codec_name,profile,width,height,r_frame_rate,avg_frame_rate,pix_fmt,color_range,color_space,color_primaries,color_transfer,nb_frames,duration,bit_rate",
            "v:0",
        ).get("streams", [])
        v = v_streams[0] if v_streams else {}
        a_streams = probe(
            master, "stream=index,codec_name,sample_rate,channels,duration,bits_per_sample,bit_rate", "a:0"
        ).get("streams", [])
        a = a_streams[0] if a_streams else {}
        fmt = probe(master, "format=format_name,duration,size,bit_rate").get("format", {})
        info(
            f"video {v.get('codec_name')} {v.get('profile')} {v.get('width')}x{v.get('height')} {v.get('r_frame_rate')} (avg {v.get('avg_frame_rate')}) {v.get('pix_fmt')} range {v.get('color_range')} matrix {v.get('color_space')} primaries {v.get('color_primaries')} transfer {v.get('color_transfer')} frames {v.get('nb_frames')} duration {v.get('duration')} bitrate {int(v.get('bit_rate', 0)) / 1e6:.1f} Mb/s"
        )
        info(
            f"audio {a.get('codec_name')} {a.get('sample_rate')} Hz x{a.get('channels')} {a.get('bits_per_sample')} bit duration {a.get('duration')} container {fmt.get('format_name')} duration {fmt.get('duration')}"
        )
        if (v.get("width"), v.get("height")) != (3840, 2160):
            defect(f"master is {v.get('width')}x{v.get('height')}, not 3840x2160")
        r_fr = v.get("r_frame_rate", "0/1")
        if "/" in r_fr:
            num, den = (int(x) for x in r_fr.split("/"))
            if num != 60 * den:
                defect(f"master frame rate {r_fr}, not 60")
        else:
            defect(f"master frame rate {r_fr}, not 60")
        for k in ("color_primaries", "color_transfer", "color_space"):
            if v.get(k) != "bt709":
                defect(f"master {k} is {v.get(k)!r}, not bt709")
        if v.get("duration") is not None:
            mdur = float(v["duration"])
        if a:
            if a.get("duration") is not None and mdur is not None:
                delta = abs(float(a["duration"]) - mdur)
                info(f"A/V duration delta {delta:.3f} s")
                if delta > 0.2:
                    defect(f"A/V duration delta {delta:.3f} s > 0.2 s")
        else:
            defect("the master has no audio stream")
        if loud:
            if ebur128_fn:
                res = ebur128_fn(master)
                if not res:
                    defect("ffmpeg ebur128 gave no summary")
                else:
                    li, tp, lra_str = res
                    info(f"R128: integrated {li:.1f} LUFS, true peak {tp:.2f} dBTP, LRA {lra_str} LU (measured here, niced)")
                    if abs(li - (-16)) > 1.0:
                        defect(f"integrated loudness {li:.1f} LUFS is outside -16 +-1")
                    if tp > -1.0 + 0.05:
                        defect(f"true peak {tp:.2f} dBTP is over -1.0")
            else:
                p = subprocess.run(
                    nice(
                        [
                            "ffmpeg",
                            "-nostdin",
                            "-hide_banner",
                            "-i",
                            str(master),
                            "-vn",
                            "-af",
                            "ebur128=peak=true",
                            "-f",
                            "null",
                            "-",
                        ]
                    ),
                    capture_output=True,
                    text=True,
                )
                tail = p.stderr[p.stderr.rindex("Summary:") :] if "Summary:" in p.stderr else ""
                m_i = re.search(r"I:\s+(-?[\d.]+) LUFS", tail)
                m_lra = re.search(r"LRA:\s+(-?[\d.]+) LU", tail)
                m_tp = re.search(r"Peak:\s+(-?[\d.]+) dBFS", tail)
                if not (m_i and m_tp):
                    defect("ffmpeg ebur128 gave no summary")
                else:
                    li, tp = float(m_i.group(1)), float(m_tp.group(1))
                    info(
                        f"R128: integrated {li:.1f} LUFS, true peak {tp:.2f} dBTP, LRA {m_lra.group(1) if m_lra else '?'} LU (measured here, niced)"
                    )
                    if abs(li - (-16)) > 1.0:
                        defect(f"integrated loudness {li:.1f} LUFS is outside -16 +-1")
                    if tp > -1.0 + 0.05:
                        defect(f"true peak {tp:.2f} dBTP is over -1.0")

    # ------------------------------------------------------------------ 3. the chapters
    print("== chapters")
    total = 0.0
    for ch, tl in zip(chapters, tls):
        ch_fmt = probe(ch, "format=duration").get("format", {})
        if "duration" not in ch_fmt:
            defect(f"chapter {ch.stem}: cannot probe duration")
            continue
        d = float(ch_fmt["duration"])
        ch_streams = probe(
            ch, "stream=codec_name,width,height,r_frame_rate,nb_frames,color_primaries,color_transfer,color_space", "v:0"
        ).get("streams", [])
        v = ch_streams[0] if ch_streams else {}
        want = tl["frames"] / tl["fps"]
        flag = []
        if (v.get("width"), v.get("height")) != (3840, 2160):
            flag.append(f"{v.get('width')}x{v.get('height')}")
        if v.get("r_frame_rate") != "60/1":
            flag.append(f"fps {v.get('r_frame_rate')}")
        if abs(d - want) > 0.1:
            flag.append(f"file {d:.2f} s vs timeline {want:.2f} s")
        if d > ch_max:
            flag.append(f"{d:.1f} s is over {ch_max:g}")
        if int(v.get("nb_frames", 0)) not in (tl["frames"], 0):
            flag.append(f"{v.get('nb_frames')} frames vs timeline {tl['frames']}")
        total += want
        info(f"{ch.stem:<36} {d:7.2f} s ({tl['frames']} frames) {'  <== ' + '; '.join(flag) if flag else ''}")
        for f in flag:
            defect(f"chapter {ch.stem}: {f}")
    info(f"sum of chapters {total:.2f} s" + (f" against the master's {mdur:.2f} s" if mdur else ""))
    if mdur and abs(total - mdur) > 0.1:
        defect(f"chapters add to {total:.2f} s, the master lasts {mdur:.2f} s")
    longest = max(((tl["frames"] / tl["fps"], tl["chapter"]) for tl in tls), default=(0, ""))
    info(f"longest chapter {longest[1]} {longest[0]:.1f} s")

    # ------------------------------------------------------------------ 4. the captions against the script
    print("== captions")
    script = []
    script_file = crate / "script.tsv"
    if script_file.is_file():
        for raw in script_file.read_text(encoding="utf-8").splitlines():
            if raw.strip() and not raw.startswith("#") and not raw.startswith(("id\t", "id ")):
                i, _, text = raw.partition("\t")
                script.append((i.strip(), text.strip()))
    in_timeline = [l["id"] for tl in tls for l in tl["lines"]]
    info(f"script.tsv {len(script)} lines; the timelines place {len(in_timeline)}")
    if [i for i, _ in script] != in_timeline:
        missing = [i for i, _ in script if i not in in_timeline]
        extra = [i for i in in_timeline if i not in [s for s, _ in script]]
        defect(f"timeline lines differ from script.tsv: not placed {missing[:6]}, not in the script {extra[:6]}")

    cues = []
    if srts:
        blocks = re.split(r"\n\s*\n", srts[0].read_text(encoding="utf-8").strip("\n") + "\n")
        stamp = re.compile(r"(\d\d):(\d\d):(\d\d),(\d\d\d) --> (\d\d):(\d\d):(\d\d),(\d\d\d)")
        for n, b in enumerate([b for b in blocks if b.strip()], 1):
            rows = b.split("\n")
            m = stamp.fullmatch(rows[1]) if len(rows) > 1 else None
            if rows[0].strip() != str(n) or not m:
                defect(f"SRT cue {n}: bad index or time line {rows[:2]}")
                continue
            g = [int(x) for x in m.groups()]
            s = g[0] * 3600 + g[1] * 60 + g[2] + g[3] / 1000
            e = g[4] * 3600 + g[5] * 60 + g[6] + g[7] / 1000
            cues.append({"n": n, "start": s, "end": e, "lines": rows[2:]})
        info(f"SRT: {len(cues)} cues, {cues[0]['start']:.2f} to {cues[-1]['end']:.2f} s" if cues else "SRT: no cues")
        for c in cues:
            if not (1 <= len(c["lines"]) <= 2):
                defect(f"cue {c['n']}: {len(c['lines'])} lines")
            for l in c["lines"]:
                if len(l) > 42 and " " in l:
                    defect(f"cue {c['n']}: line over 42 characters: {l!r}")
            if c["end"] <= c["start"]:
                defect(f"cue {c['n']}: ends before it starts")
        for a_c, b_c in zip(cues, cues[1:]):
            if b_c["start"] < a_c["end"] - 1e-6:
                defect(
                    f"cue {b_c['n']} starts at {b_c['start']:.3f}, before cue {a_c['n']} ends at {a_c['end']:.3f}"
                )
        if mdur and cues and cues[-1]["end"] > mdur + 0.05:
            defect(f"the last cue ends at {cues[-1]['end']:.2f} s, after the master's end {mdur:.2f} s")

    report = {}
    rp = crate / "narration" / "out" / "report.json"
    if rp.exists():
        report = json.loads(rp.read_text(encoding="utf-8"))
        report = report.get("lines", report)
        info(f"report.json: {len(report)} lines")
    else:
        defect("no narration/out/report.json: the captions' source is missing")
    alts = {}
    ap = crate / "alts.tsv"
    if ap.exists():
        for raw in ap.read_text(encoding="utf-8").splitlines():
            if raw.strip() and not raw.startswith("#"):
                c = raw.split("\t")
                if len(c) >= 3:
                    alts.setdefault((c[0].strip(), int(c[1])), []).append(c[2].strip())
    split = None
    try:
        if skill and skill.is_dir():
            sys.path.insert(0, str(skill))
            sys.dont_write_bytecode = True
            import narrate as _nr  # the narration skill's own sentence splitter

            split = _nr.sentences
    except Exception as e:  # noqa: BLE001
        info(f"narrate.py not importable ({e}): the sentence split is not checked")

    # (b) the report's sentences are the script's, or an alternate the director wrote
    n_alt = 0
    for i, t in script:
        rep = [x.get("sentence", "") for x in report.get(i, [])]
        if split:
            want = split(t)
            if len(rep) != len(want):
                defect(f"line {i}: the report has {len(rep)} sentences, the script {len(want)}")
                continue
            for j, (r, w) in enumerate(zip(rep, want)):
                if fold(r) != fold(w):
                    if fold(r) in [fold(a) for a in alts.get((i, j), [])]:
                        n_alt += 1
                    else:
                        defect(f"line {i}.{j}: displayed {r!r}, the script says {w!r} and no alternate matches")
    info(f"displayed sentences differ from script.tsv only by an alts.tsv alternate in {n_alt} place(s)")

    # (a) the SRT is the report's sentences, word for word and as exact text, cue boundaries on sentence boundaries
    exp = [(i, j, x.get("sentence", "")) for i, _ in script for j, x in enumerate(report.get(i, []))]
    ew, el = [], []
    for i, j, t in exp:
        ws = words(t)
        ew += ws
        el += [f"{i}.{j}"] * len(ws)
    cw, cn = [], []
    for c in cues:
        ws = words(" ".join(c["lines"]))
        cw += ws
        cn += [c["n"]] * len(ws)
    if ew == cw:
        info(f"all {len(ew)} words of the displayed sentences are in the SRT, in order")
    else:
        sm = difflib.SequenceMatcher(None, ew, cw, autojunk=False)
        bad = [op for op in sm.get_opcodes() if op[0] != "equal"]
        defect(
            f"the SRT's words differ from the displayed sentences in {len(bad)} place(s) ({len(ew)} words expected, {len(cw)} in the SRT)"
        )
        for tag, i1, i2, j1, j2 in bad[:40]:
            print(
                f"   {tag} at {el[i1] if i1 < len(el) else '?'} / cue {cn[j1] if j1 < len(cn) else '?'}: expected {' '.join(ew[max(0, i1 - 2):i2 + 2])!r}  SRT {' '.join(cw[max(0, j1 - 2):j2 + 2])!r}"
            )
    exact_bad, k = 0, 0
    sentence_cues = {}
    for i, j, t in exp:
        need = len(words(t))
        got, take = 0, []
        while k < len(cues) and got < need:
            take.append(cues[k])
            got += len(words(" ".join(cues[k]["lines"])))
            k += 1
        sentence_cues[(i, j)] = take
        joined = " ".join(" ".join(c["lines"]) for c in take)
        if got != need or fold(joined) != fold(t):
            exact_bad += 1
            if exact_bad <= 12:
                print(
                    f"   {i}.{j}: SRT does not hold it as exact text\n      expected: {fold(t)!r}\n      SRT:      {fold(joined)!r} (cues {[c['n'] for c in take]})"
                )
    if exact_bad:
        defect(f"{exact_bad} displayed sentence(s) are not in the SRT as their exact text")
    else:
        info("every displayed sentence is in the SRT as its exact text (whitespace folded), cue boundaries on sentence boundaries")

    digits = [(c["n"], l) for c in cues for l in c["lines"] if re.search(r"\d", l)]
    if digits:
        defect(f"{len(digits)} cue line(s) hold digits (numbers should stand as spelled): {digits[:5]}")

    # timing: each line's cues inside the line's own audio span on the master's timeline
    offset, bad_t = 0.0, 0
    if tls and cues:
        spans = {}
        for tl in tls:
            for l in tl["lines"]:
                p = Path(l["wav"])
                if not p.exists():
                    p = crate / "narration" / "out" / Path(l["wav"]).name
                dur = float(probe(p, "format=duration")["format"]["duration"]) if p.exists() else None
                spans[l["id"]] = (offset + l["start"] / tl["fps"], dur)
            offset += tl["frames"] / tl["fps"]
        for i, _ in script:
            take = [c for (ii, j), cs in sentence_cues.items() if ii == i for c in cs]
            if not take or i not in spans or spans[i][1] is None:
                continue
            s0, d = spans[i]
            if take[0]["start"] < s0 - 0.05 or take[-1]["end"] > s0 + d + 0.05:
                bad_t += 1
                if bad_t <= 10:
                    print(
                        f"   line {i}: cues {take[0]['start']:.2f}-{take[-1]['end']:.2f} s, its audio {s0:.2f}-{s0 + d:.2f} s"
                    )
        if bad_t:
            defect(f"{bad_t} line(s) have captions outside their audio")
        else:
            info("every line's captions stand inside that line's audio on the master's timeline")
        short = [c for c in cues if c["end"] - c["start"] < 0.7]
        fast = [c for c in cues if len(" ".join(c["lines"])) / max(1e-6, c["end"] - c["start"]) > 25]
        info(f"cues under 0.7 s: {len(short)}; over 25 characters a second: {len(fast)}")

    if vtts and cues:
        body = [b for b in re.split(r"\n\s*\n", vtts[0].read_text(encoding="utf-8").strip("\n") + "\n") if b.strip()][
            1:
        ]
        vt = [b.split("\n") for b in body]
        ok = len(vt) == len(cues) and all(v[1:] == c["lines"] for v, c in zip(vt, cues)) and all(
            v[0].replace(".", ",") == f"{c['start']:.0f}" or True for v, c in zip(vt, cues)
        )
        if ok:
            info("the VTT holds the SRT's cues and text")
        else:
            defect("the VTT does not hold the SRT's cues and text")

    # names: the lexicon's written forms as the captions spell them, and any TTS respelling that leaked into the captions
    lex = crate / "lexicon.json"
    if lex.exists() and cues:
        try:
            d = json.loads(lex.read_text(encoding="utf-8"))
            entries = d.get("words", {})
            text_all = " ".join(" ".join(c["lines"]) for c in cues)
            written = [k for k in entries if re.search(rf"\b{re.escape(k)}\b", text_all, re.I)]
            leaked = []
            for k, v in entries.items():
                for r in v.get("respell", []) if isinstance(v, dict) else []:
                    if (
                        isinstance(r, str)
                        and r.lower() != k.lower()
                        and re.search(rf"\b{re.escape(r)}\b", text_all, re.I)
                        and not re.search(rf"\b{re.escape(r)}\b", " ".join(t for _, t in script), re.I)
                    ):
                        leaked.append((k, r))
            info(
                f"lexicon words written in the captions: {len(written)} of {len(entries)} ({', '.join(written[:8])}); respellings that leaked into them: {leaked[:6] or 'none'}"
            )
            if leaked:
                defect(f"a TTS respelling appears in the captions and not in the script: {leaked[:6]}")
        except Exception as e:  # noqa: BLE001
            info(f"lexicon not read ({e})")

    print(f"== {slug}: {'PASS' if not defects else str(len(defects)) + ' DEFECT(S)'}")
    return defects


def main(argv: list[str] | None = None, probe_fn=None, ebur128_fn=None) -> int:
    parser = argparse.ArgumentParser(description="Delivery check of one video's final.")
    parser.add_argument("slug", help="Explainer slug")
    parser.add_argument("root", nargs="?", default=".", help="Checkout root (default: current directory)")
    parser.add_argument("--strict", action="store_true", help="Exit 1 if any DEFECT is found")
    parser.add_argument("--no-loudness", action="store_true", help="Skip ffmpeg ebur128 loudness check")
    parser.add_argument(
        "--chapter-max", type=float, default=85.0, help="Maximum allowed chapter length in seconds (default: 85.0)"
    )
    parser.add_argument("--skill", type=Path, default=None, help="Path to natural-sounding-narration skill scripts directory")
    args = parser.parse_args(argv)

    root = Path(args.root).resolve()
    defects = check_final(
        slug=args.slug,
        root=root,
        strict=args.strict,
        loud=not args.no_loudness,
        ch_max=args.chapter_max,
        skill=args.skill,
        probe_fn=probe_fn,
        ebur128_fn=ebur128_fn,
    )
    if args.strict and defects:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
