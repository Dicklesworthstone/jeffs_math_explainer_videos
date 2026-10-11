//! Every fixed on-screen string of Video C, for the glyph probe (`bakery
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . + − = ( )";

pub const TITLE: &str = "The Algorithm Lamport Is Proudest Of";

// 01_one_plus_one_is_one
/// Lamport's sentence (Turing lecture, CACM 58(6), 2015, p. 4), around its
/// three numbers: "…from 9 to 10 obtains the value 2496."
pub const TORN_A: &str = "The algorithm is correct even if reading a number while it is changing from";
pub const TORN_B: &str = "to";
pub const TORN_C: &str = "obtains the value";
pub const TURING_SOURCE: &str = "Leslie Lamport, Turing Award lecture, CACM 58(6), 2015";
/// "In 1973, this was considered impossible." (same page), around its year.
pub const IMPOSSIBLE_A: &str = "In";
pub const IMPOSSIBLE_B: &str = ", this was considered impossible.";
pub const X: &str = "x";
pub const CODE: &str = "x = x + 1";
pub const MAKE_BIGGER: &str = "not an equation: make x one bigger";
pub const READ_X: &str = "read x";
pub const WRITE_X: &str = "write x + 1";
pub const TIME_RUNS: &str = "time runs left to right";
pub const ARROWS: &str = "a read is an arrow in, a write is an arrow out";
pub const PLUS_ONE: &str = "+1";
pub const READ: &str = "read";
pub const WRITE: &str = "write";
pub const BLUES_STEPS: &str = "Blue's steps";
pub const GREENS_STEPS: &str = "Green's steps";
pub const ORDERS: &str = "orders";
pub const LOSE_A_POINT: &str = "lose a point";
pub const WHY_SIX: &str = "why six:";
pub const PICK_A: &str = "pick which";
pub const PICK_B: &str = "of the";
pub const PICK_C: &str = "steps are Blue's";
pub const MIDDLE_1: &str = "the middle: both have read,";
pub const MIDDLE_2: &str = "neither has written";
pub const PRECISELY: &str = "precisely:";
pub const A_COUNT: &str = "bad: a count, not a probability";
pub const STRIP_NOTE: &str = "each tick is one step (an illustration, not a measurement)";

// 02_the_room
pub const THE_ROOM: &str = "the room";
pub const NAMES: &str = "names:";
pub const CRITICAL_SECTION: &str = "critical section; the rule is called mutual exclusion";
pub const CHECK: &str = "check";
pub const FLIP: &str = "flip";
pub const LEAVE: &str = "leave";
pub const BOTH_INSIDE: &str = "both inside";
pub const PUT_BOTH_IN: &str = "put both in the room";
pub const COUNTING: &str = "counting:";
pub const COUNTING_RULE: &str = "each read, write and successful wait is one step";
pub const DIJKSTRA: &str = "Edsger W. Dijkstra";
pub const KNUTH: &str = "Donald E. Knuth";
pub const RULE_1: &str = "never two in the room";
pub const RULE_2: &str = "stopping while idle holds up nobody";
pub const RULE_3: &str = "fast or slow, it works";
pub const RULE_4: &str = "everyone gets in eventually";
pub const MAGIC: &str = "magic sign: check and flip in one motion";
pub const ORDINARY: &str = "ordinary signs, used cleverly";
pub const THE_WORD: &str = "the word:";
pub const ATOMIC: &str = "a step nobody can catch halfway is called atomic";
pub const QUESTION: &str = "?";

// 03_take_a_number
pub const LAMPORT: &str = "Leslie Lamport";
pub const PROCESS: &str = "programmers say process";
pub const TICKET_MACHINE: &str = "ticket machine";
pub const WAITS: &str = "waits for every smaller ticket";
pub const FOR_PROGRAMMERS: &str = "for programmers:";
pub const TICKET_LOCK: &str =
    "a ticket lock is the machine Lamport avoided: one atomic fetch-and-add (Linux's x86 FIFO ticket spinlocks)";

