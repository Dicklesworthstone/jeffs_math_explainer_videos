//! Lamport's bakery lock on real hardware (bead mx-e-c-7v0.17; chapter 12's
//! "measured" card): customers are threads, the room is chapter 01's
//! scoreboard increment, and a lost point is the lock failing.
//!
//! ```text
//! bakery_stress [--threads N] [--entries PER_THREAD] [--runs R] [--modes seqcst,relacq,relaxed]
//! ```
//!
//! The lock is the 12_7 panel's code line for line, with the memory
//! orderings as a parameter: `seqcst` is the panel exactly; `relacq` makes
//! every store Release and every load Acquire; `relaxed` makes every access
//! Relaxed (the closest legal Rust to "run as written": plain shared
//! variables would be undefined behaviour). Slates are unpadded, as in the
//! panel. Inside the room each customer adds one to the scoreboard with a
//! separate read and write (a Relaxed load, then a Relaxed store: chapter
//! 01's two steps, adding no ordering of their own), so every visit the lock
//! fails to keep apart can lose a point: lost = entries − final score. A
//! lost point is a failure of mutual exclusion as the memory model defines
//! it (one visit's write not ordered before another's read); zero lost
//! points is no proof of correctness. Output is tab-separated, one line per
//! run plus a total per mode, headed by the host and the exact command.

use std::hint::spin_loop;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

/// The orderings of every store and every load of the lock, fixed at
/// compile time: each mode is its own monomorphized lock, the same machine
/// code as the panel with those orderings written in.
trait Orders {
    const STORE: Ordering;
    const LOAD: Ordering;
}

/// The panel exactly.
struct SeqCst;
impl Orders for SeqCst {
    const STORE: Ordering = Ordering::SeqCst;
    const LOAD: Ordering = Ordering::SeqCst;
}

struct RelAcq;
impl Orders for RelAcq {
    const STORE: Ordering = Ordering::Release;
    const LOAD: Ordering = Ordering::Acquire;
}

struct Relaxed;
impl Orders for Relaxed {
    const STORE: Ordering = Ordering::Relaxed;
    const LOAD: Ordering = Ordering::Relaxed;
}

/// A mode: its name in the record and its run.
type Mode = (&'static str, fn(usize, u64) -> Run);

/// The modes, in the order a record lists them.
const MODES: [Mode; 3] = [
    ("seqcst", run::<SeqCst>),
    ("relacq", run::<RelAcq>),
    ("relaxed", run::<Relaxed>),
];

/// The 12_7 panel's `lock`, its orderings a parameter.
fn lock<O: Orders>(i: usize, choosing: &[AtomicBool], number: &[AtomicU64]) {
    choosing[i].store(true, O::STORE);
    // each slate read once
    let t = 1 + number.iter().map(|n| n.load(O::LOAD)).max().unwrap();
    number[i].store(t, O::STORE);
    choosing[i].store(false, O::STORE);
    for j in (0..number.len()).filter(|&j| j != i) {
        while choosing[j].load(O::LOAD) {
            spin_loop()
        }
        loop {
            let nj = number[j].load(O::LOAD);
            // pairs never tie: j != i
            if nj == 0 || (nj, j) > (t, i) {
                break;
            }
            spin_loop()
        }
    }
}

/// The 12_7 panel's `unlock`.
fn unlock<O: Orders>(i: usize, number: &[AtomicU64]) {
    number[i].store(0, O::STORE)
}

/// One run's result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Run {
    entries: u64,
    score: u64,
}

impl Run {
    fn lost(self) -> u64 {
        self.entries - self.score
    }
}

/// `threads` customers enter the room `per_thread` times each, together.
fn run<O: Orders>(threads: usize, per_thread: u64) -> Run {
    let choosing: Arc<Vec<AtomicBool>> = Arc::new((0..threads).map(|_| AtomicBool::new(false)).collect());
    let number: Arc<Vec<AtomicU64>> = Arc::new((0..threads).map(|_| AtomicU64::new(0)).collect());
    let score = Arc::new(AtomicU64::new(0));
    let start = Arc::new(Barrier::new(threads));
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let (choosing, number, score, start) = (
                Arc::clone(&choosing),
                Arc::clone(&number),
                Arc::clone(&score),
                Arc::clone(&start),
            );
            thread::spawn(move || {
                start.wait();
                for _ in 0..per_thread {
                    lock::<O>(i, &choosing, &number);
                    // the room: read the score, then write it back plus one
                    let x = score.load(Ordering::Relaxed);
                    score.store(x + 1, Ordering::Relaxed);
                    unlock::<O>(i, &number);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("a customer thread panicked");
    }
    Run {
        entries: threads as u64 * per_thread,
        score: score.load(Ordering::SeqCst),
    }
}

/// The CPU, as the OS names it (no host names: the record is public).
fn cpu() -> String {
    let sysctl = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
    let linux = || {
        std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|m| m.trim().to_owned())
        })
    };
    sysctl.or_else(linux).unwrap_or_else(|| "unknown".into())
}

