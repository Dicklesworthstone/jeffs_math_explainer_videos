"""Chapter07 specialization of frozen relay485, using reviewed draft534 and actual permit504.

No transfer on import. Run status/run/verify with the inherited CLI only after root stages the
actual successful CSS07 export on all three hosts. Retains485 lane, attempt, transport and
post-import byte-verification gates. Order requires01/04/12/14/08 imports first. Does not claim
any transfer, picture, listening, fine-sync or final AV acceptance. Original485 is untouched.
"""
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE/'cc1-css352-import-relay485.py'
assert hashlib.sha256(BASE.read_bytes()).hexdigest() == '05617cb88da8786232ad7234a4e3208d024fe3560bd4713ac035f82f07f162d0'
spec = importlib.util.spec_from_file_location('relay485_for547', BASE)
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
r.VERSION = '547'
r.CHAPTER = '07_one_point'
r.ORDER = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation', '08_loop_sums', '07_one_point', '05_gallery')
DRAFT = HERE/'cc1-master447-manifest-draft534.json'
DRAFT_SHA = '5a73491f64b81ee48519a01882b4c5197253827d68547489078108d85f6efe31'
PERMIT = '07_one_point-css-permit504.json'
PERMIT_SHA = 'f03572c6bfac1035efd422599d41bad81a28d1d8564663dc01313bfcc8ba57ce'


def route534(h, args, pin):
    if not DRAFT.is_file() or r.sha(DRAFT) != DRAFT_SHA:
        return ['draft534 absent or changed']
    row = next(x for x in json.loads(DRAFT.read_text())['chapters'] if x['chapter'] == r.CHAPTER)
    out = [f'manifest {k} differs from draft534 chapter07' for k in r.ROW_PIN if pin[k] != row[k]]
    if Path(pin['permit']).name != PERMIT or pin['permit_sha256'] != PERMIT_SHA:
        out.append('manifest does not bind actual permit504')
    if not (HERE/PERMIT).is_file() or r.sha(HERE/PERMIT) != PERMIT_SHA:
        out.append('local permit504 absent or changed')
    if r.expected_rows(h, args.chapter, args.attempt) != {r.CHAPTER: {k: row[k] for k in ('import_receipt', 'import_inputs')}}:
        out.append('attempt is not the attempt1 bound by draft534/controller534')
    return out


def local_manifest(h, args):
    if args.chapter != r.CHAPTER:
        raise r.Stop(1, '547 only imports07_one_point')
    if Path(args.manifest).name != args.manifest or not args.manifest.endswith('.json'):
        raise r.Stop(1, 'manifest must be a bare JSON filename in Q')
    data = (HERE/args.manifest).read_bytes()
    if hashlib.sha256(data).hexdigest() != args.manifest_sha:
        raise r.Stop(1, 'actual export manifest differs from root pin')
    manifest = json.loads(data)
    problems = h.manifest_problems(manifest)
    if problems or h.chapter_of(manifest) != r.CHAPTER:
        raise r.Stop(1, '371 rejected manifest shape or chapter')
    pin = manifest['chapters'][r.CHAPTER]
    problems = route534(h, args, pin)
    if problems:
        raise r.Stop(1, '; '.join(problems))
    return manifest, pin


r.local_manifest = local_manifest

if __name__ == '__main__':
    raise SystemExit(r.main())
