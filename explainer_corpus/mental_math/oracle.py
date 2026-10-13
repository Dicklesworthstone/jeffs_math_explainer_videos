#!/usr/bin/env python3
"""Facts oracle for Video F, "Mental Math Tricks That Actually Work (and Why)".

Run from this folder (`./x mental_math facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `mental facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: F-H (the hook), F-T
(the tricks), F-Q (the quizzes), F-X (the edges of the always sentences) and
F-C (cited).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model:
  * every answer by plain multiplication of Python integers or Fractions,
    never by the trick the video uses to reach it;
  * the pictures by counting unit squares: every cell of the square or the
    rectangle is put in its piece and the pieces are counted;
  * the two-digit readings of squares near 50 by slicing the decimal string
    of the plain square;
  * the paper method by a second long multiplication, written on digit
    strings;
  * the trick's own intermediate numbers (the steps a worked quiz shows) are
    then written down from the trick's description and cross-checked against
    the plain answer;
  * CITED literals from the sources listed in docs/fact-sheet.md.
"""
import sys
from fractions import Fraction as F

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    if isinstance(value, bool):
        value = "true" if value else "false"
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def join(items):
    return ",".join(str(x) for x in items)


def half_up(x, places):
    """x rounded half up to `places` decimals, as an integer of that many places."""
    scaled = F(x) * 10**places
    return (2 * scaled.numerator + scaled.denominator) // (2 * scaled.denominator)


def money(cents):
    cents = F(cents)
    assert cents.denominator == 1, f"not whole cents: {cents}"
    return f"{cents.numerator // 100}.{cents.numerator % 100:02d}"


def split(x):
    """The ones, and everything above them."""
    ones = int(str(x)[-1])
    return x - ones, ones


def round_to_ten(x):
    return 10 * half_up(F(x, 10), 0)


