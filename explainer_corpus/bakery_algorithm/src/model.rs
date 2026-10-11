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
