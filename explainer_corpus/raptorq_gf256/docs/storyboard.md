# Storyboard — The Arithmetic Where 3 × 7 = 9

The hand-edited source of this video's words and pictures (from R1, 2026-10-11;
revised at R2 the same day; it started as plan §10.4, revision 2). `python3 -I
tools/storyboard.py raptorq_gf256` checks it (gate G2) and writes `script.tsv`
from the words column. Facts: `docs/fact-sheet.md` and `oracle.py`; brief:
`docs/brief.md`; reviews and their dispositions: `docs/reviews/raptorq_gf256/`.
Every note below that is routed to "the description" lives in
`docs/description.md` (the draft YouTube description, R2), under its beat id.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds before the
next beat answers it.

## 2 The arc ("each fix exposes the next problem")

| Act | Chapters | Spine |
|---|---|---|
| I. Packets as equations | 01 hook, 02 bits and ex-or, 03 cards, 04 the cube, 05 the ex-or lottery, 06 spares and rings | loops are slow → cards are equations → exactly enough random ex-or cards fail 71%, and the danger lives at the end |
| II. Building a better number system | 07 wrapping at 256, 08 prime clocks, 09 bytes are polynomials, 10 the fold, 11 the byte clock, 12 the times table | scaling needs division; wrapping fails exactly as badly as ex-or; folding by a polynomial that can't be factored works |
| III. Spending it wisely | 13 the byte lottery, 14 the price of density, 15 where the danger lives, 16 parking a piece | byte amounts make a useless card 256× rarer; dense maths is slow; RaptorQ keeps packets ex-or and spends a few byte cards exactly where the danger lives |
| IV. Payoff | 17 two packets | measured failures, the standard's promise, deployments, the closing rhyme |

## 3 Color roles

| Idea | Color |
|---|---|
| pieces A–E | ink-white shape badges: circle A, triangle B, square C, diamond D, hexagon E (R1 creative B-7: no hue that a role uses) |
| packets / cards | warm white cards with the pieces' badges |
| ex-or / GF(2), ex-or cards | cool blue `#58C4DD` |
| the byte field, byte cards | gold `#FFD866` (hero) |
| lost packets | red `#FC6255`, dimmed |
| the lit corners ("what we can already make") | translucent teal |
| the fold polynomial 0x11D | orange `#F0AC5F` |
| wrap-around (Z/256) | desaturated purple `#9A72AC` |

## Vocabulary (one word per idea)