// 04_same_number
pub const RULE_SMALLER: &str = "rule: wait for anyone with a smaller ticket";
pub const RULE_SMALLER_OR_EQUAL: &str = "rule: wait for anyone with a smaller or equal ticket";
pub const AFTER_YOU: &str = "after you";
pub const NO_AFTER_YOU: &str = "no, after you";
pub const PAIR: &str = "pair";
pub const CAT: &str = "cat";
pub const COW: &str = "cow";
pub const COMES_BEFORE: &str = "comes before";
pub const TICKET_ACROSS: &str = "ticket";
pub const CUSTOMER_NUMBER_UP: &str = "customer number";
pub const SERVED: &str = "served";
pub const TOTALLY_ORDERED: &str = "pairs are totally ordered, and each has finitely many ahead of it";

// 06_three_customers
pub const PURPLES_STEPS: &str = "Purple's steps";
pub const TWO_INSIDE_KEY: &str = "red: two in the room at once";
/// "{count} put two in the room", the count from `facts`.
pub const PUT_TWO_IN: &str = "put two in the room";
pub const TRAILS: &str = "trails:";
/// "drawn uniformly at random from all {orders} orders, not as a real
/// machine runs them" (R15), around the count.
pub const TRAILS_A: &str = "drawn uniformly at random from all";
pub const TRAILS_B: &str = "orders, not as a real machine runs them";

// 07_one_run
/// The step clock's label (counted the 02_4 way), beside its number.
pub const STEPS: &str = "steps";
/// Green's slate when Purple meets her flag: no ticket written yet.
pub const UNWRITTEN: &str = "not written yet";
/// Pink's lane in 07: trace d has four customers.
pub const PINK_HOME: &str = "Pink stays home this run";

// 08_why_never_two
pub const TIME: &str = "time";
pub const BLUE_CHOOSING: &str = "Blue choosing";
pub const GREEN_CHOOSING: &str = "Green choosing";
pub const BLUE_IN_ROOM: &str = "Blue in the room";
pub const GREEN_IN_ROOM: &str = "Green in the room";
/// 08_8: the supposition, before the algebra.
pub const SUPPOSE_BOTH: &str = "suppose both are in the room";
pub const LAST_LOOK: &str = "Blue's last look at Green's flag";
/// "Green: {place}", the place one of the three below.
pub const GREEN_AT: &str = "Green:";
pub const NOT_YET: &str = "not yet choosing";
pub const CHOOSING: &str = "choosing";
pub const DONE_CHOOSING: &str = "done choosing";
pub const COUNTS_ON: &str = "the proof counts on:";
pub const COUNTS_ON_TEXT: &str = "each customer's steps happen in order; a finished write is seen by every later read; \
     the biggest ticket is found by reading each slate once";
pub const SEES_TICKET: &str = "Green sees Blue's ticket";
pub const FACT_ONE: &str = "fact one";
pub const FACT_ONE_TEXT: &str = "Blue finished choosing before Green started:\nGreen's ticket is bigger than Blue's";
pub const FACT_TWO: &str = "fact two";
pub const FACT_TWO_TEXT: &str = "Blue is in the room: everyone else holding\na ticket has a bigger pair than Blue's";
pub const WHY: &str = "why?";
pub const GREEN_BIGGER: &str = "Green's ticket > Blue's";
pub const FLAG_UP: &str = "flag up: Blue would wait";
pub const REAL_TICKET: &str = "Blue reads Green's real ticket";
pub const PAIR_SMALLER: &str = "Blue's pair < Green's pair";
pub const FOR_BLUE: &str = "fact two, for Blue";
pub const FOR_GREEN: &str = "fact two, for Green";
pub const IMPOSSIBLE_PAIR: &str = "a pair smaller than itself: impossible";
/// 08_9's card, around its numbers (from `facts` and the search's key):
/// "computer search: {n} customers, tickets up to {k} (runs that would pass
/// {k} are cut)," / "indivisible reads and writes: {states} reachable
/// states, no violation".
pub const SEARCH_A: &str = "computer search:";
pub const SEARCH_B: &str = "customers, tickets up to";
pub const SEARCH_C: &str = "(runs that would pass";
pub const SEARCH_D: &str = "are cut),";
pub const SEARCH_E: &str = "indivisible reads and writes:";
pub const SEARCH_F: &str = "reachable states,";
pub const NO_VIOLATION: &str = "no violation";
pub const A_VIOLATION: &str = "a violation";
pub const THE_PROOF: &str = "the proof:";
pub const EVERY_CASE: &str = "the proof: any number of customers, any tickets";
/// 08_8's pairs (TeX).
pub const PAIRS_FOR_BLUE: &str = r"(n_B,\ \text{Blue}) < (n_G,\ \text{Green})";
pub const PAIRS_FOR_GREEN: &str = r"(n_G,\ \text{Green}) < (n_B,\ \text{Blue})";
pub const PAIRS_CHAIN: &str = r"(n_B,\ \text{Blue}) < (n_G,\ \text{Green}) < (n_B,\ \text{Blue})";

