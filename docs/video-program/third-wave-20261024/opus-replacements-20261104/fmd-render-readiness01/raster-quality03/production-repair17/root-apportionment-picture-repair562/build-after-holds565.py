import hashlib,json,subprocess,time
from pathlib import Path
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
Q=Path(__file__).resolve().parent
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
p=B/'apportionment-picture-holds563/resource-receipt.json'
deadline=time.monotonic()+3600
while True:
    try: d=json.loads(p.read_text())
    except (FileNotFoundError,json.JSONDecodeError): d={}
    if d.get('finished'): break
    if time.monotonic()>deadline: raise SystemExit('validation wait expired; no build launched')
    time.sleep(5)
assert d.get('exit')==0,d
assert d['inputs_sha256']==d['final_inputs_sha256'],d
for name in ['validate563-inputs.json','build564-inputs.json']:
    for row in json.loads((Q/name).read_text())['files']:
        assert hashlib.sha256(Path(row['path']).read_bytes()).hexdigest()==row['sha256'],row['path']
assert not (B/'apportionment-picture-build564').exists()
cmd=['python3','-I','-B',str(Q.parent/'guard_owner80g_target20g.py'),'--phase','native','--job','apportionment-picture-build564','--inputs',str(Q/'build564-inputs.json'),'--','sh',str(Q/'build564.sh')]
print('Validation succeeded with stable inputs; launching guarded build564',flush=True)
raise SystemExit(subprocess.call(cmd,cwd=R))
