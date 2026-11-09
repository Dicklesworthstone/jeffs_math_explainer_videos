"""Guarded native master for all fifteen final614 chapters; no release or perceptual acceptance."""
import hashlib,importlib.util,json,os,signal,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
JOB=B/'apportionment-master619'
INPUTS=P/'master619-inputs.json'
BINARY=B/'apportionment-picture-build564/apportionment-native564'
NARR=B/'apportionment-picture-assembly566/narration'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 support=P.parent/'root-apportionment-final-support613.py'
 assert sha(support)=='dfa086950e9b771e03d0feb59f755348a9df5b4fd40519ba07ea7936df4133b0'
 spec=importlib.util.spec_from_file_location('support613_master',support);h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
 h.own_group();h.STATE['job']=JOB
 for sig in h.STOP_SIGNALS:signal.signal(sig,h.on_stop)
 receipt=json.loads((JOB/'resource-receipt.json').read_text())
 assert receipt.get('admitted') and receipt['policy']==h.POLICY and not receipt.get('finished')
 assert str(Path(__file__).resolve()) in receipt['command']
 assert sorted(x.name for x in JOB.iterdir())==['command.log','resource-receipt.json']
 bound={x['path']:x['sha256'] for x in json.loads(INPUTS.read_text())['files']}
 assert sha(INPUTS)==receipt['inputs_sha256']
 for path,digest in bound.items():assert sha(path)==digest,path
 assert sha(BINARY)=='87a34bb262419c8a24b1c75b80d2eb97046cbc1f72fd5b2be11043ba5982a107'
 registry=subprocess.check_output([str(BINARY),'chapters'],text=True,cwd=R,timeout=120).splitlines()
 assert len(registry)==15 and len(set(registry))==15
 records={};encoder=None;total=0
 for ch in registry:
  source=B/('apportionment-final614-'+ch);h.success(source/'resource-receipt.json')
  summary_path=source/'final614-summary.json';assert str(summary_path) in bound
  summary=json.loads(summary_path.read_text());assert set(summary['chapters'])=={ch}
  row=summary['chapters'][ch];assert row['folder']==str((source/'render').relative_to(R))
  assert row['frames']>0;total+=row['frames']
  if encoder is None:encoder=row['encoder_settings']
  assert row['encoder_settings']==encoder
  for rel,digest in row['files'].items():
   assert not Path(rel).is_absolute() and '..' not in Path(rel).parts
   f=source/'render'/rel;assert not f.is_symlink() and bound.get(str(f))==digest and sha(f)==digest
  for ext in ('timeline.tsv','cues.tsv','layout.tsv'):
   assert (source/'render'/(ch+'.'+ext)).read_bytes()==(B/'apportionment-picture-timeline567/timeline'/(ch+'.'+ext)).read_bytes()
  records[ch]=row
 assert total==100886
 out=JOB/'final';out.mkdir()
 links=[]
 for ch,row in records.items():
  source=B/('apportionment-final614-'+ch)/'render'
  captures=[k for k in row['files'] if k.startswith('soundtrack/')];assert len(captures)==1
  for rel,digest in row['files'].items():
   dest=out/('soundtrack/'+ch+'/soundtrack_0.wav' if rel==captures[0] else rel)
   dest.parent.mkdir(parents=True,exist_ok=True);assert not dest.exists()
   os.link(source/rel,dest)
   assert sha(dest)==digest
   links.append({'source':str(source/rel),'destination':str(dest),'sha256':digest})
 with (JOB/'materialized619.json').open('x') as f:json.dump({'links':links,'chapters':records,'frames':total},f,indent=2)
 for mode in ('master','captions'):
  command=[str(BINARY),mode]
  if mode=='captions':command.append('all')
  command+=['--narration',str(NARR),'--out',str(out)]
  code,tracking=h.run_owned(mode,command,14400);assert code==0,(mode,code)
  print('NATIVE_COMMAND_COMPLETED',mode,tracking,flush=True)
 for path,digest in bound.items():assert sha(path)==digest,path
 with (JOB/'master619-summary.json').open('x') as f:
  json.dump({'frames':total,'chapters':registry,'movie':str(out/'apportionment_master.mp4'),'movie_sha256':sha(out/'apportionment_master.mp4'),'srt_sha256':sha(out/'apportionment.en.srt'),'vtt_sha256':sha(out/'apportionment.en.vtt'),'scope':'Native master replay/frame grid and caption command passed; separate final decode/caption/loudness/picture/listening/fine-sync/independentAV checks remain.'},f,indent=2)
if __name__=='__main__':main()
