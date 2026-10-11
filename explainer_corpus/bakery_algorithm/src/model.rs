//! Video C's one model (plan §11.7): every lane, slate, staircase and step
//! clock in the video reads it.
//!
//! - [`Bakery`] is a port of the research model checker's transition
//!   relation (`docs/research/bakery_algorithm/scripts/bakery_mc.py`):
//!   Lamport's algorithm and its broken variants, atomic or safe slates,
//!   with the checker's own step labels, so the oracle's schedules replay
//!   here verbatim ([`replay`], [`Trace`]). The tests compare every replayed
//!   state with the checker's recorded output (`outputs/scenarios_out.txt`).
//! - [`orders`] is a port of the oracle's own interleaving engine
//!   (`oracle.py`, `bakery_ops` / `apply_op` / `enumerate_paths`): every
//!   order of two short programs, as a staircase in the progress square
//!   (the scoreboard of 01, the door sign of 02, the bakery of 05).
//! - [`seated`] and [`service_orders`] are the oracle's five-customer runs of
//!   chapters 03–04 (`row_cf7`): Blue takes a ticket among the slates
//!   0, 3, 0, 0, 2, then Purple and Teal both take the same one.
//!
//! Customers are 1-based in labels and on screen (Lamport's numbering) and
//! 0-based in vectors, as in the checker.

use std::collections::{BTreeSet, HashSet, VecDeque};

// ------------------------------------------------------------------ the checker

/// Where a customer is in Lamport's program (the checker's `pc`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pc {
    /// Outside the bakery (the non-critical section).
    Ncs,
    /// Reading the other slates for the biggest ticket.
    Rd,
    /// The double-fetch variant's second read of a slate.
    Rd2,
    /// About to write the ticket.
    Wn,
    /// About to lower the flag.
    Wc0,
    /// Waiting for customer `a`'s flag to be down.
    L2,
    /// Waiting for customer `a`'s pair to be bigger (or their slate empty).
    L3,
    /// In the room.
    Cs,
}

impl Pc {
    #[cfg(test)]
    fn name(self) -> &'static str {
        match self {
            Self::Ncs => "ncs",
            Self::Rd => "max-read",
            Self::Rd2 => "max-refetch",
            Self::Wn => "write-num",
            Self::Wc0 => "choose:=0",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::Cs => "CS",
        }
    }
}

/// One customer's local state (the checker's `(pc, a, m, wp, rest)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Local {
    pub pc: Pc,
    /// `Rd`: the bitmask of slates still to read; `Rd2`: the slate being
    /// re-read; `L2`/`L3`: the customer waited on (0-based).
    pub a: usize,
    /// The biggest ticket read so far.
    pub m: u32,
    /// A safe-slate write has begun and not ended.
    pub wp: bool,
    /// `Rd2`: the slates still to read after this one.
    pub rest: usize,
}

impl Local {
    const fn at(pc: Pc, a: usize, m: u32) -> Self {
        Self {
            pc,
            a,
            m,
            wp: false,
            rest: 0,
        }
    }
}

/// The whole bakery: every customer's local state, the flags and the slates.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub local: Vec<Local>,
    pub choosing: Vec<u32>,
    pub number: Vec<u32>,
}

/// Which algorithm runs (the checker's variants).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Lamport's algorithm.
    Correct,
    /// No flags: the take-a-number rule alone (chapter 05's crack).
    NoChoosing,
    /// Wait only for strictly smaller tickets, no tie-break (04_3, "both in").
    StrictNums,
    /// Wait for smaller or equal tickets (04_3, "after you").
    LeNums,
    /// The max reads each slate twice, once to compare and once to copy (10_4).
    DoubleFetch,
}

/// The checker's model: `n` customers, tickets capped at `k`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bakery {
    pub n: usize,
    pub k: u32,
    pub variant: Variant,
    /// Safe slates: a glance during a write may return any value.
    pub safe: bool,
    /// The max may read the other slates in any order (else index order).
    pub any_order: bool,
}

/// One enabled move: who moves, the checker's label, the next state.
pub type Move = (usize, String, State);

impl Bakery {
    pub const fn new(n: usize, k: u32, variant: Variant, safe: bool, any_order: bool) -> Self {
        Self {
            n,
            k,
            variant,
            safe,
            any_order,
        }
    }

    pub fn initial(&self) -> State {
        State {
            local: vec![Local::at(Pc::Ncs, 0, 0); self.n],
            choosing: vec![0; self.n],
            number: vec![0; self.n],
        }
    }

    fn uses_choosing(&self) -> bool {
        self.variant != Variant::NoChoosing
    }

    fn others(&self, i: usize) -> impl Iterator<Item = usize> {
        (0..self.n).filter(move |&j| j != i)
    }

    fn writing_choosing(st: &State, j: usize) -> bool {
        let l = st.local[j];
        l.wp && matches!(l.pc, Pc::Ncs | Pc::Wc0)
    }

    fn writing_number(st: &State, j: usize) -> bool {
        let l = st.local[j];
        l.wp && matches!(l.pc, Pc::Wn | Pc::Cs)
    }

    /// What a glance at `j`'s flag can return.
    fn read_choosing(&self, st: &State, j: usize) -> Vec<u32> {
        if self.safe && Self::writing_choosing(st, j) {
            vec![0, 1]
        } else {
            vec![st.choosing[j]]
        }
    }

    /// What a glance at `j`'s slate can return.
    fn read_number(&self, st: &State, j: usize) -> Vec<u32> {
        if self.safe && Self::writing_number(st, j) {
            (0..=self.k).collect()
        } else {
            vec![st.number[j]]
        }
    }

    /// Must customer `i` (ticket `ni`) wait for customer `j` (ticket `nj`)?
    pub fn must_wait(&self, nj: u32, j: usize, ni: u32, i: usize) -> bool {
        if nj == 0 {
            return false;
        }
        match self.variant {
            Variant::StrictNums => nj < ni,
            Variant::LeNums => nj <= ni,
            _ => (nj, j) < (ni, i),
        }
    }

    fn first_wait(&self, i: usize) -> (Pc, usize) {
        match self.others(i).next() {
            Some(j) => (if self.uses_choosing() { Pc::L2 } else { Pc::L3 }, j),
            None => (Pc::Cs, 0),
        }
    }

    fn next_wait(&self, i: usize, j: usize) -> (Pc, usize) {
        match self.others(i).find(|&k| k > j) {
            Some(k) => (if self.uses_choosing() { Pc::L2 } else { Pc::L3 }, k),
            None => (Pc::Cs, 0),
        }
    }

    /// Every enabled move, in the checker's order. A wait that cannot pass
    /// is a move that leaves the state unchanged (a self-loop), as there.
    pub fn successors(&self, st: &State) -> Vec<Move> {
        let mut out = Vec::new();
        let with = |i: usize, l: Local, ch: Option<(usize, u32)>, nu: Option<(usize, u32)>| {
            let mut s = st.clone();
            s.local[i] = l;
            if let Some((k, x)) = ch {
                s.choosing[k] = x;
            }
            if let Some((k, x)) = nu {
                s.number[k] = x;
            }
            s
        };
        let writing = |pc: Pc| Local {
            pc,
            a: 0,
            m: 0,
            wp: true,
            rest: 0,
        };
        let end = if self.safe { "end " } else { "" };
        for i in 0..self.n {
            let Local { pc, a, m, wp, rest } = st.local[i];
            let all: usize = self.others(i).map(|j| 1 << j).sum();
            match pc {
                Pc::Ncs if !self.uses_choosing() => {
                    out.push((i, "start".to_owned(), with(i, Local::at(Pc::Rd, all, 0), None, None)));
                }
                Pc::Ncs if self.safe && !wp => {
                    out.push((i, "begin choosing:=1".to_owned(), with(i, writing(Pc::Ncs), None, None)));
                }
                Pc::Ncs => {
                    let l = Local::at(Pc::Rd, all, 0);
                    out.push((i, format!("{end}choosing:=1"), with(i, l, Some((i, 1)), None)));
                }
                Pc::Rd => {
                    if a == 0 {
                        out.push((i, format!("max={m}"), with(i, Local::at(Pc::Wn, 0, m), None, None)));
                        continue;
                    }
                    let idxs: Vec<usize> = (0..self.n).filter(|&j| a >> j & 1 == 1).collect();
                    let idxs = if self.any_order { &idxs[..] } else { &idxs[..1] };
                    for &j in idxs {
                        for x in self.read_number(st, j) {
                            let rem = a & !(1 << j);
                            if self.variant == Variant::DoubleFetch && x > m {
                                let l = Local {
                                    pc: Pc::Rd2,
                                    a: j,
                                    m,
                                    wp: false,
                                    rest: rem,
                                };
                                out.push((
                                    i,
                                    format!("read number[{}]={x} (> t: re-fetch)", j + 1),
                                    with(i, l, None, None),
                                ));
                            } else {
                                let nm = m.max(x);
                                let l = if rem == 0 {
                                    Local::at(Pc::Wn, 0, nm)
                                } else {
                                    Local::at(Pc::Rd, rem, nm)
                                };
                                out.push((i, format!("read number[{}]={x}", j + 1), with(i, l, None, None)));
                            }
                        }
                    }
                }
                Pc::Rd2 => {
                    for x in self.read_number(st, a) {
                        let l = if rest == 0 {
                            Local::at(Pc::Wn, 0, x)
                        } else {
                            Local::at(Pc::Rd, rest, x)
                        };
                        out.push((
                            i,
                            format!("re-fetch number[{}]={x} -> t", a + 1),
                            with(i, l, None, None),
                        ));
                    }
                }
                Pc::Wn => {
                    let t = m + 1;
                    if t > self.k {
                        continue; // pruned: a ticket past the bound
                    }
                    if self.safe && !wp {
                        let l = Local {
                            pc: Pc::Wn,
                            a: 0,
                            m,
                            wp: true,
                            rest: 0,
                        };
                        out.push((i, format!("begin number[{}]:={t}", i + 1), with(i, l, None, None)));
                    } else {
                        let l = if self.uses_choosing() {
                            Local::at(Pc::Wc0, 0, 0)
                        } else {
                            let (p, j) = self.first_wait(i);
                            Local::at(p, j, 0)
                        };
                        out.push((
                            i,
                            format!("{end}number[{}]:={t}", i + 1),
                            with(i, l, None, Some((i, t))),
                        ));
                    }
                }
                Pc::Wc0 => {
                    if self.safe && !wp {
                        out.push((i, "begin choosing:=0".to_owned(), with(i, writing(Pc::Wc0), None, None)));
                    } else {
                        let (p, j) = self.first_wait(i);
                        let l = Local::at(p, j, 0);
                        out.push((i, format!("{end}choosing:=0"), with(i, l, Some((i, 0)), None)));
                    }
                }
                Pc::L2 => {
                    let mut seen: Vec<u32> = self.read_choosing(st, a);
                    seen.sort_unstable();
                    seen.dedup();
                    for x in seen {
                        if x != 0 {
                            out.push((i, format!("L2: choosing[{}]={x}, wait", a + 1), st.clone()));
                        } else {
                            let l = Local::at(Pc::L3, a, 0);
                            out.push((i, format!("L2: choosing[{}]=0, pass", a + 1), with(i, l, None, None)));
                        }
                    }
                }
                Pc::L3 => {
                    let ni = st.number[i];
                    let mut seen: Vec<u32> = self.read_number(st, a);
                    seen.sort_unstable();
                    seen.dedup();
                    for x in seen {
                        if self.must_wait(x, a, ni, i) {
                            out.push((i, format!("L3: number[{}]={x}, wait", a + 1), st.clone()));
                        } else {
                            let (p, j) = self.next_wait(i, a);
                            let enter = if p == Pc::Cs { " -> ENTER CS" } else { "" };
                            let label = format!("L3: number[{}]={x}, pass{enter}", a + 1);
                            out.push((i, label, with(i, Local::at(p, j, 0), None, None)));
                        }
                    }
                }
                Pc::Cs => {
                    if self.safe && !wp {
                        let label = format!("begin number[{}]:=0 (exit)", i + 1);
                        out.push((i, label, with(i, writing(Pc::Cs), None, None)));
                    } else {
                        let label = format!("{end}number[{}]:=0 (exit)", i + 1);
                        out.push((i, label, with(i, Local::at(Pc::Ncs, 0, 0), None, Some((i, 0)))));
                    }
                }
            }
        }
        out
    }

