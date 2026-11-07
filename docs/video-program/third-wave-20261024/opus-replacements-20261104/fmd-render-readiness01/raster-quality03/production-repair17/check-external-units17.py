"""Exercise provider-specific size checks without weakening native synthesis limits."""
from pathlib import Path
import hashlib
import importlib.util
import json
import shutil
from types import SimpleNamespace

q = Path(__file__).resolve().parent
root = next(p for p in q.parents if (p / 'tools/narration_cache.py').exists())
work = q / 'external-unit-check01'
work.mkdir()
tools = work / 'tools'
tools.mkdir()
shutil.copy2(q / 'coherent_narration.py', tools / 'coherent_narration.py')
shutil.copy2(root / 'tools/narration_cache.py', tools / 'narration_cache.py')
spec = importlib.util.spec_from_file_location('coherent17', tools / 'coherent_narration.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
passed = []

def case(name, text, provider=None, second=False, rejects=False):
    script = work / (name + '.tsv')
    script.write_text('01_1\t' + text + '\n' + ('01_2\tSecond cue.\n' if second else ''))
    unit = dict(id='unit01', line_ids=['01_1'] + (['01_2'] if second else []),
                text=text + (' Second cue.' if second else ''))
    if provider is not None:
        unit['external_ui_provider'] = provider
    plan = work / (name + '.json')
    plan.write_text(json.dumps(dict(schema_version=1, script_sha256=hashlib.sha256(script.read_bytes()).hexdigest(), units=[unit])))
    try:
        module.read_units(script, plan)
    except ValueError:
        assert rejects, name
    else:
        assert not rejects, name
    passed.append(name)
    return script, plan

case('native100', 'word ' * 99 + 'last.')
case('native101-refused', 'word ' * 100 + 'last.', rejects=True)
text = (q / '08_5-script.txt').read_text().strip().split('\t', 1)[1]
script, plan = case('actual-reviewed08', text, 'elevenlabs')
case('provider10000', 'x' * 10000, 'elevenlabs')
case('provider10001-refused', 'x' * 10001, 'elevenlabs', rejects=True)
case('unknown-provider-refused', text, 'other', rejects=True)
case('multiple-cues-refused', 'A cue.', 'elevenlabs', second=True, rejects=True)
try:
    module.generate(SimpleNamespace(script=script, units=plan, require_directions=False))
except ValueError as error:
    assert 'not native synthesis' in str(error)
else:
    raise AssertionError('external units reached native synthesis')
passed.append('native-synthesis-refused')
(work / 'result.json').write_text(json.dumps({'passed': passed, 'scope': 'Input validation only; not assembly, audio or release approval.'}, indent=2))
print(json.dumps({'passed': len(passed), 'cases': passed}))
