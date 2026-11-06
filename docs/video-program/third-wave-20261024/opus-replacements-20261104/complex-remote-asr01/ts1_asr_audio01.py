"""Step 1 of 2 on ts1: prove the final complex master, then decode its audio for recognition elsewhere.

ts1 has the master but no fw and no model (Mail 5523), so this job stops at the WAV. mini_asr01.py takes it from there.
The decode is logarithm's: asr02.py's two statements, by AST. Start through start-asr-audio01.sh.

    ts1_asr_audio01.py --check VERIFY_JOB MASTER_JOB NN    read-only: every gate, no job name used
    ts1_asr_audio01.py VERIFY_JOB MASTER_JOB NN            freeze the manifest, run ts1's guard, recheck, print the hash
    ts1_asr_audio01.py --run VERIFY_JOB MASTER_JOB NN      the guard's child; never call it by hand

Paths are logical, as the chain's are: ts1's repository path is a symlink, so R comes from the sealed qualifier.
A pass here shows which audio the final master holds. It is not listening, fine sync or AV acceptance.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import wave

HERE = Path(__file__).resolve().parent
QUALIFIER = HERE.parent / 'complex-remote-ts1-pipeline01/qualify01/ts1_qualify01.py'
QUALIFIER_SHA256 = 'ee2d7719218be4e8d77a14f99a352e2f6a8d287fba1befed3bf8db57d0fb619b'
with QUALIFIER.open('rb') as handle:
    assert hashlib.file_digest(handle, 'sha256').hexdigest() == QUALIFIER_SHA256, 'Sealed qualifier changed'
SPEC = importlib.util.spec_from_file_location('ts1_qualify', QUALIFIER)
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)
R, O, JOBS, C, read, sha = Q.R, Q.O, Q.JOBS, Q.C, Q.read, Q.sha

ME = R / Path(__file__).resolve().relative_to(Q.PHYSICAL)
START = ME.parent / 'start-asr-audio01.sh'
QUALIFIER_LOGICAL = R / QUALIFIER.relative_to(Q.PHYSICAL)
FINAL = ME.parent.parent / 'complex-remote-final-production01'
MASTER_HELPER, VERIFY_HELPER = FINAL / 'ts1_final_master01.py', FINAL / 'ts1_final_verify01.py'
STAGE = O / 'final-production01'
WORK = JOBS / 'complex-remote-render-work01'
MOVIE = WORK / 'complex_analysis_visual_master.mp4'
SRT, VTT = WORK / 'complex_analysis_visual.en.srt', WORK / 'complex_analysis_visual.en.vtt'
PICTURE_PYTHON = JOBS / 'complex-remote-picture-env01/env/bin/python'
TIMELINE_JOB = 'complex-remote-timeline02'
TIMELINE = JOBS / TIMELINE_JOB / 'timeline-summary.json'
N2 = JOBS / 'complex-remote-narration-derive01/narration-n2'
SCRIPT = C / 'script.tsv'
SCRIPT_SHA256 = '0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee'
CUES = 105
VOICE = 'ElevenLabs Frederick Surrey'
VERIFY_NAME = r'complex-remote-final-verify01(-r[2-9])?'
MASTER_NAME = r'complex-remote-master01(-r[2-9])?'
VERIFY_SUCCESS = 'FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; every sheet still needs actual viewing.'
MASTER_SUCCESS = 'MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, loudness and listening remain.'
POLICY = {**Q.POLICY, 'admission_gib': 12, 'phase_growth_gib': 1}  # the guard's phase asr
LENGTH_SECONDS = 0.2  # the delivery A/V limit


def select(verify_job, master_job, attempt):
    """The explicit selection, set once: root names the jobs, so a retry needs no new script."""
    global VERIFY_JOB, MASTER_JOB, ATTEMPT, JOB, OUT, MANIFEST, POSTCHECK
    assert re.fullmatch(VERIFY_NAME, verify_job), f'bad verify job {verify_job}'
    assert re.fullmatch(MASTER_NAME, master_job), f'bad master job {master_job}'
    assert re.fullmatch(r'\d\d', attempt) and int(attempt) >= 1, f'bad attempt {attempt}'
    VERIFY_JOB, MASTER_JOB, ATTEMPT = verify_job, master_job, attempt
    JOB = f'complex-remote-asr-audio{attempt}'
    OUT = JOBS / JOB
    MANIFEST = O / f'asr-audio-inputs{attempt}.json'
    POSTCHECK = STAGE / f'{verify_job}-postcheck.json'  # Lavender's verify helper writes this once


def child_command():
    return [sys.executable, '-I', '-B', str(ME), '--run', VERIFY_JOB, MASTER_JOB, ATTEMPT]


def tool(name):
    found = shutil.which(name)
    assert found, 'Missing tool: ' + name
    return Path(found).resolve()


def normalized(path):
    return [line for line in path.read_text().splitlines() if line and not line.startswith('#')]


def newest(job, pattern):
    """An explicit selection can't be a stale one: no later attempt may sit beside it."""
    later = [x.name for x in JOBS.iterdir() if re.fullmatch(pattern, x.name) and x.name > job]
    assert not later, f'{job} is not the newest: {sorted(later)}'