    /// The customers in the room (0-based).
    pub fn in_room(&self, st: &State) -> Vec<usize> {
        (0..self.n)
            .filter(|&i| st.local[i].pc == Pc::Cs && !st.local[i].wp)
            .collect()
    }

    /// The checker's one-line state (`fmt_state`), for the tests.
    #[cfg(test)]
    pub fn describe(&self, st: &State) -> String {
        let parts: Vec<String> = st
            .local
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let mut s = format!("P{}:{}", i + 1, l.pc.name());
                if matches!(l.pc, Pc::L2 | Pc::L3) {
                    s += &format!("(j={})", l.a + 1);
                }
                if matches!(l.pc, Pc::Rd | Pc::Wn | Pc::Rd2) {
                    s += &format!("(m={})", l.m);
                }
                if l.wp {
                    s += "[writing]";
                }
                s
            })
            .collect();
        let list = |v: &[u32]| v.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
        format!(
            "{} || choosing=[{}] number=[{}]",
            parts.join(" | "),
            list(&st.choosing),
            list(&st.number)
        )
    }
}

/// One replayed step.
#[derive(Clone, Debug)]
pub struct Step {
    /// Who moved, 1-based.
    pub customer: usize,
    pub label: String,
    pub before: State,
    pub after: State,
}

impl Step {
    /// A wait that could not pass (its state is unchanged).
    pub fn blocked(&self) -> bool {
        self.label.ends_with(", wait")
    }

    /// Counted the 02_4 way: each read, write and successful wait is one
    /// step; the no-flag model's `start` moves and blocked waits are not.
    pub fn counted(&self) -> bool {
        self.label != "start" && !self.blocked()
    }

    pub fn enters(&self) -> bool {
        self.label.ends_with("ENTER CS")
    }

    /// `(customer, ticket)` if this step finished writing a nonzero ticket.
    pub fn ticket_written(&self) -> Option<(usize, u32)> {
        let rest = self.label.strip_prefix("end ").unwrap_or(&self.label);
        let rest = rest.strip_prefix("number[")?;
        let (who, value) = rest.split_once("]:=")?;
        let t: u32 = value.parse().ok()?;
        (t > 0).then_some((who.parse().ok()?, t))
    }
}

/// Take each `(customer, label substring)` step: exactly one of that
/// customer's enabled moves must match it (the checker's `replay`).
pub fn replay(model: &Bakery, start: State, schedule: &[(usize, String)]) -> Result<Vec<Step>, String> {
    let mut st = start;
    let mut out = Vec::with_capacity(schedule.len());
    for (p, sub) in schedule {
        let mut cands: Vec<(String, State)> = model
            .successors(&st)
            .into_iter()
            .filter(|(i, lab, _)| *i + 1 == *p && lab.contains(sub.as_str()))
            .map(|(_, lab, ns)| (lab, ns))
            .collect();
        if cands.len() != 1 {
            return Err(format!("step P{p} {sub:?} matched {} moves", cands.len()));
        }
        let (label, next) = cands.remove(0);
        out.push(Step {
            customer: *p,
            label,
            before: st,
            after: next.clone(),
        });
        st = next;
    }
    Ok(out)
}

/// What a breadth-first search from the start found (the checker's `explore`).
#[derive(Clone, Debug)]
pub struct Search {
    /// Every reachable state (runs that would write a ticket past the bound
    /// are cut there).
    pub states: usize,
    /// The first run found that puts two customers in the room: a shortest
    /// one, step by step.
    pub violation: Option<Vec<Step>>,
}

/// Every state reachable from the start, breadth first in the checker's
/// order, and the first violation of "never two in the room" it meets.
pub fn explore(model: &Bakery) -> Search {
    let init = model.initial();
    let mut index = std::collections::HashMap::from([(init.clone(), 0usize)]);
    let mut states = vec![init];
    let mut parent: Vec<Option<(usize, usize, String)>> = vec![None];
    let mut q = VecDeque::from([0usize]);
    let mut bad = None;
    while let Some(sid) = q.pop_front() {
        let st = states[sid].clone();
        for (i, label, ns) in model.successors(&st) {
            let violating = bad.is_none() && model.in_room(&ns).len() >= 2;
            let nid = match index.get(&ns) {
                Some(&k) => k,
                None => {
                    let k = states.len();
                    index.insert(ns.clone(), k);
                    states.push(ns);
                    parent.push(Some((sid, i, label)));
                    q.push_back(k);
                    k
                }
            };
            if violating {
                bad = Some(nid);
            }
        }
    }
    let violation = bad.map(|mut sid| {
        let mut steps = Vec::new();
        while let Some((p, i, label)) = parent[sid].clone() {
            steps.push(Step {
                customer: i + 1,
                label,
                before: states[p].clone(),
                after: states[sid].clone(),
            });
            sid = p;
        }
        steps.reverse();
        steps
    });
    Search {
        states: states.len(),
        violation,
    }
}

fn sched(steps: &[(usize, &str)]) -> Vec<(usize, String)> {
    steps.iter().map(|&(p, s)| (p, s.to_owned())).collect()
}

/// The checker's named traces (`scenarios.py`; the oracle replays the same
/// schedules, copied from there).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trace {
    /// No flags: Blue decides on 1 unwritten, Green walks in, then Blue (05_1–05_3).
    A,
    /// The same order with flags: Green waits on Blue's flag (05_5–05_6).
    APrime,
    /// Numbers only, `<`: a tie lets both in.
    B1,
    /// Numbers only, `≤`: a tie, and both wait forever.
    B2,
    /// Equal tickets in the real algorithm.
    C,
    /// Three customers: two see 1 and both take 2.
    CThree,
    /// Four customers, 70 steps (07).
    D,
    /// Safe slates: a glance mid-write returns 7, so 8 is taken (09_6).
    E1,
    /// Safe slates: a glance catches Blue's old zero mid-write (10).
    E3,
    /// Tickets climb while the bakery never empties (11_3).
    Leapfrog,
}

impl Trace {
    pub fn model(self) -> Bakery {
        use Variant::*;
        match self {
            Self::A => Bakery::new(2, 5, NoChoosing, false, false),
            Self::APrime | Self::C => Bakery::new(2, 5, Correct, false, false),
            Self::B1 => Bakery::new(2, 5, StrictNums, false, false),
            Self::B2 => Bakery::new(2, 5, LeNums, false, false),
            Self::CThree => Bakery::new(3, 5, Correct, false, false),
            Self::D => Bakery::new(4, 6, Correct, false, true),
            Self::E1 | Self::E3 => Bakery::new(2, 9, Correct, true, false),
            Self::Leapfrog => Bakery::new(2, 7, Correct, false, false),
        }
    }

    pub fn schedule(self) -> Vec<(usize, String)> {
        match self {
            Self::A => sched(SCHED_A),
            Self::APrime => sched(SCHED_A2),
            Self::B1 => sched(SCHED_B),
            Self::B2 => {
                let mut s = sched(&SCHED_B[..8]);
                s.extend(sched(SCHED_B2_TAIL));
                s
            }
            Self::C => sched(SCHED_C),
            Self::CThree => sched(SCHED_C3),
            Self::D => sched(SCHED_D),
            Self::E1 => sched(SCHED_E1),
            Self::E3 => {
                let mut s = sched(SCHED_E1);
                s.extend(sched(SCHED_E3_TAIL));
                s
            }
            Self::Leapfrog => leapfrog_schedule(),
        }
    }

