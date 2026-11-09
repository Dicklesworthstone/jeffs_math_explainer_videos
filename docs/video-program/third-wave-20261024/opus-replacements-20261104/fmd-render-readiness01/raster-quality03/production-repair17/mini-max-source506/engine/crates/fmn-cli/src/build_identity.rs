// The franken_manim build identity embedded as FMN_BUILD_ID (ADR-0025):
//
//   <FMN_BUILD_ID>                     an explicit identity from the build host
//   git:<commit>                       a clean checkout of <commit>
//   git:<commit>+dirty:<sha256>        <commit> plus uncommitted compiled inputs;
//                                      the digest names that exact state
//   git:<commit>+unverified            git could not report the tree state
//   unidentified:<package>:<version>   neither .git nor FMN_BUILD_ID
//
// A certified manifest must never name a clean commit that the binary was not
// built from, so a dirty tree is identified precisely rather than hidden.
// Certified output refuses the last two forms, which cannot name their sources
// (`fmn_output::provenance::certified_build_refusal`).
//
// This file is `include!`d by the fmn-cli and fmn-python build scripts and
// unit-tested as a module of fmn-cli; keep it free of crate-relative paths and
// of dependencies beyond std and fmn-hash.

/// Repository paths whose content is compiled into, or embedded in, the
/// binaries. Edits elsewhere (docs, beads, scratch) never dirty a build.
const COMPILED_INPUTS: &[&str] = &[
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    "SUITE.lock",
    "rust-toolchain.toml",
    ".cargo",
];

/// The tree state of the checkout relative to its HEAD commit.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TreeState {
    Clean,
    Dirty(String),
    Unverified,
}

/// Compose the identity from its parts; see the table at the top of the file.
fn compose_build_identity(
    explicit: Option<&str>,
    commit: Option<&str>,
    state: &TreeState,
    package: &str,
    version: &str,
) -> String {
    if let Some(value) = explicit.filter(|value| valid_identity(value)) {
        return value.to_owned();
    }
    let Some(commit) = commit else {
        return format!("unidentified:{package}:{version}");
    };
    match state {
        TreeState::Clean => format!("git:{commit}"),
        TreeState::Dirty(digest) => format!("git:{commit}+dirty:{digest}"),
        TreeState::Unverified => format!("git:{commit}+unverified"),
    }
}

