#!/usr/bin/env python3
"""Run the sealed narration tools unchanged where the repository path is a symlink.

coherent_narration.unit_spans records each take as str(Path(wav).resolve()). report.json was
assembled on macOS, where that string equals R/wav. On ts1 R is a symlink, so the same 414 rows name
the same files by another spelling, and the sealed equality refuses (Mails 5426, 5428).

This changes that one string in the rows the sealed function returns, back to R/wav, and only when
the sealed value is exactly the real root plus wav and both name one file. The new spelling comes
from the record, never from the report. The sealed verifier then compares whole rows against
report.json as it always did: order, counts, text, spans, gaps and hashes, then the samples.

Called with the arguments the tool would get:
    python3 -I -B narration_alias01.py -I -B <workspace>/tools/<tool>.py <tool arguments>
The chain calls it directly for standalone verify. For the binary, shim/python3 calls it.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

TOOLS = {'coherent_narration.py': '95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5',
         'narration_cache.py': '47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0',
         'narration_delivery.py': '961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878'}


def refuse(message):
    print('narration alias refused: ' + message, file=sys.stderr)
    raise SystemExit(1)


def logical_source(root, record, sealed):
    """R/wav, when that is exactly the file the sealed function read through the real root."""
    relative = Path(record['wav'])
    if relative.is_absolute() or '..' in relative.parts:
        raise ValueError('take path must be repository-relative')
    logical = root / relative
    if sealed != str(root.resolve(strict=True) / relative) or not os.path.samefile(logical, sealed):
        raise ValueError('take is not this repository file by another spelling')
    return str(logical)


def install(coherent, root):
    """Wrap only unit_spans. Every row still comes from the sealed function."""
    counts = {'units': 0, 'rows': 0, 'renamed': 0}
    sealed = coherent.unit_spans

    def unit_spans(unit, script, record):
        rows = sealed(unit, script, record)
        counts['units'] += 1
        for _, row, _ in rows:
            logical = logical_source(root, record, row['source'])
            counts['rows'] += 1
            counts['renamed'] += logical != row['source']
            row['source'] = logical
        return rows

    coherent.unit_spans = unit_spans
    return counts


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    args = sys.argv[1:]
    if args[:2] != ['-I', '-B'] or len(args) < 3:
        refuse('only the two sealed narration tools may run here')
    tool = Path(args[2])
    if tool.name not in ('coherent_narration.py', 'narration_delivery.py'):
        refuse('only the two sealed narration tools may run here')
    if tool.name == 'coherent_narration.py' and args[3:4] != ['verify']:
        refuse('coherent_narration.py may only verify here')
    root = Path(os.environ['NARRATION_ALIAS_ROOT'])
    if not os.path.samefile(root, os.getcwd()) or not tool.resolve(strict=True).is_relative_to(root.resolve(strict=True)):
        refuse('the working directory and tool must be this repository')
    for name, digest in TOOLS.items():
        if hashlib.sha256((tool.parent / name).read_bytes()).hexdigest() != digest:
            refuse('sealed tool changed: ' + name)
    module = load(tool, tool.stem)
    counts = install(module if tool.name == 'coherent_narration.py' else module.coherent, root)
    sys.argv = [str(tool), *args[3:]]
    code = module.main()
    if code == 0:
        with open(os.environ['NARRATION_ALIAS_LOG'], 'a') as handle:
            print(json.dumps({'tool': tool.name, **counts, 'arguments': args[3:]}), file=handle)
    return code


if __name__ == '__main__':
    raise SystemExit(main())
