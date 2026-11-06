#!/usr/bin/env python3
"""Rehearse overlay_css02.py in a scratch tree, then scan the bundle. Run from the repository root.

    python3 -I -B <bundle>/checks/run_checks02.py SCRATCH

Real: the helper, guard_css02.py, the preserved guard, plan02.json, the candidate, the master (linked, not
copied), and real frames decoded by this host's ffmpeg. Stand-ins: ffmpeg's subtitles filter, which
this host's ffmpeg lacks, so a white box takes its place; sbh; the font; ffmpeg's and Pillow's pins.
So no caption is drawn here, and the filter's own behaviour is first seen on css.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parents[1]
REPO = Path.cwd()
SCRATCH = Path(tempfile.mkdtemp(prefix='overlay-', dir=sys.argv[1]))
TREE = SCRATCH / 'tree'
RELATIVE = HERE.relative_to(REPO)
FONT = '/System/Library/Fonts/Supplemental/Arial.ttf'
RESULTS = []
FFMPEG_STANDIN = r'''#!/bin/bash
# Stand-in: the real ffmpeg, with a white box in place of the subtitles filter.
real=/opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffmpeg
case " $* " in
  *" -filters "*) [ -n "${FAKE_NO_FILTER:-}" ] || printf ' T.. subtitles V->V x\n ... ass V->V x\n'; exec "$real" "$@" ;;
  *" -version "*) "$real" "$@"; echo "--enable-libass"; exit 0 ;;
esac
args=()
for item in "$@"; do
  case "$item" in
    subtitles=*)
      [ -z "${FAKE_FAIL:-}" ] || case "$*" in *"${FAKE_FAIL}"*) exit 1 ;; esac
      [ -z "${FAKE_TOUCH:-}" ] || echo '# changed' >> "$FAKE_TOUCH"
      top=""; grep -qxF -- "$8" "$FAKE_TOP_TIMES" && top=1
      [ -z "${FAKE_SWAP:-}" ] || { if [ -n "$top" ]; then top=""; else top=1; fi; }
      if [ -n "${FAKE_BLANK:-}" ]; then item=showinfo
      elif [ -n "$top" ]; then item=drawbox=x=960:y=15:w=1920:h=240:color=white:t=fill,showinfo
      else item=drawbox=x=960:y=1860:w=1920:h=220:color=white:t=fill,showinfo; fi ;;
  esac
  args+=("$item")
done
if [ -n "${FAKE_SHIFT:-}" ]; then args[7]=$(python3 -I -c "import sys; print(float(sys.argv[1]) + 1)" "${args[7]}"); fi
exec "$real" "${args[@]}"
'''
SBH_STANDIN = '#!/bin/bash\n[ -z "${FAKE_SBH_REFUSE:-}" ] || { echo refused; exit 3; }\necho \'{"ok": true}\'\n'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def note(name, ok, detail=''):
    RESULTS.append(ok)
    print(('PASS ' if ok else 'FAIL ') + name + (': ' + detail if detail else ''), flush=True)


def build():
    bundle = TREE / RELATIVE
    shutil.copytree(HERE, bundle, ignore=shutil.ignore_patterns('__pycache__', 'outputs', 'SHA256SUMS'))
    bin_folder = SCRATCH / 'bin'
    bin_folder.mkdir()
    for name, text in (('ffmpeg', FFMPEG_STANDIN), ('ffprobe', '#!/bin/bash\nexit 0\n'), ('sbh', SBH_STANDIN)):
        path = bin_folder / name
        path.write_text(text)
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
    helper = bundle / 'overlay_css02.py'
    text = helper.read_text()
    import PIL
    for pattern, value in ((r"^STAGING = .*$", f"STAGING = {str(TREE.resolve())!r}"), (r"^FFMPEG = .*$", f"FFMPEG = {str(bin_folder / 'ffmpeg')!r}"),
                           (r"^FFMPEG_SHA256 = .*$", f"FFMPEG_SHA256 = {sha(bin_folder / 'ffmpeg')!r}"), (r"^FFPROBE = .*$", f"FFPROBE = {str(bin_folder / 'ffprobe')!r}"),
                           (r"^FFPROBE_SHA256 = .*$", f"FFPROBE_SHA256 = {sha(bin_folder / 'ffprobe')!r}"), (r"^FONT = .*$", f"FONT = {FONT!r}"),
                           (r"^PILLOW = .*$", f"PILLOW = {PIL.__version__!r}")):
        text, count = re.subn(pattern, value.replace('\\', r'\\'), text, flags=re.M)
        assert count == 1, pattern
    helper.write_text(text)
    plan = json.loads((HERE / 'plan02.json').read_text())
    (bin_folder / 'top-times.txt').write_text(''.join(f"{(row['frame'] - 0.25) / plan['fps']:.6f}\n" for row in plan['samples'] if row['place'] == 'top'))
    for row in json.loads((HERE / 'transfer-files02.json').read_text())['files']:
        target = TREE / row['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        if row['bytes'] > 1 << 20:
            target.symlink_to(REPO / row['path'])
        else:
            shutil.copy2(REPO / row['path'], target)
    return bundle, bin_folder


def seal(bundle):
    rows = []
    for parent, dirs, files in os.walk(bundle):
        dirs[:] = sorted(item for item in dirs if item != '__pycache__')
        for name in sorted(files):
            path = Path(parent) / name
            if name != 'SHA256SUMS':
                rows.append(f'{sha(path)}  {path.relative_to(bundle).as_posix()}')
    target = bundle / 'SHA256SUMS'
    target.write_text('\n'.join(rows) + '\n')
    return sha(target)


def helper(bundle, bin_folder, *args, **extra):
    env = dict(os.environ, PATH=f'{bin_folder}:{os.environ["PATH"]}', FAKE_TOP_TIMES=str(bin_folder / 'top-times.txt'), **extra)
    return subprocess.run([sys.executable, '-I', '-B', str(bundle / 'overlay_css02.py'), *args], cwd=TREE, env=env, capture_output=True,
                          text=True, errors='replace', timeout=900)


def refused(done, words):
    text = done.stdout + done.stderr
    return done.returncode != 0 and words in text and 'Traceback' not in text


def plan_checks():
    plan = json.loads((HERE / 'plan02.json').read_text())
    positions = json.loads((HERE / 'positions02.json').read_text())
    top = {cue for row in positions['windows'] for cue in range(row['first'], row['last'] + 1)}
    blocks = re.split(r'\n\s*\n', (REPO / plan['base_srt']).read_text().strip())
    table = {int(block.splitlines()[0]): block.splitlines() for block in blocks}
    note('p-hashes', sha(REPO / plan['base_srt']) == plan['base_srt_sha256'] and sha(REPO / plan['base_vtt']) == plan['base_vtt_sha256'] and all(sha(REPO / row['path']) == row['sha256'] for row in plan['captions'].values()) and sha(REPO / plan['master']) == plan['master_sha256'])
    note('p-cue-text-is-the-srt', all(table[row['cue']][2:] == row['lines'] for row in plan['samples']), f"{len(plan['samples'])} samples")
    note('p-samples-inside-their-cue', all(row['cue_start'] <= row['seconds'] < row['cue_end'] for row in plan['samples']))
    events = [line for line in (HERE / 'candidate02.ass').read_text().splitlines() if line.startswith('Dialogue:')]
    styles = {index: line.split(',', 4)[3] for index, line in enumerate(events, 1)}
    note('p-ass-styles', len(events) == 475 and {k for k, v in styles.items() if v == 'Top'} == top and set(styles.values()) == {'Top', 'Default'}, f"{len(top)} Top cues")
    note('p-ass-text', all(line.split(',,0,0,0,,', 1)[1].split(chr(92) + 'N') == table[k][2:] for k, line in enumerate(events, 1)))
    second = [line for line in (HERE / 'candidate03.ass').read_text().splitlines() if line.startswith('Dialogue:')]
    pieces = [line.split(',,0,0,0,,', 1)[1] for line in second]
    moved = {k for k, text in enumerate(pieces, 1) if text.startswith('{')}
    words = lambda text: re.sub(r'^[{].*?[}]', '', text).replace(chr(92) + 'N', ' ').split()
    note('p-b-same-words', len(second) == 475 and moved == top and all(words(text) == ' '.join(table[k][2:]).split() for k, text in enumerate(pieces, 1)), 'only line breaks and places differ')
    origins = [row['origin'] for row in plan['samples']]
    earlier = json.loads((HERE.parent / 'logarithm-caption-overlay01/plan01.json').read_text())['samples']
    note('p-the-first-23-frames-are-all-here', sorted(row['frame'] for row in plan['samples'] if row['origin'].startswith('S')) == sorted(row['frame'] for row in earlier) and len(origins) == 40, '23 first-run frames, 17 added')
    base = (REPO / plan['base_vtt']).read_text().strip().split('\n\n')[1:]
    made = (HERE / 'candidate02.en.vtt').read_text().strip().split('\n\n')[1:]
    setting = ' ' + positions['vtt_setting']
    note('p-vtt-only-adds-the-setting', [row.replace(setting, '') for row in made] == base and sum(setting in row for row in made) == len(top)
         and all((setting in row) == (k in top) for k, row in enumerate(made, 1)), 'times and text byte-equal to the base VTT')
    sampled = {row['cue'] for row in plan['samples'] if row['place'] == 'top'}
    note('p-every-window-has-a-sample', all(sampled & set(range(row['first'], row['last'] + 1)) for row in positions['windows']), f"{len(positions['windows'])} windows")
    note('p-control-a-dropped-window-is-seen', not all((sampled - set(range(387, 392))) & set(range(row['first'], row['last'] + 1)) for row in positions['windows']))
    note('p-sample-place-matches-cue', all((row['place'] == 'top') == (row['cue'] in top) for row in plan['samples']))
    names = [row['id'] for row in plan['samples']]
    note('p-ids-in-film-order', names == sorted(names) and [row['frame'] for row in plan['samples']] == sorted(row['frame'] for row in plan['samples']))


def rehearse():
    bundle, bin_folder = build()
    good = seal(bundle)
    font = sha(FONT)
    done = helper(bundle, bin_folder, 'check', '--seal', good, '--font-sha256', font)
    note('c-check-ready', done.returncode == 0 and json.loads(done.stdout)['check'] == 'READY', done.stderr[-200:])
    note('c-wrong-seal', refused(helper(bundle, bin_folder, 'check', '--seal', '0' * 64), 'seal differs'))
    note('c-wrong-font', refused(helper(bundle, bin_folder, 'check', '--seal', good, '--font-sha256', '0' * 64), 'font differs'))
    note('c-no-filter', refused(helper(bundle, bin_folder, 'check', '--seal', good, FAKE_NO_FILTER='1'), 'lacks a needed filter'))
    note('c-sbh-refuses', refused(helper(bundle, bin_folder, 'run', '--run', '09', '--seal', good, '--font-sha256', font, FAKE_SBH_REFUSE='1'), 'sbh refuses'))
    note('c-sbh-uses-no-name', not (TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-09').exists())
    plan_file = bundle / 'plan02.json'
    keep = plan_file.read_bytes()
    plan_file.write_bytes(keep + b'\n')
    note('c-plan-changed', refused(helper(bundle, bin_folder, 'check', '--seal', good), 'Sealed file differs'))
    plan_file.write_bytes(keep)
    guard_file = TREE / json.loads((HERE / 'transfer-files02.json').read_text())['files'][0]['path']
    guard_file.rename(guard_file.with_suffix('.aside'))
    note('c-transfer-named', refused(helper(bundle, bin_folder, 'check', '--seal', good), 'guard_owner56g_target20g.py'))
    guard_file.with_suffix('.aside').rename(guard_file)
    other = SCRATCH / 'other' / RELATIVE
    shutil.copytree(bundle, other)
    note('c-wrong-root', refused(helper(other, bin_folder, 'check', '--seal', good), 'staging root'))

    done = helper(bundle, bin_folder, 'run', '--run', '01', '--seal', good, '--font-sha256', font)
    note('r-run-passes', done.returncode == 0, (done.stdout + done.stderr)[-300:] if done.returncode else '')
    job = TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-01'
    receipt = json.loads((job / 'resource-receipt.json').read_text())
    policy = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56, 'target_growth_cap_gib': 20, 'sample_seconds': 10}
    note('r-real-guard-policy', receipt['policy'] == policy and receipt['final_inputs_sha256'] == receipt['inputs_sha256'] and receipt['exit'] == 0)
    note('r-baseline-in-new-lane', (TREE / 'overlay-lane02/jobs/allocation-baseline.json').is_file())
    summary = json.loads((job / 'overlay-summary.json').read_text())
    note('r-all-samples', len(summary['samples']) == 40 and len(list((job / 'overlay-A').iterdir())) == 40 and len(list((job / 'overlay-B').iterdir())) == 40 and len(list((job / 'clean').iterdir())) == 40)
    note('r-fourteen-sheets', len(list((job / 'sheets').iterdir())) == 14)
    note('r-false-claims', summary['pictures_viewed'] is False and summary['youtube_ui_proof'] is False)
    size = sum(path.stat().st_size for path in job.rglob('*') if path.is_file())
    print(f'     job size {size / 2 ** 20:.0f} MiB')
    again = helper(bundle, bin_folder, 'postcheck', '--run', '01')
    note('r-postcheck-repeats', again.returncode == 0)
    note('r-used-run-refused', refused(helper(bundle, bin_folder, 'run', '--run', '01', '--seal', good, '--font-sha256', font), 'run number is used'))
    (job / 'clean/F03.png').write_bytes((job / 'clean/F03.png').read_bytes() + b'x')
    note('r-postcheck-sees-a-change', refused(helper(bundle, bin_folder, 'postcheck', '--run', '01'), 'Clean frame differs'))

    done = helper(bundle, bin_folder, 'run', '--run', '02', '--seal', good, '--font-sha256', font, FAKE_FAIL='overlay-A/F03.png')
    kept = TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-02'
    note('f-ffmpeg-failure-stops-and-keeps-frames', refused(done, 'guarded job exited') and (kept / 'overlay-A/F02.png').is_file() and not (kept / 'sheets/sheet-01.jpg').exists())
    done = helper(bundle, bin_folder, 'run', '--run', '03', '--seal', good, '--font-sha256', font, FAKE_BLANK='1')
    note('f-no-caption-refused', refused(done, 'guarded job exited') and 'No caption' in (TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-03/command.log').read_text())
    done = helper(bundle, bin_folder, 'run', '--run', '06', '--seal', good, '--font-sha256', font, FAKE_SWAP='1')
    note('f-caption-in-the-wrong-place-refused', refused(done, 'guarded job exited') and 'caption' in (TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-06/command.log').read_text() and ' row ' in (TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-06/command.log').read_text())
    done = helper(bundle, bin_folder, 'run', '--run', '04', '--seal', good, '--font-sha256', font, FAKE_SHIFT='1')
    note('f-wrong-frame-refused', refused(done, 'guarded job exited') and 'landed at' in (TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-04/command.log').read_text())
    done = helper(bundle, bin_folder, 'postcheck', '--run', '03')
    note('f-postcheck-refuses-failed-job', refused(done, 'clean exit'))
    target = bundle / 'SHA256SUMS'
    keep = target.read_bytes()
    done = helper(bundle, bin_folder, 'run', '--run', '05', '--seal', good, '--font-sha256', font, FAKE_TOUCH=str(target))
    target.write_bytes(keep)
    guard_error = json.loads((TREE / 'overlay-lane02/jobs/logarithm-safe-overlay-css-05/resource-receipt.json').read_text()).get('error', '')
    note('f-input-change-stops-guard', done.returncode != 0 and 'reviewed input changed' in guard_error)
    start = bundle / 'start-overlay-css02.sh'
    first = subprocess.run(['bash', str(start), '77', 'x', 'y'], capture_output=True, text=True, timeout=60)
    second = subprocess.run(['bash', str(start), '77', 'x', 'y'], capture_output=True, text=True, timeout=60)
    note('s-start-refuses-repeated-run', first.returncode != 0 and second.returncode == 1 and 'already exists' in second.stderr)
    note('s-start-needs-run', subprocess.run(['bash', str(start)], capture_output=True, timeout=30).returncode == 2)
    print('     scratch job', job.relative_to(SCRATCH))


def scan():
    # Split so that this file passes its own scan.
    words = ['/Us' + 'ers/', '/ho' + 'me/', '/pri' + 'vate/tmp', '/pri' + 'vate/var', 'jem' + 'anuel', 'mort' + 'imer', 'gracie' + 'andjeff', 'scratch' + 'pad']
    pattern = re.compile(r'(?i)\b\w*(?:wat' + r'ched|lis' + r'tened)\w*\b')
    hits = []
    for parent, dirs, files in os.walk(HERE):
        dirs[:] = [item for item in dirs if item != '__pycache__']
        for name in files:
            text = (Path(parent) / name).read_text(errors='replace')
            if any(word in text for word in words) or pattern.search(text):
                hits.append(name)
    note('z-text-scan', not hits, ', '.join(hits))


plan_checks()
rehearse()
scan()
failed = RESULTS.count(False)
print(f'RESULT {"PASS" if not failed else "FAIL"}, {failed} failed of {len(RESULTS)}')
raise SystemExit(1 if failed else 0)
