"""Independent arithmetic for what the patched chapter 17 shows or depends on.

Run with python3 -I -B. Reads nothing from the repo and imports nothing from RubyRaven's
checks. The Hamming code is built from its parity-check equations, not from the XOR of
positions the Rust uses, so the two can disagree if either is wrong.
"""
from itertools import combinations

DIGITS = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2]
W = [10 - i for i in range(10)]
problems = []


def need(ok, text):
    print(("ok   " if ok else "FAIL ") + text)
    if not ok:
        problems.append(text)


# ISBN walk
products = [w * d for w, d in zip(W, DIGITS)]
running = []
s = 0
for p in products:
    s += p
    running.append(s)
need(products == [0, 27, 0, 42, 24, 0, 24, 3, 10, 2], f"spoken contributions {products}")
need(s == 132 and s % 11 == 0 and s // 11 == 12, "132 = 12 x 11")
print("     running totals", running, "(frame 156's 93 is the total after column 5)")

# Copying error
changed = DIGITS[:]
changed[3] = 7
c = sum(w * d for w, d in zip(W, changed))
need(changed[3] * W[3] == 49 and c == 139 and c - s == 7 and c % 11 == 7, "red 7, red 49, 132 + 7 = 139, remainder 7")

# Row of 7 d mod 11
row = [(7 * d) % 11 for d in range(1, 11)]
need(row == [7, 3, 10, 6, 2, 9, 5, 1, 8, 4] and sorted(row) == list(range(1, 11)), f"7dd mod 11 {row}: a permutation of 1..10")

# Swap, with the change derived from the two totals rather than the factored form
a, b, wi, wj = DIGITS[3], DIGITS[4], W[3], W[4]
before, after = wi * a + wj * b, wi * b + wj * a
need((wi, wj, a, b) == (7, 6, 6, 4), "the swap's weights and digits: x7 over 6, x6 over 4")
need(before == 66 and after == 64 and after - before == -2 == (wi - wj) * (b - a), "66, 64, (7-6)(4-6) = -2")
swapped = DIGITS[:]
swapped[3], swapped[4] = swapped[4], swapped[3]
need(sum(w * d for w, d in zip(W, swapped)) == 130 == s - 2, "the swapped ISBN totals 130")
need(
    all((wa - wb) % 11 and (p - q) % 11 for wa, wb in combinations(range(1, 11), 2) for p, q in combinations(range(11), 2)),
    "17_7 lines: distinct weights and distinct values both differ mod 11, in every pair",
)

# Two errors
d2 = DIGITS[:]
d2[0], d2[1] = 1, 8
t1 = s + 10 * 1
t2 = t1 + 9 * 5
need(t1 == 142 and t2 == 187 == sum(w * d for w, d in zip(W, d2)) and t2 % 11 == 0, "142, 187, remainder 0")
need(45 // 11 == 4 and 45 % 11 == 1 and 10 % 11 + 45 % 11 == 11 and 55 % 11 == 0, "+45 = 4 x 11 + 1; the marker turns 10 + 1 = 11, one full circle")

# Hamming from its three parity checks; a word lists positions 1..7
CHECKS = [(1, 3, 5, 7), (2, 3, 6, 7), (4, 5, 6, 7)]


def bits(n):
    return [(n >> (7 - p)) & 1 for p in range(1, 8)]


def text(n):
    return "".join(map(str, bits(n)))


def label(n):
    w = bits(n)
    return sum(((sum(w[p - 1] for p in chk) % 2) << i) for i, chk in enumerate(CHECKS))


def flip(n, p):
    return n ^ (1 << (7 - p))


code = sorted(n for n in range(128) if label(n) == 0)
need(len(code) == 16 and all(a ^ b in code for a in code for b in code), "16 codewords, closed under XOR")
rows = {}
for n in range(128):
    rows.setdefault(label(n), []).append(n)
need(len(rows) == 8 and all(len(v) == 16 for v in rows.values()), "8 rows of 16: 'each row: a coset of 16 words', 128/16 = 8")
need(all(sorted(a ^ rows[r][0] for a in rows[r]) == code for r in rows), "each row is a coset of the code")

SENT = 0b0110011
need(SENT in code, "sent word 0110011 is a codeword")
one = flip(SENT, 5)
need(text(one) == "0110111" and label(one) == 5, "one flip at 5: label 101")
two = flip(flip(SENT, 2), 5)
need(text(two) == "0010111" and label(two) == 7, "two flips at 2 and 5: 0010111, label 111")
decoded = flip(two, 7)
need(text(decoded) == "0010110" and decoded in code and decoded != SENT, "decoder flips 7: 0010110, a different codeword")
stack2 = [p for p in range(1, 8) if bits(two)[p - 1]]
xor = 0
for p in stack2:
    xor ^= p
need(stack2 == [3, 5, 6, 7] and xor == 7, "second stack: positions 3, 5, 6, 7, XOR 111; they are the ones of the RECEIVED word")
need(
    [p for p in range(1, 8) if bits(decoded)[p - 1]] != stack2,
    "after the miscorrection the word's ones are 3, 5, 6, so 'positions holding a 1' over the stack is false until bit 7 returns",
)
need(
    [p for p in range(1, 8) if bits(SENT)[p - 1]] != [2, 3, 5, 6, 7],
    "same for the first stack once bit 5 is repaired: the word's ones are 2, 3, 6, 7",
)

# F2's parity claims
stored = bin(SENT).count("1") % 2
need(stored == 0, "parity bit of the sent word is 0")
need(bin(two).count("1") == 4 and (bin(two).count("1") + stored) % 2 == 0, "0010111 with parity bit 0: four ones, parity holds")
need(bin(decoded).count("1") == 3 and (bin(decoded).count("1") + stored) % 2 == 1, "0010110 with parity bit 0: three ones, parity BREAKS, so the baseline line was beside the wrong word")

# Every case of the extended code, positions 1..8 with 8 the parity bit
corrected = reported = 0
for c in code:
    par = bin(c).count("1") % 2

    def corrupt(ps):
        w, p = c, par
        for q in ps:
            if q == 8:
                p ^= 1
            else:
                w = flip(w, q)
        return w, p

    for x in range(1, 9):
        w, p = corrupt([x])
        broken = (bin(w).count("1") + p) % 2 == 1
        lab = label(w)
        fixed = w if lab == 0 else flip(w, lab)
        corrected += broken and fixed == c
    for x, y in combinations(range(1, 9), 2):
        w, p = corrupt([x, y])
        reported += (bin(w).count("1") + p) % 2 == 0 and label(w) != 0
need((corrected, reported) == (128, 448), f"extended code: {corrected} single flips corrected, {reported} double flips reported; both captions hold in every case")

# Grid geometry the patch uses
place = lambda n: (label(n), code.index(n ^ (0 if label(n) == 0 else 1 << (7 - label(n)))))
gx = lambda col: -5.2 + 0.62 * col
sent, landed, wrong = place(SENT), place(two), place(decoded)
print(f"     sent {sent}, two-flip word {landed}, decoded {wrong}")
need(landed == (7, wrong[1]), "two-flip word sits in row 7 of the DECODED word's column, so the ring returns straight along that column")
need(abs(gx(sent[1]) - gx(wrong[1])) > 2.0, f"'sent' at x {gx(sent[1]):.2f} and 'decoded' at x {gx(wrong[1]):.2f} cannot touch")

# Cosets and generated subgroup
H = {0, 4, 8}
cosets = {frozenset((g + h) % 12 for h in H) for g in range(12)}
need(len(cosets) == 4 and sorted(map(sorted, cosets)) == [[0, 4, 8], [1, 5, 9], [2, 6, 10], [3, 7, 11]], "four triangles are the cosets of {0,4,8}")
need({(6 * k) % 12 for k in range(12)} == {0, 6}, "6 generates {0, 6}")
need(max(next(k for k in range(1, 5) if (g * k) % 4 == 0) for g in range(4)) == 4, "four-clock has an element of order 4")
need(max((2 if g else 1) for g in range(4)) == 2, "two-clock product: every element has order at most 2")

# Timing sums the patch states
need(abs(5 * 0.1 + 0.2 - 0.7) < 1e-9 and abs(10 * 0.7 - 7.0) < 1e-9, "column 0.7 s, walk 7.0 s")
need(abs(0.35 + 7 * 0.4 - 3.15) < 1e-9, "17_15: 0.35 + 7 x 0.4 = 3.15 s")
print("\nproblems:", problems or "none")
