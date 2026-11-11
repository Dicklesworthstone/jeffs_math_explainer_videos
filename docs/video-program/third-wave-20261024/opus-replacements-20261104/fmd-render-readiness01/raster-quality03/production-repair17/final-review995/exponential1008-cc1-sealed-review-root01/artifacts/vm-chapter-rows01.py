"""Read-only, run on CSS via ssh stdin: print the chapter movie and pictures985 seed rows bound by the verify1005 inputs."""
import json

m = json.load(open('/data/projects/jmev-css-render-worker-20261107/qualification01/exponential933/css-exponential-verify1005-inputs.json'))
for x in m['files']:
    p = x['path']
    if p.endswith('.mp4') or 'pictures985.json' in p:
        print(x['sha256'], p)
