"""Exact313 metadata on native553; reuse reviewed540 path-only rebinding. No synthesis/render."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess

R=Path.home()/'scratch/jmev-complex-native-20261109'
Q=R/'qualification'
SOURCE=R/'source552'
W=SOURCE/'workspace'
TOPIC=W/'explainer_corpus/complex_analysis_visual'
JOB=R/'jobs/mini-boundary-delivery557'
META=R/'inputs557'
N313=META/'jobs/csd-review-repair-rebind313/narration'
MAPREL='explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-review313.json'

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def read(p):return json.loads(Path(p).read_text())

def success(p):
    d=read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit')==0,p
    assert d['inputs_sha256']==d['final_inputs_sha256'] and not any(d.get(k) for k in ['resource_stop','error','stopped_for_resource_limit']),p
    return d

def snapshot(root):
    out={}
    for p in root.rglob('*'):
        assert not p.is_symlink(),p
        if p.is_file():out[p.relative_to(root).as_posix()]=sha(p)
    return out

def main():
    helper=Q/'root-mini-delivery540.py'
    assert sha(helper)=='23be4c05ef843e2cd40d07eb1de09205c2f5054e3370f307c2af577db021cabb'
    spec=importlib.util.spec_from_file_location('delivery540',helper)
    h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    h.guard_binding(JOB)
    success(R/'jobs/native-boundary553/resource-receipt.json')
    qual=R/'jobs/native-boundary553/qualification.json'
    assert sha(qual)=='aee4c6fe288ad2842b19f2b88a8f5b322c0b7bdd0246322872f4e89c02c3ac1c'
    assert sha(R/'jobs/native-boundary553/complex_analysis_visual-553')==read(qual)['binary_sha256']
    for row in read(META/'inventory.json'):
        assert sha(META/row['path'])==row['sha256'],row['path']
    success(META/'jobs/csd-review-repair-rebind313/resource-receipt.json')
    summary=read(META/'jobs/csd-review-repair-rebind313/rebind-summary.json')
    assert summary['wav_files_equal_rebind269']==105 and summary['report_sentence_rows_unchanged']
    before=snapshot(SOURCE)
    bound=read(Q/'root-boundary552-source-manifest.json')['files']
    assert before==bound,'native553 source changed'
    mapfile=META/'work/review-repair313/workspace'/MAPREL
    mapping=read(mapfile)
    assert sha(mapfile)==summary['map_sha256']
    oldmap=read(R/'inputs523/topic269/narration-direction-map-frederick-review269.json')
    assert mapping['helpers']==oldmap['helpers']
    moved={}
    for old,new in zip(oldmap['visual_holds'],mapping['visual_holds'],strict=True):
        if old==new:continue
        assert old['cue'] in ['09_4','09_5','09_8'] and new['line']==old['line']+4
        assert {k:v for k,v in old.items() if k not in ['line','sha256']}=={k:v for k,v in new.items() if k not in ['line','sha256']}
        assert new['sha256']==sha(W/new['path'])
        moved[old['cue']]=[old['line'],new['line']]
    assert set(moved)=={'09_4','09_5','09_8'}
    replica=JOB/'original313-replica';replica.mkdir()
    original=R/'inputs523/rebind269-narration'
    for p in original.iterdir():
        origin=N313/p.name if p.name in ['report.json','assembly.json','external-units.json'] else p
        with (replica/p.name).open('xb') as f:f.write(origin.read_bytes())
        assert sha(replica/p.name)==sha(origin)
    assert read(replica/'report.json')['lines']==read(original/'report.json')['lines']
    for key in ['outputs']:
        assert read(replica/'assembly.json')[key]==read(original/'assembly.json')[key]
    assert read(replica/'assembly.json')['manifest']['units']==read(original/'assembly.json')['manifest']['units']
    assert len(list(replica.glob('*.wav')))==105
    for p in replica.glob('*.wav'):assert sha(p)==sha(original/p.name)
    plan=TOPIC/'narration_units.json';backup=TOPIC/'narration_units.before-mini557.json'
    newplan=replica/'external-units.json'
    assert read(newplan)['direction_contract']['map']=={'path':MAPREL,'sha256':sha(mapfile)}
    with backup.open('xb') as f:f.write(plan.read_bytes())
    with (W/MAPREL).open('xb') as f:f.write(mapfile.read_bytes())
    plan.write_bytes(newplan.read_bytes())
    expected=dict(before)
    expected[plan.relative_to(SOURCE).as_posix()]=sha(newplan)
    expected[backup.relative_to(SOURCE).as_posix()]=before[plan.relative_to(SOURCE).as_posix()]
    expected[(W/MAPREL).relative_to(SOURCE).as_posix()]=sha(mapfile)
    assert snapshot(SOURCE)==expected
    h.REPLICA=replica
    oldlocal=h.local
    def local(path):
        prefix=h.D+'work/review-repair313/workspace/'
        if path.startswith(prefix):
            rest=path[len(prefix):];assert '..' not in rest.split('/')
            return str(W/rest)
        return oldlocal(path)
    h.local=local
    narration,receipt,changes=h.rebind(JOB)
    for p in narration.glob('*.wav'):assert sha(p)==sha(original/p.name)
    results={}
    for name,chapters,wanted in [('all',h.CHAPTERS,105),('boundary',['09_boundary'],9)]:
        cmd=[shutil.which('python3'),'-I','-B',str(W/'tools/narration_delivery.py'),'--topic',str(TOPIC),
             '--narration',str(narration),'--consumer','render','--gain-db','0']
        for ch in chapters:cmd+=['--chapter-key',ch]
        with (JOB/('delivery-'+name+'.stdout')).open('xb') as out,(JOB/('delivery-'+name+'.stderr')).open('xb') as err:
            subprocess.run(cmd,cwd=R/'repo',stdout=out,stderr=err,check=True,timeout=1800)
        result=read(JOB/('delivery-'+name+'.stdout'))
        assert result['verified_cues']==105 and result['selected_cues']==wanted
        assert result['plan_mode']=='external-ui' and result['listening_acceptance'] is False
        results[name]=result
    assert snapshot(SOURCE)==expected
    with (JOB/'delivery557.json').open('x') as f:
        json.dump(dict(wavs_byte_identical=105,changed_path_fields=changes,moved_holds=moved,
                       delivery=results,source_after=expected,
                       scope='Source and sample reconstruction only; timeline, pictures, listening and fine sync remain open.'),f,indent=2)

if __name__=='__main__':main()
