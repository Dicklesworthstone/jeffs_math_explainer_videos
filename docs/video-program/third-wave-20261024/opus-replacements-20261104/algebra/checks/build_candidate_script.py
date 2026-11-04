"""Build the candidate script.tsv from the canonical one, by cue id.

Unchanged cues are copied byte for byte; this program asserts that, and that
every recorded cue keeps its exact wording. Usage (from the repository root):

    python3 -I -B <this dir>/build_candidate_script.py CANONICAL OUT

It never writes the canonical file.
"""

import sys

NEW = {
    # ---- 08 subgroups: 08_1..08_3 unchanged (08_1 is recorded) ----
    "08_7": "A missing product is not a dead end. It tells us exactly what to add. Put the three-quarter-turn back, and the rotations close up again. Let's make that repair into a machine. Start with a few moves. Combine every pair, in both orders, and add any result that is new. Repeat until a full pass adds nothing.",
    "08_8": "Feed the machine the horizontal flip alone. Flip, then flip again, and every corner returns. So the only new move is the do-nothing move. The machine stops with two moves, and those two already form a group.",
    "08_9": "Now feed it the flip and the half-turn. One pass produces two new moves, the do-nothing move again and the flip across the vertical midline. A second pass finds nothing new. We have grown a four-move subgroup. Remember it. It comes back in the next chapter for a completely different reason.",
    "08_10": "Feed it the quarter-turn and the flip. The first pass adds four moves, the second adds the last two, and the machine holds the entire rack. Those two moves generate the whole group.",
    "08_11": "Try every possible starting collection, and only ten different groups ever come out. The do-nothing move alone. Five groups of two. Three groups of four. And all eight moves. Look at those sizes. One, two, four and eight. Never three, five, six or seven. Hold on to that pattern. Cosets will explain it.",
    "08_4": "Our machine checks every pair by hand. For a group with millions of elements we want something quicker. Here is a single test for any nonempty collection H inside a group. Take any two members a and b. Undo b, then do a. If that result, written a times b inverse, always lands back in H, then H is a subgroup.",
    "08_5": "Why is one test enough? H is not empty, so it has some member h. Feed the test h twice. Undo h, then do h again, and nothing has changed. So the identity is in H.",
    "08_12": "Next, feed it the identity and h. Undo h, then do nothing. The result is h inverse. So every member of H brings its undo move with it.",
    "08_13": "Finally, take any a and b in H. We now know that b inverse is in H as well, so feed the test a and b inverse. Undoing b inverse is the same as doing b. The result is a times b, so H is closed.",
    "08_6": "That gives closure, the identity and inverses. Associativity comes for free, because we are still using the operation of the big group. And every subgroup passes the test, so the test is exactly right. Now let's meet our four-move subgroup again, through a question about order.",
    # ---- 09 centralizer: 09_1 unchanged ----
    "09_2": "Start with the quarter-turn. The left tile turns, then flips. The right tile flips, then turns. This is the experiment we opened with, and the blue corner finishes in different places. The quarter-turn does not commute with the flip.",
    "09_3": "Now the half-turn. Turn halfway and then flip, or flip and then turn halfway. Every numbered corner finishes in the same place both ways. The half-turn commutes with the flip.",
    "09_4": "Let the scoreboard handle the rest. A move earns a check when the two orders agree at every corner, and a cross when even one corner disagrees.",
    "09_5": "Four checks. The identity, the half-turn, the horizontal flip itself, and the vertical flip. Four crosses. The two quarter-turns and the two diagonal flips. Is there a reason behind this list?",
    "09_9": "Draw the flip's mirror line, the horizontal midline. Now carry that line along with each move. The half-turn spins it end over end, and it lands on itself. A quarter-turn stands it upright. A diagonal flip stands it upright too. The moves with checks are exactly the moves that put the mirror line back where it was.",
    "09_10": "Here is why. Undo a move, flip, and then redo the move. Every point on the carried line ends where it started, so the result is the flip across the carried line. If the line came back to the original mirror, that result is just s, which says the two orders agree. If the line moved, it is a different flip, and the orders disagree.",
    "09_6": "The moves that commute with s form the centralizer of s. In our notation it is e, r squared, s, and r squared s. And look. It is the same four-move subgroup the machine grew in the last chapter from the flip and the half-turn.",
    "09_7": "Is a centralizer always a subgroup, in any group at all? Use the one-step test. Suppose a and b both commute with s. Multiply both sides of b s equals s b by b inverse, on the left and on the right, and b inverse commutes with s too. Now slide s from the right end of a times b inverse to the left end. It passes b inverse, then it passes a. So a times b inverse commutes with s, and the test is passed.",
    "09_8": "Now ask for more. Which moves commute with every move? That set is called the center. A move in the center must put all four mirror lines back where they were. Only the identity and the half-turn do that, and both commute with every rotation as well. The center is what all the centralizers have in common.",
    # ---- 03 additions after the recorded 03_1..03_3 ----
    "03_4": "Why does transitivity matter so much? Gather everything equivalent to one element, and call that collection its class. On the clock, the class of four holds four, sixteen, twenty-eight, minus eight, and every other integer that lands at four.",
    "03_5": "Suppose two classes share even one member, c. Then a matches c, and c matches b. Symmetry lets us read each link in either direction, and transitivity joins them into a link from a to b. Then everything linked to a is linked to b as well, so the two classes are identical. Two classes either coincide completely or do not touch at all.",
    "03_6": "Closeness breaks exactly this. The integers close to one are zero, one and two. The integers close to zero are minus one, zero and one. These groups share two members, yet they are different groups. Without transitivity, groups can overlap without coinciding.",
    "03_7": "So the classes of an equivalence relation cut the whole collection into separate pieces, and every element lies in exactly one piece. Twelve clock lanes hold every integer, and no integer sits in two lanes. It works backwards too. Any way of cutting a collection into pieces gives an equivalence relation, lying in the same piece.",
    "03_8": "Classes also let us build new objects. A fraction is written with a pair of integers, but one half and two quarters are different pairs naming the same number. Call a over b equivalent to c over d when a times d equals b times c. Plot each pair as a point, and every class lies on one straight line through the origin. The steepness of that line is the number itself. A rational number is one whole class.",
    "03_9": "Groups will do the same thing. A subgroup will define an equivalence relation. Its classes will be called cosets. And for the right kind of subgroup, those classes will become the elements of a new group.",
    # ---- 14 cosets: 14_1, 14_2, 14_13, 14_3 and 14_9..14_12 unchanged ----
    "14_4": "Now a group where order matters. In the square group, let H hold just the identity and the flip s. Multiply each member of H on the left by a move g, and you get the set g H, a left coset. For each rotation, the coset holds that rotation, and that rotation done after the flip. In our rack, each left coset is a column.",
    "14_14": "Now multiply on the right instead. H times r holds r and s r. But s r is the other diagonal flip, r cubed s. So the right cosets pair the quarter-turn with a different flip. Both ways cut the eight moves into four pairs, but they are different cuts. When order matters, left and right cosets need not agree.",
    "14_5": "When do two representatives name the same coset? Back on the clock, one and nine name the same coset, because the step from one to nine is eight, a move inside H. One and seven do not. That step is six, and six is not in H. Two representatives name the same coset exactly when the step from one to the other lies in H.",
    "14_15": "In multiplication language, write k as g times some correction h. The correction is g inverse times k. If it lies in H, then k H is g times h H. Multiplying H by one of its own members only reshuffles H, so k H equals g H. Conversely, k lies in its own coset, so if the two cosets agree, the correction lies in H.",
    "14_6": "So call g related to k when the correction g inverse k lies in H. This is an equivalence relation. Every element is related to itself, because the identity is in H. The relation reverses, because H holds inverses. And two corrections chain into one, because H is closed. So the cosets are equivalence classes, and two cosets either coincide or never meet.",
    "14_7": "Every coset is also exactly the size of H. Multiplying by g carries H onto g H, the way the shift carried the triangle around the clock. Two members of H cannot land on the same spot, because g h one equals g h two forces h one to equal h two. So each coset is a perfect copy of H, whether the group is finite or infinite.",
    "14_8": "Now count. A finite group splits into some number k of cosets, each the size of H. So the size of the group is k times the size of H. This is Lagrange's theorem. The size of every subgroup divides the size of the group. The twelve-clock is four cosets of three. And it explains the pattern from the subgroup machine. Three, five, six and seven do not divide eight, so no subgroup of the square can have those sizes.",
    # ---- 16 quotients: originals unchanged; three additions ----
    "16_14": "Here is the same failure with every choice at once. The coset r H has two members, so multiplying r H by r H gives four possible products. Color each product by the coset it lands in. If coset multiplication made sense, all four would share one color. Two products land in r squared H, and two land in H itself.",
    "16_15": "Now look at the whole multiplication table of the square, all sixty-four products. Color each entry blue if it is a rotation and gold if it is a flip. Group the rows and the columns into rotations and flips, and the table falls into four solid blocks. The color of a product depends only on which blocks its two factors came from.",
    "16_16": "Shrink each solid block to a single cell, and a two-by-two table remains. That small table is the quotient group.",
    "16_17": "Try the same thing with the subgroup made of the identity and s. Sort the rows and columns into its four left cosets, and color each product by its left coset. Now half of the blocks are mixed. The block for r H times r H is our four-product failure, sitting inside the table. There is no smaller table to shrink to.",
    # ---- 17 applications: ISBN cues unchanged; Hamming replaces the tile ----
    "17_9": "For a second application, look at a code that repairs an error instead of only noticing it. Hamming's code hides four data bits inside seven. Its sixteen allowed words form a subgroup of all one hundred twenty-eight seven-bit words. The operation flips bits position by position, a rule called exclusive or.",
    "17_15": "A received word with an error lies in some coset of that subgroup. One hundred twenty-eight words, sixteen in each coset. By Lagrange, there are exactly eight cosets. And there are exactly eight situations we hope to repair. No error, or one flipped bit in any of seven positions.",
    "17_16": "Each of those eight situations lands in a different coset. The checks are arranged so that the receiver can compute a three-bit label for the coset, called the syndrome. That label spells the position of the flipped bit in binary. Here the label reads one, zero, one. That is five. Flip bit five back, and the word is repaired.",
    "17_17": "That label is a map onto the quotient group. It forgets which codeword was sent and keeps only the coset. So it has a limit. Flip two bits, and the word lands in the coset of some other single flip. Here two errors produce the label seven, the decoder confidently flips bit seven, and the result is a different codeword.",
    "17_18": "Computer memory that corrects its own errors commonly uses longer codes from this family, with one extra check bit, so that two flips are reported instead of miscorrected. The quotient keeps exactly the information it was designed to keep, and nothing more.",
}

