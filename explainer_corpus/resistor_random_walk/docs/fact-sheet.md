# Fact Sheet — A Resistor Network Is Secretly Answering a Probability Question

Every number shown or spoken. Each row's value is computed in `src/facts.rs`
(from `src/model.rs`) and recomputed independently by `oracle.py`; `./x
resistor_random_walk facts-check` compares the two outputs key by key.
Junction lists run top, left, right, bottom; wire lists follow
`model::LINKS` (top–left, top–right, left–right, left–bottom,
right–bottom). The Rust model eliminates the current balance and inverts
I − Q of the walker's absorbing chain; the oracle counts spanning forests
(Kirchhoff) and uses Cramer's rule with permutation determinants.

| id | quantity | shown / said | computed in | oracle key | value | source |
|---|---|---|---|---|---|---|
| F1 | junctions, middle junctions, links, identical wires | "four junctions" (01_1); the drawn network | `model::opening`, `LINKS` | `R-F1.junctions`, `.middle`, `.links`, `.wires` | `4`, `2`, `5`, `7` | computed |
| F1 | links with wires side by side, and how many wires each | "doubled in two places" (01_1); the flashing pairs | `LINKS` | `R-F1.doubled`, `R-F1.side_by_side` | `2`, `2` | computed |
| F1 | wires at each middle junction | "its four wires" (01_2, 01_4, 01_5) | `Network::far_ends` | `R-F1.wires_at_middle` | `4` | computed (one number for both, or an error) |
| F1 | the ends' voltages | "the top at one volt and the bottom at zero"; "1 volt", "0 volts" (01_1) | `Network::kirchhoff(1, 0)` | `R-F1.top_volts`, `R-F1.bottom_volts` | `1`, `0` | computed |
| F2 | every voltage | the circuit's labels 3/5 and 2/5 (01_3 on) | `Network::kirchhoff` | `R-F2.voltages`, `R-F2.left`, `R-F2.right` | `1,3/5,2/5,0`; `3/5`; `2/5` | pinned: the two averages solved by hand |
| F2 | the left's far ends, and their average | "one, one, two fifths and zero average to three fifths" (01_3); the equation (1 + 1 + 2/5 + 0)/4 = 3/5 (01_3, 01_7) | `facts::far_values`, `facts::mean`; checked against `Network::average` (conductance-weighted) | `R-F2.left.far_ends`, `R-F2.left.average` | `1,1,2/5,0`; `3/5` | computed |
| F2 | the right's far ends, and their average | the silent check and its equation (1 + 3/5 + 0 + 0)/4 = 2/5 (01_3) | same | `R-F2.right.far_ends`, `R-F2.right.average` | `1,3/5,0,0`; `2/5` | computed |
| F2 | each link's current, and the balance at the middle | not shown (the 01_2 lower-third states the law) | `Network::currents`, `Network::net_out` | `R-F2.currents`, `R-F2.balance` | `4/5,3/5,1/5,3/5,4/5`; `true` | computed |
| F3 | every chance of reaching the top first | the walk's labels 1, 0, 3/5, 2/5 (01_6, 01_7) | `Network::walk` | `R-F3.chances`, `R-F3.left`, `R-F3.right` | `1,3/5,2/5,0`; `3/5`; `2/5` | computed |
| F3 | the chance from the top and from the bottom | "From the top the chance is one, and from the bottom it's zero" (01_6) | same | `R-F3.top_chance`, `R-F3.bottom_chance` | `1`, `0` | computed |
| F3 | each of the left's wires, picked at random | "picks one of its four wires at random" (01_4) | 1 ÷ wires at the left | `R-F3.wire_chance` | `1/4` | computed |
| F3 | the left's far-end chances and their average | the tokens 1, 1, 2/5, 0 and the walk's equation (01_6, 01_7) | `facts::far_values` on the walk | `R-F3.left.far_ends`, `R-F3.left.average` | `1,1,2/5,0`; `3/5` | computed |
| F3 | the walk ends: top-first plus bottom-first, from each middle junction | "stops with chance 1" (01_5 lower-third) | `Walk::top + Walk::bottom` | `R-F3.stops` | `1,1` | computed |
| F3 | still walking after 1, 2, 3 steps | the 01_5 lower-third | `Network::still_walking` | `R-F3.still_walking` | `1/4,1/16,1/64` | computed |
| F3 | the drawn route | the walker's moves (01_4) | `model::ROUTE`, checked by `Network::is_route` | `R-F3.route` | `left,right,left,right,top` | computed (refused if not a walk) |
| F3 | the two pictures agree at every junction | "Three fifths again"; "The two numbers agree" (01_7, 01_8) | `Walk::top == kirchhoff` | `R-F3.same` | `true` | computed |
| F4 | each wire's drop into the left junction, far end minus junction | the signs of the arrowheads (02_2); the balance row (02_4) | `Network::drops_into` | `R-F4.drops_into`, `R-F4.flow` | `2/5,2/5,-1/5,-3/5`; `in,in,out,out` | computed |
| F4 | the drop along each link, the way its current flows | "drops two fifths of a volt", "one fifth to the right", "three fifths to the bottom" (02_2); the values beside the wires | `v[far] − v[left]`, by size | `R-F4.drop.top`, `.right`, `.bottom` | `2/5`, `1/5`, `3/5` | computed |
| F4 | the drops of what flows in and out, and their totals, in volts (each current is its drop over the shared resistance) | "so the drops balance too. Two fifths and two fifths in, one fifth and three fifths out" (02_3); the "drops in" and "drops out" rows | the drops split by sign | `R-F4.in`, `R-F4.out`, `R-F4.in_total`, `R-F4.out_total`, `R-F4.net` | `2/5,2/5`; `1/5,3/5`; `4/5`; `4/5`; `0` | computed |
| F4 | the unknown gathered, one per wire | "four times it" (02_4); 4v = 1 + 1 + 2/5 + 0 | the number of wires | `R-F4.coefficient` | `4` | computed |
| F5 | the walker's first step up | "One half" (02_6); "first step up: 2/4 = 1/2" | wires to the top over all wires, checked against the conductance weight | `R-F5.top_wires`, `R-F5.to_top` | `2`, `1/2` | computed |
| F5 | the tempting wrong answer | "not one third" (02_6) | one over the distinct far junctions | `R-F5.neighbours`, `R-F5.trap` | `3`, `1/3` | computed |
| F5 | the first-step terms and their sum | "A quarter of each, added" (02_7); p = ¼·1 + ¼·1 + ¼·2/5 + ¼·0 = 3/5 | a quarter times each far end's chance | `R-F5.terms`, `R-F5.sum` | `1/4,1/4,1/10,0`; `3/5` | computed (checked against `Walk::top`) |
| F6 | the left's link conductances, their total and weights | "weight each far end by its wire's conductance" (02_8); c = 2, 1, 1; weights 2/4, 1/4, 1/4 | `Network::between`, `Network::total` | `R-F6.left.conductances`, `.total`, `.weights` | `2,1,1`; `4`; `1/2,1/4,1/4` | computed |
| F6 | counting identical wires gives the conductance weights | the pair's two quarters merging into one half (02_6) | per-wire count × ¼ against c ÷ total | `R-F6.pair_adds` | `true` | computed |
| X4 | the pair counted as one wire: another network | the 02_6 lower-third | `model::single_pair` | `R-X4.single.to_top`, `.voltages`, `.chances` | `1/3`; `1,5/11,4/11,0`; `1,5/11,4/11,0` | computed (edge test) |
| X5 | the same weighted average taken at the top, where the rule does not apply | not shown: the edge behind the 02_8 note's domain ("x: any middle junction") | `Network::average` at the top | `R-X5.top_average` | `8/15` (the top is 1) | computed (edge test) |
| X1 | the current balance's determinant, and how many solutions | "only one solution" (01_7) | `Network::middle_determinant` | `R-X1.determinant`, `R-X1.solutions` | `15`, `1` | computed |
| X1 | both ends at zero | the 01_7 lower-third ("0 at both") | `kirchhoff(0, 0)` | `R-X1.zero_ends` | `0,0,0,0` | computed |
| X2 | an island: two more junctions joined only to each other | the edge of "only one solution" (not shown) | `model::with_island` | `R-X2.island.cut_off`, `.determinant`, `.refused` | `4,5`; `0`; `true` | computed (edge test) |
| X3 | every network on the four junctions, each of six links absent, one wire or two | the edge of "only one solution" and of the agreement (not shown) | `model::family` | `R-X3.family.networks`, `.determined`, `.cut_off`, `.singular`, `.agree`, `.between_ends` | `729`, `672`, `57`, `57`, `672`, `672` | computed (edge test) |
| P1 | Doyle and Snell's worked example | not shown | `model::doyle_snell` | `R-P1.voltages`, `R-P1.chances`, `R-P1.current_cd`, `R-P1.steps_from_c` | `7/16,3/8`; `7/16,3/8`; `1/8`; `1/4,1/4,1/2` | pinned: Doyle and Snell §1.3.1, §1.3.2 |
| P2 | five identical wires in a line | not shown | `model::line(5)` | `R-P2.line5` | `0,1/5,2/5,3/5,4/5,1` | pinned: Doyle and Snell §1.1.5, f(x) = x/N |