// 05_the_unwritten_ticket
pub const FLAG_UP_WORD: &str = "up";
pub const FLAG_DOWN_WORD: &str = "down";
pub const STEPS_WORD: &str = "steps";
pub const FLAG_RULE_1: &str = "a flag says: I'm choosing, don't judge me yet";
pub const FLAG_RULE_2: &str = "raise it before reading, lower it after writing";
pub const FLAG_RULE_3: &str = "before comparing with someone, wait until their flag is down";
pub const WAITS_WORD: &str = "waits";
pub const WITHOUT_FLAGS: &str = "without flags";
pub const WITH_FLAGS: &str = "with flags";
pub const ONE_VISIT_EACH: &str = "two customers, one visit each; each read, write and successful wait is one step";
pub const CHOOSING_TIME: &str = "choosing time";
pub const DOORWAY: &str = "Lamport calls this the doorway";
pub const A_LOCK: &str = "a lock: others wait";
pub const THESIS: &str = "choosing at once is fine: the flag shows who's choosing, the pair settles ties";

// 10_the_old_zero
/// "step {s}" under the time axis, the step number from trace e3.
pub const STEP: &str = "step";
/// 10_2's card (08_2's fact one with Blue and Green swapped).
pub const FACT_ONE_SWAPPED: &str = "fact one, the other way round";
pub const FACT_ONE_SWAPPED_TEXT: &str =
    "Green finished choosing before Blue started:\nBlue's ticket is bigger than Green's";
pub const CHIPS_DONT_TEAR: &str = "today's chips don't tear an aligned word; the proof just doesn't need that";
/// "Leslie Lamport, publications page".
pub const QUOTE_DISCOVERED: &str =
    "I discovered that the bakery algorithm had this property after writing a proof of its correctness";
pub const PUBLICATIONS_SOURCE: &str = "Leslie Lamport, publications page";
/// The bakery paper, p. 454; its source "Lamport, {journal}, {year}, p.
/// {page}" around the citation from `facts::cited` (also 11_6's).
pub const QUOTE_ARBITRARY: &str = "The read may return any arbitrary value!";
pub const LAMPORT_SURNAME: &str = "Lamport";
pub const PAGE: &str = "p.";
/// "Lamport on Anatol Holt, publications page".
pub const QUOTE_REFUSED: &str = "He refused to believe it could be true.";
pub const HOLT_SOURCE: &str = "Lamport on Anatol Holt, publications page";
/// 10_4's max that reads a slate twice (code, indented as shown).
pub const MAX_CODE_1: &str = "t = 0";
pub const MAX_CODE_2: &str = "for k in others:";
pub const MAX_CODE_3: &str = "    if number[k] > t:";
pub const MAX_CODE_4: &str = "        t = number[k]";
/// The two reads of number[k].
pub const COMPARE: &str = "compare";
pub const COPY: &str = "copy";
/// "model: {n} customers, flickering slates, read twice: two in the room
/// after {len} steps", the counts from the model's shortest failure.
pub const MODEL_NOTE_A: &str = "model:";
pub const MODEL_NOTE_B: &str = "customers, flickering slates, read twice: two in the room after";
pub const MODEL_STEPS: &str = "steps";
/// "Lamport, Reasoning About Nonatomic Operations, POPL {year}".
pub const NONATOMIC_SOURCE: &str = "Lamport, Reasoning About Nonatomic Operations, POPL";
pub const COULD_COMPUTERS: &str = "Could computers take turns safely with memory like that?";
pub const YES: &str = "Yes.";

