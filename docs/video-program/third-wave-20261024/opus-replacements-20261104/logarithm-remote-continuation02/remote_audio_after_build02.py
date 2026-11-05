#!/usr/bin/env python3
"""Logarithm rebind and assembly on mac-mini-max, second attempt (bead 87do.63).

rebind01 exited 1 before writing anything. The coherent tool opens every selected raw take through the decode receipt,
and the sealed freeze and transfer had missed those files. Root has since transferred and verified them.

This helper loads the sealed remote_audio_after_build01.py by hash and reuses its freeze, launch and receipt code. It
changes two things:
- the rebind runs as logarithm-remote-rebind02 with O/rebind-inputs02.json, because rebind01 stays preserved;
- the freeze gains the 161 raw-source rows, their manifest, the built binary, the repository cargo config and this bundle.

The assembly keeps the unused logarithm-remote-assemble01 and O/assemble-inputs01.json. Anything that could waste a job
name is checked first, including Lavender's --check, the same dry run --apply opens with. Technical gates only: nothing
here was heard, viewed or accepted.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve()
BUNDLE = HERE.parent
R = HERE.parents[5]
SEALED = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-remote-mac-mini-max01'
SEALED_SUMS_SHA = '26c0b341f877e6b492a4d1798e37ce11698c9d8282d759248cd9914dab2890d8'
SEALED_HELPER_SHA = '669e7ae6da7a77a295d4548b5e09d15594406592a10cfacd127cd8676e57800a'
REBIND = 'logarithm-remote-rebind02'
BINARY_SHA = 'ae54bc176e67c9580b5110a6af3af3e0796974f357844544d0c10d8345246136'
CARGO_CONFIG_SHA = '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716'
# rebind_audio.py --check, keyed as it reports them; root saw the same four on the remote.
PREDICTED = {
    'explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver17.json':
        'e9cc0e4bd497ebd5d67e788103a6a6d67a0ced64ffa54ad039f08e938cc21d93',
    'explainer_corpus/logarithm_mysteries/narration_units.json':
        '7cb6dbcd0b99d30b2d81cedc75d6046709935bc39ad2ad351439ed25228c2ed6',
    'holds-bound01.json': 'ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c',
    'spans01.json': '32d75a3e89c07a2791ef8a954f15769915a26c5d9d3ced5ec7eb2c5761c5b8c0',
}


def load_sealed():
    sums, helper = SEALED / 'SHA256SUMS', SEALED / 'remote_audio_after_build01.py'
    assert hashlib.sha256(sums.read_bytes()).hexdigest() == SEALED_SUMS_SHA, 'The sealed bundle moved'
    assert f'{SEALED_HELPER_SHA}  {helper.name}' in sums.read_text().splitlines(), 'The seal does not list this helper'
    assert hashlib.sha256(helper.read_bytes()).hexdigest() == SEALED_HELPER_SHA, 'The sealed helper moved'
    spec = importlib.util.spec_from_file_location('remote_audio_after_build01', helper)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


A = load_sealed()
RAW = A.O / 'raw-source-transfer01.json'
DECODE = A.EL / 'production-audio-stage09-01/decoded02/decode-receipt.json'
BINARY = A.O / 'logarithm_mysteries-release01'
CARGO_CONFIG = R / '.cargo/config.toml'
REBIND_INPUTS = A.O / 'rebind-inputs02.json'
ASSEMBLE_INPUTS = A.O / 'assemble-inputs01.json'
LATER_JOBS = ('logarithm-remote-timeline01', 'logarithm-remote-score01')


def output(key):
    return A.S / ('workspace/' + key if key.startswith('explainer_corpus/') else key)


def rebind_helper(mode):
    return subprocess.run([sys.executable, '-I', '-B', str(A.H), mode], cwd=R, check=True, stdout=subprocess.PIPE,
                          text=True, timeout=900).stdout


def check_seal(digest):
    sums = BUNDLE / 'SHA256SUMS'
    assert A.sha(sums) == digest, f"This bundle's SHA256SUMS is not the seal {digest[:12]}"
    for line in sums.read_text().splitlines():
        expected, name = line.split(maxsplit=1)
        assert A.sha(BUNDLE / name.lstrip('*')) == expected, name


def unused():
    """Every name this run creates. A used one would otherwise refuse only after an earlier job had spent its name."""
    paths = [A.JOBS / job for job in (REBIND, A.ASSEMBLY, *LATER_JOBS)]
    paths += [REBIND_INPUTS, ASSEMBLE_INPUTS, A.S / 'narration_units-before-rebind01.json', A.S / 'rebind-report01.json']
    paths += [output(key) for key in PREDICTED if not key.endswith('/narration_units.json')]
    used = [str(p) for p in paths if p.exists()]
    assert not used, f'Already used, preserved: {used}'


def preflight():
    assert A.sha(A.H) == A.H_SHA, 'rebind_audio.py is not the sealed helper'
    assert A.success(A.BUILD), 'The build has not passed'
    unused()
    assert json.loads(rebind_helper('--check'))['outputs_sha256'] == PREDICTED, 'The rebind no longer predicts these outputs'


def freeze():
    """The sealed rows, plus each raw source tied to the decode receipt that names it, plus what the build proved."""
    rows = A.inputs()
    raw = A.read(RAW)['files']
    for row in raw:
        assert A.sha(Path(row['path'])) == row['sha256'], row['path']
        assert rows.setdefault(row['path'], row['sha256']) == row['sha256'], row['path']
    decode = A.read(DECODE)
    records = decode['records']
    named = {str(R / decode['binding']['path'])}
    named |= {str(R / record[key]) for record in records for key in ('source', 'downloads_receipt')}
    assert len(records) == 80 and {row['path'] for row in raw} == named, 'The raw manifest is not what the decode receipt names'
    assert rows[str(R / decode['binding']['path'])] == decode['binding']['sha256'], 'The voice binding differs from the receipt'
    for record in records:
        assert rows[str(R / record['source'])] == record['source_sha256'], record['cue']
    for path in (RAW, HERE, BUNDLE / 'SHA256SUMS'):
        rows[str(path)] = A.sha(path)
    for path, digest in ((BINARY, BINARY_SHA), (CARGO_CONFIG, CARGO_CONFIG_SHA)):
        assert A.sha(path) == digest, path
        rows[str(path)] = digest
    assert str(A.UNITS) not in rows, 'Rebind output cannot be a frozen input'
    assert not [p for p in rows if '/Downloads/' in p], 'Provider originals are provenance, not inputs'
    return rows


def main(seal):
    check_seal(seal)
    preflight()
    rows = freeze()
    A.launch(REBIND, REBIND_INPUTS, rows, [sys.executable, '-I', '-B', str(A.H), '--apply'])
    rebind_helper('--verify')
    for key, digest in PREDICTED.items():
        assert A.sha(output(key)) == digest, key
        rows[str(output(key))] = digest
    receipt = A.JOBS / REBIND / 'resource-receipt.json'
    rows[str(receipt)] = A.sha(receipt)
    A.launch(A.ASSEMBLY, ASSEMBLE_INPUTS, rows, [sys.executable, '-I', '-B', str(A.ASSEMBLE)])


if __name__ == '__main__':
    main(sys.argv[1])