## Claims (non-numeric)

| id | claim | basis | checked by |
|---|---|---|---|
| C1 | Current in equals current out at a middle junction, so its voltage is the conductance-weighted average of its neighbours' (01_2) | Doyle and Snell §1.3.2 (Ohm's law ixy = (vx − vy)Cxy; Kirchhoff's current law; vx = Σ Pxy vy) | `R-F2.balance`; `Network::average` against the voltages in `facts` and `model::tests` |
| C2 | Two identical wires side by side act as one of twice the conductance, so the plain average over wires is the weighted average over links (01_1 to 01_3) | Doyle and Snell p. 43 ("its conductance is the sum of the two conductances") | `facts` checks `mean(far_values) == Network::average` at both middle junctions |
| C3 | The walker's chance of reaching the top first is the average of its chances from its wires' far ends (01_5) | first-step analysis; Doyle and Snell §1.3.2 ("hx is also harmonic") | `R-F3.left.average`; the oracle's Cramer solve of h = P h |
| C4 | The walk's routes never run out, but it stops with chance one (01_5) | from the left, staying in the middle needs the single wire to the right (1/4), and back (1/4) | `R-F3.still_walking`, `R-F3.stops`; `model::tests::the_walk_from_the_left_is_still_going_with_chance_a_quarter_per_step` |
| C5 | One rule with the same top and bottom has only one solution, so the voltage and the chance agree (01_7) | Doyle and Snell §1.1.5 (maximum and uniqueness principles), §1.2.6 (on an absorbing chain, f inside = B f on the boundary, B = (I − Q)⁻¹R), §1.3.2 ("Hence v = h") | `R-X1`; its edges `R-X2` (an island: determinant 0, no unique answer) and `R-X3` (zero determinant exactly where a junction is cut off; agreement on all 672 determined networks); the 01_7 card states the hypothesis (every middle junction wired to the top or bottom) and is built only when `Network::cut_off` is empty |
| C6 | The walker is a model of the equations, not a claim about electrons (01_8) | the slate's scope limit; Doyle and Snell's own walk-based reading of current starts from "a naive view of the process of electrical conduction" and says "realism is not what we're after" (§1.3.3) | wording only: the script makes no claim about charge carriers |
| C7 | A wire's current is its drop over its resistance, and with steady currents the currents into a middle junction add to zero; with identical wires the drops balance as the currents do, so 4v = 1 + 1 + 2/5 + 0 (02_1 to 02_4) | Doyle and Snell §1.3.2 (ixy = (vx − vy)/Rxy; Kirchhoff's current law) | `R-F4.net` is 0; `model::tests::the_drops_balance_and_the_pair_counts_twice`; the oracle checks 4·v(left) against the far ends' sum |
| C8 | After its first step the walker's chance is the chance from where it landed, so the left's chance is a quarter of each far end's (02_7) | first-step analysis on the absorbing chain (Doyle and Snell §1.3.2, "hx is also harmonic"; §1.2.6) | `R-F5.terms` sum to `R-F3.left`; the oracle's Cramer solve of h = P h |
| C9 | Wires side by side count separately in both pictures, and their conductances add; counting the pair as one wire is another network (02_6, 02_8) | Doyle and Snell p. 43 (conductances in parallel add); §1.3.1 (Pxy = Cxy / Cx) | `R-F6.pair_adds`; `R-X4` (5/11 and 1/3 instead of 3/5 and 1/2, the same in both pictures) |
