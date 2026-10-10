#!/usr/bin/env python3
"""Synthesize a TSV of (tag, text) lines with FrankenTTS, in parallel.

    python3 tts_batch.py JOBS.tsv OUT_DIR [--voice robert] [--workers 12]

Each line becomes OUT_DIR/<tag>.wav (24 kHz mono PCM from `ftts say`).
Existing non-empty WAVs are kept, so the batch is resumable. Every process
loads its own model (--no-resident) so workers run truly in parallel.
"""
import argparse
import concurrent.futures
import os
import subprocess
import sys
import time
from pathlib import Path

FTTS = os.environ.get("FTTS", os.path.expanduser("~/.local/bin/ftts"))


def synth(tag, text, out_dir, voice):
    wav = out_dir / f"{tag}.wav"
    if wav.exists() and wav.stat().st_size > 1000:
        return tag, "kept", 0.0
    partial = out_dir / f"{tag}.partial.wav"
    log = out_dir / f"{tag}.log"
    started = time.time()
    with open(log, "ab") as sink:
        rc = subprocess.run(
            [FTTS, "say", "--no-resident", "--voice", voice, text, str(partial)],
            stdin=subprocess.DEVNULL, stdout=sink, stderr=sink,
        ).returncode
    elapsed = time.time() - started
    if rc != 0 or not partial.exists() or partial.stat().st_size <= 1000:
        return tag, f"FAILED rc={rc}", elapsed
    partial.replace(wav)
    return tag, "ok", elapsed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("jobs")
    ap.add_argument("out_dir")
    ap.add_argument("--voice", default="robert")
    ap.add_argument("--workers", type=int, default=12)
    args = ap.parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    jobs = [l.rstrip("\n").split("\t", 1) for l in open(args.jobs) if "\t" in l]
    failed = 0
    with concurrent.futures.ThreadPoolExecutor(args.workers) as pool:
        futures = [pool.submit(synth, tag, text, out_dir, args.voice) for tag, text in jobs]
        for f in concurrent.futures.as_completed(futures):
            tag, status, secs = f.result()
            failed += status.startswith("FAILED")
            print(f"{status:12} {secs:6.1f}s  {tag}", flush=True)
    print(f"{len(jobs) - failed}/{len(jobs)} synthesized")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
