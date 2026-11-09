# Graph-bound agent claim guards

`agent_next.py` deterministically selects the best currently claimable Beads leaf. Selection alone is insufficient for concurrent agents: the graph or the selected task's scope can change after an agent reads the plan but before it executes `br update`. `scripts/agent_claim_guard.py` binds the recommendation to the complete exported task semantics and provides an explicit compare-before-set boundary around that interval.

The guard never mutates Beads and is not a lease. It proves only that the claim input and recommendation are still the same at revalidation. File reservations, current assignees, current `main`, Agent Mail, and coordinating messages must still be checked immediately before mutation.

For mutation, use `scripts/agent_claim.py`. It revalidates the guard, executes Beads' atomic claim operation, flushes JSONL, and verifies the postcondition under one advisory lock in Git's shared common directory. Its complete contract is in [`AGENT_CLAIM_EXECUTOR.md`](AGENT_CLAIM_EXECUTOR.md).

## Canonical workflow

```bash
# 1. Validate the graph and capture the exact guarded recommendation.
token="$(python3 scripts/agent_claim_guard.py --require)"
issue="${token##*:}"

# 2. Inspect the selected task and external coordination state.
br show "$issue"
# Check Agent Mail, file reservations, active peers, and current main here.

# 3. Optionally inspect the exact intended mutation without changing Beads.
python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --dry-run

# 4. Revalidate, mutate, flush, and verify under one shared local lock.
python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --transition-comment "Claimed after graph, reservation, and HEAD checks"
```

A successful guard comparison prints the current recommendation according to `--format`. Every nonzero guard result emits no stdout payload, so shell automation cannot accidentally consume a failed plan. The executor similarly emits a success receipt only after its complete transaction succeeds.

## Versioned contract

The public token spelling and guard envelope remain version 2:

```text
v2:<64-lowercase-hex-claim-digest>:<issue-id-or-none>
```

The data bound by that digest advanced independently:

| Layer | Version | Purpose |
|---|---:|---|
| Guard envelope | `fmn.agent.claim-guard` v2 | Stable JSON/token presentation. |
| Claim input | `fmn.agent.claim-input` v3 | Graph, complete plan, policy, and schema contract. |
| Claim graph | `fmn.agent.claim-graph` v3 | Canonical core fields plus complete exported task semantics. |
| Task semantics | `fmn.agent.task-semantics` v1 | Fields outside `agent_brief.Issue` and dependency-record metadata. |
| Planner | `fmn.agent.next` v4 | Governed leaf selection and activation policy. |

An older v2 token cannot accidentally validate against claim-graph v3: the claim-input and graph versions participate in the digest even though the shell-safe token syntax is unchanged.

## What the claim digest binds

The digest covers:

- the strict `agent_brief` snapshot schema version;
- the `agent_next` schema name and version;
- the claim-guard, claim-input, claim-graph, and token schema contracts;
- normalized `as_of`, stale-day policy, activation cap, and queue limit;
- the complete canonical `fmn.agent.next` plan, including integrity, activation, queue evidence, and recommendation;
- every issue's core planning fields:
  - ID;
  - title;
  - status;
  - priority;
  - issue type;
  - assignee;
  - normalized update timestamp;
  - dependency identities and kinds;
  - comments, including their complete exported objects and ledger order;
- every other top-level field present in the exported Beads row, including descriptions, design, acceptance criteria, notes, owners, estimates, creation/source metadata, due/defer values, labels, and unknown future extension fields;
- every non-core field present on dependency records, including metadata, thread identity, creation metadata, and unknown future dependency extensions;
- the selected recommendation identity, redundantly encoded as the token subject for shell-safe extraction.

This is a semantic projection of the exported JSONL authority, not a raw-byte hash. Harmless representation ordering is normalized:

- issue-row order is irrelevant;
- dependency-array order is irrelevant;
- JSON object key order is irrelevant at every depth;
- label order is irrelevant, while duplicate labels remain represented;
- comment order and ordinary array order remain significant.

The literal issue ID `none` is reserved because `none` denotes a valid graph with no claimable recommendation.

## Stable-source proof

`scripts/agent_task_semantics.py` does not trust two unrelated reads to describe one graph. Its guarded loader:

1. opens the JSONL authority as a no-follow regular file under the existing ledger-size limit;
2. reads and strictly decodes every row, producing both the full semantic projection and an `agent_brief.Issue` projection from the same bytes;
3. invokes the established bounded `agent_brief` loader;
4. repeats the full semantic/core read;
5. requires identical before/after content digests and projections;
6. requires the established loader's issue map to equal the core projection derived from the stable bytes.

