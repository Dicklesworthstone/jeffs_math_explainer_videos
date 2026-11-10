import os,sys
from pathlib import Path
m=Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
os.chdir(m/'repo')
os.execv(sys.argv[1],sys.argv[1:])
