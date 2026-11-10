"""Prepared native09 successor: requires actual766/775 output and every selected-picture review. Not yet run."""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

R = Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
Q = R/'qualification'
JOB = R/'jobs/mini-boundary-final772'
INPUTS = Q/'root-boundary772-final-inputs.json'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def success(path):
    r = read(path)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0 and not r.get('preflight_only')
    assert r['inputs_sha256'] == r['final_inputs_sha256']
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    return r

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--review', type=Path, required=True)
    parser.add_argument('--review-sha', required=True)
    a = parser.parse_args()
    assert not JOB.exists() and not INPUTS.exists()
    assert a.review.parent == Q and not a.review.is_symlink() and sha(a.review) == a.review_sha
    source = Q/'root-boundary775-inputs.json'
    picture_job = R/'jobs/mini-boundary-pictures775'
    state = success(picture_job/'resource-receipt.json')
    assert sha(source) == state['inputs_sha256']
    entries = read(source)['files']
    rows = {r['path']: r['sha256'] for r in entries}
    assert len(rows) == len(entries)
    for name, digest in rows.items():
        assert sha(name) == digest, name
    extra = [source, a.review, Path(__file__).resolve(), Path(sys.executable).resolve()]
    for job in ('native-boundary761', 'mini-boundary-delivery764', 'mini-boundary-timeline765',
                'mini-boundary-preview766', 'mini-boundary-pictures775'):
        p = R/'jobs'/job/'resource-receipt.json'
        success(p)
        extra.append(p)
    report_path = picture_job/'pictures775.json'
    report = read(report_path)
    viewing = read(a.review)
    movie = R/'jobs/mini-boundary-preview766/preview/09_boundary.mp4'
    assert report['movie_sha256'] == viewing['movie_sha256'] == sha(movie)
    assert viewing['extraction_manifest_sha256'] == sha(report_path)
    assert report['frames'] == 16504 and report['full_decode'] is True
    assert len(report['pictures']) == len(report['selected_frames']) == viewing['selected_pictures_viewed']
    assert len(report['pictures']) >= 313
    assert len(report['sheets']) == (len(report['pictures'])+7)//8
    assert viewing['sheets_viewed'] == list(range(1, len(report['sheets'])+1))
    assert viewing['complete'] is True
    assert viewing['picture_indices_viewed'] == list(range(1, len(report['pictures'])+1))
    assert viewing['concrete_defects'] == [] and viewing['observations']
    for row in report['pictures']+report['sheets']:
        p = Path(row['path'])
        assert p.is_relative_to(picture_job) and sha(p) == row['sha256']
        extra.append(p)
    extra.extend((report_path, movie))
    for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
        ref = R/'jobs/mini-boundary-timeline765/timeline-output'/('09_boundary.'+ext)
        actual = movie.parent/('09_boundary.'+ext)
        assert ref.read_bytes() == actual.read_bytes()
        extra.extend((ref, actual))
    # Picture-only repair must preserve the entire native preview soundtrack.
    old_pcm = R/'jobs/mini-boundary-preview571/preview/soundtrack/09_boundary/soundtrack_0.wav'
    new_pcm = R/'jobs/mini-boundary-preview766/preview/soundtrack/09_boundary/soundtrack_0.wav'
    assert sha(old_pcm) == sha(new_pcm) == 'f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51'
    extra.extend((old_pcm, new_pcm))
    score_manifest = Q/'root-boundary586-score-transfer.json'
    score = read(score_manifest)
    assert len(score['files']) == 5
    for row in score['files']:
        p = Path(row['path'])
        assert p.is_relative_to(R/'score586') and not p.is_symlink()
        assert p.stat().st_size == row['bytes'] and sha(p) == row['sha256']
        extra.append(p)
    parts = R/'score586/09_boundary.music.f32.parts.tsv'
    assert sha(parts) == 'db31e88404eb9af836c6575cf5b4178a292eaeceea283d717bf2a4a7166ffe2e'
    records = [line.split('\t') for line in parts.read_text().splitlines()[1:]]
    assert len(records) == 3
    offset = 0
    for start, count, rate, digest, name in records:
        assert int(start) == offset and int(rate) == 48000 and Path(name).name == name
        assert sha(parts.parent/name) == digest
        offset += int(count)
    assert offset == 16504*800
    assert read(R/'score586/qa/09_boundary.json')['passed'] is True
    binary = R/'jobs/native-boundary761/complex_analysis_visual-761'
    assert sha(binary) == '71736c26d9953f08af8f4ce4f6fc0f13aeb98e7135b3dd415fb3ebf00e161c26'
    extra.extend((score_manifest, binary, Q/'cc2-mini-max-guard508.py'))
    for p in extra:
        digest = sha(p)
        assert str(p) not in rows or rows[str(p)] == digest, p
        rows[str(p)] = digest
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=p, sha256=h) for p,h in sorted(rows.items())]), f, indent=2)
    cmd = [sys.executable, '-I', '-B', str(Q/'cc2-mini-max-guard508.py'), '--phase', 'native',
           '--job', JOB.name, '--inputs', str(INPUTS), '--', str(binary), 'render', '09_boundary',
           '--res', '3840x2160', '--fps', '60', '--threads', '4', '--narration',
           str(R/'jobs/mini-boundary-delivery764/narration'), '--score', str(R/'score586'),
           '--out', str(JOB/'render'), '--lint', '--crf', '16', '--preset', 'veryfast']
    env = dict(os.environ, PATH=str(Path.home()/'.local/bin')+':/opt/homebrew/bin:/usr/bin:/bin')
    return subprocess.call(cmd, env=env)

if __name__ == '__main__':
    raise SystemExit(main())
