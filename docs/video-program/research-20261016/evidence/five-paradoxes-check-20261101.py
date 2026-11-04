#!/usr/bin/env python3
"""Exact arithmetic checks for the working five-paradoxes narration.

Author checks only: the hotel bijection and assumptions need human review.
"""

from fractions import Fraction
from math import comb


def check_monty():
    # Always pick door 0. Enumerate every legal deterministic host choice;
    # the prize positions, not the (possibly unequal) host cases, are equiprobable.
    wins_by_car = []
    for car in range(3):
        legal = [door for door in range(3) if door != 0 and door != car]
        assert legal
        results = []
        for opened in legal:
            switched = next(door for door in range(3) if door not in (0, opened))
            results.append(switched == car)
        assert len(set(results)) == 1
        wins_by_car.append(results[0])
    assert wins_by_car == [False, True, True]
    assert Fraction(sum(wins_by_car), 3) == Fraction(2, 3)


def check_birthdays():
    distinct_assignments = 1
    for person in range(23):
        distinct_assignments *= 365 - person
    match = 1 - Fraction(distinct_assignments, 365**23)
    assert Fraction(50729, 100000) < match < Fraction(50731, 100000)
    assert comb(23, 2) == 253
    print(f"Birthday match: {float(match):.10f}; pairs: {comb(23, 2)}")


def check_simpson():
    a_easy, b_easy = Fraction(9, 10), Fraction(72, 90)
    a_hard, b_hard = Fraction(18, 90), Fraction(1, 10)
    a_total = Fraction(9 + 18, 10 + 90)
    b_total = Fraction(72 + 1, 90 + 10)
    assert a_easy > b_easy and a_hard > b_hard and a_total < b_total
    assert (a_total, b_total) == (Fraction(27, 100), Fraction(73, 100))


def check_race():
    catch_time = Fraction(100, 10 - 1)
    series_limit = Fraction(10, 1) / (1 - Fraction(1, 10))
    assert catch_time == series_limit == Fraction(100, 9)
    assert 10 * catch_time == 100 + catch_time == Fraction(1000, 9)
    for count in range(1, 10):
        partial = sum(Fraction(10, 10**i) for i in range(count))
        assert partial < catch_time
    print(f"Catch: {catch_time} seconds, at {10 * catch_time} metres")


if __name__ == "__main__":
    check_monty()
    check_birthdays()
    check_simpson()
    check_race()
    print("All exact author checks passed; these are not independent review.")
