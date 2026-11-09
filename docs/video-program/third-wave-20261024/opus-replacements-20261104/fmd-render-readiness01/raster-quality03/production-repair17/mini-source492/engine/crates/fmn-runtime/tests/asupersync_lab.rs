//! Deterministic-lab scheduler exploration for fm-3df.
//!
//! Asupersync stays a dev dependency: production frame work uses scoped std
//! threads, while the lab's seeded scheduler and virtual clock generate
//! replayable adversarial completion orders. Those orders drive gates around
//! the real pipeline callbacks, so ordered drain, barriers, and cancellation
//! are exercised in the implementation rather than in a duplicate model.

use asupersync::lab::{AutoAdvanceTermination, LabConfig, LabRuntime};
use asupersync::time::sleep;
use asupersync::types::{Budget, Time};
use fmn_platform::topology::HardwareTopology;
use fmn_runtime::{
    BarrierContext, CancellationToken, ExecutionPlan, FramePipeline, OutputPixelFormat,
    PipelineError, PipelineEvent, PipelineFailure, PipelineStages, PipelineStats, PlanRequest,
    RenderIntent, SurfaceSpec, TeamPlan,
};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::Duration;

const FRAME_COUNT: u64 = 6;
const WATCHDOG: Duration = Duration::from_secs(5);
static LAB_TEST_SERIAL: Mutex<()> = Mutex::new(());

fn seeded_completion_order(seed: u64) -> Vec<u64> {
    let completed = Arc::new(Mutex::new(Vec::new()));
    let config = LabConfig::new(seed).max_steps(10_000).with_auto_advance();
    let mut runtime = LabRuntime::new(config);
    let region = runtime.state.create_root_region(Budget::INFINITE);

    // The affine map is a permutation modulo six. Its offset and direction
    // come from the seed; unique virtual deadlines make every run replayable,
    // while the LabRuntime still explores how the waiting tasks are polled.
    let multiplier = if seed.is_multiple_of(2) { 1 } else { 5 };
    let offset = seed % FRAME_COUNT;
    for sequence in 0..FRAME_COUNT {
        let rank = (sequence * multiplier + offset) % FRAME_COUNT;
        let completed = Arc::clone(&completed);
        let (task, _) = runtime
            .state
            .create_task(region, Budget::INFINITE, async move {
                sleep(
                    Time::ZERO,
                    Duration::from_nanos((rank.saturating_add(1)) * 1_000),
                )
                .await;
                completed
                    .lock()
                    .expect("completion ledger is not poisoned")
                    .push(sequence);
            })
            .expect("lab task");
        runtime.scheduler.lock().schedule(task, 0);
    }

    let report = runtime.run_with_auto_advance();
    assert_eq!(report.termination, AutoAdvanceTermination::Quiescent);
    assert!(report.steps > 0);
    assert!(report.virtual_elapsed_nanos >= FRAME_COUNT * 1_000);
    let order = completed
        .lock()
        .expect("completion ledger is not poisoned")
        .clone();
    assert_eq!(order.len(), FRAME_COUNT as usize);
    assert_eq!(
        order.iter().copied().collect::<BTreeSet<_>>(),
        (0..FRAME_COUNT).collect()
    );
    order
}

struct CompletionGate {
    order: Vec<u64>,
    next: Mutex<usize>,
    started: Mutex<BTreeSet<u64>>,
    changed: Condvar,
}

impl CompletionGate {
    fn new(order: Vec<u64>) -> Self {
        Self {
            order,
            next: Mutex::new(0),
            started: Mutex::new(BTreeSet::new()),
            changed: Condvar::new(),
        }
    }

    fn complete(&self, sequence: u64, cancellation: &CancellationToken) {
        // `started` participates in the condition-variable predicate below, so
        // publish its transition while holding the mutex that `wait` releases.
        // Otherwise the final starter can notify between another worker's
        // predicate check and sleep, leaving every worker parked indefinitely.
        let mut next = self.next.lock().expect("completion gate is not poisoned");
        {
            self.started
                .lock()
                .expect("completion ledger is not poisoned")
                .insert(sequence);
        }
        self.changed.notify_all();
        while !cancellation.is_cancelled()
            && (self
                .started
                .lock()
                .expect("completion ledger is not poisoned")
                .len()
                < self.order.len()
                || self
                    .order
                    .get(*next)
                    .copied()
                    .is_some_and(|turn| turn != sequence))
        {
            next = self
                .changed
                .wait(next)
                .expect("completion gate is not poisoned");
        }
        if cancellation.is_cancelled() {
            return;
        }
        assert_eq!(
            self.order.get(*next),
            Some(&sequence),
            "the lab schedule omitted frame {sequence}"
        );
        *next += 1;
        self.changed.notify_all();
    }

    fn cancel_and_wake(&self, cancellation: &CancellationToken) {
        // Cancellation is part of the condition-variable predicate. Publish it
        // while holding the same mutex so a waiter cannot observe `false`,
        // lose this notification, and then sleep forever.
        let _next = self.next.lock().expect("completion gate is not poisoned");
        cancellation.cancel();
        self.changed.notify_all();
    }

    fn snapshot(&self) -> (usize, Vec<u64>) {
        let next = *self.next.lock().expect("completion gate is not poisoned");
        let started = self
            .started
            .lock()
            .expect("completion ledger is not poisoned")
            .iter()
            .copied()
            .collect();
        (next, started)
    }
}

struct LabStages {
    gate: Arc<CompletionGate>,
    cancellation: CancellationToken,
    cancel_on: Option<u64>,
}

impl PipelineStages for LabStages {
    type Frame = u64;
    type Prepared = u64;
    type Rasterized = u64;
    type Output = u64;
    type Error = &'static str;