def passed(job, command, helper, success):
    """One finished job, exactly: Q.success, phase, command, frozen inputs, its helper and last log line."""
    folder = JOBS / job
    inputs = STAGE / f'{job}-inputs.json'
    assert Q.success(job)
    d = read(folder / 'resource-receipt.json')
    assert d.get('phase') == 'native', f'{job} ran in phase {d.get("phase")}'
    assert d.get('command') == command, f'{job} ran another command'
    assert d['inputs_sha256'] == sha(inputs), f'{job}: its frozen inputs moved'
    assert {'path': str(helper), 'sha256': sha(helper)} in read(inputs)['files'], f'{job}: its helper moved'
    lines = [x for x in (folder / 'command.log').read_text(errors='replace').splitlines() if x.strip()]
    assert lines and lines[-1] == success, f'{job}: the log does not end with its success line'


def script_ids():
    assert sha(SCRIPT) == SCRIPT_SHA256, 'script.tsv is not the frozen Frederick script'
    return [line.split('\t', 1)[0] for line in normalized(SCRIPT)]


def final_placements(timeline):
    """The placements again, from the actual final tables, as the verify job builds them."""
    placements, chapters, offset = [], [], 0
    for chapter in timeline['chapters']:
        name = chapter['chapter']
        table, source = WORK / f'{name}.timeline.tsv', JOBS / TIMELINE_JOB / 'timeline-output' / f'{name}.timeline.tsv'
        assert sha(source) == chapter['timeline_sha256'], f'{name}: the timeline table moved'
        assert normalized(table) == normalized(source), f'{name}: the final table differs from the timeline'
        rows = [line.split('\t') for line in normalized(table)]
        frames = int(next(row[1] for row in rows if row[0] == 'frames'))
        assert frames == chapter['frames'], f'{name}: the final table has another length'
        for row in rows:
            if row[0] == 'line':
                assert Path(row[4]).resolve() == (N2 / f'{row[1]}.wav').resolve(), f'{row[1]}: the final table names another narration file'
                with wave.open(row[4]) as take:
                    placements.append({'cue': row[1], 'chapter': name, 'start_frame': offset + int(row[2]),
                                       'duration_s': take.getnframes() / take.getframerate()})
        chapters.append({'chapter': name, 'start_frame': offset, 'frames': frames})
        offset += frames
    return placements, chapters, offset


