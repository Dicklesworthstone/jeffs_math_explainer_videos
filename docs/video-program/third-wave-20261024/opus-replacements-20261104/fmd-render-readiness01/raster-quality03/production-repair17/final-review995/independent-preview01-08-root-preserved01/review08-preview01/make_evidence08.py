"""Write CC2's independent 08 preview picture evidence (gate pictures() schema) from the verified local copy.
Usage: make_evidence08.py COPY_DIR OUT.json UTC. Refuses an existing OUT. Reads only the copy."""
import hashlib, json, sys
from pathlib import Path

copy, out, utc = Path(sys.argv[1])/'08_loop_sums', Path(sys.argv[2]), sys.argv[3]
CSD = '/data/projects/jmev-stroke-probe-20261106/jobs/csd-native-combined-pictures01/'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
rec_path = copy/'pictures.json'
rec = json.loads(rec_path.read_text())
assert sha(rec_path) == '57dba91ddc6d744c1d01648a82a134697f0e23105ad8c6f06e183425cf2e302c'
assert rec['movie_sha256'] == '63984b4473a1cb391546a9fa2c1157a9813fc0e37afb307cb0b64c6a5c65f147'
for item in rec['pictures'] + rec['sheets']:
    assert item['path'].startswith(CSD + '08_loop_sums/')
    assert sha(copy.parent/item['path'][len(CSD):]) == item['sha256'], item['path']
frames = rec['selected_frames']
assert frames == sorted(set(frames)) and len(frames) == 484 and len(rec['sheets']) == 61
assert set(range(10200, 10381)) <= set(frames)
runs, start = [], frames[0]
for a, b in zip(frames, frames[1:] + [None]):
    if b != a + 1:
        runs.append([start, a]); start = b
crops = {
    'crops01/c-name-01.png (5x)': [10785, 10800, 10836, 10846],
    'crops01/four-n-01.png (3x)': [12336, 12420, 12444, 12456],
    'crops01/four-n-02.png (4x)': [12336, 12420],
    'crops01/nested-c-01.png (5x)': [6990, 7000, 7026],
    'crops01/delta-disk-01.png (5x)': [9150, 9168, 9195, 9740],
    'crops01/triangle-arrows-01.png (3x)': [13536, 13788],
    'pixel measurement of the footer box': [14688, 14736],
}
assert all(set(v) <= set(frames) for v in crops.values())
evidence = {
    'kind': 'independent preview picture review',
    'chapter': '08_loop_sums',
    'reviewer': 'RubyRaven (CC2)',
    'root_reviewer': 'BlueSalmon',
    'reviewed_utc': utc,
    'movie': rec['movie'],
    'movie_sha256': rec['movie_sha256'],
    'extraction': CSD + '08_loop_sums/pictures.json',
    'extraction_sha256': sha(rec_path),
    'extraction_receipt': {'path': CSD + 'resource-receipt.json', 'exit': 0,
                           'finished': '2026-11-11T01:00:03.257494+00:00',
                           'inputs_sha256': '19eb221bfbec74771116d57e76f4f71d91fe48ce761ed9b1f407a54650e29b50'},
    'source_sha256': 'a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef',
    'frames': rec['frames'],
    'selected_count': len(frames),
    'viewed_frame_indices': frames,
    'viewed_sheets': [s['path'] for s in rec['sheets']],
    'viewed_runs': runs,
    'required_run_10200_10380_viewed': True,
    'viewing_method': 'every selected picture seen as its 640x360 cell in the 61 contact sheets (1280x1536); '
                      'the frames in enlarged_crop_frames also cut from the 960x540 pictures and enlarged',
    'enlarged_crop_frames': crops,
    'concrete_defects': [],
    'observations': [
        'O1 179.8 s to 217.8 s (10788 to 13073): the c label sits on the amber quarter fill at lower contrast; one glyph, same weight as at 10785, legible.',
        'O2 244.8 s (14688, 14736): the precisely footer box ink spans x 26 to 933 of 960 and down to row 527 of 540; its text spans x 43 to 822 and rows 471 to 510. '
        'The box border and the first 0.5% of the text lie outside a 5% title-safe area; all of it is inside the frame and legible.',
        'O3 170.45 s to 172.45 s (10227 to 10347): the c and D_n labels fade out from 10227 (gone by 10235) while the leg walks the perimeter counterclockwise, and fade back in with perimeter P_n from 10339 after the walk closes at 10337 (full by 10347).',
        'O4 106.0 s and 108.4 s (6360, 6506): I_0 = I_1 + I_2 + I_3 + I_4 numbers the four quarters 1 to 4, while I_n elsewhere is the n-th nested integral. A notation overlap in the script, not a picture fault.',
    ],
    'safe_to_advance_to_final4k': True,
    'authored_repair': False,
    'open_reviews': ['final 4K pictures', 'motion', 'audio and listening', 'fine sync', 'AV'],
}
with out.open('x') as fh:
    json.dump(evidence, fh, indent=1)
    fh.write('\n')
print(out, sha(out), len(frames), 'runs', len(runs))
