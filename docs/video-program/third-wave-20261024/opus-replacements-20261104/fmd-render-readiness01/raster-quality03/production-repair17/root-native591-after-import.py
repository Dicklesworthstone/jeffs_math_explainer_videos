"""After actual576 success, freeze and launch581 once. No master or publication."""
import hashlib,importlib.util,json,pathlib,subprocess,time,os
D=pathlib.Path('/data/projects/jmev-stroke-probe-20261106')
Q=D/'qualification01/production-repair17'
IMPORT='csd-mini-import576-10-a1'
JOB='csd-native-stage581-10-a1'
INPUT_SHA='036fdc7fea3f9ce459fd8e5ffd8ea2a8fba96304b55a9a034ec647f7ac406f93'
HELPER_SHA='822047a7f57877378140bb7c81ffd5d89ebd99d47310d17f7d341db6a3b59806'
def read(p):return json.loads(p.read_text())
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 source=D/'jobs'/IMPORT
 receipt=source/'resource-receipt.json'
 deadline=time.monotonic()+86400
 while not receipt.exists() or not read(receipt).get('finished'):
  assert time.monotonic()<deadline,'576 wait deadline; no stage launched'
  time.sleep(30)
 helper=Q/'root-native581-materialize.py'
 assert sha(helper)==HELPER_SHA
 spec=importlib.util.spec_from_file_location('stage581',helper);stage=importlib.util.module_from_spec(spec);spec.loader.exec_module(stage)
 transport=stage.load('root-mini576-transport.py');transport.terminal(read(receipt))
 inputs=Q/(IMPORT+'-inputs.json')
 assert sha(inputs)==INPUT_SHA==read(receipt)['inputs_sha256']
 files={pathlib.Path(__file__).resolve(),helper,receipt,inputs,source/'import-result.json'}
 for row in read(inputs)['files']:
  p=pathlib.Path(row['path']);assert sha(p)==row['sha256'];files.add(p)
 record=read(source/'import-result.json');manifest=pathlib.Path(record['manifest'])
 assert sha(manifest)==record['manifest_sha256'];files.add(manifest)
 physical={r['path']:pathlib.Path(r['physical']) for r in record['files']}
 transport.closure(read(manifest),physical);files.update(physical.values())
 for name in stage.PINS: files.add(Q/name)
 master=stage.load('combined-master447.py')
 origin=stage.MAC/'jobs/mini-final555/render'
 for line in physical[str(origin/'10_rigidity.cuelog.tsv')].read_text().splitlines():
  if not line.strip() or line.startswith('#'):continue
  row=line.split('\t');old=pathlib.Path(row[0])
  if old.parent==stage.MAC/'jobs/mini-delivery540/narration':target=master.NARRATION/old.name
  else:
   assert old==stage.MAC/'inputs523/score21/10_rigidity.music.f32.wav';target=master.STEMS/old.name
  assert sha(target)==row[1]==sha(physical[str(old)]);files.add(target)
 files.add(master.SCORE/'score21-summary.json')
 for ext in ('timeline.tsv','cues.tsv','cuelog.tsv'):files.add(master.TIMELINES/('10_rigidity.'+ext))
 out=Q/(JOB+'-inputs.json')
 assert not os.path.lexists(D/'jobs'/JOB) and not os.path.lexists(out),'existing attempt; inspect, never retry'
 rows=[dict(path=str(p),sha256=sha(p)) for p in sorted(files)]
 with out.open('x') as f:json.dump(dict(files=rows),f,indent=2)
 command=['/usr/bin/python3','-I','-B',str(D/'guard_csd02.py'),'--phase','native','--job',JOB,'--inputs',str(out),'--','/usr/bin/python3','-I','-B',str(helper),'--import-job',IMPORT,'--job',JOB,'--inputs',str(out)]
 print(json.dumps(dict(command=command,inputs_sha256=sha(out),files=len(rows))),flush=True)
 result=subprocess.run(command,cwd=D)
 assert result.returncode==0,('581 failed; preserve attempt',result.returncode)
 transport.terminal(read(D/'jobs'/JOB/'resource-receipt.json'))
 print('581 completed; no master or audiovisual acceptance',flush=True)
if __name__=='__main__':main()
