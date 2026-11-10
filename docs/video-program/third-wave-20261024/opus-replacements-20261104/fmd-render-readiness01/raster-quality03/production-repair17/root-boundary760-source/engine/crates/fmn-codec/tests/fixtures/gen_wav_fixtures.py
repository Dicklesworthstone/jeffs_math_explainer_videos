#!/usr/bin/env python3
"""Generate the WAV fixture matrix (fm-65l): the same 64-sample ramp
(sample k = (k-32)/32) in u8/s16/s24/s32 via CPython's wave module and
f32 via a hand-built RIFF — implementations independent of fmn-codec.
Deterministic: rerunning reproduces identical bytes."""

import os
import struct
import wave

HERE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "wav")
os.makedirs(HERE, exist_ok=True)

RAMP = [(k - 32) / 32.0 for k in range(64)]


def clamp(v, lo, hi):
    return max(lo, min(hi, v))


def write_pcm(name, width, scale, offset=0, signed=True):
    path = os.path.join(HERE, name)
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(width)
        w.setframerate(8000)
        data = bytearray()
        for v in RAMP:
            q = clamp(round(v * scale) + offset,
                      -scale if signed else 0,
                      scale - 1 if signed else 2 * scale - 1)
            data += int(q).to_bytes(width, "little", signed=signed)
        w.writeframes(bytes(data))
    print(name, os.path.getsize(path))


write_pcm("ramp_u8.wav", 1, 128, offset=128, signed=False)
write_pcm("ramp_s16.wav", 2, 32768)
write_pcm("ramp_s24.wav", 3, 8388608)
write_pcm("ramp_s32.wav", 4, 2147483648)

# IEEE float32 (format tag 3), hand-built RIFF.
payload = b"".join(struct.pack("<f", v) for v in RAMP)
fmt = struct.pack("<HHIIHH", 3, 1, 8000, 8000 * 4, 4, 32)
body = (b"fmt " + struct.pack("<I", len(fmt)) + fmt
        + b"data" + struct.pack("<I", len(payload)) + payload)
riff = b"RIFF" + struct.pack("<I", 4 + len(body)) + b"WAVE" + body
open(os.path.join(HERE, "ramp_f32.wav"), "wb").write(riff)
print("ramp_f32.wav", len(riff))
