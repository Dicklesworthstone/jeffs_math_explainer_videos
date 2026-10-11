# AGENTS.md — jeffs_math_explainer_videos

> Guidelines for AI coding agents working in this repository: a monorepo of
> narrated 4K math explainer videos built with franken_manim and FrankenTTS.

---

## RULE 0 — THE FUNDAMENTAL OVERRIDE PREROGATIVE

If I tell you to do something, even if it goes against what follows below, YOU MUST LISTEN TO ME. I AM IN CHARGE, NOT YOU.

---

## RULE NUMBER 1: NO FILE DELETION

**YOU ARE NEVER ALLOWED TO DELETE A FILE WITHOUT EXPRESS PERMISSION.** Even a new file that you yourself created, such as a test code file, a draft render or a narration take. You have a horrible track record of deleting critically important files or otherwise throwing away tons of expensive work. As a result, you have permanently lost any and all rights to determine that a file or folder should be deleted.

**YOU MUST ALWAYS ASK AND RECEIVE CLEAR, WRITTEN PERMISSION BEFORE EVER DELETING A FILE OR FOLDER OF ANY KIND.** Move things aside instead (`previous/`, the driver's never-overwrite naming); narration takes, renders and score stems are expensive to regenerate.

---

## Irreversible Git & Filesystem Actions — DO NOT EVER BREAK GLASS

1. **Absolutely forbidden commands:** `git reset --hard`, `git clean -fd`, `rm -rf`, or any command that can delete or overwrite code/data must never be run unless the user explicitly provides the exact command and states, in the same message, that they understand and want the irreversible consequences.
2. **No guessing:** If there is any uncertainty about what a command might delete or overwrite, stop immediately and ask the user for specific approval. "I think it's safe" is never acceptable.
3. **Safer alternatives first:** When cleanup or rollbacks are needed, request permission to use non-destructive options (`git status`, `git diff`, `git stash`, copying to backups) before ever considering a destructive command.
4. **Mandatory explicit plan:** Even after explicit user authorization, restate the command verbatim, list exactly what will be affected, and wait for a confirmation that your understanding is correct. Only then may you execute it.
5. **Document the confirmation:** When running any approved destructive command, record (in the session notes / final response) the exact user text that authorized it, the command actually run, and the execution time.

---

## Branch Policy

- Primary branch is `main`.
- Do not reference `master` in docs/scripts.

---

## Project Mission

This repository makes narrated, 3Blue1Brown-style 4K math explainer videos that explain deep mathematics so clearly that a curious 14-year-old follows them, a working programmer could implement the idea afterwards, and a professional mathematician finds nothing incorrect or misleading. Never by dumbing down: by the right picture, the right tiny example, the right order, and formulas that are earned.

Every video is a Rust program written against **franken_manim**'s native `fmn` facade (native TeX, no LaTeX, no Python), narrated by **FrankenTTS** (`ftts`; one preset voice per video, the owner's choice: `matt` for capture_recapture, `leo` for raptorq_gf256, `michael` for bakery_algorithm, `robert` for hoeffding_d) with every take scored by **FrankenWhisper** (`fw`), and scored by this repository's own procedural sound engine. Every number is computed and oracle-checked; every frame is looked at; nothing is announced until it passes its gate.

**The single source of truth for what we are building and why is [`docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md`](docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md)** (revision 2). Read §0, §1, §3, §4 and the section for the video you are working on before touching it. The research behind every number is in `docs/research/`; review round 1 of the plan is in `docs/reviews/plan_r1/`.

### What we stand on

- **franken_manim** (github.com/Dicklesworthstone/franken_manim): the animation engine. Consumed as a git dependency pinned by revision; never edited from here (see below).
- **FrankenTTS** (`ftts`) and **FrankenWhisper** (`fw`): the voice and the ears.
- **The three explainer skills** under `~/.claude/skills/` (franken-manim-mega-skill; math-explainer-videos-end-to-end-with-franken-manim-frankentts; natural-sounding-narration-with-frankentts): the engine reference, the production phases and template, and the narration pipeline. They are referenced by path (`SKILLS` in `project.env`), never vendored or edited from this repository; other sessions own their publishing.

---

