<!--
Draft video description for Video K (P3, 2026-10-13). Every statement below
rests on a row of docs/fact-sheet.md, an oracle key, or the P3 claim walk's
own recomputation; the bracketed tags name them. It holds the exact versions
of what the narration simplifies (plan rule 11), the programmer notes that
have no room on screen, and what review round R1 sent "to the description"
(R1.md: D8, D12, D13, D19, D20, D27, D30, R7, R26). The release package adds
the chapter timestamps and the repository link, and trims this to what the
upload page accepts, linking this file for the rest.
-->

# The One Idea Behind Bridges, Concert Halls and Economies

A bridge, the air in a room and an economy have a few special patterns of
their own: patterns the system hands back unchanged except for size. Described
in those patterns, a tangle of equations in which everything pulls on
everything falls apart into separate one-line problems. That one calculation,
the eigendecomposition of a table of numbers, tells an engineer how a steel
truss sags and sways, which notes a room sings back and whether an economy can
feed itself, and it is just as useful to know where it stops.

## Chapters

(Timestamps come with the finished cut.)

1. The bridge that swayed
2. Two carts
3. Own patterns
4. The split
5. A steel truss
6. Twenty-one patterns
7. Every mode a note
8. Two bridges
9. The air in a room
10. Two million modes
11. Where modes rule
12. An economy
13. Can it feed itself
14. One idea

## Precisely

What the narration says in plain words, stated exactly. The truss, the hall,
the small rooms and the two-industry economy are models built for this video;
the screen says "a model" each time.

**The Millennium Bridge, London.** Opened 10 June 2000; between 80,000 and
100,000 people crossed on the first day, with at most 2,000 on the deck at
once; closed on 12 June, two days later (Dallard et al. 2001, pp. 20 to 21)
[K-B2.opened, .closed, .first_day, .on_deck]. The deck's maximum sway was
about 70 mm (Arup) [K-B2.sway_mm]. The measured sideways modes were 0.48 and
0.95 Hz (centre span), 0.80 Hz (south) and 1.03 Hz (north) (Dallard et al.,
Table 3) [K-B2.lateral_hz]: "between half a swing and one swing a second".
Walking has a vertical force at about 2.0 Hz and a sideways force at half the
walking frequency, so about 1.0 Hz [K-B2.walk_hz, .walker_lateral_hz]. The
effect "could occur on any bridge with a lateral frequency below about 1.3 Hz
loaded with a sufficient number of pedestrians" [K-B2.risk_below_hz]. The
engineers' own paper calls the cause "a loading effect which had not been
anticipated during design"; vertical excitation had been assessed in design by
a modified BS 5400 approach; Arup's page says the British Standard "is being
updated to cover the phenomenon". The paper also says "the measured
frequencies of the modes which had been observed to vibrate were within 10% of
the values predicted in the design analysis" [K-B2.predicted_within_percent]:
the design model had the sideways modes right, and what no code had asked was
what would push along with them. The bridge reopened on 22 February 2002 with
37 viscous dampers and tuned mass dampers, 4 pairs for the sideways modes and
26 pairs for the vertical ones (Dallard et al.; Taylor Devices)
[K-B2.dampers_viscous, .tmd_lateral_pairs, .tmd_vertical_pairs, .reopened].
A magazine of 1 February 2002 gives 54 tuned mass dampers; the video shows
only the 4 sideways pairs.

**Why the sway grew is disputed, and the video says only what holds under
both accounts.** Dallard et al. (2001) and Strogatz and others (2005) have the
walkers falling into step with the deck; Belykh and others (2021) find that
synchrony is "a consequence of, not a cause of the instability" and that
uncorrelated walkers give negative damping on average. All agree on a critical
crowd size and on a sideways force that grows with the deck's motion ("as the
amplitude of the motion increases, the lateral force imparted by individuals
increases"). The video says that and no more. Arup's test: the north span was
stable with 156 people and with just 10 more the movement increased suddenly
[K-B2.test_steady, .test_added].