def bound_proof():
    """Everything ASR rests on, read from the selected jobs' own records."""
    verify, master = JOBS / VERIFY_JOB, JOBS / MASTER_JOB
    newest(VERIFY_JOB, VERIFY_NAME)
    newest(MASTER_JOB, MASTER_NAME)
    passed(VERIFY_JOB, [str(PICTURE_PYTHON), '-I', '-B', str(VERIFY_HELPER), '--run', VERIFY_JOB], VERIFY_HELPER, VERIFY_SUCCESS)
    passed(MASTER_JOB, [sys.executable, '-I', '-B', str(MASTER_HELPER), '--run', MASTER_JOB], MASTER_HELPER, MASTER_SUCCESS)
    proof = read(verify / 'verification.json')
    timeline = read(TIMELINE)
    assert proof['master_job'] == MASTER_JOB, f"the proof is for {proof['master_job']}"
    assert proof['pictures_viewed'] is False, 'the proof claims a picture viewing'
    assert (proof['host'], proof['voice']) == ('ts1', VOICE), 'the proof is for another host or voice'
    assert proof['binary_sha256'] == Q.BIN_SHA256 == timeline['binary_sha256'], 'the proof names another binary'
    assert proof['timeline_job'] == TIMELINE_JOB and proof['narration'] == str(N2), 'the proof names another timeline or narration'
    assert proof['video'] == str(MOVIE), 'the proof names another master file'
    assert isinstance(proof['frames'], int) and proof['frames'] > 0
    assert proof['frames'] == timeline['total_frames'] and proof['fps'] == timeline['fps'] == 60, 'the proof has another length'
    assert tuple(x['chapter'] for x in timeline['chapters']) == Q.CHAPTERS, 'not the 15 registered chapters'
    placements, chapters, total = final_placements(timeline)
    assert proof['chapters'] == chapters and total == proof['frames'], 'the proof has other chapters'
    assert len(placements) == CUES and [x['cue'] for x in placements] == script_ids(), 'the final tables do not place the 105 cues in order'
    assert proof['placements'] == placements, 'the proof has other placements'
    strict = (verify / 'strict-video-verification.log').read_text(errors='replace').splitlines()
    assert strict and strict[-1] == 'VERIFY PASS', 'the strict video check did not pass'
    assert sha(MOVIE) == proof['sha256'], 'the master changed after verification'
    summary = read(master / 'master-summary.json')
    outputs = summary['outputs']
    assert outputs['master'] == {'path': str(MOVIE), 'sha256': proof['sha256']}, 'the summary is for another master'
    assert outputs['srt'] == {'path': str(SRT), 'sha256': sha(SRT)} and outputs['vtt'] == {'path': str(VTT), 'sha256': sha(VTT)}, 'the summary names other captions'
    assert summary['frames'] == proof['frames'] and summary['binary_sha256'] == Q.BIN_SHA256, 'the summary is for another film'
    assert POSTCHECK.is_file(), f'no verify post-check record: {POSTCHECK.name}'
    record = read(POSTCHECK)
    assert record.get('job') == VERIFY_JOB and record.get('master_job') == MASTER_JOB, 'the post-check is for another job'
    assert record.get('verification_sha256') == sha(verify / 'verification.json'), 'the post-check read another proof'
    assert record.get('master_sha256') == proof['sha256'] and record.get('frames') == proof['frames'], 'the post-check read another master'
    assert record.get('srt_sha256') == outputs['srt']['sha256'] and record.get('vtt_sha256') == outputs['vtt']['sha256'], 'the post-check read other captions'
    assert record.get('pictures_viewed') is False, 'the post-check claims a picture viewing'
    return proof


def bound_ffmpeg():
    """The bytes the verify job froze and decoded with, not whatever PATH finds today."""
    ffmpeg = tool('ffmpeg')
    rows = {row['path']: row['sha256'] for row in read(STAGE / f'{VERIFY_JOB}-inputs.json')['files']}
    assert rows.get(str(ffmpeg)) == sha(ffmpeg), 'ffmpeg is not the verify job\'s tool'
    return ffmpeg


def live_receipt():
    """This job's receipt while the guard owns it. The guard rewrites it in place just after starting the job, so a
    read can land inside that write. A receipt that never parses refuses."""
    for _ in range(100):
        try:
            return read(OUT / 'resource-receipt.json')
        except json.JSONDecodeError:
            time.sleep(0.1)
    raise RuntimeError('The guard receipt never settled')


def check_files(rows):
    for row in rows:
        assert sha(row['path']) == row['sha256'], row['path']


def run():
    receipt = live_receipt()
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt.get('phase') == 'asr' and receipt.get('inputs_sha256') == sha(MANIFEST), 'not this guarded job'
    assert receipt['command'] == child_command(), 'the guard ran another command'
    assert receipt.get('policy') == POLICY, 'Guard policy changed'
    check_files(read(MANIFEST)['files'])
    proof = bound_proof()
    ffmpeg = str(bound_ffmpeg())
    movie = Path(proof['video'])
    wav = OUT / 'encoded-master-16k-mono.wav'
    subprocess.run([ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-i', str(movie),
                    '-map', '0:a:0', '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', str(wav)],
                   check=True, timeout=1800)
    with wave.open(str(wav)) as audio:
        shape = (audio.getframerate(), audio.getnchannels(), audio.getsampwidth())
        samples = audio.getnframes()
    assert shape == (16000, 1, 2), f'the decode is not 16 kHz mono PCM: {shape}'
    assert abs(samples / 16000 - proof['frames'] / proof['fps']) <= LENGTH_SECONDS, f'the decode lasts {samples / 16000} s'
    with (OUT / 'audio-summary.json').open('x') as handle:
        json.dump({'job': JOB, 'verify_job': VERIFY_JOB, 'master_job': MASTER_JOB, 'host': 'ts1', 'voice': VOICE,
                   'video': proof['video'], 'sha256': proof['sha256'], 'frames': proof['frames'], 'fps': proof['fps'],
                   'chapters': proof['chapters'], 'placements': proof['placements'], 'pictures_viewed': False,
                   'binary_sha256': Q.BIN_SHA256, 'timeline_job': TIMELINE_JOB, 'script_sha256': SCRIPT_SHA256,
                   'wav': str(wav), 'wav_sha256': sha(wav), 'wav_samples': samples,
                   'verification_sha256': sha(JOBS / VERIFY_JOB / 'verification.json'), 'postcheck_sha256': sha(POSTCHECK),
                   'ffmpeg': {'path': ffmpeg, 'sha256': sha(ffmpeg)},
                   'decode_command': [ffmpeg, '-nostdin', '-n', '-v', 'error', '-xerror', '-i', str(movie),
                                      '-map', '0:a:0', '-ac', '1', '-ar', '16000', '-c:a', 'pcm_s16le', str(wav)],
                   'scope': 'Audio decoded from the verified master for ASR triage on another host. No recognition has run. No speech correctness, listening, fine sync or independent AV acceptance.'}, handle, indent=2)
    print(f'ASR AUDIO DECODED: {samples} samples from master {proof["sha256"]}.', flush=True)


