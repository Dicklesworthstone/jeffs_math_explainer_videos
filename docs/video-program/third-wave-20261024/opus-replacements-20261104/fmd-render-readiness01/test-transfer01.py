"""Synthetic bounded transport tests; not media or release acceptance."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('receiver',HERE/'receive-fmd-final01.py')
R=importlib.util.module_from_spec(spec);spec.loader.exec_module(R)
OUT=HERE/'transfer-tests01'
OUT.mkdir(exist_ok=False)
payloads=[b'' if i%7==0 else ('fixture-'+str(i)).encode() for i in range(72)]
rows=[dict(source=str(R.REMOTE/'fixture'/str(i)),destination='fixture/'+str(i),
           bytes=len(data),sha256=hashlib.sha256(data).hexdigest()) for i,data in enumerate(payloads)]
raw=b''.join(payloads)
result=R.receive(io.BytesIO(raw),OUT/'positive',rows)
assert len(result)==72
assert all((OUT/'positive'/row['destination']).read_bytes()==data for row,data in zip(rows,payloads))
rejected=[]
def negative(name,records,data):
    try:R.receive(io.BytesIO(data),OUT/name,records)
    except (AssertionError,FileExistsError):rejected.append(name);return
    raise AssertionError('Bad stream accepted: '+name)
negative('truncated',rows,raw[:-1])
negative('trailing',rows,raw+b'!')
negative('corrupt',rows,b'X'+raw[1:])
bad=copy.deepcopy(rows);bad[1]['destination']=bad[0]['destination'];negative('duplicate',bad,raw)
bad=copy.deepcopy(rows);bad[0]['destination']='../escape';negative('escape',bad,raw)
bad=copy.deepcopy(rows);bad[0]['source']=str(R.REMOTE/'../escape');negative('source-escape',bad,raw)
bad=copy.deepcopy(rows);bad[0]['bytes']=5*1024**3;negative('oversized',bad,raw)
(OUT/'existing-output').mkdir()
negative('existing-output',rows,raw)
report=dict(synthetic=True,files=72,zero_byte_files=sum(not x for x in payloads),
            bytes=len(raw),negative_cases=rejected,
            scope='Transport mechanics only. Real guarded transfer, original media checks and master replay not yet run.')
with (OUT/'result.json').open('x') as f:json.dump(report,f,indent=2)
print(json.dumps(report))