**Tacoma Narrows.** Opened 1 July 1940, failed 7 November 1940, 129 days
later [K-B1.opened, .failed, .days_open]; the wind was clocked at 42 miles an
hour that morning ("about forty") [K-B1.wind_mph]. The torsional motion began
at 14 cycles a minute, and the final destructive motion was 12 a minute,
0.2 Hz, one twist every five seconds (Farquharson, through Billah and Scanlan
1991) [K-B1.twist_per_min, .twist_hz, .twist_period_s]. Vortex shedding at that
wind would come "close to 1 Hz", which is why natural vortex shedding "was not
the cause"; Billah and Scanlan call it "single-degree-of-freedom torsional
flutter", a self-excitation, and add that one may still say "resonance" "if we
now identify the source of the periodic impulses as self-induced" [K-B1.vortex_hz].
The board's report (1941) says the bridge "was well designed and built to
resist safely all static forces, including wind, usually considered in the
design of similar structures"; its failure "resulted from excessive
oscillations caused by wind action". Billah and Scanlan's page quotations were
read in the research pass from a scan; the copy kept outside the repository
holds only the abstract, so at R1 they were re-checked against the abstract
and the board's report only.

**Two carts.** Three springs of 1 N per centimetre (100 N/m) between two walls
and two carts: the table is [2, −1; −1, 2], the pushes for moves (1, 0), (1, 1)
and (1, −1) are (2, −1), (1, 1) and (3, −3), the own patterns are (1, 1) with
eigenvalue 1 and (1, −1) with eigenvalue 3 [K-A1.*, K-A2.*]. The angle between a
move and the push that holds it is at most 30°, since
sin θ = (3 − 1) ÷ (3 + 1); the moves for a push of (1, 0) are (2/3, 1/3), found
as half of each pattern, each divided by its eigenvalue: 1/2 ÷ 1 and 1/2 ÷ 3
[K-A2.worst_turn_deg, K-A3.*]. An own pattern is a direction: any non-zero
multiple of it, of either sign, is the same one; the gold marks are lines. In
general the table is K = QΛQᵀ with Q's columns of length 1, for tables like
this one; for tables with too few own patterns (a shear) there is no such Q.

**The truss.** A Pratt truss, 24 m span in six panels of 4 m, 4 m deep, 12
joints and 21 members, a pin at one end and a roller at the other, steel with
E = 200 GPa (the US round figure, 29,000 ksi; Eurocode practice uses 210),
chords and end posts of 6,000 mm², verticals and diagonals of 3,000 mm²,
7,850 kg per m³ [K-T1.*]. A member is a spring k = EA ÷ L: a 4 m bottom
member is 200 GPa × 6,000 mm² ÷ 4,000 mm = 300 kN per mm, the weight of about
30.6 tonnes [K-T1.spring_kn_per_mm, .spring_tonnes]. 12 joints × 2 − 3 held
directions = 21 moves left. The table K is 21 by 21 with 97 of 441 entries
not zero; unsupported it has exactly three eigenvalues of zero (slide, lift,
turn); one 100 kN load (about 10.2 tonnes) on a deck joint moves all 21
directions, 4.7 mm down there and 1.3 mm sideways at the roller, and the
inverse of K has 441 of 441 entries not zero [K-T2.*, K-T3.*, K-X3.*].