    /// The trace, replayed from the start.
    ///
    /// # Panics
    /// If a schedule step is not exactly one enabled move: the schedules are
    /// the checker's verified ones, so this is a porting bug the tests catch.
    pub fn steps(self) -> Vec<Step> {
        let m = self.model();
        replay(&m, m.initial(), &self.schedule()).expect("the checker's schedules replay step by step")
    }
}

const SCHED_A: &[(usize, &str)] = &[
    (1, "start"),
    (1, "read number[2]=0"),
    (2, "start"),
    (2, "read number[1]=0"),
    (2, "number[2]:=1"),
    (2, "L3: number[1]=0, pass"),
    (1, "number[1]:=1"),
    (1, "L3: number[2]=1, pass"),
];
const SCHED_A2: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (1, "read number[2]=0"),
    (2, "choosing:=1"),
    (2, "read number[1]=0"),
    (2, "number[2]:=1"),
    (2, "choosing:=0"),
    (2, "L2: choosing[1]=1, wait"),
    (1, "number[1]:=1"),
    (1, "choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, pass"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
    (1, "number[1]:=0"),
    (2, "L3: number[1]=0, pass"),
];
const SCHED_B: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (2, "choosing:=1"),
    (1, "read number[2]=0"),
    (2, "read number[1]=0"),
    (1, "number[1]:=1"),
    (2, "number[2]:=1"),
    (1, "choosing:=0"),
    (2, "choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, pass"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, pass"),
];
const SCHED_B2_TAIL: &[(usize, &str)] = &[
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, wait"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
    (1, "L3: number[2]=1, wait"),
    (2, "L3: number[1]=1, wait"),
];
const SCHED_C: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (2, "choosing:=1"),
    (1, "read number[2]=0"),
    (2, "read number[1]=0"),
    (1, "number[1]:=1"),
    (2, "number[2]:=1"),
];
const SCHED_C3: &[(usize, &str)] = &[
    (3, "choosing:=1"),
    (3, "read number[1]=0"),
    (3, "read number[2]=0"),
    (3, "number[3]:=1"),
    (3, "choosing:=0"),
    (1, "choosing:=1"),
    (2, "choosing:=1"),
    (1, "read number[2]=0"),
    (2, "read number[1]=0"),
    (1, "read number[3]=1"),
    (2, "read number[3]=1"),
    (1, "number[1]:=2"),
    (2, "number[2]:=2"),
];
const SCHED_D: &[(usize, &str)] = &[
    (3, "choosing:=1"),
    (2, "choosing:=1"),
    (3, "read number[1]=0"),
    (3, "read number[2]=0"),
    (3, "read number[4]=0"),
    (2, "read number[1]=0"),
    (2, "read number[3]=0"),
    (3, "number[3]:=1"),
    (3, "choosing:=0"),
    (3, "L2: choosing[1]=0, pass"),
    (3, "L3: number[1]=0, pass"),
    (3, "L2: choosing[2]=1, wait"),
    (2, "read number[4]=0"),
    (2, "number[2]:=1"),
    (2, "choosing:=0"),
    (4, "choosing:=1"),
    (4, "read number[1]=0"),
    (4, "read number[2]=1"),
    (4, "read number[3]=1"),
    (4, "number[4]:=2"),
    (4, "choosing:=0"),
    (3, "L2: choosing[2]=0, pass"),
    (3, "L3: number[2]=1, wait"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=0, pass"),
    (2, "L3: number[3]=1, pass"),
    (2, "L2: choosing[4]=0, pass"),
    (2, "L3: number[4]=2, pass"),
    (1, "choosing:=1"),
    (1, "read number[2]=1"),
    (1, "read number[3]=1"),
    (1, "read number[4]=2"),
    (1, "number[1]:=3"),
    (1, "choosing:=0"),
    (2, "number[2]:=0"),
    (3, "L3: number[2]=0, pass"),
    (3, "L2: choosing[4]=0, pass"),
    (3, "L3: number[4]=2, pass"),
    (4, "L2: choosing[1]=0, pass"),
    (4, "L3: number[1]=3, pass"),
    (4, "L2: choosing[2]=0, pass"),
    (4, "L3: number[2]=0, pass"),
    (4, "L2: choosing[3]=0, pass"),
    (4, "L3: number[3]=1, wait"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=0, pass"),
    (1, "L2: choosing[3]=0, pass"),
    (1, "L3: number[3]=1, wait"),
    (3, "number[3]:=0"),
    (4, "L3: number[3]=0, pass"),
    (1, "L3: number[3]=0, pass"),
    (1, "L2: choosing[4]=0, pass"),
    (1, "L3: number[4]=2, wait"),
    (2, "choosing:=1"),
    (2, "read number[1]=3"),
    (2, "read number[3]=0"),
    (2, "read number[4]=2"),
    (2, "number[2]:=4"),
    (2, "choosing:=0"),
    (4, "number[4]:=0"),
    (1, "L3: number[4]=0, pass"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=3, wait"),
    (1, "number[1]:=0"),
    (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=0, pass"),
    (2, "L3: number[3]=0, pass"),
    (2, "L2: choosing[4]=0, pass"),
    (2, "L3: number[4]=0, pass"),
];
const SCHED_E1: &[(usize, &str)] = &[
    (1, "begin choosing:=1"),
    (1, "end choosing:=1"),
    (1, "read number[2]=0"),
    (1, "begin number[1]:=1"),
    (2, "begin choosing:=1"),
    (2, "end choosing:=1"),
    (2, "read number[1]=7"),
    (1, "end number[1]:=1"),
    (1, "begin choosing:=0"),
    (1, "end choosing:=0"),
    (2, "begin number[2]:=8"),
    (2, "end number[2]:=8"),
    (2, "begin choosing:=0"),
    (2, "end choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=8, pass"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
];
#[cfg(test)]
const SCHED_E2_TAIL: &[(usize, &str)] = &[
    (1, "begin number[1]:=0"),
    (2, "L3: number[1]=9, pass"),
    (1, "end number[1]:=0"),
];
const SCHED_E3_TAIL: &[(usize, &str)] = &[
    (1, "begin number[1]:=0"),
    (1, "end number[1]:=0"),
    (1, "begin choosing:=1"),
    (1, "end choosing:=1"),
    (1, "read number[2]=8"),
    (1, "begin number[1]:=9"),
    (2, "L3: number[1]=0, pass"),
    (1, "end number[1]:=9"),
    (1, "begin choosing:=0"),
    (1, "end choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=8, wait"),
];

/// `leapfrog.py`'s schedule: two customers overtake each other five times.
fn leapfrog_schedule() -> Vec<(usize, String)> {
    let mut s = sched(&[
        (1, "choosing:=1"),
        (1, "read number[2]=0"),
        (1, "number[1]:=1"),
        (1, "choosing:=0"),
        (1, "L2: choosing[2]=0, pass"),
        (1, "L3: number[2]=0, pass"),
    ]);
    for r in 0..5u32 {
        let (a, b) = if r % 2 == 0 { (2, 1) } else { (1, 2) };
        let nb = 1 + r;
        let na = nb + 1;
        s.extend([
            (a, "choosing:=1".to_owned()),
            (a, format!("read number[{b}]={nb}")),
            (a, format!("number[{a}]:={na}")),
            (a, "choosing:=0".to_owned()),
            (a, format!("L2: choosing[{b}]=0, pass")),
            (a, format!("L3: number[{b}]={nb}, wait")),
            (b, format!("number[{b}]:=0")),
            (a, format!("L3: number[{b}]=0, pass")),
        ]);
    }
    s
}

// ------------------------------------------- 03–04: five customers, seated

/// The slates of 03_4: customers 1–5 (Blue, Green, Purple, Teal, Pink).
pub const SLATES: [u32; 5] = [0, 3, 0, 0, 2];

/// The five-customer model of 03–04 (the oracle's `row_cf7`).
pub const FIVE: Bakery = Bakery::new(5, 6, Variant::Correct, false, false);

/// Green (2) and Pink (5) already hold 3 and 2 and wait in line; the others
/// are outside.
pub fn seated() -> State {
    let mut st = FIVE.initial();
    st.local[1] = Local::at(Pc::L2, 0, 0);
    st.local[4] = Local::at(Pc::L2, 0, 0);
    st.number = SLATES.to_vec();
    st
}

/// Customer `who`'s whole doorway (1-based), reads in index order:
/// raise the flag, read every other slate, write, lower the flag.
pub fn doorway(n: usize, who: usize) -> Vec<(usize, String)> {
    let mut s = vec![(who, "choosing:=1".to_owned())];
    s.extend(
        (1..=n)
            .filter(|&k| k != who)
            .map(|k| (who, format!("read number[{k}]"))),
    );
    s.push((who, format!("number[{who}]:=")));
    s.push((who, "choosing:=0".to_owned()));
    s
}

/// 03_4: Blue (1) runs her doorway from [`seated`].
pub fn blue_takes_a_ticket() -> Vec<Step> {
    replay(&FIVE, seated(), &doorway(5, 1)).expect("Blue's doorway is enabled step by step")
}

/// 04_2: from Blue's slates, Purple (3) and Teal (4) run their doorways side
/// by side, each read of one right after the same read of the other, so both
/// see the same biggest ticket and write the same one.
pub fn purple_and_teal_tie() -> Vec<Step> {
    let start = blue_takes_a_ticket().last().expect("a doorway has steps").after.clone();
    let (d3, d4) = (doorway(5, 3), doorway(5, 4));
    let mut s = Vec::new();
    for (a, b) in d3.into_iter().zip(d4) {
        s.push(a);
        s.push(b);
    }
    // the oracle's lockstep: both flags, the four reads in pairs, both
    // writes, both flags down (the zip above already pairs them that way)
    replay(&FIVE, start, &s).expect("the lockstep doorways are enabled step by step")
}

/// Every interleaving of `movers` (1-based) from `st`, nobody arriving anew:
/// the set of room-entry orders at the leaves (the oracle's
/// `model_service_orders`).
pub fn service_orders(model: &Bakery, st: &State, movers: &[usize]) -> BTreeSet<Vec<usize>> {
    let movers: HashSet<usize> = movers.iter().map(|p| p - 1).collect();
    let mut seen: HashSet<(State, Vec<usize>)> = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert((st.clone(), Vec::new()));
    q.push_back((st.clone(), Vec::new()));
    let mut orders = BTreeSet::new();
    while let Some((s, seq)) = q.pop_front() {
        let mut moved = false;
        for (i, label, ns) in model.successors(&s) {
            if !movers.contains(&i) || ns == s || s.local[i].pc == Pc::Ncs {
                continue;
            }
            moved = true;
            let mut seq2 = seq.clone();
            if label.ends_with("ENTER CS") {
                seq2.push(i + 1);
            }
            let key = (ns, seq2);
            if seen.insert(key.clone()) {
                q.push_back(key);
            }
        }
        if !moved {
            orders.insert(seq);
        }
    }
    orders
}

/// The tickets each customer read in a run of steps, in order (1-based keys).
pub fn reads_by(steps: &[Step], who: usize) -> Vec<u32> {
    steps
        .iter()
        .filter(|s| s.customer == who && s.label.starts_with("read number["))
        .filter_map(|s| s.label.rsplit_once('=').and_then(|(_, x)| x.parse().ok()))
        .collect()
}

// ------------------------------------------------- orders and staircases

/// How a ticket comparison treats the other customer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmp {
    /// Lamport's: compare (ticket, customer number) pairs.
    Lex,
    /// Wait only for strictly smaller tickets.
    Strict,
    /// Wait for smaller or equal tickets.
    Le,
}

/// One step of a short program (the oracle's ops). Memory is the scoreboard
/// alone (01), the door sign alone (02), or every flag then every slate
/// (the bakery).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// Read the scoreboard into a private copy.
    ReadScore,
    /// Write the private copy plus one.
    WriteScore,
    /// Wait until the sign says vacant (a successful wait is a step).
    AwaitVacant,
    /// Flip the sign to occupied, and go in.
    FlipOccupied,
    /// Flip it back on the way out.
    ExitSign,
    /// Raise (1) or lower (0) your own flag.
    SetFlag(u32),
    /// Read customer `j`'s slate into the running biggest.
    ReadMax(usize),
    /// Write one more than the biggest.
    WriteTicket,
    /// Wait until `j`'s flag is down.
    AwaitFlagDown(usize),
    /// Wait until `j` has no ticket or comes after you.
    AwaitTurn(usize, Cmp),
    /// Leave: wipe your slate.
    Exit,
}

/// The scoreboard's start (01_4; fact sheet C-F1).
pub const SCORE_START: u32 = 41;

/// 01: read the score, write it back plus one.
pub fn scoreboard_program() -> Vec<Op> {
    vec![Op::ReadScore, Op::WriteScore]
}

/// 02: check the sign says vacant, flip it to occupied and go in, flip it
/// back on the way out.
pub fn sign_program() -> Vec<Op> {
    vec![Op::AwaitVacant, Op::FlipOccupied, Op::ExitSign]
}

/// One visit to the bakery by customer `i` of `n` (0-based), with or
/// without the flag, comparing by `cmp`.
pub fn bakery_program(n: usize, i: usize, flag: bool, cmp: Cmp) -> Vec<Op> {
    let others: Vec<usize> = (0..n).filter(|&j| j != i).collect();
    let mut ops = Vec::new();
    if flag {
        ops.push(Op::SetFlag(1));
    }
    ops.extend(others.iter().map(|&j| Op::ReadMax(j)));
    ops.push(Op::WriteTicket);
    if flag {
        ops.push(Op::SetFlag(0));
    }
    for &j in &others {
        if flag {
            ops.push(Op::AwaitFlagDown(j));
        }
        ops.push(Op::AwaitTurn(j, cmp));
    }
    ops.push(Op::Exit);
    ops
}

/// Take `op` for customer `i` of `n`: the new memory and private value, or
/// `None` for a wait that cannot pass.
pub fn apply(op: Op, i: usize, n: usize, mem: &[u32], m: u32) -> Option<(Vec<u32>, u32)> {
    let set = |k: usize, x: u32| {
        let mut v = mem.to_vec();
        v[k] = x;
        v
    };
    match op {
        Op::ReadScore => Some((mem.to_vec(), mem[0])),
        Op::WriteScore => Some((set(0, m + 1), m)),
        Op::AwaitVacant => (mem[0] == 0).then(|| (mem.to_vec(), m)),
        Op::FlipOccupied => Some((set(0, 1), m)),
        Op::ExitSign => Some((set(0, 0), m)),
        Op::SetFlag(x) => Some((set(i, x), m)),
        Op::ReadMax(j) => Some((mem.to_vec(), m.max(mem[n + j]))),
        Op::WriteTicket => Some((set(n + i, m + 1), m)),
        Op::AwaitFlagDown(j) => (mem[j] == 0).then(|| (mem.to_vec(), m)),
        Op::AwaitTurn(j, cmp) => {
            let (nj, ni) = (mem[n + j], mem[n + i]);
            let blocked = nj != 0
                && match cmp {
                    Cmp::Lex => nj < ni || (nj == ni && j < i),
                    Cmp::Strict => nj < ni,
                    Cmp::Le => nj <= ni,
                };
            (!blocked).then(|| (mem.to_vec(), m))
        }
        Op::Exit => Some((set(n + i, 0), m)),
    }
}

/// One maximal order: who moved at each step, and where it ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    /// The customer (0-based) who took each step.
    pub moves: Vec<usize>,
    /// Every program ran to its end (else a deadlock).
    pub done: bool,
    /// Memory at the end.
    pub end: Vec<u32>,
    /// Some point had two or more customers "inside": about to take their
    /// program's last step (the room for the sign and the bakery; between
    /// the read and the write for the scoreboard).
    pub both_inside: bool,
}

impl Order {
    /// The lattice points of a two-customer order: (Blue's steps, Green's
    /// steps), from (0, 0).
    pub fn lattice(&self) -> Vec<(usize, usize)> {
        let mut at = (0, 0);
        let mut out = vec![at];
        for &i in &self.moves {
            if i == 0 {
                at.0 += 1;
            } else {
                at.1 += 1;
            }
            out.push(at);
        }
        out
    }

    /// The staircase's corners: start, every turn, end.
    pub fn corners(&self) -> Vec<(usize, usize)> {
        let pts = self.lattice();
        let mut out = vec![pts[0]];
        for k in 1..self.moves.len() {
            if self.moves[k] != self.moves[k - 1] {
                out.push(pts[k]);
            }
        }
        out.push(pts[self.moves.len()]);
        out
    }

    /// The order passes through lattice point `p`.
    pub fn visits(&self, p: (usize, usize)) -> bool {
        self.lattice().contains(&p)
    }

    /// The order ends (stuck) with both inside, for programs `len` long.
    pub fn end_both_inside(&self, len: usize) -> bool {
        self.lattice().last() == Some(&red_point(len))
    }
}

/// Every maximal order of `progs` from memory `mem0`, depth first with the
/// lower customer first (the oracle's `enumerate_paths`).
pub fn orders(progs: &[Vec<Op>], n: usize, mem0: &[u32]) -> Vec<Order> {
    struct Walk<'a> {
        progs: &'a [Vec<Op>],
        n: usize,
        pcs: Vec<usize>,
        ms: Vec<u32>,
        moves: Vec<usize>,
        out: Vec<Order>,
    }
    impl Walk<'_> {
        fn go(&mut self, mem: &[u32], inside: bool) {
            let mut moved = false;
            for i in 0..self.progs.len() {
                let p = &self.progs[i];
                if self.pcs[i] >= p.len() {
                    continue;
                }
                let Some((mem2, m2)) = apply(p[self.pcs[i]], i, self.n, mem, self.ms[i]) else {
                    continue;
                };
                moved = true;
                let m_old = self.ms[i];
                self.pcs[i] += 1;
                self.ms[i] = m2;
                self.moves.push(i);
                let now = self
                    .pcs
                    .iter()
                    .zip(self.progs)
                    .filter(|(pc, p)| **pc + 1 == p.len())
                    .count()
                    >= 2;
                self.go(&mem2, inside || now);
                self.moves.pop();
                self.ms[i] = m_old;
                self.pcs[i] -= 1;
            }
            if !moved {
                self.out.push(Order {
                    moves: self.moves.clone(),
                    done: self.pcs.iter().zip(self.progs).all(|(pc, p)| *pc >= p.len()),
                    end: mem.to_vec(),
                    both_inside: inside,
                });
            }
        }
    }
    let mut w = Walk {
        progs,
        n,
        pcs: vec![0; progs.len()],
        ms: vec![0; progs.len()],
        moves: Vec::new(),
        out: Vec::new(),
    };
    w.go(mem0, false);
    w.out
}

