"""Exact arithmetic for the provisional 4:9:26 storyboard.

This independently recomputes displayed examples; it does not prove the
general theorems or accept the script, narration, pictures or delivery.
"""
from fractions import Fraction
import hashlib
import json
from pathlib import Path

POPULATIONS = (4, 9, 26)


def hamilton(house):
    quotas = [Fraction(house * p, sum(POPULATIONS)) for p in POPULATIONS]
    floors = [q.numerator // q.denominator for q in quotas]
    remainders = [q - n for q, n in zip(quotas, floors)]
    extra = house - sum(floors)
    order = sorted(range(3), key=lambda i: (-remainders[i], i))
    if 0 < extra < 3 and remainders[order[extra - 1]] == remainders[order[extra]]:
        raise ValueError('tie at Hamilton cutoff: example needs a rule')
    result = floors[:]
    for i in order[:extra]:
        result[i] += 1
    return quotas, remainders, result


def jefferson(house):
    seats = [0, 0, 0]
    awards = []
    for _ in range(house):
        priorities = [Fraction(p, n + 1) for p, n in zip(POPULATIONS, seats)]
        best = max(priorities)
        winners = [i for i, q in enumerate(priorities) if q == best]
        if len(winners) != 1:
            raise ValueError('winning-priority tie: example needs a rule')
        winner = winners[0]
        awards.append({'region': 'ABC'[winner], 'priority': str(best)})
        seats[winner] += 1
    return seats, awards


def main():
    ham = {n: hamilton(n) for n in (15, 16, 17)}
    assert [ham[n][2] for n in (15, 16, 17)] == [[2, 3, 10], [1, 4, 11], [2, 4, 11]]
    assert ham[15][0] == [Fraction(20, 13), Fraction(45, 13), Fraction(10)]
    assert ham[16][1] == [Fraction(25, 39), Fraction(27, 39), Fraction(26, 39)]
    jeff = {n: jefferson(n) for n in (14, 15, 16, 17)}
    assert [jeff[n][0] for n in (14, 15, 16, 17)] == [[1, 3, 10], [1, 3, 11], [1, 4, 11], [1, 4, 12]]
    assert jeff[17][1][14:16] == [{'region': 'C', 'priority': '26/11'}, {'region': 'B', 'priority': '9/4'}]
    divisor = Fraction(23, 10)
    assert Fraction(9, 4) < divisor < Fraction(26, 11)
    assert [p * divisor.denominator // divisor.numerator for p in POPULATIONS] == [1, 3, 11]
    selected = {15: [2, 3, 10], 16: [2, 4, 10], 17: [2, 4, 11]}
    for n, allocation in selected.items():
        assert sum(allocation) == n
        for actual, quota in zip(allocation, ham[n][0]):
            lower = quota.numerator // quota.denominator
            upper = -(-quota.numerator // quota.denominator)
            assert lower <= actual <= upper
    for a, b in zip(selected[15], selected[16]):
        assert a <= b
    for a, b in zip(selected[16], selected[17]):
        assert a <= b
    script = Path(__file__).with_name('script.tsv')
    report = {'script_sha256': hashlib.sha256(script.read_bytes()).hexdigest(),
              'populations': POPULATIONS,
              'hamilton': {str(n): {'quotas': list(map(str, q)), 'remainders': list(map(str, r)), 'seats': s} for n, (q, r, s) in ham.items()},
              'jefferson': {str(n): {'seats': s, 'awards': awards} for n, (s, awards) in jeff.items()},
              'modified_divisor_15': str(divisor), 'hand_picked_quota_and_monotone_rows': selected,
              'scope': 'Author exact-arithmetic screen of these finite examples only; no independent theorem, pedagogy, listening or visual acceptance.'}
    output = Path(__file__).with_name('facts-oracle01.json')
    with output.open('x') as f:
        json.dump(report, f, indent=2)
        f.write('\n')
    print('Exact Hamilton, Jefferson, divisor and quota examples checked for 15, 16 and 17 seats.')


if __name__ == '__main__':
    main()