/// The load average when the run starts (the machine may be shared).
fn load_average() -> String {
    let sysctl = std::process::Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_matches(['{', '}', ' '])
                .to_owned()
        });
    let linux = || {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
    };
    sysctl.or_else(linux).unwrap_or_else(|| "unknown".into())
}

struct Args {
    threads: usize,
    per_thread: u64,
    runs: usize,
    modes: Vec<Mode>,
}

fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args {
        threads: 2,
        per_thread: 3_000_000,
        runs: 5,
        modes: MODES.to_vec(),
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        let bad = |e: std::num::ParseIntError| format!("{flag} {value}: {e}");
        match flag.as_str() {
            "--threads" => a.threads = value.parse().map_err(bad)?,
            "--entries" => a.per_thread = value.parse().map_err(bad)?,
            "--runs" => a.runs = value.parse().map_err(bad)?,
            "--modes" => {
                a.modes = value
                    .split(',')
                    .map(|m| {
                        MODES
                            .iter()
                            .find(|(name, _)| *name == m)
                            .copied()
                            .ok_or_else(|| format!("unknown mode {m}"))
                    })
                    .collect::<Result<_, _>>()?
            }
            _ => return Err(format!("unknown flag {flag}")),
        }
    }
    if a.threads < 2 || a.runs == 0 || a.per_thread == 0 {
        return Err("need --threads >= 2, --runs >= 1 and --entries >= 1".into());
    }
    Ok(a)
}

