#!/usr/bin/env python3
"""Transcribe every WAV in a directory with faster-whisper (one model load),
writing word-level timings to OUT.json. Used to score TTS takes.

    uv run --with faster-whisper python3 asr_batch.py WAV_DIR OUT.json [--device cpu|cuda]
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

import numpy as np


def preload_cuda_libs():
    """CTranslate2 dlopens cuBLAS/cuDNN by soname; load them from the
    nvidia-*-cu12 wheels (uv run --with nvidia-cublas-cu12 --with
    nvidia-cudnn-cu12) so no system CUDA toolkit is needed."""
    import ctypes
    import importlib
    for module, names in (("nvidia.cublas", ("libcublasLt.so.12", "libcublas.so.12")),
                          ("nvidia.cudnn", ("libcudnn.so.9",))):
        lib_dir = Path(importlib.import_module(module).__path__[0]) / "lib"
        for name in names:
            ctypes.CDLL(str(lib_dir / name), mode=ctypes.RTLD_GLOBAL)
        for extra in sorted(lib_dir.glob("libcudnn_*.so.9")):
            ctypes.CDLL(str(extra), mode=ctypes.RTLD_GLOBAL)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("wav_dir")
    ap.add_argument("out")
    ap.add_argument("--model", default="large-v3-turbo")
    ap.add_argument("--device", default="cpu")
    ap.add_argument("--only", default="", help="comma-separated tag prefixes")
    args = ap.parse_args()
    if args.device == "cuda":
        preload_cuda_libs()
    from faster_whisper import WhisperModel
    compute = "float16" if args.device == "cuda" else "int8"
    model = WhisperModel(args.model, device=args.device, compute_type=compute,
                         cpu_threads=32 if args.device == "cpu" else 0)
    out_path = Path(args.out)
    results = json.loads(out_path.read_text()) if out_path.exists() else {}
    wavs = sorted(Path(args.wav_dir).glob("*.wav"))
    if args.only:
        prefixes = tuple(args.only.split(","))
        wavs = [w for w in wavs if w.stem.startswith(prefixes)]
    for wav in wavs:
        if wav.stem in results or wav.name.endswith(".partial.wav"):
            continue
        # Decode with ffmpeg to 16 kHz mono float32 (the PyAV loader in
        # faster-whisper is version-fragile).
        pcm = subprocess.run(
            ["ffmpeg", "-v", "error", "-i", str(wav), "-f", "f32le", "-ac", "1", "-ar", "16000", "-"],
            capture_output=True, check=True,
        ).stdout
        audio = np.frombuffer(pcm, dtype=np.float32)
        segments, _ = model.transcribe(audio, language="en", beam_size=5,
                                       word_timestamps=True, vad_filter=False)
        words = []
        text = []
        for seg in segments:
            text.append(seg.text.strip())
            for w in seg.words or []:
                words.append({"w": w.word.strip(), "s": round(w.start, 3), "e": round(w.end, 3),
                              "p": round(w.probability, 3)})
        results[wav.stem] = {"text": " ".join(text), "words": words}
        print(f"{wav.stem}: {results[wav.stem]['text']}", flush=True)
        out_path.write_text(json.dumps(results, indent=1))
    print(f"{len(results)} transcripts -> {out_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
