"""make_evidence09.py: CC0's independent picture-review evidence for the repaired 09_boundary preview (scope02
extraction), in the gate's pictures() schema. It rehashes every picture and sheet in CC3's local copy against the
extraction record, then writes the evidence JSON once (open 'x'). It decodes nothing and opens no image; the viewing
itself was done by eye, sheet by sheet, and is described in review01.md.

usage: python3 -I -B make_evidence09.py COPY_DIR OUT_JSON
"""
import hashlib
import json
import sys
from pathlib import Path

COPY, OUT = Path(sys.argv[1]), Path(sys.argv[2])
SOURCE_SHA = '4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932'   # candidate03 boundary.rs
RECORD_SHA = 'e2f1c855ce0037da9e72ddce9d8a0f59fec5e784212a4f3e606b968862408411'   # CSD pictures.json (CC3: equal)
CLAIM_WORDS = ('accept', 'release', 'publication')


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def claimed(value, at=''):
    if type(value) is dict:
        for key, item in value.items():
            if any(word in key.lower() for word in CLAIM_WORDS) and item is not False:
                yield at+key
            yield from claimed(item, at+key+'.')
    elif type(value) is list:
        for n, item in enumerate(value):
            yield from claimed(item, at+str(n)+'.')


ch = COPY/'09_boundary'
assert sha(ch/'pictures.json') == RECORD_SHA
rec = json.loads((ch/'pictures.json').read_text())
frames = rec['selected_frames']
assert frames == sorted(set(frames)) and len(frames) == len(rec['pictures']) == 1506
assert len(rec['sheets']) == (len(frames)+7)//8 == 189
for item in rec['pictures']:
    assert sha(ch/'pictures'/Path(item['path']).name) == item['sha256'], item['path']
for item in rec['sheets']:
    assert sha(ch/'sheets'/Path(item['path']).name) == item['sha256'], item['path']
for name in ('decode.stderr', 'extract.stderr', 'probe.stderr'):
    assert (ch/name).stat().st_size == 0
extraction_path = str(Path(rec['sheets'][0]['path']).parent.parent/'pictures.json')


def full(picture, note):
    return dict(picture=picture, frame=frames[picture-1], note=note)


FULL_SIZE = [
    full(329, 'control: "inside the dashes" about 5 px above the "same endpoints" box, no overlap'),
    full(384, 'ARC_TEX: closing-bar gap visible, narrower by eye than the opening gap'),
    full(440, 'SUM_TEX: closing-bar gap visible, narrower than the opening gap; the final "0" about 17 px from the right edge'),
    full(512, 'leader in: riding a above, rho right; the leader crosses the small circle top'),
    full(518, 'no jump of a or rho between 7096 and 7097'),
    full(519, 'no jump of a or rho between 7096 and 7097'),
    full(565, 'hold at MID_R: P1, the leader still reads as a radius from the centre with a stub on to a'),
    full(606, 'SHRINK_TEX: closing-bar gap visible, narrower than the opening gap'),
    full(658, 'second shrink: a not crossed'),
    full(684, 'rest: dark gap to the hole ring visible; leader a short stub, its gaps not resolvable at 540p'),
    full(796, '09_3 anchor: the plot vertical axis ends under the opaque pill (pre-existing)'),
    full(889, '09_5 "precisely:" card covers the bottom ~15 px of the plate (pre-existing)'),
    full(965, '09_7 anchor: "x" axis label right edge about 12 px (0.18 unit) from the frame edge (pre-existing)'),
    full(1010, '09_7 growth: a nearly gone, ring radius about 10 px, a about 19 px from the centre; no crossing'),
    full(1088, '09_7 growth: a back inside the ring, ring well clear of it'),
    full(1176, '09_7 rest: ring of radius 1 through both poles, a shown'),
    full(1200, '09_7 after: radius line to i and "radius 1" card clear of the ring and of a'),
    full(1252, '09_8: slide done, "1" card in below a with a gap of about 10 px'),
    full(1290, '09_9 before growth: a over the red plate reads pinkish at 540p'),
    full(1330, '09_9: a fading out as the growth starts'),
    full(1360, '09_9: ring bottom meets the top of the "1" card'),
    full(1372, '09_9: ring bottom arc hidden behind the opaque "1" card; a fading back inside the ring'),
    full(1468, '09_9 rest at sqrt 2: ring through i and -i, meets the sqrt-2 card at its top-left corner'),
    full(1490, '09_9 after: a reads as at 15581 (same tint over the red plate)'),
    full(1500, 'ending: labels gone, plate alone'),
    full(1505, 'ending: plate fade tail, nothing legible'),
]

