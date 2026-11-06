"""Compare actual completed native schedules; never treats a live job as success."""
from pathlib import Path
import hashlib
import json
import subprocess

Q = Path(__file__).parent/'timeline-path17-comparison'
R = '/data/projects/jmev-stroke-probe-20261106'
F = '/data/projects/jmev-fmd-render-qualification-20261106-1111'
job = R+'/jobs/csd-raster-timeline-path17'
r = json.loads(subprocess.check_output(['ssh','-o','BatchMode=yes','-o','ConnectTimeout=10','csd','cat '+job+'/resource-receipt.json'], timeout=30))
assert r.get('finished') and r.get('exit') == 0, 'native qualification not successfully finished'
assert r['inputs_sha256'] == r['final_inputs_sha256'] and not r.get('resource_stop')
rows = []
for kind in ('timeline', 'cues', 'cuelog'):
    name = '01_the_bump.'+kind+'.tsv'
    target = Q/'csd'/name
    assert not target.exists()
    subprocess.run(['scp','-q','csd:'+job+'/timeline/'+name,str(target)], check=True, timeout=30)
    actual = target.read_bytes()
    expected = (Q/'fmd'/name).read_bytes()
    normalized = actual.replace((R+'/jobs/csd-raster-media15/narration').encode(), (F+'/jobs/fmd-raster-rebind12/narration').encode())
    assert normalized == expected, kind
    rows.append({'table':kind,'csd_sha256':hashlib.sha256(actual).hexdigest(),'fmd_sha256':hashlib.sha256(expected).hexdigest(),'normalized_byte_equal':True})
with (Q/'comparison.json').open('x') as f:
    json.dump({'chapter':'01_the_bump','tables':rows,'csd_resource_receipt':r,'scope':'Actual native 60fps timeline equality after exact narration directory substitution; not final 4K render, soundtrack replay, listening or independent AV acceptance.'},f,indent=2)
print('Three actual native schedule tables match after exact narration-directory substitution.')
