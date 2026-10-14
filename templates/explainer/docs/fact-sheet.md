# Fact Sheet — __TITLE__

Every number shown or spoken. `value` must equal `oracle output` at the
displayed precision, and each row is pinned in `src/facts.rs::self_check()`.

| id | quantity | shown / said | computed in | oracle | oracle output | source |
|---|---|---|---|---|---|---|
| F1 | Points in the set ($N$) | `5,000` ("five thousand") | `facts::POINTS` | `points` | `5000` | Demo parameter |
| F2 | Points chosen per subset ($k$) | `4` ("four") | `facts::PICK` | `pick` | `4` | Demo parameter |
| F3 | Number of 4-point subsets $C(5000, 4)$ | `26,010,428,123,750` ("more than twenty-six trillion") | `model::quadruples(5000, 4)` | `quadruples_5000` | `26010428123750` | $\binom{5000}{4} = \frac{5000 \cdot 4999 \cdot 4998 \cdot 4997}{24}$ |
| F4 | Whole trillions in $C(5000, 4)$ | `26` ("twenty-six trillion") | `model::trillions_floor` | `quadruples_trillions_floor` | `26` | $\lfloor C(5000, 4) / 10^{12} \rfloor$ |

## Claims (non-numeric)

| id | claim as narrated | source | verified how |
|---|---|---|---|
| C1 | Every number in the video is computed from the model and checked against `oracle.py` | Production doctrine (`AGENTS.md`) | `facts::self_check()` and `./x __SLUG__ facts-check` |

## Conventions chosen

- Exact integer binomial coefficients via `explainer_kit::choose(n, k)` in `u128`, verified against `math.comb(n, k)` in `oracle.py`.

## Oracle scripts

- `oracle.py` (crate root) — run with `uv run --no-project --with numpy python3 -I oracle.py`,
  or `./x __SLUG__ facts-check` to diff it against the binary's `facts`.