/// One record line: mode, threads, entries per thread, run (or "total"),
/// entries, score, lost.
fn line(mode: &str, threads: usize, per_thread: u64, run: &str, r: Run) -> String {
    format!(
        "{mode}\t{threads}\t{per_thread}\t{run}\t{}\t{}\t{}",
        r.entries,
        r.score,
        r.lost()
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a = match parse(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("bakery_stress: {e}");
            std::process::exit(2);
        }
    };
    println!("# bakery_stress: the 12_7 panel's bakery lock on real hardware (mx-e-c-7v0.17)");
    println!(
        "# cpu: {} ({}-{}), {} logical CPUs",
        cpu(),
        std::env::consts::ARCH,
        std::env::consts::OS,
        thread::available_parallelism().map_or(0, |n| n.get())
    );
    println!("# load average at start (1, 5, 15 min): {}", load_average());
    println!("# command: bakery_stress {}", args.join(" "));
    println!("mode\tthreads\tentries_per_thread\trun\tentries\tscore\tlost");
    for &(mode, run) in &a.modes {
        let mut total = Run { entries: 0, score: 0 };
        for k in 1..=a.runs {
            let r = run(a.threads, a.per_thread);
            println!("{}", line(mode, a.threads, a.per_thread, &k.to_string(), r));
            total.entries += r.entries;
            total.score += r.score;
        }
        println!("{}", line(mode, a.threads, a.per_thread, "total", total));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The body (between its outer braces) of the first function whose
    /// header contains `header`.
    fn body<'a>(src: &'a str, header: &str) -> &'a str {
        let at = src.find(header).unwrap_or_else(|| panic!("no {header:?}"));
        let open = at + src[at..].find('{').expect("a body");
        let mut depth = 0;
        for (i, c) in src[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return &src[open + 1..open + i];
                    }
                }
                _ => {}
            }
        }
        panic!("unbalanced braces after {header:?}")
    }

    /// Code with this file's ordering parameters read as the panel's
    /// SeqCst, and layout (whitespace, semicolons, the spin_loop path)
    /// ignored.
    fn tokens(code: &str) -> String {
        code.replace("O::STORE", "SeqCst")
            .replace("O::LOAD", "SeqCst")
            .replace("std::hint::", "")
            .chars()
            .filter(|c| !c.is_whitespace() && *c != ';')
            .collect()
    }

    #[test]
    fn the_lock_measured_is_the_storyboards_panel() {
        let board = include_str!("../../docs/storyboard.md");
        let ch12 = &board[board.find("## 12_your_cpu").expect("chapter 12")..];
        let panel: String = ch12
            .lines()
            .skip_while(|l| !l.starts_with("```rust"))
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n");
        let ours = include_str!("bakery_stress.rs");
        assert_eq!(
            tokens(body(ours, "fn lock<O: Orders>(")),
            tokens(body(&panel, "pub fn lock("))
        );
        assert_eq!(
            tokens(body(ours, "fn unlock<O: Orders>(")),
            tokens(body(&panel, "pub fn unlock("))
        );
    }

    #[test]
    fn alone_a_customer_takes_ticket_one_and_gives_it_back() {
        let choosing: Vec<AtomicBool> = (0..3).map(|_| AtomicBool::new(false)).collect();
        let number: Vec<AtomicU64> = (0..3).map(|_| AtomicU64::new(0)).collect();
        lock::<SeqCst>(1, &choosing, &number);
        assert_eq!(number[1].load(Ordering::SeqCst), 1);
        assert!(!choosing[1].load(Ordering::SeqCst));
        unlock::<SeqCst>(1, &number);
        assert_eq!(number[1].load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_later_customer_takes_one_more_and_waits_for_the_smaller_pair() {
        let choosing: Arc<Vec<AtomicBool>> = Arc::new((0..2).map(|_| AtomicBool::new(false)).collect());
        let number: Arc<Vec<AtomicU64>> = Arc::new((0..2).map(|_| AtomicU64::new(0)).collect());
        let inside = Arc::new(AtomicBool::new(false));
        // customer 0 is in the room with ticket 1
        lock::<SeqCst>(0, &choosing, &number);
        let second = {
            let (choosing, number, inside) = (Arc::clone(&choosing), Arc::clone(&number), Arc::clone(&inside));
            thread::spawn(move || {
                lock::<SeqCst>(1, &choosing, &number);
                inside.store(true, Ordering::SeqCst);
                let t = number[1].load(Ordering::SeqCst);
                unlock::<SeqCst>(1, &number);
                t
            })
        };
        // customer 1 reads ticket 1 and writes 2, then waits behind (1, 0)
        while number[1].load(Ordering::SeqCst) == 0 {
            spin_loop()
        }
        assert_eq!(number[1].load(Ordering::SeqCst), 2);
        thread::sleep(std::time::Duration::from_millis(50));
        assert!(!inside.load(Ordering::SeqCst), "customer 1 got in past a smaller pair");
        unlock::<SeqCst>(0, &number);
        assert_eq!(second.join().unwrap(), 2);
        assert!(inside.load(Ordering::SeqCst));
    }

    #[test]
    fn the_panel_as_written_loses_no_points_on_this_machine() {
        // SeqCst is the panel exactly: the proof says no point is ever lost
        let r = run::<SeqCst>(2, 50_000);
        assert_eq!(r.entries, 100_000);
        assert_eq!(r.lost(), 0, "{r:?}");
    }

    #[test]
    fn arguments_parse_and_bad_ones_are_refused() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let a = parse(&s(&[
            "--threads",
            "4",
            "--entries",
            "10",
            "--runs",
            "2",
            "--modes",
            "relaxed",
        ]))
        .unwrap();
        assert_eq!((a.threads, a.per_thread, a.runs, a.modes.len()), (4, 10, 2, 1));
        assert_eq!(a.modes[0].0, "relaxed");
        assert!(parse(&s(&["--threads", "1"])).is_err());
        assert!(parse(&s(&["--modes", "fast"])).is_err());
        assert!(parse(&s(&["--runs"])).is_err());
        let d = parse(&[]).unwrap();
        assert_eq!((d.threads, d.per_thread, d.runs, d.modes.len()), (2, 3_000_000, 5, 3));
    }

    #[test]
    fn a_record_line_carries_entries_score_and_lost() {
        let r = Run { entries: 10, score: 7 };
        assert_eq!(line("relaxed", 2, 5, "total", r), "relaxed\t2\t5\ttotal\t10\t7\t3");
    }
}
