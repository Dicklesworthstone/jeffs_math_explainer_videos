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
];

/// Code, in CM Typewriter.
pub const MONO: &[&str] = &[CODE, X];

/// Math, in TeX.
pub const TEX: &[&str] = &[PAIRS_FOR_BLUE, PAIRS_FOR_GREEN, PAIRS_CHAIN];
