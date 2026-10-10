#!/usr/bin/env python3
"""Narration QA: transcribe every narration/<id>.wav with franken_whisper and
compare it word-by-word with the script, so a dropped, repeated or garbled
TTS line is caught without anyone having to listen.

    python3 qa_narration.py HOEFFDING_BIN [--jobs 2] [--only ID,ID]
"""
import argparse
import concurrent.futures
import difflib
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

# Spellings the ASR reasonably prefers for names the TTS says correctly.
ALIASES = {
    "hoefting": "hoeffding", "hoefting's": "hoeffding's", "hofding": "hoeffding",
    "hoeffdings": "hoeffding's", "manam": "manim", "mannim": "manim",
    "tts": "t t s", "1948": "nineteen forty-eight",
}
NUMBER_WORDS = {"7": "seven", "4": "four", "5": "five", "3": "three", "9": "nine", "6": "six",
                "8": "eight", "10": "ten", "2": "two", "1": "one", "48": "forty-eight",
                "150": "a hundred and fifty", "300": "three hundred", "2,000": "two thousand",
                "2000": "two thousand", "5,000": "five thousand", "5000": "five thousand",
                "0.41": "zero point four one", "0.93": "zero point nine three",
                "0.86": "zero point eight six", "414": "four hundred fourteen", "78": "seventy-eight",
                "8.5": "eight and a half", "-0.5": "minus one half", "4x4": "four by four"}


def words(text):
    text = text.lower().replace("—", " ").replace("-", " ")
    out = []
    for w in re.findall(r"[a-z0-9.,']+", text):
        w = w.strip(".,'")
        if not w:
            continue
        w = ALIASES.get(w, NUMBER_WORDS.get(w, w))
        out.extend(w.replace("-", " ").split())
    return out


def transcribe(wav):
    proc = subprocess.run(
        ["franken_whisper", "transcribe", "--input", str(wav), "--no-diarize", "--language", "en"],
        capture_output=True, text=True, timeout=600,
    )
    lines = [l.strip() for l in proc.stdout.splitlines() if l.strip()]
    return " ".join(lines), proc.returncode


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("hoeffding")
    ap.add_argument("--jobs", type=int, default=2)
    ap.add_argument("--only", default="")
    args = ap.parse_args()
    script = subprocess.run([args.hoeffding, "script"], capture_output=True, text=True, check=True).stdout
    lines = [l.split("\t", 1) for l in script.splitlines() if "\t" in l]
    if args.only:
        keep = set(args.only.split(","))
        lines = [l for l in lines if l[0] in keep]

    def check(item):
        lid, text = item
        heard, rc = transcribe(HERE / "narration" / f"{lid}.wav")
        ratio = difflib.SequenceMatcher(None, words(text), words(heard)).ratio()
        return lid, ratio, rc, text, heard

    bad = 0
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        for lid, ratio, rc, text, heard in pool.map(check, lines):
            flag = "OK " if ratio >= 0.85 and rc == 0 else "BAD"
            bad += flag == "BAD"
            print(f"{flag} {lid:4} {ratio:.2f}  heard: {heard}", flush=True)
            if flag == "BAD":
                print(f"         want: {text}", flush=True)
    print(f"{len(lines) - bad}/{len(lines)} lines match the script")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
