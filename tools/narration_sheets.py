#!/usr/bin/env python3
"""Spectrogram sheets of a narration, to read as images: listening by eye.

    uv run --no-project --with numpy --with matplotlib python3 -I tools/narration_sheets.py \
        SKILL_SCRIPTS NARRATION_DIR SHEETS_DIR [--takes] [LINE_ID...]

NARRATION_DIR holds `out/` (the line WAVs and report.json) and `work/` (the
takes and asr.json). With no LINE_ID, every line.

Default, one PNG per line, of the audio that ships: the spectrogram to
8 kHz, each sentence under its own words, the pitch track in cyan (drawn at
20 times its frequency: 100 Hz sits at 2 kHz), the silence put between
sentences hatched, and the level underneath.

--takes: one PNG per line, of each chosen take as ftts made it, with what the
editor threw away hatched and the ASR's words at their start times.

What to look for. In a take: sound in a hatched region that is part of a
word (a final "s" is a tall bright column above 4 kHz); bursts, clicks or
smears that are not speech left unhatched. In a line: a sentence that starts
or stops as a hard vertical edge; a pitch track that is a flat line (a read
with no shape) or that rises at the end of a statement; words packed with no
dark between them (rushed); long dark stretches inside a sentence (a stall).
"""
import json
import sys
from pathlib import Path

import numpy as np

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

sys.dont_write_bytecode = True  # importing narrate must leave no __pycache__ in the skill
sys.path.insert(0, sys.argv[1])
import narrate as nr  # noqa: E402

PX_PER_S, DPI = 150, 80


