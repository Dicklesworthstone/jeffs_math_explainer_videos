# PROGRESS — __TITLE__

Update at every gate (ME-LEDGER). After a context reset, read this first.

| Gate | Status | Evidence (numbers, paths, host) |
|---|---|---|
| G0 brief, tools, host | | docs/brief.md; ftts x.y.z; ffmpeg encoders; host |
| G1 fact sheet = oracle | | docs/fact-sheet.md rows N/N (`./x __SLUG__ facts-check`) |
| G2 storyboard | | chapters, est. seconds each (`./x __SLUG__ storyboard`) |
| G2r storyboard review | | persona review R1 in `docs/reviews/__SLUG__/R1.md` |
| G3 script (lint, claims, de-slopify) | | lint PASS; lexicon verified (`./x __SLUG__ lint`) |
| G3r script review | | persona review R2 + mathematician audit in `docs/reviews/__SLUG__/R2.md` |
| G4 build, tests, clippy, glyph probe | | franken_manim commit (+dirty); probe PNG read (`./x __SLUG__ test`, `./x __SLUG__ probe`) |
| G5 visual drafts clean ×2 | | draft dir; defects fixed (`./x __SLUG__ draft`, `./x __SLUG__ reviewed draft`) |
| G6 narration QA + validator | | qa mean; validate PASS; narration dir + host (`./x __SLUG__ narrate`) |
| G7 narrated draft cue check | | cue PASS; chapter seconds; warnings reviewed (`./x __SLUG__ narrated`) |
| G7s score QA + listening log | | `stems/qa/*.json` PASS (`./x __SLUG__ score`) |
| G7r narrated review | | persona review R3 + owner OK to go 4K |
| G8 final 4K verified | | summary.txt; verify PASS; LUFS/TP; sheet read (`./x __SLUG__ final`, `./x __SLUG__ master`, `./x __SLUG__ verify`) |
| G9 delivered | | owner path; md5; README; beads |
| G10 published | | captions, metadata, thumbnail, release bundle |

## Hosts and paths

- Crate: `explainer_corpus/__SLUG__`
- Render host / dir:
- Narration dir (on which host):
- Final master:

## Open issues

-