/// The scoreboard's six orders (01).
pub fn scoreboard_orders() -> Vec<Order> {
    orders(&[scoreboard_program(), scoreboard_program()], 2, &[SCORE_START])
}

/// The door sign's fourteen orders (02).
pub fn sign_orders() -> Vec<Order> {
    orders(&[sign_program(), sign_program()], 1, &[0])
}

/// Two customers, one visit each (05), with or without flags.
pub fn bakery_orders(flag: bool, cmp: Cmp) -> Vec<Order> {
    let progs = [bakery_program(2, 0, flag, cmp), bakery_program(2, 1, flag, cmp)];
    orders(&progs, 2, &[0; 4])
}

/// The lattice point where both are inside: both one step from the end.
pub fn red_point(len: usize) -> (usize, usize) {
    (len - 1, len - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The checker's recorded run of every scenario (`scenarios.py`).
    const RECORDED: &str = include_str!("../../../docs/research/bakery_algorithm/scripts/outputs/scenarios_out.txt");

    /// The recorded `(label, state)` lines of the scenario whose header starts
    /// with `prefix`.
    fn recorded(prefix: &str) -> Vec<(String, String)> {
        let mut inside = false;
        let mut out = Vec::new();
        for line in RECORDED.lines() {
            if line.starts_with('(') {
                if inside {
                    break;
                }
                inside = line.starts_with(prefix);
                continue;
            }
            let t = line.trim_start();
            let Some((k, rest)) = t.split_once(". P") else {
                continue;
            };
            if !inside || k.parse::<usize>().is_err() {
                continue;
            }
            // "P1: label<spaces>state[   <== TWO PROCESSES IN CS]"
            let rest = rest.trim_end_matches("   <== TWO PROCESSES IN CS");
            let (who_label, state) = rest.split_once(" P1:").expect("a state follows the label");
            let label = who_label.split_once(": ").expect("P<k>: label").1.trim_end();
            out.push((label.to_owned(), format!("P1:{state}")));
        }
        out
    }

    #[test]
    fn every_replayed_state_equals_the_checkers_recorded_output() {
        for (trace, prefix) in [
            (Trace::A, "(a) "),
            (Trace::APrime, "(a') "),
            (Trace::B1, "(b1) "),
            (Trace::B2, "(b2) "),
            (Trace::C, "(c) "),
            (Trace::CThree, "(c') "),
            (Trace::D, "(d) "),
            (Trace::E1, "(e1) "),
            (Trace::E3, "(e3) "),
        ] {
            check(&trace.model(), &trace.steps(), &recorded(prefix), &format!("{trace:?}"));
        }
        // (e2): the glance that overlaps the exit write (no chapter replays it)
        let m = Trace::E1.model();
        let mut e2 = Trace::E1.schedule();
        e2.extend(sched(SCHED_E2_TAIL));
        let steps = replay(&m, m.initial(), &e2).expect("e2 replays");
        check(&m, &steps, &recorded("(e2) "), "E2");
    }

    fn check(m: &Bakery, got: &[Step], want: &[(String, String)], name: &str) {
        assert_eq!(got.len(), want.len(), "{name}: step count");
        for (k, (s, (label, state))) in got.iter().zip(want).enumerate() {
            assert_eq!(&s.label, label, "{name} step {}", k + 1);
            assert_eq!(&m.describe(&s.after), state, "{name} step {}", k + 1);
            let before = if k == 0 { m.initial() } else { got[k - 1].after.clone() };
            assert_eq!(s.before, before, "{name} step {}: the state before", k + 1);
        }
    }

    #[test]
    fn the_traces_say_what_the_oracle_says() {
        // C-F6 (oracle row_cf6 and row_r2_counted_steps)
        let a = Trace::A.steps();
        assert_eq!(a.len(), 8);
        assert_eq!(a.iter().filter(|s| s.counted()).count(), 6);
        assert_eq!(Trace::A.model().in_room(&a[7].after).len(), 2);
        let a2 = Trace::APrime.steps();
        assert_eq!(a2.len(), 15);
        assert_eq!(a2.iter().filter(|s| s.counted()).count(), 13);
        let stop = a2
            .iter()
            .position(|s| s.customer == 2 && s.label.contains("choosing[1]=1, wait"))
            .map(|k| k + 1);
        assert_eq!(stop, Some(7));
        let m = Trace::APrime.model();
        assert!(a2.iter().all(|s| m.in_room(&s.after).len() <= 1));
        let b1 = Trace::B1.steps();
        assert_eq!(Trace::B1.model().in_room(&b1[11].after).len(), 2);
        let b2 = Trace::B2.steps();
        let end = &b2.last().expect("steps").after;
        let succ = Trace::B2.model().successors(end);
        assert!(!succ.is_empty() && succ.iter().all(|(_, _, ns)| ns == end));
        assert_eq!(Trace::C.steps()[5].after.number, [1, 1]);
        assert_eq!(Trace::CThree.steps()[12].after.number, [2, 2, 1]);
        let d = Trace::D.steps();
        assert_eq!((d.len(), d.iter().filter(|s| s.counted()).count()), (70, 64));
        let taken: Vec<(usize, u32)> = d.iter().filter_map(Step::ticket_written).collect();
        assert_eq!(taken, [(3, 1), (2, 1), (4, 2), (1, 3), (2, 4)]);
        let leap: Vec<u32> = Trace::Leapfrog
            .steps()
            .iter()
            .filter_map(|s| s.ticket_written())
            .map(|t| t.1)
            .collect();
        assert_eq!(leap, [1, 2, 3, 4, 5, 6]);
        assert_eq!(Trace::Leapfrog.steps().len(), 46);
        assert_eq!(Trace::E3.steps().last().expect("steps").after.number, [9, 8]);
    }

    #[test]
    fn the_orders_match_the_oracles_counts() {
        // C-F1: six orders, four lose a point, exactly the four through (1, 1)
        let sb = scoreboard_orders();
        assert_eq!(sb.len(), 6);
        let lost: Vec<bool> = sb.iter().map(|o| o.end[0] == SCORE_START + 1).collect();
        assert_eq!(lost.iter().filter(|&&l| l).count(), 4);
        for (o, l) in sb.iter().zip(&lost) {
            assert_eq!(o.visits((1, 1)), *l);
            assert_eq!(o.both_inside, *l);
            assert!(o.end[0] == SCORE_START + 1 || o.end[0] == SCORE_START + 2);
        }
        // C-F2: fourteen orders, eight with both inside, none deadlocked or
        // ending inside
        let sign = sign_orders();
        assert_eq!(sign.len(), 14);
        assert_eq!(sign.iter().filter(|o| o.both_inside).count(), 8);
        assert_eq!(sign.iter().filter(|o| o.visits(red_point(3))).count(), 8);
        assert!(sign.iter().all(|o| o.done && o.end == [0]));
        // C-F3 and C-F5 (tree_count_out.txt)
        for (flag, cmp, want) in [
            (false, Cmp::Lex, (28, 4, 0)),
            (false, Cmp::Strict, (52, 24, 0)),
            (false, Cmp::Le, (24, 0, 4)),
            (true, Cmp::Lex, (322, 0, 0)),
            (true, Cmp::Strict, (898, 432, 0)),
            (true, Cmp::Le, (250, 0, 72)),
        ] {
            let os = bakery_orders(flag, cmp);
            let len = bakery_program(2, 0, flag, cmp).len();
            let got = (
                os.len(),
                os.iter().filter(|o| o.both_inside).count(),
                os.iter().filter(|o| !o.done).count(),
            );
            assert_eq!(got, want, "flag {flag} {cmp:?}");
            let red = os.iter().filter(|o| o.visits(red_point(len))).count();
            assert_eq!(red, want.1, "flag {flag} {cmp:?}: through the red point");
        }
    }

    #[test]
    fn staircases_are_monotone_and_distinct() {
        for os in [scoreboard_orders(), sign_orders(), bakery_orders(true, Cmp::Lex)] {
            let distinct: HashSet<Vec<usize>> = os.iter().map(|o| o.moves.clone()).collect();
            assert_eq!(distinct.len(), os.len());
            for o in &os {
                let c = o.corners();
                assert!(c.windows(2).all(|w| w[0].0 <= w[1].0 && w[0].1 <= w[1].1));
                assert_eq!(c.first(), Some(&(0, 0)));
            }
        }
    }

    #[test]
    fn the_seated_runs_match_the_oracle() {
        // C-F7.model.customer1_*: Blue reads 3, 0, 0, 2 and writes 4; Pink,
        // Green, Blue in every order
        let blue = blue_takes_a_ticket();
        assert_eq!(reads_by(&blue, 1), [3, 0, 0, 2]);
        let st1 = blue.last().expect("steps").after.clone();
        assert_eq!(st1.number, [4, 3, 0, 0, 2]);
        let orders1 = service_orders(&FIVE, &st1, &[1, 2, 5]);
        assert_eq!(orders1.into_iter().collect::<Vec<_>>(), [vec![5, 2, 1]]);
        // C-F7.model.tie.*: Purple and Teal both read 4, 3, 0, 2 and write 5
        let tie = purple_and_teal_tie();
        assert_eq!(reads_by(&tie, 3), [4, 3, 0, 2]);
        assert_eq!(reads_by(&tie, 4), [4, 3, 0, 2]);
        let st2 = tie.last().expect("steps").after.clone();
        assert_eq!(st2.number, [4, 3, 5, 5, 2]);
        let orders2 = service_orders(&FIVE, &st2, &[1, 2, 3, 4, 5]);
        assert_eq!(orders2.into_iter().collect::<Vec<_>>(), [vec![5, 2, 1, 3, 4]]);
    }

    #[test]
    fn the_tie_without_a_tie_breaker_lets_both_in_or_stalls() {
        // 04_3 from 04_2's slates: "<" lets Purple and Teal both in in some
        // orders and not in others; "<=" stalls them both
        let st2 = purple_and_teal_tie().last().expect("steps").after.clone();
        let strict = Bakery::new(5, 6, Variant::StrictNums, false, false);
        // every run to its end from 04_2's slates, nobody arriving anew:
        // did two ever share the room?
        let mut seen = HashSet::from([(st2.clone(), false)]);
        let mut q = VecDeque::from([(st2, false)]);
        let mut endings = BTreeSet::new();
        while let Some((s, shared)) = q.pop_front() {
            let mut moved = false;
            for (i, _, ns) in strict.successors(&s) {
                if s.local[i].pc == Pc::Ncs || ns == s {
                    continue;
                }
                moved = true;
                let key = (ns.clone(), shared || strict.in_room(&ns).len() >= 2);
                if seen.insert(key.clone()) {
                    q.push_back(key);
                }
            }
            if !moved {
                endings.insert(shared);
            }
        }
        assert_eq!(endings.into_iter().collect::<Vec<_>>(), [false, true]);
        let st2 = purple_and_teal_tie().last().expect("steps").after.clone();
        let le = Bakery::new(5, 6, Variant::LeNums, false, false);
        let orders = service_orders(&le, &st2, &[1, 2, 3, 4, 5]);
        assert_eq!(orders.into_iter().collect::<Vec<_>>(), [vec![5, 2, 1]]);
    }
}

// ------------------------------------------- liveness and the largest ticket

/// Every state reachable from the start, breadth first.
pub fn reachable(model: &Bakery) -> Vec<State> {
    let init = model.initial();
    let mut seen = HashSet::from([init.clone()]);
    let mut out = vec![init.clone()];
    let mut q = VecDeque::from([init]);
    while let Some(s) = q.pop_front() {
        for (_, _, ns) in model.successors(&s) {
            if seen.insert(ns.clone()) {
                out.push(ns.clone());
                q.push_back(ns);
            }
        }
    }
    out
}

/// A finished nonzero ticket write, `number[i]:=t` or `end number[i]:=t`
/// with t > 0 (the oracle's `TICKET_DONE`): not a `begin`, not an exit.
fn ticket_done(label: &str) -> bool {
    let rest = label.strip_prefix("end ").unwrap_or(label);
    rest.strip_prefix("number[")
        .and_then(|r| r.split_once("]:="))
        .is_some_and(|(i, t)| {
            i.bytes().all(|b| b.is_ascii_digit())
                && t.bytes().all(|b| b.is_ascii_digit())
                && t.parse::<u32>().is_ok_and(|t| t > 0)
        })
}

/// C-X5 (11_2, "each other customer can get ahead of you at most once"):
/// a breadth-first search over (state, ghost), where for each customer whose
/// ticket write has finished and who has not yet gone in, the ghost counts
/// every other customer's room entries since then (capped at 2). Returns the
/// ghost states and the largest count reached (the oracle's `bypass_bfs`).
pub fn bypass_bfs(model: &Bakery) -> (usize, u8) {
    let n = model.n;
    let start = (model.initial(), vec![false; n], vec![vec![0u8; n]; n]);
    let mut seen = HashSet::from([start.clone()]);
    let mut q = VecDeque::from([start]);
    let mut worst = 0;
    while let Some((s, active, counts)) = q.pop_front() {
        for (p, label, ns) in model.successors(&s) {
            if ns == s {
                continue;
            }
            let (mut a, mut c) = (active.clone(), counts.clone());
            if ticket_done(&label) {
                a[p] = true;
                c[p] = vec![0; n];
            }
            if label.contains("ENTER CS") {
                for i in (0..n).filter(|&i| i != p && a[i]) {
                    c[i][p] = (c[i][p] + 1).min(2);
                    worst = worst.max(c[i][p]);
                }
                a[p] = false;
                c[p] = vec![0; n];
            }
            let key = (ns, a, c);
            if !seen.contains(&key) {
                seen.insert(key.clone());
                q.push_back(key);
            }
        }
    }
    (seen.len(), worst)
}

/// What [`frozen_bfs`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frozen {
    pub ghost_states: usize,
    /// The most room entries by any one other customer after the freeze.
    pub worst: u8,
    /// Room entries by a customer whose pair is bigger than the frozen one's.
    pub behind: usize,
}

