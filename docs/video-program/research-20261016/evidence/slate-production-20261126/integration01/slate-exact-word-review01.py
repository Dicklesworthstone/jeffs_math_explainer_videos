"""Check disputed wording in short, exact excerpts of the two joined mixes."""

from pathlib import Path
import json
import runpy
import shutil
import subprocess
import sys
import wave


HERE = Path(__file__).resolve().parent
JOB = "slate-exact-word-review01"
MODEL = Path("/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin").resolve(strict=True)
MODEL_SHA256 = "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69"


def main():
    mode, source=sys.argv[1],Path(sys.argv[2]).resolve()
    helpers=runpy.run_path(str(HERE/"slate-companion-join-spectra01.py"))
    host=runpy.run_path(str(HERE/"queues-secondary-clock01.py"))
    sha,read,stable,write=(helpers[name] for name in ("sha","read","stable","write"))
    repo,work,wrapper=(host[name] for name in ("REPO","WORK","WRAPPER"))
    baseline=work/"allocation-baseline.json"
    assert sha(baseline)==helpers["BASELINE"] and sha(MODEL)==MODEL_SHA256
    assert sha(wrapper)=="1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    fw=Path(shutil.which("fw")).resolve(strict=True)
    plan=read(source)
    deck=source.parent/"plan01.json"
    movies={row["film"]:row["sha256"]["movie"] for row in read(deck)["movies"]}
    bound={source,deck,MODEL,fw,Path(__file__).resolve(),HERE/"slate-companion-join-spectra01.py",
           HERE/"queues-secondary-clock01.py",baseline,wrapper}
    for item in plan:
        files={name:(source.parent/rel).resolve(strict=True) for name,rel in item["files"].items()}
        assert all(sha(path)==item["sha256"][name] for name,path in files.items())
        binding=read(files["binding"])
        stable(files["receipt"])
        assert binding["master_sha256"]==movies[item["film"]]
        assert binding["sample_rate"]==16000 and binding["derived_wav_sha256"]==sha(files["wave"])
        item["source_wave"]=files["wave"]
        item["samples"]=binding["samples"]
        bound.update(files.values())
    job=work/JOB
    if mode=="execute":
        live=read(job/"resource-receipt.json")
        assert live["admitted"] and live["phase"]=="asr" and not live.get("finished")
        outcomes=[]
        for item in plan:
            with wave.open(str(item["source_wave"]),"rb") as audio:
                assert (audio.getnchannels(),audio.getsampwidth(),audio.getframerate(),audio.getnframes())==(
                    1,2,16000,item["samples"])
                for sample in item["excerpts"]:
                    start=round(sample["start"]*16000)-1920
                    end=round(sample["end"]*16000)+1920
                    assert 0<=start<end<=item["samples"] and end-start<16000*30
                    audio.setpos(start)
                    pcm=audio.readframes(end-start)
                    assert len(pcm)==2*(end-start)
                    name=f"{item['film']}-{sample['cue']}"
                    outfile=job/f"{name}.wav"
                    with outfile.open("xb") as out, wave.open(out,"wb") as clip:
                        clip.setnchannels(1)
                        clip.setsampwidth(2)
                        clip.setframerate(16000)
                        clip.writeframes(pcm)
                    result=job/f"{name}.json"
                    with result.open("xb") as log:
                        subprocess.run([fw,"transcribe","--input",outfile,"--backend","whisper-cpp","--language","en",
                            "--model",MODEL,"--no-persist","--no-diarize","--json","--threads","4","--timeout","80"],
                            stdout=log,check=True,timeout=95)
                    recognized=read(result)
                    assert Path(recognized["input_path"])==outfile and recognized["result"]["segments"]
                    outcomes.append({"film":item["film"],"cue":sample["cue"],"master_sha256":movies[item["film"]],
                         "start_sample":start,"end_sample_exclusive":end,"audio_sha256":sha(outfile),"recognition_sha256":sha(result),
                         "source_wav_sha256":item["sha256"]["wave"],"expected":sample["expected"],
                         "recognized":" ".join(s["text"].strip() for s in recognized["result"]["segments"])})
        assert len(outcomes)==4
        write(job/"readback01.json",{"plan_sha256":sha(source),"model_sha256":MODEL_SHA256,"outcomes":outcomes,
              "scope":"A different context for automatic speech recognition. Human listening and perceptual sync remain open."})
        return

    assert mode=="launch" and not job.exists()
    carrier=runpy.run_path(str(wrapper))
    guard=carrier["load"]()
    assert guard.REPO==repo and guard.WORK==work and guard.TARGET==repo/"target"
    assert guard.ADMISSION_GIB["asr"]==12 and guard.CAPS["asr"]==1 and guard.STOP_FREE_GIB==8
    assert guard.TARGET_GROWTH_CAP_GIB==20
    guard.WORK_CAP_GIB=200
    bound.add(carrier["SOURCE"])
    manifest=work/"navier-production-20261120"/f"{JOB}-inputs.json"
    write(manifest,{"files":[{"path":str(p),"sha256":sha(p)} for p in sorted(bound)]})
    sys.argv=[__file__,"--phase","asr","--job",JOB,"--inputs",str(manifest),"--",
              sys.executable,"-I","-B",__file__,"execute",str(source)]
    raise SystemExit(guard.main())


if __name__=="__main__":
    main()
