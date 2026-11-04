#!/usr/bin/env python3
"""Separate exact-arithmetic oracle for the five chapter models."""
from fractions import Fraction

cars = range(1, 4)
wins = sum(car != 1 for car in cars)
losses = len(cars) - wins
print(f"monty.doors\t{len(cars)}")
print(f"monty.switch_wins\t{wins}")
print(f"monty.switch_losses\t{losses}")

days = 365
no_match = Fraction(1)
people = 0
while 1 - no_match <= Fraction(1, 2):
    no_match *= Fraction(days - people, days)
    people += 1
print(f"birthday.days\t{days}")
print(f"birthday.people\t{people}")
print(f"birthday.second_numerator\t{days - 1}")
print(f"birthday.third_numerator\t{days - 2}")
print(f"birthday.no_match_pct\t{float(no_match * 100):.2f}")
print(f"birthday.match_pct\t{float((1 - no_match) * 100):.2f}")
print(f"birthday.pairs\t{people * (people - 1) // 2}")

for name, passed, total in [
    ("easy.a", 9, 10), ("easy.b", 72, 90),
    ("hard.a", 18, 90), ("hard.b", 1, 10),
    ("total.a", 9 + 18, 10 + 90), ("total.b", 72 + 1, 90 + 10),
]:
    percent = 100 * Fraction(passed, total)
    assert percent.denominator == 1
    print(f"simpson.{name}\t{passed}/{total} ({percent.numerator}%)")
head, achilles, tortoise = 100, 10, 1
for name, value in [("head_start", head), ("pursuer_speed", achilles),
                    ("other_speed", tortoise), ("relative_speed", achilles-tortoise)]:
    print(f"zeno.{name}\t{value}")
stage = Fraction(head, achilles)
for i in range(4):
    print(f"zeno.stage{i}\t{float(stage):g}")
    stage *= Fraction(tortoise, achilles)
catch = Fraction(head, achilles - tortoise)
print(f"zeno.catch_fraction\t{catch.numerator}/{catch.denominator}")
print(f"zeno.catch_decimal\t{float(catch):.2f}")
print("hotel.examples\t4")
print("hotel.new_room\t1")