/// C-X8, C-X9 (11_7, the costs card): customer `c` (0-based) never moves
/// again, from every state in `starts`; the others move freely. A
/// breadth-first search over (state, entries since the freeze, capped at 2)
/// (the oracle's `frozen_bfs`).
pub fn frozen_bfs(model: &Bakery, starts: &[State], c: usize) -> Frozen {
    let n = model.n;
    let mut seen: HashSet<(State, Vec<u8>)> = HashSet::new();
    let mut q = VecDeque::new();
    for s in starts {
        let key = (s.clone(), vec![0u8; n]);
        if seen.insert(key.clone()) {
            q.push_back(key);
        }
    }
    let (mut worst, mut behind) = (0, 0);
    while let Some((s, g)) = q.pop_front() {
        for (p, label, ns) in model.successors(&s) {
            if p == c || ns == s {
                continue;
            }
            let mut g2 = g.clone();
            if label.contains("ENTER CS") {
                behind += usize::from((ns.number[p], p) > (s.number[c], c));
                g2[p] = (g2[p] + 1).min(2);
                worst = worst.max(g2[p]);
            }
            let key = (ns, g2);
            if !seen.contains(&key) {
                seen.insert(key.clone());
                q.push_back(key);
            }
        }
    }
    Frozen {
        ghost_states: seen.len(),
        worst,
        behind,
    }
}