    fn prepare(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
        Ok(frame)
    }

    fn rasterize(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
        self.gate.complete(frame, &self.cancellation);
        if self.cancel_on == Some(frame) {
            self.gate.cancel_and_wake(&self.cancellation);
        }
        Ok(frame)
    }

    fn convert(&self, frame: u64, _team: &TeamPlan) -> Result<u64, Self::Error> {
        Ok(frame)
    }
}

fn six_team_plan() -> ExecutionPlan {
    // The baseline divides each 64-CPU processor group into 32-core teams:
    // 192 synthetic physical cores therefore yield six independent leaders,
    // making every six-frame completion permutation admissible.
    let mut topology = HardwareTopology::fallback(192);
    topology.total_memory_bytes = Some(64 * 1024 * 1024 * 1024);
    let plan = ExecutionPlan::derive(
        PlanRequest::standard(
            RenderIntent::Offline,
            SurfaceSpec::lumen(64, 64),
            OutputPixelFormat::Rgba8,
        )
        .with_max_frames_in_flight(FRAME_COUNT as usize),
        &topology,
        None,
    )
    .expect("six-team fixture plan");
    assert_eq!(plan.frames_in_flight, FRAME_COUNT as usize);
    assert_eq!(plan.render_teams.len(), FRAME_COUNT as usize);
    plan
}

struct RunReceipt {
    result: Result<PipelineStats, PipelineFailure<&'static str>>,
    emitted: Vec<u64>,
    barrier_seen: bool,
}

fn run_with_watchdog(order: Vec<u64>, cancel_on: Option<u64>) -> RunReceipt {
    let plan = six_team_plan();
    let schedule = order.clone();
    let gate = Arc::new(CompletionGate::new(order));
    let cancellation = CancellationToken::new();
    let stages = Arc::new(LabStages {
        gate: Arc::clone(&gate),
        cancellation: cancellation.clone(),
        cancel_on,
    });
    let thread_cancellation = cancellation.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    let handle = std::thread::spawn(move || {
        let mut emitted = Vec::new();
        let barrier_seen = Arc::new(AtomicBool::new(false));
        let barrier_receipt = Arc::clone(&barrier_seen);
        let events = (0..FRAME_COUNT)
            .map(|sequence| PipelineEvent::frame(sequence, sequence))
            .chain([PipelineEvent::barrier("observe-pixels")]);
        let result = FramePipeline::with_cancellation(&plan, stages.as_ref(), thread_cancellation)
            .run(
                events,
                |sequence, output| {
                    assert_eq!(sequence, output);
                    emitted.push(sequence);
                    Ok(())
                },
                |name, context: BarrierContext| {
                    assert_eq!(name, "observe-pixels");
                    assert_eq!(context.submitted, FRAME_COUNT);
                    assert_eq!(context.emitted, FRAME_COUNT);
                    assert_eq!(context.outstanding_slots, 0);
                    barrier_receipt.store(true, Ordering::Release);
                    Ok(())
                },
            );
        let _ = sender.send(RunReceipt {
            result,
            emitted,
            barrier_seen: barrier_seen.load(Ordering::Acquire),
        });
    });

    match receiver.recv_timeout(WATCHDOG) {
        Ok(receipt) => {
            handle.join().expect("pipeline coordinator did not panic");
            receipt
        }
        Err(error) => {
            let before_cancel = gate.snapshot();
            gate.cancel_and_wake(&cancellation);
            let after_cancel = receiver.recv_timeout(Duration::from_secs(1));
            let after_wake = gate.snapshot();
            if after_cancel.is_ok() {
                handle.join().expect("pipeline coordinator did not panic");
            } else {
                drop(handle);
            }
            panic!(
                "pipeline failed the deterministic deadlock watchdog: {error}; \
                 schedule={schedule:?}, cancel_on={cancel_on:?}, \
                 gate_before={before_cancel:?}, gate_after={after_wake:?}, \
                 drained_after_external_cancel={}",
                after_cancel.is_ok()
            );
        }
    }
}

#[test]
fn seeded_adversarial_completions_drain_in_order_before_barriers() {
    let _serial = LAB_TEST_SERIAL
        .lock()
        .expect("lab test lock is not poisoned");
    let mut explored = BTreeSet::new();
    for seed in 0..12 {
        let order = seeded_completion_order(seed);
        assert_eq!(seeded_completion_order(seed), order, "seed {seed} replay");
        explored.insert(order.clone());

        let receipt = run_with_watchdog(order, None);
        let stats = receipt.result.expect("pipeline completes");
        assert_eq!(receipt.emitted, (0..FRAME_COUNT).collect::<Vec<_>>());
        assert!(receipt.barrier_seen);
        assert_eq!(stats.barriers, 1);
        assert_eq!(stats.outstanding_slots, 0);
        assert_eq!(stats.max_in_flight, FRAME_COUNT as usize);
    }
    assert!(
        explored.len() > 1,
        "the seed corpus must explore multiple completion schedules"
    );
}

#[test]
fn seeded_cancellation_releases_all_slots_and_skips_barrier() {
    let _serial = LAB_TEST_SERIAL
        .lock()
        .expect("lab test lock is not poisoned");
    for seed in 20..84 {
        let order = seeded_completion_order(seed);
        let cancel_on = order[2];
        let receipt = run_with_watchdog(order, Some(cancel_on));
        let failure = receipt.result.expect_err("pipeline is cancelled");
        assert!(matches!(failure.error, PipelineError::Cancelled));
        assert_eq!(failure.stats.outstanding_slots, 0);
        assert!(failure.stats.max_in_flight <= FRAME_COUNT as usize);
        assert!(!receipt.barrier_seen);
    }
}