// 11_fair_at_a_price
pub const BLUE_FINISHES_FIRST: &str = "Blue finishes choosing first";
pub const BLUE_SERVED_FIRST: &str = "Blue is served first";
/// "{name} first", the name from `draw::NAMES`.
pub const FIRST: &str = "first";
/// "two customers, one visit each: in all {disjoint} orders where one
/// finishes choosing before the other starts, she goes in first; in the
/// other {overlap}, either may", the counts from `facts`.
pub const FCFS_A: &str = "two customers, one visit each: in all";
pub const FCFS_B: &str = "orders where one finishes choosing before the other starts, she goes in first; in the other";
pub const FCFS_C: &str = ", either may";
pub const BLUES_TICKET_WRITTEN: &str = "Blue's ticket written";
pub const ONCE: &str = "once";
/// "each other customer ahead of Blue at most {n}".
pub const AHEAD_AT_MOST: &str = "each other customer ahead of Blue at most";
pub const BIGGEST_TICKET: &str = "biggest ticket";
/// The words of 11_4–11_7's sums, set in TeX's `\text{…}` around numbers
/// from `facts`.
pub const ONE_TICKET_A_MS: &str = "one ticket a millisecond";
pub const ONE_YEAR: &str = "one year";
pub const DAYS: &str = "days";
pub const MS: &str = "ms";
/// Rate × time is a count: one ticket a millisecond for a year is this many.
pub const TICKETS: &str = "tickets";
pub const BILLION: &str = "billion";
pub const LAMPORTS_BOUND: &str = "Lamport's bound";
pub const SIGNED_JAVA_INT: &str = "signed, as Java's int";
pub const BILLION_A_SECOND: &str = "tickets at a billion a second:";
pub const YEARS: &str = "years";
pub const JAVA_LONG: &str = "as Java's long";
pub const READS: &str = "reads";
/// "a {bits}-bit slate".
pub const BIT_SLATE_A: &str = "a";
pub const BIT_SLATE_B: &str = "-bit slate";
/// The bakery paper, p. 455 (its source as 10_3's).
pub const QUOTE_NEVER_LARGER: &str =
    "a read of number[i] can never obtain a value larger than one which has been written there";
pub const GLANCE_MID_WRITE: &str = "a glance mid-write reads";
/// "next ticket: {n}".
pub const NEXT_TICKET: &str = "next ticket:";
/// 11_7's costs card.
pub const FOR_PROGRAMMERS_TITLE: &str = "for programmers";
pub const COST_FIXED: &str = "Customers fixed in advance";
pub const COST_PAUSED: &str = "Paused in line → everyone behind waits";
pub const COST_DEAD: &str = "A dead customer's slate stays set:";
pub const COST_RESET: &str = "    resetting it is your job";
pub const COST_PAD: &str = "Pad each slate to its own cache line";
/// "{bits}-bit tickets (Java: AtomicLongArray)".
pub const WIDE_TICKETS: &str = "-bit tickets (Java: AtomicLongArray)";
pub const FROZEN_TAG: &str = "frozen";
/// "everyone else: at most {n} more visit, then stuck".
pub const STALL_A: &str = "everyone else: at most";
pub const STALL_B: &str = "more visit, then stuck";

// 12_your_cpu
/// The x-ray: stations along a lane, the frames, the fences, what a flag
/// check saw.
pub const STATION_FLAG_UP: &str = "flag up";
pub const STATION_READ: &str = "read";
pub const STATION_TICKET: &str = "ticket";
pub const STATION_FLAG_DOWN: &str = "flag down";
pub const STATION_CHECK_FLAG: &str = "check flag";
pub const STATION_CHECK_SLATE: &str = "check slate";
pub const CORE: &str = "core";
pub const OUTBOX: &str = "outbox";
pub const MEMORY: &str = "memory";
pub const F1: &str = "F1";
pub const F2: &str = "F2";
pub const UP: &str = "up";
pub const DOWN: &str = "down";
/// 12_1: the chip, the promise that breaks, and the measured card.
pub const CPU: &str = "CPU";
pub const PROMISE: &str = "a finished write is seen by every later read";
pub const PROMISE_LEFT: &str = "a finished write is seen";
pub const PROMISE_RIGHT: &str = "by every later read";
pub const MEASURED_AS_WRITTEN: &str = "measured, run as written";
/// "{n} customers · {runs} runs × {visits} visits".
pub const CUSTOMERS_DOT: &str = "customers ·";
pub const RUNS_TIMES: &str = "runs ×";
pub const VISITS: &str = "visits";
pub const CUSTOMERS: &str = "customers";
/// "x86: {cpu}", "Arm: {cpu}", the CPUs from `facts::measured`.
pub const X86_COLON: &str = "x86:";
pub const ARM_COLON: &str = "Arm:";
/// "{n} points lost".
pub const POINTS_LOST: &str = "points lost";
pub const RUST_RELAXED: &str = "Rust, Relaxed loads and stores; the scoreboard sits in the room";
pub const STORE_BUFFER: &str = "programmers say store buffer:";
pub const STORE_BUFFER_TEXT: &str =
    "it's not the cache; caches agree with each other. It's the queue in front of them.";
