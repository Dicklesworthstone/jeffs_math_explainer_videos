from pathlib import Path
import json,hashlib,subprocess
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027';I=B/'algebra-fixes-inputs02';J=B/'algebra-fixes-score07';N=B/'algebra-fixes-assemble03/narration';T=B/'algebra-fixes-timeline04/timeline'
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
subprocess.run([str(I/'abstract_algebra_expanded'),'score','all','--narration',str(N),'--timelines',str(T),'--out',str(J/'score')],check=True,timeout=7200)
chapters=(I/'native-chapters01.txt').read_text().splitlines();qa=J/'score/stems/qa';assert {p.stem for p in qa.glob('*.json')}==set(chapters);assert all(json.loads((qa/(c+'.json')).read_text())['passed'] is True for c in chapters)
(J/'score07-summary.json').write_text(json.dumps(dict(chapters=chapters,all16_qa_passed=True,scope='Native music QA, not listening or mix acceptance'),indent=2))
