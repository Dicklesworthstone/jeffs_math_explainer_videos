from pathlib import Path
import json,subprocess
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos');Q=R/'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17';A=Q/'algebra-fixes';B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
chapters=['11_permutations','14_cosets']+[p.stem.replace('.timeline','') for p in sorted((B/'algebra-fixes-timeline04/timeline').glob('*.timeline.tsv')) if not p.name.startswith(('11_','14_'))]
assert len(chapters)==16 and len(set(chapters))==16
for c in chapters:
 job='algebra-final11-'+c;assert not (B/job).exists()
 cmd=['python3','-I','-B',str(Q/'guard_owner80g_target20g.py'),'--phase','native','--job',job,'--inputs',str(A/'final11-inputs.json'),'--','python3','-I','-B',str(A/'final-chapter11.py'),c]
 with (A/(job+'-controller.log')).open('x') as log:subprocess.run(cmd,cwd=R,stdout=log,stderr=subprocess.STDOUT,check=True)
 a=json.loads((B/job/'resource-receipt.json').read_text());assert a['exit']==0 and a['inputs_sha256']==a['final_inputs_sha256'] and not a.get('resource_stop')
 print('completed',c,flush=True)
