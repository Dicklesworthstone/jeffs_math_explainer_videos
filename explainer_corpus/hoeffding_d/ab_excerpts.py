"""v5 vs v6 excerpts for the owner's A/B (bead W3-ab): the hook, the
gallery's ring verdict, the shuffle test. Each pair is levelled to one
common integrated loudness by a fixed gain per file (-16 LUFS unless a
-1.5 dBTP ceiling binds), with 0.3 s fades at the cut edges. No loudnorm.

    python3 -I ab_excerpts.py RENDERS_DIR WORK_DIR

RENDERS_DIR holds score_ab/hoeffdings_d_4k_v5_score_old_ab.mp4 (v5's
picture and sound) and hoeffding_d_4k_v6.mp4; the excerpts go to
RENDERS_DIR/ab_v6/ (which must not exist), with levels.json. WORK_DIR keeps
the intermediates (cut video, PCM audio)."""
import json, pathlib, re, subprocess, sys

ROOT = pathlib.Path(sys.argv[1])          # explainer_corpus/hoeffding_d/renders
OUT = ROOT / "ab_v6"
WORK = pathlib.Path(sys.argv[2])     # intermediates (scratch)
V5 = ROOT / "score_ab/hoeffdings_d_4k_v5_score_old_ab.mp4"
V6 = ROOT / "hoeffding_d_4k_v6.mp4"
# chapter starts (s) from the chapters' frame counts at 60 fps
V5_START = {"01_hook": 0, "02_gallery": 2642 / 60, "07_shuffle": 22687 / 60}
V6_START = {"01_hook": 0, "02_gallery": 2792 / 60, "07_shuffle": 23433 / 60}
# (name, chapter, chapter-relative start, v5 seconds, v6 seconds)
CUTS = [
    ("hook", "01_hook", 0.0, 30.0, 30.0),
    ("gallery_ring", "02_gallery", 33.0, 30.0, 30.0),
    # v5's chapter ends at 82.6 s; v6 runs on through its listen window
    ("shuffle_test", "07_shuffle", 53.0, 29.5, 36.5),
]

def run(args):
    r = subprocess.run(["ffmpeg", "-nostdin", "-hide_banner", "-y"] + args, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"ffmpeg failed: {' '.join(args)}\n{r.stderr[-2000:]}")
    return r.stderr

def loudness(wav):
    err = run(["-i", str(wav), "-af", "ebur128=peak=true", "-f", "null", "-"])
    summary = err[err.rindex("Summary:"):]
    i = float(re.search(r"I:\s+(-?[\d.]+) LUFS", summary).group(1))
    tp = float(re.search(r"Peak:\s+(-?[\d.]+) dBFS", summary).group(1))
    return i, tp

OUT.mkdir(exist_ok=False)
WORK.mkdir(parents=True, exist_ok=True)
report = []
for name, ch, rel, d5, d6 in CUTS:
    parts = {}
    for tag, src, start, dur in [("v5", V5, V5_START[ch] + rel, d5), ("v6", V6, V6_START[ch] + rel, d6)]:
        base = WORK / f"{name}_{tag}"
        v = base.with_suffix(".video.mp4")
        a = base.with_suffix(".wav")
        run(["-ss", f"{start:.4f}", "-t", f"{dur:.4f}", "-i", str(src), "-map", "0:v:0", "-an",
             "-c:v", "libx264", "-crf", "16", "-preset", "medium", "-tune", "animation", "-pix_fmt", "yuv420p",
             "-bsf:v", "h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1", str(v)])
        run(["-ss", f"{start:.4f}", "-t", f"{dur:.4f}", "-i", str(src), "-map", "0:a:0", "-vn",
             "-af", f"afade=t=in:d=0.3,afade=t=out:st={dur - 0.3:.4f}:d=0.3", "-c:a", "pcm_f32le", str(a)])
        parts[tag] = (v, a, *loudness(a))
    target = min(-16.0, *(i + (-1.5 - tp) for (_, _, i, tp) in parts.values()))
    for tag, (v, a, i, tp) in parts.items():
        g = target - i
        out = OUT / f"{name}_{tag}.mp4"
        run(["-i", str(v), "-i", str(a), "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy",
             "-af", f"volume={g:.2f}dB", "-c:a", "aac", "-b:a", "256k", "-ar", "48000",
             "-movflags", "+faststart", str(out)])
        report.append({"file": out.name, "source_lufs": i, "source_dbtp": tp, "gain_db": round(g, 2),
                       "lufs": round(i + g, 2), "dbtp": round(tp + g, 2)})
print(json.dumps(report, indent=1))
(OUT / "levels.json").write_text(json.dumps(report, indent=1))