# Film order for the chapters this candidate changes; others keep file order.
ORDER = {
    "08": ["08_1", "08_2", "08_3", "08_7", "08_8", "08_9", "08_10", "08_11", "08_4", "08_5", "08_12", "08_13", "08_6"],
    "09": ["09_1", "09_2", "09_3", "09_4", "09_5", "09_9", "09_10", "09_6", "09_7", "09_8"],
    "03": ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6", "03_7", "03_8", "03_9"],
    "14": ["14_1", "14_2", "14_13", "14_3", "14_4", "14_14", "14_5", "14_15", "14_6", "14_7", "14_8", "14_9", "14_10", "14_11", "14_12"],
    "16": ["16_1", "16_2", "16_3", "16_14", "16_4", "16_5", "16_6", "16_7", "16_8", "16_9", "16_10", "16_15", "16_16", "16_11", "16_17", "16_12", "16_13"],
    "17": ["17_1", "17_2", "17_3", "17_4", "17_5", "17_6", "17_7", "17_8", "17_14", "17_9", "17_15", "17_16", "17_17", "17_18", "17_11", "17_12", "17_13"],
}
REMOVED = {"17_10"}
# Cues with retained Oliver takes (root's current-retained-coverage38.json,
# 2026-11-04T04:36Z, 57 cues; coverage32 had 49, before 08_2, 08_3, 10_1-10_6).
RECORDED = {
    "01_1", "01_2", "01_3", "01_4", "02_1", "02_2", "02_3", "02_4", "03_1", "03_2", "03_3",
    "05_1", "05_2", "05_3", "05_4", "05_5", "05_6", "05_7", "05_8", "06_1", "06_2", "06_3",
    "06_4", "06_5", "06_6", "06_7", "06_8", "06_9", "06_10", "06_11", "07_1", "07_2", "07_3",
    "07_4", "07_5", "07_6", "08_1", "08_2", "08_3", "10_1", "10_2", "10_3", "10_4", "10_5",
    "10_6", "13_1", "13_2", "13_3", "13_4", "13_5", "13_6", "13_7", "13_8", "13_9", "13_10",
    "13_11", "13_12",
}


