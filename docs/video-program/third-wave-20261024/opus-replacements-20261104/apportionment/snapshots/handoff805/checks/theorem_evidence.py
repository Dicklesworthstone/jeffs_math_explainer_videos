"""Supporting evidence (NOT proofs) for three claims the film narrates,
each of which has a written argument in the candidate's docs/description.md:

  1. Balinski and Young's quota method (PNAS 1974, p. 4604: Jefferson's
     priority p/(a+1) among towns with a < p(h+1)/P) keeps every town within
     its share rounded down and up at every house size, whichever way ties
     are broken (the film's "tiles" argument covers any tie rule).
  2. Webster's rule keeps the quota promise for every map of three towns
     (cited to Balinski and Young 1982, Prop. 6.2, via Goelz, Peters and
     Procaccia; the film's half-seat argument), and can fail with four.
  3. Hill's rule gives every town a seat whenever there are at least as
     many seats as towns (its threshold between 0 and 1 is 0).

    python3 -I -B theorem_evidence.py [--seed N] [--maps N]

Random integer maps, exact integer arithmetic. A pass is evidence on the
maps tried; a failure would be a counterexample and is printed.
"""

import random
import sys


def quota_method(pops, house, tie_last):
    """Seats for every house 0..house. Ties go to the lowest index, or the
    highest when tie_last (the proof does not care)."""
    total, n = sum(pops), len(pops)
    seats = [0] * n
    hist = [seats[:]]
    for h in range(1, house + 1):
        best = None
        for i in range(n):
            if seats[i] * total >= h * pops[i]:
                continue  # not eligible: would exceed its share rounded up
            if best is None:
                best = i
                continue
            # compare p_i/(a_i+1) with p_best/(a_best+1)
            lhs, rhs = pops[i] * (seats[best] + 1), pops[best] * (seats[i] + 1)
            if lhs > rhs or (lhs == rhs and tie_last):
                best = i
        assert best is not None, "nobody eligible"
        seats[best] += 1
        hist.append(seats[:])
    return hist


def within_quota(seats, pops, h):
    total = sum(pops)
    for a, p in zip(seats, pops):
        # floor(hp/P) <= a <= ceil(hp/P)
        if a * total > h * p + total - 1 or (a + 1) * total <= h * p:
            return False
    return True


def webster(pops, house):
    """Webster by walking signposts 2p/(2k-1), highest first; None on a tie
    at the cutoff."""
    seats = [0] * len(pops)
    for step in range(1, house + 1):
        pri = [(2 * p, 2 * a + 1) for p, a in zip(pops, seats)]
        best = max(range(len(pops)), key=lambda i: (pri[i][0] / pri[i][1], -i))
        tied = [i for i in range(len(pops)) if pri[i][0] * pri[best][1] == pri[best][0] * pri[i][1]]
        if len(tied) > 1 and step == house:
            return None
        seats[best] += 1
    return seats


def hill_first_seats(pops, house):
    """Hill's signposts squared, p²/(k(k-1)); a first seat (k=1) is
    infinite. True if every town has a seat once house >= towns."""
    seats = [0] * len(pops)
    for _ in range(house):
        def key(i):
            k = seats[i] + 1
            return float("inf") if k == 1 else pops[i] ** 2 / (k * (k - 1))
        seats[max(range(len(pops)), key=key)] += 1
    return min(seats) >= 1


def main(argv):
    seed = int(argv[argv.index("--seed") + 1]) if "--seed" in argv else 20261104
    maps = int(argv[argv.index("--maps") + 1]) if "--maps" in argv else 3000
    rng = random.Random(seed)

    checked = 0
    for _ in range(maps):
        n = rng.randint(2, 9)
        pops = [rng.randint(1, 60_000) for _ in range(n)]
        for tie_last in (False, True):
            hist = quota_method(pops, 120, tie_last)
            for h in range(1, 121):
                if not within_quota(hist[h], pops, h):
                    print("COUNTEREXAMPLE quota method", pops, h, hist[h])
                    return 1
                checked += 1
    print(f"quota method: {maps} random maps of 2 to 9 towns, both tie rules, houses 1..120: "
          f"{checked} apportionments, all within quota")

    # the same with tiny populations, where ties are common
    small = 0
    for _ in range(maps):
        n = rng.randint(2, 6)
        pops = [rng.randint(1, 12) for _ in range(n)]
        for tie_last in (False, True):
            hist = quota_method(pops, 60, tie_last)
            for h in range(1, 61):
                if not within_quota(hist[h], pops, h):
                    print("COUNTEREXAMPLE quota method (small)", pops, h, hist[h])
                    return 1
                small += 1
    print(f"quota method, tie-heavy maps (populations 1..12): {small} apportionments, all within quota")

    three, skipped = 0, 0
    for _ in range(maps):
        pops = [rng.randint(1, 60_000) for _ in range(3)]
        for h in range(1, 81):
            s = webster(pops, h)
            if s is None:
                skipped += 1
                continue
            if not within_quota(s, pops, h):
                print("COUNTEREXAMPLE Webster three towns", pops, h, s)
                return 1
            three += 1
    print(f"Webster, three towns: {three} apportionments within quota ({skipped} cutoff ties skipped)")

    four_fail = None
    for _ in range(maps):
        pops = [rng.randint(1, 60_000) for _ in range(4)]
        for h in range(1, 41):
            s = webster(pops, h)
            if s is not None and not within_quota(s, pops, h):
                four_fail = (pops, h, s)
                break
        if four_fail:
            break
    print(f"Webster, four towns: a random failure {'found: ' + str(four_fail) if four_fail else 'not found in this sample'}")

    hill_ok = all(hill_first_seats([rng.randint(1, 60_000) for _ in range(n)], h)
                  for n in range(2, 8) for h in range(n, n + 12) for _ in range(40))
    print(f"Hill: every town has a seat whenever seats >= towns: {'yes' if hill_ok else 'NO'}")
    return 0 if hill_ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
