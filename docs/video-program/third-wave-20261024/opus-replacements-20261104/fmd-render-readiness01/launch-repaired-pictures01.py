"""Freeze the successful preview and actual Pillow runtime before extraction.
Extracting sheets does not establish that anyone viewed them.
"""
import importlib.util
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
HERE = ROOT / 'qualification01'


def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def pillow_runtime():
    import PIL
    from PIL import Image, ImageDraw, ImageFont
    image = Image.new('RGB',(960,580),(15,15,20))
    ImageDraw.Draw(image).text((8,545),'0001 09_boundary 12.500s',font=ImageFont.load_default(),fill='white')
    stream = io.BytesIO()
    image.save(stream,format='PNG')
    stream.seek(0)
    with Image.open(stream) as decoded:
        decoded.load()
        assert decoded.size == image.size and decoded.mode == 'RGB'
        assert decoded.tobytes() == image.tobytes()
    blank = Image.new('RGB',(960,40),(15,15,20))
    assert image.crop((0,540,960,580)).tobytes() != blank.tobytes()
    folder = Path(PIL.__file__).parent
    paths = {p.resolve() for p in folder.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    libraries = set()
    for extension in sorted(p for p in paths if p.suffix == '.so'):
        output = subprocess.check_output(['/usr/bin/ldd',str(extension)],text=True,timeout=30)
        assert 'not found' not in output, extension
        for line in output.splitlines():
            match = re.search(r'(/\S+)',line)
            if match:
                libraries.add(Path(match[1]).resolve())
    assert libraries, 'No native Pillow dependencies found'
    return sorted(paths | libraries), {'version':PIL.__version__,'package':str(folder),
        'pixel_roundtrip_equal':True,'label_pixels_present':True,'scope':'Synthetic runtime check, not actual video picture review'}


def main():
    assert sys.argv[1:] == ['--run']
    pictures = load('repaired-pictures01.py')
    bindings = load('launch-preview02.py')
    deadline = time.monotonic() + 86400
    while True:
        path = pictures.P.JOB/'resource-receipt.json'
        receipt = pictures.read(path) if path.exists() else {}
        if receipt.get('finished') or 'exit' in receipt:
            break
        # Abort a failed upstream controller rather than waiting indefinitely.
        for name in ('stage08-picture-timeline01', 'stage08-picture-encoder-stop01'):
            upstream = ROOT/'jobs'/name/'resource-receipt.json'
            state = pictures.read(upstream) if upstream.exists() else {}
            if 'exit' in state: pictures.S.success(name)
        assert time.monotonic() < deadline, 'Waiting deadline; no job restarted'
        time.sleep(20)
    pictures.P.check_score()
    terminal = pictures.S.success('stage08-picture-preview01')
    previous = ROOT/'stage08-picture-preview01-inputs.json'
    assert pictures.sha(previous) == terminal['inputs_sha256']
    assert not pictures.JOB.exists(), 'Never duplicate picture extraction'
    summary_path = pictures.P.JOB/'preview-summary.json'
    summary = pictures.read(summary_path)
    assert summary['binary_sha256'] == pictures.BINARY_SHA
    assert [row['chapter'] for row in summary['chapters']] == list(pictures.CHAPTERS)
    extra = [previous,Path(__file__),HERE/'repaired-pictures01.py',HERE/'picture-preview01.py',HERE/'score01.py',
             HERE/'compare-full01.py',HERE/'launch-preview02.py',ROOT/'guard_csd02.py',
             pictures.P.JOB/'resource-receipt.json',summary_path,pictures.PRIOR]
    for row in summary['chapters']:
        chapter = row['chapter']
        for suffix, key in [('mp4','mp4_sha256'),('timeline.tsv','timeline_sha256'),('layout.tsv','layout_sha256')]:
            path = pictures.PREVIEW_OUT/(chapter+'.'+suffix)
            assert pictures.sha(path) == row[key], path
            extra.append(path)
        extra.extend(pictures.PREVIEW_OUT/(chapter+'.'+suffix) for suffix in ('cues.tsv','cuelog.tsv'))
        extra.append(pictures.OLD_PREVIEW/(chapter+'.layout.tsv'))
    environment, report = pillow_runtime()
    extra.extend(environment)
    for tool in ('ffmpeg','ffprobe'):
        path = shutil.which(tool)
        assert path, tool
        extra.append(Path(path).resolve())
    extra.append(Path(sys.executable).resolve())
    report['files_sha256'] = {str(p):pictures.sha(p) for p in environment}
    report_path = HERE/'repaired-pictures-runtime01.json'
    with report_path.open('x') as stream:
        json.dump(report,stream,indent=2)
    extra.append(report_path)
    manifest = ROOT/'stage08-repaired-pictures01-inputs.json'
    frozen = bindings.collect(previous,extra)
    pictures.P.check_score()
    with manifest.open('x') as stream:
        json.dump(frozen,stream,indent=2)
    command = ['/usr/bin/python3','-I','-B',str(ROOT/'guard_csd02.py'),'--phase','native',
               '--job','stage08-repaired-pictures01','--inputs',str(manifest),'--',
               '/usr/bin/python3','-I','-B',str(HERE/'repaired-pictures01.py'),'--run']
    print('Launching guarded picture extraction with',len(frozen['files']),'bindings',flush=True)
    return subprocess.call(command)


if __name__ == '__main__':
    raise SystemExit(main())
