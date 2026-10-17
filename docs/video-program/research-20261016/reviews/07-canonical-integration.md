# Canonical catalog and native Beads integration review

Independent bounded review, 2026-10-16. Consumer: the parent orchestrator and owner accepting the local catalog/backlog handoff. This report reviews data, documentation and scheduling semantics; it is not narration, render, publication or cross-machine runtime qualification. Archive it when this handoff is superseded.

## Outcome

No structural blocker found in the integrated graph or catalog. The actual native tasks match the reviewed dependency specification, all catalog work anchors resolve, and historical publication records retain their cut identities and bounded evidence. The handoff has reached a useful bounded steady state. Further schema/gate automation belongs to the existing A2–A6 work rather than another planning layer.

One small wording correction was sent to the parent: native **A5** (`mx-video-program-program-87do.1.5`) correctly names `docs/video-program/publication-audit-20261015.json`, but its validation sentence still calls that path “relative to this research workspace.” It is relative to the repository root. Correct that qualification so an isolated worker does not look under the research subdirectory. The target file exists and its records match the catalog. The portable research document's `../publication-audit-20261015.json` is already correct in its own location. This is not a graph or publication blocker.

## Independently checked

Read the research governance and the canonical `docs/video-program/README.md`, `catalog.json`, `catalog.schema.json`, `BEADS_INDEX.md`, selected canonical plan/task sections, the publishing manifest, publication audit, and research task/map/evidence snapshots. Native queries used `--no-auto-import --no-auto-flush`; no direct database reads, Beads edits, repairs, synthesis or browser actions were performed.

- Native `br list --all --deferred --limit 0 --json` returned **327 issues**. All **94** mapped new IDs resolve. The **233** imported issue list records compare equal to the saved post-import snapshot across every available list-record field, including status and dependency/dependent counts. That comparison does not independently re-export comments and full edge objects from the source host. The parent's fuller source-export comparison is separately recorded in `evidence/clone-reconciliation.json` with no original-issue differences.
- Queried all 94 new issues through native `br show`. All **35 implementation tasks** have the exact expected blocking prerequisites: **38 static blocking edges**. Titles, priorities, acceptance fields and task bodies match the specifications, except the intentional portable evidence-path substitution in A5 noted above. Parent-child grouping edges are distinguished from blocking edges.
- All **53 candidate epics** remain deferred. N8 and N13–N16 remain deferred; S6 remains deferred until a chosen brief is attached. DUEL is blocked and explicitly records unavailable independent Claude output, not fabricated cross-scores. A1's expected transition to closed occurred during this review and is not treated as drift.
- Native `br ready` returns exactly **A2, N1 and S1** among the new tasks. It also returns the existing Video K epic, whose historical status was intentionally preserved. The new catalog does not infer that a public upload closes every existing venue/revision task.
- Native `br dep cycles` reports zero active cycles. A separate traversal of the actual 38-edge task graph also finds no cycle. Ten activation cases remain acyclic: each of S2/S3/S4/S5/S7 selected for S6, with and without recovery `N3 -> N8 -> N13`; the optional qualified-detector edge `N6 -> N12` was included in each case. Here arrows mean “requires.” No unchosen topic brief is a mandatory prerequisite.
- Catalog counts independently reproduce **64 stable video identities, 37 cuts, 17 upload records, 11 public records and 53 unapproved candidates**. All 64 `work_epic` anchors resolve to native epics, and candidate anchors match `beads-map.json`. All related-video references resolve.
- Every publication refers to a revision under the same video. All 17 venue/external-ID pairs are unique and have consistent YouTube URLs. Revision IDs are unique **within their video**, as specified in the canonical plan; repeated local cut names across different videos are not global-key collisions.
- The 11 public IDs, URLs and recorded master MD5 values match `publication-audit-20261015.json`. Publication event dates match the manifest: the first four on October 12; mental math, fair coin and matrix exponential on October 14; G/H/I/K on October 15. `recorded_as_of` remains a distinct observation date.
- All catalog local source/evidence references and all Markdown local links in the canonical README and Beads index resolve. Every native issue ID printed by those two documents resolves. Digest algorithm/length checks pass for recorded complete hashes; unknown and partial hashes are not promoted to complete digests.

