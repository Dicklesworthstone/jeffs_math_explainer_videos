"""Read-only export manifest for completed local native09 render819 and verification820.

Prints JSON only after terminal stable receipts, every frozen input, and all
actual picture-extraction outputs match. The manifest is transport evidence,
not picture viewing, listening, independent review, or publication acceptance.
No files, processes, uploads or transfers are created by this helper.
"""
import hashlib
import json
from pathlib import Path

R = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
Q = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17')
W = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/narration_lab/third-wave-20261024/root/five-film-release-20261027')
FINAL = W/'complex-local-final819'
PICTURES = W/'complex-local-pictures820'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(path.read_text())

def terminal(path):
    d = read(path)
    assert d.get('finished') and d.get('exit') == 0, ('Not a completed successful job', str(path))
    assert d.get('admitted') and not d.get('preflight_only')
    assert not any(d.get(k) for k in ('resource_stop', 'stopped_for_resource_limit', 'error'))
    assert d['inputs_sha256'] == d['final_inputs_sha256'], path
    return d

def main():
    final = terminal(FINAL/'resource-receipt.json')
    pictures = terminal(PICTURES/'resource-receipt.json')
    manifest_paths = [Q/'local-final819-inputs.json', Q/'local-pictures820-inputs.json']
    rows = {}
    for manifest, receipt in zip(manifest_paths, [final, pictures]):
        assert sha(manifest) == receipt['inputs_sha256'], manifest
        entries = read(manifest)['files']
        assert len({x['path'] for x in entries}) == len(entries)
        for entry in entries:
            path, digest = entry['path'], entry['sha256']
            assert path not in rows or rows[path] == digest, ('Inconsistent frozen binding', path)
            assert sha(path) == digest, path
            rows[path] = digest
        rows[str(manifest)] = sha(manifest)
    report = read(PICTURES/'pictures820.json')
    movie = FINAL/'render/09_boundary.mp4'
    assert report['movie'] == str(movie) and report['movie_sha256'] == sha(movie)
    assert report['frames'] == 16504 and report['full_decode'] is True
    assert len(report['pictures']) == len(report['selected_frames']) >= 814
    assert len(report['sheets']) == (len(report['pictures'])+7)//8
    prior = read(R/'jobs/mini-boundary-pictures775/pictures775.json')
    assert set(prior['selected_frames']) <= set(report['selected_frames'])
    assert report['preview_sampling_sha256'] == sha(R/'jobs/mini-boundary-pictures775/pictures775.json')
    for entry in report['pictures']+report['sheets']:
        path = Path(entry['path'])
        assert path.is_relative_to(PICTURES) and sha(path) == entry['sha256'], path
    for ext, digest in report['tables'].items():
        assert sha(FINAL/'render'/('09_boundary.'+ext)) == digest
    # Preserve receipts, raw tables, captures, all samples, and their full input
    # closure. Destination import must create new names, never replace old ones.
    for folder in [FINAL, PICTURES]:
        for path in folder.rglob('*'):
            if path.is_file():
                rows[str(path)] = sha(path)
    rows[str(Path(__file__).resolve())] = sha(Path(__file__).resolve())
    exported = []
    for name, digest in sorted(rows.items()):
        path = Path(name)
        assert path.is_absolute() and '..' not in path.parts
        assert not any(c in name for c in ('\t', '\n', '\r', '\x00'))
        assert sha(path) == digest, ('File changed during export', name)
        exported.append(dict(path=name, sha256=digest, bytes=path.stat().st_size))
    output = dict(schema='native-local-export827-v1', chapter='09_boundary',
                  source_root=str(R), render_job=str(FINAL), verification_job=str(PICTURES),
                  render_inputs_sha256=final['inputs_sha256'], verification_inputs_sha256=pictures['inputs_sha256'],
                  movie_sha256=sha(movie), frames=16504, files=exported,
                  total_bytes=sum(x['bytes'] for x in exported),
                  scope='Verified transport closure only. Actual picture viewing, listening, fine sync, independent final AV and publication are not accepted.')
    print(json.dumps(output, indent=2))

if __name__ == '__main__':
    main()
