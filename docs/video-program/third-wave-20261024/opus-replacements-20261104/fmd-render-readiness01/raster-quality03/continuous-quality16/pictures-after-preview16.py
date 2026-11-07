import json,hashlib,subprocess,time,io,re,shutil,sys
from pathlib import Path
R=Path('/data/projects/jmev-stroke-probe-20261106');Q=R/'qualification01/continuous-quality16'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
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


for _ in range(86400):
 p=R/'jobs/csd-continuous-preview16/resource-receipt.json'
 try:d=json.loads(p.read_text())
 except (FileNotFoundError,json.JSONDecodeError):d={}
 if 'exit' in d:break
 t=json.loads((R/'jobs/csd-continuous-timeline16/resource-receipt.json').read_text())
 if 'exit' in t:assert t['exit']==0 and t['inputs_sha256']==t['final_inputs_sha256'] and not t.get('resource_stop')
 time.sleep(2)
else:raise RuntimeError('Preview waiting deadline')
assert d['exit']==0 and d['inputs_sha256']==d['final_inputs_sha256'] and not d.get('resource_stop')
paths={Path(__file__),Q/'pictures16.py',Q/'preview16.py'}
for row in json.loads((R/'csd-continuous-preview16-inputs.json').read_text())['files']:
 p=Path(row['path']);assert sha(p)==row['sha256'],p;paths.add(p)
for folder in [R/'jobs/csd-continuous-preview16',Q/'prior-review12']:
 paths.update(p for p in folder.rglob('*') if p.is_file())
env,report=pillow_runtime();paths.update(env)
for tool in ['ffmpeg','ffprobe']:paths.add(Path(shutil.which(tool)).resolve())
paths.add(Path(sys.executable).resolve())
report['files_sha256']={str(p):sha(p) for p in env}
proof=Q/'pictures-runtime16.json'
with proof.open('x') as f:json.dump(report,f,indent=2)
paths.add(proof)
manifest=R/'csd-continuous-pictures16-inputs.json'
with manifest.open('x') as f:json.dump({'files':[{'path':str(p),'sha256':sha(p)} for p in sorted(paths)]},f,indent=2)
assert not (R/'jobs/csd-continuous-pictures16').exists()
subprocess.run(['/usr/bin/python3','-I','-B',str(R/'guard_csd02.py'),'--phase','native','--job','csd-continuous-pictures16','--inputs',str(manifest),'--','/usr/bin/timeout','172900',sys.executable,'-I','-B',str(Q/'pictures16.py'),'--run'],check=True)
print('New picture extraction complete; every resulting sheet still requires actual viewing.')
