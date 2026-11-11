"""Pin step for pictures1008 (70dfce04). Root runs it; it launches nothing.

  check                 On CSS, read-only. Refuses unless css-exponential-verify1005 is terminal and stable and its
                        verification1005.json carries what pictures1008.gather() asserts. Prints the sha256 to pin.
                        Also works over ssh: ssh css /usr/bin/python3 -I -B - check < pin1008.py
  write SRC DEST SHA    Writes DEST (exclusive create; refuses if it exists) = SRC (must be 70dfce04) with only
                        `VERIFICATION_SHA256=None` replaced by the quoted SHA. Prints DEST's sha256 and the one changed line.
"""
import hashlib
import json
import os
import re
import sys
from pathlib import Path

CANDIDATE_SHA = '70dfce04fc178332d0ef9928dcbc11c35616e234be7e33f5a52eb01fcdc01b13'
D = Path('/data/projects/jmev-css-render-worker-20261107')
Q = D / 'qualification01/exponential933'
VERIFY = D / 'jobs/css-exponential-verify1005'
VM = Q / 'css-exponential-verify1005-inputs.json'
VERIFY_COMMAND = ['/usr/bin/python3', '-I', '-B', str(Q / 'verify1005.py'), 'run', '--phase', 'verify']
VERIFY_INPUTS_SHA = '5d45eac1c04c351ffdba66ca1ee97a88dc8536978a94e0a4928dc1c51189aa41'
MOVIE_SHA = 'fe33b9e48bcb456b3cad41f7651bd8e5f8e3bb1b7c6deba657e6b4d9e5fa2b80'
DELIVERY_SRT_SHA = '127f3d1ade016f125dfddff861b9614fd2850387be6e4cd9e05063e15c1d9f99'
DELIVERY_VTT_SHA = '0d2698d0c386263e9b946a5a395423b998c70b08c96cb69d21649810fa8114d2'


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def check():
    live = []
    for d in os.listdir('/proc'):
        if d.isdigit():
            try:
                cmd = Path(f'/proc/{d}/cmdline').read_bytes().replace(b'\0', b' ').decode(errors='replace')
            except OSError:
                continue
            if 'css-exponential-verify1005' in cmd or 'verify1005.py run --phase verify' in cmd:
                live.append(int(d))
    assert not live, ('verify1005 is not terminal: live pids', live)
    r = json.loads((VERIFY / 'resource-receipt.json').read_text())
    # stable(), verbatim conditions from verify1005 line 16 / pictures1008 stable().
    assert r.get('admitted') is True and r.get('finished') and r.get('exit') == 0, ('not a successful terminal receipt', {k: r.get(k) for k in ('admitted', 'finished', 'exit')})
    assert not any(r.get(k) for k in ('resource_stop', 'error', 'stopped_for_resource_limit')), 'receipt records a stop or error'
    assert r['inputs_sha256'] == r['final_inputs_sha256'] == VERIFY_INPUTS_SHA == sha(VM), 'inputs changed or differ from 5d45eac1'
    assert r['command'] == VERIFY_COMMAND, r['command']
    vp = VERIFY / 'verification1005.json'
    assert vp.is_file() and not vp.is_symlink(), 'verification1005.json missing'
    v = json.loads(vp.read_text())
    keys = json.loads((D / 'jobs/css-exponential-build959/qualification.json').read_text())['chapters']
    # The assertions pictures1008.gather() makes on v (unchanged from pictures1007 lines 104-108).
    movie = D / 'jobs/css-exponential-master987/final/exponential_magic_master.mp4'
    assert v['movie'] == str(movie) and v['movie_sha256'] == MOVIE_SHA and v['frames'] == 164420
    assert v['full_av_decode'] is True and v['all_decoded_video_frames_match_chapters'] is True
    assert v['fast_captions'] == [] and v['delivery_srt_sha256'] == DELIVERY_SRT_SHA and v['delivery_vtt_sha256'] == DELIVERY_VTT_SHA
    chapters = v['chapters']
    assert [c['chapter'] for c in chapters] == keys and len(keys) == 15
    offset = 0
    for c in chapters:
        assert c['offset_frames'] == offset and c['frames'] > 0 and c['samples'] > 0
        assert c['seed_path'] == str(D / 'jobs' / ('css-exponential-pictures985-' + c['chapter']) / 'pictures985.json')
        offset += c['frames']
    assert offset == 164420
    assert [(x['chapter'], x['offset_frames'], x['frames']) for x in v['chapter_frame_identity']] == [(c['chapter'], c['offset_frames'], c['frames']) for c in chapters]
    existing = sorted(x.name for x in (D / 'jobs').iterdir() if re.search(r'pictures100[5678]', x.name))
    staged = sorted(x.name for x in Q.iterdir() if re.search(r'pictures100[78]', x.name))
    print(json.dumps({'terminal_success': True, 'finished': r['finished'], 'exit': r['exit'],
                      'verification1005_json_sha256': sha(vp), 'resource_receipt_sha256': sha(VERIFY / 'resource-receipt.json'),
                      'lufs': v.get('lufs'), 'true_peak_dbfs': v.get('true_peak_dbfs'), 'samples': sum(c['samples'] for c in chapters),
                      'existing_picture_jobs': existing, 'staged_pictures1007_or_1008_files': staged}, indent=1))


def write(src, dest, digest):
    assert re.fullmatch('[0-9a-f]{64}', digest), 'SHA must be 64 lowercase hex digits'
    raw = Path(src).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == CANDIDATE_SHA, 'SRC is not the candidate 70dfce04'
    text = raw.decode()
    old = 'VERIFICATION_SHA256=None\n'
    assert text.count(old) == 1
    new = text.replace(old, f"VERIFICATION_SHA256='{digest}'\n")
    a, b = text.splitlines(), new.splitlines()
    changed = [i + 1 for i, (x, y) in enumerate(zip(a, b)) if x != y]
    assert len(a) == len(b) and changed == [57], changed
    with open(dest, 'x', encoding='utf-8') as f:
        f.write(new)
    print(json.dumps({'dest': str(dest), 'sha256': sha(dest), 'changed_line': 57, 'line': b[56]}))


if __name__ == '__main__':
    assert sys.flags.isolated and sys.dont_write_bytecode, 'run with python3 -I -B'
    if sys.argv[1:] == ['check']:
        check()
    elif len(sys.argv) == 5 and sys.argv[1] == 'write':
        write(*sys.argv[2:])
    else:
        raise SystemExit(__doc__)
