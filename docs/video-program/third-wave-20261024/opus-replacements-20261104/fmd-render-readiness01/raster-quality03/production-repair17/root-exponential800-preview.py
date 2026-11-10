"""Native 1080p sampled visual draft; scratch timing, no narration or motion acceptance."""
from pathlib import Path
import hashlib,json,os,subprocess
R=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
W=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
B=W/'exponential-local-build799'
O=W/'exponential-native-preview800'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
review=R/'narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/expanded-stage13/glyph-review799.json'
assert sha(review)=='25cfc45c2e97c330bbea4611a705ff5693e5434e04f90a0aa746f14d93f5f362'
assert json.loads(review.read_text())['qualification_sha256']==sha(B/'qualification.json')
r=json.loads((B/'resource-receipt.json').read_text())
assert r['exit']==0 and r['finished'] and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
q=json.loads((B/'qualification.json').read_text())
assert q['tests_passed']==218 and q['original_tests_passed']==95 and q['clippy_passed']
assert q['facts']==1771 and len(q['chapters'])==15 and len(q['cues'])==94 and q['cues_said_in_script_order']
assert q['source_tree_sha256']=='b82fcd2de6dec554c3a2c3970947f4b0f71ce6a5c1f10f9afa8219ba9f1a071a'
b=Path(q['binary']);assert sha(b)==q['binary_sha256']
f=Path(q['ffmpeg']['path']);assert sha(f)==q['ffmpeg']['sha256']
out=O/'render';assert not out.exists()
cmd=[str(b),'render','all','--scratch-voice','--res','1920x1080','--fps','2','--threads','4','--lint','--crf','16','--preset','veryfast','--out',str(out)]
subprocess.run(cmd,check=True,timeout=10800,env=dict(os.environ,PATH=str(f.parent)+':/usr/bin:/bin'))
assert sorted(p.stem for p in out.glob('*.mp4'))==sorted(q['chapters'])
with (O/'preview-receipt.json').open('x') as h:
 json.dump(dict(binary_sha256=sha(b),build_qualification_sha256=sha(B/'qualification.json'),chapters=q['chapters'],command=cmd,scope='1080p two frames per second native sampled picture draft. Estimated timing, no audio. Not final motion, narration, synchronization or publication acceptance.'),h,indent=2)