def main(canonical, out_path):
    raw = open(canonical, encoding="utf-8").read()
    assert raw.endswith("\n")
    lines = raw[:-1].split("\n")
    rows = [line.split("\t", 1) for line in lines]
    text = {cue: words for cue, words in rows}
    assert len(text) == len(rows), "duplicate cue id"
    assert not (RECORDED & set(NEW)), "a recorded cue would change"
    chapters_in_file = []
    for cue, _ in rows:
        chapter = cue.split("_")[0]
        if chapter not in chapters_in_file:
            chapters_in_file.append(chapter)
    merged = dict(text)
    merged.update(NEW)
    for cue in REMOVED:
        del merged[cue]
    out = []
    for chapter in chapters_in_file:
        ids = ORDER.get(chapter, [cue for cue, _ in rows if cue.split("_")[0] == chapter])
        expected = sorted(c for c in merged if c.split("_")[0] == chapter)
        assert sorted(ids) == expected, (chapter, sorted(set(ids) ^ set(expected)))
        out += [f"{cue}\t{merged[cue]}" for cue in ids]
    for cue, words in rows:
        if cue not in NEW and cue not in REMOVED:
            assert f"{cue}\t{words}" in out
    for words in NEW.values():
        assert not any(ch.isdigit() for ch in words), words
        assert not any(ch in "=+*/^<>%$#&@|~_\\{}[]():;—–" for ch in words), words
    open(out_path, "w", encoding="utf-8").write("\n".join(out) + "\n")
    changed = sorted(c for c in NEW if c in text)
    added = sorted(c for c in NEW if c not in text)
    words = lambda s: len(s.split())
    print(f"cues\t{len(rows)} -> {len(out)}")
    print(f"words\t{sum(words(w) for _, w in rows)} -> {sum(words(l.split(chr(9), 1)[1]) for l in out)}")
    print(f"reworded\t{len(changed)}\t{' '.join(changed)}")
    print(f"added\t{len(added)}\t{' '.join(added)}")
    print(f"removed\t{len(REMOVED)}\t{' '.join(sorted(REMOVED))}")
    print(f"recorded_untouched\t{len(RECORDED & set(text))}")
    print(f"longest_new_cue_words\t{max(words(w) for w in NEW.values())}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