100 kN on each of the five deck joints sags the middle 17.96 mm, span ÷ 1,336;
each support gives 250 kN; the top chord is squeezed 450 kN (75 N per mm², 30%
of A36 steel's minimum yield of 250 N per mm²), the bottom chord stretched
400 kN, the end posts squeezed 353.6 kN, the diagonals pulled 212.1 kN, the
middle post carries 0 [K-T4.*, K-T5.*]. The yield figure is A36's 36 ksi
through a secondary citation of the AISC manual; a squeezed member is also
checked for buckling in practice, which is another eigenvalue problem and is
not checked here. The span ÷ 1,000 comparison is the AASHTO deflection limit
for urban bridges used by pedestrians, through NCHRP Web Document 46; it is
not a footbridge-specific limit.

K's 21 own patterns run from 3.12 to 1,226 MN per m, a factor of 393
[K-P1.*]. The softest pattern alone gives 99.6% of the sag at the middle,
from two larger terms that cancel: the readout runs 99.6, 102.0, 101.8, 99.1,
98.9, then 99.1 at ten patterns and 100.0 at fifteen [K-P2.sag_percent.*],
and the largest error anywhere in the moves is 1.2 mm [K-P2.move_error_mm.1].
The worst member force is off by 82 kN with one pattern (18% of the largest),
44 with five, 8 with fifteen and 0 with all 21 [K-P2.force_error_kn.*]. The
split's formula is u = Σ (qᵢ · f) ÷ (λᵢ qᵢ · qᵢ) qᵢ; it reduces to qᵢ · f ÷ λᵢ
when the patterns have length one [K-X4.rebuilt]. A force is a stretch times a
stiffness, which puts back what the division took out, so the stiff patterns
count for forces; that is why truncating the sum settles the sag and not the
forces.

With a 4 tonne deck mass at each interior deck joint (half at each end joint)
and the steel's own 3.4 tonnes lumped half to each end of every member, the
modes of K φ = ω² M φ are 5.91, 11.78, 14.86 and 20.93 Hz, in ratio
1 : 1.99 : 2.51 : 3.54; the second mostly slides the deck along its length,
because the right support is a roller, and the third bends it into an S. With
8 tonne decks the lowest is 4.32 Hz, not 5.91 ÷ √2 = 4.18 Hz, since the steel
did not get heavier [K-V1.*, K-V2.*]. The eigenvalue is ω², hertz = ω ÷ 2π.
The modes are at right angles when mass is counted (φᵢᵀ M φⱼ = 0) and not
otherwise [K-V1.at_right_angles_with_mass, .at_right_angles_without]. The
model has no damping and moves only in its own plane: with damping that is
not proportional the modes couple.

**The air in a room.** Slices of air with mass, and squeezed air pushes back
like a spring: a row of carts and springs with the same table, 2 down the
diagonal and −1 beside it, closed at both ends. With 40 slices the lowest tone
is within 0.03% of the continuous tube's [K-R1.error_percent,
.error_bound_percent]; one slice pushed out of place comes back as
(−1, 2, −1), 35.3° away from itself, and one smooth hump comes back as itself,
0° away [K-R1.lump_turn_deg]. The speed of sound is 343 m per second (air at
about 20 °C; FrankenSim's air is 293.15 K) [K-R0.sound_m_per_s]. For a box
with hard walls the tones are (c ÷ 2) √((nx ÷ Lx)² + (ny ÷ Ly)² + (nz ÷ Lz)²)
for whole numbers nx, ny, nz not all zero (Jacobsen and others, eq. 3.1.5);
pressure is greatest at the walls and corners, where the air moves least. One
swing of the bathroom's longest side is a round trip: 343 ÷ (2 × 2.5 m) =
68.6 Hz. The bathroom of 2.5 × 1.6 × 2.4 m has nine tones between 60 and
150 Hz: 68.6, 71.5, 99.1, 107.2, 127.3, 128.8, 137.2, 142.9 and 146.0 Hz, of
which 68.6, 71.5, 107.2, 137.2 and 142.9 fit one side (the length, the height,
the width, twice the length, twice the height) and the other four need two or
three [K-R2.*; the tone list is the P3 claim walk's enumeration, to be pinned
as a fact before the scene is built].

**The concert hall.** A box 45 × 23 × 18 m is 18,630 m³, 99.4% of Boston
Symphony Hall's 18,750 m³ (Beranek). Beranek's own width, height and length
are averages and their product is not the volume. The model box has 29 modes
at or below 20 Hz, 2,246 at or below 100 Hz and 1,964,040 at or below 1 kHz
(the textbook estimate N = (4πV ÷ 3)(f ÷ c)³ + (πS ÷ 4)(f ÷ c)² + (L ÷ 8)(f ÷ c)
gives 1,964,120), about 5,800 in every hertz there [K-H1.*, K-H2.*]. The
count grows as the cube of the frequency and depends on the volume, hardly on
the shape. Occupied, at mid frequencies, the hall takes 1.9 s to die away
(Beranek), so each mode is blurred about 2.2 ÷ 1.9 = 1.2 Hz wide, and some
6,700 modes sit inside one mode's width at 1 kHz [K-H1.t60_s, K-H2.mode_width_hz,
.modes_per_width_at_1000]. The Schroeder frequency 2000 √(T60 ÷ V) is
20.2 Hz here, the bottom edge of hearing: a crossover, not a wall; above it
the modes still exist but overlap. "Three or more modes in one width" is the
1962 criterion (Schroeder and Kuttruff; his 1954 figure was ten); the video's
"almost every note you can hear in a hall lies above the crossover" is
20.2 Hz against the 20 Hz bottom of hearing, with organ notes lower still
[K-H3.*]. The 20 Hz hearing floor is a convention; hearing extends lower at
high levels.