The system Python used for this audit lacks `jsonschema`; I did not claim an independent JSON Schema engine run. The parsing, cross-record checks, digest checks and link/anchor checks above were independently executed. The schema itself was read as the deliberately limited v1 structure, not mistaken for the planned richer artifact verifier.

## Conditional and authority semantics

The integrated tasks preserve the important distinction between **completing an evaluation** and **qualifying a production method**. N7 can close with a negative result, but N3 and N11 require a positive exact-input decision. An acceptable unchanged baseline may qualify without a fabricated improvement claim. Activated recovery adds the real producing task as a blocker and issues a new decision receipt rather than overwriting the earlier negative evaluation. N13's short screen cannot itself qualify production. N16 remains a decision task, not automatic training authority. N12 and speculative sidecars do not block the cheap N5/N7 listening experiment.

S6's native prerequisites expressly require owner selection and one qualified brief before releasing its deferred status. Candidate inclusion and shortlisting do not confer production approval. P1 still requires both the chosen storyboard and positive N11 qualification. P4/P5 retain the publisher-of-record restriction, distinct private-upload/public-release authority and exact-cut owner approval; historical grants for existing videos do not become permission for a new pilot.

These conditions are currently explicit task contracts and manual producer checks. A2–A6 will implement stronger receipt and transition enforcement. The README accurately says a catalog edit or native task closure alone cannot establish artifact quality. No claim of already implemented automatic prevention is warranted.

The independent second-family duel and complete competitor viewing remain unfinished requirements with honestly stated limits. S1 can perform source and coverage research now; its ready status is not evidence that those missing inputs or final editorial qualification are complete.

## Preserved limitations

Four documented historical master paths are absent in the copied checkout: RaptorQ's `final_review2d` and `final_review2_93da69d`, Lost Submarine's `final_v1_fbe959c`, and Matrix Exponential's `final_v1_ab6c734`. These paths match historical manifest statements and are already enumerated in `evidence/catalog-handoff-validation.json`. Their catalog records retain `historical_record` and do-not-upload dispositions. I did not reconstruct, remove or claim to hash those missing files. The catalog explicitly disclaims fresh local media verification; the missing files do not invalidate the recorded public cuts.

The parent reports that br emitted a poisoned-WAL-index quarantine warning after bv queries while preserving the main database/WAL, and that doctor exits 1 for retained import-vacuum recovery artifacts despite healthy integrity checks. The native read commands in this review succeeded, and the original list records remained equal. This supports the bounded graph/data result, **not** a perfectly clean tooling claim. Recovery artifacts should remain preserved; the planned A6 runtime qualification and the handoff should carry that observation. I performed no repair.

No current YouTube page was opened and no master was decoded or listened to. Historical publication checks, source plausibility and license analysis remain distinct from a measured improvement in Michael's narration.

## Snapshot identifiers

SHA-256 at review:

| Input | SHA-256 |
|---|---|
| Canonical README | `5142f994288df69439d31f426a975e8a330002a70ae834ddec63bc166f3274fd` |
| Canonical catalog | `a5b5092942d225cc6ff5d8b6dc1d65841435a2f852f32142680c2b9db83012c5` |
| Catalog v1 schema | `097856d5d2fad16462558530b94a7f52c17f6d64f88284b32ce0653bb83a05be` |
| Canonical Beads index | `a06b7b41801f146dff15aa0baf1301cae63ac9cf356fbb2e874f759ee666c34d` |
| Research task specifications | `c30da59907a9433e11847e965aba39aa5b5e7291b1b1a9fc943ed2d00bccc95b` |
| Research native-ID map | `48851be1005725ee02f8c2a9718667e9b2635456b73eee58393c6d118ed49233` |

Only this assigned review file was written. No canonical catalog, source, Beads, memory or production artifact was modified by this reviewer.