This catches mutation during loading, including the case where semantic fields change while the broad planning projection would otherwise remain identical. Per-record JSON depth and node ceilings bound unknown nested metadata before planning.

## Post-claim semantic invariant

The executor already computes `after_graph_sha256` through `agent_claim_guard.graph_digest`. Claim-graph v3 makes that step a semantic postcondition as well as a digest calculation.

For the exact ledger path guarded before mutation, the after-export read must preserve every `fmn.agent.task-semantics` value for every issue. A description, acceptance criterion, label, estimate, dependency metadata value, or unknown extension change on either the selected issue or an unrelated issue fails the transaction proof.

The separately verified core delta still permits only the intended claim transition:

```text
selected issue:
  status      open -> in_progress
  assignee    null -> requested assignee
  updated_at  may advance, never regress
  comments    unchanged, or exactly one requested transition comment appended

all other represented core fields: unchanged
all exported task-semantic fields: unchanged
all other issues and represented membership: unchanged
```

The semantic baseline is context-local and scoped to the exact ledger path. A guard built for one fixture, worktree, or repository cannot poison verification for another path.

## Strictness and boundedness

The task-semantic reader inherits the repository's 32 MiB ledger, per-line, issue-count, and dependency-count contracts and additionally enforces:

- no-follow regular-file admission;
- stable descriptor identity and size during each read;
- strict UTF-8 JSONL with one final LF per row;
- duplicate-key and duplicate-ID refusal;
- rejection of `NaN`, positive infinity, and negative infinity anywhere, including future extension fields;
- rejection of unpaired Unicode surrogates;
- maximum decoded depth 64 per issue row;
- maximum 100,000 decoded JSON nodes per issue row;
- canonical JSON encoding with finite values only.

The guard report remains capped at 1 MiB and uses canonical compact UTF-8 with sorted keys and one terminal LF.

```bash
python3 scripts/agent_claim_guard.py --format json
python3 scripts/agent_claim_guard.py --format token --require
python3 scripts/agent_claim_guard.py --format id --expect-token "$token" --require
```

## Guard exit codes

| Exit | Meaning | Safe automation response |
|---:|---|---|
| `0` | Current claim input matches, and any requested recommendation exists. | Continue immediately to the checked mutation. |
| `1` | Blocking/containment integrity or activation state is unsafe. | Repair governance state; do not claim. |
| `2` | Ledger, arguments, token, canonicalization, semantic projection, or output contract is malformed. | Repair the input or invocation; do not claim. |
| `3` | The graph is valid but `--require` found no recommendation. | Stop or repair task scope/coordination. |
| `4` | The supplied token is stale. | Discard it, refresh the plan, and repeat every coordination check. |

Integrity and activation failures take precedence over token parsing. A malformed token therefore cannot obscure a newly unsafe graph.

## What the guard does not prove

The guard and executor bind every field exported in `.beads/issues.jsonl`; they do not prove fields that Beads never exports, raw database-page identity, or insignificant JSON formatting. They also cannot turn the local executor lock into a distributed lease.

Another clone, a direct manual `br` invocation, Agent Mail, reservations, or unrelated changes to `main` can still conflict. Use external coordination, inspect `br show "$issue"`, and keep the interval between those checks and executor invocation minimal. A failed mutation is a fresh conflict; never override it or reuse the old token blindly.

The guard itself does not edit, close, assign, export, or commit issues. The executor performs only the guarded `open` → `in_progress` claim path through `br`, followed by `br sync --flush-only` and parsed-ledger verification. Committing the resulting `.beads/` export remains an explicit repository action.

## Verification

`scripts/test_agent_claim_guard.py` covers v2 token round trips, graph and recommendation changes, canonical order independence, every policy input, schema-version changes, unchanged-recommendation planner drift, the reserved sentinel, no-work behavior, malformed and legacy tokens, output bounds, and integrity precedence.

`scripts/test_agent_task_semantics.py` covers:

- token invalidation for every major task-semantic field and unknown nested metadata;
- dependency metadata binding;
- harmless issue, dependency, label, and object-key order normalization;
- permitted claim-core transitions with invariant semantic fields;
- selected and unrelated semantic-drift refusal;
- depth limits;
- mutation between projections;
- disagreement between the established core loader and the core projection derived from the same semantic read.

`scripts/test_agent_claim.py` covers the shared-lock mutation path and post-export failure semantics. `scripts/test_agent_brief_strict_json.py` ratchets claim-graph/input version 3 and the strict JSON grammar. `scripts/check.sh` compiles and runs all focused suites, issues a token against the complete live ledger, immediately revalidates it, and exercises the executor's live-ledger dry-run path before entering the Rust gates.
