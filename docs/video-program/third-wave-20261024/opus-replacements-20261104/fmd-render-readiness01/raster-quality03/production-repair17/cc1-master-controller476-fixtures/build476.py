"""task476 (CC1): write cc1-master447-manifest-draft476.json from draft460 (fcf0e7d7): row 08_loop_sums moves to the
CSS352 + 371 route; the other 14 rows are kept byte-identical. Every 08 name comes from frozen 371's own functions
(ba337aca), as controller 464's rows_and_receipts() checks them. Usage: python3 -I -B build476.py Q"""
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

Q = Path(sys.argv[1])
DRAFT460, DRAFT460_SHA = Q/'master447-manifest-draft460.json', 'fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3'
I371, I371_SHA = Q/'csd-css352-import371.py', 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'
PERMIT474, PERMIT474_SHA = Q/'08_loop_sums-css-permit474.json', '92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a'
OUT = Q/'cc1-master447-manifest-draft476.json'
CH = '08_loop_sums'


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


assert sha(DRAFT460) == DRAFT460_SHA and sha(I371) == I371_SHA and sha(PERMIT474) == PERMIT474_SHA
spec = importlib.util.spec_from_file_location('import371', I371)
i = importlib.util.module_from_spec(spec)
spec.loader.exec_module(i)
raw = DRAFT460.read_bytes()
d = json.loads(raw)
assert (json.dumps(d, indent=2)+'\n').encode() == raw
rows = d['chapters']
old = next(r for r in rows if r['chapter'] == CH)
template = next(r for r in rows if r['chapter'] == '01_the_bump')      # the CSS352 + 371 row shape, key order
job = i.job_name(CH, 1)
new = dict(chapter=CH, kind='combined', worker=str(i.WORKER), worker_sha256=i.WORKER_SHA256, job=i.css_job(CH),
           inputs=str(i.css_inputs(CH)), permit=str(i.PERMITS/PERMIT474.name),
           import_receipt=str(i.D/'jobs'/job/'resource-receipt.json'), import_inputs=str(i.inputs_path(job)))
assert list(new) == list(template) and new['worker'] == template['worker'] and new['worker_sha256'] == template['worker_sha256']
pin = {k: (new[k] if k in new else ('0'*64 if k.endswith('sha256') else 1)) for k in i.PIN_KEYS}   # names real, digests dummy
assert i.pin_problems({CH: pin}) == [], i.pin_problems({CH: pin})
chapters = [new if r['chapter'] == CH else r for r in rows]
body = dict(chapters=chapters, draft476=dict(
    status='Unlaunched candidate. This file records planned bindings, not terminal results or visual or audio acceptance.',
    reader='combined-master447.py reads chapters; draft476 is explanatory metadata.',
    supersedes=dict(path=DRAFT460.name, sha256=DRAFT460_SHA),
    bindings=[
        'Fourteen chapter rows are byte-identical to draft460; the registry still totals 133662 frames (08 keeps 14959).',
        '08_loop_sums moves from CSD352 (csd-reviewed-final-08-loop-sums352, permit375) to CSS352: worker css-final-repair352.py '
        '(1074051b), job css-reviewed-final-08-loop-sums352, inputs final-08_loop_sums-inputs352.json, and root\'s host-only permit '
        '08_loop_sums-css-permit474.json (92119d3a; permit375 with host css) staged in CSS repair352-permits; imported by 371 attempt a1.',
        'Gallery05 keeps the exact SSD442 worker and gallery inputs, imported by 448 attempt a1.',
        'CSS01/04/12/14 keep their 352 workers and 371 attempt a1 imports; CSD06/07/10/11 keep 352 and 09 keeps 344; '
        'retained 02/03/13/15 keep their worker, import and retention-decision bindings.'],
    pending=[
        "Root's CSS 08 render (css-final-repair352.py with permit474) and an inspection of its actual terminal receipt.",
        "Root's 371 export of 08 once terminal; controller 476 takes its Q file name and sha256 as --loop-sums-manifest and --loop-sums-sha.",
        'A 371 import of 08 at attempt a1: relay394 lists only 01/04/12/14/05, so the 08 import needs a relay successor or another '
        'root-approved transport.',
        'Withdrawal or transfer of the CSD 08 job is root\'s, with CC3; this draft neither reads nor needs that job. 371 refuses '
        'the 08 import while any CSD D/jobs/csd-reviewed-final-08-loop-sums* folder exists (one host per chapter), and needs '
        "root's Q copy of permit474 at its sha.",
        'All final master, technical, picture, audio-review, fine-sync, independent AV and public playback checks remain open.']))
data = (json.dumps(body, indent=2)+'\n').encode()
with open(OUT, 'xb') as f:
    f.write(data)
same = [r['chapter'] for r in rows if r['chapter'] != CH and json.dumps(r, indent=2) in data.decode()]
print(json.dumps(dict(out=OUT.name, sha256=hashlib.sha256(data).hexdigest(), bytes=len(data), old08=old, new08=new,
                      rows_byte_identical=len(same), changed=[r['chapter'] for r, s in zip(rows, chapters) if r != s]), indent=1))
