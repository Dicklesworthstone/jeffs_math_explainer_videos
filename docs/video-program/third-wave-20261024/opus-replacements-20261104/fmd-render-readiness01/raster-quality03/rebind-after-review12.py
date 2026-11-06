import json,hashlib,subprocess,shutil
from pathlib import Path
R=Path('/data/projects/jmev-fmd-render-qualification-20261106-1111');T=R/'work/raster-quality-fmd04/workspace';J=R/'jobs/fmd-raster-rebind12';OLD=R/'jobs/fmd-picture-narration01/narration';N=J/'narration'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
r=read(J/'resource-receipt.json');assert r['admitted'] and 'exit' not in r
failed=read(R/'jobs/fmd-raster-rebind11/resource-receipt.json');assert failed['exit']==1 and failed['inputs_sha256']==failed['final_inputs_sha256'] and not failed.get('resource_stop')
review=read(R/'qualification01/raster-quality03/native-detail11-reviewed.json');q=read(R/'jobs/fmd-raster-quality-build05/qualification.json')
assert review['decision']=='pass_native_detail_for_pipeline' and review['binary_sha256']==q['binary_sha256']==sha(R/'work/bin/production09i')
src=R/'jobs/fmd-raster-rebind11/narration';shutil.copytree(src,N)
changed={p.name for p in OLD.iterdir() if sha(p)!=sha(N/p.name)};assert changed=={'external-units.json','report.json','assembly.json'}
assert len(list(N.glob('*.wav')))==105
new=read(N/'assembly.json');old=read(OLD/'assembly.json')
assert new['outputs']==old['outputs'] and new['manifest']['units']==old['manifest']['units']
assert read(N/'report.json')['lines']==read(OLD/'report.json')['lines']
C=T/'explainer_corpus/complex_analysis_visual';map=read(C/'docs/narration-direction-map-frederick-stage09.json')
assert len(map['visual_holds'])==23 and len(map['helpers'])==5
for row in map['visual_holds']+map['helpers']:
 f=T/row['path'];assert sha(f)==row['sha256'];assert f.read_text().splitlines()[row['line']-1].strip()==row['marker']
assert read(C/'narration_units.json')['direction_contract']==read(N/'external-units.json')['direction_contract']
commands=[('coherent',['/usr/bin/python3','-I','-B',str(T/'tools/coherent_narration.py'),'verify',str(N),'--script',str(C/'script.tsv'),'--units',str(N/'external-units.json'),'--require-directions'])]
args=['/usr/bin/python3','-I','-B',str(T/'tools/narration_delivery.py'),'--topic',str(C),'--narration',str(N),'--consumer','render','--gain-db','0']
for c in q['chapters']:args+=['--chapter-key',c]
commands.append(('delivery',args))
for name,cmd in commands:
 r=subprocess.run(cmd,cwd=R/'production-inputs01/production-repo07',capture_output=True,timeout=900)
 for ext,b in [('stdout',r.stdout),('stderr',r.stderr)]:
  with (J/(name+'.'+ext)).open('xb') as f:f.write(b)
 assert r.returncode==0,(name,r.stderr.decode()[-2000:])
d=read(J/'delivery.stdout');assert d['verified_cues']==d['selected_cues']==105
with (J/'rebind-summary.json').open('x') as f:json.dump({'wav_files_byte_identical':105,'holds':23,'helpers':5,'native_delivery':d,'changed_metadata_files':sorted(changed),'scope':'Recovered verification with correct retained recording cwd. No audio or timings changed; listening/fine-sync/independent AV remain open.'},f,indent=2)
