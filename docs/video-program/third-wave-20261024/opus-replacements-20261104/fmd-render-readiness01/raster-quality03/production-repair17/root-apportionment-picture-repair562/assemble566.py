import hashlib,json,subprocess,sys,shutil
from pathlib import Path
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q=Path(__file__).resolve().parent
S=R/'narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89'
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
OUT=B/'apportionment-picture-assembly566/narration'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
for name in ['apportionment-picture-holds563','apportionment-picture-build564']:
    d=json.loads((B/name/'resource-receipt.json').read_text())
    assert d.get('finished') and d.get('exit')==0 and not d.get('stopped_for_resource_limit'),name
    assert d['inputs_sha256']==d['final_inputs_sha256'],name
script=R/'explainer_corpus/apportionment/script.tsv'
units=R/'explainer_corpus/apportionment/narration_units.json'
subprocess.run([sys.executable,'-I','-B',str(R/'tools/coherent_narration.py'),'assemble','--script',str(script),'--units',str(units),'--spans',str(Q/'spans566.json'),'--out',str(OUT),'--require-directions'],check=True,timeout=300)
old=S/'narration437'
wavs=sorted(old.glob('*.wav')); assert len(wavs)==93
assert {p.name for p in wavs}=={p.name for p in OUT.glob('*.wav')}
for p in wavs: assert sha(p)==sha(OUT/p.name),p.name
with (old/'phrase-timings.json').open('rb') as src,(OUT/'phrase-timings.json').open('xb') as dst: shutil.copyfileobj(src,dst)
assert sha(old/'phrase-timings.json')==sha(OUT/'phrase-timings.json')
assert json.loads((old/'report.json').read_text())['lines']==json.loads((OUT/'report.json').read_text())['lines']
subprocess.run([sys.executable,'-I','-B',str(R/'tools/coherent_narration.py'),'verify',str(OUT),'--script',str(script),'--units',str(units),'--require-directions'],check=True,timeout=300)
with (OUT.parent/'comparison566.json').open('x') as f:json.dump({'unchanged_wavs':93,'sentence_spans_identical':True,'phrase_timings_sha256':sha(OUT/'phrase-timings.json'),'listening_acceptance':False,'fine_sync_acceptance':False},f,indent=2)
