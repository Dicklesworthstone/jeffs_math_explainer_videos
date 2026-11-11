"""Write evidence.json for the CC3 author review of the 09 scope02 extraction.

usage: python3 -I -B make-evidence.py PICTURES_JSON INDEX_TSV OUT.json
Reads pictures.json and index.tsv; checks that the selected frames and the index frames agree and that the movie hash
is the reviewed one; writes a new OUT.json (refuses an existing one). viewed_frame_indices = every selected frame,
because every selected picture was seen on its contact sheet.
"""
import csv
import hashlib
import json
import sys
from pathlib import Path

MOVIE = 'da4df9de2a8271e8b9d27740a4943b1e789df96a5efbc5f178c8897bd9f05f74'
SOURCE = '4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932'

pj, idx, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
raw = pj.read_bytes()
d = json.loads(raw)
frames = d['selected_frames']
with idx.open(newline='') as f:
    index_frames = sorted(int(r['frame']) for r in csv.DictReader(f, delimiter='\t'))
if d['movie_sha256'] != MOVIE:
    sys.exit(f'movie hash {d["movie_sha256"]} is not the reviewed movie')
if frames != sorted(set(frames)) or frames != index_frames or len(frames) != len(d['pictures']):
    sys.exit('selected frames, index frames and pictures disagree')

evidence = {
    'chapter': '09_boundary',
    'reviewer': 'WhiteSwan (CC3)',
    'review_kind': 'author',
    'independent_review': False,
    'authored_repair': True,
    'authored_repair_note': ('Source 4b689791 is candidate03 over base cd1900aa, which is CC3 repair752 (09_1 inner '
                             'heads in yellow with a ground rim, INNER_HEAD 0.16, p at (-0.245, 0)). CC3 did not '
                             'author or review candidate03.'),
    'movie': 'preview/09_boundary.mp4',
    'movie_sha256': MOVIE,
    'extraction_sha256': hashlib.sha256(raw).hexdigest(),
    'index_sha256': hashlib.sha256(idx.read_bytes()).hexdigest(),
    'source_sha256': SOURCE,
    'evidence_types': [
        'all 189 contact sheets read as images: 640x360 tiles of 960x540 preview pictures',
        'single 960x540 pictures 208, 224, 256, 340, 384, 440, 888, 938',
        'nearest-neighbour enlarged crops and same-region strips in crops/',
        'on-screen numbers recomputed in plain Python',
    ],
    'not_claimed': [
        '4K', 'motion or playback', 'listening', 'fine sync or AV', 'frames outside the selected 1506',
    ],
    'concrete_defects': [],
    'low_findings': [
        {'id': 'L-A', 'line': '09_1', 'frames': '5280 until the arc-and-chord card leaves',
         'what': '"inside the dashes" about 3 px (540p) above the card top border', 'predates': True},
        {'id': 'L-B', 'line': '09_3', 'frames': '9408 on (lint anchor)',
         'what': 'card "opposite samples share a value" on the samples box bottom border; axis shows through '
                 'during its fade', 'predates': True},
        {'id': 'L-C', 'line': '09_7', 'frames': '13392 to the graph fade (lint anchor)',
         'what': '"x" 0.18 units from the right edge, inside the 0.25 margin; not clipped', 'predates': True},
        {'id': 'L-D', 'line': '09_9', 'frames': '15780 to the end fade',
         'what': '"the bump" ends about 6 px (540p) left of the recipe spike at 2.41', 'predates': True},
    ],
    'observations': [
        {'id': 'O-1', 'what': 'rho at RHO_AT outside the yellow loop, tied to the small circle by colour'},
        {'id': 'O-2', 'what': 'shared "precisely" footnote covers about 15 px of the plate bottom, 11460 to 12069; '
                              'circle and samples about 23 px clear'},
    ],
    'final4k_checks': ['752 ground rim', 'M1 2 px gaps', 'L14 7.6 px gap', 'root 343 instants 1661 1914 2131 2347'],
    'safe_to_advance_to_final4k': True,
    'acceptance': False,
    'release': False,
    'publication': False,
    'viewed_frame_count': len(frames),
    'viewed_frame_indices': frames,
}
with out.open('x') as f:
    json.dump(evidence, f, indent=1)
    f.write('\n')
print(out, len(frames), evidence['extraction_sha256'])