Halls are designed with statistics (Sabine's 1900 paper takes "the new Boston
Music Hall" as its example): T60 = 0.161 V ÷ A, which for this hall asks
1,579 m² of absorption [K-H4.*]. 0.161 is the modern metric constant; his own
was 0.164 (Collected Papers, p. 50). He advised on Symphony Hall, opened on
15 October 1900; he did not design it. Three of the most admired concert
halls are shoeboxes (Beranek): the trouble is sides that match, not boxes.
"Statistical energy analysis" is a different method, for coupled panels and
cavities; to search for how halls are designed use "Sabine equation",
"geometrical acoustics" and "image sources". Rays work once the waves are
short against the walls; below that, cells.

**Small rooms and the cells.** A study of 4.8 × 4 × 2.6 m (50 m³, a model
decay of 0.5 s) has its crossover at 200 Hz with 67 modes below, none on one
tone; a narrow room of 6.1 × 2.1 × 2.5 m has 48 below 200 Hz, none shared; a
3 m cube has 38, of which 36 share a tone, the lowest being 343 ÷ 6 = 57.2 Hz,
three times over [K-R3.*, K-R4.*, K-R5.*]. "Rooms with the three principal
dimensions related by simple integers are especially poor" (Walker, BBC
RD 1993/8); in a cube "threefold and even sixfold degeneracies are common"
(Morse). Measured two ways by the P3 claim walk (most modes inside one window
of 0.5, 2 and 4.4 Hz below 200 Hz): the cube 6, 6, 6; the narrow room 2, 4, 6;
the study 2, 4, 6. The cube is never worse and is the only one with exact
coincidences; at the study's own mode width the three tie, so "piles its tones
up most" means tones at the same pitch, which is what the video says. An attic
studio of 6 × 3 m with its ceiling sloping to 1.25 m, cut into 232 cells of
0.25 m, has modes at 29.8, 56.1 and 59.9 Hz; the same method on a full box of
288 cells gives 28.56 Hz against the formula's 28.58 (0.07%) [K-N1.*].
FrankenSim's modal casebook runs a 317 × 317 grid, 100,489 unknowns, for
its lowest modes (`fs-modal`); the video says so of its solver and not of
its rooms, which FrankenSim does not hold.