**piece** (a chunk of the file, about a kilobyte; never a packet), **packet**
(what travels; each packet carries one **card**, its equation, from 03 on we say
"card"), **useful** (tells you something you couldn't already work out; never
"independent" or "rank" aloud or on screen), **bit** (one binary digit, from 02
on), **slot** (only for the bucket's places), **loop** (only for the
carousel), **ring** (the five cards in 06 and 16), **ex-or card** and **byte
card**, **wrap around** (said "mod" once), **byte field** (named aloud at 10_6,
GF(256) once, in a lower-third), **the ex-or lottery**, **the danger meter**,
**the byte clock** (each named aloud where it first appears), **result** (a
card's right side, 03_1), **spare** (a card beyond one per piece, 06_1),
**extra pieces** and **free cards** (RaptorQ's added unknowns and its
agreed-in-advance cards whose result is zero, 15_2), **block** (15_6),
**factor** (never "split", 10_3).

## 01_hook: A million phones (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | I'm sending this picture to a million phones. Each one randomly loses about one packet in a hundred. | A tower; a field of about 1,000 phone glyphs (each stands for 1,000), each with its own red loss flecks (flat, grouped one mobject per row, E-B4) | particle field, seeded losses |  |
| 01_2 | Yet almost every phone that catches just two packets more than it needs rebuilds the picture exactly. The secret is a kind of arithmetic where three times seven is nine. | `3 × 7 = 9` in chalk | write |  |
| 01_3 | *(title)* | **The Arithmetic Where 3 × 7 = 9** | title card | listen 3 s (the motif E–G♯–B–C♯) |
| 01_4 | A file travels as thousands of packets, each carrying one small piece of it. Our picture is cut into ten thousand pieces. Lose some, and the usual fix is to ask again. | The picture slices into 10,000 pieces; each piece slides into its own packet | morph |  |
| 01_5 | With a million phones, each packet goes missing for about ten thousand of them, and the requests swamp the sender. A TV antenna can't ask at all. | Request arrows (at most 300, by placement) flood the tower; then "no return channel" | arrow swarm |  |
| 01_6 | So send the file on a loop. Miss a piece, and you wait a whole loop for it. Most phones need two or three loops. The unluckiest of a million need five or six. | A carousel; the phone field turns green in a wave at the end of each loop; after loop 3 a few red stragglers pulse; zoom on the last one, still waiting through loops 5 and 6, with a loop counter | carousel prop |  |
| 01_7 | ⟨predict⟩ Sending random pieces is worse. It's a sticker album. How many random pieces until you've seen all ten thousand? | An album of 10,000 slots | prediction card |  |
| 01_8 | On average, about ninety-eight thousand. The last few take forever, because you keep getting doubles. | Count-up to 97,876 labelled "average"; then three counts side by side, all packets a phone catches: random ≈ 98,000, an average carousel phone ≈ 23,500 (it finishes partway through its last loop), two extra packets 10,002 | count-up | hold |

- **Ruth:** "about ten thousand" assumes 1% loss, each packet independently
  (description: "model: each phone loses each packet independently with
  probability 1%", RB-07); 97,876 = K·H_K on average (RB-08); "two more
  packets" and "almost every" are probabilistic, stated exactly in 17; the
  promise's probability space (random packet IDs, not this phone's mix) is in
  the description under 01_2 and 17_4 (R2 Ruth R2).
- **Facts:** carousel at 1% loss, K = 10,000: mean 2.64 loops; all 10⁶ phones
  done 0.37 by 5 loops, 0.99 by 6 (B-F24). The cold open (01_1–01_3) is about
  20 s (creative B-14). The average carousel phone, 2.64 × 10,000 = 26,400:
  a phone stops partway through its last loop (claim walk). All three counts are packets caught: 97,876 (B-F1),
  23,493 (B-F24.carousel_mean_packets_caught_until_done; 23,730 sent) and 10,002
  (B-F24.fountain_packets_caught_K_plus_2); sent, they would be 98,865, 23,730 and 10,103 (fact sheet, G3 rows; Maya D1).
  "The last few take forever": the last piece alone takes 10,000 draws, the last ten 29,290 (B-F1). After loop 3,
  0.995% of phones still wait, about 10 of the 1,000 glyphs (B-F24).

## 02_bits_and_ex_or (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | First, how computers write numbers. Each spot holds a bit, a zero or a one, and each spot is worth double the one to its right. One, two, four, eight. | Place values under a row of eight bits | binary primer prop |  |
| 02_2 | Five is four plus one, so it's one, zero, one. A byte is eight bits, so it holds zero to two fifty-five. | 101 lights; then 11111111 = 255 | bit row |  |
| 02_3 | Instead of sending pieces, we'll send mixes of pieces, so no phone waits for one particular packet. A piece of the picture is just numbers, so mixing pieces means mixing numbers. The mixing move is called ex-or, and it works like this. | Two pieces merge into one packet card | statement card |  |
| 02_4 | Line up the two numbers in bits. Wherever the second has a one, flip that bit of the first, like a light switch. | Bit rows as light switches; 5 ⊕ 7 | bitwise animation |  |
| 02_5 | ⟨predict⟩ Five is one, zero, one. Seven is one, one, one. What's five ex-or seven? | The two rows lined up | prediction card |  |
| 02_6 | Zero, one, zero. Two. And flip twice, and you're back where you started. Two ex-or seven is five again. | 5 ⊕ 7 = 2; then 2 ⊕ 7 = 5 | bitwise animation | listen 3 s (a byte as a chord: each set bit a note; ex-or toggles notes) |

- **Ruth:** ex-or of several bits is one exactly when an odd number of them are
  one (lower-third).
- **Sound (creative B-1, B-2):** the byte chord is voiced an octave up (E5–G♯6)
  or without F♯, so no two notes sit 15–60 Hz apart; checked with the
  roughness metric on 3, 7, 9, 29 and 255 before the voicing is locked.

## 03_cards: Packets are equations (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Each packet says which pieces it mixed, and the result. That's an equation. We'll draw each packet as a card. | Cards A⊕C = 2, B⊕C⊕D = 6, A⊕B = 6, A⊕B⊕C⊕D = 3, the pieces as shape badges | equation cards |  |
| 03_2 | The same cards as a grid. A row per card, and a dot where it uses a piece. | The grid (rows = cards, columns = pieces) builds beside them, one row per card | grid |  |
| 03_3 | Solving works like elimination in algebra class. Ex-or card one with card three, and the A cancels. B ex-or C is four. | Card 1 ⊕ card 3: A cancels, slowly | TransformMatchingTex |  |
| 03_4 | Put that into card two, and D is two. And so on. | The rest fast-forward; "A? B? C? D?" fill in as 5, 3, 7, 2 | TransformMatchingTex |  |
| 03_5 | The phone is just a bucket. It doesn't care which cards arrive, or in what order. It needs as many useful cards as there are pieces. Useful means it tells you something you couldn't already work out. | A bucket of four slots; it fills one slot per useful card | bucket prop |  |
| 03_6 | ⟨predict⟩ Here's a fifth card. B ex-or C is four. Is it useful? | The card hovers over the full bucket | prediction card |  |
| 03_7 | No. You could make it yourself from cards one and three. | The card bounces off | bounce |  |
| 03_8 | A sender can make endless fresh mixes, like water from a fountain, so these are called fountain codes. Both sides own the same recipe book, so a packet's number tells the phone which pieces it mixed. | Fountain icon; "packet #48213 → pieces {…}" | statement card |  |
| 03_9 | The fountain code we'll open up is called RaptorQ, an internet standard since twenty eleven. This video is about the arithmetic inside it. | **RaptorQ** spelled once; beneath it "RFC 6330 · IETF Proposed Standard · August 2011" (R2 Ruth R15, Dev dev-08) | name card | hold |

- **Lower-thirds:** "each piece is really a chunk, about a kilobyte; every
  operation runs across all its bytes at once"; "lost, not corrupted:
  checksums throw bad packets away; one that slipped through would poison the
  decode" (D23 keeps both); 03_8: "the recipe book is written into the
  standard (RFC 6330's packet number, the ESI, is the seed); the block size
  travels in a header" (D10).
- **For programmers:** RAID-5 parity is one of these cards.

## 04_the_cube (~45 s; 04_2–04_5 on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Use three pieces for a moment. A card either uses a piece or it doesn't, so there are eight possible cards, from nothing at all to A, B and C together. | The 8 patterns laid out where the cube's corners will fall; hold 1.5 s | layout |  |
| 04_2 | *(camera)* Put them on the corners of a cube. One direction per piece. Step along A if the card uses A. | The layout lifts into 3D; corners labelled; "nothing" already lit | **3D cube, orbit < π (S-5)** |  |
| 04_3 | With one useful card, A, you can make two things, nothing or A. Add B, and you can make four. Each useful card doubles what you can make. | The lit corners grow 1 → 2 → 4: the bottom face (cards A and B) | lit corners |  |
| 04_4 | ⟨predict⟩ Two cards in hand. The next card is random. Each piece goes in or stays out by a coin flip. What's the chance it's useless? | Random arrows to the corners | Monte Carlo arrows |  |
| 04_5 | Four lit corners out of eight. Half. | Half the arrows land on lit corners; the camera segment ends here (E-B3) | lit corners |  |

## 05_the_ex_or_lottery (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | ⟨predict⟩ So for three random cards in a row, how often are all three useful? | The cube's three steps as fractions to fill: ?/8 · ?/8 · ?/8 | prediction card |  |
| 05_2 | Seven eighths, times six eighths, times four eighths. About one time in three, so it fails two times in three. With many pieces it fails seventy-one percent of the time. Call it the ex-or lottery. | Retained route: 7/8 · 6/8 · 4/8 = 0.328 → failure 0.672 → 0.7112, labelled **model** (R2 Ruth R9) | live product |  |
| 05_3 | Watch this danger meter as the bucket fills. For almost the whole fill, a random card is nearly certain to help. Almost all the danger is in the last few slots. | **The danger meter**: the 03 bucket's rim shows the chance the next card is useless: flat near zero, then rising | danger meter prop |  |
| 05_4 | Three slots from the end, a random card is useless one time in eight. Then one in four. At the last slot, one time in two. Ten pieces or ten million, only the last few slots matter, so the lottery fails about seventy-one percent either way. | The rim reads 1/8, 1/4, 1/2 | danger meter prop | listen 3 s (the rank chord: each useful card adds a tone, each useless one is a muted thud; the thuds crowd in at the end) |

- **Ruth:** P(K uniform 0/1 cards all useful) = ∏(1 − 2⁻ⁱ), i = 1…K (lower-third
  at 05_2, RB-05); 0.328 at K = 3, → 0.288788; almost all, not all, the danger
  sits at the end: outside the last 3 slots 12.0%, last 5 3.1%, last 8 0.39%
  (RB-11). The example cards are A and B, so the lit set is a face of the cube
  (the span of A⊕B, B⊕C, A⊕C is not a plane; RB-12, creative B-10).

## 06_spares_and_rings (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ Spares help. With two spare cards, beyond one per piece, how often does the lottery still fail? | The risk ladder: a vertical log scale from 1 down to one in a million, labelled **model** (R2 Ruth R9) | prediction card |  |
| 06_2 | Nearly one time in four. The first spare takes it from seventy-one to forty-two percent, and each one after that roughly halves it. With a million phones, that's almost a quarter of a million stuck. | Each ex-or spare steps the risk down one rung: 71 / 42 / 23 / 12% (**model**) | risk ladder |  |
| 06_3 | Reaching one in a million takes about twenty spares. Ten halvings make about a thousand. Twenty make about a million. | Twenty rungs (**model**) | risk ladder |  |
| 06_4 | And sometimes it's structure, not luck. Five cards in a ring, each the ex-or of two neighbours. Every letter appears twice, so together they cancel to nothing. The five are worth only four. | A ring of five cards; letters pair up and vanish | ring prop |  |
| 06_5 | So far a card can only use a piece or leave it out. Then a card at the last slot is useless half the time, and the ring stays stuck. What if a card could scale a piece, three times A, or a hundred times A? With any byte as the amount, two hundred fifty-six choices, and the right arithmetic, a card would hardly ever be useless. | The amounts {0, 1} beside the last-slot meter and the stuck ring; then {0, 1} grow to 256 dots, one per byte | statement card |  |
| 06_6 | Then a byte times a byte must give a byte, and we must be able to undo it. | "byte × byte → byte, and undo it" | statement card | hold |

- **Ruth:** the factors are 1.68, 1.84, 1.92, 1.96 (71.1 → 42.2 → 23.0 → 12.0 →
  6.1%): halving is the limit (RB-13). Any four of the ring's five cards are
  useful together; all five are worth four (RB-14). Average spares: Σ 1/(2ⁱ − 1)
  = 1.6067, the Erdős–Borwein constant, which Erdős proved irrational in 1948
  (on screen in 13; RB-D2, RB-32).
- **Scaling, not counting (RB-03):** "three times A" is a byte scalar, never
  "A plus A plus A" (in the byte field A ⊕ A ⊕ A = A).

## 07_why_not_wrap_around_at_256 (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Bytes could wrap around at two fifty-six, like a clock. Mathematicians call that mod. But to undo times three, we must be able to divide. | The 256-hour clock | clock prop |  |
| 07_2 | ⟨predict⟩ Try two times one twenty-eight on this clock. What do you get? | The bit row of 128 | prediction card |  |
| 07_3 | The top bit slides off the end, and it's gone. Zero. | Bits slide left; the top bit drops into a dark gutter; `2·128 ≡ 0` | **fallen-bit animation** | listen 3 s (the byte's one note slides up off the top and leaves silence) |
| 07_4 | So a card saying two times A is six can't tell you A. It could be three, or one thirty-one. | 2·3 = 6 and 2·131 = 262 → 6 | statement card |  |
| 07_5 | Even amounts lose information, like two did. Odd ones never do. And a number is odd exactly when its last bit is one. So with random byte amounts, only the last bit matters. | Each card's top seven bits fade to grey, leaving the last bit lit | bit fade |  |
| 07_6 | That's the coin flip from ex-or again. This lottery fails seventy-one percent of the time, just like ex-or. | The race: ex-or on the left in blue, wrap-around on the right in purple, same layout as every later race (creative B-18); both counters converge to 71% | **lottery race** |  |

## 08_prime_clocks (~50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ A prime number of hours fixes it. On a seven-hour clock, what times three is one? | The 7-hour clock; `? × 3 = 1` | prediction card |  |
| 08_2 | Five. Every row of its times table, except zero's, is a reshuffle of all seven values. So every number but zero has a partner that multiplies it back to one, and that lets you divide. | The 7-hour table; each row lights as a reshuffle | small table |  |
| 08_3 | Zero sneaks in at two fifty-six because two fifty-six is two times one twenty-eight. The nearest prime is two fifty-seven, but that clock needs an hour two fifty-six, which doesn't fit in a byte. | 256 = 2 × 128 beside the 2·128 = 0 card; then 256 is written into an 8-bit box and its ninth bit spills out, lit red | factor split, byte box |  |
| 08_4 | A number system where you can add, subtract, multiply, and divide by anything but zero, with all the usual rules of algebra, is called a field. We need one with exactly two hundred fifty-six members, one for every byte. So catch the falling bit, and fold it back in. | Name card "field"; the gutter bit lifts back | statement card | hold |

- **Precisely (lower-thirds):** 07_6 "a K×K matrix over Z/256 is invertible ⇔
  its determinant is odd ⇔ it is invertible mod 2; uniform entries reduce to
  uniform bits, so success = ∏(1 − 2⁻ⁱ)" (B-F25; RB-D1); 08_4 "finite fields
  exist exactly for prime-power sizes".
- **For programmers (07_6):** "3 × 171 = 513 = 2·256 + 1: multiplying by an odd
  constant is reversible; it's why hash mixers use odd multipliers" (D13).
- **Cut (R1 creative B-11):** the 12-hour clock and the 16-vs-7 heat maps; the
  byte table in 12 does that job.

## 09_bytes_are_polynomials (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | So we need a field with two hundred fifty-six members. Whole numbers can't do it, but polynomials can. | Goal card | statement card |  |
| 09_2 | Remember the place values, one, two, four, and eight? Swap the two for an x. One, x, x squared, x cubed. Three was two plus one, so now it's x plus one. | 10000000 → x⁷; 00000011 → x + 1; the place values beneath morph into powers of x (bits typeset in TeX, E-B5) | bits → Tex morph |  |
| 09_3 | The numbers in front are only zero or one, and one plus one is zero, like flipping a switch twice. So x plus x is zero, and adding is ex-or again. | `x + x = 0`; column-wise ex-or | callback to 02 |  |
| 09_4 | ⟨predict⟩ Three is x plus one, and seven is x squared plus x plus one. Multiply them out, remembering that one plus one is zero. What do you get? | `(x + 1)(x² + x + 1)`; a "pause and try it" card | prediction card | hold |
| 09_5 | Six terms, and two pairs match. x squared plus x squared vanishes, and so does x plus x. What's left is x cubed plus one, which is one, zero, zero, one. Nine. | FOIL one term at a time: x³ + x² + x + x² + x + 1; the matching pairs (never written "2x²") vanish; x³ + 1 = 1001 = 9 | **FOIL with vanishing terms** |  |
| 09_6 | Three times seven is nine. | Chalk `3 × 7 = 9` (callback to 01) | write | listen 3 s (the chords of 3, then 7, then 9) |

- **Ruth:** 3 · 7 = 9 holds under every one of the 30 degree-8 fold
  polynomials, because the product has degree 3 and no fold happens (the brief's
  title row is corrected to say so, RB-31). The FOIL shows x² + x² and x + x,
  not 2x² (an integer 2 that is zero here, beside the byte 2 that is not;
  RB-04).
- **Maya:** the prediction card says "pause and try it": the 09_4 hold gives
  the viewer about 5 s with the hold (M27).

## 10_the_fold (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A product past x to the seventh doesn't fit in a byte. The simplest fold drops the fallen bit back in at the bottom. A rotation. | Rotation fold: the top bit wraps to bit 0 | fold animation |  |
| 10_2 | ⟨predict⟩ Three times a byte is double it, ex-or itself. Under that fold, what's three times two fifty-five? | `3 · 255`; the hint card `3·a = 2·a ⊕ a` | prediction card |  |
| 10_3 | Zero. The rotation turns x to the eighth into one, so x to the eighth plus one is zero. And it factors into x plus one, which is three, times a long polynomial, two fifty-five. Two fifty-six was two times one twenty-eight all over again. | First the hint worked: 11111111 rotates to itself, `2·255 = 255`, and `255 ⊕ 255 = 0`; then "x⁸ → 1", so `x⁸ + 1 → 0`; `(x + 1)(x⁷ + … + 1) = x⁸ + 1` with 3 and 255 under the factors; `3·255 = 0` | fold animation |  |
| 10_4 | So we need a polynomial that can't be factored, a prime polynomial. RaptorQ's is x to the eighth plus x to the fourth plus x cubed plus x squared plus one. Treat it as zero, and an x to the eighth that falls off becomes the rest of it. | "a prime polynomial: it can't be factored" over the whole `x⁸ + x⁴ + x³ + x² + 1` (= 285, 0x11D beside it), which stays up through 10_6 (Maya R3 maya-02); at "Treat it as zero" the rule `x⁸ → x⁴ + x³ + x² + 1` under it, its right side 29 (0x1D) | the fold |  |
| 10_5 | Two times one twenty-eight is x to the eighth. Fold it, and you get sixteen plus eight plus four plus one. Twenty-nine. Not zero. | The same bit row, gutter and scale as 07_3: the fallen bit lands back as 00011101 with place values | **overflow-fold animation** | listen 3 s (the same slide as 07_3, now blooming into the 0x1D chord E–G♯–B–C♯) |
| 10_6 | And every nonzero byte now has a partner. Two times one forty-two is exactly one. So bytes form a field, the byte field. | 142 × 2 folded step by step; everything cancels except 1; then the name card "the byte field", lower-third "GF(256), also written GF(2⁸)" | fold |  |

- **Code panel (for programmers), at 12_5 (R3: Dev D1 wants it readable and
  spoken, Maya maya-14 wants it out of 10_5–10_6, and chapter 10 has no time
  for a beat of its own):** centred and large while 12_5 says what it does,
  then in the corner, where `0x1D` swaps to `0x1B`, `0x31` flips to `0xC1` and the comment `mul(2, 142) == 1` flips to `mul(2, 141) == 1` (AES's inverse of 2; claim walk) (D6, D20).

```rust
fn xtime(a: u8) -> u8 { (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 } }
fn mul(mut a: u8, mut b: u8) -> u8 {      // shift-and-add, no tables
    let mut acc = 0;
    while b != 0 { if b & 1 != 0 { acc ^= a } a = xtime(a); b >>= 1 }
    acc                                   // mul(0x57, 0x83) == 0x31, mul(2, 142) == 1
}
```

- **Lower-thirds:** "written a CRC? Same move: shift, and if a bit falls off,
  ex-or in the polynomial. CRC-8/SAE-J1850 uses this very 0x1D" (D20); "p can't
  be factored ⇒ no two nonzero bytes multiply to 0 ⇒ each row of the table has
  no repeats ⇒ it contains 1" (RB-21); the labels lower-third ("GF(2⁸) ≅
  GF(2)[x]/(p) for any irreducible p; all fields of order 256 are isomorphic;
  the polynomial is a choice of labels") moved with the AES comparison to 12_5.
- **Maya:** decimals first, hex beneath for programmers (M11).

## 11_the_byte_clock (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | ⟨predict⟩ Start at one and keep multiplying by two. One, two, four, eight, sixteen, thirty-two, sixty-four, one twenty-eight. What comes next? | The times-two sequence on byte-ordered points | prediction card |  |
| 11_2 | Twenty-nine. Keep going, and you visit every nonzero byte exactly once before coming back. Two hundred fifty-five steps. | In byte order the times-two path is a tangled 255-point star; only the named nodes are labelled (E-B6); lower-third 11_2–11_3 (RB-19, R2 Dev dev-13): "under AES's polynomial, times two comes back to 1 after 51 steps (its hand is 3); every finite field has such a clock" | **255-point star** |  |
| 11_3 | Now move every nonzero byte to the step where it was reached. The star untangles into a circle. | Bytes slide to their log positions; zero sits at the centre, off the clock | **star → circle untangling** |  |
| 11_4 | Times two is one click around. A clock with two hundred fifty-five hours. Call it the byte clock. | The clock face, 255 hours; lower-third 11_4 (for programmers, D12): "Linux RAID-6 gives drive i the weight 2ⁱ; with AES's polynomial only 51 of those weights differ" | byte clock | listen 3 s (a gold dot laps the clock with a 255-step glide that climbs one octave and lands home) |
| 11_5 | To multiply, add clicks, just as two cubed times two to the fourth is two to the seventh. One twenty-eight times four. Seven clicks plus two is nine clicks. Fifty-eight. | 2⁷·2² = 2⁹ = 58, read off the clock; lower-third 11_5–11_6 (RB-D5, on screen only): "7 × 7: 198 + 198 = 396 → 141 clicks → 21; by hand (x²+x+1)² = x⁴+x²+1 = 21" | slide-rule rings |  |
| 11_6 | One fifty times three? One fifty is a hundred eighty clicks, and three is twenty-five clicks. Together, two hundred five clicks, which is one sixty-seven. That click count is called a logarithm. One fifty times itself goes past the top. Three hundred sixty clicks, less one full turn of two fifty-five, is a hundred five clicks. Twenty-six. | Two concentric rings; the inner ring rotates 25 clicks (labels counter-rotate); "log 150 = 180" with "in the byte field (base 2)" under it, not log₂ (Ruth R3-10); then 150 × 150: the hand goes 180 more clicks, past the top, to 105; `180 + 180 = 360`, `360 − 255 = 105 clicks → 26` (Dev R3 D4) | rings |  |
| 11_7 | And every nonzero byte's partner sits at its mirror image across the top. | The 127 mirror pairs drawn across the circle as string art (log a⁻¹ = 255 − log a); 1 at the top is its own partner; zero stays at the centre with none | string art |  |

## 12_the_times_table (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_0 | That's how the standard multiplies. Look up two click counts, add them, and look up the answer. In code, zero has no clicks, so check it first. And two click counts can add up past two fifty-five, so the answer table is twice as long. | The byte clock, small at the left: two click counts added round it; then the lookup code panel (for programmers) alone, centred and large (Dev R3 D1), its zero guard and EXP's length lit as they are said, the u8-wrap count in its comments (Maya R3 maya-14) | lookup |  |
| 12_1 | Every pair at once. Here's the whole times table in byte order. Each square's colour is its answer. And it looks like noise. | The 256×256 table (per-point stroke strips, spike S-3) in byte order; row and column 0 set aside, dimmed | **table** |  |
| 12_2 | ⟨predict⟩ Sort the rows and the columns by clicks. What will the table look like? | The table, row and column 0 set aside, dimmed | prediction card |  |
| 12_3 | *(no narration: the sort)* | Rows sort by clicks (3 s), then columns (3 s); a granular texture of the rows' chords settles into one held E chord as the stripes lock | **table reorder (S-3)** | listen 6 s |
| 12_4 | Sorted by clicks, it's perfect diagonal stripes. Each stripe holds one total of clicks. | The stripes; lower-third 12_4 (RB-20): "orders 1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, …, 128 times; 255 = (2+1)(2²+1)(2⁴+1)" | stripes | hold |
| 12_5 | Here's the whole multiply as code. Keep doubling with the fold, and ex-or together the doublings that the other byte's bits pick. A famous encryption standard picked a different prime polynomial. Then the same two bytes can multiply to different answers, so both ends must agree. | The shift-and-add code panel alone, centred and large (Dev R3 D1), `xtime`'s fold byte and the loop lit as they are said; then it shrinks to the corner. Side-by-side: 87 · 131 = 49 here, 193 in AES (hex beneath: 0x57 · 0x83 = 0x31 vs 0xC1); the code panel flips; lower-third: every prime polynomial of degree 8 builds the same field, the polynomial a choice of labels | side-by-side |  |
| 12_6 | The wrong clock counted bytes, and multiplying on it broke. The right one counts clicks, two hundred fifty-five of them, and going around it is multiplying. | The two clocks side by side: "bytes" under the 256-hour clock, "clicks" under the 255-hour one | rhyme | hold |

- **Lower-thirds, each with its beat (R2 Ruth R19; each is also in the
  description under its beat, in full):** 11_2–11_3 RB-19 (0x11B: ord 2
  = 51, ord 3 = 255; Dev dev-13 puts it where the claim is spoken); 11_4 D12
  (RAID-6); 11_5–11_6 RB-D5 (7 × 7); 12_4 RB-20, shortened to fit the hold
  (the full text, "each count is a power of 2 because 255 = (2+1)(2²+1)(2⁴+1);
  together they make 11111111₂ = 255", is in the description).
- **Moved (Codex review, lane B-2 round 1, P2):** with leo's narration 10
  ran 91.1 s and 11 ran 92.2 s. 12_0 was 11_7's first two sentences, 12_5
  was 10_7 and 12_6 was 11_8, word for word, so their takes are reused; 10
  now ends on the byte field, 11 on the mirror pairs, and act II on the
  rhyme. The table lookup leads into the whole table; AES's different labels
  for the same field follow its stripes.
- **For programmers (code panel at 12_0, R2 Dev dev-03: the types are part of
  the lesson; a `u8` LOG wraps at 256 for 32,131 of the 65,025 nonzero pairs,
  chapter 07's bug, which panics in debug builds and silently gives a wrong
  product in release builds):**

```rust
static EXP: [u8; 510] = [/* 2^0, 2^1, ..., 2^509 */]; // largest index 254 + 254 = 508: no % 255
static LOG: [u16; 256] = [/* LOG[0] is never read */]; // u16: a u8 sum wraps at 256 (chapter 7's bug)
fn mul(a: u8, b: u8) -> u8 {                            // the tables are elided here
    if a == 0 || b == 0 { 0 } else { EXP[(LOG[a as usize] + LOG[b as usize]) as usize] }
}
```
- **Engine:** the 4K table segment renders alone (≈ 83 CPU-minutes per 12.5 s,
  S-3). The byte-order table shows structure in its corner and centre cross, so
  the words say "looks like noise", not "is noise".

## 13_the_byte_lottery (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Now give every card random byte amounts. This design really exists. RaptorQ does something cheaper, but first see why it's so good. | A gold byte card; lower-third "random linear network coding" (M34) | card |  |
| 13_2 | Remember the last slot. Half the corners were lit, so a random ex-or card was useless half the time. | The danger meter returns at the last slot, blue | danger meter |  |
| 13_3 | ⟨predict⟩ With random byte amounts, how often is a card useless at the last slot? | The meter turns gold | prediction card |  |
| 13_4 | Only one time in two fifty-six. Fix every amount but one. Scaling by a nonzero byte reshuffles all two fifty-six values, so exactly one choice of that amount lands in the lit part. | The same gold danger meter at the last slot (R2 Maya M5, no ghost): the last card's 256 possible values of one amount spread as 256 dots; 255 land outside the teal lit part, 1 lands in it; lower-third (Ruth D1): "precisely: fix the other amounts; for a piece whose own card isn't lit, exactly one of its 256 amounts puts the card in the lit part" | dots on the meter |  |
| 13_5 | So every extra byte card divides the risk by two fifty-six. With ex-or, each extra card only roughly halves it. | The risk ladder: each byte spare drops eight rungs; labelled **model**: 0.39% / 0.0015% / 0.000006% | risk ladder |  |
| 13_6 | Let's race them on ten pieces. Keep adding random cards until every piece is pinned down, a thousand times over. Ex-or needs one or two spares on average. The byte cards rarely need even one. | The race (blue left, gold right): histograms of spares needed with this run's own counts; "exact, ten pieces: 1.6057 / 0.0039" (R2 Ruth R7, Dev dev-11); lower-third "with many pieces, 1 + 1/3 + 1/7 + 1/15 + … = 1.6067…; with ten pieces the sum stops at 1/1023: 1.6057. Erdős proved the full sum irrational in 1948" | **Monte Carlo race** |  |
| 13_7 | One subtlety. A card whose amounts are only zeros and ones is still an ex-or card, even in the byte field. All the gain comes from the other two fifty-four amounts. | The same 0/1 card in both fields: "same number of useful cards" | statement card | hold |

- **Precisely (lower-third at 13_5):** "union bound: P(fail) < 256^−h/255.
  Exact: 3.9215×10⁻³, 1.5319×10⁻⁵, 5.984×10⁻⁸ at h = 0, 1, 2 (the bound 1/255
  is 3.9216×10⁻³)" (RB-22).
- **Facts:** the race is pinned in the oracle (D15, RB-23): B-F6.race_K10_seeded.*, CPython's
  `random.Random(1306)`, 1,000 races per side; ex-or spares 0, 1, 2, … occur 297, 303, 171, 94, 65, 39, 17, 4, 7, 0,
  2, 0, 0, 0, 0, 1 times (mean 1.603), byte spares 995, 5 (mean 0.005). The scene shows these counts as pinned
  constants (as for B-F18). The exact 1.6057 / 0.0039 are B-F6.mean_spares_gf2_K10 / _gf256_K10; 1.6067 is
  B-F6.mean_spares_gf2; 1023 is B-F6.last_term_denominator_K10.
- **R2 keys:** 13_4's one-amount rule is checked exhaustively at two pieces
  (B-F7.last_slot_exactly_one_amount_lit_exhaustive_K2_ok, all 257 lit lines);
  13_7's "still an ex-or card" by B-F9.rank_01_matrices_same_over_gf2_and_gf256_300_random_ok.
  The ghost copy of the file (creative B-13) is gone from 13_4; 16_1's parked
  "c" is the only ghost in the video.

## 14_the_price_of_density (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | So use random byte amounts everywhere? Solving cards that each use every piece takes about the number of pieces cubed, over three. For a file of fifty thousand pieces, that's about forty-two trillion steps. At a billion a second, half a day. | K³/3 count-up: ≈ 4.2×10¹³; "≈ 11.6 h at 10⁹/s"; lower-third (for programmers, R2 Dev dev-09): "even ten times faster, still ≈ 1.2 h; the matrix alone is 2.5 GB; double the pieces, eight times the work" | count-up |  |
| 14_2 | Fast codes keep cards sparse. Each card mixes only a few pieces. About half mix two, and a few mix many. | Each card morphs into a dot joined to its pieces: the graph "pieces / packets"; degree histogram; lower-third (R2 Ruth R17): "RFC 6330's degrees: P(d) ≈ 1/(d(d−1)) for d = 2…29, the ideal soliton; mean 4.82" | graph prop |  |
| 14_3 | Then decoding is a chain reaction, like Sudoku. A card with one unknown left gives it away. Remove that piece everywhere, and new single-unknown cards appear. This is called peeling. | Peeling cascade on the five-piece toy: C, A, B, D, E; the ripple glows | **peeling cascade** | listen 3 s (each peeled piece a pluck stepping down the E pentatonic scale) |
| 14_4 | ⟨predict⟩ Lose the card that starts it. What happens? | The starting card fades | prediction card |  |
| 14_5 | Nothing moves. | The graph freezes; the music stops for 1.5 s | stall |  |
| 14_6 | With exactly enough sparse cards, peeling usually gets stuck early. Even with two percent extra, most runs get stuck around ninety-eight percent. That sounds great, but over a hundred pieces are missing, and the picture is broken. | Progress bars stalling; caption "10,000 pieces, the standard's degree table, 5 seeded runs; one run with 2% extra stuck at 16%" | stall bars |  |
| 14_7 | On average, about seventy of them aren't in any card. No decoder can find those. | Those pieces outlined red; caption "2% extra: on average ≈ 70 of the 10,000 pieces are in no card" | highlight |  |

- **For programmers (lower-third at 14_3):** "per piece, a list of the cards
  that use it; plus a queue of cards with one unknown left (the ripple)" (D19).
- **Ruth:** peeling is not elimination (it can fail where elimination
  succeeds); the standard's degree table is the ideal soliton 1/(d(d−1)) for
  d = 2…29 quantized to 2⁻²⁰, mean 4.817 (on screen at 14_2 since R2); a plain
  sparse code with it leaves e^(−4.82) ≈ 0.81% of pieces untouched at N = K
  (D4: one in 124), and at 2% extra (N = 10,200) on average 73.4 of 10,000,
  one in 136 (B-F17; R2 Ruth R6, Dev dev-04: 14_7 now speaks the 2% case it
  follows). "Perfect code" is a technical term for other codes; Reed–Solomon
  codes are MDS (RB-06, D7): any K of the N = 257 columns (1, a, a²), (0, 0, 1)
  rebuild (B-X8, 3,000 seeded triples; and the 257 directions of GF(256)²
  make 257 the most columns at K = 2).

## 15_where_the_danger_lives (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | RaptorQ does something cheaper. Its packets never use byte amounts. Every one is a plain ex-or card, of about seven pieces on average. | Blue ex-or cards, each with its handful of badges; "≈ 7 pieces per packet (average)" | cards |  |
| 15_2 | Before sending, the sender adds some extra pieces, picked so that certain fixed cards all have the result zero. These free cards are never sent, because both sides know them. And every piece is in some free card, so even a piece that no packet mixed can still be worked out. | The grid for ten pieces, 27 × 27: columns "10 pieces + 17 extra pieces", rows "7 free ex-or cards + 10 free byte cards + 10 packets" (R2 Maya M22), each free card's result "= 0"; a column no packet row uses, outlined red as in 14_7, lights in a free-card row | matrix schematic |  |
| 15_6 | Big files are cut into blocks, each solved on its own. In every block, only ten to sixteen free cards are byte cards. In big blocks, hundreds of others are cheap ex-ors. | A file splits into blocks; then a schematic big block (not a zoom): "10,017 pieces + 252 extra pieces; 241 free ex-or cards + 11 free byte cards" | matrix schematic |  |
| 15_3 | Remember the danger meter? Now the bucket has a slot for every piece, old and new, and it still doesn't care about order. Count the ex-or cards first, and they fill it up to the last ten to sixteen slots, before the danger starts. The byte cards fill those, where a card is useless only one time in two fifty-six. | One bucket, its block size on its label (R2 Ruth R16): the big block, 10,269 slots, blue water fills 10,258, the top 11 gold; an inset for ten pieces, 27 slots, 17 blue and 10 gold | danger meter callback |  |
| 15_4 | ⟨predict⟩ What if those last cards were plain ex-or cards? | The gold layer turns blue | prediction card |  |
| 15_5 | The ex-or lottery again. About seventy-one percent failure. With the byte cards, about six in a thousand. | Ablation counters on the bucket: 70.7% vs 0.646%; caption "10 pieces · 10,000 / 100,000 trials · our RFC 6330 implementation" (R2 Ruth R9) | **ablation counters** | listen 3 s (the rank chord: the ex-or-only run ends in thuds and an unfinished chord; the byte run completes it) |

- **R2 rewrite (Ruth R1, Maya M2, M8, M13, M22, Dev dev-01, dev-02):** as
  spoken at R1, "extra cards that tie the pieces together" with free answers
  could not exist: a card's result depends on the file, and K arbitrary pieces
  cannot satisfy S + H fixed equations. The sender adds S + H extra pieces
  (RFC 6330 §5.3.3.3: the relations "evaluate to zero"), so every free card's
  result is zero, and the bucket has a slot for each extra piece. Every piece,
  old or new, sits in a free card (B-X7, all 477 block sizes), which is why the
  pieces no packet touched (14_7) are no longer lost. The bucket ignores order
  (03_5), so the ex-or cards are counted first and the byte cards close the
  last H slots. Beat 15_6 is new at R2 (the old 15_2 split at its picture
  change: the ten-piece grid, then the big block).
- **Precisely (lower-thirds):** 15_2 "precisely: RaptorQ adds S + H extra
  unknowns tied to the pieces by S + H fixed equations whose right side is
  zero; packets are ex-or cards over all of them; the sender solves for them all
  so the first K′ packets are the originals" (D3, RB-26); 15_6 "big files are
  cut into blocks of up to 56,403 pieces; each block decodes on its own" (D3).
- **Ruth (RB-01):** the ex-or cards always stop exactly H short of the
  unknowns (L − H = K′ + S), at every K′: 17 of 27 at ten pieces, 10,258 of
  10,269 at 10,017 (B-X7, B-F19). "About seven pieces" is B-F16's 7.152
  intermediate symbols per packet (Table 1's mean degree 4.82 plus 2.34 PI
  symbols), extra pieces included.
- **Model (description, D9, RB-30; keyed at R2, Ruth R5):** random rows in
  RaptorQ's own shape at K′ = 10 fail 0.489% (0.098% ex-or part + 0.392% byte
  part; B-F21.shape_model.*); the gap to the measured 0.63% is the price of
  sparse, structured ex-or rows. Description only.

## 16_parking_a_piece (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | Back to the stuck Sudoku. In big blocks, RaptorQ's decoder mostly peels. When it's stuck, it parks one stubborn piece, here C. Call it little c, and carry it along like an unknown in algebra. | The frozen five-piece graph from 14_5; C's badge becomes a ghost "c" | ghost node |  |
| 16_2 | Three of these are our cards from the start. A is two ex-or c, and B is six ex-or A. D is six ex-or B ex-or c, so the two c's cancel, and D is two. And E follows from D. | Chapter 03's card objects return for A ⊕ C = 2, B ⊕ C ⊕ D = 6, A ⊕ B = 6, beside the new D ⊕ E = 14 and B ⊕ E = 15 (R2 Maya M6); each line lights the card it comes from: A = 2 ⊕ c, B = 6 ⊕ A = 6 ⊕ 2 ⊕ c, D = 6 ⊕ (6 ⊕ 2 ⊕ c) ⊕ c = 2 (R2 Dev dev-05), E = 14 ⊕ D | peel with a ghost |  |
| 16_3 | At the end, the leftover cards and the byte cards pin down the parked pieces. Here the leftover card, B ex-or E is fifteen, gives c as seven. It's elimination in a clever order, so it works exactly when the cards pin down every piece. | The leftover card B ⊕ E = 15 lights, with D ⊕ E = 14 and B ⊕ C ⊕ D = 6: c = 15 ⊕ 14 ⊕ 6 = 7; back-substitution A = 5, B = 3, D = 2, E = 12 | live solve |  |
| 16_4 | ⟨predict⟩ Now the ring of five that ex-or couldn't solve. Let a new byte card replace the fifth card, E plus two times A is six. Walk around the ring. This time, does the A cancel? | The ring walk, one edge per beat; the fifth card turns gold in place (R2 Ruth R11: five cards, not six) | **ring walk** |  |
| 16_5 | With ex-or, A met itself and cancelled. Now one A plus two A is three A, because adding amounts is ex-or too, and one ex-or two is three. The five results ex-or to fifteen, so three A is fifteen, and A is five. | `A ⊕ 2A = (1 ⊕ 2)A = 3A`; the results `6 ⊕ 4 ⊕ 5 ⊕ 14 ⊕ 6 = 15` (R2 Maya M7); `3A = 15 ⇒ A = 5`; "three times five is fifteen here too, but three times seven was nine" | TransformMatchingTex | hold |

- **Lower-thirds:** "RaptorQ parks 158 pieces on purpose at 10,017 pieces;
  search: inactivation decoding"; "the sender also runs one solve, to send the
  originals unchanged first"; 16_3 "succeeds iff rank = L, in any order" (RFC
  6330 §5.4.2.1; RB-02); 16_5 "here 3 is the byte x + 1, not a count: A ⊕ A ⊕ A
  is just A" (RB-03); 16_5 "a ring of cards a·X + b·Y is stuck exactly when the
  ratios b/a multiply to 1 going around; with ex-or every ratio is 1" (RB-D4,
  corrected at R2 per Ruth R3: the R1 wording, "amounts multiply to 1", is false
  for the ring of cards 2X + 2Y, which is stuck although its amounts multiply to
  2¹⁰ = 116; B-F9.all_2_ring_*, B-F9.cycle_rule_stuck_iff_prod_b_over_a_is_1_ok).
