# 506: mac-mini-max staging source tree (CC0)

cc0, 2026-11-09, 05:57 UTC. Q is this folder.

**`Q/mini-max-source506` is ready to transfer.** Its contents are meant to become `~/scratch/jmev-complex-native-20261109/source` on mac-mini-max, on the internal APFS volume (`~` is the owner account's home). The two changed Cargo files hold that path in full.

**Manifest.** The manifest is `mini-max-source506-manifest.json` (`2c63f5c8b559e5fdcb3b51ed280302d12b34b6d6f45f1923c219e8db8173acce`). It gives the path, sha256 and size of every staged file. The listing sha256 is `770e5239a547a6b81b079400c7a11d47a401b2c76a8e90f95746da55d3211cea`, computed over the sorted `"<sha256>  <path>\n"` lines. The tree holds 4,197 files, 49,004,935 bytes.

## What changed from 492

Only the root in two Cargo files changed, from `/Volumes/EERCHExport/jmev-complex-native-20261109/source` to the mac-mini-max root:

| File | Occurrences | 492 | 506 | Mapped back to CSD | 268 row |
|---|---|---|---|---|---|
| `engine-budget.toml` | 19 | `088dd6b6` | `116b317b` | `7277a6c0` | `7277a6c0` |
| `workspace/…/complex_analysis_visual/Cargo.toml` | 2 | `356eff0f` | `be1426d3` | `7644838e` | `7644838e` |

The 268 rows are from `root-csd-build268-inputs496.json` (`53f1360e`).

**The other 4,195 files** are byte-identical to 492, with the same modes. 492 binds each of them to that 268 manifest.

**No other file mentions either root.** I checked every file of 492 for its root first, and only these two contained it.

## Preserved

I re-hashed the 492 tree before and after the copy: all 4,197 files are at their 492 manifest rows, and the manifest is still `e5a6d12f`. Nothing in 492 or in the originals was written.

## For the transfer

- **Keep file modes.** 14 files are executable, listed in the manifest.
- **Build from the crate directory.** Run cargo from `workspace/explainer_corpus/complex_analysis_visual` with 268's flags, so that `.cargo/config.toml` applies.
- **Never commit this tree.** Q is not git-ignored, and the two Cargo files hold the full home path.

I made no SSH, build, transfer, launch or Beads change. No audio heard or checked.