def spectrogram(ax, x, rate):
    n, hop = 512, 120
    fr = np.lib.stride_tricks.sliding_window_view(np.pad(x, n // 2), n)[::hop] * np.hanning(n)
    s = 20 * np.log10(np.abs(np.fft.rfft(fr, axis=1)).T + 1e-9)
    dur = len(x) / rate
    ax.imshow(s[: int(8000 / (rate / n))], origin="lower", aspect="auto", cmap="magma",
              extent=[0, dur, 0, 8], vmin=s.max() - 85, vmax=s.max())
    ax.set_xlim(0, dur)
    ax.set_xticks(np.arange(0, dur, 0.5))
    ax.tick_params(labelsize=8)
    return dur


def level(ax, x, rate, dur, line=None):
    db = nr.frame_db(x, rate)
    ax.plot(np.arange(len(db)) * 0.01, db, color="black", lw=1)
    if line is not None:
        ax.axhline(line, color="red", lw=1)
    ax.set_xlim(0, dur)
    ax.set_ylim(-100, -5)
    ax.set_xticks(np.arange(0, dur, 0.5))
    ax.tick_params(labelsize=8)
    ax.set_ylabel("dBFS", fontsize=9)


def figure(rows, dur):
    width = min(60.0, max(10.0, 1.0 + dur * PX_PER_S / DPI))
    return plt.subplots(2 * rows, 1, figsize=(width, 4.4 * rows), dpi=DPI, squeeze=False,
                        gridspec_kw={"height_ratios": [3, 1] * rows})


def line_sheet(lid, sents, out_dir, png):
    x, rate = nr.load(out_dir / f"{lid}.wav")
    fig, axes = figure(1, len(x) / rate)
    ax, bx = axes[0][0], axes[1][0]
    dur = spectrogram(ax, x, rate)
    f0 = nr.f0_track(x, rate)
    ax.plot(np.arange(len(f0)) * 0.01, f0 * 20 / 1000, ".", color="cyan", ms=3)
    pos = 0
    for s in sents:
        a, b = pos / rate, (pos + s["out_samples"]) / rate
        gap = s["gap_after_samples"] / rate
        room = max(8, int((b - a + gap) * PX_PER_S / 9))
        text = s["sentence"] if len(s["sentence"]) <= room else s["sentence"][: room - 1] + "…"
        ax.text(a, 8.15, f"{s['j']}: {text}", fontsize=12, ha="left", va="bottom")
        ax.text(a, 8.75, f"{s['syl_rate']} syl/s, end {s['final_st']:+.1f} st, asr {s['asr_ratio']}", fontsize=9,
                ha="left", va="bottom", color="gray")
        for q in (ax, bx):
            q.axvline(a, color="cyan", lw=1)
            q.axvline(b, color="cyan", lw=1)
            if gap:
                q.axvspan(b, b + gap, facecolor="none", edgecolor="white" if q is ax else "red", hatch="///", lw=0)
        pos += s["out_samples"] + s["gap_after_samples"]
    ax.set_ylabel(lid, fontsize=12)
    level(bx, x, rate, dur)
    fig.tight_layout()
    fig.savefig(png)
    plt.close(fig)


def take_sheet(lid, sents, work, asr, png):
    takes = [nr.load(work / "takes" / f"{s['read']}.wav") for s in sents]
    fig, axes = figure(len(sents), max(len(x) / r for x, r in takes))
    for k, (s, (x, rate)) in enumerate(zip(sents, takes)):
        ax, bx = axes[2 * k][0], axes[2 * k + 1][0]
        dur = spectrogram(ax, x, rate)
        if "cuts" in s:  # narration_choose.py's report: the cuts are written down
            cut = [(0, s["start"]), (s["end"], dur)] + [(a, b) for a, b, _ in s["cuts"]]
        else:            # narrate.py's own editor, from its rules
            cut = [(0, max(0.0, s["start"] - 0.05)), (min(dur, s["end"] + 0.12), dur)]
            for p in s["pauses"]:
                cap = nr.COMMA_PAUSE_CAP if p["punct"] in (",", ".") else (
                    nr.UNMATCHED_PAUSE_CAP if p["punct"] == nr.UNMATCHED else nr.OTHER_PAUSE_CAP)
                if p["len"] > cap:
                    mid, half = p["at"] + p["len"] / 2, (p["len"] - cap) / 2
                    cut.append((mid - half, mid + half))
        for a, b in cut:
            for q in (ax, bx):
                q.axvspan(a, b, facecolor="none", edgecolor="white" if q is ax else "red", hatch="///", lw=0)
                q.axvline(a, color="cyan", lw=1)
                q.axvline(b, color="cyan", lw=1)
        for w in asr[s["read"]]["words"]:
            ax.text(w["s"], 8.15, w["w"].strip(), fontsize=11, rotation=30, ha="left", va="bottom")
            ax.axvline(w["s"], color="white", lw=0.4, alpha=0.5)
        ax.set_ylabel(f"{lid}.{s['j']}\ncut {s['pause_cut_s']} s", fontsize=11)
        level(bx, x, rate, dur, s["top_db"] - 20.0 if "top_db" in s else -48.0)
    fig.tight_layout()
    fig.savefig(png)
    plt.close(fig)


def main():
    args = [a for a in sys.argv[2:] if a != "--takes"]
    takes = "--takes" in sys.argv
    if len(args) < 2:
        sys.exit(__doc__)
    narration, sheets = Path(args[0]), Path(args[1])
    report = json.loads((narration / "out" / "report.json").read_text())
    lines = report.get("lines", report)
    asr = json.loads((narration / "work" / "asr.json").read_text()) if takes else None
    sheets.mkdir(parents=True, exist_ok=True)
    for lid in args[2:] or list(lines):
        if lid not in lines:
            sys.exit(f"no line {lid} in {narration / 'out' / 'report.json'}")
        png = sheets / (f"{lid}.takes.png" if takes else f"{lid}.png")
        if takes:
            take_sheet(lid, lines[lid], narration / "work", asr, png)
        else:
            line_sheet(lid, lines[lid], narration / "out", png)
        print(png)


if __name__ == "__main__":
    main()