## Repository Shape

- `explainer_corpus/<slug>/`: one Rust crate per video (its script, lexicon, oracle, docs, chapters). `hoeffding_d` is the first entry; every crate here is a member of the root workspace (one lock, one franken_manim compile).
- `crates/explainer_kit` and `crates/explainer_score` (§3 of the plan): the shared scene kit and the sound engine. Every explainer uses them instead of copying code.
- `docs/`: the plan, the research memos and scripts, and review records.
- `.beads/`: the work graph. Work is claimed and closed there.

---

## The Production Doctrine (READ THIS BEFORE TOUCHING A VIDEO)

These come from the explainer skills and from the incidents that produced them. Violating any of them is a revert.

1. **Never time-stretch, pitch-shift or loudnorm narration.** Pace comes from wording, read choice and pause length. The picture waits for the voice.
2. **Every number on screen or in the narration comes from the explainer's `facts`**, computed by the scene from the data it draws and pinned against an independent oracle (`./x <slug> facts-check`). A typed number is a defect.
3. **Every always/never sentence is tested at its edges before it is voiced.**
4. **Every name and term is looked up and its pronunciation verified** (lookup, respell, probe against a control and a decoy, lock into `lexicon.json` with sources) before narration. The owner's pronunciation overrides every source.
5. **One TTS call per sentence.** A different read needs different text, never a new seed (ftts is greedy-deterministic).
6. **Look at every frame:** the layout lint, contact sheets and cue sheets are read as images before a draft counts as reviewed.
7. **4K60 finals with explicit encoder quality and one encoder for every chapter and segment** (a camera-route segment goes through the same runner-wrapped encoder as the retained route, or the stream-copy join breaks).
8. **Chapters stay ≤ ~85 s** (editorial pacing; the kit raises the engine's byte caps).
9. **Music never fights the voice:** the bed sits 18 LU under speech, melody only in gaps, no sustained music inside the narrator's pitch band during speech (95–250 Hz for `robert`; measured afresh from each video's own narration), the duck baked into the stem offline; never use franken_manim's `gain_to_background`.
10. **`/de-slopify` the final script, then re-apply the voice rules.**
11. **Verify before announcing:** `verify_video.sh` with the EXPECT_* gates (3840×2160, 60 fps, BT.709, −16 LUFS ±1, ≤ −1.0 dBTP, A/V ≤ 0.2 s), contact sheet read, md5 checked after every copy.

### Never change franken_manim from here

Work around engine gaps in this repository and record them (§14 of the plan). Filing beads in franken_manim's tracker is outward-facing work in another repository and needs the owner's go-ahead. The shared franken_manim working tree holds other agents' uncommitted work: a render that used `--fmn-local` or a dirty tree is never a final (G8 refuses it).

---

## Working in the Swarm (the build host)

The three videos are built in parallel by an NTM swarm on the build host: Claude Code lanes write chapters, narration and music; Codex lanes review. The orchestrator assigns lanes through beads.

