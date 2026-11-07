"""Preserved preview sample sample selection functions, reused for final frames."""
import re,wave,hashlib
from pathlib import Path
def sha(p):
    with Path(p).open("rb") as f:return hashlib.file_digest(f,"sha256").hexdigest()

def layout_paths(chapter, required_folders, optional_folders=()):
    required = [folder / (chapter + '.layout.tsv') for folder in required_folders]
    for path in required:
        assert path.is_file(), 'Required rendered layout report missing: ' + str(path)
    optional = [folder / (chapter + '.layout.tsv') for folder in optional_folders]
    return required + [path for path in optional if path.is_file()]

def select(chapter, rows, frames, phrases, sentences, layouts):
    selected = {}
    def add(seconds, reason):
        frame = round(seconds * 60)
        if 0 <= frame < frames:
            selected.setdefault(frame, set()).add(reason)
    for sec in range(1, frames // 60, 5):
        add(sec, 'five-second chapter progression')
    add((frames - 1) / 60, 'last frame')
    for row in rows:
        if row[0] == 'line':
            cue, start, wav = row[1], int(row[2]) / 60, Path(row[4])
            assert sha(wav) == phrases[cue]['wav_sha256'], cue
            with wave.open(str(wav)) as handle:
                duration = handle.getnframes() / handle.getframerate()
            for delta, label in [(-.2, 'before'), (.2, 'start'), (duration / 2, 'middle'),
                                 (max(0, duration - .1), 'end')]:
                add(start + delta, cue + ' ' + label)
            for index, sentence in enumerate(sentences[cue]):
                add(start + sentence['start'] + .5, f'{cue} provisional sentence {index+1}')
            for mark in phrases[cue]['marks']:
                for delta in (-.2, .5, 1.5):
                    add(start + mark['seconds'] + delta, f"{cue} provisional phrase byte{mark['byte']} +{delta}")
        elif row[0] == 'breath':
            begin, duration = int(row[1]) / 60, float(row[2])
            for delta in (0, duration / 2, max(0, duration - .1), duration + .5):
                add(begin + delta, 'authored hold +' + str(delta))
    for path in layouts:
        for line in path.read_text().splitlines():
            if not line.strip():
                continue
            match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', line)
            assert match, (path, line)
            anchor = float(match[1])
            deltas = [-.2, 0, .2, 1, 3, 6, 10]
            persistent = re.search(r'still there at (\d+) later looks', line)
            if persistent:
                count = int(persistent[1])
                deltas += [1 + .2 * count, 2 + .2 * count]
                deltas += list(range(5, max(5, int(frames / 60 - anchor) + 1), 5))
            for delta in deltas:
                add(anchor + delta, f'{path.name} {line} +{delta}s')
    return [{'chapter': chapter, 'frame': frame, 'seconds': frame / 60,
             'reasons': sorted(reasons)} for frame, reasons in sorted(selected.items())]
