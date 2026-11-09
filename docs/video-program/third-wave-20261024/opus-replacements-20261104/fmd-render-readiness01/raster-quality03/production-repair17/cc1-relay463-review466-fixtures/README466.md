# Review 466 fixtures (CC1)

Targeted fixtures for `cc1-relay463-review466.md`. They test CC3's relay 463 (`d1822a51`) against the pinned import 448 (`b6653757`).

They are fixtures only. They run on one machine, in a scratch world. They contact no host, render nothing and import nothing. A passing fixture is not evidence that any actual import happened, and it is not final AV acceptance.

## How the world is built (`sim466.py`)

- **448 runs as a "twin".**
  - The pinned 448 bytes sit at the CSD Q path of the world.
  - `launch448_466.py` executes 448's text with only its four root literals rewritten into the world: D, CSS, RECOVERY and DATA.
  - It sets `__file__` to the genuine-bytes path. Reversing the rewrite gives `b6653757` again; World asserts that.
- **463 runs unchanged, in-process.** `load463` asserts `d1822a51`. Only three module attributes are rebound:
  - `HERE`, the controller's Q;
  - `load_helper`, which returns the 448 twin;
  - `subprocess`, a shim for ssh. The shim runs 463's own probe texts on this machine. On "css" it maps the CSD namespace to the CSS namespace; on "csd" it maps `/proc/locks` to a scratch file.
- **The receive end is scripted** (`receiver466.py`):
  - the shim emulates the guard's admission: the job folder plus an admitted receipt, or a refusal;
  - the receiver reads the stream to EOF and writes the outcome the scenario asks for.

  It is not 448 receive. The only places 448 receive itself runs are F1/F1r and F5r, where the twin's real `receive` mode is called.

## Files

| File | What it is |
|---|---|
| `sim466.py` | the world: CSD and CSS trees, the twin, the ssh shim, `load463`, `args` |
| `launch448_466.py` | runs the twin as if it were the genuine 448 file; asserts the genuine bytes are `b6653757` |
| `receiver466.py` | scripted CSD receive end (complete / fail / refused, then an exit code; 255 stands for a lost ssh connection) |
| `test466.py` | trial a: F1 (root's g question), F2 (run/verify/status through 13 interruption and guard scenarios) |
| `test466b.py` | trial b: F1r (receive binding with cleaned refusal texts), F3 (each of a–h failing on its own), F4 (448 SSD routing and serial-gate refusals) |
| `test466c.py` | trial c: F5 (verify on a completed job whose receipt names another command), F5r (448 receive on the same) |
| `test466.out.json` | trial a output; see the note below |
| `test466b.out.json`, `test466c.out.json` | trial b and c outputs, scratch paths shown as `W` |
| `MANIFEST466.json` | sha256 and size of every file here |

## Running

```
python3 -I -B test466.py  Q OUTDIR
python3 -I -B test466b.py Q OUTDIR
python3 -I -B test466c.py Q OUTDIR
```

- Q is the production-repair17 folder holding `cc3-css-gallery-relay463.py` and `csd-gallery-import448.py`.
- OUTDIR must not exist yet, and it must be outside Q.
- Each run writes its world and its `*.out.json` there, and writes nothing in Q.
- The outputs here came from macOS with Python 3.15.0.

## Note on trial a's output

In trial a, three F1 `receive_binding` refusal texts were cut to 160 characters before the scratch prefix was replaced. Those three strings are cleaned here by one rule: each scratch path fragment becomes `W/[path truncated before cleaning; full text in test466b.out.json F1r]`.

`MANIFEST466.json` records the raw output's hash and the rule. Every other byte of `test466.out.json` is as the run wrote it. Trial b repeats the same binding cases with full, cleaned texts.
