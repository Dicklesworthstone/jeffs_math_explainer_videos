"""Freeze actual reviewed inputs and run fifteen independent guarded final chapters serially. No publication."""
import hashlib,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert sha(P/'final614.py')=='894240840e44174fc190bc945f0a589d4a8109da2d1b7cafe5e22ec0189faa28'
 assert sha(P.parent/'root-apportionment-final-support613.py')=='dfa086950e9b771e03d0feb59f755348a9df5b4fd40519ba07ea7936df4133b0'
 rows={x['path']:x['sha256'] for x in json.loads((P/'preview569-inputs.json').read_text())['files']}
 for path,digest in rows.items():assert sha(path)==digest,path
 for name in ('apportionment-picture-preview569','apportionment-picture-review584'):
  path=B/name/'resource-receipt.json';d=json.loads(path.read_text())
  assert d.get('admitted') and d.get('finished') and d.get('exit')==0
  assert not any(d.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
  assert d['inputs_sha256']==d['final_inputs_sha256']
  rows[str(path)]=sha(path)
 review=P/'pictures-viewed610.json';assert sha(review)=='a76504654c821ab8bd7de38e0a0748d3dee9e68e2a55483fdd2de46916f58398'
 old=P.parent/'root-apportionment545-review-progress551.json';d=json.loads(old.read_text())
 assert len(d['viewed'])==15 and sum(map(len,d['viewed'].values()))==142 and d['selected_pictures_viewed']==1078
 registry=subprocess.check_output([str(B/'apportionment-picture-build564/apportionment-native564'),'chapters'],text=True,cwd=R,timeout=120).splitlines()
 assert len(registry)==15 and set(registry)==set(d['viewed'])
 for path in [review,old,P/'final614.py',P.parent/'root-apportionment-final-support613.py',Path(__file__).resolve(),Path(sys.executable).resolve()]:rows[str(path)]=sha(path)
 for ch in registry:
  assert not (B/('apportionment-final614-'+ch)).exists()
  assert not (P/('final614-'+ch+'-inputs.json')).exists()
 enc=None
 for ch in registry:
  for path,digest in rows.items():assert sha(path)==digest,path
  inputs=P/('final614-'+ch+'-inputs.json')
  with inputs.open('x') as f:json.dump({'files':[{'path':k,'sha256':v} for k,v in sorted(rows.items())]},f,indent=2)
  command=[sys.executable,'-I','-B',str(P.parent/'guard_owner80g_target20g.py'),'--phase','native','--job','apportionment-final614-'+ch,'--inputs',str(inputs),'--',sys.executable,'-I','-B',str(P/'final614.py'),ch]
  print('LAUNCH',ch,flush=True)
  result=subprocess.run(command,cwd=R);assert result.returncode==0,(ch,result.returncode)
  job=B/('apportionment-final614-'+ch);rec=json.loads((job/'resource-receipt.json').read_text())
  assert rec.get('finished') and rec.get('exit')==0 and rec['inputs_sha256']==rec['final_inputs_sha256']
  assert not any(rec.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
  row=json.loads((job/'final614-summary.json').read_text())['chapters'][ch]
  if enc is None:enc=row['encoder_settings']
  assert row['encoder_settings']==enc,(ch,'encoder differs from first final chapter')
  print('FINAL_TECHNICALLY_CHECKED',ch,row['frames'],flush=True)
 print('ALL15_FINAL_CHAPTERS_TECHNICALLY_CHECKED; actual final picture/master/audio/release checks remain',flush=True)
if __name__=='__main__':main()
