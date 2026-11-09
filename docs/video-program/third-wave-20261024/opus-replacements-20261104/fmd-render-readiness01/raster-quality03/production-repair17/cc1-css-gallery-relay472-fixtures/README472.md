# Relay 472 fixtures (CC1)

These are the checks behind `cc1-css-gallery-relay472-receipt.md`. They reuse review 466's world (`cc1-relay463-review466-fixtures/`): 448 runs as a twin, the receive end is scripted, and nothing contacts a host. They are fixtures only. They are not an actual import and not final AV acceptance.

| File | What it is |
|---|---|
| `sim472.py` | `load472`: loads 472 into a sim466 world exactly as `load463` loads 463 (rebinds only `HERE`, `load_helper`, `subprocess`) |
| `run472.py` | reruns review 466's trials a, b and c against 472. It checks each test file against MANIFEST466. Trial a's record-folder glob gets one substitution (`relay463-` to `relay472-`); b and c run byte for byte |
| `test472x.py` | the targeted refusals, with 463 and 472 run on the same state: X1 has six same-count row substitutions plus the exact rows; X2 has a forged helper hash in 448 check's output |
| `compare472.py` | lists every difference between review 466's 463 outputs and the 472 reruns, after normalizing times, hex, scratch paths and a–h labels |
| `delta472.py` | the top-level statement delta from 463 to 472 (ast) |
| `cc1-css-gallery-relay472.diff` | `diff -u` from 463 to 472 |
| `rerun472-test466.out.json`, `rerun472-test466b.out.json`, `rerun472-test466c.out.json` | the rerun outputs. In trial a's, three truncated scratch paths are cleaned by review 466's rule |
| `test472x.out.json`, `compare472.out.txt`, `delta472.out.json` | the outputs of the other three scripts |
| `MANIFEST472.json` | the sha256 and size of every file here |

## Running

Run these from the repository root. Q is production-repair17. Each OUTDIR must be new and outside Q.

```
H=<sha256 of Q/cc1-css-gallery-relay472.py>
python3 -I -B run472.py    Q Q/cc1-relay463-review466-fixtures $H OUTDIR
python3 -I -B compare472.py Q/cc1-relay463-review466-fixtures OUTDIR
python3 -I -B test472x.py  Q Q/cc1-relay463-review466-fixtures $H OUTDIR2
python3 -I -B delta472.py  Q
```

The outputs here came from macOS with Python 3.15.0.