/// "x86-TSO model:" / "{n} steps," / "nothing delivered".
pub const TSO_MODEL: &str = "x86-TSO model:";
pub const NOTHING_DELIVERED: &str = "nothing delivered";
/// The machine badges (chip label, caption).
pub const X86: &str = "x86";
pub const ARM: &str = "Arm";
pub const INTEL_AMD: &str = "Intel, AMD";
pub const RECENT_MACS: &str = "recent Macs";
/// "MFENCE, … checked up to {n} customers, F1 and F2 suffice, as Abdulla et
/// al. found ({venue})." (the venue from `facts::cited`).
pub const FENCE_NOTE_A: &str = "MFENCE, or a locked instruction such as XCHG. In the x86-TSO model, checked up to";
pub const FENCE_NOTE_B: &str = "customers, F1 and F2 suffice, as Abdulla et al. found";
pub const REORDER_1: &str = "model: an outbox";
pub const REORDER_2: &str = "that may deliver";
pub const REORDER_3: &str = "out of order";
/// 12_7: what not to do, and the cost.
pub const UB_1: &str = "C, C++ and Rust:";
pub const UB_2: &str = "plain shared variables are";
pub const UB_3: &str = "undefined behaviour;";
pub const UB_4: &str = "volatile doesn't help.";
pub const UB_5: &str = "Java: AtomicLongArray works;";
pub const UB_6: &str = "a volatile long[] doesn't.";
pub const SEQCST_COST: &str = "SeqCst is the easy way, not the cheapest: on x86 it fences every store.";
/// 12_7's measured table: "measured: points lost in {visits} visits, {n}
/// customers", the locks, and "x86: {cpu}. Arm: {cpu}; there
/// Release/Acquire lost {lost} of {entries} in a longer run."
pub const POINTS_LOST_IN: &str = "measured: points lost in";
pub const RELAXED: &str = "Relaxed";
pub const RELEASE_ACQUIRE: &str = "Release/Acquire";
pub const SEQCST: &str = "SeqCst";
pub const LONGER_RUN_A: &str = "; there Release/Acquire lost";
pub const LONGER_RUN_B: &str = "of";
pub const LONGER_RUN_C: &str = "in a longer run";

// 13_proudest
pub const RMW: &str = "atomic read-modify-write instructions:";
pub const RMW_TEXT: &str = "test-and-set, compare-and-swap, fetch-and-add";
/// The Turing lecture, its source "Lamport, Turing lecture, {journal},
/// {year}, §{section}" around the citation from `facts::cited`.
pub const QUOTE_LITTLE_USE: &str = "Although of little if any practical use, the bakery algorithm…";
pub const TURING_LECTURE_SOURCE: &str = "Lamport, Turing lecture";
pub const SECTION_SIGN: &str = "§";
/// 13_2: Arm's firmware.
pub const TF_A: &str = "Trusted Firmware-A (TF-A)";
pub const TF_A_WHY: &str = "a bakery lock for a core waking with its cache off";
pub const CACHE_OFF: &str = "data cache: off";
pub const NOT_GUARANTEED: &str = "exclusive-access spinlocks: not guaranteed";
pub const TF_A_PRECISELY: &str = "TF-A takes the bakery lock right after reset, before the data cache is on (warm boot), \
     and only on platforms without hardware coherency (HW_ASSISTED_COHERENCY off): exclusive-access spinlocks need \
     cacheable memory; the bakery lock needs only plain loads and stores";
