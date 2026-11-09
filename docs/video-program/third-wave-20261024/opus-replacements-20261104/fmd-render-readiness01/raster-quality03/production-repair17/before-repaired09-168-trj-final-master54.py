"""Join qualified production19 chapters on TRJ. Run only inside its resource guard.

No rendering, acceptance invention, or publication. Every chapter must already
have a successful stable guard receipt and its actual visual-review permit.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import wave

ROOT = Path('/data/projects/jmev-render-worker-20261107')
D = Path('/data/projects/jmev-stroke-probe-20261106')


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def success(path):
    row = read(path)
    assert row.get('finished') and row.get('exit') == 0
    assert row['inputs_sha256'] == row['final_inputs_sha256']
    assert not any(row.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))
    return row


def run(args):
    assert args.job == 'trj-production19-master01'
    job = ROOT / 'jobs' / args.job
    receipt = read(job / 'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8
    assert receipt['policy']['phase_growth_gib'] == 4
    assert sha(args.inputs) == receipt['inputs_sha256']
    frozen = read(args.inputs)['files']
    bound = {row['path']: row['sha256'] for row in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path

    def require(path):
        assert bound.get(str(path)) == sha(path), str(path)

    require(Path(__file__).resolve())
    require(args.worker)
    require(args.plan)
    spec = importlib.util.spec_from_file_location('chapter_worker', args.worker)
    worker = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(worker)
    require(worker.BIN)
    qualification_path = D / 'jobs/csd-production-repair-build19/qualification.json'
    require(qualification_path)
    chapters = read(qualification_path)['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    plan = read(args.plan)
    assert [row['chapter'] for row in plan['chapters']] == chapters
    out = job / 'final'
    assert not out.exists()
    out.mkdir()
    links = []
    total = 0

    def link(src, dst):
        require(src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        os.link(src, dst)
        links.append(dict(source=str(src), path=str(dst), sha256=sha(src)))

    for row in plan['chapters']:
        chapter = row['chapter']
        chapter_worker_path = Path(row.get('worker', str(args.worker)))
        require(chapter_worker_path)
        chapter_spec = importlib.util.spec_from_file_location('worker_' + chapter, chapter_worker_path)
        chapter_worker = importlib.util.module_from_spec(chapter_spec)
        chapter_spec.loader.exec_module(chapter_worker)
        source_root = chapter_worker.ROOT
        prefixes = {
            ROOT: 'trj-',
            Path('/data/projects/jmev-hz2-render-worker-20261107'): 'hz2-',
            Path('/data/projects/jmev-css-render-worker-20261107'): 'css-',
            Path('/data/projects/jmev-ts1-render-worker-20261107'): 'ts1-',
            Path('/data/projects/jmev-ts2-render-worker-20261107'): 'ts2-',
        }
        assert source_root in prefixes
        require(chapter_worker.BIN)
        assert Path(row['job']).name == row['job']
        expected_prefix = prefixes[source_root]
        assert row['job'].startswith(expected_prefix + 'production19-final-')
        source_job = source_root / 'jobs' / row['job']
        if source_root != ROOT:
            # The local import must itself be a completed guarded operation.
            # Foreign paths remain unchanged, with exact content available locally.
            import_receipt = Path(row['import_receipt'])
            require(import_receipt)
            imported = success(import_receipt)
            import_manifest = Path(row['import_inputs'])
            require(import_manifest)
            assert sha(import_manifest) == imported['inputs_sha256']
            for item in read(import_manifest)['files']:
                require(Path(item['path']))
                assert sha(item['path']) == item['sha256']
        prior_receipt = source_job / 'resource-receipt.json'
        require(prior_receipt)
        prior = success(prior_receipt)
        manifest = Path(row['inputs'])
        require(manifest)
        assert sha(manifest) == prior['inputs_sha256']
        for item in read(manifest)['files']:
            assert bound.get(item['path']) == item['sha256'] == sha(item['path'])
        permit = Path(row['permit'])
        require(permit)
        review, _ = chapter_worker.check(chapter, permit)
        frames = review['frames']
        source = source_job / 'render'
        chapter_worker.verify_tables(chapter, source, frames)
        summary_path = source_job / 'render-summary.json'
        require(summary_path)
        summary = read(summary_path)
        assert (summary['chapter'], summary['frames']) == (chapter, frames)
        assert summary['binary_sha256'] == sha(chapter_worker.BIN)
        assert summary['permit_sha256'] == sha(permit)
        for item in summary['files']:
            require(Path(item['path']))
            assert sha(item['path']) == item['sha256']
        for suffix in ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv'):
            link(source / (chapter + suffix), out / (chapter + suffix))
        captures = sorted((source / 'soundtrack' / chapter).glob('soundtrack_*.wav'),
                          key=lambda p: int(p.stem.split('_')[-1]))
        assert captures
        capture = captures[-1]
        with wave.open(str(capture)) as stream:
            assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth(),
                    stream.getnframes()) == (48000, 2, 2, frames * 800)
        link(capture, out / 'soundtrack' / chapter / 'soundtrack_0.wav')
        total += frames
    assert total == 133662
    (job / 'staged-inputs.json').write_text(json.dumps(links, indent=2))
    os.environ.update(TMPDIR=str(ROOT / 'work/tmp'), XDG_CACHE_HOME=str(ROOT / 'work/cache'))
    for command, positional in [('master', []), ('captions', ['all'])]:
        argv = [str(worker.BIN), command, *positional, '--narration', str(worker.NARRATION)]
        if command == 'master':
            argv += ['--score', str(worker.STEMS)]
        argv += ['--out', str(out)]
        with (job / (command + '-native.log')).open('x') as log:
            subprocess.run(argv, cwd=D / 'work/production-repo07', stdout=log,
                           stderr=subprocess.STDOUT, check=True, timeout=14400)
    replays = [line.split()[0] for line in (job / 'master-native.log').read_text().splitlines()
               if "replay = fmn's soundtrack, bit for bit" in line]
    assert replays == chapters
    outputs = {key: out / ('complex_analysis_visual' + suffix) for key, suffix in
               [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams',
                                               '-of', 'json', str(outputs['master'])]))
    videos = [x for x in probe['streams'] if x['codec_type'] == 'video']
    audios = [x for x in probe['streams'] if x['codec_type'] == 'audio']
    assert len(videos) == len(audios) == 1
    video, audio = videos[0], audios[0]
    assert (video['width'], video['height'], video['avg_frame_rate'],
            int(video['nb_frames'])) == (3840, 2160, '60/1', total)
    assert (audio['sample_rate'], audio['channels'], audio['codec_name']) == ('48000', 2, 'aac')
    assert abs(float(audio['duration']) - total / 60) <= 1024 / 48000 + .001
    assert outputs['srt'].stat().st_size and outputs['vtt'].read_text().startswith('WEBVTT')
    for item in links:
        assert sha(item['source']) == sha(item['path']) == item['sha256']
    for path, digest in bound.items():
        assert sha(path) == digest
    result = dict(chapters=chapters, frames=total, native_bit_identical_replays=len(replays),
                  outputs={key: dict(path=str(path), sha256=sha(path)) for key, path in outputs.items()},
                  full_decode=False, final_picture_review=False, motion_review=False,
                  listening_accepted=False, independent_av_acceptance=False,
                  exact_caption_check=False, publication_accepted=False)
    (job / 'master-summary.json').write_text(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--job', required=True)
    parser.add_argument('--inputs', required=True, type=Path)
    parser.add_argument('--worker', required=True, type=Path)
    parser.add_argument('--plan', required=True, type=Path)
    run(parser.parse_args())
