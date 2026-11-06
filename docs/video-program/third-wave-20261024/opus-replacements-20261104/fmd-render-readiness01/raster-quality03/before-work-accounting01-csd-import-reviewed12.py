"""Guarded import of exact FMD review12 transfer; does not launch chapters."""
import hashlib, importlib.util, json, shutil, sys
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106')
Q=R/'qualification01/raster-quality12'
J=R/'jobs/csd-raster-review-import12'
def read(p):return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def run():
    assert sys.argv[1:]==['--run']
    spec=importlib.util.spec_from_file_location('worker',Q/'csd-final-chapter12.py')
    w=importlib.util.module_from_spec(spec);spec.loader.exec_module(w)
    receipt=read(J/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt and receipt['policy']==w.POLICY
    src=R/'work/incoming-review12'
    transfer=read(src/'transfer.json')
    assert transfer['origin_host']=='fmd' and transfer['binary_sha256']==w.BINARY_SHA
    rows=transfer['files'];names=[r['relative_path'] for r in rows]
    assert len(names)==len(set(names))==53
    for row in rows:
        rel=Path(row['relative_path'])
        assert not rel.is_absolute() and '..' not in rel.parts
        assert sha(src/rel)==row['sha256']
    comparison=read(src/'comparison.json')
    assert comparison['table_count']==len(comparison['tables'])==45
    assert all(r['full_rows_equal_after_exact_narration_directory_substitution'] is True for r in comparison['tables'])
    permit=read(src/'distributed-review-permit12.json')
    assignment=read(src/'chapter-assignment12.json')
    assert permit['assignment_sha256']==sha(src/'chapter-assignment12.json')
    selected=[r['chapter'] for r in assignment['assignments'] if r['host']=='csd']
    assert selected
    for chapter in selected:w.validate_permit(permit,assignment,chapter)
    w.check_media_paths()
    assert sha(w.BIN)==w.BINARY_SHA
    w.success('csd-raster-encoder-stop13')
    target=R/'production-inputs-raster12'
    assert not target.exists() and not w.PERMIT.exists() and not w.ASSIGNMENT.exists()
    target.mkdir()
    for name in names+['transfer.json']:
        source=src/name;dest=target/name;dest.parent.mkdir(parents=True,exist_ok=True)
        with source.open('rb') as a,dest.open('xb') as b:shutil.copyfileobj(a,b)
        assert sha(source)==sha(dest)
    for dest in (w.PERMIT,w.ASSIGNMENT):
        with (target/dest.name).open('rb') as a,dest.open('xb') as b:shutil.copyfileobj(a,b)
    results={chapter:w.check(chapter) for chapter in selected}
    with (J/'import-summary.json').open('x') as f:
        json.dump({'transfer_sha256':sha(src/'transfer.json'),'file_count':len(rows),'chapters':selected,'checks':results,'scope':'Exact imported producer picture review and CSD technical bindings; no new listening or final AV acceptance.'},f,indent=2)
    print('Review transfer validated; no render launched.')
if __name__=='__main__':run()