/// 13_3: the timeline beneath its years, and the pair grid relabelled.
pub const THE_BAKERY: &str = "the bakery";
pub const TIME_CLOCKS: &str = "Time, Clocks";
pub const PAXOS: &str = "Paxos";
pub const BALLOT_NUMBERS: &str = "ballot numbers";
pub const CLOCK_VALUE_ACROSS: &str = "clock value";
pub const PROCESS_UP: &str = "process";
/// "Lamport, publications page".
pub const QUOTE_DID_NOT_INVENT: &str = "I feel that I did not invent the bakery algorithm, I discovered it.";
pub const PUBLICATIONS_SOURCE_SHORT: &str = "Lamport, publications page";
/// The Computer History Museum oral history, its source "Lamport, Computer
/// History Museum oral history ({year}), pp. {pages}", the pages' hyphen
/// an en dash.
pub const QUOTE_THIN_AIR: &str =
    "the bakery algorithm, it just seemed to come out of thin air to me … perhaps that's why I'm proudest of it";
pub const CHM_SOURCE: &str = "Lamport, Computer History Museum oral history";
pub const PAGES: &str = "pp.";
pub const EN_DASH: &str = "–";
/// The receipts card's title, drawn by `explainer_kit::props::receipts_card`
/// (listed for the probe; the chapter does not pass it).
pub const RECEIPTS: &str = "Receipts";
/// The receipts: "Counts and traces: a model checker of Lamport's {year}
/// algorithm", "Outboxes: the x86-TSO model, checked up to {n} customers",
/// "Measured, {n} customers, {visits} visits per kind of lock:", "Quotes:
/// Lamport, {journal}, {year}; Turing lecture, {year};", "    his
/// publications page; Computer History Museum oral history, {year}".
pub const RECEIPT_COUNTS_A: &str = "Counts and traces: a model checker of Lamport's";
pub const RECEIPT_COUNTS_B: &str = "algorithm";
pub const RECEIPT_OUTBOXES: &str = "Outboxes: the x86-TSO model, checked up to";
pub const RECEIPT_MEASURED: &str = "Measured,";
pub const VISITS_PER_LOCK: &str = "visits per kind of lock:";
pub const RECEIPT_QUOTES: &str = "Quotes: Lamport,";
pub const RECEIPT_TURING: &str = "Turing lecture,";
pub const RECEIPT_ORAL_HISTORY: &str = "his publications page; Computer History Museum oral history,";
pub const RECEIPT_FIRMWARE: &str = "Firmware: Trusted Firmware-A, lib/locks/bakery";
pub const RECEIPT_CODE: &str = "Code, data and every number:";
pub const RECEIPT_REPO: &str = "    github.com/Dicklesworthstone/jeffs_math_explainer_videos";

// 09_reading_garbage_safely
/// 09_1: which digit a step of the two-digit rewrite writes.
pub const TENS_DIGIT: &str = "tens digit";
pub const ONES_DIGIT: &str = "ones digit";
/// 09_3: Lamport's three kinds of memory ("On Interprocess Communication"):
/// safe defined on its card, the two stronger kinds by name (R3 maya-05).
pub const ATOMIC_KIND: &str = "atomic";
pub const REGULAR_KIND: &str = "regular";
pub const SAFE_KIND: &str = "safe";
pub const NO_OVERLAP: &str = "a read that overlaps no write returns the last value written";
pub const SAFE_READ: &str = "a read that overlaps a write may return any value the memory can hold";
pub const STRONGER: &str = "stronger";
/// "{source} {year}", the year from `facts::cited`.
pub const INTERPROCESS_SOURCE: &str = "Leslie Lamport, On Interprocess Communication, Distributed Computing,";
/// 09_5's lower-third (R13).
pub const FLAGS_TAG: &str = "flags:";
pub const FLAG_GLANCES: &str = "a glance at a flag mid-raise counts as not yet choosing; mid-lowering, as finished";
/// 09_6: Green, after the garbage.
pub const GREEN_WAITS: &str = "Green waits";
/// 09_6's corner card (R14), around its numbers: "model: flickering slates
/// and flags," / "{n} customers, tickets ≤ {k}: {states} states, never two
/// in the room".
pub const MODEL_A: &str = "model: flickering slates and flags,";
pub const MODEL_B: &str = "customers, tickets ≤";
pub const MODEL_C: &str = "states, never two in the room";

