#!/usr/bin/env python3
"""Move this unsealed bundle to a newly sealed chain: its path and hash, then the two pins that follow.

    python3 -I -B <bundle>/checks/repin_chain01.py --chain FOLDER/ts1_chain.py --sha256 HEX [--write]

--chain is relative to the folder above this bundle. Without --write nothing changes, and it prints
the hashes the three helpers would have. It refuses a chain file whose hash differs, a chain whose own
SHA256SUMS does not list that hash, and a sealed bundle: sealed bytes are preserved, so a sealed bundle
needs a new folder. It proves nothing about the chain itself. --check on ts1 tests the names the
helpers need, and the rehearsal must be run again against the new chain's fixture.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

BUNDLE = Path(__file__).resolve().parents[1]
RENDER, MASTER, VERIFY = (BUNDLE / name for name in ('ts1_final_render01.py', 'ts1_final_master01.py', 'ts1_final_verify01.py'))


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def swap(text, pattern, new):
    text, count = re.subn(pattern, new, text, flags=re.M)
    assert count == 1, 'Expected one line: ' + pattern
    return text


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--chain', required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    assert not (BUNDLE / 'SHA256SUMS').exists(), 'This bundle is sealed; preserve it and start a new folder'
    assert re.fullmatch(r'[0-9a-f]{64}', args.sha256), 'Expected a full sha256'
    assert re.fullmatch(r"[^'\\\n]+\.py", args.chain) and not args.chain.startswith('/'), 'Expected a path relative to the folder above this bundle'
    chain = BUNDLE.parent / args.chain
    assert chain.is_file() and hashlib.sha256(chain.read_bytes()).hexdigest() == args.sha256, 'The chain file does not have this hash'
    seal = chain.parent / 'SHA256SUMS'
    assert seal.is_file() and f'{args.sha256}  {chain.name}' in seal.read_text().splitlines(), 'The chain folder seal does not list this hash'
    render = swap(RENDER.read_text(), r"^CHAIN = HERE\.parent / '[^']+'$", f"CHAIN = HERE.parent / '{args.chain}'")
    render = swap(render, r"^CHAIN_SHA256 = '[0-9a-f]{64}'$", f"CHAIN_SHA256 = '{args.sha256}'")
    master = swap(MASTER.read_text(), r"^RENDER_SHA256 = '[0-9a-f]{64}'$", f"RENDER_SHA256 = '{digest(render)}'")
    verify = swap(VERIFY.read_text(), r"^    MASTER_HELPER: '[0-9a-f]{64}',$", f"    MASTER_HELPER: '{digest(master)}',")
    changed = [path.name for path, text in ((RENDER, render), (MASTER, master), (VERIFY, verify)) if path.read_text() != text]
    if args.write:
        for path, text in ((RENDER, render), (MASTER, master), (VERIFY, verify)):
            path.write_text(text)
    print(json.dumps({'chain': args.chain, 'chain_sha256': args.sha256, 'render_sha256': digest(render), 'master_sha256': digest(master),
                      'verify_sha256': digest(verify), 'changed': changed, 'written': args.write}, indent=2))


if __name__ == '__main__':
    main()