/// The states where customer 1 is choosing (flag up, in her doorway): C-X8's
/// starts. And those where she is past choosing, waiting in line: C-X9's.
pub fn customer1_choosing_and_in_line(states: &[State]) -> (Vec<State>, Vec<State>) {
    let choosing = states
        .iter()
        .filter(|s| s.choosing[0] == 1 && matches!(s.local[0].pc, Pc::Rd | Pc::Rd2 | Pc::Wn | Pc::Wc0))
        .cloned()
        .collect();
    let in_line = states
        .iter()
        .filter(|s| matches!(s.local[0].pc, Pc::L2 | Pc::L3))
        .cloned()
        .collect();
    (choosing, in_line)
}

/// C-X7 (11_3, "they reset only when the bakery empties"): over every
/// reachable state's moves, how often the largest ticket held (a nonzero
/// slate not in the middle of its exit write) drops, and how often it drops
/// to a value other than zero.
pub fn largest_ticket_drops(model: &Bakery) -> (usize, usize) {
    let top = |s: &State| {
        (0..model.n)
            .filter(|&k| s.number[k] != 0 && !(s.local[k].wp && matches!(s.local[k].pc, Pc::Wn | Pc::Cs)))
            .map(|k| s.number[k])
            .max()
            .unwrap_or(0)
    };
    let (mut drops, mut to_nonzero) = (0, 0);
    for s in reachable(model) {
        let h = top(&s);
        for (_, _, ns) in model.successors(&s) {
            let h2 = top(&ns);
            if h2 < h {
                drops += 1;
                to_nonzero += usize::from(h2 != 0);
            }
        }
    }
    (drops, to_nonzero)
}

#[cfg(test)]
mod liveness_tests {
    use super::*;

    /// The checker's default: the max reads the other slates in any order.
    const ATOMIC_3_4: Bakery = Bakery::new(3, 4, Variant::Correct, false, true);

    #[test]
    fn the_bypass_and_frozen_searches_match_the_oracle() {
        // C-X5.bypass_from_ticket_write.n3_k4_atomic.{ghost_states, worst}
        assert_eq!(bypass_bfs(&ATOMIC_3_4), (45_868, 1));
        let states = reachable(&ATOMIC_3_4);
        let (choosing, in_line) = customer1_choosing_and_in_line(&states);
        // C-X8.frozen_while_choosing.n3_k4_atomic.*
        assert_eq!(choosing.len(), 9_763);
        let f = frozen_bfs(&ATOMIC_3_4, &choosing, 0);
        assert_eq!(f.worst, 1);
        // C-X9.paused_in_line.n3_k4_atomic.*
        assert_eq!(in_line.len(), 7_098);
        let p = frozen_bfs(&ATOMIC_3_4, &in_line, 0);
        assert_eq!((p.worst, p.behind), (1, 0));
    }

    #[test]
    fn the_largest_ticket_drops_only_to_zero() {
        // C-X7.largest_ticket_drops{,_to_nonzero}.n3_k5_atomic
        assert_eq!(
            largest_ticket_drops(&Bakery::new(3, 5, Variant::Correct, false, true)),
            (545, 0)
        );
    }

    #[test]
    fn a_finished_ticket_write_is_told_from_the_rest() {
        assert!(ticket_done("number[2]:=3"));
        assert!(ticket_done("end number[1]:=12"));
        assert!(!ticket_done("begin number[1]:=3"));
        assert!(!ticket_done("number[1]:=0 (exit)"));
        assert!(!ticket_done("end number[1]:=0 (exit)"));
        assert!(!ticket_done("read number[2]=3"));
    }
}

// ------------------------------------------------------ store buffers (12)

pub mod tso {
    //! The bakery algorithm under an x86-TSO memory model, ported from
    //! `bakery_tso.py` (Owens, Sarkar and Sewell's model): each customer's core
    //! has a FIFO outbox (store buffer); a store joins its own outbox; a load
    //! reads the newest store to that slate in its own outbox, else memory; at
    //! any time an outbox may deliver its oldest store to memory; a fence waits
    //! until its own outbox is empty. Fences F1 (after raising the flag) and F2
    //! (after lowering it) are optional. Tickets above `k` are pruned and an
    //! outbox holds at most `b` stores (a store waits while it is full), so a
    //! violation found is real and "none" means none within the bounds.
    //!
    //! Chapter 12 replays the oracle's voiced order on it (`SCHED_TSO_VOICED`),
    //! and `facts` pins the model's own state counts against the oracle's.