/// Every string above, for the probe.
pub const TEXT: &[&str] = &[
    DIGITS,
    TITLE,
    TORN_A,
    TORN_B,
    TORN_C,
    TURING_SOURCE,
    IMPOSSIBLE_A,
    IMPOSSIBLE_B,
    X,
    MAKE_BIGGER,
    READ_X,
    WRITE_X,
    TIME_RUNS,
    ARROWS,
    PLUS_ONE,
    READ,
    WRITE,
    BLUES_STEPS,
    GREENS_STEPS,
    ORDERS,
    LOSE_A_POINT,
    WHY_SIX,
    PICK_A,
    PICK_B,
    PICK_C,
    MIDDLE_1,
    MIDDLE_2,
    PRECISELY,
    A_COUNT,
    STRIP_NOTE,
    "“”",
    THE_ROOM,
    NAMES,
    CRITICAL_SECTION,
    super::draw::VACANT,
    super::draw::OCCUPIED,
    CHECK,
    FLIP,
    LEAVE,
    BOTH_INSIDE,
    PUT_BOTH_IN,
    COUNTING,
    COUNTING_RULE,
    DIJKSTRA,
    KNUTH,
    RULE_1,
    RULE_2,
    RULE_3,
    RULE_4,
    MAGIC,
    ORDINARY,
    THE_WORD,
    ATOMIC,
    QUESTION,
    LAMPORT,
    PROCESS,
    TICKET_MACHINE,
    WAITS,
    FOR_PROGRAMMERS,
    TICKET_LOCK,
    RULE_SMALLER,
    RULE_SMALLER_OR_EQUAL,
    AFTER_YOU,
    NO_AFTER_YOU,
    PAIR,
    CAT,
    COW,
    COMES_BEFORE,
    TICKET_ACROSS,
    CUSTOMER_NUMBER_UP,
    SERVED,
    TOTALLY_ORDERED,
    PURPLES_STEPS,
    TWO_INSIDE_KEY,
    PUT_TWO_IN,
    TRAILS,
    TRAILS_A,
    TRAILS_B,
    STEPS,
    UNWRITTEN,
    PINK_HOME,
    TIME,
    BLUE_CHOOSING,
    GREEN_CHOOSING,
    BLUE_IN_ROOM,
    GREEN_IN_ROOM,
    SUPPOSE_BOTH,
    LAST_LOOK,
    GREEN_AT,
    NOT_YET,
    CHOOSING,
    DONE_CHOOSING,
    COUNTS_ON,
    COUNTS_ON_TEXT,
    SEES_TICKET,
    FACT_ONE,
    FACT_ONE_TEXT,
    FACT_TWO,
    FACT_TWO_TEXT,
    WHY,
    GREEN_BIGGER,
    FLAG_UP,
    REAL_TICKET,
    PAIR_SMALLER,
    FOR_BLUE,
    FOR_GREEN,
    IMPOSSIBLE_PAIR,
    SEARCH_A,
    SEARCH_B,
    SEARCH_C,
    SEARCH_D,
    SEARCH_E,
    SEARCH_F,
    NO_VIOLATION,
    A_VIOLATION,
    THE_PROOF,
    EVERY_CASE,
    FLAG_UP_WORD,
    FLAG_DOWN_WORD,
    STEPS_WORD,
    FLAG_RULE_1,
    FLAG_RULE_2,
    FLAG_RULE_3,
    WAITS_WORD,
    WITHOUT_FLAGS,
    WITH_FLAGS,
    ONE_VISIT_EACH,
    CHOOSING_TIME,
    DOORWAY,
    A_LOCK,
    THESIS,
    // 10_the_old_zero
    STEP,
    FACT_ONE_SWAPPED,
    FACT_ONE_SWAPPED_TEXT,
    CHIPS_DONT_TEAR,
    QUOTE_DISCOVERED,
    PUBLICATIONS_SOURCE,
    QUOTE_ARBITRARY,
    LAMPORT_SURNAME,
    PAGE,
    QUOTE_REFUSED,
    HOLT_SOURCE,
    COMPARE,
    COPY,
    MODEL_NOTE_A,
    MODEL_NOTE_B,
    MODEL_STEPS,
    NONATOMIC_SOURCE,
    COULD_COMPUTERS,
    YES,
    // 11_fair_at_a_price
    BLUE_FINISHES_FIRST,
    BLUE_SERVED_FIRST,
    FIRST,
    FCFS_A,
    FCFS_B,
    FCFS_C,
    BLUES_TICKET_WRITTEN,
    ONCE,
    AHEAD_AT_MOST,
    BIGGEST_TICKET,
    ONE_TICKET_A_MS,
    ONE_YEAR,
    DAYS,
    MS,
    TICKETS,
    BILLION,
    LAMPORTS_BOUND,
    SIGNED_JAVA_INT,
    BILLION_A_SECOND,
    YEARS,
    JAVA_LONG,
    READS,
    BIT_SLATE_A,
    BIT_SLATE_B,
    QUOTE_NEVER_LARGER,
    GLANCE_MID_WRITE,
    NEXT_TICKET,
    FOR_PROGRAMMERS_TITLE,
    COST_FIXED,
    COST_PAUSED,
    COST_DEAD,
    COST_RESET,
    COST_PAD,
    WIDE_TICKETS,
    FROZEN_TAG,
    STALL_A,
    STALL_B,
    // 12_your_cpu
    STATION_FLAG_UP,
    STATION_READ,
    STATION_TICKET,
    STATION_FLAG_DOWN,
    STATION_CHECK_FLAG,
    STATION_CHECK_SLATE,
    CORE,
    OUTBOX,
    MEMORY,
    F1,
    F2,
    UP,
    DOWN,
    CPU,
    PROMISE,
    PROMISE_LEFT,
    PROMISE_RIGHT,
    MEASURED_AS_WRITTEN,
    CUSTOMERS_DOT,
    RUNS_TIMES,
    VISITS,
    CUSTOMERS,
    X86_COLON,
    ARM_COLON,
    POINTS_LOST,
    RUST_RELAXED,
    STORE_BUFFER,
    STORE_BUFFER_TEXT,
    TSO_MODEL,
    NOTHING_DELIVERED,
    X86,
    ARM,
    INTEL_AMD,
    RECENT_MACS,
    FENCE_NOTE_A,
    FENCE_NOTE_B,
    REORDER_1,
    REORDER_2,
    REORDER_3,
    UB_1,
    UB_2,
    UB_3,
    UB_4,
    UB_5,
    UB_6,
    SEQCST_COST,
    POINTS_LOST_IN,
    RELAXED,
    RELEASE_ACQUIRE,
    SEQCST,
    LONGER_RUN_A,
    LONGER_RUN_B,
    LONGER_RUN_C,
    // 13_proudest
    RMW,
    RMW_TEXT,
    QUOTE_LITTLE_USE,
    TURING_LECTURE_SOURCE,
    SECTION_SIGN,
    TF_A,
    TF_A_WHY,
    CACHE_OFF,
    NOT_GUARANTEED,
    TF_A_PRECISELY,
    THE_BAKERY,
    TIME_CLOCKS,
    PAXOS,
    BALLOT_NUMBERS,
    CLOCK_VALUE_ACROSS,
    PROCESS_UP,
    QUOTE_DID_NOT_INVENT,
    PUBLICATIONS_SOURCE_SHORT,
    QUOTE_THIN_AIR,
    CHM_SOURCE,
    PAGES,
    EN_DASH,
    RECEIPTS,
    RECEIPT_COUNTS_A,
    RECEIPT_COUNTS_B,
    RECEIPT_OUTBOXES,
    RECEIPT_MEASURED,
    VISITS_PER_LOCK,
    RECEIPT_QUOTES,
    RECEIPT_TURING,
    RECEIPT_ORAL_HISTORY,
    RECEIPT_FIRMWARE,
    RECEIPT_CODE,
    RECEIPT_REPO,
    TENS_DIGIT,
    ONES_DIGIT,
    ATOMIC_KIND,
    REGULAR_KIND,
    SAFE_KIND,
    SAFE_READ,
    NO_OVERLAP,
    STRONGER,
    INTERPROCESS_SOURCE,
    FLAGS_TAG,
    FLAG_GLANCES,
    GREEN_WAITS,
    MODEL_A,
    MODEL_B,
    MODEL_C,
];

/// Code, in CM Typewriter.
pub const MONO: &[&str] = &[CODE, X, MAX_CODE_1, MAX_CODE_2, MAX_CODE_3, MAX_CODE_4];

/// Math, in TeX.
pub const TEX: &[&str] = &[PAIRS_FOR_BLUE, PAIRS_FOR_GREEN, PAIRS_CHAIN];
