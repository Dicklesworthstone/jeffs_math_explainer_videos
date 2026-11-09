//! fm-hv4 acceptance: adversarial completion order, bounded backpressure,
//! publication/torn-frame safety, multi-sink delivery, and explicit preview
//! drops.

use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_output::{
    CancelOutcome, EmitterConfig, EmitterError, FrameReservation, FrameSink, OrderedEmitter,
    SinkBinding, SinkFailure, SinkMode, SinkWrite,
};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

type RecordedFrames = Arc<Mutex<Vec<(u64, Vec<u8>)>>>;

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(PoisonError::into_inner)
}

fn layout() -> FrameLayout {
    FrameLayout::tight(PixelFormat::Rgba8, 8, 1).expect("valid test layout")
}

fn config(capacity: usize, first_sequence: u64) -> EmitterConfig {
    EmitterConfig::new(layout(), capacity, first_sequence).expect("valid emitter config")
}

fn fill(mut reservation: FrameReservation, byte: u8) -> FrameReservation {
    reservation.frame_mut().as_bytes_mut().fill(byte);
    reservation
}

#[derive(Clone)]
struct Recorder {
    frames: RecordedFrames,
}

impl FrameSink for Recorder {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        lock(&self.frames).push((sequence, frame.as_bytes().to_vec()));
        Ok(SinkWrite::Consumed)
    }
}

fn recorder(name: &str) -> (SinkBinding, RecordedFrames) {
    let frames = Arc::new(Mutex::new(Vec::new()));
    (
        SinkBinding::reliable(
            name,
            Recorder {
                frames: Arc::clone(&frames),
            },
        ),
        frames,
    )
}

fn wait_for_backpressure(emitter: &OrderedEmitter) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while emitter.stats().backpressure_waits == 0 {
        assert!(
            Instant::now() < deadline,
            "reservation did not reach the bounded wait"
        );
        thread::yield_now();
    }
}

#[test]
fn config_accounts_for_the_exact_preallocated_budget() {
    let layout = layout();
    let config = EmitterConfig::new(layout.clone(), 3, 17).expect("budget");
    assert_eq!(config.capacity(), 3);
    assert_eq!(config.first_sequence(), 17);
    assert_eq!(config.frame_bytes(), layout.total_bytes());
    assert_eq!(config.ring_bytes(), 3 * layout.total_bytes());
    assert_eq!(
        EmitterConfig::new(layout.clone(), 0, 0),
        Err(EmitterError::ZeroCapacity)
    );
    assert!(matches!(
        EmitterConfig::new(layout, usize::MAX, 0),
        Err(EmitterError::RingBudgetOverflow { .. })
    ));
}

#[test]
fn constructor_refuses_missing_or_unnamed_sinks() {
    assert!(matches!(
        OrderedEmitter::new(config(1, 0), Vec::new()),
        Err(EmitterError::NoSinks)
    ));
    let sink = SinkBinding::reliable("", |_sequence: u64, _frame: &FrameBuffer| {
        Ok(SinkWrite::Consumed)
    });
    assert!(matches!(
        OrderedEmitter::new(config(1, 0), vec![sink]),
        Err(EmitterError::EmptySinkName { index: 0 })
    ));
}

#[test]
fn adversarial_deposits_always_emit_the_contiguous_prefix() {
    const FIRST: u64 = 40;
    const COUNT: usize = 8;
    let (sink, frames) = recorder("record");
    let emitter = OrderedEmitter::new(config(COUNT, FIRST), vec![sink]).expect("emitter");

    let mut reservations = (0..COUNT)
        .map(|offset| {
            let sequence = FIRST + offset as u64;
            Some(fill(
                emitter.reserve(sequence).expect("ordered reserve"),
                sequence as u8,
            ))
        })
        .collect::<Vec<_>>();

    for index in [7, 3, 6, 1, 5, 2, 4, 0] {
        reservations[index]
            .take()
            .expect("one reservation")
            .publish()
            .expect("publish");
    }

    let report = emitter.finish().expect("drain");
    let frames = lock(&frames);
    assert_eq!(
        frames
            .iter()
            .map(|(sequence, _)| *sequence)
            .collect::<Vec<_>>(),
        (FIRST..FIRST + COUNT as u64).collect::<Vec<_>>()
    );
    for (sequence, bytes) in frames.iter() {
        assert!(bytes.iter().all(|byte| *byte == *sequence as u8));
    }
    assert_eq!(report.stats.emitted, COUNT as u64);
    assert_eq!(report.stats.max_outstanding, COUNT);
    assert_eq!(report.stats.outstanding, 0);
    assert_eq!(report.stats.available, COUNT);
}

