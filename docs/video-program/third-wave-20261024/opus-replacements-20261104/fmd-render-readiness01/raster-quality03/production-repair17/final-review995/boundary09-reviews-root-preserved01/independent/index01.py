"""index01.py: CC0's sheet index for the repaired 09_boundary preview pictures (scope02 extraction). JSON only: it
reads the extraction's own pictures.json and the existing sampling-plan02 / prior775 records, and writes a table of
every sheet slot with the plan02 window ids and expectation it serves. It opens no picture and decodes nothing.

Also reports: plan02 09 frames (required and extended) and prior775 frames missing from the actual selection;
sheet slot ranges against the selection order; the four root 343 instants (local 1661, 1914, 2131, 2347 +-2).

usage: python3 -I -B index01.py PICTURES_JSON SAMPLING_PLAN02 PRIOR775 OUT_DIR
"""
import hashlib
import json
import sys
from pathlib import Path

PICS, PLAN, PRIOR, OUT = (Path(x) for x in sys.argv[1:5])


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


r = read(PICS)
plan = read(PLAN)
prior = set(read(PRIOR)['selected_frames'])
frames = r['selected_frames']
assert frames == sorted(set(frames)) and len(frames) == len(r['pictures'])
windows = {w['id']: w for w in plan['windows'] if w.get('chapter') == '09_boundary'}
tier = {}
wins = {}
for x in plan['samples']:
    if x.get('chapter') == '09_boundary':
        tier[x['local_frame']] = x['tier']
        wins[x['local_frame']] = x['windows']
anchors = {}
for a in r['layout_anchors']:
    anchors.setdefault(a['frame'], []).append(a['row'].split('\t', 1)[0])
INSTANTS = (1661, 1914, 2131, 2347)
out = []
say = out.append
say('pictures.json %s; movie %s; frames %d; full_decode %s' % (sha(PICS)[:8], r['movie_sha256'][:8], r['frames'], r['full_decode']))
say('selected %d; sheets %d; plan02 09 samples %d (required %d); prior775 %d; layout anchors %d'
    % (len(frames), len(r['sheets']), len(tier), sum(t == 'required' for t in tier.values()), len(prior), len(r['layout_anchors'])))
miss_req = sorted(f for f, t in tier.items() if t == 'required' and f not in set(frames))
miss_ext = sorted(f for f, t in tier.items() if t != 'required' and f not in set(frames))
miss_prior = sorted(prior - set(frames))
say('missing from the selection: plan02 required %d %s; extended %d %s; prior775 %d %s'
    % (len(miss_req), miss_req[:20], len(miss_ext), miss_ext[:20], len(miss_prior), miss_prior[:20]))
for c in INSTANTS:
    have = [f for f in range(c-2, c+3) if f in set(frames)]
    say('root 343 instant centre %d: selected %s' % (c, have))
ok_sheets = all(s['first_picture'] == 8*i+1 and s['last_picture'] == min(8*i+8, len(frames)) for i, s in enumerate(r['sheets']))
say('sheet slot ranges follow the selection order in eights: %s' % ok_sheets)
rows = ['sheet\tslot\tindex\tlocal_frame\tseconds\ttier\twindows\tprior775\tlayout_anchor']
for i, s in enumerate(r['sheets']):
    for k in range(s['first_picture'], s['last_picture']+1):
        f = frames[k-1]
        rows.append('%03d\t%d\t%d\t%d\t%.3f\t%s\t%s\t%s\t%s' % (i+1, k-s['first_picture']+1, k, f, f/60, tier.get(f, ''),
                                                               ','.join(wins.get(f, [])), 'yes' if f in prior else '',
                                                               '; '.join(anchors.get(f, []))))
with open(OUT/'index01.tsv', 'x', encoding='utf-8') as stream:
    stream.write('\n'.join(rows)+'\n')
by_window = {}
for i, s in enumerate(r['sheets']):
    for k in range(s['first_picture'], s['last_picture']+1):
        for w in wins.get(frames[k-1], []):
            by_window.setdefault(w, set()).add(i+1)
for w in windows:
    sh = sorted(by_window.get(w, []))
    say('window %s: sheets %s; expect: %s' % (w, '%d-%d (%d)' % (sh[0], sh[-1], len(sh)) if sh else 'none', windows[w]['expect']))
with open(OUT/'index01.out', 'x', encoding='utf-8') as stream:
    stream.write('\n'.join(out)+'\n')
print('\n'.join(out))
