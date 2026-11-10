"""Wait exact complete timeline, then serial guarded remaining chapters, no retries."""
from pathlib import Path
import json,hashlib,subprocess,time
D=Path('/data/projects/jmev-stroke-probe-20261106');Q=D/'qualification01/production-repair17';T=D/'jobs/csd-apportionment-timeline666';B=D/'jobs/csd-apportionment-build656'
PINS={'apportionment-csd-final672.py': '54262bf86dda15e86d08c23d90f19d58d7d0c0e5c61c466720072646be544f24', 'apportionment-csd-support671.py': '244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a'}
def sha(p):
 with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(Path(p).read_text())
def success(p):
 d=read(p);assert d.get('finished')and d.get('admitted')and d.get('exit')==0 and d['inputs_sha256']==d['final_inputs_sha256'],str(p)
 assert not any(d.get(k)for k in ['error','resource_stop','stopped_for_resource_limit']),str(p)
for _ in range(2160):
 try:d=read(T/'resource-receipt.json')
 except (FileNotFoundError,json.JSONDecodeError):d={}
 if d.get('finished'):success(T/'resource-receipt.json');break
 time.sleep(20)
else:raise RuntimeError('Timeline wait deadline; no finals started')
comp=read(T/'comparison666.json');qual=read(B/'qualification.json')
assert comp['full_normalized_tables_equal']and comp['total_frames']==100886 and comp['score_qa_passed']==15
chapters=qual['chapters'];assert len(chapters)==15 and set(chapters)==set(comp['chapters'])
base={Path(x['path']):x['sha256']for x in read(Q/'csd-apportionment-timeline666-inputs.json')['files']}
for n,h in PINS.items():assert sha(Q/n)==h;base[Q/n]=h
for p in [Path(__file__).resolve(),T/'resource-receipt.json',T/'comparison666.json']+[p for p in (T/'timeline').glob('*.tsv')]:base[p]=sha(p)
for c in chapters[2:]:
 name='csd-apportionment-final672-'+c;job=D/'jobs'/name;inputs=Q/(name+'-inputs.json')
 assert not job.exists()and not inputs.exists(),('No duplicate',name)
 for p,h in base.items():assert sha(p)==h,p
 with inputs.open('x')as f:json.dump(dict(files=[dict(path=str(p),sha256=h)for p,h in sorted(base.items())]),f,indent=2)
 print('LAUNCH',name,flush=True)
 code=subprocess.call(['/usr/bin/python3','-I','-B',str(D/'guard_csd02.py'),'--phase','native','--job',name,'--inputs',str(inputs),'--','/usr/bin/python3','-I','-B',str(Q/'apportionment-csd-final672.py'),c])
 assert code==0,(name,code);success(job/'resource-receipt.json')
 summary=read(job/'final672-summary.json');assert summary['chapter']==c and summary['result']['frames']==comp['chapters'][c]['frames']
 print('FINISHED',name,flush=True)
