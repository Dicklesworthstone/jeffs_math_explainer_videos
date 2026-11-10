# BN-10 — Skip mode delivers the same updater time as playback (§9.3)

**Status:** Draft (W4, fm-x79). Consumed by the scene runtime (fm-5xm),
the segment-purity classifier (fm-3xk), and the Parity Ledger.

## The Reference's defect

Skipped playback in the Reference advances the segment in one step, then
double-applies its duration to dt-updaters. `progress_through_animations`
over the skip progression `[run_time]` calls `update_frame(run_time)` —
which increments time and runs scene updaters with `dt = run_time` — and
then `finish_animations` runs a **second** full-duration pass:

```python
def finish_animations(self, animations):
    for animation in animations:
        animation.finish()
        animation.clean_up_from_scene(self)
    if self.skip_animations:
        self.update_mobjects(self.get_run_time(animations))   # dt = run_time, again
    else:
        self.update_mobjects(0)
```

A played segment delivers `run_time` of total dt to scene updaters (one
frame at a time) plus a final `dt = 0` pass; a skipped segment delivers
`2 × run_time`. Any scene with a dt-updater (a clock readout, a physics
integration, a `TracedPath`) ends a skipped segment in a **different state**
than a played one — skipping (`-s`, `skip_animations`) is supposed to be a
preview accelerator, not a state change.

## The ruling

FrankenManim runs the same frame-grid state transitions under playback and
skip. Skip disables capture and emission; it does **not** collapse N updater
steps into one large `dt`. The distinction is essential: one large step
preserves `sum(dt)` but changes any nonlinear integrator, state machine, or
updater whose result depends on invocation count. Bit-identical terminal
arena state requires the same ordered calls.

The `finish_animations` pass then runs at `dt = 0` in **both** modes. Total
updater time and the sequence of updater inputs are identical, while all
renderer and emitter work remains absent. The frame order itself (steps 1–6),
the no-capture/no-emit skip behavior, and the `dt = 0` finish pass are all
kept exactly.

Locked by the update-order corpus:
`crates/fmn-anim/tests/frame_order.rs::skip_mode_matches_played_final_state_and_emits_nothing`.

**Migration:** scenes that (knowingly or not) relied on skipped segments
running their dt-updaters at double speed or as one coarse integration step
will now see the same terminal state as ordinary playback. There is no way to
ask for either defective behavior.

## The skipped clock (Appendix C-28)

The Reference's clock also moves differently when skipping. A played segment
advances over `arange(0, run_time, 1/fps) + 1/fps` and ends on the first frame
at or after `run_time` (scene.py:477). A skipped one advances by `run_time`
itself (scene.py:474). So `self.time` after `wait(0.25)` at 30 fps is 0.2667
played and 0.25 under `-s`. Measured on `_2020/hamming.py`
`PowerOfTwoPositions`, which waits 0.25 four times: 3.067 s played, 3.000 s
skipped. FrankenManim's skipped segment lands on the same frame as playback
(BN-02), so a scene's clock does not depend on `-s`.

The corpus differential runs the Reference under `-s`. Its `time` fact is the
Reference's playback clock: `structural_facts.py` adds, per skipped segment,
the rounding playback would add, without changing the Reference's behavior.

**Migration:** none. A scene that reads `self.time` sees the played value
whether or not it is skipped.
