"""Use reviewed462 sampling on the actual mixed458/528 preview; no visual acceptance."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
BASE = HERE/'cc0-apportionment-preview-review462.py'
assert hashlib.sha256(BASE.read_bytes()).hexdigest() == '235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37'
spec = importlib.util.spec_from_file_location('sampling462', BASE)
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)
P528 = h.WORK/'apportionment-frederick-preview528'
JOB = h.WORK/'apportionment-frederick-pictures545'
INPUTS = h.S/'pictures545-inputs.json'


def build():
    receipts = [h.T444/'resource-receipt.json', h.S445/'resource-receipt.json', P528/'resource-receipt.json']
    for p in receipts:
        h.success(p)
    h.stable(receipts)
    for p, digest in h.GUARD_CHAIN.items():
        assert h.sha(p) == digest
    r = json.loads(receipts[-1].read_text())
    original = h.S/'preview528-inputs538.json'
    assert h.sha(original) == r['inputs_sha256']
    frozen = json.loads(original.read_text())['files']
    for row in frozen:
        assert h.sha(Path(row['path'])) == row['sha256'], row['path']
    summary_path = P528/'preview528-summary.json'
    summary = json.loads(summary_path.read_text())
    chapters = summary['chapters']
    timeline_summary = h.T444/'timeline444-summary.json'
    t = json.loads(timeline_summary.read_text())
    assert chapters == t['chapters'] and len(set(chapters)) == len(chapters) == 15
    assert summary['all_timelines_byte_identical'] is True
    assert summary['reused']['chapters'] == chapters[:8]
    assert summary['rendered']['chapters'] == chapters[8:]
    assert summary['reused']['job'] == str(h.P458.relative_to(h.REPO))
    assert summary['rendered']['folder'] == str((P528/'preview').relative_to(h.REPO))
    assert h.sha(h.P458/'resource-receipt.json') == summary['reused']['receipt_sha256'] == 'a8d25c31a921545220e6911d73fc870c827dede9311d4a3cf00cb59e0fe4d96d'
    plans, sources, offset = {}, {}, 0
    read = [BASE, Path(__file__).resolve(), original, summary_path, timeline_summary, *receipts,
            *h.GUARD_CHAIN, h.FFMPEG.resolve(), h.FFPROBE.resolve(), Path(sys.executable).resolve()]
    read.extend(Path(x['path']) for x in frozen)
    for index, c in enumerate(chapters):
        source = (h.P458 if index < 8 else P528)/'preview'
        record = summary['chapter'][c]
        assert record['folder'] == str(source.relative_to(h.REPO))
        for name, digest in record['files'].items():
            assert not Path(name).is_absolute() and '..' not in Path(name).parts
            p = source/name
            assert not p.is_symlink() and h.sha(p) == digest, p
            read.append(p)
        name, frames, lines, marks = h.read_timeline(source/(c+'.timeline.tsv'))
        assert name == c and frames == t['frames'][c] == record['frames']
        assert (source/(c+'.timeline.tsv')).read_bytes() == (h.TIMELINE/(c+'.timeline.tsv')).read_bytes()
        cues = h.read_cues(source/(c+'.cues.tsv'), lines)
        anchors = h.merge_anchors([h.read_layout(h.TIMELINE/(c+'.layout.tsv'), 'timeline444'),
                                   h.read_layout(source/(c+'.layout.tsv'), 'preview458' if index < 8 else 'preview528')])
        if c == '08_signposts':
            assert set(h.ANCHORS08) <= {a['seconds'] for a in anchors}
        h.probe(source/(c+'.mp4'), frames)
        plans[c] = dict(chapter=c, frames=frames, fps=h.FPS, registry_offset_frames=offset,
                        **h.plan_chapter(c, frames, cues, marks, anchors))
        sources[c] = source
        offset += frames
        read += [h.TIMELINE/(c+'.timeline.tsv'), h.TIMELINE/(c+'.layout.tsv')]
    assert offset == 100886
    assert sum(len(p['samples']) for p in plans.values()) <= h.MAX_SAMPLES
    for base in [h.P458, P528]:
        read.extend(p for p in base.rglob('*') if p.is_file())
    return plans, sources, sorted(set(read))


def main():
    mode = sys.argv[1]
    assert mode in ('inputs', 'extract')
    if mode == 'inputs':
        assert not JOB.exists() and not INPUTS.exists()
    else:
        r = json.loads((JOB/'resource-receipt.json').read_text())
        assert r['admitted'] and r['phase'] == 'native' and not r['preflight_only']
        assert 'finished' not in r and 'exit' not in r and r['inputs_sha256'] == h.sha(INPUTS)
        assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    plans, sources, read = build()
    rows = [{'path': str(p), 'sha256': h.sha(p)} for p in read]
    if mode == 'inputs':
        with INPUTS.open('x') as f:
            json.dump({'files': rows}, f, indent=2)
            f.write('\n')
        print(json.dumps({'inputs': str(INPUTS), 'bindings': len(rows), 'samples': sum(len(p['samples']) for p in plans.values())}))
        return
    bound = {x['path']: x['sha256'] for x in json.loads(INPUTS.read_text())['files']}
    assert all(bound.get(x['path']) == x['sha256'] for x in rows)
    done = {}
    for c, plan in plans.items():
        folder = JOB/c
        movie = sources[c]/(c+'.mp4')
        pictures = h.extract_frames(movie, [s['frame'] for s in plan['samples']], folder)
        for sample, path in zip(plan['samples'], pictures):
            sample.update(path=str(path), sha256=h.sha(path))
        plan.update(movie={'path': str(movie), 'sha256': h.sha(movie)}, sheets=h.sheets(c, plan['samples'], folder),
                    scope='Selected mixed458/528 preview pictures, not yet viewed. No motion, listening, fine-sync or AV acceptance.')
        with (folder/'sampling.json').open('x') as f:
            json.dump(plan, f, indent=2)
            f.write('\n')
        done[c] = {'samples': len(plan['samples']), 'sheets': len(plan['sheets'])}
        print(json.dumps({'chapter': c, **done[c]}), flush=True)
    with (JOB/'pictures545-summary.json').open('x') as f:
        json.dump({'chapters': done, 'scope': 'Extraction only; actual picture review remains.'}, f, indent=2)
        f.write('\n')


if __name__ == '__main__':
    main()