struct GateSink {
    entered: Option<SyncSender<()>>,
    release: Receiver<()>,
    frames: RecordedFrames,
}

impl FrameSink for GateSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        if let Some(entered) = self.entered.take() {
            entered
                .send(())
                .map_err(|error| SinkFailure::new(error.to_string()))?;
            self.release
                .recv()
                .map_err(|error| SinkFailure::new(error.to_string()))?;
        }
        lock(&self.frames).push((sequence, frame.as_bytes().to_vec()));
        Ok(SinkWrite::Consumed)
    }
}

#[test]
fn slow_reliable_sink_backpressures_dispatch_without_deadlock() {
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let frames = Arc::new(Mutex::new(Vec::new()));
    let sink = SinkBinding::reliable(
        "slow",
        GateSink {
            entered: Some(entered_tx),
            release: release_rx,
            frames: Arc::clone(&frames),
        },
    );
    let emitter = OrderedEmitter::new(config(2, 0), vec![sink]).expect("emitter");

    let frame0 = fill(emitter.reserve(0).expect("reserve zero"), 0);
    let frame1 = fill(emitter.reserve(1).expect("reserve one"), 1);
    frame1.publish().expect("out-of-order publish");
    frame0.publish().expect("publish drain head");
    entered_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("sink entered");

    let handle = emitter.handle();
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let waiter = thread::spawn(move || {
        let result = handle
            .reserve(2)
            .map(|reservation| fill(reservation, 2))
            .and_then(FrameReservation::publish);
        done_tx.send(result).expect("test receiver alive");
    });

    wait_for_backpressure(&emitter);
    assert!(matches!(done_rx.try_recv(), Err(TryRecvError::Empty)));
    release_tx.send(()).expect("release slow sink");
    done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("waiter completed")
        .expect("third frame published");
    waiter.join().expect("waiter thread");

    let report = emitter.finish().expect("drain");
    assert_eq!(report.stats.backpressure_waits, 1);
    assert_eq!(report.stats.max_outstanding, 2);
    assert_eq!(
        lock(&frames)
            .iter()
            .map(|(sequence, _)| *sequence)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
}

struct ObservationSink {
    observed: mpsc::Sender<Vec<u8>>,
}

impl FrameSink for ObservationSink {
    fn write_frame(
        &mut self,
        _sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        self.observed
            .send(frame.as_bytes().to_vec())
            .map_err(|error| SinkFailure::new(error.to_string()))?;
        Ok(SinkWrite::Consumed)
    }
}

#[test]
fn sink_cannot_observe_a_frame_mid_write() {
    let (observed_tx, observed_rx) = mpsc::channel();
    let sink = SinkBinding::reliable(
        "observer",
        ObservationSink {
            observed: observed_tx,
        },
    );
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    let reservation = emitter.reserve(0).expect("reservation");
    let (half_tx, half_rx) = mpsc::sync_channel(1);
    let (continue_tx, continue_rx) = mpsc::sync_channel(1);

    let writer = thread::spawn(move || {
        let mut reservation = reservation;
        let bytes = reservation.frame_mut().as_bytes_mut();
        let middle = bytes.len() / 2;
        bytes[..middle].fill(0xaa);
        half_tx.send(()).expect("half signal");
        continue_rx.recv().expect("continue signal");
        bytes[middle..].fill(0x55);
        reservation.publish().expect("publication");
    });

    half_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("half written");
    assert!(matches!(observed_rx.try_recv(), Err(TryRecvError::Empty)));
    continue_tx.send(()).expect("finish frame");
    writer.join().expect("writer");
    emitter.finish().expect("drain");

    let observed = observed_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("one complete frame");
    let middle = observed.len() / 2;
    assert!(observed[..middle].iter().all(|byte| *byte == 0xaa));
    assert!(observed[middle..].iter().all(|byte| *byte == 0x55));
    assert!(matches!(
        observed_rx.try_recv(),
        Err(TryRecvError::Disconnected)
    ));
}

#[test]
fn every_reliable_sink_observes_the_same_order_and_bytes() {
    let (sink_a, frames_a) = recorder("a");
    let (sink_b, frames_b) = recorder("b");
    let emitter = OrderedEmitter::new(config(3, 10), vec![sink_a, sink_b]).expect("emitter");
    for sequence in 10..18 {
        fill(
            emitter.reserve(sequence).expect("ordered reservation"),
            sequence as u8,
        )
        .publish()
        .expect("publication");
    }
    let report = emitter.finish().expect("drain");
    assert_eq!(*lock(&frames_a), *lock(&frames_b));
    assert_eq!(report.sinks.len(), 2);
    assert!(
        report
            .sinks
            .iter()
            .all(|sink| sink.accepted == 8 && sink.dropped == 0)
    );
}

#[test]
fn preview_drops_are_explicit_counted_and_do_not_affect_reliable_sinks() {
    let (reliable, frames) = recorder("archive");
    let preview =
        SinkBinding::preview_dropping("preview", |sequence: u64, _frame: &FrameBuffer| {
            Ok(if sequence.is_multiple_of(2) {
                SinkWrite::Consumed
            } else {
                SinkWrite::WouldBlock
            })
        });
    let emitter = OrderedEmitter::new(config(2, 0), vec![reliable, preview]).expect("emitter");
    for sequence in 0..6 {
        fill(emitter.reserve(sequence).expect("reserve"), sequence as u8)
            .publish()
            .expect("publish");
    }
    let report = emitter.finish().expect("drain");
    assert_eq!(lock(&frames).len(), 6);
    assert_eq!(report.sinks[0].mode, SinkMode::Reliable);
    assert_eq!((report.sinks[0].accepted, report.sinks[0].dropped), (6, 0));
    assert_eq!(report.sinks[1].mode, SinkMode::PreviewDrop);
    assert_eq!((report.sinks[1].accepted, report.sinks[1].dropped), (3, 3));
}

#[test]
fn reliable_sink_busy_signal_fails_closed() {
    let sink = SinkBinding::reliable("must-block", |_sequence: u64, _frame: &FrameBuffer| {
        Ok(SinkWrite::WouldBlock)
    });
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    fill(emitter.reserve(0).expect("reserve"), 7)
        .publish()
        .expect("publish");
    let failure = emitter.finish().expect_err("reliable contract violation");
    assert!(matches!(
        failure.error,
        EmitterError::ReliableSinkWouldBlock { sequence: 0, .. }
    ));
    assert_eq!(failure.report.stats.outstanding, 0);
}

#[test]
fn cancellation_wakes_blocked_dispatch_and_reclaims_published_slot() {
    let (sink, _frames) = recorder("record");
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    let reservation = fill(emitter.reserve(0).expect("reserve"), 1);

    let handle = emitter.handle();
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let waiter = thread::spawn(move || {
        done_tx
            .send(handle.reserve(1).map(|_| ()))
            .expect("test receiver alive");
    });
    wait_for_backpressure(&emitter);
    emitter.cancel();
    assert_eq!(
        done_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("waiter woke"),
        Err(EmitterError::Cancelled)
    );
    waiter.join().expect("waiter thread");
    assert_eq!(reservation.publish(), Err(EmitterError::Cancelled));

    let failure = emitter.finish().expect_err("cancelled stream");
    assert_eq!(failure.error, EmitterError::Cancelled);
    assert_eq!(failure.report.stats.outstanding, 0);
    assert_eq!(failure.report.stats.available, 1);
}

#[test]
fn abandoned_reservation_is_terminal_and_releases_occupancy() {
    let (sink, _frames) = recorder("record");
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    drop(emitter.reserve(0).expect("reservation"));
    let failure = emitter.finish().expect_err("abandonment");
    assert_eq!(
        failure.error,
        EmitterError::ReservationAbandoned { sequence: 0 }
    );
    assert_eq!(failure.report.stats.outstanding, 0);
}

#[test]
fn sink_panics_are_contained_and_reported() {
    let sink = SinkBinding::reliable(
        "panics",
        |_sequence: u64, _frame: &FrameBuffer| -> Result<SinkWrite, SinkFailure> {
            std::panic::panic_any("deliberate sink panic")
        },
    );
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    fill(emitter.reserve(0).expect("reserve"), 1)
        .publish()
        .expect("publish");
    let failure = emitter.finish().expect_err("contained panic");
    assert!(matches!(
        failure.error,
        EmitterError::SinkPanicked {
            sequence: Some(0),
            ..
        }
    ));
    assert_eq!(failure.report.stats.outstanding, 0);
}

#[test]
fn finish_waits_for_the_missing_prefix_then_drains_in_order() {
    let (sink, frames) = recorder("record");
    let emitter = OrderedEmitter::new(config(2, 5), vec![sink]).expect("emitter");
    let first = fill(emitter.reserve(5).expect("first"), 5);
    let second = fill(emitter.reserve(6).expect("second"), 6);
    second.publish().expect("later publication");
    let handle = emitter.handle();
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let finisher = thread::spawn(move || {
        done_tx.send(emitter.finish()).expect("test receiver alive");
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    while !handle.stats().closed {
        assert!(Instant::now() < deadline, "finish did not close dispatch");
        thread::yield_now();
    }
    assert!(matches!(done_rx.try_recv(), Err(TryRecvError::Empty)));
    first.publish().expect("missing prefix publication");
    let report = done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("finish completed")
        .expect("successful drain");
    finisher.join().expect("finisher");
    assert_eq!(report.stats.emitted, 2);
    assert_eq!(
        lock(&frames)
            .iter()
            .map(|(sequence, _)| *sequence)
            .collect::<Vec<_>>(),
        vec![5, 6]
    );
}

#[test]
fn reservation_order_is_fail_closed_without_poisoning_the_stream() {
    let (sink, frames) = recorder("record");
    let emitter = OrderedEmitter::new(config(1, 9), vec![sink]).expect("emitter");
    assert_eq!(
        emitter.reserve(10).expect_err("gap"),
        EmitterError::UnexpectedSequence {
            expected: 9,
            actual: 10
        }
    );
    fill(emitter.reserve(9).expect("correct sequence"), 9)
        .publish()
        .expect("publish");
    emitter.finish().expect("drain");
    assert_eq!(lock(&frames).len(), 1);
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Lifecycle {
    writes: u64,
    finishes: u64,
    aborts: u64,
}

struct LifecycleSink {
    lifecycle: Arc<Mutex<Lifecycle>>,
    fail_finish: bool,
    panic_abort: bool,
}

impl FrameSink for LifecycleSink {
    fn write_frame(
        &mut self,
        _sequence: u64,
        _frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        lock(&self.lifecycle).writes += 1;
        Ok(SinkWrite::Consumed)
    }

    fn finish(&mut self) -> Result<(), SinkFailure> {
        lock(&self.lifecycle).finishes += 1;
        if self.fail_finish {
            Err(SinkFailure::new("deliberate finish failure"))
        } else {
            Ok(())
        }
    }

    fn abort(&mut self) {
        lock(&self.lifecycle).aborts += 1;
        assert!(!self.panic_abort, "deliberate abort panic");
    }
}

fn lifecycle_sink(
    name: &str,
    fail_finish: bool,
    panic_abort: bool,
) -> (SinkBinding, Arc<Mutex<Lifecycle>>) {
    let lifecycle = Arc::new(Mutex::new(Lifecycle::default()));
    (
        SinkBinding::reliable(
            name,
            LifecycleSink {
                lifecycle: Arc::clone(&lifecycle),
                fail_finish,
                panic_abort,
            },
        ),
        lifecycle,
    )
}

#[test]
fn successful_stream_finishes_without_aborting() {
    let (sink, lifecycle) = lifecycle_sink("artifact", false, false);
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    fill(emitter.reserve(0).expect("reserve"), 1)
        .publish()
        .expect("publish");
    emitter.finish().expect("successful finish");
    assert_eq!(
        *lock(&lifecycle),
        Lifecycle {
            writes: 1,
            finishes: 1,
            aborts: 0,
        }
    );
}

#[test]
fn cancellation_aborts_every_sink_without_finishing() {
    let (first, first_lifecycle) = lifecycle_sink("first", false, true);
    let (second, second_lifecycle) = lifecycle_sink("second", false, false);
    let emitter = OrderedEmitter::new(config(1, 0), vec![first, second]).expect("ordered emitter");
    emitter.cancel();
    let failure = emitter.finish().expect_err("cancelled");
    assert_eq!(failure.error, EmitterError::Cancelled);
    assert_eq!(
        *lock(&first_lifecycle),
        Lifecycle {
            writes: 0,
            finishes: 0,
            aborts: 1,
        }
    );
    assert_eq!(
        *lock(&second_lifecycle),
        Lifecycle {
            writes: 0,
            finishes: 0,
            aborts: 1,
        }
    );
}

#[test]
fn finish_failure_aborts_the_failing_and_remaining_sinks() {
    let (first, first_lifecycle) = lifecycle_sink("first", true, false);
    let (second, second_lifecycle) = lifecycle_sink("second", false, false);
    let emitter = OrderedEmitter::new(config(1, 0), vec![first, second]).expect("ordered emitter");
    fill(emitter.reserve(0).expect("reserve"), 1)
        .publish()
        .expect("publish");
    let failure = emitter.finish().expect_err("first finish fails");
    assert!(matches!(
        failure.error,
        EmitterError::SinkFailed { sequence: None, .. }
    ));
    assert_eq!(
        *lock(&first_lifecycle),
        Lifecycle {
            writes: 1,
            finishes: 1,
            aborts: 1,
        }
    );
    assert_eq!(
        *lock(&second_lifecycle),
        Lifecycle {
            writes: 1,
            finishes: 0,
            aborts: 1,
        }
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
struct FinalizationRace {
    prepares: u64,
    commits: u64,
    aborts: u64,
}

enum FinalizationGate {
    Prepare,
    Commit,
}

struct FinalizationGateSink {
    lifecycle: Arc<Mutex<FinalizationRace>>,
    gate: FinalizationGate,
    entered: Option<SyncSender<()>>,
    release: Receiver<()>,
}

impl FinalizationGateSink {
    fn wait_at_gate(&mut self) -> Result<(), SinkFailure> {
        if let Some(entered) = self.entered.take() {
            entered
                .send(())
                .map_err(|error| SinkFailure::new(error.to_string()))?;
            self.release
                .recv()
                .map_err(|error| SinkFailure::new(error.to_string()))?;
        }
        Ok(())
    }
}

impl FrameSink for FinalizationGateSink {
    fn write_frame(
        &mut self,
        _sequence: u64,
        _frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        Ok(SinkWrite::Consumed)
    }

    fn prepare_finish(&mut self) -> Result<(), SinkFailure> {
        lock(&self.lifecycle).prepares += 1;
        if matches!(self.gate, FinalizationGate::Prepare) {
            self.wait_at_gate()?;
        }
        Ok(())
    }

    fn commit_finish(&mut self) -> Result<(), SinkFailure> {
        lock(&self.lifecycle).commits += 1;
        if matches!(self.gate, FinalizationGate::Commit) {
            self.wait_at_gate()?;
        }
        Ok(())
    }

    fn abort(&mut self) {
        lock(&self.lifecycle).aborts += 1;
    }
}

fn finalization_gate_sink(
    gate: FinalizationGate,
) -> (
    SinkBinding,
    Arc<Mutex<FinalizationRace>>,
    Receiver<()>,
    SyncSender<()>,
) {
    let lifecycle = Arc::new(Mutex::new(FinalizationRace::default()));
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let binding = SinkBinding::reliable(
        "publication",
        FinalizationGateSink {
            lifecycle: Arc::clone(&lifecycle),
            gate,
            entered: Some(entered_tx),
            release: release_rx,
        },
    );
    (binding, lifecycle, entered_rx, release_tx)
}

#[test]
fn cancellation_during_prepare_wins_and_publication_never_runs() {
    let (sink, lifecycle, entered, release) = finalization_gate_sink(FinalizationGate::Prepare);
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    let handle = emitter.handle();
    let finisher = thread::spawn(move || emitter.finish());

    entered
        .recv_timeout(Duration::from_secs(2))
        .expect("prepare entered");
    assert_eq!(handle.cancel(), CancelOutcome::Won);
    release.send(()).expect("release prepare");

    let failure = finisher
        .join()
        .expect("finisher thread")
        .expect_err("cancellation wins");
    assert_eq!(failure.error, EmitterError::Cancelled);
    assert_eq!(
        *lock(&lifecycle),
        FinalizationRace {
            prepares: 1,
            commits: 0,
            aborts: 1,
        }
    );
}

#[test]
fn cancellation_after_commit_grant_is_too_late_and_cannot_change_success() {
    let (sink, lifecycle, entered, release) = finalization_gate_sink(FinalizationGate::Commit);
    let emitter = OrderedEmitter::new(config(1, 0), vec![sink]).expect("emitter");
    let handle = emitter.handle();
    let finisher = thread::spawn(move || emitter.finish());

    entered
        .recv_timeout(Duration::from_secs(2))
        .expect("commit entered");
    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(1);
    let canceller = thread::spawn(move || {
        cancelled_tx
            .send(handle.cancel())
            .expect("cancellation receiver");
    });
    assert!(
        matches!(
            cancelled_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ),
        "cancellation must wait behind the publication grant"
    );
    release.send(()).expect("release commit");

    finisher
        .join()
        .expect("finisher thread")
        .expect("publication succeeds");
    assert_eq!(
        cancelled_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("cancellation result"),
        CancelOutcome::TooLate
    );
    canceller.join().expect("canceller thread");
    assert_eq!(
        *lock(&lifecycle),
        FinalizationRace {
            prepares: 1,
            commits: 1,
            aborts: 0,
        }
    );
}