/// The identity of the checkout at `repo`, printing the `rerun-if-changed`
/// lines that keep it current when `emit_rerun` is set (build scripts only).
fn build_identity(
    repo: &std::path::Path,
    explicit: Option<&str>,
    package: &str,
    version: &str,
    emit_rerun: bool,
) -> String {
    if let Some(value) = explicit.filter(|value| valid_identity(value)) {
        return value.to_owned();
    }
    let commit = git_commit(repo, emit_rerun);
    let state = match commit {
        Some(_) => {
            if emit_rerun {
                for input in COMPILED_INPUTS {
                    let path = repo.join(input);
                    if path.exists() {
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                }
            }
            tree_state(repo)
        }
        None => TreeState::Unverified,
    };
    compose_build_identity(None, commit.as_deref(), &state, package, version)
}

/// Clean when `git status` lists no change to a compiled input. Otherwise a
/// SHA-256 over every compiled input that differs from HEAD, staged or not,
/// tracked or not: its path, then its content or a deletion marker, in sorted
/// path order. The digest depends on what would be compiled, not on git's
/// index state.
fn tree_state(repo: &std::path::Path) -> TreeState {
    let Some(status) = run_git(
        repo,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    ) else {
        return TreeState::Unverified;
    };
    if status.is_empty() {
        return TreeState::Clean;
    }
    let (Some(changed), Some(untracked)) = (
        run_git(repo, &["diff", "--name-only", "-z", "--no-renames", "HEAD"]),
        run_git(repo, &["ls-files", "-z", "--others", "--exclude-standard"]),
    ) else {
        return TreeState::Unverified;
    };
    let mut paths: Vec<&[u8]> = changed
        .split(|byte| *byte == 0)
        .chain(untracked.split(|byte| *byte == 0))
        .filter(|path| !path.is_empty())
        .collect();
    paths.sort_unstable();
    paths.dedup();
    let mut hasher = fmn_hash::Sha256::new();
    hasher.update(b"fmn-build-dirty-v1\0");
    for path in paths {
        let Ok(name) = std::str::from_utf8(path) else {
            return TreeState::Unverified;
        };
        hasher.update(&(path.len() as u64).to_le_bytes());
        hasher.update(path);
        match std::fs::read(repo.join(name)) {
            Ok(content) => {
                hasher.update(b"F");
                hasher.update(&(content.len() as u64).to_le_bytes());
                hasher.update(&content);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => hasher.update(b"D"),
            Err(_) => return TreeState::Unverified,
        }
    }
    TreeState::Dirty(hasher.finalize().to_hex())
}

/// `git -C <repo> <args> -- <compiled inputs>`; `None` when git is missing
/// or fails. Optional locks are off: a build must never take index.lock
/// from under a concurrent git operation in the same working tree.
fn run_git(repo: &std::path::Path, args: &[&str]) -> Option<Vec<u8>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .arg("--")
        .args(COMPILED_INPUTS)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

fn git_commit(repo: &std::path::Path, emit_rerun: bool) -> Option<String> {
    let git_dir = resolve_git_dir(repo)?;
    let head_path = git_dir.join("HEAD");
    let index_path = git_dir.join("index");
    if emit_rerun {
        println!("cargo:rerun-if-changed={}", head_path.display());
        println!("cargo:rerun-if-changed={}", index_path.display());
    }
    let head = std::fs::read_to_string(&head_path).ok()?;
    let head = head.trim();
    let hash = if let Some(reference) = head.strip_prefix("ref: ") {
        let reference_path = git_dir.join(reference);
        if emit_rerun {
            println!("cargo:rerun-if-changed={}", reference_path.display());
        }
        std::fs::read_to_string(&reference_path)
            .ok()
            .map(|value| value.trim().to_owned())
            .or_else(|| packed_ref(&git_dir, reference, emit_rerun))?
    } else {
        head.to_owned()
    };
    valid_git_hash(&hash).then_some(hash)
}

fn resolve_git_dir(repo: &std::path::Path) -> Option<std::path::PathBuf> {
    let dot_git = repo.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let marker = std::fs::read_to_string(dot_git).ok()?;
    let target = marker.trim().strip_prefix("gitdir: ")?;
    let path = std::path::PathBuf::from(target);
    Some(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

fn packed_ref(git_dir: &std::path::Path, reference: &str, emit_rerun: bool) -> Option<String> {
    let path = git_dir.join("packed-refs");
    if emit_rerun {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    std::fs::read_to_string(path)
        .ok()?
        .lines()
        .find_map(|line| {
            let (hash, name) = line.split_once(' ')?;
            (name == reference && valid_git_hash(hash)).then(|| hash.to_owned())
        })
}

fn valid_git_hash(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'\'' | b'"' | b'\\'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    struct Repo(std::path::PathBuf);

    impl Repo {
        fn new(tag: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fmn-build-identity-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::create_dir_all(root.join("crates/demo/src")).unwrap();
            std::fs::create_dir_all(root.join("docs")).unwrap();
            let repo = Self(root);
            repo.git(&["init", "-q"]);
            repo.write("crates/demo/src/lib.rs", "pub fn one() -> u32 { 1 }\n");
            repo.write("Cargo.toml", "[workspace]\n");
            repo.write("docs/notes.md", "notes\n");
            repo.git(&["add", "-A"]);
            repo.git(&["commit", "-q", "-m", "initial"]);
            repo
        }

        fn git(&self, args: &[&str]) {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args([
                    "-c",
                    "user.name=fmn",
                    "-c",
                    "user.email=fmn@example.invalid",
                ])
                .args(["-c", "commit.gpgsign=false"])
                .args(args)
                .status()
                .unwrap();
            assert!(status.success(), "git {args:?}");
        }

        fn write(&self, path: &str, text: &str) {
            std::fs::write(self.0.join(path), text).unwrap();
        }

        fn identity(&self) -> String {
            build_identity(&self.0, None, "fmn-cli", "0.4.0", false)
        }
    }

    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn composition_covers_every_form() {
        let compose = |explicit, commit, state: &TreeState| {
            compose_build_identity(explicit, commit, state, "fmn-cli", "0.4.0")
        };
        assert_eq!(
            compose(None, Some(COMMIT), &TreeState::Clean),
            format!("git:{COMMIT}")
        );
        assert_eq!(
            compose(None, Some(COMMIT), &TreeState::Dirty("ab".into())),
            format!("git:{COMMIT}+dirty:ab")
        );
        assert_eq!(
            compose(None, Some(COMMIT), &TreeState::Unverified),
            format!("git:{COMMIT}+unverified")
        );
        assert_eq!(
            compose(None, None, &TreeState::Clean),
            "unidentified:fmn-cli:0.4.0"
        );
        assert_eq!(
            compose(Some("release:v1"), None, &TreeState::Clean),
            "release:v1"
        );
        // An explicit identity must be printable and quote-free to be used.
        assert_eq!(
            compose(Some("bad id"), None, &TreeState::Clean),
            "unidentified:fmn-cli:0.4.0"
        );
    }

    #[test]
    fn a_checkout_is_identified_clean_or_by_its_exact_uncommitted_state() {
        let repo = Repo::new("states");
        let clean = repo.identity();
        assert!(clean.starts_with("git:") && !clean.contains('+'), "{clean}");

        repo.write("crates/demo/src/lib.rs", "pub fn one() -> u32 { 2 }\n");
        let dirty = repo.identity();
        assert!(dirty.starts_with(&format!("{clean}+dirty:")), "{dirty}");
        assert_eq!(
            repo.identity(),
            dirty,
            "the digest names the state, deterministically"
        );

        repo.write("crates/demo/src/lib.rs", "pub fn one() -> u32 { 3 }\n");
        let other = repo.identity();
        assert_ne!(
            other, dirty,
            "a different uncommitted state has a different identity"
        );

        repo.write("crates/demo/src/extra.rs", "pub fn two() {}\n");
        let untracked = repo.identity();
        assert_ne!(
            untracked, other,
            "untracked compiled inputs are part of the state"
        );

        repo.git(&["add", "-A"]);
        assert_eq!(
            repo.identity(),
            untracked,
            "staging does not change the compiled state"
        );
        repo.git(&["commit", "-q", "-m", "second"]);
        let committed = repo.identity();
        assert!(
            committed.starts_with("git:") && !committed.contains('+'),
            "{committed}"
        );
        assert_ne!(committed, clean, "a new commit is a new identity");
    }

    #[test]
    fn edits_outside_compiled_inputs_never_dirty_a_build() {
        let repo = Repo::new("scope");
        let clean = repo.identity();
        repo.write("docs/notes.md", "edited notes\n");
        repo.write("docs/new.md", "untracked docs\n");
        assert_eq!(repo.identity(), clean);
    }

    #[test]
    fn a_tree_without_git_metadata_is_unidentified() {
        let root =
            std::env::temp_dir().join(format!("fmn-build-identity-bare-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        assert_eq!(
            build_identity(&root, None, "fmn-cli", "0.4.0", false),
            "unidentified:fmn-cli:0.4.0"
        );
        assert_eq!(
            build_identity(
                &root,
                Some("release:fmn-cli:0.4.0"),
                "fmn-cli",
                "0.4.0",
                false
            ),
            "release:fmn-cli:0.4.0"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
