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
pub const RULE_2: &str = "a stop outside holds up nobody";
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
];

/// Code, in CM Typewriter.
pub const MONO: &[&str] = &[CODE, X];
