#!/usr/bin/env python3
"""Facts oracle for Video F, "Mental Math Tricks That Actually Work (and Why)".

Run from this folder (`./x mental_math facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `mental facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: F-H (the hook), F-T
(the tricks), F-M (the one move), F-N (the product with nothing to grab), F-Q
(the quizzes), F-X (the edges of the always sentences) and F-C (cited).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model:
  * every answer by plain multiplication of Python integers or Fractions,
    never by the trick the video uses to reach it;
  * the pictures by counting unit squares: the moved-strip figure is built
    as a set of cells, and every cell of a rectangle is put in its piece;
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
    rows, held = [], []
    for shift, m in enumerate(str(b)[::-1]):
        out, carry = "", 0
        for i, t in enumerate(top):
            p = int(t) * int(m) + carry
            out = str(p % 10) + out
            carry = p // 10
            if carry and i + 1 < len(top):
                held.append(carry)
        if carry:
            out = str(carry) + out
        rows.append(int(out + "0" * shift))
    width = max(len(str(r)) for r in rows)
    carry = 0
    for col in range(width):
        column = sum(r // 10**col % 10 for r in rows) + carry
        carry = column // 10
        if carry and col + 1 < width:
            held.append(carry)
    return rows, held


# --- pictures, by counting unit squares -----------------------------------------


def count_cells(height, width, row_cut, col_cut):
    """The cells of a height-by-width grid, counted by which side of each cut they are on."""
    counts = {(r, c): 0 for r in (0, 1) for c in (0, 1)}
    for i in range(height):
        for j in range(width):
            counts[(int(i >= row_cut), int(j >= col_cut))] += 1
    return counts


def moved(side, strip):
    """The one move, by counting cells: a strip `strip` deep sliced off the top of a
    square and stood against its right side. (wide, tall, cells in the rectangle, cells
    that stand above it, all cells)."""
    cells = set()
    for i in range(side - strip):  # rows, from the bottom: what is left of the square
        for j in range(side):
            cells.add((i, j))
    for i in range(side):  # the strip, turned, against the right side
        for j in range(strip):
            cells.add((i, side + j))
    wide, tall = side + strip, side - strip
    rectangle = sum(1 for (i, j) in cells if i < tall)
    return wide, tall, rectangle, len(cells) - rectangle, len(cells)


# --- the quizzes: line, trick, what is asked, countdown ---------------------------

QUIZZES = [
    ("01_1", "ends_in_five", ("square", 85), 4),
    ("03_1", "ends_in_five", ("square", 25), 5),
    ("03_3", "ends_in_five", ("square", 105), 6),
    ("05_4", "by_parts", ("times", 6, 43), 5),
    ("05_6", "by_parts", ("times", 13, 14), 6),
    ("06_4", "percent_blocks", ("percent_of_money", 15, 6000), 5),
    ("07_3", "halve_and_double", ("times", 16, 45), 5),
    ("07_6", "round_then_fix", ("times_money", 3, 499), 5),
    ("08_6", "pair", ("times", 19, 21), 5),
    ("11_1", "near_fifty", ("square", 46), 5),
    ("12_1", "percent_blocks", ("percent_of_money", 20, 4500), 5),
    ("12_3", "pair", ("times", 49, 51), 5),
    ("13_1", "by_parts", ("times", 67, 78), 6),
    ("14_1", "ends_in_five", ("square", 55), 5),
]


def ask_text(ask):
    kind, *n = ask
    return {
        "square": lambda: f"{n[0]}^2",
        "times": lambda: f"{n[0]}x{n[1]}",
        "percent_of_money": lambda: f"{n[0]}% of ${money(n[1])}",
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
    if trick == "halve_and_double":
        a, b = n
        check(f"{a} x {b}: {a} is even", a % 2 == 0)
        check(f"{a} x {b} = {a // 2} x {2 * b}", (a // 2) * (2 * b) == a * b)
        return [a // 2, 2 * b], a * b
    if trick == "round_then_fix":
        count, cents = n
        round_ = up_to(cents, 100)
        fix = count * round_ - count * cents
        check(f"{count} x {money(cents)}: the fix is {count} x {round_ - cents} cents", fix == count * (round_ - cents))
        return [money(round_), money(count * round_), money(fix)], money(count * cents)
    raise SystemExit(f"oracle: no trick {trick}")


def main():
    # --- F-H: eighty-five squared on paper, and by the trick ---------------------
    hook = 85
    rows, held = paper(hook, hook)
    emit("F-H1.paper.rows", join(rows))
    emit("F-H1.paper.row_count", len(rows))
    emit("F-H1.paper.carries", len(held))
    emit("F-H1.paper.carried", join(held))
    emit("F-H1.paper.product", hook * hook)
    check("the paper rows add up", sum(rows) == hook * hook)
    steps, answer = worked("ends_in_five", ("square", hook))
    emit("F-H1.trick.front", steps[0])
    emit("F-H1.trick.next", steps[1])
    emit("F-H1.trick.hundreds", steps[2])
    emit("F-H1.trick.square", answer)

    # --- F-T1: squares ending in 5 ---------------------------------------------
    for number in (35, 105, 135, 5, 55):
        steps, answer = worked("ends_in_five", ("square", number))
        emit(f"F-T1.n{number}.front", steps[0])
        emit(f"F-T1.n{number}.next", steps[1])
        emit(f"F-T1.n{number}.hundreds", steps[2])
        emit(f"F-T1.n{number}.square", answer)
    fronts = list(range(1, 11))
    squares = [(10 * n + 5) ** 2 for n in fronts]
    emit("F-T1.sweep.fronts", join(fronts))
    emit("F-T1.sweep.hundreds", join(sq // 100 for sq in squares))
    emit("F-T1.sweep.squares", join(squares))
    check("every swept square ends in 25", all(sq % 100 == 25 for sq in squares))
    half = F(15, 2) ** 2
    emit("F-T1.half.squared", money(half * 100))
    check("7.5^2 is 7 x 8 and a quarter", half == 7 * 8 + F(1, 4))

    # --- F-M: the one move, by counting cells ----------------------------------------
    for side, strip in ((35, 5), (34, 4), (48, 2), (53, 3), (50, 3), (55, 5), (62, 12)):
        wide, tall, rectangle, overhang, cells = moved(side, strip)
        emit(f"F-M.n{side}.strip_wide", strip)
        emit(f"F-M.n{side}.strip_long", side)
        emit(f"F-M.n{side}.wide", wide)
        emit(f"F-M.n{side}.tall", tall)
        emit(f"F-M.n{side}.rectangle_area", rectangle)
        emit(f"F-M.n{side}.corner", overhang)
        emit(f"F-M.n{side}.square", cells)
        check(f"{side}^2: the move keeps every cell", cells == side * side)
        check(f"{side}^2: the rectangle is {wide} by {tall}", rectangle == wide * tall)
    wide, tall, rectangle, _, _ = moved(35, 5)
    emit("F-M.n35.hundreds_tiles", f"{tall // 10}x{wide // 10}")
    emit("F-M.n35.hundreds", rectangle // 100)

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
    ra, rb = round_to_ten(a), round_to_ten(b)
    emit("F-T2.estimate", join([ra, rb, ra * rb]))
    # 20 x 50 by size: the leading digits multiplied, and the zeros of both
    emit("F-T2.estimate.lead", (ra // 10) * (rb // 10))
    emit("F-T2.estimate.zeros", 2)
    check("two times five is ten, in hundreds", (ra // 10) * (rb // 10) * 10**2 == ra * rb)
    emit("F-T2.held.pieces", sum(1 for p in pieces if p))
    emit("F-T2.bounds", join([a_tens * b_tens, (a_tens + 10) * (b_tens + 10)]))
    check("23 x 46 lies from 20 x 40 up to 30 x 50", a_tens * b_tens <= a * b < (a_tens + 10) * (b_tens + 10))
    emit("F-T2.piece_counts", join([len(running_totals(6, 43)), len(running_totals(a, b))]))

    # --- F-T5: percents ----------------------------------------------------------
    bill = 4800
    emit("F-T5.bill", money(bill))
    emit("F-T5.bill.ten", money(F(10, 100) * bill))
    emit("F-T5.bill.five", money(F(5, 100) * bill))
    emit("F-T5.bill.fifteen", money(F(15, 100) * bill))
    emit("F-T5.bill.twenty", money(F(20, 100) * bill))
    seventeen = [F(10, 100) * bill, F(5, 100) * bill, F(1, 100) * bill, F(1, 100) * bill]
    emit("F-T5.seventeen.pieces", join(money(x) for x in seventeen))
    emit("F-T5.seventeen.total", money(F(17, 100) * bill))
    check("17% is a tenth, half of it, and two hundredths", sum(seventeen) == F(17, 100) * bill)
    # 8% of $25: a grid 25 across and 8 deep, one cell a cent
    grid = [(i, j) for i in range(8) for j in range(25)]
    emit("F-T5.swap.across", len({j for _, j in grid}))
    emit("F-T5.swap.deep", len({i for i, _ in grid}))
    emit("F-T5.swap.cents", len(grid))
    emit("F-T5.swap.one_way", money(F(8, 100) * 2500))
    emit("F-T5.swap.other_way", money(F(25, 100) * 800))
    check("eight cents from each of 25 dollars is 25 cents from each of 8", 25 * 8 == 8 * 25 == len(grid))

    # --- F-T6, F-T7: halving and doubling; round, then fix ---------------------------
    emit("F-T6.five.ten", 86 * 10)
    emit("F-T6.five.product", 5 * 86)
    for name, (x, y) in (("halve", (14, 35)), ("no_easier", (14, 37))):
        emit(f"F-T6.{name}.halved", x // 2)
        emit(f"F-T6.{name}.doubled", y * 2)
        emit(f"F-T6.{name}.product", x * y)
        check(f"{x} x {y} = {x // 2} x {y * 2}", (x // 2) * (y * 2) == x * y)
    emit("F-T6.halve.check", 7 * 70)
    emit("F-T7.price.each", money(1999))
    emit("F-T7.price.round", money(up_to(1999, 100)))
    emit("F-T7.price.rounded", money(4 * up_to(1999, 100)))
    emit("F-T7.price.fix", money(4 * up_to(1999, 100) - 4 * 1999))
    emit("F-T7.price.total", money(4 * 1999))
    emit("F-T7.no_easier.rounded", 7 * up_to(34, 10))
    emit("F-T7.no_easier.fix", 7 * up_to(34, 10) - 7 * 34)
    emit("F-T7.no_easier.product", 7 * 34)
    emit("F-T7.no_easier.by_parts", join(running_totals(7, 34)))
    check("7 x 34: the fix is seven sixes, and parts is 210 then 28", 7 * 40 - 7 * 34 == 7 * 6 and running_totals(7, 34) == [210, 238])

    # --- F-T3: 47 x 53 -----------------------------------------------------------
    x, y = 47, 53
    steps, answer = worked("pair", ("times", x, y))
    middle, distance = int(steps[0]), int(steps[1])
    emit("F-T3.middle", middle)
    emit("F-T3.distance", distance)
    emit("F-T3.middle_squared", steps[2])
    emit("F-T3.distance_squared", steps[3])
    emit("F-T3.product", answer)
    # the rectangle, 47 wide and 53 tall, over the square of 50, counted cell by cell:
    # what stands above the square, and the gap down the square's side
    above = sum(1 for i in range(y) for j in range(x) if i >= middle)
    gap = sum(1 for i in range(middle) for j in range(middle) if j >= x)
    emit("F-T3.strip_area", above)
    emit("F-T3.gap_area", gap)
    emit("F-T3.hole", gap - above)
    check("the strip fills the gap but for a 3 by 3 hole", gap - above == distance**2)
    emit("F-T3.no_middle", (47 + 56) % 2 == 1)
    emit("F-T3.no_middle.sum", 47 + 56)
    emit("F-T3.far.middle", F(12 + 88, 2))
    emit("F-T3.far.distance", F(88 - 12, 2))
    emit("F-T3.far.distance_squared", F(88 - 12, 2) ** 2)
    emit("F-T3.far.product", 12 * 88)
    check("12 x 88 = 50^2 - 38^2", 50**2 - 38**2 == 12 * 88)

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
        # the moved rectangle against the square of 50: how many fifties it is short or over
        wide, tall, rectangle, _, _ = moved(number, abs(d))
        emit(f"F-T4.{name}.fifties", abs(rectangle - 2500) // 50)
        emit(f"F-T4.{name}.fifties_area", abs(rectangle - 2500))
        check(f"{number}^2 = 2500 + 100 x {d} + {corner}", square == 2500 + 100 * d + corner)
        check(f"{number}: one side of the moved rectangle is 50", 50 in (wide, tall))
    for name, number in (("above", 53), ("below", 48)):
        text = str(number * number)
        emit(f"F-T4.{name}.digit_pairs", f"{text[:-2]},{text[-2:]}")
    emit("F-T4.carries.hundreds_after", int(str(62 * 62)[:-2]))
    emit("F-T4.carries.left", int(str(62 * 62)[-2:]))
    twist = str(55 * 55)
    emit("F-T4.twist.digit_pairs", f"{twist[:-2]},{twist[-2:]}")
    emit("F-T4.twist.same_two_numbers", int(twist[:-2]) == 5 * 6 == 25 + 5 and twist[-2:] == "25")

    # --- F-N: the product with nothing to grab -------------------------------------
    a, b = 67, 78
    ra, rb = round_to_ten(a), round_to_ten(b)
    emit("F-N.product", a * b)
    emit("F-N.estimate", join([ra, rb, ra * rb]))
    emit("F-N.slipped_low", str(a * b)[:-1] + "." + str(a * b)[-1])
    emit("F-N.slipped_high", a * b * 10)
    emit("F-N.middle_halves", a + b)
    emit("F-N.has_whole_middle", (a + b) % 2 == 0)
    emit("F-N.ends_in_five", a % 10 == 5 or b % 10 == 5)
    emit("F-N.round_fix.rounded", a * rb)
    emit("F-N.round_fix.fix", a * rb - a * b)
    emit("F-N.round_fix.total", a * b)
    check("67 x 80 - 134", a * rb - 134 == a * b)

    # --- F-D: numbers only docs/description.md states --------------------------------
    wide, tall, rectangle, overhang, cells = moved(97, 3)
    emit("F-D.n97.tall", tall)
    emit("F-D.n97.wide", wide)
    emit("F-D.n97.corner", overhang)
    emit("F-D.n97.square", 97 * 97)
    check("97^2 = 94 x 100 + 9", rectangle + overhang == 97 * 97 == 9409)
    emit("F-D.same_front.hundreds", 23 * 27 // 100)
    emit("F-D.same_front.ones", 23 * 27 % 100)
    emit("F-D.same_front.product", 23 * 27)
    check("23 x 27 is 2 x 3 in hundreds, then 3 x 7", 23 * 27 == 2 * 3 * 100 + 3 * 7)
    half_middle, half_gap = F(47 + 56, 2), F(56 - 47, 2)
    emit("F-D.half_middle.middle_squared", money(half_middle**2 * 100))
    emit("F-D.half_middle.distance_squared", money(half_gap**2 * 100))
    emit("F-D.half_middle.product", 47 * 56)
    check("47 x 56 = 51.5^2 - 4.5^2", half_middle**2 - half_gap**2 == 47 * 56)

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

    # --- F-X: the edges of the always sentences -------------------------------------
    def tally(key, cases):
        cases = list(cases)
        emit(f"{key}.holds", sum(1 for ok in cases if ok))
        emit(f"{key}.checked", len(cases))

    tally("F-X1.ends_in_five", ((10 * n + 5) ** 2 == 100 * n * (n + 1) + 25 for n in range(10_000)))

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
        floor, ceiling = split(a)[0] * split(b)[0], (split(a)[0] + 10) * (split(b)[0] + 10)
        return totals and total == a * b and floor <= a * b < ceiling

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
    emit("F-X1.ends_in_five.largest", max(10 * n + 5 for n in range(10_000)))
    reads = [
        x
        for x in range(101)
        if (x - 50) ** 2 < 100 and f"{25 + (x - 50)}{(x - 50) ** 2:02d}" == str(x * x)
    ]
    emit("F-X4.digit_pairs.first", reads[0])
    emit("F-X4.digit_pairs.last", reads[-1])
    check("the two-digit reading holds on an unbroken run", reads == list(range(reads[0], reads[-1] + 1)))

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
    # Karatsuba's algorithm on the digits of 23 x 46: 2 x 4, 3 x 6, and one product of
    # the two digit sums; the strips are what is left of that product, in tens
    block, corner, sums = 2 * 4, 3 * 6, (2 + 3) * (4 + 6)
    emit("F-C4.karatsuba.multiplications", len((block, corner, sums)))
    emit("F-C4.karatsuba.block", block)
    emit("F-C4.karatsuba.corner", corner)
    emit("F-C4.karatsuba.sums", sums)
    emit("F-C4.karatsuba.strips_in_tens", sums - block - corner)
    emit("F-C4.karatsuba.product", 23 * 46)
    check("the strips come from the product of the digit sums", (sums - block - corner) * 10 == 20 * 6 + 3 * 40)
    check("three small products rebuild 23 x 46", block * 100 + (sums - block - corner) * 10 + corner == 23 * 46)
    check("none of the three products is the problem itself", 23 * 46 not in (block, corner, sums))

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
