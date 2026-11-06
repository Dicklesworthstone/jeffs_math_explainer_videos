"""Replay for the staged 09_5 repair. From the repository root: python3 -I -B <this file>.

Reads the live stage06 files and writes only to a new temporary folder, which it leaves in place. Prints nothing
that varies between runs. The tools are the unmodified stage06 copies, run against mirror trees.
"""
import cmath
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import shutil
import sys
import tempfile

BUNDLE = Path(__file__).resolve().parents[1]
EC = Path('narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104')
WORK = EC / 'native-stage06-root03/workspace'
LIVE = WORK / 'explainer_corpus/complex_analysis_visual'
STAGED = BUNDLE / 'staged/explainer_corpus/complex_analysis_visual'
N = EC / 'production-audio-stage06-01/native-narration01'
BIND = json.loads((BUNDLE / 'bindings.json').read_text())
OLD_MAP = 'docs/narration-direction-map-frederick-stage06.json'
NEW_MAP = 'docs/narration-direction-map-frederick-stage07.json'
PAIRS = {'script.tsv': ('script.tsv', 1), 'src/chapters/boundary.rs': ('src/chapters/boundary.rs', 1),
         'docs/storyboard.md': ('docs/storyboard.md', 1), OLD_MAP: (NEW_MAP, 6),
         'narration_units.json': ('narration_units.json', 6)}
LAST = 'For this formula, a sufficient condition is that the function is holomorphic throughout the disk and a little beyond its edge.'
UNSPOKEN = 'an open set containing the closed disk'
failed = 0


def report(good, name, detail=''):
    global failed
    failed += not good
    print(f"{'PASS' if good else 'FAIL'} {name}{': ' + detail if detail else ''}", flush=True)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def cue_text(root, cue):
    rows = [line.split('\t', 1)[1] for line in (root / 'script.tsv').read_text().splitlines() if line.startswith(cue + '\t')]
    assert len(rows) == 1, cue
    return rows[0]


def card(root):
    return re.search(r'pub const NEEDS: &str =\n    "([^"]+)";', (root / 'src/chapters/boundary.rs').read_text()).group(1)


# 1. Bindings
olds = {'script.tsv': 'script.tsv', 'src/chapters/boundary.rs': 'src/chapters/boundary.rs',
        'docs/storyboard.md': 'docs/storyboard.md', 'direction map': OLD_MAP, 'narration_units.json': 'narration_units.json'}
news = {**olds, 'direction map': NEW_MAP}
report(all(sha((LIVE / olds[key]).read_bytes()) == row['old'] for key, row in BIND['files'].items()), 'live inputs still match the old hashes')
report(all(sha((STAGED / news[key]).read_bytes()) == row['new'] for key, row in BIND['files'].items()), 'staged files match the new hashes')

# 2. Exact change
counts = {}
for before, (after, wanted) in PAIRS.items():
    old_lines, new_lines = (LIVE / before).read_text().split('\n'), (STAGED / after).read_text().split('\n')
    counts[before] = (len(old_lines) == len(new_lines), sum(a != b for a, b in zip(old_lines, new_lines)), wanted)
report(all(same and got == wanted for same, got, wanted in counts.values()), 'line counts unchanged, changed lines exactly as listed',
       ', '.join(f'{Path(name).name} {got}' for name, (_, got, _) in counts.items()))
old_cue, new_cue = cue_text(LIVE, '09_5'), cue_text(STAGED, '09_5')
keep = old_cue[:old_cue.index('The formula only holds')]
report(new_cue == keep + LAST, 'cue is the first three sentences unchanged plus root\'s exact last sentence')
report(sha(old_cue.encode()) == BIND['old_cue_sha256'] and sha(new_cue.encode()) == BIND['new_cue_sha256'], 'cue text hashes')
report(not re.search(r'[:;()\d—–-]', new_cue) and max(len(s.split()) for s in new_cue.split('. ')) <= 28, 'voice rules: no colon, semicolon, bracket, digit or dash; sentences at most 28 words')

# 3. Card and storyboard
old_card, new_card = card(LIVE), card(STAGED)
report('needs' not in new_card and 'holds if' in new_card and UNSPOKEN in new_card, 'card states sufficiency and keeps the test\'s unspoken clause', new_card)
report(set(new_card) <= set(old_card), 'card adds no new character, so no new glyph')
report(max(len(line) for line in (STAGED / 'src/chapters/boundary.rs').read_text().splitlines()) <= 120, 'boundary.rs stays within max_width 120')
row = [line for line in (STAGED / 'docs/storyboard.md').read_text().splitlines() if line.startswith('| 09_5 |')]
report(len(row) == 1 and row[0].split(' | ')[1] == new_cue and f'"precisely: {new_card}"' in row[0], 'storyboard row carries the new cue and card')


