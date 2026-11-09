"""Freeze actual reviewed inputs and run fifteen independent guarded final chapters serially. No publication."""
import hashlib,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 assert sha(P/'final632.py')=='fae320302725ac5232c42145eb9ac9b6e6bdae6ac8c17c2ca4960a3bc9a03a0e'
 assert sha(P.parent/'root-apportionment-final-support630.py')=='8e82072531189a936442e77e85b2c5a24f66bf10ca650bce0c1b97eecdae1388'
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
 for path in [review,old,P/'final632.py',P.parent/'root-apportionment-final-support630.py',Path(__file__).resolve(),Path(sys.executable).resolve()]:rows[str(path)]=sha(path)
 for ch in registry:
  assert not (B/('apportionment-final632-'+ch)).exists()
  assert not (P/('final632-'+ch+'-inputs.json')).exists()
 # Preserve and bind the completed first render and all failed-check evidence.
 prior=B/'apportionment-final614-01_one_more_seat'
 for path in prior.rglob('*'):
  assert not path.is_symlink()
  if path.is_file():rows[str(path)]=sha(path)
 old_inputs=P/'final614-01_one_more_seat-inputs.json';rows[str(old_inputs)]=sha(old_inputs)
 for item in json.loads(old_inputs.read_text())['files']:
  assert sha(item['path'])==item['sha256']
  assert item['path'] not in rows or rows[item['path']]==item['sha256']
  rows[item['path']]=item['sha256']
 enc=None
 for ch in registry:
  for path,digest in rows.items():assert sha(path)==digest,path
  inputs=P/('final632-'+ch+'-inputs.json')
  with inputs.open('x') as f:json.dump({'files':[{'path':k,'sha256':v} for k,v in sorted(rows.items())]},f,indent=2)
  command=[sys.executable,'-I','-B',str(P.parent/'guard_owner80g_target20g.py'),'--phase','native','--job','apportionment-final632-'+ch,'--inputs',str(inputs),'--',sys.executable,'-I','-B',str(P/'final632.py'),ch]
  print('LAUNCH',ch,flush=True)
  result=subprocess.run(command,cwd=R);assert result.returncode==0,(ch,result.returncode)
  job=B/('apportionment-final632-'+ch);rec=json.loads((job/'resource-receipt.json').read_text())
  assert rec.get('finished') and rec.get('exit')==0 and rec['inputs_sha256']==rec['final_inputs_sha256']
  assert not any(rec.get(k) for k in ('error','resource_stop','stopped_for_resource_limit'))
  row=json.loads((job/'final632-summary.json').read_text())['chapters'][ch]
  if enc is None:enc=row['encoder_settings']
  assert row['encoder_settings']==enc,(ch,'encoder differs from first final chapter')
  print('FINAL_TECHNICALLY_CHECKED',ch,row['frames'],flush=True)
 print('ALL15_FINAL_CHAPTERS_TECHNICALLY_CHECKED; actual final picture/master/audio/release checks remain',flush=True)
if __name__=='__main__':main()