evidence = dict(
    chapter='09_boundary',
    reviewer='CC0 (LavenderPike), independent of the repair752 author',
    movie=rec['movie'],
    movie_sha256=rec['movie_sha256'],
    extraction_path=extraction_path,
    extraction_sha256=RECORD_SHA,
    source_sha256=SOURCE_SHA,
    concrete_defects=[],
    safe_to_advance_to_final4k=True,
    authored_repair=False,
    viewed_frame_indices=frames,
    viewed_sheets=[x['path'] for x in rec['sheets']],
    full_size_pictures=FULL_SIZE,
    open_judgement=dict(id='P1', frames=[7180], text='Held at MID_R the leader still reads as a radius from the '
                        'centre through the circle top to a, rather than a pointer to the fixed hole. Owner or root '
                        'judgement; not a concrete defect.'),
    observations=[
        dict(id='O1', severity='Low', frames=[5760, 6300, 7320], text='M1: the closing-bar gap is visible in '
             'ARC_TEX, SUM_TEX and SHRINK_TEX (about 2-3 px at 540p) but narrower by eye than the opening gap '
             '(about 6-8 px). A 4K measurement would settle "like the opening gap".'),
        dict(id='O2', severity='Low', frames=[6300], text='SUM_TEX: the final "0" sits about 17 px (about 0.25 '
             'unit) from the right edge at 960; a ~1% rescale cannot be resolved at this size.'),
        dict(id='O3', severity='Low, pre-existing', frames=[13392, 13860, 14430, 14954, 15581, 15780, 16260],
             text='The bump plot "x" label stands about 0.18 unit from the right edge from 09_7 to the end (the '
             'lint anchor).'),
        dict(id='O4', severity='Low, pre-existing', frames=[11520], text='The 09_5 "precisely:" card covers the '
             'bottom ~15 px of the plate.'),
        dict(id='O5', severity='Low', frames=[15780, 16260], text='At rest the sqrt-2 ring meets the sqrt-2 card '
             'at its top-left corner; it reads as passing behind the card, as the window expects, but tightly.'),
        dict(id='O6', severity='Note', frames=[14954, 15581, 15780, 16260], text='Over the saturated red plate '
             'the thin a reads pinkish at 540p (white over other colours). It looks the same before and after '
             'the 09_9 growth; not compared with master993 pixels.'),
    ],
    windows_seen=dict(
        M2='old a and rho leave with the small circle 7067-7076; bar, riding a/rho and leader fade in 7077-7090',
        shrinks='a never crossed in either shrink; rest-zero swap about 7728-7751',
        proof_gone='everything leaves together 7980-7996, no orphan leader',
        growth_09_7='a fades 13658-13676, hidden 13677-13735, faint from 13736, full by about 13750',
        slide_09_8='ring, card and bar fade 14821-14833; a rides the dot 14833-14940; "1" card in 14944-14954',
        growth_09_9='a fades 15619-15622, hidden 15623-15657, back by about 15666; ring behind the "1" card '
                    'about 15651-15705; rest by 15758',
    ),
    scope='Still pictures of the preview movie (960x540, sheets at 640x360): every selected frame viewed in its '
          'sheet cell, 26 pictures at full size. No motion, fine-sync, AV or final-4K claim; no audio heard or '
          'checked; Frederick narration unchanged and not assessed. No pixel comparison with master993.',
)
assert not list(claimed(evidence))
with open(OUT, 'x', encoding='utf-8') as stream:
    json.dump(evidence, stream, indent=1)
    stream.write('\n')
print('wrote %s: %d viewed frames, %d sheets, %d full-size, defects %s' % (
    OUT.name, len(frames), len(evidence['viewed_sheets']), len(FULL_SIZE), evidence['concrete_defects']))
