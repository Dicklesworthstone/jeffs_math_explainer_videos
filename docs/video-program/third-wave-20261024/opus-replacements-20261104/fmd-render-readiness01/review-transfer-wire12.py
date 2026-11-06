"""Bounded review-package wire format; no render or review acceptance."""
import hashlib
import json
from pathlib import Path
import struct

CAP = 32 * 1024 * 1024

def digest(data):
    return hashlib.sha256(data).hexdigest()

def read_exact(stream, size):
    chunks = []
    while size:
        block = stream.read(min(size, 1024 * 1024))
        assert block, 'Truncated transfer; preserve partial files'
        chunks.append(block)
        size -= len(block)
    return b''.join(chunks)

def check_rows(rows):
    assert len(rows) == 54
    names = [r['path'] for r in rows]
    assert len(set(names)) == 54 and 'transfer.json' in names
    for r in rows:
        p = Path(r['path'])
        assert not p.is_absolute() and '..' not in p.parts and str(p) == r['path']
        assert type(r['bytes']) is int and 0 <= r['bytes'] <= CAP
        assert len(r['sha256']) == 64 and all(c in '0123456789abcdef' for c in r['sha256'])
    assert sum(r['bytes'] for r in rows) <= CAP

def send(folder, stream, expected):
    raw = (folder/'transfer.json').read_bytes()
    assert digest(raw) == expected
    transfer = json.loads(raw)
    rows = []
    for r in transfer['files'] + [{'relative_path':'transfer.json','sha256':expected}]:
        name = r['relative_path']
        p = Path(name)
        assert not p.is_absolute() and '..' not in p.parts
        data = (folder/p).read_bytes()
        assert digest(data) == r['sha256']
        rows.append({'path':name,'sha256':r['sha256'],'bytes':len(data)})
    check_rows(rows)
    header = json.dumps(rows).encode()
    assert len(header) <= 65536
    stream.write(struct.pack('>I',len(header)))
    stream.write(header)
    for row in rows:
        data = (folder/row['path']).read_bytes()
        assert len(data) == row['bytes'] and digest(data) == row['sha256']
        stream.write(data)
    stream.flush()

def receive(stream, folder, expected):
    length = struct.unpack('>I',read_exact(stream,4))[0]
    assert 0 < length <= 65536
    rows = json.loads(read_exact(stream,length))
    check_rows(rows)
    assert next(r['sha256'] for r in rows if r['path']=='transfer.json') == expected
    assert not folder.exists()
    folder.mkdir()
    for row in rows:
        data = read_exact(stream,row['bytes'])
        assert digest(data) == row['sha256']
        p = folder/row['path']
        p.parent.mkdir(parents=True,exist_ok=True)
        with p.open('xb') as f:
            f.write(data)
    assert stream.read(1) == b'', 'Trailing transfer bytes'
    transfer = json.loads((folder/'transfer.json').read_text())
    expected_rows = {r['relative_path']:r['sha256'] for r in transfer['files']}
    assert len(transfer['files']) == len(expected_rows) == 53
    assert expected_rows == {r['path']:r['sha256'] for r in rows if r['path'] != 'transfer.json'}
    return rows