def manifest_paths():
    verify, master = JOBS / VERIFY_JOB, JOBS / MASTER_JOB
    paths = [ME, START, QUALIFIER_LOGICAL, SCRIPT, MOVIE, SRT, VTT, MASTER_HELPER, VERIFY_HELPER, TIMELINE, POSTCHECK,
             Q.GUARD, O / 'guard-pins01.json', tool('python3'), bound_ffmpeg(),
             verify / 'resource-receipt.json', verify / 'command.log', verify / 'verification.json', verify / 'strict-video-verification.log',
             master / 'resource-receipt.json', master / 'command.log', master / 'master-summary.json',
             STAGE / f'{VERIFY_JOB}-inputs.json', STAGE / f'{MASTER_JOB}-inputs.json']
    paths += [Path(row['path']) for row in read(O / 'guard-pins01.json')['files']]
    for chapter in Q.CHAPTERS:
        paths += [WORK / f'{chapter}.timeline.tsv', JOBS / TIMELINE_JOB / 'timeline-output' / f'{chapter}.timeline.tsv']
    paths += [N2 / f'{cue}.wav' for cue in script_ids()]
    return paths


def check():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate decode refused'
    Q.guard_policy()
    proof = bound_proof()
    bound_ffmpeg()
    print(json.dumps({'job': JOB, 'master_sha256': proof['sha256'], 'frames': proof['frames'], 'cues': len(proof['placements']), 'launched': False}))


def launch():
    assert not OUT.exists() and not MANIFEST.exists(), 'Duplicate decode refused'
    Q.guard_policy()
    proof = bound_proof()
    rows = [{'path': str(p), 'sha256': sha(p)} for p in manifest_paths()]
    with MANIFEST.open('x') as handle:
        json.dump({'files': rows}, handle, indent=2)
    subprocess.run([sys.executable, '-I', '-B', str(Q.GUARD), '--phase', 'asr', '--job', JOB, '--inputs', str(MANIFEST), '--', *child_command()],
                   cwd=R, check=True)
    d = read(OUT / 'resource-receipt.json')
    assert d.get('finished') and d.get('admitted') is True and d.get('exit') == 0, f'{JOB} did not pass'
    assert not d.get('stopped_for_resource_limit') and 'error' not in d, f'{JOB} stopped or raised'
    assert d['inputs_sha256'] == d['final_inputs_sha256'] == sha(MANIFEST), f'{JOB}: its frozen inputs moved'
    summary = read(OUT / 'audio-summary.json')
    assert summary['sha256'] == proof['sha256'] == sha(MOVIE) and summary['wav_sha256'] == sha(OUT / 'encoded-master-16k-mono.wav'), 'the decode moved'
    print(f'ASR AUDIO PASS: {JOB} audio-summary.json sha256 {sha(OUT / "audio-summary.json")}; transfer it, the WAV and script.tsv to the ASR host.', flush=True)


if __name__ == '__main__':
    if not __debug__:
        raise SystemExit('ts1_asr_audio01.py checks with assertions: launch without -O')
    args = sys.argv[1:]
    mode = args.pop(0) if args[:1] in (['--run'], ['--check']) else None
    select(*args)
    {None: launch, '--check': check, '--run': run}[mode]()
