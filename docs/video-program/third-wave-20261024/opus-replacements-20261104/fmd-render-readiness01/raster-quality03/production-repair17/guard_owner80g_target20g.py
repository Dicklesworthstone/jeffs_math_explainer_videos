"""Owner-authorized work-cap expansion; all other resource and input guards retained.
Existing active jobs keep their original policy; this wrapper is for later jobs only.
"""
from pathlib import Path
import hashlib,importlib.util
SOURCE=Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py')
SOURCE_SHA='1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097'
def load():
 assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()==SOURCE_SHA
 spec=importlib.util.spec_from_file_location('prior_owner_guard',SOURCE);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 guard=module.load()
 assert guard.WORK_CAP_GIB==56 and guard.ADMISSION_GIB['native']==10 and guard.STOP_FREE_GIB==8 and guard.CAPS['native']==4 and guard.TARGET_GROWTH_CAP_GIB==20
 guard.WORK_CAP_GIB=80
 return guard
if __name__=='__main__':raise SystemExit(load().main())
