from pathlib import Path
import json,hashlib,subprocess
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';I=B/'algebra-fixes-inputs02';J=B/'algebra-fixes-timeline04';N=B/'algebra-fixes-assemble03/narration'
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
chapters=(I/'native-chapters01.txt').read_text().splitlines();assert len(chapters)==16
subprocess.run([str(I/'abstract_algebra_expanded'),'render','all','--timeline-only','--fps','60','--threads','4','--narration',str(N),'--out',str(J/'timeline')],check=True,timeout=14400)
assert {p.name for p in (J/'timeline').glob('*.timeline.tsv')}=={c+'.timeline.tsv' for c in chapters}
rows=[]
for c in chapters:
 p=J/'timeline'/(c+'.timeline.tsv');lines=[l.split('\t') for l in p.read_text().splitlines()];frames=[int(l[1]) for l in lines if l[0]=='frames'];assert len(frames)==1;rows.append(dict(chapter=c,frames=frames[0],sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(J/'timeline04-summary.json').write_text(json.dumps(dict(chapters=rows,total_frames=sum(x['frames'] for x in rows),scope='Native schedules only; no new picture, listening or release acceptance'),indent=2))
