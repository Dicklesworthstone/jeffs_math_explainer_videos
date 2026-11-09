"""task455 (CC1): 451's whole master350() on a small master447-shaped tree, positive and mutated.

451's source is executed with only its three host-root literals rewritten to scratch roots (each found once; the
rewritten text is checked to equal 451 byte for byte after reversing the rewrite). The tree mirrors CSD: D is real,
the CSS root and the SSD442 root are symlinks into D/work (imported-css-root311, imported-css-ssd442, as 371/448
require), retained hosts are plain folders. The master job's records follow 447's own writers: chapter_gate's record
(:1105-1119, combined_scope/sampled_scope basis), master()'s staged-inputs.json links (:1302-1318) and summary
(:1356-1373), launch()'s guard command (:1246-1248); each chapter's render summary follows the SSD442 worker's
render() (:904-913: files listed from the job's own lexical render path). Real worker, verifier and master bytes
are copied so every pinned hash is genuine. Chapters are 12 frames (wav 9,600 sample frames) to stay small.
Usage: python3 -I -B master350_455.py Q OUTDIR
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import types
import wave

Q0, OUTDIR = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
V451_SHA = 'c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03'
ROOTS = {'D': '/data/projects/jmev-stroke-probe-20261106', 'CSS': '/data/projects/jmev-css-render-worker-20261107',
         'SSD': '/data/ssd/work/jmev-gallery-recovery440'}
OTHER = {'TRJ': '/data/projects/jmev-render-worker-20261107', 'HZ2': '/data/projects/jmev-hz2-render-worker-20261107'}
F = 12
QREL = 'qualification01/production-repair17'


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def put(p, data):
    p = Path(p)
    p.parent.mkdir(parents=True, exist_ok=True)
    with open(p, 'xb') as f:
        f.write(data if isinstance(data, bytes) else data.encode())
    return p


def receipt(p, inputs_sha='0'*64, command=()):
    return put(p, json.dumps(dict(started='t0', phase='native', command=list(command), preflight_only=False, samples=[],
                                  admitted=True, inputs_sha256=inputs_sha, pid=1, exit=0, final_inputs_sha256=inputs_sha,
                                  finished='t1'), indent=2))


def wav(p, frames):
    p = Path(p)
    p.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(p), 'wb') as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(48000)
        w.writeframes(b'\0' * (frames*800*4))
    return p


def build(name, variant):
    run = OUTDIR/name
    run.mkdir(parents=True)
    root = {k: run/k for k in ('D', 'CSS', 'SSD', 'TRJ', 'HZ2')}
    D = root['D']
    (D/'jobs').mkdir(parents=True)
    top, top_ssd = D/'work/imported-css-root311', D/'work/imported-css-ssd442'
    top.mkdir(parents=True)
    top_ssd.mkdir(parents=True)
    os.symlink(top, root['CSS'])
    os.symlink(top_ssd, root['SSD'])
    root['TRJ'].mkdir()
    root['HZ2'].mkdir()
    # 451 with its three root literals rewritten, nothing else
    text = (Q0/'csd-final-verify451.py').read_bytes()
    assert hashlib.sha256(text).hexdigest() == V451_SHA
    text = text.decode()
    for k, lit in ROOTS.items():
        assert text.count("'%s'" % lit) == 1, k
        text = text.replace("'%s'" % lit, repr(str(root[k])))
    assert "'/data/" not in text
    back = text
    for k, lit in ROOTS.items():
        back = back.replace(repr(str(root[k])), "'%s'" % lit)
    assert hashlib.sha256(back.encode()).hexdigest() == V451_SHA
    Q = D/QREL
    m = types.ModuleType('v451_fixture')
    m.__file__ = str(Q/'csd-final-verify451.py')
    exec(compile(text, m.__file__, 'exec'), m.__dict__)
    assert m.D == D and m.CSS == root['CSS'] and m.SSD442 == root['SSD']

    def local(s):
        for k, lit in list(ROOTS.items()) + list(OTHER.items()):
            if s.startswith(lit):
                return Path(str(root[k]) + s[len(lit):])
        raise ValueError(s)

    # real bytes at the pinned paths
    put(m.MASTER_HELPER, (Q0/'combined-master447.py').read_bytes())
    real = {m.WORKERS['boundary313'][0]: 'csd-reviewed-boundary-final344.py', m.WORKERS[('combined', D)][0]: 'csd-reviewed-final352.py',
            m.WORKERS[('combined', m.CSS)][0]: 'css-final-repair352.py', m.WORKER442: 'css-gallery-ssd442.py'}
    for dst, src in real.items():
        put(dst, (Q0/src).read_bytes())
    for w, (path, digest) in ((k, (v[0], v[1])) for k, v in m.WORKERS.items()):
        assert sha(path) == digest
    assert sha(m.WORKER442) == m.WORKER442_SHA
    registry = sorted(m.KINDS)
    receipt(m.BUILD268/'resource-receipt.json')
    put(m.BUILD268/'qualification.json', json.dumps(dict(chapters=registry)))
    receipt(m.BUILD313/'resource-receipt.json')
    put(m.BUILD313/'qualification.json', json.dumps(dict(base_binary_sha256=m.BINARY_268, binary_sha256='b'*64,
                                                          changed=['explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs'])))
    binary313 = 'b'*64
    want = {'combined': m.BINARY_268, 'retained': m.BINARY_19, 'boundary313': binary313}
    rows = json.loads((Q0/'master447-manifest-draft449.json').read_text())['chapters']
    rows = [dict(r, **{k: str(local(r[k])) for k in ('worker', 'inputs', 'permit', 'import_receipt', 'import_inputs') if k in r}) for r in rows]
    manifest = put(Q/'master447-manifest-fixture455.json', json.dumps(dict(chapters=rows), indent=2))
    retained_host = {'02_turning_numbers': root['TRJ'], '03_two_planes': root['TRJ'], '13_real_integral': root['CSS'],
                     '15_closing': root['HZ2']}
    out = m.OUT
    records, links, inputs, start = [], [], {m.MASTER_HELPER.resolve(), manifest, *real}, 0
    for row in rows:
        key, kind = row['chapter'], row['kind']
        worker, job = Path(row['worker']), row['job']
        # 447 :926-947 host rule; :912-919 / :976 source rule
        if kind == 'retained':
            host = retained_host[key]
            source = host/'jobs'/job
        elif kind == 'boundary313':
            host, source = D, D/'jobs'/job
        else:
            host = D if worker == m.WORKERS[('combined', D)][0] else root['CSS']
            source = root['SSD']/'jobs'/job if worker == m.WORKER442 else host/'jobs'/job
        rec_worker, rec_sha, rec_host = str(worker), (sha(worker) if worker.exists() else row['worker_sha256']), str(host)
        if key == '05_gallery' and variant == 'gallery_old_css_job':
            job = 'css-reviewed-final-05-gallery352'
            worker = m.WORKERS[('combined', m.CSS)][0]
            rec_worker, rec_sha, source = str(worker), sha(worker), root['CSS']/'jobs'/job
        if key == '05_gallery' and variant == 'gallery_job_ssd440':
            job = 'css-reviewed-final-05-gallery352-ssd440'
            source = root['SSD']/'jobs'/job
        if key == '05_gallery' and variant == 'gallery_host_D':
            rec_host = str(D)
        if key == '04_reading_color' and variant == 'ssd_worker_on_04':
            rec_worker, rec_sha = str(m.WORKER442), m.WORKER442_SHA
        render = source/'render'
        permit = put(Path(row['permit']) if not Path(row['permit']).exists() else run/'unused', json.dumps(dict(chapter=key)))
        for suffix in m.SUFFIXES:
            body = '# cue log\nc1\nc2\nc3\n' if suffix == '.cuelog.tsv' else '%s %s\n' % (key, suffix)
            put(render/(key+suffix), body)
        capture = wav(render/'soundtrack'/key/('soundtrack_1.wav' if key == '05_gallery' else 'soundtrack_0.wav'), F)
        receipt(source/'resource-receipt.json', command=['stub', str(worker)])
        scope = dict(basis='sampled_preview', purpose='technical_4k_render', preview_movie_sha256='0'*64,
                     sampled_intervals=[[1, 3], [5, 8]], playback_seconds=[0.1] if kind == 'boundary313' else [],
                     open_reviews=['audio', 'final_picture', 'motion'])
        listed = [dict(path=str(p), sha256=sha(p)) for p in sorted(render.rglob('*')) if p.is_file()]
        if key == '05_gallery' and variant == 'gallery_summary_lists_css_paths':
            listed = [dict(x, path=x['path'].replace(str(root['SSD']), str(root['CSS']))) for x in listed]
        summary = dict(host='csd' if host == D else 'css', chapter=key, frames=F + (key == '05_gallery' and variant == 'gallery_render_frames_mismatch'),
                       binary_sha256=want[kind], permit_sha256=sha(permit), files=listed)
        if kind != 'retained':
            summary['review_scope'] = scope
        if kind == 'boundary313':
            summary.update(boundary_sha256=m.REPAIR310_SHA, build313_qualification_sha256=sha(m.BUILD313/'qualification.json'))
        if key == '05_gallery':
            summary['worker352_sha256'] = m.WORKER442_SHA
        put(source/'render-summary.json', json.dumps(summary, indent=2))
        staged = [render/(key+s) for s in m.SUFFIXES] + [capture]
        if key == '05_gallery' and variant == 'gallery_staged_from_css_copy':
            # 447 sourcing 05 through the CSS root: same bytes (hard links), another lexical path
            alt = root['CSS']/'jobs'/job/'render'
            copies = []
            for p in staged:
                q = alt/p.relative_to(render)
                q.parent.mkdir(parents=True, exist_ok=True)
                os.link(p, q)
                copies.append(q)
            staged = copies
        for src, dst in zip(staged, [out/(key+s) for s in m.SUFFIXES] + [out/'soundtrack'/key/'soundtrack_0.wav']):
            dst.parent.mkdir(parents=True, exist_ok=True)
            os.link(src, dst)
            links.append(dict(source=str(src), path=str(dst), sha256=sha(src)))
            inputs.add(src)
        inputs.update({source/'render-summary.json', source/'resource-receipt.json', permit})
        record = dict(chapter=key, kind=kind, host=rec_host, job=job, worker=rec_worker, worker_sha256=rec_sha,
                      permit=str(permit), permit_sha256=sha(permit), binary_sha256=want[kind],
                      narration=str(m.NARRATION313 if kind == 'boundary313' else m.NARRATION), frames=F,
                      movie_sha256=sha(render/(key+'.mp4')), start_frame=start)
        if kind == 'retained':
            record.update(review_basis='historical_permit', retention_decision='fixture')
        else:
            record.update(review_basis='sampled_preview', review_scope=scope, sampled_intervals=scope['sampled_intervals'],
                          sampled_frames=7, open_reviews=scope['open_reviews'], stated_false=[])
        if host != D:
            record['import_receipt'] = row.get('import_receipt', '')
        records.append(record)
        start += F
    outputs = {}
    for k, suffix in (('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')):
        p = put(out/('complex_analysis_visual'+suffix), 'WEBVTT\n' if k == 'vtt' else k)
        outputs[k] = dict(path=str(p), sha256=sha(p))
    master_inputs = put(m.MASTER_INPUTS, json.dumps(dict(files=[dict(path=str(p), sha256=sha(p)) for p in sorted(inputs)]), indent=2))
    command = ['/usr/bin/python3', '-I', '-B', str(m.MASTER_HELPER.resolve()), 'master', '--job', m.MASTER.name,
               '--inputs', str(m.MASTER_INPUTS), '--manifest', str(manifest)]
    receipt(m.MASTER/'resource-receipt.json', sha(master_inputs), command)
    put(m.MASTER/'staged-inputs.json', json.dumps(links, indent=2))
    cap09 = out/'soundtrack'/m.BOUNDARY/'soundtrack_0.wav'
    src09 = next(x['source'] for x in links if x['path'] == str(cap09))
    summary = dict(chapters=registry, frames=m.TOTAL_FRAMES, native_bit_identical_replays=15,
                   replays={k: dict(cues=3, samples=F*800) for k in registry},
                   boundary_replay=dict(cues=3, samples=F*800, chapter=m.BOUNDARY, capture=dict(path=str(cap09), source=src09, sha256=sha(cap09)),
                                        captured_by_binary_sha256=binary313, replayed_by_binary_sha256=m.BINARY_268, bit_identical=True,
                                        capture_check_before_master={}),
                   chapter_binaries={r['chapter']: r['binary_sha256'] for r in records}, outputs=outputs,
                   version='350' if variant == 'summary_version_350' else '447', host='csd', binary_sha256=m.BINARY_268,
                   consumer_binary_sha256=m.BINARY_268, narration=str(m.NARRATION), score='stems',
                   manifest=dict(path=str(manifest), sha256=sha(manifest)), rows=records,
                   review_bases={r['chapter']: r['review_basis'] for r in records})
    put(m.MASTER/'master-summary.json', json.dumps(summary, indent=2))
    return m, root


VARIANTS = ['positive', 'gallery_host_D', 'gallery_old_css_job', 'gallery_job_ssd440', 'ssd_worker_on_04',
            'gallery_summary_lists_css_paths', 'gallery_render_frames_mismatch', 'gallery_staged_from_css_copy',
            'summary_version_350']
results = {}
assert not OUTDIR.exists()
for i, variant in enumerate(VARIANTS):
    m, root = build('%02d_%s' % (i, variant), variant)
    try:
        r = m.master350()
        g = r['sources'][m.OUT/'05_gallery.mp4']
        results[variant] = dict(outcome='accepted', gallery_source=str(g).replace(str(root['SSD']), 'SSD').replace(str(root['CSS']), 'CSS'),
                                gallery_source_is_ssd_lexical=g == m.SSD442/'jobs'/m.GALLERY442_JOB/'render/05_gallery.mp4',
                                gallery_source_resolves_into_D=str(g.resolve()).startswith(str(root['D'].resolve())),
                                ssd_job_records_bound=all(p in r['files'] for p in (m.SSD442/'jobs'/m.GALLERY442_JOB/'render-summary.json',
                                                                                     m.SSD442/'jobs'/m.GALLERY442_JOB/'resource-receipt.json')),
                                intervals=sorted(r['intervals']), playback=r['playback'],
                                sources_by_root={k: sum(str(v).startswith(str(root[k])) for v in r['sources'].values()) for k in root},
                                files=len(r['files']))
    except (AssertionError, KeyError, ValueError) as error:
        results[variant] = dict(outcome='refused', error='%s: %s' % (type(error).__name__,
                                str(error).replace(str(OUTDIR), 'T')[:200]))
print(json.dumps(results, indent=1))
