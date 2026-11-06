#!/usr/bin/env python3
"""Sampled caption-overlay frames of the logarithm master on css, through ffmpeg's real subtitles filter.

    overlay_css01.py check --seal HEX [--font-sha256 HEX]       read-only; prints JSON; exit 1 refused
    overlay_css01.py run --run NN --seal HEX --font-sha256 HEX  one guarded job, then its postcheck
    overlay_css01.py postcheck --run NN                         repeat the postcheck
    overlay_css01.py job --run NN --font-sha256 HEX             the guarded child

Default libass styling on the real SRT is a proxy. It is not YouTube's caption UI, and no check here
is a picture verdict: someone must view the sheets. Nothing is deleted or installed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
R = HERE.parents[4]
STAGING = '/data/projects/jeffs-math-logarithm-overlay-20261106'
FFMPEG = '/usr/bin/ffmpeg'
FFMPEG_SHA256 = 'bdf6aabffdba7411edff8d36c389d695257fcdf823d196020176e117612862f6'
FFPROBE = '/usr/bin/ffprobe'
FFPROBE_SHA256 = 'a6dac1e9e8631e04075d06854cc4069308aec279bb04683b4a0bca5e4f54b2fe'
FONT = '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'
PILLOW = '12.1.1'
LANE = R / 'overlay-lane01'
JOBS = LANE / 'jobs'
GUARD = HERE / 'guard_css01.py'
POLICY = {'admission_gib': 10, 'stop_free_gib': 8, 'phase_growth_gib': 4, 'work_cap_gib': 56, 'target_growth_cap_gib': 20, 'sample_seconds': 10}
SUCCESS = 'OVERLAY PASS: sampled frames and sheets written; default libass styling is a proxy, not YouTube UI; every sheet still needs actual viewing.'
SHEET_TOP, SHEET_SCALE, PER_SHEET = 1400, 2, 2


def refuse(why):
    raise SystemExit('REFUSED: ' + why)


def require(ok, why):
    if not ok:
        refuse(why)


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as handle:
        for block in iter(lambda: handle.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def plan():
    return json.loads((HERE / 'plan01.json').read_text())


def job_name(run):
    require(re.fullmatch(r'[0-9][0-9]', run), 'RUN must be two digits')
    return 'logarithm-overlay-css-' + run


def tool_run(command, timeout=60):
    done = subprocess.run(command, capture_output=True, text=True, errors='replace', timeout=timeout)
    require(done.returncode == 0, f'{command[0]} failed: {done.stderr[-300:]}')
    return done.stdout + done.stderr


def check_seal(seal):
    path = HERE / 'SHA256SUMS'
    require(path.is_file() and sha(path) == seal, 'The bundle seal differs from the one named')
    for row in path.read_text().splitlines():
        digest, name = row.split(maxsplit=1)
        target = HERE / name.lstrip('*')
        require(target.is_file() and sha(target) == digest, 'Sealed file differs: ' + name)


def check_transfers():
    rows = json.loads((HERE / 'transfer-files01.json').read_text())['files']
    missing = [row['path'] for row in rows if not (R / row['path']).is_file() or sha(R / row['path']) != row['sha256']]
    require(not missing, 'Transfer these files to the staging root, exact bytes: ' + ', '.join(missing))
    return rows


def check_tools(font_sha256):
    require(sha(FFMPEG) == FFMPEG_SHA256, 'ffmpeg differs from the bound hash')
    require(sha(FFPROBE) == FFPROBE_SHA256, 'ffprobe differs from the bound hash')
    filters = tool_run([FFMPEG, '-hide_banner', '-filters'])
    found = {name: bool(re.search(rf'^\s*\S+\s+{name}\s', filters, re.M)) for name in ('subtitles', 'ass', 'showinfo')}
    require(all(found.values()), f'ffmpeg lacks a needed filter: {found}')
    version = tool_run([FFMPEG, '-hide_banner', '-version'])
    require('--enable-libass' in version, 'This ffmpeg was not built with libass')
    require(Path(FONT).is_file(), 'Font missing: ' + FONT)
    require(font_sha256 is None or sha(FONT) == font_sha256, 'The font differs from the hash named')
    require(hasattr(hashlib, 'file_digest'), 'The preserved guard needs Python 3.11 or later')
    try:
        import PIL
    except ImportError:
        refuse('Pillow does not import under python3 -I')
    require(PIL.__version__ == PILLOW, 'Pillow differs: ' + PIL.__version__)
    return {'ffmpeg': version.splitlines()[0], 'filters': found, 'font': FONT, 'font_sha256': sha(FONT), 'pillow': PIL.__version__,
            'python': sys.executable, 'python_version': sys.version.split()[0], 'python_sha256': sha(sys.executable)}


def admission():
    done = subprocess.run(['sbh', 'check', '--target-free', '0.2', '--need', f"{POLICY['admission_gib']}G", '--json'],
                          cwd=R, capture_output=True, text=True, errors='replace', timeout=45)
    require(done.returncode == 0, f'sbh refuses admission: {done.stdout[-300:]}{done.stderr[-300:]}')


def ready(seal, font_sha256):
    require(str(R) == STAGING, f'Run this in the staging root {STAGING}, not {R}')
    check_seal(seal)
    rows = check_transfers()
    facts = check_tools(font_sha256)
    admission()
    data = plan()
    return {'check': 'READY', 'staging': STAGING, 'lane': str(LANE), 'samples': len(data['samples']), 'transfers': len(rows), **facts}


def grab(frame, vf, path, log):
    """One frame by exact pts. Input seek with -copyts keeps film time, which the subtitles filter reads."""
    data = plan()
    chain = ','.join(part for part in (vf, 'showinfo') if part)
    command = [FFMPEG, '-hide_banner', '-nostdin', '-n', '-threads', '4', '-copyts', '-ss', f"{(frame - 0.25) / data['fps']:.6f}",
               '-i', str(R / data['master']), '-map', '0:v:0', '-frames:v', '1', '-vf', chain, '-update', '1', '-c:v', 'png', str(path)]
    done = subprocess.run(command, capture_output=True, text=True, errors='replace', timeout=300)
    log.write_text(' '.join(command) + '\n' + done.stderr)
    require(done.returncode == 0 and path.is_file(), f'ffmpeg failed for frame {frame}; read {log}')
    times = re.findall(r'pts_time:([0-9.]+)', done.stderr)
    require(times and abs(float(times[0]) - frame / data['fps']) < 0.5 / data['fps'], f'Frame {frame} landed at {times[:1]}')
    return done.stderr


def compare(clean, over, row_px):
    """Where the filter changed the frame, and whether the footer row holds ink in the clean frame."""
    from PIL import Image, ImageChops
    with Image.open(clean) as first, Image.open(over) as second:
        require(first.size == second.size == (3840, 2160), 'Expected 3840x2160 frames')
        left, right = first.convert('RGB'), second.convert('RGB')
    box = ImageChops.difference(left, right).getbbox()
    require(box and box[1] >= 1000, f'No caption in the lower frame: {over.name}')
    peak = left.crop((0, row_px - 32, 3840, row_px + 32)).convert('L').getextrema()[1]
    require(peak >= 90, f'No footer ink at row {row_px}: {clean.name}')
    return list(box), peak


def sheets(out, rows, data):
    from PIL import Image, ImageDraw, ImageFont
    big, small = ImageFont.truetype(FONT, 28), ImageFont.truetype(FONT, 22)
    wide, tall = 3840 // SHEET_SCALE, (2160 - SHEET_TOP) // SHEET_SCALE
    banner = 74
    block = 42 + 2 * tall + 2 * 26
    written = []
    for start in range(0, len(rows), PER_SHEET):
        group = rows[start:start + PER_SHEET]
        sheet = Image.new('RGB', (wide, banner + PER_SHEET * block), (24, 24, 24))
        draw = ImageDraw.Draw(sheet)
        draw.text((12, 6), 'PROXY ONLY: real SRT with default libass styling, lowest 760 pixels of 2160.', font=big, fill=(255, 210, 90))
        draw.text((12, 40), f"Not YouTube's caption UI. SRT {data['srt_sha256'][:8]}, master {data['master_sha256'][:8]}.", font=big, fill=(255, 210, 90))
        y = banner
        for row in group:
            draw.text((12, y + 8), f"{row['id']}  {row['group']}  frame {row['frame']}  {row['seconds']:.3f} s  cue {row['cue']}  {row['state']}", font=big, fill=(255, 255, 255))
            y += 42
            for label, folder in (('with proxy caption', 'overlay'), ('clean frame, same pts', 'clean')):
                draw.text((12, y + 1), label, font=small, fill=(190, 190, 190))
                y += 26
                with Image.open(out / folder / (row['id'] + '.png')) as frame:
                    sheet.paste(frame.convert('RGB').crop((0, SHEET_TOP, 3840, 2160)).resize((wide, tall), Image.LANCZOS), (0, y))
                y += tall
        path = out / 'sheets' / f'sheet-{start // PER_SHEET + 1:02d}.jpg'
        sheet.save(path, quality=90)
        written.append({'path': path.relative_to(out).as_posix(), 'sha256': sha(path), 'samples': [row['id'] for row in group]})
    return written


def child(args):
    data = plan()
    name = job_name(args.run)
    out = JOBS / name
    for folder in ('clean', 'overlay', 'logs', 'sheets'):
        (out / folder).mkdir()
    srt = R / data['srt']
    require(re.fullmatch(r'[A-Za-z0-9/._-]+', str(srt)), 'The SRT path needs filter escaping')
    require(sha(R / data['master']) == data['master_sha256'] and sha(srt) == data['srt_sha256'], 'Master or SRT differs')
    facts = check_tools(args.font_sha256)
    rows, fonts = [], set()
    for row in data['samples']:
        clean, over = out / 'clean' / (row['id'] + '.png'), out / 'overlay' / (row['id'] + '.png')
        grab(row['frame'], None, clean, out / 'logs' / (row['id'] + '-clean.log'))
        text = grab(row['frame'], f'subtitles=filename={srt}', over, out / 'logs' / (row['id'] + '-overlay.log'))
        fonts.update(re.findall(r'fontselect: \(.*?\) -> ([^,]+),', text))
        box, peak = compare(clean, over, row['row_px'])
        rows.append({**row, 'clean_sha256': sha(clean), 'overlay_sha256': sha(over), 'caption_box': box, 'footer_row_peak': peak})
        print(row['id'], row['frame'], 'caption box', box, flush=True)
    require(fonts <= {FONT}, f'libass chose another font: {sorted(fonts)}')
    written = sheets(out, rows, data)
    summary = {'schema': 'logarithm-caption-overlay-summary-v1', 'job': name, 'master_sha256': data['master_sha256'], 'srt_sha256': data['srt_sha256'],
               'tools': facts, 'libass_fonts_logged': sorted(fonts), 'samples': rows, 'sheets': written, 'pictures_viewed': False, 'youtube_ui_proof': False,
               'scope': 'Proxy caption frames only. No picture verdict, listening, independent AV acceptance or YouTube check.'}
    (out / 'overlay-summary.json').write_text(json.dumps(summary, indent=1, ensure_ascii=False) + '\n')
    print(SUCCESS, flush=True)


def last_line(path):
    lines = path.read_text(errors='replace').splitlines()
    return lines[-1] if lines else ''


def postcheck(args):
    data = plan()
    name = job_name(args.run)
    out = JOBS / name
    receipt = json.loads((out / 'resource-receipt.json').read_text())
    require(receipt.get('finished') and receipt.get('admitted') is True and receipt.get('exit') == 0, 'The guard does not report a clean exit')
    require(not receipt.get('stopped_for_resource_limit') and 'error' not in receipt, 'The guard stopped this job')
    require(receipt.get('policy') == POLICY, 'Guard policy differs')
    require(receipt.get('inputs_sha256') and receipt.get('final_inputs_sha256') == receipt['inputs_sha256'], 'Inputs changed during the job')
    require(last_line(out / 'command.log') == SUCCESS, 'The success line is missing')
    summary = json.loads((out / 'overlay-summary.json').read_text())
    require([row['id'] for row in summary['samples']] == [row['id'] for row in data['samples']], 'A sample is missing')
    for row in summary['samples']:
        require(sha(out / 'clean' / (row['id'] + '.png')) == row['clean_sha256'], 'Clean frame differs: ' + row['id'])
        require(sha(out / 'overlay' / (row['id'] + '.png')) == row['overlay_sha256'], 'Overlay frame differs: ' + row['id'])
    for row in summary['sheets']:
        require(sha(out / row['path']) == row['sha256'], 'Sheet differs: ' + row['path'])
    require(sorted(path.name for path in (out / 'sheets').iterdir()) == sorted(Path(row['path']).name for row in summary['sheets']), 'Unlisted sheet')
    require(summary['pictures_viewed'] is False and summary['youtube_ui_proof'] is False, 'A false claim changed')
    record = {'job': name, 'summary_sha256': sha(out / 'overlay-summary.json'), 'samples': len(summary['samples']), 'sheets': len(summary['sheets']),
              'pictures_viewed': False, 'youtube_ui_proof': False, 'scope': summary['scope']}
    text = json.dumps(record, indent=1) + '\n'
    path = LANE / (name + '-postcheck.json')
    if path.exists():
        require(path.read_text() == text, 'A different postcheck exists: ' + path.name)
    else:
        with path.open('x') as handle:
            handle.write(text)
    print(text, end='', flush=True)


def launch(args):
    name = job_name(args.run)
    facts = ready(args.seal, args.font_sha256)
    require(not (JOBS / name).exists(), 'This run number is used: ' + name)
    LANE.mkdir(exist_ok=True)
    data = plan()
    paths = [HERE / 'overlay_css01.py', GUARD, HERE / 'plan01.json', HERE / 'SHA256SUMS', Path(FFMPEG), Path(FFPROBE), Path(FONT)]
    paths += [R / row['path'] for row in json.loads((HERE / 'transfer-files01.json').read_text())['files']]
    manifest = LANE / (name + '-inputs.json')
    rows = [{'path': str(path), 'sha256': sha(path)} for path in paths]
    with manifest.open('x') as handle:
        json.dump({'files': rows}, handle, indent=1)
    print(json.dumps({**facts, 'job': name, 'samples': len(data['samples'])}, indent=1), flush=True)
    python = [sys.executable, '-I', '-B']
    command = python + [str(GUARD), '--phase', 'native', '--job', name, '--inputs', str(manifest), '--'] + python
    command += [str(HERE / 'overlay_css01.py'), 'job', '--run', args.run, '--font-sha256', args.font_sha256]
    done = subprocess.run(command, cwd=R, timeout=3600)
    require(done.returncode == 0, f'The guarded job exited {done.returncode}; read {JOBS / name}/command.log')
    postcheck(args)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('step', choices=('check', 'run', 'postcheck', 'job'))
    parser.add_argument('--run')
    parser.add_argument('--seal')
    parser.add_argument('--font-sha256')
    args = parser.parse_args()
    if args.step == 'check':
        require(args.seal, '--seal is required')
        print(json.dumps(ready(args.seal, args.font_sha256), indent=1))
    elif args.step == 'run':
        require(args.seal and args.font_sha256 and args.run, '--run, --seal and --font-sha256 are required')
        launch(args)
    elif args.step == 'postcheck':
        require(args.run, '--run is required')
        postcheck(args)
    else:
        require(args.run and args.font_sha256, '--run and --font-sha256 are required')
        child(args)


if __name__ == '__main__':
    main()
