# complex-picture-repair-06-10 progress (LavenderPike, Mail 1654)

Frozen for root. See `README.md`.

- Baselines copied from canonical at 13:19Z and rechecked before delivery; see `checks/baseline-hashes.sha256`.
- Built in this order: helper patch, then chapters 06, 07, 08, 09, 10, then the supplement lists and storyboard cells.
- Corrections made before delivery, each found by a check or a source read:
  - `carded`'s padding (0.5 x 0.35) moved nine card placements off their rings. `checks/geometry_checks.py` now tests cards against rings.
  - `MAP_TEX` and `THE_RATIO` repeated strings already on the lists, so the chapter reuses `two_planes::FN_TEX[0]` and `one_point::RATIO`.
  - `whole()` trimmed an integer's zeros (`"20"` to `"2"`), so it now trims only after a decimal point, with a test.
  - An early version of `source_checks.py` cut `one_derivative.rs` at its first `#[cfg(test)]`, skipping that chapter's waits. Fixed before its result was used: 80 waits checked became 106.
- `chain_arrows` line agreed with WhiteSwan: this folder's helper patch owns it (Mails 1747, 1759).
- Not done: any build, test, render, synthesis or listening. Canonical, Beads and git untouched.
