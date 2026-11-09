"""Review repaired569 with all prior545 frames, new lint and dense prior-defect samples; no visual acceptance."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
BASE = HERE.parent/'root-apportionment-picture-support616.py'
assert hashlib.sha256(BASE.read_bytes()).hexdigest() == '4deb8cf2b846607c1a4733494c502c2d1ae0834ddfb59e46361f2bce69d9d112'
spec = importlib.util.spec_from_file_location('sampling462', BASE)
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)
P528 = h.WORK/'apportionment-frederick-preview528'
CHAPTER = sys.argv[2] if len(sys.argv) == 3 else ''
assert CHAPTER and all(c.isalnum() or c == '_' for c in CHAPTER)
JOB = h.WORK/('apportionment-final-pictures634-'+CHAPTER)
INPUTS = HERE/('pictures634-'+CHAPTER+'-inputs.json')


def build():
    for path, digest in h.GUARD_CHAIN.items():
        assert h.sha(path) == digest, path
    prior = h.WORK/('apportionment-final632-'+CHAPTER)
    timeline = h.WORK/'apportionment-picture-timeline567/timeline'
    original = HERE/('final632-'+CHAPTER+'-inputs.json')
    receipt = prior/'resource-receipt.json'
    h.success(receipt)
    h.stable([receipt])
    assert h.sha(original) == json.loads(receipt.read_text())['inputs_sha256']
    frozen = json.loads(original.read_text())['files']
    for row in frozen:
        assert h.sha(Path(row['path'])) == row['sha256'], row['path']
    summary_path = prior/'final632-summary.json'
    summary = json.loads(summary_path.read_text())
    chapters = (CHAPTER,)
    assert set(summary['chapters']) == set(chapters)
    source = prior/'render'
    read = [BASE, Path(__file__).resolve(), HERE/'pictures635.py', original, receipt, summary_path,
            h.FFMPEG.resolve(), h.FFPROBE.resolve(), Path(sys.executable).resolve(), *h.GUARD_CHAIN]
    read.extend(Path(x['path']) for x in frozen)
    plans, sources = {}, {}
    ranges = {'08_signposts': (53.4, 57.717), '13_population': (25.6, 54.417), '15_recap': (43.65, 47.767)}
    for c in chapters:
        record = summary['chapters'][c]
        assert record['folder'] == str(source.relative_to(h.REPO))
        for name, digest in record['files'].items():
            assert not Path(name).is_absolute() and '..' not in Path(name).parts
            p = source/name
            assert not p.is_symlink() and h.sha(p) == digest, p
            read.append(p)
        name, frames, lines, marks = h.read_timeline(source/(c+'.timeline.tsv'))
        assert name == c and frames == record['frames']
        assert (source/(c+'.timeline.tsv')).read_bytes() == (timeline/(c+'.timeline.tsv')).read_bytes()
        cues = h.read_cues(source/(c+'.cues.tsv'), lines)
        anchors = h.merge_anchors([h.read_layout(timeline/(c+'.layout.tsv'), 'timeline567'),
                                   h.read_layout(source/(c+'.layout.tsv'), 'final632')])
        old_job = 'apportionment-picture-review584' if c in ranges else 'apportionment-frederick-pictures545'
        old_path = h.WORK/old_job/c/'sampling.json'
        old_plan = json.loads(old_path.read_text())
        assert old_plan['frames'] == frames
        plan = h.plan_chapter(c, frames, cues, marks, anchors)
        picks = {x['frame']: x for x in plan['samples']}
        for sample in old_plan['samples']:
            entry = picks.setdefault(sample['frame'], dict(frame=sample['frame'], seconds=sample['seconds'], reasons=[]))
            entry['reasons'].extend('retained-preview: '+r for r in sample['reasons'])
        plan['samples'] = [picks[f] for f in sorted(picks)]
        h.probe(source/(c+'.mp4'), frames)
        plans[c] = dict(chapter=c, frames=frames, fps=60,
                        registry_offset_frames=old_plan['registry_offset_frames'], **plan)
        sources[c] = source
        read += [old_path, timeline/(c+'.timeline.tsv'), timeline/(c+'.layout.tsv')]
    assert sum(len(p['samples']) for p in plans.values()) <= h.MAX_SAMPLES
    return plans, sources, sorted(set(read))


def main():
    mode = sys.argv[1]
    assert mode in ('inputs', 'extract')
    if mode == 'inputs':
        assert not JOB.exists() and not INPUTS.exists()
    else:
        r = json.loads((JOB/'resource-receipt.json').read_text())
        assert r['admitted'] and r['phase'] == 'native' and not r['preflight_only']
        assert r['command'][-5:] == ['-I', '-B', str(Path(__file__).resolve()), 'extract', CHAPTER]
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
                    scope='Selected native4K final632 pictures, not yet viewed. No motion, listening, fine-sync or AV acceptance.')
        with (folder/'sampling.json').open('x') as f:
            json.dump(plan, f, indent=2)
            f.write('\n')
        done[c] = {'samples': len(plan['samples']), 'sheets': len(plan['sheets'])}
        print(json.dumps({'chapter': c, **done[c]}), flush=True)
    with (JOB/'pictures634-summary.json').open('x') as f:
        json.dump({'chapters': done, 'scope': 'Extraction only; actual picture review remains.'}, f, indent=2)
        f.write('\n')


if __name__ == '__main__':
    main()