def up_to(x, step):
    """The next multiple of `step` at or above x."""
    return -(-x // step) * step


# --- the paper method, on digit strings -----------------------------------------


def paper(a, b):
    """Rows and held carries of school long multiplication.

    A carry is held when another digit product of the same row, or another
    column of the sum that still has digits in it, comes after it.
    """
    top = str(a)[::-1]
    rows, held = [], 0
    for shift, m in enumerate(str(b)[::-1]):
        out, carry = "", 0
        for i, t in enumerate(top):
            p = int(t) * int(m) + carry
            out = str(p % 10) + out
            carry = p // 10
            if carry and i + 1 < len(top):
                held += 1
        if carry:
            out = str(carry) + out
        rows.append(int(out + "0" * shift))
    width = max(len(str(r)) for r in rows)
    carry = 0
    for col in range(width):
        column = sum(r // 10**col % 10 for r in rows) + carry
        carry = column // 10
        if carry and col + 1 < width:
            held += 1
    return rows, held


# --- pictures, by counting unit squares -----------------------------------------


def count_cells(height, width, row_cut, col_cut):
    """The cells of a height-by-width grid, counted by which side of each cut they are on."""
    counts = {(r, c): 0 for r in (0, 1) for c in (0, 1)}
    for i in range(height):
        for j in range(width):
            counts[(int(i >= row_cut), int(j >= col_cut))] += 1
    return counts


# --- the quizzes: line, trick, what is asked, countdown ---------------------------

QUIZZES = [
    ("01_1", "ends_in_five", ("square", 85), 4),
    ("03_1", "ends_in_five", ("square", 25), 4),
    ("03_3", "ends_in_five", ("square", 65), 4),
    ("03_5", "ends_in_five", ("square", 105), 6),
    ("05_4", "by_parts", ("times", 6, 43), 5),
    ("05_6", "by_parts", ("times", 13, 14), 6),
    ("06_6", "pair", ("times", 19, 21), 4),
    ("06_8", "pair", ("times", 65, 75), 6),
    ("09_1", "near_fifty", ("square", 52), 4),
    ("09_3", "near_fifty", ("square", 46), 5),
    ("09_5", "near_fifty", ("square", 57), 5),
    ("10_4", "percent_blocks", ("percent_of_money", 15, 6000), 5),
    ("10_9", "percent_swap", ("percent_of", 4, 75), 5),
    ("11_5", "times_five", ("times", 5, 64), 4),
    ("11_7", "halve_and_double", ("times", 16, 45), 5),
    ("11_9", "round_then_fix", ("times_money", 3, 499), 5),
    ("12_1", "ends_in_five", ("square", 75), 5),
    ("12_3", "pair", ("times", 49, 51), 5),
    ("12_5", "near_fifty", ("square", 54), 5),
    ("12_7", "percent_blocks", ("percent_of_money", 20, 4500), 5),
    ("12_9", "round_then_fix", ("times", 6, 49), 5),
    ("13_1", "by_parts", ("times", 67, 78), 6),
]


def ask_text(ask):
    kind, *n = ask
    return {
        "square": lambda: f"{n[0]}^2",
        "times": lambda: f"{n[0]}x{n[1]}",
        "percent_of_money": lambda: f"{n[0]}% of ${money(n[1])}",
        "percent_of": lambda: f"{n[0]}% of {n[1]}",
        "times_money": lambda: f"{n[0]}x${money(n[1])}",
    }[kind]()


def running_totals(a, b):
    """The pieces of a times b by its parts, big block first, and their running sum."""
    totals, total = [], 0
    for h in split(a):
        for w in split(b):
            if h * w:
                total += h * w
                totals.append(total)
    return totals


def worked(trick, ask):
    """(steps, answer): the answer by plain arithmetic, the steps from the trick's description."""
    kind, *n = ask
    if trick == "ends_in_five":
        (x,) = n
        square = x * x
        front = int(str(x)[:-1] or "0")
        check(f"{x}^2 ends in 25", square % 100 == 25)
        hundreds = square // 100
        check(f"{x}^2: the hundreds are {front} x {front + 1}", hundreds == front * (front + 1))
        return [front, front + 1, hundreds], square
    if trick == "by_parts":
        a, b = n
        totals = running_totals(a, b)
        check(f"{a} x {b}: the pieces add up", totals[-1] == a * b)
        return totals, a * b
    if trick == "pair":
        a, b = sorted(n)
        middle, distance = F(a + b, 2), F(b - a, 2)
        check(f"{a} x {b}: a whole middle", middle.denominator == 1)
        check(f"{a} x {b} = {middle}^2 - {distance}^2", middle**2 - distance**2 == a * b)
        return [middle, distance, middle**2, distance**2], a * b
    if trick == "near_fifty":
        (x,) = n
        square = str(x * x)
        front, back = square[:-2], square[-2:]
        check(f"{x}^2 reads as 25 + ({x} - 50), then ({x} - 50)^2", int(front) == 25 + (x - 50) and int(back) == (x - 50) ** 2)
        return [x - 50, front, back], int(square)
    if trick == "percent_blocks":
        percent, cents = n
        ten = F(cents, 10)
        answer = F(percent, 100) * cents
        if percent == 15:
            check(f"15% of {money(cents)} is the tenth and half of it", ten + ten / 2 == answer)
            return [money(ten), money(ten / 2)], money(answer)
        check(f"20% of {money(cents)} is the tenth doubled", percent == 20 and 2 * ten == answer)
        return [money(ten)], money(answer)
    if trick == "percent_swap":
        x, y = n
        answer = F(x, 100) * y
        check(f"{x}% of {y} = {y}% of {x}", answer == F(y, 100) * x and answer.denominator == 1)
        return [y, x], answer
    if trick == "times_five":
        five, x = n
        check("times five", five == 5 and (x * 10) % 2 == 0)
        return [x * 10], 5 * x
    if trick == "halve_and_double":
        a, b = n
        check(f"{a} x {b}: {a} is even", a % 2 == 0)
        check(f"{a} x {b} = {a // 2} x {2 * b}", (a // 2) * (2 * b) == a * b)
        return [a // 2, 2 * b], a * b
    if trick == "round_then_fix":
        count, each = n
        step = 100 if kind == "times_money" else 10
        show = money if kind == "times_money" else (lambda v: v)
        round_ = up_to(each, step)
        fix = count * round_ - count * each
        check(f"{count} x {each}: the fix is {count} x {round_ - each}", fix == count * (round_ - each))
        return [show(round_), show(count * round_), show(fix)], show(count * each)
    raise SystemExit(f"oracle: no trick {trick}")


def main():
    # --- F-H: eighty-five squared on paper, and by the trick ---------------------
    hook = 85
    rows, held = paper(hook, hook)
    emit("F-H1.paper.rows", join(rows))
    emit("F-H1.paper.row_count", len(rows))
    emit("F-H1.paper.carries", held)
    emit("F-H1.paper.product", hook * hook)
    check("the paper rows add up", sum(rows) == hook * hook)
    steps, answer = worked("ends_in_five", ("square", hook))
    emit("F-H1.trick.front", steps[0])
    emit("F-H1.trick.next", steps[1])
    emit("F-H1.trick.hundreds", steps[2])
    emit("F-H1.trick.square", answer)

    # --- F-T1: squares ending in 5 ---------------------------------------------
    for number in (35, 105, 135, 5):
        steps, answer = worked("ends_in_five", ("square", number))
        emit(f"F-T1.n{number}.front", steps[0])
        emit(f"F-T1.n{number}.next", steps[1])
        emit(f"F-T1.n{number}.hundreds", steps[2])
        emit(f"F-T1.n{number}.square", answer)
        # the block with both strips beside it: as tall as the tens, ten longer
        tens, ones = split(number)
        emit(f"F-T1.n{number}.grown", f"{tens}x{tens + 2 * ones}")
        check(f"{number}^2 is that rectangle and a corner", tens * (tens + 2 * ones) + ones * ones == number * number)
    side = 35
    tens, ones = split(side)
    cells = count_cells(side, side, tens, tens)
    block, strip, other_strip, corner = cells[(0, 0)], cells[(0, 1)], cells[(1, 0)], cells[(1, 1)]
    check("the two strips of the square are the same size", strip == other_strip)
    emit("F-T1.cut.parts", join([tens, ones]))
    emit("F-T1.cut.block", block)
    emit("F-T1.cut.strip", strip)
    emit("F-T1.cut.two_strips", strip + other_strip)
    emit("F-T1.cut.corner", corner)
    # the block with both strips laid beside it: tens wide, tens + 2 ones long
    grown = (tens, tens + 2 * ones)
    emit("F-T1.cut.grown", f"{grown[0]}x{grown[1]}")
    emit("F-T1.cut.grown_area", block + strip + other_strip)
    check("the grown block is a rectangle", grown[0] * grown[1] == block + strip + other_strip)
    # ruled into squares of a hundred, ten on a side
    emit("F-T1.cut.hundreds_tiles", f"{grown[0] // 10}x{grown[1] // 10}")
    check("twelve squares of a hundred", (grown[0] // 10) * (grown[1] // 10) * 100 == block + strip + other_strip)
    emit("F-T1.cut.total", sum(cells.values()))
    check("the cells of the square", sum(cells.values()) == side * side)
    half = F(15, 2) ** 2
    emit("F-T1.half.squared", money(half * 100))
    emit("F-T1.half.front_product", 7 * 8)
    check("7.5^2 is 7 x 8 and a quarter", half == 7 * 8 + F(1, 4))

    # --- F-T2: 23 x 46 by parts --------------------------------------------------
    a, b = 23, 46
    (a_tens, a_ones), (b_tens, b_ones) = split(a), split(b)
    cells = count_cells(a, b, a_tens, b_tens)
    pieces = [cells[(0, 0)], cells[(0, 1)], cells[(1, 0)], cells[(1, 1)]]
    emit("F-T2.parts.first", join([a_tens, a_ones]))
    emit("F-T2.parts.second", join([b_tens, b_ones]))
    emit("F-T2.pieces.sides", join(f"{h}x{w}" for h in (a_tens, a_ones) for w in (b_tens, b_ones)))
    emit("F-T2.pieces", join(pieces))
    running = [sum(pieces[: i + 1]) for i in range(len(pieces))]
    emit("F-T2.running", join(running))
    emit("F-T2.product", a * b)
    check("23 x 46: the counted pieces add up", running[-1] == a * b)
    check("23 x 46: the running totals of the description", running == running_totals(a, b))
    lead = str(pieces[0]).rstrip("0")
    emit("F-T2.size.lead", lead)
    emit("F-T2.size.zeros", len(str(pieces[0])) - len(lead))
    check("the big block is the leading digits multiplied", int(lead) == (a_tens // 10) * (b_tens // 10))
    emit("F-T2.size.too_small", a_tens * (b_tens // 10))
    emit("F-T2.size.too_big", (a_tens * 10) * b_tens)
    emit("F-T2.estimate", join([round_to_ten(a), round_to_ten(b), round_to_ten(a) * round_to_ten(b)]))
    emit("F-T2.held.pieces", sum(1 for p in pieces if p))

    # --- F-T3: 47 x 53 -----------------------------------------------------------
    x, y = 47, 53
    steps, answer = worked("pair", ("times", x, y))
    middle, distance = int(steps[0]), int(steps[1])
    emit("F-T3.middle", middle)
    emit("F-T3.distance", distance)
    emit("F-T3.middle_squared", steps[2])
    emit("F-T3.distance_squared", steps[3])
    emit("F-T3.product", answer)
    # the 47-by-53 rectangle laid over the 50-by-50 square, counted cell by cell
    outside = sum(1 for i in range(x) for j in range(y) if j >= middle)
    gap = sum(1 for i in range(middle) for j in range(middle) if i >= x)
    emit("F-T3.strip", f"{distance}x{x}")
    emit("F-T3.strip_area", outside)
    emit("F-T3.gap", f"{distance}x{middle}")
    emit("F-T3.gap_area", gap)
    check("the strip fills the gap but for the corner", gap - outside == distance**2)
    emit("F-T3.no_middle", (47 + 56) % 2 == 1)
    emit("F-T3.no_middle.sum", 47 + 56)

    # --- F-T4: squares near 50 ---------------------------------------------------
    emit("F-T4.fifty_squared", 50 * 50)
    for name, number in (("below", 48), ("above", 53), ("carries", 62)):
        d = number - 50
        square = number * number
        corner = d * d
        emit(f"F-T4.{name}.number", number)
        emit(f"F-T4.{name}.distance", d)
        emit(f"F-T4.{name}.hundreds", (square - corner) // 100)
        emit(f"F-T4.{name}.rough", square - corner)
        emit(f"F-T4.{name}.corner", corner)
        emit(f"F-T4.{name}.corner_fits", corner < 100)
        emit(f"F-T4.{name}.square", square)
        # one strip: fifty long, as wide as the distance
        strip = sum(1 for i in range(50) for j in range(abs(d)))
        emit(f"F-T4.{name}.strip", strip)
        emit(f"F-T4.{name}.two_strips", 2 * strip)
        check(f"{number}^2 = 2500 + 100 x {d} + {corner}", square == 2500 + 100 * d + corner)
    cells = count_cells(53, 53, 50, 50)
    emit("F-T4.above.long_strip", f"{2 * 50}x{3}")
    emit("F-T4.above.total", sum(cells.values()))
    check("the two strips of 53^2 make three hundreds", cells[(0, 1)] + cells[(1, 0)] == 300)
    for name, number in (("above", 53), ("below", 48)):
        text = str(number * number)
        emit(f"F-T4.{name}.digit_pairs", f"{text[:-2]},{text[-2:]}")
    # below 50: the square of 50 less two strips, with the corner they share put back
    inside = sum(1 for i in range(50) for j in range(50) if i < 48 and j < 48)
    check("48^2 is 2500 less two strips of 100 plus the corner", inside == 2500 - 2 * 100 + 4 == 48 * 48)

    # --- F-T5: percents ----------------------------------------------------------
    bill = 4800
    emit("F-T5.bill", money(bill))
    emit("F-T5.bill.ten", money(F(10, 100) * bill))
    emit("F-T5.bill.five", money(F(5, 100) * bill))
    emit("F-T5.bill.fifteen", money(F(15, 100) * bill))
    emit("F-T5.bill.twenty", money(F(20, 100) * bill))
    emit("F-T5.swap.area", sum(1 for i in range(8) for j in range(25)))
    emit("F-T5.swap.one_way", F(8, 100) * 25)
    emit("F-T5.swap.other_way", F(25, 100) * 8)
    no_swap = F(17, 100) * 43
    emit("F-T5.no_swap", money(no_swap * 100))
    emit("F-T5.no_swap.is_whole", no_swap.denominator == 1)
    check("17% of 43 is 43% of 17", no_swap == F(43, 100) * 17)

    # --- F-T6: trading a factor of two ----------------------------------------------
    emit("F-T6.five.ten", 86 * 10)
    emit("F-T6.five.product", 5 * 86)
    emit("F-T6.twenty_five.hundred", 36 * 100)
    emit("F-T6.twenty_five.half", 36 * 100 // 2)
    emit("F-T6.twenty_five.product", 25 * 36)
    check("a quarter of 3600", F(3600, 4) == 25 * 36)
    emit("F-T6.halve.sides", f"{14 // 2}x{35 * 2}")
    emit("F-T6.halve.product", 14 * 35)
    check("14 x 35 = 7 x 70", 7 * 70 == 14 * 35)
    emit("F-T6.no_halve", 15 % 2 == 1 and 37 % 2 == 1)

    # --- F-T7: round, then fix ---------------------------------------------------
    emit("F-T7.nines.rounded", 10 * 37)
    emit("F-T7.nines.fix", 10 * 37 - 9 * 37)
    emit("F-T7.nines.product", 9 * 37)
    emit("F-T7.price.each", money(1999))
    emit("F-T7.price.round", money(up_to(1999, 100)))
    emit("F-T7.price.rounded", money(4 * up_to(1999, 100)))
    emit("F-T7.price.fix", money(4 * up_to(1999, 100) - 4 * 1999))
    emit("F-T7.price.total", money(4 * 1999))

    # --- F-T9: the one move: a square is a rectangle and a small square -----------
    for a, d in ((35, 5), (48, 2)):
        # the square with a strip d wide moved from the top to the side, counted:
        # every cell below the strip, then the strip's cells beside it but for the corner
        kept = sum(1 for i in range(a - d) for j in range(a))
        moved = sum(1 for i in range(a - d) for j in range(d))
        emit(f"F-T9.n{a}.rectangle", f"{a - d}x{a + d}")
        emit(f"F-T9.n{a}.rectangle_area", kept + moved)
        emit(f"F-T9.n{a}.corner", a * a - kept - moved)
        emit(f"F-T9.n{a}.square", a * a)
        check(f"{a}^2: the corner is {d} by {d}", a * a - kept - moved == d * d)

    # --- F-Q: the quizzes ------------------------------------------------------------
    emit("F-Q.count", len(QUIZZES))
    emit("F-Q.countdown_seconds", sum(q[3] for q in QUIZZES))
    for line, trick, ask, seconds in QUIZZES:
        steps, answer = worked(trick, ask)
        emit(f"F-Q.{line}.ask", ask_text(ask))
        emit(f"F-Q.{line}.trick", trick)
        emit(f"F-Q.{line}.seconds", seconds)
        emit(f"F-Q.{line}.steps", join(steps))
        emit(f"F-Q.{line}.answer", answer)
    # the last quiz: its estimate, and a decimal point slipped either way
    line, _, (_, a, b), _ = QUIZZES[-1]
    ra, rb = round_to_ten(a), round_to_ten(b)
    emit(f"F-Q.{line}.estimate", join([ra, rb, ra * rb]))
    emit(f"F-Q.{line}.slipped_low", str(a * b)[:-1] + "." + str(a * b)[-1])
    emit(f"F-Q.{line}.slipped_high", a * b * 10)
    emit(f"F-Q.{line}.middle_halves", a + b)
    check("the last quiz has no whole middle and no five", (a + b) % 2 == 1 and a % 10 != 5 and b % 10 != 5)

    # --- F-X: the edges of the always sentences -------------------------------------
    def tally(key, cases):
        cases = list(cases)
        emit(f"{key}.holds", sum(1 for ok in cases if ok))
        emit(f"{key}.checked", len(cases))

    tally("F-X1.ends_in_five", ((10 * n + 5) ** 2 == 100 * n * (n + 1) + 25 for n in range(10_000)))
    emit("F-X1.ends_in_five.largest", max(10 * n + 5 for n in range(10_000)))

    def parts_hold(a, b):
        totals, total, last = True, 0, 0
        for h in split(a):
            for w in split(b):
                if h * w:
                    # the piece is its leading digits multiplied, then the zeros of both
                    lead = int(str(h).rstrip("0")) * int(str(w).rstrip("0"))
                    zeros = len(str(h)) - len(str(h).rstrip("0")) + len(str(w)) - len(str(w).rstrip("0"))
                    totals &= lead * 10**zeros == h * w
                    total += h * w
                    totals &= total >= last
                    last = total
        return totals and total == a * b

    tally("F-X2.by_parts", (parts_hold(a, b) for a in range(1, 100) for b in range(1, 100)))
    tally(
        "F-X3.pair",
        (
            F(x + y, 2) ** 2 - F(y - x, 2) ** 2 == x * y
            for x in range(1, 201)
            for y in range(x, 201)
            if (x + y) % 2 == 0
        ),
    )
    tally("F-X4.near_fifty", (2500 + 100 * (x - 50) + (x - 50) ** 2 == x * x for x in range(101)))
    reads = [
        x
        for x in range(101)
        if (x - 50) ** 2 < 100 and f"{25 + (x - 50)}{(x - 50) ** 2:02d}" == str(x * x)
    ]
    emit("F-X4.digit_pairs.first", reads[0])
    emit("F-X4.digit_pairs.last", reads[-1])
    check("the two-digit reading holds on an unbroken run", reads == list(range(reads[0], reads[-1] + 1)))
    tally("F-X5.swap", (F(x, 100) * y == F(y, 100) * x for x in range(201) for y in range(201)))
    tally("F-X6.halve", ((a // 2) * (2 * b) == a * b for a in range(2, 201, 2) for b in range(1, 201)))
    tally(
        "F-X7.round_then_fix",
        (
            count * up_to(each, 10) - count * (up_to(each, 10) - each) == count * each
            for count in range(1, 100)
            for each in range(1, 1000)
        ),
    )
    tally("F-X8.one_move", ((a - d) * (a + d) + d * d == a * a for a in range(1, 201) for d in range(a + 1)))

    # --- F-C: cited ----------------------------------------------------------------
    # Feynman, "Surely You're Joking, Mr. Feynman!" (W. W. Norton, 1985), "Lucky
    # Numbers", pp. 193-94
    emit("F-C1.feynman.year", 1985)
    emit("F-C1.feynman.pages", "193-94")
    emit("F-C1.bethe.number", 48)
    emit("F-C1.bethe.rough", 2300)
    emit("F-C1.bethe.exact", 2304)
    check("Bethe's exact answer is 48 squared", 48 * 48 == 2304)
    check("Bethe's first answer is 2500 less 100 times 2", 2500 - 100 * 2 == 2300)
    # Chase, The Elements of Arithmetic, Part Second (Philadelphia, 1844), p. 60
    emit("F-C3.chase.year", 1844)
    emit("F-C3.chase.page", 60)
    emit("F-C3.chase.square", 1225)
    emit("F-C3.chase.pair", 6351)
    emit("F-C3.chase.near_fifty", 2809)
    check("Chase's three examples", 35 * 35 == 1225 and 87 * 73 == 6351 and 53 * 53 == 2809)
    # Karatsuba and Ofman, Doklady Akademii Nauk SSSR 145 (1962): 23 x 46 from the
    # block, the corner and one product of the two sums
    emit("F-C4.karatsuba.year", 1962)
    block, corner, sums = 20 * 40, 3 * 6, (20 + 3) * (40 + 6)
    emit("F-C4.karatsuba.multiplications", len((block, corner, sums)))
    emit("F-C4.karatsuba.strips", sums - block - corner)
    emit("F-C4.karatsuba.product", 23 * 46)
    check("the two strips come from one product", sums - block - corner == 20 * 6 + 3 * 40)

    print(
        f"# {len(_EMITTED)} keys, {_CHECKS[0] - len(_FAILED)} of {_CHECKS[0]} cross-checks passed",
        file=sys.stderr,
    )
    if _FAILED:
        print("# FAILED: " + "; ".join(_FAILED), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
