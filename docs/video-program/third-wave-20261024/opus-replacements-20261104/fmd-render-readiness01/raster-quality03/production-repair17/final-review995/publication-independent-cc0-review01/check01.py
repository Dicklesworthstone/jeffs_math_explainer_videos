"""check01.py: does publication965 (title, description, tags) fit the completed master993 as it is?

Independent of root's publication-chapters-checked993.json: chapter starts are re-derived from the summary's
own rows (start_frame, frames), not from root's list, and then compared with both.

usage: python3 -I -B check01.py PR17_DIR OUT
"""
import hashlib
import json
import re
import sys

D, OUT = sys.argv[1], sys.argv[2]
FPS = 60
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def stamp(sec):
    s = int(sec)
    return "%d:%02d" % divmod(s, 60) if s < 3600 else "%d:%02d:%02d" % (s // 3600, s % 3600 // 60, s % 60)


summ = json.load(open(D + "/completed-master993-summary.json"))
checked = json.load(open(D + "/publication-chapters-checked993.json"))
desc = open(D + "/publication965/description.txt", encoding="utf-8").read()
title = open(D + "/publication965/title.txt", encoding="utf-8").read().strip()
tags = open(D + "/publication965/tags.txt", encoding="utf-8").read().strip()
say("summary sha256 %s (checked993 binds %s)" % (sha(D + "/completed-master993-summary.json"), checked["master_summary_sha256"]))
say("description sha256 %s (checked993 binds %s)" % (sha(D + "/publication965/description.txt"), checked["description_sha256"]))

rows = summ["rows"]
total = summ["frames"]
say("== master993: %d rows, %d frames = %.3f s; outputs master %s" % (len(rows), total, total / FPS, summ["outputs"]["master"]["sha256"][:16]))
at = 0
starts = []
for r in rows:
    if r["start_frame"] != at:
        bad.append("%s starts at %d, expected %d" % (r["chapter"], r["start_frame"], at))
    starts.append((r["chapter"], r["start_frame"], r["frames"]))
    at += r["frames"]
if at != total:
    bad.append("rows sum to %d frames, master has %d" % (at, total))
say("rows contiguous and summing to the master: %s" % (at == total and not bad))
say("total %s (%d:%06.3f)" % (stamp(total / FPS), total // FPS // 60, total / FPS % 60))

marks = re.findall(r"(?m)^(\d+:\d{2}(?::\d{2})?) (.+)$", desc.split("Chapters:")[1].split("\n\n")[0])
say("== description chapter list: %d marks" % len(marks))
for (ch, sf, n), (ts, name), c in zip(starts, marks, checked["chapters"]):
    want = stamp(sf / FPS)
    ok = ts == want == c["timestamp"] and c["start_frame"] == sf and c["chapter"] == ch and c["title"] == name
    if not ok:
        bad.append("%s: description %s, frames give %s, checked993 %s/%d" % (ch, ts, want, c["timestamp"], c["start_frame"]))
    say("%-17s start %6d = %9.3f s -> %-6s description %-6s  length %7.3f s  %s  | %s"
        % (ch, sf, sf / FPS, want, ts, n / FPS, "ok" if ok else "MISMATCH", name))
if len(marks) != len(starts):
    bad.append("%d marks for %d chapters" % (len(marks), len(starts)))
say("YouTube chapter rules: first 0:00 %s; at least 3 %s; ascending %s; each >= 10 s %s"
    % (marks[0][0] == "0:00", len(marks) >= 3, [m for _, m, _ in starts] == sorted(m for _, m, _ in starts),
       all(n / FPS >= 10 for _, _, n in starts)))

say("== captions (local copies)")
for ext, key in (("srt", "srt"), ("vtt", "vtt")):
    p = D + "/complex-analysis-master993.en." + ext
    h = sha(p)
    text = open(p, encoding="utf-8").read()
    ends = re.findall(r"--> (\d{2}):(\d{2}):(\d{2})[,.](\d{3})", text)
    last = max(int(a) * 3600 + int(b) * 60 + int(c) + int(d) / 1000 for a, b, c, d in ends)
    match = h == summ["outputs"][key]["sha256"]
    if not match or last > total / FPS:
        bad.append("%s: sha match %s, last end %.3f" % (ext, match, last))
    say("%s sha256 %s… = summary %s; %d cues; last end %.3f s <= %.3f %s"
        % (ext, h[:16], match, len(ends), last, total / FPS, last <= total / FPS))

say("== limits and credits")
say("title %d chars (<= 100): %r" % (len(title), title))
say("description %d chars (<= 5000)" % len(desc))
tag_list = [t.strip() for t in tags.split(",")]
yt = sum(len(t) + (2 if " " in t else 0) for t in tag_list) + len(tag_list) - 1
say("tags %d, %d chars raw, %d counted with quotes for multi-word tags (<= 500)" % (len(tag_list), len(tags), yt))
say("angle brackets in title/description/tags: %s" % any(c in s for s in (title, desc, tags) for c in "<>"))
for want in ("Narration: ElevenLabs Frederick Surrey, a synthetic voice.", "Animation: FrankenManim.",
             "Music: procedural score generated in code.", "https://youtu.be/lgpTDROHgQY",
             "Sister video, Logarithms Made Visual: From Slide Rules to Matrices"):
    say("present: %-62s %s" % (want, want in desc))
for word in ("Oliver", "Michael", "FrankenTTS"):
    if word in desc:
        bad.append("description names %s" % word)
say("other voice names (Oliver, Michael, FrankenTTS): %s" % [w for w in ("Oliver", "Michael", "FrankenTTS") if w in desc])
say("score present in master993: %s" % bool(summ.get("score")))
say("RESULT: %s" % ("no mismatch" if not bad else "; ".join(bad)))
with open(OUT, "x", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