def wrap(text, width):
    lines = []
    for word in text.split():
        if lines and len(lines[-1]) + 1 + len(word) <= width:
            lines[-1] += ' ' + word
        else:
            lines.append(word)
    return lines


report(len(wrap(old_card, 92)) == len(wrap(new_card, 92)) == 2, 'card wraps to two lines before and after (animatic::wrap mirror at 92)', ' / '.join(wrap(new_card, 92)))
for width in (88, 84, 80, 76, 72, 68, 64, 60):
    assert len(wrap(new_card, width)) == 2, width
report(True, 'card stays two lines down to the wrap loop\'s step at 60')
report(len(wrap(old_card, 92)) == 2 and 0.6 + 3.0 >= len(UNSPOKEN) / 15 and 0.6 + 3.0 <= 2 * len(new_card) / 15, 'reading window inequalities hold unchanged', f'3.60 s against {len(UNSPOKEN) / 15:.2f} and {2 * len(new_card) / 15:.2f}')
caption_old, caption_new = wrap(old_card and old_cue[len(keep):], 42), wrap(LAST, 42)
report(max(map(len, caption_new)) <= 42, 'caption wrap mirror at 42', f'old {len(caption_old)} lines, new {len(caption_new)} lines, longest {max(map(len, caption_new))}')

# 4. Scene and anchors
scene = (STAGED / 'src/chapters/boundary.rs').read_text()
scene = scene[scene.index('say(&kit, stage, "09_5")'):scene.index('say(&kit, stage, "09_6")')]
indices = sorted(int(i) for i in re.findall(r'wait_sentence\(stage, (\d+)\)', scene))
phrases = json.loads((N / 'phrase-timings.json').read_text())['cues']
report(indices == [1, 2, 3] and 'wait_for' not in scene and phrases['09_5']['marks'] == [], '09_5 uses sentences 1 to 3 and no phrase anchor or mark', str(indices))
all_scene = (LIVE / 'src/chapters/boundary.rs').read_text()
report(not any(p in all_scene for p in ('only holds', 'out to the circle')), 'no scene hook names a removed phrase')
requests = [json.loads((EC / name).read_text()) for name in ('frederick-stage05-01/09_4-request01.json', 'frederick-stage06-01/09_6-request01.json')]
report(all(not any('context' in key for key in request) for request in requests), '09_4 and 09_6 requests carry no following or previous context')
report(cue_text(LIVE, '09_4') == cue_text(STAGED, '09_4') and cue_text(LIVE, '09_6') == cue_text(STAGED, '09_6'), 'neighbouring cues unchanged')

# 5. Unmodified tools on mirrors
TMP = Path(tempfile.mkdtemp(prefix='cauchy-repair-'))
for name in ('coherent_narration.py', 'narration_cache.py'):
    (TMP / 'tools').mkdir(exist_ok=True)
    shutil.copyfile(WORK / 'tools' / name, TMP / 'tools' / name)
sys.path.insert(0, str(TMP / 'tools'))
spec = importlib.util.spec_from_file_location('coherent_narration', TMP / 'tools/coherent_narration.py')
TOOL = importlib.util.module_from_spec(spec)
spec.loader.exec_module(TOOL)
TREE = 'explainer_corpus/complex_analysis_visual'
serial = iter(range(1, 100))


def run(script, units, maps):
    """A fresh repository mirror each time, because a map path may differ."""
    root = TMP / f'case{next(serial):02}'
    (root / TREE / 'docs').mkdir(parents=True)
    (root / TREE / 'script.tsv').write_bytes(script)
    (root / TREE / 'narration_units.json').write_bytes(units)
    for name, data in maps.items():
        (root / TREE / name).write_bytes(data)
    TOOL.REPO = root
    try:
        _, rows = TOOL.read_units(root / TREE / 'script.tsv', root / TREE / 'narration_units.json', True)
        contract = TOOL.direction_contract(root / TREE / 'script.tsv', json.loads((root / TREE / 'narration_units.json').read_text()))
        return len(rows), contract
    except ValueError as error:
        return str(error), None


