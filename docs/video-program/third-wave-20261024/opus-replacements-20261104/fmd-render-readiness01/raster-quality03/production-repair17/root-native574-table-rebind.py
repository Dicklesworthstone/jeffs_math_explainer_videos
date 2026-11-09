"""Read-only path migration for imported native tables; never alters audio or source files.

The caller supplies exact per-file source path, destination path and SHA256 bindings.
Every destination is checked against the recorded cue hash and byte count before
any table is returned. Only the documented path columns may differ. Import receipts,
render qualification, picture review and replay/null tests remain the caller's gates.
"""
import hashlib
from pathlib import Path


def rebind(tables, bindings):
    assert set(tables) == {'timeline.tsv', 'cues.tsv', 'cuelog.tsv'}
    assert bindings and all(isinstance(k, str) and Path(k).is_absolute() for k in bindings)
    destinations = [v['path'] for v in bindings.values()]
    assert len(set(destinations)) == len(destinations), 'Destination paths must be one-to-one'
    assert all(Path(x).is_absolute() and '\t' not in x and '\n' not in x for x in destinations)
    for old, binding in bindings.items():
        assert set(binding) == {'path', 'sha256'}
        assert '\t' not in old and '\n' not in old
        target = Path(binding['path'])
        assert target.is_file() and not target.is_symlink(), target
        with target.open('rb') as f:
            digest = hashlib.file_digest(f, 'sha256').hexdigest()
        assert digest == binding['sha256'], ('Destination audio hash differs', target)

    seen, cue_paths = set(), set()
    result, edits = {}, []
    for kind in ('timeline.tsv', 'cuelog.tsv'):
        output = []
        for number, line in enumerate(tables[kind].splitlines(keepends=True), 1):
            fields = line.split(b'\t')
            if kind == 'timeline.tsv':
                if fields[0] != b'line':
                    output.append(line)
                    continue
                assert len(fields) == 5
                column = 4
                # The timeline path is last, so retain its original line ending.
                ending = b'\r\n' if fields[column].endswith(b'\r\n') else b'\n' if fields[column].endswith(b'\n') else b''
                raw_path = fields[column][:-len(ending)] if ending else fields[column]
            else:
                if line.startswith(b'#') or not line.strip():
                    output.append(line)
                    continue
                assert len(fields) == 13
                column, ending, raw_path = 0, b'', fields[0]
            old = raw_path.decode('utf-8')
            assert old in bindings, ('Unmapped audio path', kind, old)
            binding = bindings[old]
            if kind == 'cuelog.tsv':
                assert fields[1].decode() == binding['sha256'], ('Cue hash differs', old)
                assert int(fields[2]) == Path(binding['path']).stat().st_size, ('Cue byte count differs', old)
                cue_paths.add(old)
            seen.add(old)
            fields[column] = binding['path'].encode('utf-8')+ending
            rewritten = b'\t'.join(fields)
            restored = rewritten.split(b'\t')
            restored[column] = raw_path+ending
            assert b'\t'.join(restored) == line, 'Non-path bytes changed'
            output.append(rewritten)
            edits.append(dict(table=kind, line=number, source=old, destination=binding['path']))
        result[kind] = b''.join(output)
    assert seen == set(bindings) == cue_paths, 'Unused binding or timeline audio absent from cue log'
    result['cues.tsv'] = tables['cues.tsv']
    return result, edits
