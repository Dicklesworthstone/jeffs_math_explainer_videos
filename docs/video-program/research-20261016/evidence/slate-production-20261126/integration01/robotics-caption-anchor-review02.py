"""Verify the uncovered caption blocks using bounded excerpts of the exact exported mix."""

from pathlib import Path
import json
import runpy
import shutil
import socket
import subprocess
import sys
import wave


HERE = Path(__file__).resolve().parent
JOB = "robotics-caption-anchor-review02"
MODEL = Path("/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin").resolve(strict=True)
MODEL_SHA256 = "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69"


def main():
    mode, staged = sys.argv[1], Path(sys.argv[2]).resolve()
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    prior = runpy.run_path(str(HERE/"slate-companion-join-spectra01.py"))
    sha, read, write, stable = [prior[name] for name in ("sha","read","write","stable")]
    toolkit = runpy.run_path(str(HERE/"queues-secondary-clock01.py"))
    repo, work, wrapper = [toolkit[name] for name in ("REPO","WORK","WRAPPER")]
    baseline = work/"allocation-baseline.json"
    assert sha(baseline)==prior["BASELINE"] and sha(MODEL)==MODEL_SHA256
    assert sha(wrapper)=="1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    fw = Path(shutil.which("fw")).resolve(strict=True)
    original = work/"robotics-native-master-join-asr01"
    wav = original/"export-16k-mono.wav"
    asr_binding = original/"binding01.json"
    receipt = original/"resource-receipt.json"
    binding = read(asr_binding)
    stable(receipt)
    plan = read(staged)
    assert plan["master_sha256"]==binding["master_sha256"]
    assert plan["full_recognition_sha256"]==binding["segments_sha256"]==sha(original/"export-words.json")
    assert sha(wav)==binding["derived_wav_sha256"] and binding["sample_rate"]==16000
    windows=plan["windows"]
    indices=[row["index"] for cue in windows for row in cue["captions"]]
    assert len(set(indices))==len(indices)==plan["expected_missing_blocks"]==23 and len(windows)==11
    job = work/JOB
    if mode=="execute":
        current=read(job/"resource-receipt.json")
        assert current["admitted"] and current["phase"]=="asr" and not current.get("finished")
        positions, inputs=[], []
        with wave.open(str(wav),"rb") as src:
            assert (src.getnchannels(),src.getsampwidth(),src.getframerate())==(1,2,16000)
            assert src.getnframes()==binding["samples"]
            for window in windows:
                cue=window["cue"]
                start=round(window["start_seconds"]*16000)-1920
                end=round(window["end_seconds"]*16000)+1920
                assert 0<=start<end<=src.getnframes() and (end-start)/16000 < 40
                src.setpos(start)
                buffer=src.readframes(end-start)
                assert len(buffer)==2*(end-start)
                path=job/f"{cue}.wav"
                with path.open("xb") as f, wave.open(f,"wb") as out:
                    out.setparams(src.getparams())
                    out.writeframes(buffer)
                inputs.extend(("--input",path))
                positions.append({"cue":cue,"start_sample":start,"samples":end-start,"wav_sha256":sha(path)})
        output=job/"recognition01.jsonl"
        with output.open("xb") as log:
            subprocess.run([fw,"transcribe",*inputs,"--backend","whisper-cpp","--language","en",
                            "--model",MODEL,"--no-diarize","--no-persist","--json",
                            "--max-segment-length","1","--split-on-word","--threads","4","--timeout","120"],
                           stdout=log,check=True,timeout=1550)
        rows=[json.loads(line) for line in output.read_text().splitlines() if line.strip()]
        assert len(rows)==len(positions) and all(row["status"]=="ok" for row in rows)
        assert [Path(row["input"]).stem for row in rows]==[row["cue"] for row in positions]
        write(job/"binding01.json",{"master_sha256":plan["master_sha256"],"srt_sha256":plan["srt_sha256"],
             "model_sha256":MODEL_SHA256,"plan_sha256":sha(staged),"export_wav_sha256":sha(wav),
             "recognition_sha256":sha(output),"positions":positions,
             "scope":"Targeted automatic recognition from the exact encoded mix. Hearing and perceptual sync remain open."})
        return

    assert mode=="launch" and not job.exists()
    layer=runpy.run_path(str(wrapper))
    guard=layer["load"]()
    assert guard.REPO==repo and guard.WORK==work and guard.TARGET==repo/"target"
    assert guard.ADMISSION_GIB["asr"]==12 and guard.CAPS["asr"]==1 and guard.STOP_FREE_GIB==8
    assert guard.TARGET_GROWTH_CAP_GIB==20
    guard.WORK_CAP_GIB=200
    bound={staged,MODEL,fw,Path(__file__).resolve(),HERE/"slate-companion-join-spectra01.py",
           HERE/"queues-secondary-clock01.py",wrapper,layer["SOURCE"],baseline,
           wav,asr_binding,receipt,original/"export-words.json"}
    manifest=work/"navier-production-20261120"/f"{JOB}-inputs.json"
    write(manifest,{"files":[{"path":str(path),"sha256":sha(path)} for path in sorted(bound)]})
    sys.argv=[__file__,"--phase","asr","--job",JOB,"--inputs",str(manifest),"--",
              sys.executable,"-I","-B",__file__,"execute",str(staged)]
    raise SystemExit(guard.main())


if __name__=="__main__":
    main()