**An economy.** Leontief's input-output tables (1936; The Structure of
American Economy, 1941) had 44 sectors, 41 of them producing; Nobel prize in
economics, 1973 [K-S.leontief.*]. The video's table is two industries and a
model: a unit of food takes 0.4 food and 0.2 tools, a unit of tools 0.3 food
and 0.5 tools [K-E1.*]. Its eigenvalues are 0.7, with the pattern (1, 1), and
0.2, with (3, −2), which needs a negative amount of one good [K-E2.*]. 24 of
each wanted takes 80 of each, in rounds of 24, 16.8, 11.76, 8.232 and so on
(99% of the total after 13 rounds) [K-E3.*]; a demand of (30, 10) is 18 of
(1, 1) plus 4 of (3, −2), which divided by 1 − 0.7 and 1 − 0.2 gives 60 and 5
of them, a plan of (75, 50), and its rounds run (30, 10), (15, 11), (9.3, 8.5),
(6.27, 6.11) [K-E5.*]. With 1.1 for each one made, the rounds grow, and the
formula still answers, 24 ÷ (1 − 1.1) = −240 of each, which is why a program
should check the eigenvalue, or that the plan has no negative entry [K-E4.*].
The exact statement: for a table C with no negative entry, every demand
d ≥ 0 has a plan x ≥ 0 with x = Cx + d exactly when the largest eigenvalue of
C is below 1 (equivalently the Hawkins and Simon condition, 1949). At 1 or
more some demand fails, and every demand when C is irreducible, as here; a
table that splits into separate parts may still meet some demands (the P3
claim walk's table [1.1, 0; 0.3, 0.2] meets (0, 1) with the plan (0, 1.25)).
Checked on the video's table scaled by k ÷ 20 for k = 1 to 40, and by 10 ÷ 7
(largest eigenvalue exactly 1): 41 of 41 agree with the inverse, 28 feed
themselves, and at exactly 1 it does not [K-X7.*]. The rounds add up to
(I − C)⁻¹ d, the Leontief inverse, the Neumann series 1 + C + C² + ….

**The theorems behind the pictures.** For a symmetric table (the same on both
sides of the diagonal) a full set of own patterns always exists, at right
angles, with real eigenvalues (the spectral theorem); checked on 1,000 seeded
symmetric tables of sizes 2 to 6 and on two with repeated eigenvalues, where
the patterns are not unique but can be chosen at right angles [K-X1.*]. For a
table with no negative entry, Perron (1907) and Frobenius (1912) give that the
largest eigenvalue in size, ρ, is itself an eigenvalue with a pattern with no
negative entry (Horn and Johnson); if the table is irreducible that pattern is
positive and unique up to scale; checked on 1,000 seeded tables of positive
entries and 1,004 with zeros, among them a shear, a swap, a table that sends
everything to zero and diag(1, 2) [K-X6.*]. That is all nonnegativity gives:
the economy's table splits because its two eigenvalues differ. The pattern of
the largest eigenvalue "ranked web pages" in the sense that PageRank
"corresponds to the principal eigenvector of the normalized link matrix of the
web" (Brin and Page 1998).

**Where it stops.** A shear [1, 1; 0, 1] has no negative entry and one own
direction [K-X2.shear]. A quarter turn [0, −1; 1, 0] has no real own pattern
you can draw; over the complex numbers its eigenvalues are ±i
[K-X2.quarter_turn]. Steel bent too far stops answering in proportion (yield).
Where it holds, the same split gives powers of a table, Cⁿ = V Λⁿ V⁻¹, and its
exponential; that is safe for symmetric tables, and for the others libraries
compute the exponential another way (the companion video on the matrix
exponential). The factory of the owner's third image is a linear program, not
an eigenproblem: three limits (2x + y ≤ 100, x + 2y ≤ 80, x + y ≤ 60), profit
8x + 7y, corners (0, 0), (0, 40), (40, 20) and (50, 0) with profits 0, 280, 460
and 400; the best is (40, 20) at 460, which no eigenvector finds, checked
against all 1,361 allowed whole-number plans [K-L1.*, K-L2.*, K-X5.*]. The
image's plotted corner (0, 60) breaks the aluminium limit, since x + 2y ≤ 80
caps y at 40. The simplex method (Dantzig, 1947) walks corners; other
solvers, FrankenSim's among them, do not; the corner is where the answer is,
not how every solver finds it. Here the corners are whole numbers; in general
they are not, and whole-number plans are integer programming, a harder problem.
All three limits are tight at (40, 20), so the third line only touches the
corner.

**"Eigen".** German for "own". Helmholtz writes of the Eigentöne of a body of
air in 1863 (Die Lehre von den Tonempfindungen, p. 147) and Hilbert names
Eigenfunktionen and Eigenwerte in 1904, for integral equations (his
Eigenwerte are the reciprocals of today's eigenvalues): the same word, 41
years apart [K-S.helmholtz_year, .hilbert_year, .eigen_gap_years]; whether
Hilbert had Helmholtz's word in mind is conjecture.

## For programmers

- **Assembly.** For each member from joint a to joint b, with (c, s) the unit
  vector along it and g = (−c, −s, c, s): `K[r_i][r_j] += (E A / L) g_i g_j`
  over the rows r = (2a, 2a + 1, 2b, 2b + 1); then strike out the three held
  rows and columns. The truss's solved moves and member forces are checked
  against joint equilibrium (BᵀN = f) and the unit-load sum, neither of which
  forms the table [`oracle.py`].
- **One static load.** Real programs factor K once, or iterate, and reuse it
  for every load; nobody computes K's eigenvectors for that. The split
  explains the answer; it is not how the answer is computed. A library call
  finds all 21 patterns: LAPACK's symmetric eigen-solver, `numpy.linalg.eigh`,
  which `oracle.py` uses; the video's own `model.rs` uses cyclic Jacobi
  rotations (the method of FrankenSim's dense kernel).
- **Why not invert.** The inverse of a sparse table is full (441 of 441), so
  programs factor and never invert. The condition number here is 393.
- **A few modes of a huge table.** All modes of an n-move table cost about n³
  steps and n² numbers; the Lanczos algorithm (1950) finds the lowest few of a
  huge sparse table without the rest. FrankenSim's `fs-modal` does this with
  certified counts and tests it on a spring-and-mass chain of 40 and on a
  317 × 317 grid; it finds a handful of the lowest modes, not 100,489.
- **Modal truncation.** Keeping the lowest few modes settles moves quickly and
  forces slowly; the names to search are modal truncation and static
  correction. How many modes a code asks for (a modal mass participation
  threshold) was not checked and the video does not say.
- **The economy without an eigen-solver.** Iterate x ← Cx + d and stop when
  the step is small; the ratio of one round to the last settles at the largest
  eigenvalue (power iteration). The same loop on the hungry table never
  settles.
- **Read the toy.** `oracle.py` is a short independent implementation of every
  number in the video; it can be read in an hour.

## Names to search

eigendecomposition, diagonalization, superposition (a mix of patterns),
modal analysis, modal truncation, static correction, the Lanczos algorithm,
aeroelastic flutter, synchronous lateral excitation, negative damping, Schroeder
frequency, Sabine equation, geometrical acoustics, image sources, the finite
element method, Leontief inverse, Neumann series, Hawkins–Simon condition,
Perron–Frobenius, spectral theorem, linear programming, the simplex method.

## What was checked, and how

Computed numbers: every number the video shows or says that is computed (the
carts, the truss, the modes, the rooms, the hall's counts, the economy, the
factory) is printed by the video's own code and by a second, separately
written program (`oracle.py`), and the two agree key by key (300 keys). The
second program derives the truss's forces by joint equilibrium, its sag by
the unit-load sum and its eigenvalues from the table of member stretches, and
counts room modes by a second enumeration. The P3 claim walk then recomputed
every number the script speaks from first principles, in a third program that
shares no code with either, and ran each always or never sentence at its
edges (decisions D-15 to D-54; the fact sheet's edge tables).

Cited numbers (dates, 70 mm, 156 and 10, 42 mph, 12 a minute, 1.9 s, 18,750 m³,
44 sectors, the years) are typed from their sources into both programs, which
checks nothing about them; each is sourced above and in `docs/fact-sheet.md`
with what was and was not seen.

Not claimed, and so not said: a source for "the first hall designed by a
formula"; English pronunciations for Schroeder, Perron and Frobenius beyond
the dictionaries'; a footbridge-specific deflection limit; a modal mass
participation threshold.

## Sources

Dallard, Fitzpatrick, Flint, Le Bourva, Low, Ridsdill Smith and Willford,
"The London Millennium Footbridge", The Structural Engineer 79(22), 2001;
Arup's project pages; Taylor Devices, "Damper Retrofit of the London
Millennium Footbridge"; Strogatz, Abrams, McRobie, Eckhardt and Ott, "Crowd
synchrony on the Millennium Bridge", Nature 438 (2005); Belykh and others,
Nature Communications 12:7223 (2021); Ammann, von Kármán and Woodruff, The
Failure of the Tacoma Narrows Bridge (1941); Billah and Scanlan, American
Journal of Physics 59(2) (1991); Helmholtz, Die Lehre von den Tonempfindungen
(1863); Hilbert, Nachrichten der Gesellschaft der Wissenschaften zu Göttingen
(1904); Sabine, Collected Papers on Acoustics (1922); Schroeder and Kuttruff,
Journal of the Acoustical Society of America 34(1) (1962) (through Skålevik,
Forum Acusticum 2011); Jacobsen and others, Fundamentals of Acoustics and
Noise Control, DTU Note 31200 (2007); Beranek, Concert Halls and Opera Houses
(2004) and his 2007 lecture; Walker, BBC RD 1993/8; Morse, Vibration and
Sound (1948); Leontief (1936; 1941, through Kohli, Monthly Labor Review 2001);
Hawkins and Simon, Econometrica 17 (1949); Perron, Mathematische Annalen 64
(1907); Frobenius, Sitzungsberichte der Königlich Preussischen Akademie der
Wissenschaften (1912); Horn and Johnson, Matrix Analysis; Brin and Page (1998);
Bertsimas and Tsitsiklis, Introduction to Linear Optimization (1997);
Lanczos, Journal of Research of the National Bureau of Standards 45 (1950);
FHWA, Steel Bridge Design Handbook (2012); NCHRP Web Document 46 (2002).
FrankenSim (the owner's workspace) is cited by path only where its subject is
there: the generalized eigenproblem solver and its chain test, the dense
Jacobi kernel, the rectangular-cavity formula and its steel frames. It has no
pin-jointed truss solved by a stiffness table, no room acoustics and no
input-output model.
