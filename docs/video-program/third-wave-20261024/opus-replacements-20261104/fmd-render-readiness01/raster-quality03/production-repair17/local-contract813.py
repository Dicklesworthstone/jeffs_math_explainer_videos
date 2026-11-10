import subprocess,json,sys
from pathlib import Path
R=Path('/Users/jemanuel/scratch/jmev-complex-native-20261109')
subprocess.run([sys.executable,'-I','-B',str(R/'source760/workspace/tools/narration_delivery.py'),'--topic',str(R/'source760/workspace/explainer_corpus/complex_analysis_visual'),'--narration',str(R/'jobs/mini-boundary-delivery764/narration'),'--consumer','render','--gain-db','0','--chapter-key','09_boundary'],check=True)