live_script, live_units, live_map = (LIVE / 'script.tsv').read_bytes(), (LIVE / 'narration_units.json').read_bytes(), (LIVE / OLD_MAP).read_bytes()
new_script, new_units, new_map = (STAGED / 'script.tsv').read_bytes(), (STAGED / 'narration_units.json').read_bytes(), (STAGED / NEW_MAP).read_bytes()
count, _ = run(live_script, live_units, {OLD_MAP: live_map})
report(count == 105, 'control: live stage06 script, units and map pass the unmodified read_units', str(count))
count, contract = run(new_script, new_units, {NEW_MAP: new_map})
ending = [row for row in contract['resolved'] if row['cue'] == '09_5'] if contract else []
report(count == 105 and len(contract['resolved']) == 23 and len(ending) == 1 and ending[0]['end_word_index'] == len(TOOL.direction_words(new_cue)) - 1,
       'staged script, units and map pass; 23 directions; 09_5 direction ends on the cue\'s last word')
message, _ = run(new_script, live_units, {OLD_MAP: live_map})
report(message == 'unit plan does not bind current script bytes', 'refused: new script with stage06 units', str(message))
message, _ = run(new_script, new_units, {NEW_MAP: new_map + b' '})
report(message == 'direction map changed', 'refused: staged map changed by one byte', str(message))
stale_map = new_map.replace(new_cue.encode(), old_cue.encode())
stale_units = new_units.replace(sha(new_map).encode(), sha(stale_map).encode())
message, _ = run(new_script, stale_units, {NEW_MAP: stale_map})
report(message == 'direction phrase is missing or ambiguous', 'refused: map that still names the old cue', str(message))
stale_text = new_units.replace(new_cue.encode(), old_cue.encode())
message, _ = run(new_script, stale_text, {NEW_MAP: new_map})
report(message == 'unit oliver_09_5 must contain exact script text, at most 100 words', 'refused: units that still hold the old cue', str(message))
old_rows = json.loads((N / 'report.json').read_text())['lines']['09_5']
report(TOOL.sentences(old_cue) == [row['sentence'] for row in old_rows] and len(TOOL.sentences(new_cue)) == 4 and TOOL.sentences(new_cue)[:3] == TOOL.sentences(old_cue)[:3],
       'tool sentence split: four before and after, first three identical', f'sentence ends {[row["source_out_sample"] for row in old_rows]}')

# 6. The mathematics at the edge
samples = 4096
circle = [cmath.exp(2j * math.pi * k / samples) for k in range(samples)]
witness = sum(cmath.sqrt(1 - z) for z in circle) / samples
report(abs(witness - 1) < 1e-4, 'witness against necessity: sqrt(1 - z) has no holomorphic extension past z = 1, yet its circle average equals its centre value', f'average {witness.real:.5f}, centre 1')
bump = sum(1 / (1 + (2 * z) ** 2) for z in circle) / samples
report(abs(bump) < 1e-9, 'control: bump on radius 2 still averages 0, not 1', f'{abs(bump):.1e}')

# 7. Bindings for the next stage
listing = ''.join(f'{p.name} {sha(p.read_bytes())}\n' for p in sorted((LIVE / 'src/chapters').glob('*.rs')))
staged_listing = listing.replace(BIND['files']['src/chapters/boundary.rs']['old'], BIND['files']['src/chapters/boundary.rs']['new'])
report(sha(listing.encode()) == BIND['chapter_sources_sha256']['old'] and sha(staged_listing.encode()) == BIND['chapter_sources_sha256']['new'], 'chapter-source listing hash, old and new, by the bridge\'s rule')
request = json.loads((BUNDLE / '09_5-request-candidate.json').read_text())
live_request = json.loads((EC / 'frederick-stage06-01/09_5-request02.json').read_text())
same = all(request[key] == live_request[key] for key in ('voice', 'model', 'stability', 'similarity', 'clone_strength', 'output'))
report(same and request['text'] == new_cue and request['text_sha256'] == BIND['new_cue_sha256'] and request['script_sha256'] == BIND['files']['script.tsv']['new'] and request['submitted'] is False,
       'request record has the new text and script hash, the live settings, and is unsubmitted')
actual_path = EC / 'remote-ts1-01/cauchy-cue-repair01/request01.json'
actual = json.loads(actual_path.read_text())
same = all(actual[key] == request[key] for key in ('cue', 'text', 'text_sha256', 'voice', 'model', 'stability', 'similarity', 'clone_strength', 'output'))
report(same and sha(actual_path.read_bytes()) == 'e9b0aef22537f507fcf968b42065c442f536cd82e0c10a76344c9771e2f9a84a', "root's submitted request has this exact cue and these settings")
print(f'RESULT {"PASS" if not failed else "FAIL"}, {failed} failed')
sys.exit(1 if failed else 0)