- **Dev (D5, open; R2 dev-02 deferred to bead mx-e-b-qyy.10):** a measured
  decode time and inactivation count for one 10,000-piece block become RECORDED
  oracle rows before any RaptorQ decode cost is spoken (the PyPI `raptorq`
  2.0.0 harness is not installed on this machine). Do not voice "1.6 million
  steps": 169³/3 is only the dense core, and back-substitution alone is up to
  L·u ≈ 10,269 × 169 whole-piece operations (Dev). The words of 16_1 ("In big
  blocks, RaptorQ's decoder mostly peels") rest on RFC 6330 §5.4.2.4, not on a
  measurement: the rows solved one at a time are "generally much larger" in
  number than the parked columns (fact sheet B-S3). The cost RaptorQ does state
  aloud since R2 is its packets' density, about seven pieces each (15_1).

## 17_two_packets (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 17_1 | We ran a real RaptorQ decoder a million times, on small blocks of ten pieces. With exactly enough packets, it failed six thousand three hundred sixteen times. About six in a thousand. | Bar with K, trials, failures, interval; caption "raptorq 2.0.0 (Rust); also 0.63% at 101 pieces, 0.54% at 1,002 (200,000 / 20,000 trials)" (R2 Ruth R9) | measured chart |  |
| 17_2 | ⟨predict⟩ One more ex-or packet usually fills one more slot, leaving a byte card spare, and that divides the risk by about two fifty-six. So how many failures now? | The 15_3 bucket: one more blue card rises one slot, and the top gold card is left over, glowing "spare"; then the risk ladder returns | prediction card |  |
| 17_3 | Twenty-four. Two extra, just once. That's too rare to measure precisely. An unlucky phone just waits for one more packet. | Ticks "1 in 100 / 1 in 10,000 / 1 in a million" on the ladder; the measured 6,316 / 24 / 1 beside them; beside the "1": "95% interval: 0.025 to 5.6 per million" (R2 Ruth R8); lower-third (R2 Dev dev-10): "÷256 twice predicts ≈ 0.1 failures here; one or more happens ≈ 9% of the time" | log chart in words |  |
| 17_4 | Averaged over random packets, the standard promises at most one failure in a hundred with exactly enough packets. With one extra, one in ten thousand, and with two, one in a million. | RFC 6330 requirement overlaid, each tick beside its packet count (R2 Maya M9); lower-third "RFC 6330 §5.8: averaged over randomly chosen packet IDs, for each padded block size K′" | statement card |  |
| 17_5 | The originals can go first, unchanged, so a phone with a clean connection does no decoding at all. | Originals then repair packets | statement card |  |
| 17_6 | It's in a standard for next-generation TV broadcasts. Its older sibling was chosen for mobile broadcast in two thousand five. The same byte field protects some disk arrays that survive two dead drives, and every QR code. | Text cards, no logos: "ATSC 3.0 (A/331:2021; the FEC is optional)", "3GPP MBMS, 2005 (Raptor)", "Linux RAID-6", "QR codes" (R2 Dev dev-07) | cards |  |
| 17_7 | Two extra packets buy the standard's one in a million, because every nonzero byte has a partner to divide by. And that's because a polynomial refuses to factor. Three times seven is nine. | The byte clock's mirror pairs; the fold glowing at the top; the bucket fills; the chalk `3 × 7 = 9` returns; the phone field turns green; receipts card | coda; names-to-search lower-third (fountain code, LT code, Raptor, RaptorQ, RFC 6330, inactivation decoding, GF(256), carry-less multiplication, primitive polynomial, Gaussian elimination; R2 Dev dev-13); credits lower-third from RFC 6330's header and §9.2 (R2 Ruth R20): "LT codes: Luby, 2002 · Raptor codes: Shokrollahi and Luby · RFC 6330 (2011): Luby, Shokrollahi, Watson, Stockhammer, Minder" | listen 3 s (the motif resolves) |

- **Ruth:** the requirement is over uniformly random received packet IDs, at
  K′ (padded K); our measurement is K = K′ = 10 (0.631% at 101, 0.54% at 1,002
  at K); the exact 95% interval for 1 failure in 10⁶ is 2.5×10⁻⁸–5.6×10⁻⁶ (on
  screen at 17_3 since R2). The trials drew packets from a fixed pool of the K
  originals and 20,000 repair packets (`rq_mc.py`), not uniformly from 2²⁴ (description; RB-29, D16). A phone that loses random
  packets from a stream that sends the originals first holds a third mix that
  neither the standard's figures nor our runs cover (R2 Ruth R2; description;
  the run that would cover it is deferred to bead mx-e-b-qyy.10).
