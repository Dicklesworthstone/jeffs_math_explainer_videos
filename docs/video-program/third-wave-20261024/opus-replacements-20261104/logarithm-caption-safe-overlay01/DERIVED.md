# Derived from sealed overlay01

`logarithm-caption-overlay01` is sealed (`faf87e03…b410`) and unchanged. These files start from its bytes, edited by exact
replacement.

| File | From | Change |
|---|---|---|
| `overlay_css02.py` | `overlay_css01.py` | New lane and job names. Draws two caption files on each frame. The caption place is tested in place of the footer peak, on candidate A only. Sheets show both whole frames side by side |
| `guard_css02.py` | `guard_css01.py` | Lane name only. The preserved guard, its pins and its asserted policy are the same |
| `start-overlay-css02.sh` | `start-overlay-css01.sh` | Helper and lane names only |
| `transfer-files02.json` | `transfer-files01.json` | The SRT row is gone, because the captions now live in this folder |
| `checks/run_checks02.py` | `checks/run_checks01.py` | New plan checks. The stand-in draws at the top for top samples. One new refusal for a caption in the wrong place |

`POLICY`, the tool pins, the font rule, the exact frame seek and the staging root are unchanged.
