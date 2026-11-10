# Complete frozen-source comparison845

Independent Codex BlackThrush. Read-only SSH source hashing, no remote files created, no media or whole video corpus hashed.

All4125 engine/repo-view files named by760's frozen manifest exist under CSD work/tree and match exactly:47038054bytes, zero missing or mismatched files. This resolves the engine-source equality uncertainty from843 for those named files.

All70 workspace source files were also freshly hashed under CSD work/review-repair313:1964518bytes,67 equal and3 different. They are Cargo.toml (Linux absolute dependency paths), boundary.rs (old310 versus exact reviewed752), and narration_units.json (live313 runtime map versus frozen build plan). The earlier843 metadata-only comparison saw only two differences because313's historical source-copy manifest predates its runtime plan rebind; actual-file hashing correctly finds the third.

Cargo.lock is already byte-identical on both hosts, SHA6a080058520a17788a934bc9eea7e7794e091651f46a1b89827f8c4ff105d9f1. Do not rewrite it. engine-budget.toml is identical after replacing only the Mac engine prefix with CSD work/tree/engine. CSD work/tree/.cargo/config.toml is absent. Fresh844 must preserve760's exact config79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716, including -Zthreads=4, content fingerprint and checksum-freshness, in an ancestor Cargo actually reads. Audit any other ancestor/CARGO_HOME config separately.

For build-source equality use Q/root-boundary760-source/workspace/explainer_corpus/complex_analysis_visual/narration_units.json, checked SHA995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a. That frozen plan maps production19. The transferred live M/source760 plan is now0885eeb6… and maps review764; CSD313 live plan9308c2b2… maps review313. Do not use the live mutated plan as purported frozen761 build input. Runtime narration can receive a separately preserved, source-bound rebind after the fresh build.

Exact mismatch hashes and paths are in JSON. No unknown engine delta was found among named inputs. Proceeding to prepare fresh844 source is supported;235tests, fade probe, Clippy, facts, schedule/PCM comparisons and actual platform visuals remain required. No build or audiovisual acceptance is claimed.