- **17_2 (R2 Maya M1):** why a plain ex-or packet divides the risk by about
  256: counted first (15_3), one more ex-or card usually takes one more slot
  (probability 0.998 in the shape model), so one byte card has nothing left to
  fill, and a spare byte card divides the risk by about 256 (13_5, B-F8). In
  the shape model at ten pieces that case is 65% of the failures left at K + 1,
  and the model's factor is 208; the real decoder measured 6,316 / 24 = 263
  (B-F21.shape_model.*, B-F22). Description.
- **Not spoken (R2 Maya M10):** "about one of our million phones waits" would
  attach a number to the phones' own mix of packets, which nobody computed
  (Ruth R2); 17_3 says the rare unlucky phone waits for one more packet.
- **Dev (verify before narration):** ATSC 3.0 A/331 citing RFC 6330 is checked
  in A/331:2021 (normative reference [28], Annex A.4.2.4; fact sheet B-S1, D17); 3GPP chose Raptor (RFC 5053) for MBMS in 2005 (RaptorQ's
  adoption into TS 26.346 is unconfirmed, hence "older sibling"); RAID-6 uses
  0x11D (Anvin); QR codes use 0x11D (285) per the fact sheet. R2 (Dev dev-07):
  "a standard" (ATSC 3.0 is one of several next-generation broadcast systems,
  and A/331:2021 Annex A.4.1 makes the FEC optional, "may be protected"), and
  "some disk arrays" with "Linux RAID-6" on the card (the source covers
  Linux md RAID-6 only; other dual-parity schemes are not claimed; the name
  stays off the voice because ASR cannot verify its vowel, lexicon round 3).
