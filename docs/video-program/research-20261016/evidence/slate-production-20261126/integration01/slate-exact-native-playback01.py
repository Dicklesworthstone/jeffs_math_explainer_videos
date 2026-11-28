"""Run the known AVPlayer continuity probe on the exact local slate masters."""

from itertools import accumulate
from pathlib import Path
import json
import runpy
import shutil
import socket
import subprocess
import sys


HERE = Path(__file__).resolve().parent
JOB = "slate-exact-native-playback01"
XCRUN = Path("/usr/bin/xcrun")
KEEP_AWAKE = Path("/usr/bin/caffeinate")
CHAPTERS_AND_FRAMES = [
    ("anytime",14,61685), ("conformal",11,83525), ("queues",10,45039), ("robotics",12,128487)
]


def main():
    mode, source = sys.argv[1], Path(sys.argv[2]).resolve()
    assert socket.gethostname().lower().split(".")[0]=="mac-mini-max"
    scout = runpy.run_path(str(HERE/"slate-companion-join-spectra01.py"))
    toolkit = runpy.run_path(str(HERE/"queues-secondary-clock01.py"))
    sha, read, stable, write = (scout[name] for name in ("sha","read","stable","write"))
    repo, work, wrapper = (toolkit[name] for name in ("REPO","WORK","WRAPPER"))
    baseline = work/"allocation-baseline.json"
    assert sha(baseline)==scout["BASELINE"]
    assert sha(wrapper)=="1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    job = work/JOB
    deck = read(source)
    swift = Path(subprocess.check_output([XCRUN,"--find","swift"],text=True,timeout=25).strip()).resolve(strict=True)
    ffprobe = Path(shutil.which("ffprobe")).resolve(strict=True)
    base = source.parent
    probe=(base/deck["swift_source"]).resolve(strict=True)
    assert sha(probe)==deck["swift_source_sha256"]
    bound={source,probe,swift,XCRUN,KEEP_AWAKE,ffprobe,
           Path(__file__).resolve(),HERE/"slate-companion-join-spectra01.py",HERE/"queues-secondary-clock01.py",
           wrapper,baseline,Path(sys.executable).resolve()}
    clips=[]
    for plan, (film, chapters, frames) in zip(deck["movies"],CHAPTERS_AND_FRAMES,strict=True):
        assert plan["film"]==film
        files={name:(base/rel).resolve(strict=True) for name,rel in plan["paths"].items()}
        hashes=plan["sha256"]
        assert all(sha(path)==hashes[name] for name,path in files.items())
        manifest=read(files["manifest"])
        receipt=stable(files["guard"])
        movie=files["movie"]
        assert receipt["phase"]=="native" and sha(files["guard"])==manifest["master_guard_sha256"]
        assert sha(files["report"])==manifest["master_receipt_sha256"]
        technical=read(files["report"])
        assert technical["full_av_decode"] and technical["media"]["frames"]==manifest["frames"]==frames
        assert manifest["assets"][movie.name]["sha256"]==hashes["movie"]
        assert manifest["assets"]["captions.en.srt"]["sha256"]==hashes["srt"]
        starts=[frame for _,frame,_ in manifest["chapter_starts"]]
        counts=technical["chapters"]
        if isinstance(counts,dict):
            counts=list(counts.items())
        assert len(counts)==len(starts)==chapters
        assert [key for key,_ in counts]==[key for key,_,_ in manifest["chapter_starts"]]
        assert starts==[0,*list(accumulate(n for _,n in counts))[:-1]]
        assert sum(n for _,n in counts)==frames
        v,=json.loads(subprocess.check_output([
            ffprobe,"-v","error","-select_streams","v:0","-show_entries",
            "stream=width,height,r_frame_rate,nb_frames,start_time,duration","-of","json",movie,
        ],timeout=30))["streams"]
        assert (v["width"],v["height"],v["r_frame_rate"],int(v["nb_frames"]))==(3840,2160,"60/1",frames)
        assert float(v["start_time"])==0 and abs(float(v["duration"])-frames/60)<1e-5
        bound.update(files.values())
        clips.append((film,movie,hashes["movie"],frames,starts))
    assert len(clips)==4

    if mode=="execute":
        live=read(job/"resource-receipt.json")
        assert live["admitted"] and live["phase"]=="preview" and not live.get("finished")
        results=[]
        for film,movie,digest,frames,starts in clips:
            print(f"Starting {film} at normal speed, {frames} frames.",flush=True)
            path=job/f"{film}-player01.log"
            with path.open("x") as handle:
                subprocess.run([KEEP_AWAKE,"-i",XCRUN,"swift",probe,movie,str(frames),"60",*[str(f) for f in starts]],
                               stdout=handle,stderr=subprocess.STDOUT,check=True,timeout=frames//60+150)
            result,=[json.loads(line) for line in path.read_text().splitlines() if line.startswith("{")]
            assert result["passed"] and result["ended"] and result["sound_output_muted"] and result["error"] is None
            assert len(result["chapter_sample_counts"])==len(starts) and sha(movie)==digest
            record={"film":film,"movie_sha256":digest,"frames":frames,"playback":result,"log_sha256":sha(path),
                    "scope":"Normal-speed AVPlayer continuity with sampled 4K picture buffers and muted audio."}
            write(job/f"{film}-playback01.json",record)
            results.append(record)
            print(f"{film} reached the end with {result['sampled_4k_pictures']} UHD picture samples.",flush=True)
        write(job/"playback-result01.json",{"movies":results,"plan_sha256":sha(source),
              "scope":"Native playback checks. Continuous listening, perceptual fine sync and independent audiovisual acceptance remain open."})
        return

    assert mode=="launch" and not job.exists()
    layer=runpy.run_path(str(wrapper))
    guard=layer["load"]()
    assert guard.REPO==repo and guard.WORK==work and guard.TARGET==repo/"target"
    assert guard.ADMISSION_GIB["preview"]==20 and guard.CAPS["preview"]==4 and guard.STOP_FREE_GIB==8
    assert guard.TARGET_GROWTH_CAP_GIB==20
    guard.WORK_CAP_GIB=200
    bound.add(layer["SOURCE"])
    manifest=work/"navier-production-20261120"/f"{JOB}-inputs.json"
    write(manifest,{"files":[{"path":str(p),"sha256":sha(p)} for p in sorted(bound)]})
    sys.argv=[__file__,"--phase","preview","--job",JOB,"--inputs",str(manifest),"--",
              sys.executable,"-I","-B",__file__,"execute",str(source)]
    raise SystemExit(guard.main())


if __name__=="__main__":
    main()
