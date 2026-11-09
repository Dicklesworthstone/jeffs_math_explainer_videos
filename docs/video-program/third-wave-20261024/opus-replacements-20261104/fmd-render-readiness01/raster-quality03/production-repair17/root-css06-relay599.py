"""Chapter06 specialization of frozen relay485, using reviewed host-only route578 and permit577.

Prepared only; run after actual terminal CSS06 export is staged and pinned on all three hosts.
Requires existing01/04/12/14/08/07 imports first; gallery uses its separate448 route.
Retains original guard, duplicate-attempt refusal, stream admission and full post-import verification.
No render, transfer, viewing, listening or final audiovisual acceptance is claimed here.
"""
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE/'cc1-css352-import-relay485.py'
assert hashlib.sha256(BASE.read_bytes()).hexdigest() == '05617cb88da8786232ad7234a4e3208d024fe3560bd4713ac035f82f07f162d0'
spec = importlib.util.spec_from_file_location('relay485_for599', BASE)
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
r.VERSION = '599'
r.CHAPTER = '06_one_derivative'
r.ORDER = ('01_the_bump', '04_reading_color', '12_residues', '14_continuation', '08_loop_sums', '07_one_point', '06_one_derivative', '05_gallery')
DRAFT = HERE/'root-master-manifest578.json'
DRAFT_SHA = '32ed2941ffb8ec443328dbef7adb0eab01c7e321ed5232bca0c4f57668fc073b'
PERMIT = '06_one_derivative-css-permit577.json'
PERMIT_SHA = 'acd8b1744fdce09f1c3eda93fcac49621b932b42e1b0ac46cdeddcfc21f42f9a'


def route578(h, args, pin):
    if not DRAFT.is_file() or r.sha(DRAFT) != DRAFT_SHA:
        return ['draft578 absent or changed']
    row = next(x for x in json.loads(DRAFT.read_text())['chapters'] if x['chapter'] == r.CHAPTER)
    out = [f'manifest {k} differs from draft578 chapter06' for k in r.ROW_PIN if pin[k] != row[k]]
    if Path(pin['permit']).name != PERMIT or pin['permit_sha256'] != PERMIT_SHA:
        out.append('manifest does not bind actual permit577')
    if not (HERE/PERMIT).is_file() or r.sha(HERE/PERMIT) != PERMIT_SHA:
        out.append('local permit577 absent or changed')
    if r.expected_rows(h, args.chapter, args.attempt) != {r.CHAPTER: {k: row[k] for k in ('import_receipt', 'import_inputs')}}:
        out.append('attempt is not the attempt1 bound by draft578/controller534')
    return out


def local_manifest(h, args):
    if args.chapter != r.CHAPTER:
        raise r.Stop(1, '599 only imports06_one_derivative')
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
    problems = route578(h, args, pin)
    if problems:
        raise r.Stop(1, '; '.join(problems))
    return manifest, pin


r.local_manifest = local_manifest

if __name__ == '__main__':
    raise SystemExit(r.main())