    use std::collections::{HashMap, VecDeque};

    /// Where a customer is in her program.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Pc {
        /// Not in line.
        Outside,
        /// Fence F1, after raising the flag.
        Fence1,
        /// Reading the other slates for the biggest ticket.
        Reading,
        /// Writing her ticket.
        WriteTicket,
        /// Lowering her flag.
        LowerFlag,
        /// Fence F2, after lowering the flag.
        Fence2,
        /// Waiting for customer `j`'s flag to be down.
        WaitFlag,
        /// Waiting while customer `j`'s pair comes before hers.
        WaitTicket,
        /// In the room.
        Room,
    }

    /// A flag or a slate.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum Var {
        Flag,
        Slate,
    }

    /// A store waiting in an outbox: `var` of customer `owner` := `value`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Store {
        pub var: Var,
        pub owner: usize,
        pub value: u32,
    }

    impl Store {
        /// As the record writes it: `c1=1`, `n2=0` (customers from 1).
        pub fn short(&self) -> String {
            let k = if self.var == Var::Flag { 'c' } else { 'n' };
            format!("{k}{}={}", self.owner + 1, self.value)
        }
    }

    /// One customer's program state. In `Reading`, `arg` is the mask of slates
    /// still to read and `max` the biggest so far; in the waits, `arg` is `j`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Local {
        pub pc: Pc,
        pub arg: u32,
        pub max: u32,
    }

    /// The whole machine: programs, memory (flags and slates) and outboxes.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct State {
        pub local: Vec<Local>,
        pub flags: Vec<u32>,
        pub slates: Vec<u32>,
        pub outbox: Vec<Vec<Store>>,
    }

    /// What one transition does (customers from 0; labels from 1).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Move {
        /// An outbox delivers its oldest store to memory.
        Deliver(Store),
        /// A reordering outbox delivers a later store first (only when no
        /// earlier store to the same slate waits ahead of it).
        DeliverEarly(Store),
        /// A store joins the customer's outbox.
        Buffer(Store),
        /// A fence passes (the outbox is empty).
        Fence,
        /// A read of slate `j` during the biggest-ticket loop.
        ReadSlate { j: usize, value: u32 },
        /// A glance at customer `j`'s flag: `up` means wait.
        CheckFlag { j: usize, up: bool },
        /// A glance at customer `j`'s slate: wait, pass, or pass into the room.
        CheckSlate {
            j: usize,
            value: u32,
            wait: bool,
            enter: bool,
        },
    }

    impl Move {
        /// The oracle's label (`bakery_tso.py`), so its schedules replay as written.
        pub fn label(&self) -> String {
            match *self {
                Move::Deliver(s) => format!(
                    "flush {}[{}]={} to memory",
                    if s.var == Var::Flag { "choosing" } else { "number" },
                    s.owner + 1,
                    s.value
                ),
                Move::DeliverEarly(s) => format!(
                    "flush {}[{}]={} to memory (out of order)",
                    if s.var == Var::Flag { "choosing" } else { "number" },
                    s.owner + 1,
                    s.value
                ),
                Move::Buffer(Store {
                    var: Var::Flag, value, ..
                }) => format!("store choosing:={value} (buffered)"),
                Move::Buffer(Store {
                    var: Var::Slate,
                    value: 0,
                    ..
                }) => "store number:=0 (exit, buffered)".into(),
                Move::Buffer(Store {
                    var: Var::Slate, value, ..
                }) => format!("store number:={value} (buffered)"),
                Move::Fence => "mfence".into(),
                Move::ReadSlate { j, value } => format!("load number[{}]={value}", j + 1),
                Move::CheckFlag { j, up } => {
                    format!(
                        "L2 load choosing[{}]={}, {}",
                        j + 1,
                        u8::from(up),
                        if up { "wait" } else { "pass" }
                    )
                }
                Move::CheckSlate { j, value, wait, enter } => format!(
                    "L3 load number[{}]={value}, {}",
                    j + 1,
                    match (wait, enter) {
                        (true, _) => "wait",
                        (false, false) => "pass",
                        (false, true) => "pass -> ENTER CS",
                    }
                ),
            }
        }
    }

    /// The model: `n` customers, tickets up to `k`, outboxes of `b`, fences.
    #[derive(Clone, Copy, Debug)]
    pub struct Tso {
        pub n: usize,
        pub k: u32,
        pub b: usize,
        pub fence1: bool,
        pub fence2: bool,
        /// Outboxes may deliver stores to different slates out of order (the
        /// oracle's `OutOfOrderOutbox`, R2 dev-02): a model of a weaker
        /// machine, not a model of Arm, which reorders in more ways.
        pub reorder: bool,
    }

    impl Tso {
        /// `bakery_tso.TSOBakery(n, k, fences)` with its default outbox of 4.
        pub fn new(n: usize, k: u32, fence1: bool, fence2: bool) -> Self {
            Self {
                n,
                k,
                b: 4,
                fence1,
                fence2,
                reorder: false,
            }
        }

        pub fn initial(&self) -> State {
            State {
                local: vec![
                    Local {
                        pc: Pc::Outside,
                        arg: 0,
                        max: 0
                    };
                    self.n
                ],
                flags: vec![0; self.n],
                slates: vec![0; self.n],
                outbox: vec![Vec::new(); self.n],
            }
        }

        /// Customer `i` loads `var` of `j`: her own newest buffered store to
        /// it, else memory.
        pub fn load(&self, st: &State, i: usize, var: Var, j: usize) -> u32 {
            st.outbox[i]
                .iter()
                .rev()
                .find(|s| s.var == var && s.owner == j)
                .map_or_else(
                    || if var == Var::Flag { st.flags[j] } else { st.slates[j] },
                    |s| s.value,
                )
        }

        /// Every enabled move, in the oracle's order: per customer, the
        /// delivery first, then the program step. A blocked wait is a move
        /// that leaves the state as it was.
        pub fn successors(&self, st: &State) -> Vec<(usize, Move, State)> {
            let mut out = Vec::new();
            for i in 0..self.n {
                let Local { pc, arg, max } = st.local[i];
                let others: Vec<usize> = (0..self.n).filter(|&j| j != i).collect();
                if let Some(&oldest) = st.outbox[i].first() {
                    let mut ns = st.clone();
                    match oldest.var {
                        Var::Flag => ns.flags[oldest.owner] = oldest.value,
                        Var::Slate => ns.slates[oldest.owner] = oldest.value,
                    }
                    ns.outbox[i].remove(0);
                    out.push((i, Move::Deliver(oldest), ns));
                }
                let full = st.outbox[i].len() >= self.b;
                if full && matches!(pc, Pc::Outside | Pc::WriteTicket | Pc::LowerFlag | Pc::Room) {
                    continue;
                }
                let goto = |pc: Pc, arg: u32, max: u32, store: Option<Store>| {
                    let mut ns = st.clone();
                    ns.local[i] = Local { pc, arg, max };
                    if let Some(s) = store {
                        ns.outbox[i].push(s);
                    }
                    ns
                };
                let own = |var, value| Store { var, owner: i, value };
                let first_wait = (Pc::WaitFlag, others[0] as u32, 0);
                match pc {
                    Pc::Outside => {
                        let mask = others.iter().map(|&j| 1u32 << j).sum();
                        let next = if self.fence1 { Pc::Fence1 } else { Pc::Reading };
                        let s = own(Var::Flag, 1);
                        out.push((i, Move::Buffer(s), goto(next, mask, 0, Some(s))));
                    }
                    Pc::Fence1 | Pc::Fence2 => {
                        if st.outbox[i].is_empty() {
                            let (p, a, m) = if pc == Pc::Fence1 {
                                (Pc::Reading, arg, 0)
                            } else {
                                first_wait
                            };
                            out.push((i, Move::Fence, goto(p, a, m, None)));
                        }
                    }
                    Pc::Reading => {
                        let j = arg.trailing_zeros() as usize;
                        let value = self.load(st, i, Var::Slate, j);
                        let rest = arg & !(1 << j);
                        let m = max.max(value);
                        let ns = if rest != 0 {
                            goto(Pc::Reading, rest, m, None)
                        } else {
                            goto(Pc::WriteTicket, 0, m, None)
                        };
                        out.push((i, Move::ReadSlate { j, value }, ns));
                    }
                    Pc::WriteTicket => {
                        let t = max + 1;
                        if t <= self.k {
                            let s = own(Var::Slate, t);
                            out.push((i, Move::Buffer(s), goto(Pc::LowerFlag, 0, 0, Some(s))));
                        }
                    }
                    Pc::LowerFlag => {
                        let (p, a, m) = if self.fence2 { (Pc::Fence2, 0, 0) } else { first_wait };
                        let s = own(Var::Flag, 0);
                        out.push((i, Move::Buffer(s), goto(p, a, m, Some(s))));
                    }
                    Pc::WaitFlag => {
                        let j = arg as usize;
                        let up = self.load(st, i, Var::Flag, j) != 0;
                        let ns = if up {
                            st.clone()
                        } else {
                            goto(Pc::WaitTicket, arg, 0, None)
                        };
                        out.push((i, Move::CheckFlag { j, up }, ns));
                    }
                    Pc::WaitTicket => {
                        let j = arg as usize;
                        let nj = self.load(st, i, Var::Slate, j);
                        let ni = self.load(st, i, Var::Slate, i);
                        let wait = nj != 0 && (nj, j) < (ni, i);
                        let k = others.iter().position(|&o| o == j).expect("j is another customer");
                        let enter = !wait && k + 1 == others.len();
                        let ns = if wait {
                            st.clone()
                        } else if enter {
                            goto(Pc::Room, 0, 0, None)
                        } else {
                            goto(Pc::WaitFlag, others[k + 1] as u32, 0, None)
                        };
                        out.push((
                            i,
                            Move::CheckSlate {
                                j,
                                value: nj,
                                wait,
                                enter,
                            },
                            ns,
                        ));
                    }
                    Pc::Room => {
                        let s = own(Var::Slate, 0);
                        out.push((i, Move::Buffer(s), goto(Pc::Outside, 0, 0, Some(s))));
                    }
                }
            }
            if self.reorder {
                for i in 0..self.n {
                    for m in 1..st.outbox[i].len() {
                        let s = st.outbox[i][m];
                        if st.outbox[i][..m].iter().any(|e| e.var == s.var && e.owner == s.owner) {
                            continue;
                        }
                        let mut ns = st.clone();
                        match s.var {
                            Var::Flag => ns.flags[s.owner] = s.value,
                            Var::Slate => ns.slates[s.owner] = s.value,
                        }
                        ns.outbox[i].remove(m);
                        out.push((i, Move::DeliverEarly(s), ns));
                    }
                }
            }
            out
        }

        /// The customers in the room.
        pub fn in_room(&self, st: &State) -> usize {
            st.local.iter().filter(|l| l.pc == Pc::Room).count()
        }

        /// Replay `(customer from 1, label substring)` steps: exactly one of
        /// that customer's enabled moves must match each. Returns every state,
        /// the initial one first, and the moves between them.
        pub fn replay(&self, schedule: &[(usize, &str)]) -> Result<(Vec<State>, Vec<Move>), String> {
            let mut states = vec![self.initial()];
            let mut moves = Vec::new();
            for &(p, sub) in schedule {
                let st = states.last().expect("starts with the initial state");
                let found: Vec<(Move, State)> = self
                    .successors(st)
                    .into_iter()
                    .filter(|(i, m, _)| *i + 1 == p && m.label().contains(sub))
                    .map(|(_, m, s)| (m, s))
                    .collect();
                let [(m, s)] = <[_; 1]>::try_from(found)
                    .map_err(|f: Vec<_>| format!("step P{p} {sub:?} matched {} moves", f.len()))?;
                moves.push(m);
                states.push(s);
            }
            Ok((states, moves))
        }

        /// Breadth-first search of every reachable state: (states, the length
        /// of the shortest run that puts two in the room, if any).
        pub fn explore(&self) -> (usize, Option<usize>) {
            let init = self.initial();
            let mut depth: HashMap<State, usize> = HashMap::from([(init.clone(), 0)]);
            let mut queue = VecDeque::from([init]);
            let mut shortest = None;
            while let Some(st) = queue.pop_front() {
                let d = depth[&st];
                for (_, _, ns) in self.successors(&st) {
                    if !depth.contains_key(&ns) {
                        if shortest.is_none() && self.in_room(&ns) >= 2 {
                            shortest = Some(d + 1);
                        }
                        depth.insert(ns.clone(), d + 1);
                        queue.push_back(ns);
                    }
                }
            }
            (depth.len(), shortest)
        }
    }

    /// 12_3–12_4 as voiced (the oracle's `SCHED_TSO_VOICED`, R11): both raise
    /// their flags, read, and write their tickets into their outboxes; then each
    /// checks the other's flag and slate. Nothing is ever delivered.
    pub const VOICED: [(usize, &str); 12] = [
        (1, "store choosing:=1"),
        (2, "store choosing:=1"),
        (1, "load number[2]=0"),
        (2, "load number[1]=0"),
        (1, "store number:=1"),
        (2, "store number:=1"),
        (1, "store choosing:=0"),
        (2, "store choosing:=0"),
        (1, "L2 load choosing[2]=0, pass"),
        (1, "L3 load number[2]=0, pass -> ENTER CS"),
        (2, "L2 load choosing[1]=0, pass"),
        (2, "L3 load number[1]=0, pass -> ENTER CS"),
    ];

    /// 12_5's fix on the same race: both fences, so each outbox is delivered
    /// before its customer reads; both choose at once, take 1, and the tie goes
    /// to Blue while Green waits.
    pub const FENCED: [(usize, &str); 22] = [
        (1, "store choosing:=1"),
        (2, "store choosing:=1"),
        (1, "flush choosing[1]=1"),
        (1, "mfence"),
        (2, "flush choosing[2]=1"),
        (2, "mfence"),
        (1, "load number[2]=0"),
        (2, "load number[1]=0"),
        (1, "store number:=1"),
        (2, "store number:=1"),
        (1, "store choosing:=0"),
        (2, "store choosing:=0"),
        (1, "flush number[1]=1"),
        (1, "flush choosing[1]=0"),
        (1, "mfence"),
        (2, "flush number[2]=1"),
        (2, "flush choosing[2]=0"),
        (2, "mfence"),
        (1, "L2 load choosing[2]=0, pass"),
        (1, "L3 load number[2]=1, pass -> ENTER CS"),
        (2, "L2 load choosing[1]=0, pass"),
        (2, "L3 load number[1]=1, wait"),
    ];

    /// 12_6 on the reordering model, both fences in place: Blue's lowered flag
    /// reaches memory before her ticket, Green sees no flag and no ticket and
    /// walks in (05_2's crack), then Blue's ticket lands, wins the tie, and she
    /// walks in too.
    pub const REORDERED: [(usize, &str); 22] = [
        (1, "store choosing:=1"),
        (1, "flush choosing[1]=1"),
        (1, "mfence"),
        (1, "load number[2]=0"),
        (1, "store number:=1"),
        (1, "store choosing:=0"),
        (1, "flush choosing[1]=0 to memory (out of order)"),
        (2, "store choosing:=1"),
        (2, "flush choosing[2]=1"),
        (2, "mfence"),
        (2, "load number[1]=0"),
        (2, "store number:=1"),
        (2, "store choosing:=0"),
        (2, "flush number[2]=1"),
        (2, "flush choosing[2]=0"),
        (2, "mfence"),
        (2, "L2 load choosing[1]=0, pass"),
        (2, "L3 load number[1]=0, pass -> ENTER CS"),
        (1, "flush number[1]=1"),
        (1, "mfence"),
        (1, "L2 load choosing[2]=0, pass"),
        (1, "L3 load number[2]=1, pass -> ENTER CS"),
    ];

    #[cfg(test)]
    mod tests {
        use super::*;

        fn list(v: &[u32]) -> String {
            v.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
        }

        #[test]
        fn the_voiced_order_puts_both_in_the_room_with_memory_still_zero() {
            let m = Tso::new(2, 4, false, false);
            let (states, moves) = m.replay(&VOICED).unwrap();
            let end = states.last().unwrap();
            assert_eq!(moves.len(), 12);
            assert_eq!(m.in_room(end), 2);
            assert_eq!(list(&end.flags), "0,0");
            assert_eq!(list(&end.slates), "0,0");
            let boxes: Vec<String> = end
                .outbox
                .iter()
                .map(|b| b.iter().map(Store::short).collect::<Vec<_>>().join(","))
                .collect();
            assert_eq!(boxes.join(";"), "c1=1,n1=1,c1=0;c2=1,n2=1,c2=0");
            assert!(moves.iter().all(|m| !matches!(m, Move::Deliver(_))));
        }

        #[test]
        fn the_state_counts_match_the_python_model() {
            // outputs/bakery_tso_out.txt and tso3.txt, via the oracle's C-F9 keys
            assert_eq!(Tso::new(2, 4, false, false).explore(), (48_646, Some(12)));
            assert_eq!(Tso::new(2, 4, true, false).explore(), (2_298, Some(18)));
            assert_eq!(Tso::new(2, 4, false, true).explore(), (7_484, Some(20)));
            assert_eq!(Tso::new(2, 4, true, true).explore(), (1_135, None));
            assert_eq!(Tso::new(2, 6, true, true).explore(), (1_811, None));
            let mut n3 = Tso::new(3, 3, true, true);
            n3.b = 3;
            assert_eq!(n3.explore(), (37_580, None));
        }

        #[test]
        fn with_both_fences_the_same_race_lets_one_in() {
            let m = Tso::new(2, 4, true, true);
            let (states, _) = m.replay(&FENCED).unwrap();
            let end = states.last().unwrap();
            assert_eq!(m.in_room(end), 1);
            assert_eq!(end.local[0].pc, Pc::Room);
            assert_eq!(end.local[1].pc, Pc::WaitTicket);
            assert!(states.iter().all(|s| m.in_room(s) <= 1));
        }

        #[test]
        fn a_reordering_outbox_breaks_the_two_fences() {
            // C-F9.ooo_outbox.f1_f2.*: 1,673 states, a 22-step shortest failure
            let mut ooo = Tso::new(2, 4, true, true);
            ooo.reorder = true;
            assert_eq!(ooo.explore(), (1_673, Some(22)));
            let (states, moves) = ooo.replay(&REORDERED).unwrap();
            assert_eq!(moves.len(), 22);
            assert_eq!(ooo.in_room(states.last().unwrap()), 2);
            // Green walks in while Blue's ticket is still in her outbox
            let green_in = states.iter().position(|s| s.local[1].pc == Pc::Room).unwrap();
            let s = &states[green_in];
            assert_eq!((s.flags[0], s.slates[0]), (0, 0));
            assert!(s.outbox[0].iter().any(|e| e.var == Var::Slate && e.value == 1));
            // the in-order model refuses the overtaking delivery
            assert!(Tso::new(2, 4, true, true).replay(&REORDERED).is_err());
        }

        #[test]
        fn a_replay_refuses_a_step_that_is_not_enabled() {
            let m = Tso::new(2, 4, false, false);
            assert!(m.replay(&[(1, "load number[2]=0")]).is_err());
            // a fence step does not exist without fences
            assert!(m.replay(&[(1, "store choosing:=1"), (1, "mfence")]).is_err());
        }
    }
}
