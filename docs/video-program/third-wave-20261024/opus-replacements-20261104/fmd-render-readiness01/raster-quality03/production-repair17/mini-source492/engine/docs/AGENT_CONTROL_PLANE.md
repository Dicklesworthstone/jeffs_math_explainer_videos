# Agent control plane

This document defines the operational layer that lets an agent reconstruct current work, select one governed leaf, bind that choice to exact planner state, and claim it through Beads' atomic compare-and-set without creating a second task database.

## Authority hierarchy

1. **`.beads/issues.jsonl`** is the authoritative exported task graph after `br sync --flush-only`.
2. **`scripts/agent_brief.py`** is the bounded, read-only broad situational projection.
3. **`scripts/agent_next.py`** is the fail-closed autonomous planner and the sole authority for leaf eligibility, workstream scope, and activation count.
4. **`scripts/agent_claim_guard.py`** binds the canonical planning graph, complete planner output, policy, and schema contracts into one version-2 token.
5. **`scripts/agent_claim.py`** revalidates that token, invokes Beads' atomic `update --claim`, validates its structured response, explicitly exports JSONL, and verifies the permitted post-export delta.
6. **`scripts/generate_agent_brief.py`** renders deterministic Markdown from the same planner contract while retaining the broad queues as non-claimable context.
7. A committed `docs/AGENT_BRIEF.md` is only a cache of one exported ledger state. It never outranks Beads.

Source authority, compatibility rulings, task authority, and evidence remain separate:

- source code says what behavior exists;
- API schema and overlay say what compatibility status is claimed;
- Beads says what work is open, active, blocked, or closed;
- gates and retained artifacts say what was actually exercised.

## Governed workstream vocabulary

Autonomous work is scoped only when the issue title begins with one of these exact governance prefixes:

```text
G0
W1 through W11
```

The prefix must be followed by a word boundary or `:`. Examples such as `G0: spike`, `W7: drawings`, and `W5/fm-4wt: SIMD` are scoped. `W0`, `W12`, `W999`, lower-case `w10`, and a mid-title `W10` are `UNSCOPED`.

This rule is part of `fmn.agent.next` schema version 4 and therefore participates in every claim token through the planner schema contract.

- An open unscoped leaf remains visible in diagnostics but is never an autonomous recommendation.
- When only unscoped leaves remain, `agent_next.py --require` exits `3` with no stdout payload.
- Any active unscoped issue makes planner integrity invalid. Planner, guard, executor, and generated-brief publication then fail closed with exit `1`.
- `G0` counts as a real active workstream. `G0` plus four W-streams is a five-stream cap breach.

The broad `agent_brief.py` projection is intentionally not the scope authority. The planner recomputes governed scope from issue titles, and the generated human brief normalizes queue labels and activation state to that planner result before rendering.

## Safe session start and claim

```bash
python3 scripts/agent_brief.py --format json --check >/dev/null
python3 scripts/agent_next.py --format json --check

token="$(python3 scripts/agent_claim_guard.py --require)"
issue="${token##*:}"

br show "$issue"
# Check current main, Agent Mail, file reservations, and active peers.

python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --command-output-budget-bytes 16777216 \
    --transition-comment "Claimed after graph, reservation, and HEAD checks" \
    --dry-run

python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --command-output-budget-bytes 16777216 \
    --transition-comment "Claimed after graph, reservation, and HEAD checks"
```

A recommendation and token are not a lease. The executor serializes cooperating worktrees in one clone; current HEAD, another clone, Agent Mail, reservations, and direct manual Beads activity remain outside that lock.

## Claimability rules

An issue is recommendation-eligible only when all of the following hold:

- status is `open`;
- it has one governed workstream prefix;
- every `blocks` target exists and is closed or tombstoned;
- it has no assignee;
- it is not an epic;
- it has no live issue whose `parent-child` edge points to it;
- neither the live blocking graph nor live containment graph has a cycle;
- there is no active unscoped issue;
- claiming it does not violate the four-workstream activation cap.

A task, bug, or feature with live children is a container regardless of its type label.

Recommendation order is deterministic:

1. leaves in an already-active governed workstream;
2. lower numeric priority;
3. more dependents for which completion removes the sole unresolved blocker;
4. more direct blocked dependents;
5. most recently updated;
6. lexical issue ID.

Unscoped leaves never enter that ranking.

## Machine contracts

### Broad situational projection

`agent_brief.py --format json` emits snapshot schema version 5. It owns strict bounded JSONL parsing, blocking-graph integrity, stale and unowned claim diagnostics, and broad dependency-ready queues. It has no autonomous claim output; the former `--format next` spelling is a fail-closed refusal.

The broad projection's historical workstream grouping is diagnostic only. `agent_next.py` owns the exact `G0`/`W1`–`W11` scope and activation contract.

### Leaf plan

`agent_next.py --format json` emits schema `fmn.agent.next`, version 4. The canonical compact record includes:

- blocking, containment, and unscoped-active integrity;
- strictly governed activation state;
- one scoped recommendation or `null`;
- scoped and unscoped leaf counts and bounded queues;
- topology-derived containers;
- direct and immediate-unblock pressure.

The default `as_of` is the newest ledger timestamp. Identical parsed state and policy produce identical output. The payload is capped at 4 MiB.

### Claim token

`agent_claim_guard.py` emits schema `fmn.agent.claim-guard`, version 2, and tokens of the form:

```text
v2:<claim-sha256>:<issue-id-or-none>
```

