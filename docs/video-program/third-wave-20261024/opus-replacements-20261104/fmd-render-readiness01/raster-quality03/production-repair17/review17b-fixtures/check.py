from pathlib import Path
import hashlib, importlib.util, json, wave, math, cmath
q=Path(__file__).resolve().parent.parent
d=Path(__file__).resolve().parent
root=next(p for p in q.parents if (p/'tools/narration_cache.py').exists())
import os
os.chdir(root)
spec=importlib.util.spec_from_file_location('review17b_coherent',q/'external-unit-check01/tools/coherent_narration.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
assert (q/'coherent_narration.py').read_bytes()==(q/'external-unit-check01/tools/coherent_narration.py').read_bytes()
w=d/'synthetic-silence.wav'
if not w.exists():
 with wave.open(str(w),'wb') as f:
  f.setparams((1,2,24000,0,'NONE','not compressed'));f.writeframes(bytes(480))
receipt=d/'synthetic-native-receipt.json'
if not receipt.exists():receipt.write_text(json.dumps({'wav':m.cache.wav_identity(w),'returncode':0,'request':{},'generation':{},'completion':{}}))
u={'id':'guard','line_ids':['01_1'],'text':'Guard test.','external_ui_provider':'elevenlabs'}
r={'wav':str(w),'generation_receipt':str(receipt)}
try:m.unit_spans(u,{'01_1':'Guard test.'},r)
except ValueError as e:
 assert str(e)=='declared external UI unit requires a verified external UI receipt';negative=str(e)
else:raise AssertionError('native receipt accepted')
# Real retained external take: first cue from the earlier assembly, unchanged.
u=json.loads((q/'assembly-inputs16/external-units.json').read_text())['units'][0]
u['external_ui_provider']='elevenlabs'
r=json.loads((q/'assembly-inputs16/assembly.json').read_text())['manifest']['units'][0]
try:
 rows=m.unit_spans(u,{u['line_ids'][0]:u['text']},r)
 positive={'status':'PASS','cue':u['line_ids'][0],'sentence_spans':len(rows),'samples':sum(x[1]['out_samples'] for x in rows),'receipt':r['generation_receipt'],'receipt_sha256':m.cache.digest(r['generation_receipt'])}
except (ValueError,FileNotFoundError) as e:positive={'status':'NOT QUALIFIED','reason':str(e)}
# Independent arithmetic mirror of the provisional ring construction.
triangles=0
for n in (8,16,32,64):
 o=[cmath.rect(.5,math.tau*k/n) for k in range(n)];i=[cmath.rect(.18,math.tau*k/n) for k in range(n)];edges={}
 for k in range(n):
  for t in ((i[k],o[k],o[(k+1)%n]),(i[k],o[(k+1)%n],i[(k+1)%n])):
   assert ((t[1]-t[0]).conjugate()*(t[2]-t[0])).imag>0
   triangles+=1
   for a,b in zip(t,t[1:]+t[:1]):edges[(a,b)]=edges.get((a,b),0)+1
 for (a,b),count in edges.items():
  if abs(abs(a)-abs(b))>1e-9:assert edges.get((b,a))==count
 assert .18*math.cos(math.pi/n)>0
result={'source_sha256':m.cache.digest(q/'coherent_narration.py'),'native_receipt_rejected':negative,'real_external_receipt':positive,'ring_triangle_mirror':{'counterclockwise_triangles':triangles,'interior_edge_reverse_pairs':'PASS','nonzero_inner_clearance':'PASS'},'scope':'Branch validation and arithmetic mirror only. Synthetic negative fixture is not provider evidence; no assembly artifact, rendering, synthesis or listening.'}
(d/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
