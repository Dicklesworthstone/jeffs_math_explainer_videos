from pathlib import Path
import importlib.util,hashlib
P=Path(__file__).with_name('guard_owner80g_target20g.py')
assert hashlib.sha256(P.read_bytes()).hexdigest()=='4f34e0d1e9a8f9d164860828f87a734e99b697d2d3048c7b65508e5d84b8b4ad'
s=importlib.util.spec_from_file_location('existing_guard',P);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);g=m.load()
assert g.ADMISSION_GIB['native']==10 and g.STOP_FREE_GIB==8 and g.CAPS['native']==4 and g.WORK_CAP_GIB==80
g.ADMISSION_GIB['native']=20
if __name__=='__main__':raise SystemExit(g.main())
