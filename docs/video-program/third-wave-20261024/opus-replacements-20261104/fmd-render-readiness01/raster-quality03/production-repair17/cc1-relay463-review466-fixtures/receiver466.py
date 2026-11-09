"""task466: scripted stand-in for the CSD end (ssh + guard + 448 receive). Reads the stream to EOF, then:
complete: places the foreign files (copies), writes import-result.json and a terminal receipt;
fail:     writes a finished receipt with exit 1 and no import-result (what a refused receive leaves);
then exits with the scripted code (255 = the ssh client lost the connection). Fixture only, not 448 receive."""
import datetime
import json
import os
import shutil
import sys
from pathlib import Path

p = json.loads(sys.argv[1])
sys.stdin.buffer.read()
job = Path(p['job'])
r = json.loads((job/'resource-receipt.json').read_text())
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
if p['outcome'] == 'complete':
    for src, dst in p['place']:
        if not os.path.exists(dst):
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copyfile(src, dst)
    with open(job/'import-result.json', 'x') as f:
        json.dump(p['result'], f, indent=1)
    r.update(exit=0, final_inputs_sha256=r['inputs_sha256'], finished=now)
elif p['outcome'] == 'fail':
    r.update(exit=1, final_inputs_sha256=r['inputs_sha256'], finished=now)
if p['outcome'] in ('complete', 'fail'):                  # 'refused': the guard already finished its receipt
    (job/'resource-receipt.json').write_text(json.dumps(r, indent=2))
sys.exit(p['exit'])
