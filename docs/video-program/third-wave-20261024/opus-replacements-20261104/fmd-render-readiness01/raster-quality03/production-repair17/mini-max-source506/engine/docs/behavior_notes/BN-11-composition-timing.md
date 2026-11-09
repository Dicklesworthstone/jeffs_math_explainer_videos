# BN-11 — Composition honors what it was told (§9.4)

**Status:** Draft (W4, fm-hfe). Consumed by the scene runtime (fm-5xm),
the Studio's scrubbing (fm-yh0), the WASM timeline player (fm-oee), and the
Parity Ledger.

Two Reference defects in `manimlib/animation/composition.py`, both of the
same shape: a composition derives a number from its inputs and then ignores
it. Both are fixed; nothing else about the operators changes.

## C-10 — a group's `rate_func` and `time_span` are inert

`AnimationGroup.interpolate` overrides `Animation.interpolate` and consumes
raw alpha:

```python
def interpolate(self, alpha: float) -> None:
    time = alpha * self.max_end_time
    for anim, start_time, end_time in self.anims_with_timings:
        ...
        anim.interpolate(sub_alpha)
```

The constructor still *accepts* `rate_func=` and `time_span=` (they are
`Animation.__init__`'s parameters and `AnimationGroup` forwards `**kwargs`
to it), and `self.rate_func` is duly set — it is simply never read. So
`AnimationGroup(..., rate_func=there_and_back)` is a silent no-op, and so is
a group's `time_span`. Nesting compounds it: an outer group's curve cannot
reach an inner one, which is exactly what nesting is *for*.

**The ruling.** A composition's alpha runs the same normalized-alpha
pipeline every leaf animation runs — `time_spanned_alpha` then the rate
curve — before it becomes a position on the internal timeline. The group's
`lag_ratio` is **not** re-applied as a per-submobject lag: it is already
spent in the interval table (`build_animations_with_timings`), and applying
it twice would double-count.

The corollary that keeps the common case unchanged: **a composition's
`rate_func` defaults to `linear`, not `smooth`.** Members own their easing;
the group's curve shapes the composition's timeline. With the identity
default, a group built the ordinary way produces exactly the Reference's
member alphas — which the fixture corpus asserts, case for case, against the
Reference's own arithmetic.

Locked by
`crates/fmn-anim/tests/composition.rs::group_member_alphas_match_the_reference_corpus`
(the Reference's mapping, 1008 cases),
`::the_default_group_rate_func_is_the_identity`,
`::a_group_rate_func_shapes_the_composition`,
`::a_group_time_span_re_windows_the_composition`, and
`::nested_groups_compose_with_independent_rate_funcs`.

## C-11 — `Succession` ignores its members' run times, and drops the ones a coarse step passes

`Succession` derives its own `run_time` from its members' run times (it is
an `AnimationGroup` at `lag_ratio = 1`, so `max_end_time` is their sum), and
then picks the active member by **equal shares**:

```python
def interpolate(self, alpha: float) -> None:
    index, subalpha = integer_interpolate(0, len(self.animations), alpha)
```

`Succession(a(run_time=3), b(run_time=1))` therefore runs for 4 seconds and
gives each member 2 of them: `a` is rushed through at 1.5× and `b` crawls at
0.5×. The longer the spread, the worse it reads.

The same line drops members. When one frame's alpha step crosses more than
one member — a low fps, a short `run_time`, a scrub — the Reference jumps
straight to the target index, so every member in between is never begun and
never finished. Their effects (a remover's removal, a transform's end state)
simply do not happen.

**The ruling.** `Succession` maps alpha through the *same interval table*
every other operator uses, so a member's share of the composition is its own
run time; and it **walks** the active member forward one step at a time,
finishing each member it passes and beginning the next, so no member is ever
skipped. Just-in-time `begin` is kept exactly as the Reference has it — it
is the whole point of the operator, since member *k*'s starting copy must
freeze what member *k-1* left behind.

Because `interpolate` has no error channel, a member whose just-in-time
`begin` fails records the failure (`Animation::deferred_error`) and the
segment driver surfaces it by name at the end of the segment, rather than
continuing on stale state.

