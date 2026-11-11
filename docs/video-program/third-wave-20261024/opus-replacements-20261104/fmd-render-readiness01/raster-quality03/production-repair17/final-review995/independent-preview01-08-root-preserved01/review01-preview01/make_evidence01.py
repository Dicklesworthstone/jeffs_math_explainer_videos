"""Write CC2's independent 01 preview picture evidence (gate pictures() schema) from the verified local copy.
Usage: make_evidence01.py COPY_DIR OUT.json UTC. Refuses an existing OUT. Reads only the copy."""
import hashlib, json, sys
from pathlib import Path

copy, out, utc = Path(sys.argv[1])/'01_the_bump', Path(sys.argv[2]), sys.argv[3]
CSD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-combined-pictures01/'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
rec_path = copy/'pictures.json'
rec = json.loads(rec_path.read_text())
assert sha(rec_path) == '82e3fce62972ae4c9330e9dbe2466c43e0297d902a706b4bf0c4e46e69c70852'
assert rec['movie_sha256'] == '005271e97d32246645b8c36340258d9be83b590962be2c97af1cf922c95e6beb'
assert rec['frames'] == 6311
for item in rec['pictures'] + rec['sheets']:
    assert item['path'].startswith(CSD + '01_the_bump/')
    assert sha(copy.parent/item['path'][len(CSD):]) == item['sha256'], item['path']
frames = rec['selected_frames']
assert frames == sorted(set(frames)) and len(frames) == 120 and len(rec['sheets']) == 15
assert set(range(6273, 6311)) <= set(frames) and set(range(5562, 5598)) <= set(frames)
runs, start = [], frames[0]
for a, b in zip(frames, frames[1:] + [None]):
    if b != a + 1:
        runs.append([start, a]); start = b
fade = [f for f in frames if 6155 <= f <= 6310]
crops = {
    'crops01/fraction-top-01.png (3x)': [48, 120, 1248, 6269],
    'crops01/fade-end-01.png (1x)': [6305, 6306, 6307, 6308],
    'crops01/degree-6-8-40.png (1x)': [1548, 1848, 2148],
    'crops01/predict-box-top-01.png (3x)': [3440, 3648],
    'crops01/predict-box-right-01.png (2x)': [3440, 3648],
    'crops01/thumbs-fadein-01.png (1x)': [5497, 5527, 5562, 5566, 5570, 5574, 5578, 5582, 5586, 5590, 5594, 5600,
                                          5659, 5696],
    'pixel measurement of header ink margins (fraction_margin01.py)': [48, 60, 72, 120, 348, 1248, 2448, 3440, 3648,
                                                                       4848, 5527, 6000, 6269, 6290],
    'pixel measurement of the bottom ink margin': [348, 1248, 2448, 3440, 3948, 4848, 5527, 6269],
    'pixel measurement of curve heights (curve_heights01.py)': [1548, 1848],
    'pixel measurement of the ending fade (fade_profile01.py)': fade,
}
assert all(set(v) <= set(frames) for v in crops.values())
evidence = {
    'kind': 'independent preview picture review',
    'chapter': '01_the_bump',
    'reviewer': 'RubyRaven (CC2)',
    'root_reviewer': 'BlueSalmon',
    'reviewed_utc': utc,
    'movie': rec['movie'],
    'movie_sha256': rec['movie_sha256'],
    'extraction': CSD + '01_the_bump/pictures.json',
    'extraction_sha256': sha(rec_path),
    'extraction_receipt': {'path': CSD + 'resource-receipt.json', 'exit': 0,
                           'finished': '2026-11-11T01:00:03.257494+00:00',
                           'inputs_sha256': '19eb221bfbec74771116d57e76f4f71d91fe48ce761ed9b1f407a54650e29b50'},
    'source_sha256': '90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7',
    'frames': rec['frames'],
    'selected_count': len(frames),
    'viewed_frame_indices': frames,
    'viewed_sheets': [s['path'] for s in rec['sheets']],
    'viewed_runs': runs,
    'layout_anchor_checked': rec['layout_anchors'],
    'viewing_method': 'every selected picture seen as its 640x360 cell in the 15 contact sheets (1280x1536); '
                      'the frames in enlarged_crop_frames also cut from the 960x540 pictures and enlarged or measured',
    'enlarged_crop_frames': crops,
    'concrete_defects': [],
    'observations': [
        'O1 title-safe flag, 0.8 s to 105.1 s (48 to 6305): the top ink of the fraction 1/(1 + x^2) is row 14 of 540 '
        '(row 15 at 48 while it fades in), 0.207 scene units or 2.6% of the height, and it is the topmost ink in every '
        'measured frame. The kit wants 0.25 units (16.9 px at 540p, 67.5 px at 4K), so it is short by about 0.04 units '
        '(about 3 px at 540p, about 12 px at 4K). This agrees with the extraction anchor (0.22 at frame 48, kept in later '
        'looks). It is also outside a 5% title-safe band (27 px) and a 3.5% action-safe band (19 px). Fully inside the '
        'frame, not clipped, legible in every viewed frame, and the header row (title top row 31) is steady through the '
        'chapter. Low: a framing choice for root, not a concrete defect.',
        'O2 ending fade, 104.6 s to 105.17 s (6276 to 6310): full contrast through 6275 (mean distance 7.07, p99.9 239), '
        'then a monotonic fall: 6290 mean 5.54, 6300 2.58, 6305 0.88, 6307 0.31, 6309 0.03, 6310 0.00 (every sampled '
        'pixel equals the background). Header, curve with ring, and thumbnail column fall together (region maxima 22/19/18 '
        'at 6307, 13/8/8 at 6308), so no element leaves early or lingers. About 0.6 s, as the source fade_out_all(0.6). '
        'The broken outlines at 6307 and 6308 are content at under 10% of full contrast in the preview JPEGs.',
        'O3 30.8 s (1848): the degree readout shows 8 while the yellow curve is still nearly degree 6. The readout is '
        '2*ceil(k) and leads during the k tween from 3 to 5; measured heights at x = 0.8, 0.9, 0.95, 1.0 fit '
        'd6 + 0.07 (d8 - d6), k about 3.07. Settled frames agree: 1548 is degree 6 with roots at +-1, 2148 is degree 40.',
        'O4 57.3 s and 60.8 s (3440, 3648): the predict box border reaches column 917 (4.4% right margin), outside a 5% '
        'title-safe band and inside 3.5% action-safe; its text ends near column 900 (6.1%). Recipe 25.2 and bump 0.452 at '
        'x = 1.1 agree with the sum to degree 40 (25.24) and 1/2.21 (0.4525).',
        'O5 65.8 s onward (3948 to 6305): the ring of radius 1 uses the x scale (92 px per unit) while the dimmed bump keeps '
        'its own y scale (about 141 px per unit), so the poles at +-i sit below the bump peak, at about 0.65 on the bump '
        'scale. The peak label 1 is gone before this, so no two labels disagree. A teaching note for root, not a fault.',
    ],
    'safe_to_advance_to_final4k': True,
    'authored_repair': False,
    'open_reviews': ['final 4K pictures', 'motion', 'audio and listening', 'fine sync', 'AV'],
}
with out.open('x') as fh:
    json.dump(evidence, fh, indent=1)
    fh.write('\n')
print(out, sha(out), len(frames), 'runs', len(runs))
