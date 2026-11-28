"""Inspect the stereo soundtrack of the exact companion masters at chapter cuts."""

from itertools import accumulate
from pathlib import Path
import hashlib
import json
import re
import runpy
import shutil
import socket
import subprocess
import sys

import numpy as np
from PIL import Image, ImageDraw


HERE = Path(__file__).resolve().parent
JOB = "slate-companion-join-spectra01"
WORK_REL = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
RATE, HOP, FFT, HALF = 48000, 240, 1024, 120000
BASELINE = "5df0f17125e127d943c4478cc21d23439902a8e7e8ee696bf9008831fd484595"


def read(path):
    return json.loads(path.read_text())


def sha(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def write(path, data):
    with path.open("x") as handle:
        json.dump(data, handle, indent=2)
        handle.write("\n")


def stable(path):
    receipt = read(path)
    assert receipt["finished"] and receipt["admitted"] and receipt["exit"] == 0
    assert receipt["inputs_sha256"] == receipt["final_inputs_sha256"]
    assert not receipt.get("stop_reason") and not receipt.get("stopped_for_resource_limit")
    return receipt


def db(value):
    return round(float(20 * np.log10(max(value, 1e-12))), 2)


def panel(image, waveform, row, top):
    draw = ImageDraw.Draw(image)
    x, y, w, h = 58, top + 49, 1420, 148
    frames = np.lib.stride_tricks.sliding_window_view(waveform, FFT, axis=0)[::HOP]
    taper = np.hanning(FFT)
    bins = np.abs(np.fft.rfft(frames * taper, axis=-1)).max(axis=1) * (2 / taper.sum())
    power = np.clip((20 * np.log10(np.maximum(bins, 1e-8)) + 95) / 70, 0, 1)
    plane = power[:, :int(FFT / 3) + 1].T[::-1]
    rgb = np.stack((22 + 226*plane, 24 + 155*plane**1.4, 39 + 74*plane), axis=-1)
    image.paste(Image.fromarray(rgb.astype(np.uint8)).resize((w, h)), (x, y))
    mid = x + w//2
    draw.text((x, top + 9), f"{row['film']} {row['join']}   frame {row['frame']}   {row['seconds']:.3f} s", fill="white")
    draw.text((x, top + 29), row["word_context"], fill="#dcdde5")
    draw.text((4, y), "16k", fill="white")
    draw.text((22, y+h-11), "0", fill="white")
    trace_y = y + h + 48
    for index, channel in enumerate(waveform.T):
        signal = channel.reshape(-1, 240)
        lows = np.min(signal, axis=1)
        highs = np.max(signal, axis=1)
        offset = trace_y + (index*48 - 24)
        draw.line((x, offset, x+w, offset), fill="#41464a")
        draw.text((25, offset-6), "LR"[index], fill="white")
        for sample, (low, high) in enumerate(zip(lows, highs, strict=True)):
            xx = x + round((sample+.5)*w/len(lows))
            y0, y1 = [offset - round(float(np.clip(a, -.25, .25))*76) for a in (low, high)]
            draw.line((xx, y0, xx, y1), fill="#74cfce" if index==0 else "#86a1ff")
    draw.line((mid, y, mid, trace_y+44), fill="#ef575d", width=2)
    draw.text((x, trace_y+51), "-2.50 s", fill="white")
    draw.text((mid-14, trace_y+51), "cut", fill="#ff7a81")
    draw.text((x+w-65, trace_y+51), "+2.50 s", fill="white")


def discard(pipe, size):
    while size:
        data = pipe.read(min(size, 1 << 20))
        if not data:
            raise EOFError("decoded AAC ended before the chapter window")
        size -= len(data)


def audit_movie(tag, master, chapter_starts, contexts, ffmpeg, ffprobe):
    movie, frames = master
    args = [ffmpeg, "-nostdin", "-v", "error", "-xerror", "-i", movie,
            "-map", "0:a:0", "-c:a", "pcm_f32le", "-f", "f32le", "pipe:1"]
    clock = subprocess.check_output([ffprobe, "-v", "error", "-select_streams", "a:0",
              "-show_entries", "stream=start_pts,time_base,sample_rate,channels",
              "-of", "json", movie], timeout=60)
    stream, = json.loads(clock)["streams"]
    assert stream == {"sample_rate":"48000", "channels":2, "time_base":"1/48000", "start_pts":0}, stream
    position = 0
    with subprocess.Popen(args, stdout=subprocess.PIPE) as child:
        try:
            for index, (frame, context) in enumerate(zip(chapter_starts, contexts, strict=True), 1):
                sample = frame * 800
                cut_match = re.match(r"(\d+->\d+) at\s+([0-9.]+)s\s+(.*)", context)
                assert cut_match and abs(float(cut_match[2])-frame/60) < 0.006, context
                start = sample - HALF
                assert start >= position and sample + HALF <= frames * 800
                discard(child.stdout, 8*(start-position))
                take = child.stdout.read(16*HALF)
                assert len(take)==16*HALF, (tag,frame)
                pcm = np.frombuffer(take, dtype="<f4").reshape(-1,2)
                assert np.isfinite(pcm).all()
                sample_step = np.abs(pcm[HALF] - pcm[HALF-1])
                flank = pcm[HALF-12000:HALF+12000]
                max_step = np.max(np.abs(np.diff(flank,axis=0)),axis=0)
                row = {
                    "film":tag, "join":cut_match[1],"frame":frame,"seconds":frame/60,
                    "sample":sample,"word_context":cut_match[3],
                    "cut_sample_step_dbfs":[db(a) for a in sample_step],
                    "nearby_maximum_step_dbfs":[db(a) for a in max_step],
                    "center_20ms_peak_dbfs":[db(a) for a in np.max(np.abs(pcm[HALF-480:HALF+480]),axis=0)],
                    "decoded_window_sha256":hashlib.sha256(take).hexdigest(),
                }
                yield row, pcm
                position = sample+HALF
            while chunk := child.stdout.read(1<<20):
                assert len(chunk)%8==0
                position += len(chunk)//8
            assert child.wait(timeout=30)==0 and 0 <= position-frames*800 <= 1024
        except BaseException:
            child.terminate()
            raise


def prepare(staging):
    plan = read(staging / "inputs01.json")
    paths = {staging / "inputs01.json"}
    results = []
    for row in plan:
        files = {name:(staging/rel).resolve(strict=True) for name,rel in row["files"].items()}
        assert set(files)==set(row["sha256"])
        assert all(sha(path)==row["sha256"][name] for name,path in files.items())
        package, asr = read(files["manifest"]), read(files["binding"])
        stable(files["guard"])
        frames = package["frames"]
        assert asr["frames"]==frames and sum(n for _,n in asr["chapters"])==frames
        starts = [frame for _,frame,_ in package["chapter_starts"]]
        assert starts == [0,*list(accumulate(n for _,n in asr["chapters"]))[:-1]]
        assert [key for key,_,_ in package["chapter_starts"]]==[key for key,_ in asr["chapters"]]
        movie = files["movie"]
        assert sha(movie)==asr["master_sha256"]==package["assets"][movie.name]["sha256"]
        assert sha(files["joins"])==asr["join_report_sha256"]
        contexts = files["joins"].read_text().splitlines()
        assert len(contexts)==len(starts) and contexts[-1].startswith("cut ")
        results.append((row["film"],(movie,frames),starts[1:],contexts[:-1]))
        paths.update(files.values())
    return results, paths


def main():
    mode, staging = sys.argv[1], Path(sys.argv[2]).resolve()
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    toolkit = runpy.run_path(str(HERE/"queues-secondary-clock01.py"))
    repo, work, wrapper = [toolkit[name] for name in ("REPO", "WORK", "WRAPPER")]
    baseline = work/"allocation-baseline.json"
    assert sha(baseline)==BASELINE
    assert sha(wrapper)=="1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    decoder, probe = [Path(shutil.which(name)).resolve(strict=True) for name in ("ffmpeg","ffprobe")]
    films, paths = prepare(staging)
    assert [(tag,len(starts)) for tag,_,starts,_ in films]==[("conformal",10),("queues",9),("robotics",11)]
    job = work / JOB
    guard_file = job/"resource-receipt.json"
    if mode=="execute":
        guard = read(guard_file)
        assert guard["admitted"] and not guard.get("finished") and guard["phase"]=="asr"
        rows, sheets = [], []
        for film, master, starts, contexts in films:
            canvas = None
            for index, (row, pcm) in enumerate(audit_movie(film,master,starts,contexts,decoder,probe)):
                index_on_sheet = index%3
                name = f"{film}-{index//3+1:02d}.png"
                if index_on_sheet==0:
                    canvas = Image.new("RGB",(1536,324*min(len(starts)-index,3)),"#11141b")
                panel(canvas,pcm,row,324*index_on_sheet)
                rows.append(row | {"sheet":name})
                if index_on_sheet==2 or index+1==len(starts):
                    with (job/name).open("xb") as handle:
                        canvas.save(handle,format="PNG")
                    sheets.append({"file":name,"sha256":sha(job/name)})
        assert len(rows)==30
        write(job/"plan01.json",{"inputs_sha256":sha(staging/"inputs01.json"),"rows":rows,"sheets":sheets,
             "sample_rate":RATE,"half_window_samples":HALF,"fft_samples":FFT,"hop_samples":HOP,
             "scope":"Exact sample-clock stereo join and word context inspection. Hearing and perceptual sync remain open."})
        print(f"{len(rows)} windows on {len(sheets)} sheets")
        return

    assert mode=="launch" and not job.exists()
    layer = runpy.run_path(str(wrapper))
    guard = layer["load"]()
    assert guard.REPO==repo and guard.WORK==work and guard.TARGET==repo/"target"
    assert guard.ADMISSION_GIB["asr"]==12 and guard.CAPS["asr"]==1 and guard.STOP_FREE_GIB==8
    assert guard.TARGET_GROWTH_CAP_GIB==20
    guard.WORK_CAP_GIB=200
    paths.update((baseline,wrapper,layer["SOURCE"],HERE/"queues-secondary-clock01.py",
                  Path(__file__).resolve(),decoder,probe))
    manifest = work/"navier-production-20261120"/f"{JOB}-inputs.json"
    write(manifest,{"files":[{"path":str(path),"sha256":sha(path)} for path in sorted(paths)]})
    sys.argv=[__file__,"--phase","asr","--job",JOB,"--inputs",str(manifest),
              "--",sys.executable,"-I","-B",__file__,"execute",str(staging)]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