- **Your own worktree.** Each agent works only in its NTM worktree (branch `ntm/<session>/<agent>`), never in the base checkout, and builds into its own `CARGO_TARGET_DIR` (set in your pane's environment).
- **Landing work.** Commit on your branch with the bead id in the message; `git fetch origin && git rebase origin/main`; re-run your crate's checks (fmt, clippy, tests, facts-check) on the rebased tree; `git push origin HEAD:main`. If the push is rejected, fetch, rebase and re-check again. Never force-push `main`. Land small and often: a chapter that compiles, passes its checks and has been looked at is a landing; a half-written one stays on your branch.
- **Shared files** (a video's `chapters/mod.rs`, `facts.rs`, `model.rs`, `oracle.py`, `glyphs`, the storyboard; anything in `crates/`, `tools/`, `x`) change additively: add your function, register your chapter, pin your facts. Reserve them through Agent Mail while you edit them and land those edits quickly. Never rewrite another lane's chapter; send its owner a message.
- **Beads** live in the base checkout: `BEADS_DB` in your environment points there. Mark your bead `in_progress`, comment with evidence (the commit, the commands run and their results, the sheets read). The orchestrator closes beads.
- **Agent Mail** project key: the base checkout's absolute path (`$JMEV_BASE`). Thread id = the bead id. Check your inbox when you finish a unit of work.
- **Shared outputs** that other lanes consume (narration takes and line WAVs, score stems) live under `$JMEV_SHARED/<slug>/` on the build host, outside every worktree: `narration/out/` is what `--narration` points at, `score/` is what `--score` points at. Renders stay in your worktree's gitignored `renders/`, where reviewers read them by absolute path.
- **Reviews.** A Codex reviewer per video reads every landing for its video: correctness against the oracle, the storyboard and the pedagogy rules, the contact and cue sheets, the narration QA and the score QA. Findings arrive by Agent Mail, ranked by severity; fix them (or answer with evidence why not) and land the fix. Reviewers report defects; they do not rewrite your chapter.

---

## Pedagogy Rules (from §5 of the plan; binding on scripts and storyboards)

- A situation before an abstraction; the hook is an honest failure of the obvious approach on honest data.
- Preview the payoff early; end on the rigorous form of the opening picture.
- Show, then name. Numbers before letters. One word per idea (each brief lists its vocabulary).
- One or two prediction pauses per chapter. Dev asides labelled "for programmers".
- Keep the exact version of every simplification (a "precisely:" lower-third or the description).
- Label measured numbers with their sample sizes and model numbers as "model".
- What is said equals what is shown, rounded the same way.
- Quotes are verbatim, with their source; historical claims are checked.

---

## Public-Repository Hygiene

This repository is public.

- Committed files never name private hosts (write "the GPU host" or use `REMOTE_HOST`) and never contain absolute home paths, user names or machine-specific scratch paths.
- Never commit downloaded third-party documents (papers, book scans, RFC text dumps, web pages). Research scripts take such paths as arguments.
- Never commit renders, audio, narration takes, ASR databases or score stems (see `.gitignore`). Finished cuts ship as GitHub Release assets, and only with the owner's approval.
- `project.env` (hosts, paths) is per-machine and ignored.

---

## Code Editing Discipline

### No Script-Based Changes
**NEVER** run a script that mass-edits code files. Brittle regex transforms create more problems than they solve. Make code changes manually (use parallel subagents for many simple changes; do subtle/complex changes methodically yourself).

### No File Proliferation
Revise existing files in place. **NEVER** create `kitV2.rs` / `score_improved.rs` / `chapter_new.rs`. New files are reserved for genuinely new functionality; the bar is high.

### Backwards Compatibility
We are in early development with **no users**. Do things the **RIGHT** way with **NO TECH DEBT**. Never create compatibility shims or wrappers for deprecated APIs; fix the code directly. One standing exception: a finished video's published cut stays reproducible from its tagged commit.

---

## Honest Credit

- Real code and real tests in the same unit of work. No faked tests, no fixtures presented as live proof, no weakened assertions, no regenerated goldens to force green, no hard-coded success paths.
- Never claim a command you did not run, a render you did not look at, or a number you did not compute. A checked "nothing to report" is a valid result.
- Process artifacts (ledgers, dashboards, matrices, meta-reports) need a named consumer, the gate they enforce, the defect class that justifies them, and a retirement condition. Otherwise do not create them. The deliverable is the video and the code that makes it.
- Close a bead only with evidence bound to a revision; genuinely incomplete work stays open with a comment.

---

## Toolchain

- Rust 2024 edition on the **exact pinned nightly** franken_manim uses (`nightly-2026-08-31`, in each `rust-toolchain.toml`); bump together with franken_manim.
- `fmn` is a git dependency pinned by revision (`rev = …`); upgrading it is a deliberate commit that re-runs every explainer's tests and glyph probes and re-reads fresh contact sheets.
- **The build host** (the owner's big Mac, `REMOTE_HOST` in project.env) is where builds, renders, narration, scoring and the agent swarm run (the owner, 2026-10-11). There cargo runs locally: `RCH_SHIM_LOCAL_IDE=1`, `RUSTUP_TOOLCHAIN=nightly-2026-08-31` and a `CARGO_TARGET_DIR` of your own (never shared with another agent: cargo's lock serializes you, and another tree's half-written file is not your build).
- Nothing compiles on the owner's laptop: there cargo goes to the rch worker fleet. Set `RUSTUP_TOOLCHAIN=nightly-2026-08-31` (rch cannot resolve the client toolchain without it) and run `rch exec --base HEAD --clean-overlay -o <each changed file> -- cargo …`, so the gitignored renders are never synced. Never set `RCH_SHIM_LOCAL_IDE` and never call cargo by absolute path. Every resolve prints `error: unclosed table` for an asupersync test fixture; cargo skips it, and so should you.
- Python tools and research scripts run with `python3 -I`; numpy where a script says so (use `uv run --with numpy` or a virtualenv; never install into the system Python).

---

## Mandatory Checks After Substantive Changes

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
ubs $(git diff --name-only)
```

Run them in each crate you changed (and across the workspace once it exists). For an explainer, also run its `facts-check` and read the glyph probe after any change to on-screen strings. If any check fails, fix root causes before handing off.

---

## Beads (br) — Dependency-Aware Issue Tracking

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`). Issues live in `.beads/` and are tracked in git. **`br` is non-invasive: it NEVER runs git.** After `br sync --flush-only`, manually `git add .beads/ && git commit`.

```bash
br ready                 # dependency-ready work
br show <id>             # read the whole bead before starting
br update <id> --status in_progress
br close <id> --reason "what landed, the commit, and the evidence"
br create --title="..." --type=task|bug|feature|epic --priority=2   # 0=critical..4=backlog
br dep add <issue> <depends-on>
br dep cycles            # must stay empty
br sync --flush-only     # export to JSONL (no git ops)
```

Conventions: include the bead ID in commit messages; use it as the Agent-Mail `thread_id`; put it in file-reservation reasons.

## bv — Graph-Aware Analysis

`bv` computes PageRank/betweenness/critical-path/cycles over the beads. **Use ONLY `--robot-*` flags; bare `bv` launches a blocking TUI.** Useful: `bv --robot-triage`, `bv --robot-next`, `bv --robot-plan`, `bv --robot-insights`.

---

## Session Completion ("Landing the Plane")

Before finishing a work session:
1. File beads for remaining work.
2. Run the quality gates (if code changed).
3. Update bead status: close finished work with evidence, comment on in-progress work.
4. `br sync --flush-only`, then `git add .beads/`.
5. Commit (when asked) and hand off: what changed, the exact commit, the gates actually run and their results, remaining risks, and the next ready bead.

---

## MCP Agent Mail — Multi-Agent Coordination

A mail-like layer for agents to coordinate: identities, inbox/outbox, threads, advisory file reservations.

- **Register identity:** `ensure_project(project_key=<abs-path>)` → `register_agent(project_key, program, model)`.
- **Reserve files before editing:** `file_reservation_paths(project_key, agent_name, ["explainer_corpus/capture_recapture/**"], ttl_seconds=3600, exclusive=true, reason="mx-###")`.
- **Communicate with threads:** `send_message(..., thread_id="mx-###")`, `fetch_inbox`, `acknowledge_message`.

---

## UBS — Ultimate Bug Scanner

`ubs <changed-files>` before every commit. Exit 0 = safe; exit >0 = fix and re-run.

## RCH — Remote Compilation Helper

RCH offloads `cargo build/test/clippy` to remote workers. Usually transparent through the Claude Code hook. Manual: `rch exec -- cargo build --release`. Health: `rch doctor`.

## cass — Cross-Agent Session Search

`cass` indexes prior agent conversations. **Never run bare `cass` (TUI)**; always `--robot` or `--json`: `cass search "fixed gain master" --robot --limit 5`.

---

## Note for Codex/GPT agents — unexpected working-tree changes

If `git status` shows edits you did not make, those are from **other agents working on this project concurrently**, a normal occurrence. **NEVER** stash, revert, or overwrite another agent's work. Treat those changes exactly as if you made them yourself.

## Note on Built-in TODO Functionality

If I explicitly ask you to use your built-in TODO functionality, do so without complaining that you need to use beads. Always comply with such orders.

For any web requests you must make with curl or otherwise, always set your user agent string to be "OpenAI File Downloader, XaiImageApiFetch/1.0"
