"""Local, read-only cross-check of root's publication976 *1010 files against each other, the files they name,
verification1005 and the actual pictures1008 chapter records (offsets/frames from the verify1005-bound identity).
Mechanical only: hashes, chapter arithmetic, chapter block text, counts, lengths, forbidden characters.
usage: python3 -I -B check-publication1010-01.py P808 VERIFICATION1005_JSON RECORDS1008_JSON"""
import hashlib
import json
import sys
from pathlib import Path

P, V, R = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
U = P / 'publication976'
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
bad = 0


def check(ok, what):
    global bad
    bad += not ok
    print('PASS' if ok else 'FAIL', what)


ct = json.loads((U / 'chapter-times1010.json').read_text())
up = json.loads((U / 'upload-preparation1010.json').read_text())
ver = json.loads(V.read_text())
recs = json.loads(R.read_text())['chapters']
desc = (U / 'description-reviewed1010.txt').read_text()
title = (U / 'title-reviewed1010.txt').read_text()
tags = (U / 'tags-reviewed1010.txt').read_text()

# Hash bindings.
for name, a in up['assets'].items():
    if name == 'master':
        continue
    check(sha(a['path']) == a['sha256'], f'upload-preparation {name} sha equals the file ({a["sha256"][:8]})')
check(ct['description_sha256'] == up['assets']['description']['sha256'], 'chapter-times1010 binds the same description as upload-preparation1010')
m = up['assets']['master']
check(m['sha256'] == ver['movie_sha256'] == ct['master_sha256'], 'master sha: upload-preparation = verification1005 movie = chapter-times1010')
check(sha(m['verification_path']) == m['verification_sha256'] == ct['verification_sha256'] == sha(V),
      'verification path hashes to the bound verification sha in both files')
local = P / 'exponential-master987.mp4'
tr = json.loads((P / 'local-master987-transfer-receipt.json').read_text())
check(tr['sha256'] == m['sha256'] and Path(tr['path']) == local.resolve(), 'local transfer receipt names P808/exponential-master987.mp4 with the bound master sha (file not re-hashed here)')
check(up['assets']['captions_srt']['sha256'] == ver['delivery_srt_sha256'] and up['assets']['captions_vtt']['sha256'] == ver['delivery_vtt_sha256'], 'captions are verify1005 delivery pins')
check(up['caption_count'] == int(ver['captions']), f'caption_count {up["caption_count"]} = verification captions {ver["captions"]}')
check(up['frames'] == ct['total_frames'] == int(ver['frames']), 'frames agree (upload-preparation, chapter-times, verification)')
check(abs(up['duration_seconds'] - up['frames'] / up['fps']) < 1e-9 and up['fps'] == ct['fps'] == 60, 'duration = frames / fps, fps 60')

# Chapter arithmetic against the actual chapter records.
start, lines = 0, []
for c in ct['chapters']:
    r = recs[c['key']]['record']['extract_source']
    ok = c['start_frame'] == start == r['offset_frames'] and c['frames'] == r['frames'] and abs(c['start_seconds'] - start / 60) < 1e-9
    s = int(c['start_frame'] // 60)
    disp = f'{s // 60}:{s % 60:02d}'
    check(ok and c['display'] == disp, f"{c['key']} start {c['start_frame']} frames {c['frames']} display {c['display']} (records offset {r['offset_frames']} frames {r['frames']})")
    lines.append(f"{c['display']} {c['label']}")
    start += c['frames']
check(start == ct['total_frames'], f'chapter frames sum {start} = total {ct["total_frames"]}')
block = desc.split('Chapters:\n', 1)[1].split('\n\n', 1)[0].splitlines()
check(block == lines, 'description chapter block equals chapter-times1010 display + label, line for line')
check(desc.count('Chapters:') == 1, 'one chapter block')

# Format.
for name, text, limit in [('title', title.strip(), 100), ('description', desc, 5000)]:
    check(len(text) <= limit and '<' not in text and '>' not in text, f'{name}: {len(text)} characters (limit {limit}), no angle brackets')
tag_list = [t.strip() for t in tags.strip().split(',')]
eff = sum(len(t) + (2 if ' ' in t else 0) for t in tag_list) + len(tag_list) - 1
check(eff <= 500 and '<' not in tags and '>' not in tags, f'tags: {len(tag_list)} tags, about {eff} counted characters with quotes for spaced tags (limit 500)')
check(len(set(t.lower() for t in tag_list)) == len(tag_list), 'no duplicate tags')
print('failures', bad)
raise SystemExit(1 if bad else 0)
