# Relay 463: implementation receipt

CC3, 2026-11-09 UTC. This is source plus local fake-transport tests. Nothing ran on either host: no ssh, no staging, no export, no transfer, no import. Root inspects and launches.

## Files

| File | sha256 | |
|---|---|---|
| `cc3-css-gallery-relay463.py` | `d1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892` | new, 509 lines |
| `cc3-css-gallery-relay463-tests.py` | `9ba910a37f3dbfa48ef5e912610e259cd8214cb3dd98c329930fab5b7a6fd2ac` | new, 96/96 |
| `cc3-css352-import-relay394.py` | `371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c` | base, unchanged |
| `css-gallery-relay453.py` | `c8f5539d2290416543e4a9bee50d7ef5bc2af82b61cf687ea73f30198881ec0d` | root's, unchanged |
| `csd-gallery-import448.py` | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` | pinned helper, unchanged |
| `cc3-relay453-review457.md` and `cc3-relay453-review457-fixtures.py` | `7f1d807b…`, `c07f25bb…` | review, unchanged |

## What 463 is

463 began as a byte copy of 394 at `371f5d1f`, and every change was a separate edit. Test section 0 compares the two files top-level statement by statement:
- 15 statements are edited, exactly as the tests' `DELTA` lists them;
- 2 are added: `HELPER_LABEL = 'import448'` and `EARLIER371`;
- 1 is renamed: `run_371` → `run_helper`;
- every other statement is byte-identical to 394;
- the module docstring is rewritten for gallery.

The edits fall into four groups:
- **448 routing (root's 453 hunks):**
  - `HELPER` is `csd-gallery-import448.py` at `b6653757`;
  - `ORDER` is `('05_gallery',)`;
  - the four earlier chapters need completed 371 imports, found under the `csd-css352-import371-` prefix;
  - `CSS_PROBE` and the sender use `RECOVERY` for the helper, the manifest and the 442 job.
- **`VERSION`:** 463. Records go to `relay463-05-gallery-a<N>-run<K>/`.
- **L1 messages:** helper-mismatch stops name the file and the pin (`CSS RECOVERY copy of csd-gallery-import448.py is not b6653757`, `CSD Q copy of … is not b6653757`). Refusals and validation labels say `import448`. One exception: 448's own `not a version-371 manifest` text belongs to 448, so it is unchanged.
- **From 394 at `371f5d1f`, closing M1 and L2:**
  - post-import `validate()` a–h and the read-only `verify` mode;
  - `POST_PROBE` (the guard-inputs re-hash);
  - the probe timeout;
  - `--prepared` refuses an unparsed attempt name;
  - unexpected errors are recorded before they re-raise.

**Byte-identical to 394, so behavior is retained:**
- `start` is identical apart from its two `RECOVERY` sender paths: the admission poll, the send only after admission of this exact receive command, and stdin closed on any stop before admission;
- `finish`, `verify`, `wait_lane` (the lane holder and waiter scan before check and before receive), `remote` and `probe` (a transport stop, nothing retried);
- `spawn` (one receive and one send per run), `expected_rows` and every probe;
- all timeouts, and `main` apart from the choices that `ORDER` drives.

The attempt rule (refuse N at or below any existing 448 gallery job or inputs) and `check_prepared` keep 394's logic. Only their message labels change.

## Tests run

```
python3 -I -B cc3-css-gallery-relay463-tests.py <Q> <temp-parent>     # 96/96, ~219 KB temp
```

- **Section 0:** the delta proof above.
- **The 457 battery, replayed on 453 and on the actual 463:**
  - routing: the `RECOVERY` probe and send, the 448 guard command under `guard_csd02`, rows350 `csd-css352-import448-05-gallery-a1`;
  - refusals, the serial gate for each of the four chapters, the lane, transport at every stage, and refused admission;
  - the real `CSD_PROBE` run on a synthetic jobs tree;
  - L1, M1 and L2, each shown on 453 and closed on 463.
- **Read-only `verify` on 463:**
  - a pass gives the exact 448 check command, and the inputs re-hash runs on the a1 job with the 3600 s timeout;
  - 15 failure cases cover a, b, c, d, f, g and h, each exit 3 with the condition named;
  - transport at the check and at the re-hash gives exit 2, and a wrong manifest sha gives exit 1 before any remote call;
  - `--attempt 2` validates the a2 names;
  - `verify` spawned nothing, sent no `prepare`, and wrote nothing.

A mutation check ran from a scratch folder, not saved in Q. Five mutants of 463 each failed the suite:
- stdin left open after a refused admission;
- g dropped;
- the sender back on CQ;
- the admission window changed;
- the serial gate skipping 14.

Not run: anything on either host, 448's real modes, the guard, a stream.

## Root's commands (after the gallery export exists)

```
python3 -I -B cc3-css-gallery-relay463.py status --chapter 05_gallery --attempt 1 --manifest <448 export name> --manifest-sha <sha>
python3 -I -B cc3-css-gallery-relay463.py run    --chapter 05_gallery --attempt 1 --manifest <448 export name> --manifest-sha <sha> [--lane-wait-minutes M]
python3 -I -B cc3-css-gallery-relay463.py verify --chapter 05_gallery --attempt 1 --manifest <448 export name> --manifest-sha <sha>
```

Before `run`, root must do the following (none of it is done here):
1. **Export with 448.** Export the finished 442 job with 448's own `export` on the second render host. CC2's review 456 (F1) found that 448 refuses a 446 manifest and a 446 stream.
2. **Stage the helper and manifest.** Put 448 and that manifest under `RECOVERY` on the second render host and in Q on the build host.
3. **Create the alias.** Establish the `RECOVERY` alias and `TOP_SSD` on the build host. Until they exist, 448 check refuses before prepare.
4. **Import and verify the earlier chapters.** Import 01, 04, 12 and 14 with 394, and check each one with 394 `verify`.

## Manifest pins suggested for 461's successor (I have not edited the controller)

461 (`6f5fa078…`) still accepts whichever manifest the import result names, provided it is consistent with the inputs. My suggestion: add a constant and assert it in `verify_css_imports` after the manifest is read.

```
EXPORTS = {   # chapter: (Q file name, sha256) of root's named export
    '01_the_bump':      ('css352-import371-01-the-bump-manifest392.json',      '633c06d0542f6669eeb42993a3b1b91a1403a752ec259757ef5ab7297a24ac4b'),
    '04_reading_color': ('css352-import371-04-reading-color-manifest396.json', '04d1cd99e0114e63abfeaefa23aab7d3ce6e56092d77d8a4459fa3cb2da6896c'),
    '12_residues':      ('css352-import371-12-residues-manifest397.json',      '73c62b8d3c92ced7b4f48a9316ce12440b72424ae29ddc6d8b9a8346451a5359'),
    '14_continuation':  ('css352-import371-14-continuation-manifest424.json',  '79fc3bf9e5c9ffa86f5f30df19a22431acb2b51d0ac620ac259706344caa5206'),
    # '05_gallery': (<448 export name>, <sha>)  -- pending: a version-448 export of css-reviewed-final-05-gallery352-ssd442
}
# in verify_css_imports, after `manifest = i.q_manifest(Path(result['manifest']))`:
assert (manifest.name, digest) == EXPORTS[row['chapter']], 'import used another manifest than root named'
```

The four existing exports, read here: version 371, one chapter each, the job matching the row.

| Chapter | Rows (foreign/shared/system) | Job | Frames | `receipt_sha256` |
|---|---|---|---|---|
| 01_the_bump | 8,939 (3,467/3,203/2,269) | `css-reviewed-final-01-the-bump352` | 6,311 | `c09f7d31…` |
| 04_reading_color | 8,940 (3,468/3,203/2,269) | `css-reviewed-final-04-reading-color352` | 6,774 | `b5d4800b…` |
| 12_residues | 8,939 (3,468/3,202/2,269) | `css-reviewed-final-12-residues352` | 3,943 | `6ba9bf0f…` |
| 14_continuation | 8,939 (3,468/3,202/2,269) | `css-reviewed-final-14-continuation352` | 11,756 | `a3f237e9…` |

If a chapter is ever re-exported, its pin, and the controller version, must change with it.