`graph_sha256` identifies the canonical planning graph. `claim_sha256` additionally binds the complete plan, policy, and parser/planner/guard schema contract. Because the planner version is included, a token issued under planner versions 2 or 3 cannot revalidate under planner version 4.

### Atomic claim receipt

`agent_claim.py` emits schema `fmn.agent.claim`, version 6.

The executor invokes:

```text
br update ISSUE --claim --actor ASSIGNEE --json [--transition-comment TEXT]
br sync --flush-only
```

The first command is Beads' storage-level unassigned compare-and-set. A generic `--status in_progress --assignee ...` update is not the autonomous claim path.

Every receipt includes the token, pre-claim digests, planner policy, schema contracts, exact argv, and executor policy. A successful mutating receipt additionally includes normalized atomic-response evidence, the post-export graph digest, and a normalized claim delta.

The atomic response is parsed as strict UTF-8 JSON with duplicate-key and non-finite-number rejection, a maximum structural depth of 64, and a maximum of 100,000 decoded nodes. It must identify exactly one guarded issue with status `in_progress`, the requested assignee, unchanged title and priority, a valid timestamp, and no success-path stderr. Ordinary array output and `{updated, warnings}` envelopes are accepted.

## Canonical graph boundary

The current token and post-export delta deliberately bind the field set represented by `agent_brief.Issue`:

- ID;
- title;
- status;
- priority;
- issue type;
- assignee;
- normalized update timestamp;
- dependencies;
- comments.

The executor verifies exact issue membership, unchanged non-target records in that field set, and only the expected target status, assignee, timestamp, and optional appended transition comment.

Other persisted Beads fields—including description, design, acceptance criteria, notes, owner, estimates, dates, labels, and unknown extension metadata—are not yet part of this canonical planning graph. A successful receipt must not be described as raw-record or whole-database identity. Agents must still inspect `br show` immediately before claiming, and expanding the canonical graph remains a valid hardening task.

## Shared claim-lock contract

The executor resolves a real `.git` directory or bounded linked-worktree marker, then `commondir` when present, and locks persistent `fmn-agent-claim.lock` in the shared common directory. The primary checkout and sibling linked worktrees therefore contend on one inode.

The lock does not cover another clone, direct manual `br`, an agent that ignores the executor, Agent Mail, reservations, or unrelated changes to `main`.

## Command lifetime and output contract

Each Beads child command has independent controls:

- `--command-timeout-seconds`: default 60 seconds, finite positive maximum 3,600;
- `--command-output-budget-bytes`: default 16 MiB combined stdout/stderr, positive integer maximum 1 GiB;
- retained diagnostics: at most 1 MiB per stream.

stdout and stderr are drained concurrently. Every produced chunk is charged to the shared budget before retained surplus is discarded. Exact-bound output succeeds; the first byte above the total limit triggers bounded process-tree cleanup and exit `5`.

POSIX children run in a new session and cleanup targets the process group. Windows children run in a new process group and use best-effort `taskkill.exe /T /F` plus direct-child fallback. Reader joins and process waits are bounded. A descendant retaining inherited pipes cannot strand the shared claim lock indefinitely.

The timeout and output budget are per command, not transaction-wide.

## Failure semantics

Executor exit `5` means **no verified receipt**, not necessarily **no mutation**. `br update --claim` may commit before malformed output, timeout, export failure, or postcondition failure. Recovery starts with:

```bash
br show "$issue"
br sync --status
git status
```

Never replay the old token blindly.

Other exits:

- `1`: graph, unscoped-active, or activation refusal;
- `2`: malformed input or resource policy;
- `3`: no governed recommendation;
- `4`: stale token or issue mismatch.

Every failure emits no success receipt on stdout.

## Deterministic human rendering

```bash
python3 scripts/generate_agent_brief.py --stdout
python3 scripts/generate_agent_brief.py
python3 scripts/generate_agent_brief.py --check
```

The generator consumes planner version 4 before publication. It:

- counts `G0` and `W1`–`W11` through the planner's exact activation model;
- refuses unscoped active claims and cap breaches before emitting or replacing output;
- keeps unscoped open leaves visible but non-claimable;
- suppresses an unscoped broad priority rather than presenting it as actionable;
- normalizes broad queue labels to the planner's governed vocabulary;
- uses ledger time and atomic descriptor-safe publication.

## Mutation protocol

The brief, planner, guard, and generator are read-only. Use the atomic executor for autonomous claim transitions and tracker-native Beads commands for every other mutation:

```bash
br show <id>
br update <id> ...
br close <id> --reason "..."
br sync --flush-only
git add .beads/
git commit -m "chore(beads): ..."
```

Never reconstruct the large ledger from truncated API output. If a tracker-native `br` environment is unavailable, leave `.beads/issues.jsonl` unchanged and record that limitation.

## Verification policy

`scripts/check.sh` compiles and runs the strict parser, planner, claim-token, atomic executor, real-process lifetime/output, generator, publication-I/O, portal-refusal, and namespace suites before the Rust, Python, WASM, and structural checks. The updated planner, guard, and generator regressions are already part of that existing gate; no hosted workflow is required.

Hosted GitHub Actions is not part of the authority chain. Correctness evidence comes from the exact local or owned-host gate and the named commit it exercised.

## Handoff

Record exact commits, changed paths, commands actually run, source commit identity, remaining evidence gaps, current graph state, and any `.beads/` export still requiring a commit. Never describe an unrun platform or a different commit as current evidence.