Locked by
`crates/fmn-anim/tests/composition.rs::succession_honours_member_run_times_where_the_reference_does_not`
(both answers per case, 168 cases — 31 of them divergent),
`::succession_walks_the_members_a_coarse_step_would_skip`, and
`::a_just_in_time_begin_failure_surfaces_from_the_segment`.

## Reverse curves and successive member lifetimes

An accepted group curve can move its internal time backward. The native
`Succession` previously located the earlier member but kept its active index
moving forward, so a `there_and_back` curve continued sampling the last
visited animation. Returning through separate mobjects left later objects
transformed; returning through a chain on one mobject used the wrong starting
copy.

Initialized members now retain bounded content checkpoints from before
`begin`, after preparation, and after their first `finish`. Returning to an
earlier window restores later members in reverse order, applies the completed
predecessors in author order, and samples the active member from its prepared
state. This preserves shared child identities, record schemas, render state,
and each just-in-time starting copy. It also works when successions are nested.
Each member's authored `begin` and `finish` callbacks run once per play, even
if the curve revisits it repeatedly. A begun member paused by a reverse curve
still receives its closing lifecycle before the composition finishes.
The existing member endpoint rule below still applies: a `there_and_back`
sequence may capture its returned-to-start picture and then leave the live
object at the selected member's own finish endpoint. That cleanup endpoint
is not an additional captured frame.

These checkpoints cover only the animated families. They do not restore the
whole Stage, reset time or scene roots, replace updater registrations, or undo
unrelated concurrent animation. Storage depends on initialized member families,
with CoW record and image payloads, rather than the number of rendered frames.
Ordinary forward sampling uses the existing live buffers; cached restoration
starts only on reverse traversal or when a parent re-enters a nested member.
An actual content restore detaches live record views under the ordinary V6 rule.
Restoring a checkpoint refuses a deleted original handle instead of reviving
it or rebinding a different object occupying its former slot.

Backward sampling requires locally replayable interpolation and no updaters on
the member's input/output families. Native pure transforms, fades, reveals, and
compositions of replayable members meet that contract. An unclassified callback
or updater-driven history cannot be undone by restoring drawable content; a
backward sample therefore reports `AnimError::StatefulSuccessionRewind` before
invoking that sample. Forward playback keeps its normal stateful semantics.
Rounding the last clock sample past alpha one does not count as a reversal:
the internal time is clipped to the composition's endpoint.

`Succession` is conservatively classified as a stateful **segment**, including
when its leaves support local backward sampling. Later members' starting copies
do not yet exist in the whole segment's begin snapshot. Treating that snapshot
as sufficient for independent frame interpolation previously reused an advanced
active index and stale future handles. `Timeline.seek` now uses its existing
checkpoint-and-serial-replay route for these segments, so arbitrary seek order
matches the serial captures. Ordinary eagerly initialized `AnimationGroup`
compositions retain their existing purity classification.

Locked by the `succession_there_and_back_rewinds_distinct_members_without_resetting_other_state`,
`succession_revisits_a_same_mobject_chain_from_its_original_member_inputs`,
`succession_reverse_sampling_does_not_repeat_begin_or_finish_callbacks`,
`nested_successions_restore_their_own_prepared_members`,
`stateful_succession_reverse_is_named_and_releases_its_active_lifecycle`, and
`succession_timeline_seek_matches_serial_frames_in_arbitrary_order` tests in
`crates/fmn-anim/tests/composition.rs`.

## What is *not* changed

A member still lands on **its own** `final_alpha_value` at `finish`; a
composition's `final_alpha_value` is not a second landing point. That is
load-bearing rather than incidental — `FadeOut` finishes at alpha 0 exactly
so a removed mobject is left in its original state — and a container that
overrode it would break every remover it contains. Removal likewise stays
each member's decision: `clean_up_from_scene` delegates, so a `FadeOut`
inside a group still leaves the scene and the container never does.

**Migration:** scenes whose look depended on `Succession`'s equal-share
timing will see members run at their declared speeds instead. A scene that
wants equal shares asks for it directly — give the members equal run times.
Group `rate_func`/`time_span` arguments that were previously ignored now
take effect; a group that should progress linearly needs no argument at all,
because linear is the default.
