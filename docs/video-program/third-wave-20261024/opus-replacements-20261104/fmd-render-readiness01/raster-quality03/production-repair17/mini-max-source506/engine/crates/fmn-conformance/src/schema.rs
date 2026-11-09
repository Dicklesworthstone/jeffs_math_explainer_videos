//! The one API schema — the single source of API truth (§16.2, fm-vn6).
//!
//! One machine-readable description of the surface (classes, methods,
//! parameters and their defaults, CLI flags, config keys) from which the
//! generated artifacts are produced: `fmn-config`'s typed extraction, the
//! Parity Ledger's rows (§16.1), the CLI flag table, and the docs. **Drift
//! between the front doors is a build error** — the generators are pure
//! functions of the schema, and `tests/api_schema.rs` regenerates every
//! artifact in-memory and fails on any byte of difference.
//!
//! # Two layers, deliberately kept apart
//!
//! * **Extracted** (`API_SCHEMA.tsv`) — mechanically derived from the pinned
//!   Reference by `scripts/gen_api_schema.py`; regenerable, never
//!   hand-edited. Regenerating it must never silently discard a ruling.
//! * **Authored** (`API_OVERLAY.tsv`) — C-9's canonical names, the §16.1
//!   semantic tiers, and the Rust bindings the generators need. Maintained by
//!   hand. Authoring a ruling must never require re-deriving 2276 rows of
//!   Reference surface.
//!
//! [`Schema::parse`] merges them into the *effective* schema, which is the
//! only thing generators ever see.
//!
//! # Why a section-tagged TSV
//!
//! The same reason `SUITE.lock` is one: the auditor of the closure cannot
//! itself expand the closure. A schema that needed a YAML or JSON crate to
//! read would put a dependency underneath the artifact that governs
//! dependencies. Sections are `[name]` on their own line; rows are
//! tab-separated; `#` comments and blank lines are skipped everywhere.

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// A schema file that could not be read as one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// A whole schema document exceeded a resource or encoding boundary.
    Document {
        /// The file carrying the invalid document.
        file: &'static str,
        /// The bounded reason for refusal.
        detail: String,
    },
    /// A line did not use the canonical sectioned-TSV syntax.
    Syntax {
        /// The file carrying the invalid line.
        file: &'static str,
        /// 1-based line number.
        line: usize,
        /// The bounded reason for refusal.
        detail: String,
    },
    /// A row carried the wrong number of tab-separated fields.
    Arity {
        /// The file the row came from.
        file: &'static str,
        /// 1-based line number.
        line: usize,
        /// The section the row was in.
        section: String,
        /// Fields the section requires.
        expected: usize,
        /// Fields the row actually had.
        found: usize,
    },
    /// A row appeared before any `[section]` header.
    Sectionless {
        /// The file the row came from.
        file: &'static str,
        /// 1-based line number.
        line: usize,
    },
    /// Two rows or headers declared the same semantic identity.
    DuplicateIdentity {
        /// File containing the duplicate.
        file: &'static str,
        /// 1-based line of the duplicate.
        line: usize,
        /// Section containing the duplicate.
        section: &'static str,
        /// Identity that was repeated.
        key: String,
        /// File containing the first declaration.
        previous_file: &'static str,
        /// 1-based line of the first declaration.
        previous_line: usize,
    },
    /// A field that must parse as a number or an enumerated word did not.
    Field {
        /// The file the row came from.
        file: &'static str,
        /// 1-based line number.
        line: usize,
        /// Which column.
        column: &'static str,
        /// What was there.
        found: String,
    },
    /// The overlay named a symbol, parameter, or config key the extracted
    /// layer does not have — the overlay has gone stale against the pin.
    DanglingOverlay {
        /// Which overlay section.
        section: &'static str,
        /// The key that resolved to nothing.
        key: String,
    },
    /// A config key exists with no binding, or a binding with no key.
    ConfigCoverage {
        /// Human-readable statement of the mismatch.
        detail: String,
    },
    /// The authored CLI layer does not cover the extracted flag surface
    /// exactly once.
    FlagCoverage {
        /// Human-readable statement of the mismatch.
        detail: String,
    },
    /// The authored CLI contract refers to an unknown or duplicate identity.
    CliContract {
        /// Human-readable statement of the invalid contract.
        detail: String,
    },
    /// A semantic ruling does not carry evidence of the kind its status
    /// requires, or names an absent out-of-tier entry.
    EvidenceContract {
        /// Human-readable statement of the invalid ruling.
        detail: String,
    },
    /// The reviewed-identity checksum changed without the authored ratchet
    /// authority changing with it.
    ReviewRatchet {
        /// Digest committed in `API_OVERLAY.tsv [meta]`.
        expected: String,
        /// Digest of the effective reviewed ledger rows.
        actual: String,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document { file, detail } => write!(f, "{file}: {detail}"),
            Self::Syntax { file, line, detail } => write!(f, "{file}:{line}: {detail}"),
            Self::Arity {
                file,
                line,
                section,
                expected,
                found,
            } => write!(
                f,
                "{file}:{line}: [{section}] row has {found} fields, expected {expected}"
            ),
            Self::Sectionless { file, line } => {
                write!(f, "{file}:{line}: row before any [section] header")
            }
            Self::DuplicateIdentity {
                file,
                line,
                section,
                key,
                previous_file,
                previous_line,
            } => write!(
                f,
                "{file}:{line}: [{section}] identity {key:?} duplicates {previous_file}:{previous_line}"
            ),
            Self::Field {
                file,
                line,
                column,
                found,
            } => write!(f, "{file}:{line}: bad `{column}` field: {found:?}"),
            Self::DanglingOverlay { section, key } => write!(
                f,
                "API_OVERLAY.tsv [{section}] names `{key}`, which API_SCHEMA.tsv does not \
                 contain — the overlay is stale against the Reference pin; \
                 rerun scripts/gen_api_schema.py and reconcile"
            ),
            Self::ConfigCoverage { detail } => write!(f, "config-key coverage: {detail}"),
            Self::FlagCoverage { detail } => write!(f, "CLI flag coverage: {detail}"),
            Self::CliContract { detail } => write!(f, "CLI contract: {detail}"),
            Self::EvidenceContract { detail } => write!(f, "ledger evidence: {detail}"),
            Self::ReviewRatchet { expected, actual } => write!(
                f,
                "reviewed-ledger digest is {actual}, but API_OVERLAY.tsv records {expected}; \
                 update the digest only with the written semantic amendment that explains the change"
            ),
        }
    }
}

impl std::error::Error for SchemaError {}

// ---------------------------------------------------------------------------
// The sectioned-TSV reader
// ---------------------------------------------------------------------------

/// One parsed row, kept with its provenance so every later failure can name
/// the line it came from.
#[derive(Debug, Clone)]
struct Row {
    line: usize,
    fields: Vec<String>,
}

/// Sections in file order, each a list of rows.
#[derive(Debug, Default)]
struct Sections {
    file: &'static str,
    map: BTreeMap<String, Vec<Row>>,
}

const MAX_SCHEMA_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_SCHEMA_LINE_BYTES: usize = 16 * 1024;
const MAX_SCHEMA_ROWS: usize = 100_000;
const MAX_SCHEMA_SECTION_BYTES: usize = 64;
const MAX_SCHEMA_FIELDS_PER_ROW: usize = 16;
const EXTRACTED_SECTIONS: &[&str] = &["meta", "symbols", "params", "flags", "config"];
const EXTRACTED_META_KEYS: &[&str] = &[
    "schema_version",
    "reference_commit",
    "generator",
    "wildcard_exports",
];
const OVERLAY_SECTIONS: &[&str] = &[
    "meta",
    "canonical",
    "param_canonical",
    "constants",
    "optional_config",
    "config_binding",
    "config_status",
    "out_of_tier",
    "status",
    "flag_binding",
    "native_flags",
    "subcommands",
    "exit_codes",
    "flag_interaction",
];
const OVERLAY_META_KEYS: &[&str] = &["overlay_version", "ledger_reviewed_digest"];

impl Sections {
    fn parse(
        file: &'static str,
        text: &str,
        allowed_sections: &[&str],
    ) -> Result<Self, SchemaError> {
        if text.len() > MAX_SCHEMA_DOCUMENT_BYTES {
            return Err(SchemaError::Document {
                file,
                detail: format!(
                    "document exceeds the {MAX_SCHEMA_DOCUMENT_BYTES}-byte format limit"
                ),
            });
        }
        if text.as_bytes().contains(&b'\r') {
            return Err(SchemaError::Document {
                file,
                detail: "CR line endings are not canonical".to_owned(),
            });
        }
        if !text.is_empty() && !text.ends_with('\n') {
            return Err(SchemaError::Document {
                file,
                detail: "document must end with a newline".to_owned(),
            });
        }
        let mut out = Self {
            file,
            map: BTreeMap::new(),
        };
        let mut current: Option<String> = None;
        let mut section_lines = BTreeMap::new();
        let mut row_count = 0_usize;
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            if raw.len() > MAX_SCHEMA_LINE_BYTES {
                return Err(SchemaError::Syntax {
                    file,
                    line,
                    detail: format!("line exceeds {MAX_SCHEMA_LINE_BYTES} bytes"),
                });
            }
            if raw.is_empty() {
                continue;
            }
            if raw.trim().is_empty() {
                return Err(SchemaError::Syntax {
                    file,
                    line,
                    detail: "whitespace-only rows are not canonical".to_owned(),
                });
            }
            if raw.starts_with('#') {
                if raw.trim_end() != raw {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: "comment has trailing whitespace".to_owned(),
                    });
                }
                continue;
            }
            if raw.trim() != raw {
                return Err(SchemaError::Syntax {
                    file,
                    line,
                    detail: "surrounding whitespace is not canonical".to_owned(),
                });
            }
            if raw.starts_with('[') {
                let name = raw
                    .strip_prefix('[')
                    .and_then(|value| value.strip_suffix(']'))
                    .ok_or_else(|| SchemaError::Syntax {
                        file,
                        line,
                        detail: "malformed section header".to_owned(),
                    })?;
                if name.is_empty()
                    || name.len() > MAX_SCHEMA_SECTION_BYTES
                    || !name.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' // ubs:ignore -- public schema syntax, not secret data
                    })
                {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: "section name is not canonical lowercase ASCII".to_owned(),
                    });
                }
                if !allowed_sections.contains(&name) {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: format!("unknown section [{name}]"),
                    });
                }
                if let Some(&previous_line) = section_lines.get(name) {
                    return Err(SchemaError::DuplicateIdentity {
                        file,
                        line,
                        section: "section",
                        key: name.to_owned(),
                        previous_file: file,
                        previous_line,
                    });
                }
                current = Some(name.to_owned());
                section_lines.insert(name.to_owned(), line);
                out.map.insert(name.to_owned(), Vec::new());
                continue;
            }
            let Some(section) = current.clone() else {
                return Err(SchemaError::Sectionless { file, line });
            };
            if row_count == MAX_SCHEMA_ROWS {
                return Err(SchemaError::Syntax {
                    file,
                    line,
                    detail: format!("data row count exceeds {MAX_SCHEMA_ROWS}"),
                });
            }
            row_count += 1;
            let mut fields = Vec::new();
            for field in raw.split('\t') {
                if fields.len() == MAX_SCHEMA_FIELDS_PER_ROW {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: format!(
                            "row exceeds {MAX_SCHEMA_FIELDS_PER_ROW} tab-separated fields"
                        ),
                    });
                }
                if field.is_empty() {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: "empty TSV fields must use the `-` placeholder".to_owned(),
                    });
                }
                if field.trim() != field {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: "TSV fields must not have surrounding whitespace".to_owned(),
                    });
                }
                if field.bytes().any(|byte| byte.is_ascii_control()) {
                    return Err(SchemaError::Syntax {
                        file,
                        line,
                        detail: "TSV fields must not contain ASCII control bytes".to_owned(),
                    });
                }
                fields.push(field.to_owned());
            }
            out.map
                .entry(section)
                .or_default()
                .push(Row { line, fields });
        }
        Ok(out)
    }

    fn rows(&self, section: &str) -> &[Row] {
        self.map.get(section).map_or(&[], Vec::as_slice)
    }

    /// Rows of `section`, each checked to have exactly `arity` fields.
    fn typed(&self, section: &str, arity: usize) -> Result<Vec<&Row>, SchemaError> {
        let mut out = Vec::new();
        for row in self.rows(section) {
            if row.fields.len() != arity {
                return Err(SchemaError::Arity {
                    file: self.file,
                    line: row.line,
                    section: section.to_owned(),
                    expected: arity,
                    found: row.fields.len(),
                });
            }
            out.push(row);
        }
        Ok(out)
    }

    /// `key\tvalue` rows of a `[meta]`-shaped section.
    fn meta(
        &self,
        section: &'static str,
        expected_keys: &[&str],
    ) -> Result<BTreeMap<String, String>, SchemaError> {
        let mut values = BTreeMap::new();
        let mut identities = BTreeMap::new();
        for row in self.typed(section, 2)? {
            if !expected_keys.contains(&row.fields[0].as_str()) {
                return Err(SchemaError::Syntax {
                    file: self.file,
                    line: row.line,
                    detail: format!("unknown [{section}] key {:?}", row.fields[0]),
                });
            }
            record_identity(
                &mut identities,
                self.file,
                row.line,
                section,
                row.fields[0].clone(),
            )?;
            values.insert(row.fields[0].clone(), row.fields[1].clone());
        }
        for expected in expected_keys {
            if !values.contains_key(*expected) {
                return Err(SchemaError::Document {
                    file: self.file,
                    detail: format!("[{section}] is missing required key {expected:?}"),
                });
            }
        }
        Ok(values)
    }
}

/// The TSV placeholder for "no value" — chosen over an empty field so a row's
/// arity is visible to the eye and a trailing tab cannot be lost by an editor.
const NONE: &str = "-";

fn opt(field: &str) -> Option<&str> {
    if field == NONE { None } else { Some(field) }
}

fn binary_bool(
    file: &'static str,
    line: usize,
    column: &'static str,
    field: &str,
) -> Result<bool, SchemaError> {
    match field {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(SchemaError::Field {
            file,
            line,
            column,
            found: field.to_owned(),
        }),
    }
}

fn canonical_unsigned<T>(
    file: &'static str,
    line: usize,
    column: &'static str,
    field: &str,
) -> Result<T, SchemaError>
where
    T: std::str::FromStr,
{
    let canonical = field == "0"
        || (!field.starts_with('0') && field.bytes().all(|byte| byte.is_ascii_digit()));
    if !canonical {
        return Err(SchemaError::Field {
            file,
            line,
            column,
            found: field.to_owned(),
        });
    }
    field.parse().map_err(|_| SchemaError::Field {
        file,
        line,
        column,
        found: field.to_owned(),
    })
}

type IdentityLocations = BTreeMap<String, (&'static str, usize)>;

fn record_identity(
    identities: &mut IdentityLocations,
    file: &'static str,
    line: usize,
    section: &'static str,
    key: String,
) -> Result<(), SchemaError> {
    if let Some(&(previous_file, previous_line)) = identities.get(&key) {
        return Err(SchemaError::DuplicateIdentity {
            file,
            line,
            section,
            key,
            previous_file,
            previous_line,
        });
    }
    identities.insert(key, (file, line));
    Ok(())
}

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

/// What a symbol is on the Reference's public surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolKind {
    /// A class.
    Class,
    /// A method on a class (including `__init__`).
    Method,
    /// A `@property` on a class.
    Property,
    /// A class-level attribute — public surface, since the Reference
    /// configures whole lineages by overriding them.
    Attribute,
    /// A module-level function.
    Function,
    /// A module-level constant.
    Constant,
    /// A name the wildcard surface binds that the package never defines —
    /// `np`, `math`, `moderngl`. Real exports, whether or not anyone wanted
    /// them (§1.6).
    LeakedImport,
}

impl SymbolKind {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "class" => Self::Class,
            "method" => Self::Method,
            "property" => Self::Property,
            "attribute" => Self::Attribute,
            "function" => Self::Function,
            "constant" => Self::Constant,
            "leaked_import" => Self::LeakedImport,
            _ => return None,
        })
    }

    /// The word used in the schema file and in generated artifacts.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Method => "method",
            Self::Property => "property",
            Self::Attribute => "attribute",
            Self::Function => "function",
            Self::Constant => "constant",
            Self::LeakedImport => "leaked_import",
        }
    }
}

/// How a parameter may be passed. Reproduced exactly by the Python front
/// door: a Reference parameter that is positional-or-keyword must not become
/// keyword-only in `fmn-python`, or source-unedited scenes break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Before a `/`.
    PositionalOnly,
    /// The ordinary kind.
    PositionalOrKeyword,
    /// `*args`.
    VarPositional,
    /// After a `*`.
    KeywordOnly,
    /// `**kwargs`.
    VarKeyword,
}

impl ParamKind {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "positional_only" => Self::PositionalOnly,
            "positional_or_keyword" => Self::PositionalOrKeyword,
            "var_positional" => Self::VarPositional,
            "keyword_only" => Self::KeywordOnly,
            "var_keyword" => Self::VarKeyword,
            _ => return None,
        })
    }

    /// The word used in the schema file.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PositionalOnly => "positional_only",
            Self::PositionalOrKeyword => "positional_or_keyword",
            Self::VarPositional => "var_positional",
            Self::KeywordOnly => "keyword_only",
            Self::VarKeyword => "var_keyword",
        }
    }
}

/// One symbol of the Reference's surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// Defining module, dotted.
    pub module: String,
    /// `Name`, or `Class.member` for class members.
    pub name: String,
    /// What it is.
    pub kind: SymbolKind,
    /// `defined`, or the module a leaked import came from.
    pub origin: String,
    /// Whether `from manimlib import *` binds this name.
    pub exported: bool,
    /// Bases for a class, default expression for a constant or attribute.
    pub detail: Option<String>,
}

impl Symbol {
    /// `module:name` — the key the overlay addresses symbols by.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}:{}", self.module, self.name)
    }
}

/// One parameter of one callable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    /// `module:Class.method` or `module:function`.
    pub owner: String,
    /// Position in the declaration.
    pub ordinal: u32,
    /// Parameter name as the Reference spells it.
    pub name: String,
    /// How it may be passed.
    pub kind: ParamKind,
    /// Type annotation source text, if any.
    pub annotation: Option<String>,
    /// Default *expression* source text, if any. Never an evaluated value:
    /// most Reference defaults name module constants (`ORIGIN`, `TAU / 4`)
    /// that only exist once the package is imported.
    pub default: Option<String>,
}

/// One `parser.add_argument` of the Reference's CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    /// Comma-joined option strings, e.g. `-w,--write_file`.
    pub options: String,
    /// argparse `dest`, if given explicitly.
    pub dest: Option<String>,
    /// argparse action (`store`, `store_true`, ...).
    pub action: String,
    /// argparse `nargs`, if given.
    pub nargs: Option<String>,
    /// Default expression, if given.
    pub default: Option<String>,
    /// `type=` callable, if given.
    pub ty: Option<String>,
    /// Help text.
    pub help: Option<String>,
}

/// Which `fmn` command accepts a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CliCommand {
    /// Accepted before or after every command name.
    Global,
    /// The default `fmn render` command.
    Render,
    /// `fmn doctor`.
    Doctor,
    /// `fmn batch` (behind the `batch` feature at runtime).
    Batch,
    /// `fmn studio`.
    Studio,
}

impl CliCommand {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "global" => Self::Global,
            "render" => Self::Render,
            "doctor" => Self::Doctor,
            "batch" => Self::Batch,
            "studio" => Self::Studio,
            _ => return None,
        })
    }

    /// Stable schema/generated-code spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Render => "render",
            Self::Doctor => "doctor",
            Self::Batch => "batch",
            Self::Studio => "studio",
        }
    }
}

/// One authored ruling and Rust binding for an extracted Reference flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagBinding {
    /// Exact comma-joined key from `API_SCHEMA.tsv [flags]`.
    pub options: String,
    /// Same/improved/tiered/excluded ruling.
    pub status: Status,
    /// Behavior Note or out-of-tier identity required by the ruling.
    pub evidence: String,
    /// Command accepting the flag.
    pub command: CliCommand,
    /// Stable generated-parser field name.
    pub binding: String,
    /// Executed coverage that owns this reviewed ruling.
    pub tests: String,
    /// User-facing semantic note.
    pub note: String,
}

/// One FrankenManim-native flag absent from the pinned Reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeFlag {
    /// Comma-joined option aliases.
    pub options: String,
    /// Parser action (`store` or `store_true`).
    pub action: String,
    /// Parser arity (`?`, `*`, or absent).
    pub nargs: Option<String>,
    /// Authored default literal, if any.
    pub default: Option<String>,
    /// Stable value-type name, if the flag takes a value.
    pub ty: Option<String>,
    /// Command accepting the flag.
    pub command: CliCommand,
    /// Stable generated-parser field name.
    pub binding: String,
    /// Semantic status relative to the Reference surface.
    pub status: Status,
    /// Behavior Note or out-of-tier identity required by the ruling.
    pub evidence: String,
    /// Executed coverage that owns this reviewed ruling.
    pub tests: String,
    /// Help text.
    pub help: String,
}

/// One native command in the `fmn` command tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcommand {
    /// Command name.
    pub command: CliCommand,
    /// Semantic status relative to the Reference's single command.
    pub status: Status,
    /// Behavior Note or out-of-tier identity required by the ruling.
    pub evidence: String,
    /// Executed coverage that owns this reviewed ruling.
    pub tests: String,
    /// Help-table summary.
    pub help: String,
}

/// One stable process exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitCode {
    /// Numeric process status.
    pub code: u8,
    /// Stable robot-schema identity.
    pub name: String,
    /// User-facing meaning.
    pub meaning: String,
}

/// Executable relationship between authored flag bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagInteractionKind {
    /// No more than one operand may be present.
    AtMostOne,
    /// The two operands cannot coexist.
    Conflicts,
    /// The first operand requires at least one later operand.
    RequiresAny,
    /// Presence of the first operand sets the second.
    Implies,
    /// The operand cannot accompany any other non-global action.
    Exclusive,
}

impl FlagInteractionKind {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "at_most_one" => Self::AtMostOne,
            "conflicts" => Self::Conflicts,
            "requires_any" => Self::RequiresAny,
            "implies" => Self::Implies,
            "exclusive" => Self::Exclusive,
            _ => return None,
        })
    }

    /// Stable schema/generated-code spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AtMostOne => "at_most_one",
            Self::Conflicts => "conflicts",
            Self::RequiresAny => "requires_any",
            Self::Implies => "implies",
            Self::Exclusive => "exclusive",
        }
    }
}

/// One generated parser rule from `[flag_interaction]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagInteraction {
    /// Stable rule identity used by fixtures and diagnostics.
    pub id: String,
    /// Executable rule form.
    pub kind: FlagInteractionKind,
    /// `|`-separated authored binding names.
    pub operands: Vec<String>,
    /// Stable exit-code name, or absent for non-failing implications.
    pub exit_code: Option<String>,
    /// User-facing diagnostic/contract note.
    pub message: String,
}

/// The YAML shape a config value takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// `True` / `False`.
    Bool,
    /// An integer scalar.
    Int,
    /// A floating scalar.
    Float,
    /// A quoted or bare string.
    Str,
    /// A tuple-string like `(1920, 1080)`.
    Tuple,
    /// A parent key. Some parents ARE the key — `colors`, `key_bindings`,
    /// and `directories.subdirs` are open, user-extensible maps bound as a
    /// whole rather than child by child.
    Map,
}

impl ValueKind {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "bool" => Self::Bool,
            "int" => Self::Int,
            "float" => Self::Float,
            "string" => Self::Str,
            "tuple" => Self::Tuple,
            "map" => Self::Map,
            _ => return None,
        })
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::Str => "string",
            Self::Tuple => "tuple",
            Self::Map => "map",
        }
    }
}

/// One config key of the shipped defaults document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigKey {
    /// Dotted path into the document.
    pub path: String,
    /// The value's YAML shape.
    pub kind: ValueKind,
    /// The default literal as written. `None` for an optional key, which has
    /// no default by definition, and for a parent.
    pub default: Option<String>,
    /// Whether the Reference's own `default_config.yml` also has this key.
    pub in_reference: bool,
}

/// A C-9 canonical-name ruling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rename {
    /// `module:Name` of the Reference symbol.
    pub symbol: String,
    /// The name the Rust front door and the docs use.
    pub canonical: String,
    /// The Appendix C row this ruling comes from.
    pub ruling: String,
    /// Free-text note.
    pub note: Option<String>,
}

impl Rename {
    /// The Reference's own spelling — the alias `fmn-python` must also bind.
    #[must_use]
    pub fn reference_name(&self) -> &str {
        self.symbol
            .rsplit_once(':')
            .map_or(self.symbol.as_str(), |(_, name)| name)
            .rsplit_once('.')
            .map_or_else(
                || self.symbol.rsplit_once(':').map_or("", |(_, n)| n),
                |(_, leaf)| leaf,
            )
    }
}

/// A parameter-level canonical-name ruling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamRename {
    /// The callable the parameter belongs to.
    pub owner: String,
    /// The Reference's spelling.
    pub reference_name: String,
    /// The canonical spelling.
    pub canonical: String,
    /// The Appendix C row.
    pub ruling: String,
}

/// A C-16 constant-value ruling: one Reference module constant whose
/// effective value is a governed override of its extracted spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantOverride {
    /// `module:Name` of the Reference constant.
    pub symbol: String,
    /// The spelling the front doors resolve instead of the extracted
    /// default — itself an expression over already-resolved constants.
    pub spelling: String,
    /// The Appendix C row this ruling comes from.
    pub ruling: String,
    /// Free-text note.
    pub note: Option<String>,
}

/// One config key's binding into the Rust config types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Dotted config path.
    pub path: String,
    /// The Rust struct that carries it.
    pub struct_name: String,
    /// The field within that struct.
    pub field: String,
    /// The typed accessor on `Cx` that extracts it.
    pub accessor: String,
}

impl Binding {
    /// The `Config` field holding this key's struct: the path's first
    /// segment for a nested key, the path itself for a top-level one.
    /// Derived rather than stored, so it cannot disagree with the path.
    #[must_use]
    pub fn outer_field(&self) -> &str {
        self.path.split('.').next().unwrap_or(&self.path)
    }

    /// Whether this key lives directly on `Config`.
    #[must_use]
    pub fn is_top_level(&self) -> bool {
        self.struct_name == "Config"
    }
}

/// The §16.1 semantic tier of one symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Same behavior as the Reference.
    Same,
    /// Deliberately better; carries a Behavior Note.
    Improved,
    /// Reduced scope, with a recorded reason.
    Tiered,
    /// Not offered, with a recorded reason.
    Excluded,
    /// Nobody has adjudicated this symbol yet. The honest default, and the
    /// number the Parity Ledger ratchets down.
    Unreviewed,
}

/// Authored review fields attached to one Ledger surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerAnnotation {
    /// Same/improved/tiered/excluded/unreviewed ruling.
    pub status: Status,
    /// Behavior Note, out-of-tier identity, or other authority.
    pub evidence: String,
    /// Executed coverage that owns the ruling.
    pub tests: String,
    /// Concise user-facing semantic statement.
    pub notes: String,
}

/// One §16.6 out-of-tier ruling with the condition that can reopen it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutOfTier {
    /// Stable `OOT-*` identity used from the Parity Ledger.
    pub id: String,
    /// User-facing surface or capability covered by the ruling.
    pub surface: String,
    /// Whether the surface is reduced (`tiered`) or absent (`excluded`).
    pub status: Status,
    /// Why the current boundary is honest and deliberate.
    pub rationale: String,
    /// Concrete evidence or demand that requires reconsideration.
    pub revisit_trigger: String,
}

impl Status {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "same" => Self::Same,
            "improved" => Self::Improved,
            "tiered" => Self::Tiered,
            "excluded" => Self::Excluded,
            "unreviewed" => Self::Unreviewed,
            _ => return None,
        })
    }

    /// The word used in the schema file and the Ledger.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::Improved => "improved",
            Self::Tiered => "tiered",
            Self::Excluded => "excluded",
            Self::Unreviewed => "unreviewed",
        }
    }
}

fn is_behavior_note_id(text: &str) -> bool {
    let Some(number) = text.strip_prefix("BN-") else {
        return false;
    };
    number.len() >= 2
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && number.bytes().any(|byte| byte != b'0')
        && (number.len() == 2 || !number.starts_with('0'))
}

fn is_out_of_tier_id(text: &str) -> bool {
    let Some(name) = text.strip_prefix("OOT-") else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        && name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && name
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && !name.contains("--")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LedgerRow {
    module: String,
    symbol: String,
    canonical: String,
    kind: String,
    exported: bool,
    signature_defaults: String,
    status: Status,
    evidence: String,
    tests: String,
    notes: String,
}

impl LedgerRow {
    fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.module,
            self.symbol,
            self.canonical,
            self.kind,
            u8::from(self.exported),
            self.signature_defaults,
            self.status.as_str(),
            self.evidence,
            self.tests,
            self.notes
        )
    }

    fn reviewed_identity_line(&self) -> Option<String> {
        (self.status != Status::Unreviewed) // ubs:ignore -- public ledger enum, not secret data
            .then(|| format!("{}\n", self.line()))
    }
}

/// The effective schema: the extracted layer with the authored layer applied.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    /// `[meta]` of the extracted layer (schema version, Reference commit).
    pub meta: BTreeMap<String, String>,
    /// Every symbol of the Reference's surface.
    pub symbols: Vec<Symbol>,
    /// Every parameter of every callable.
    pub params: Vec<Param>,
    /// The Reference's CLI flag surface.
    pub flags: Vec<Flag>,
    /// Authored rulings and generated-parser bindings for Reference flags.
    pub flag_bindings: Vec<FlagBinding>,
    /// FrankenManim-native flags.
    pub native_flags: Vec<NativeFlag>,
    /// Native command tree.
    pub subcommands: Vec<Subcommand>,
    /// Stable CLI process statuses.
    pub exit_codes: Vec<ExitCode>,
    /// Executable flag relationships.
    pub flag_interactions: Vec<FlagInteraction>,
    /// The shipped config-key surface.
    pub config: Vec<ConfigKey>,
    /// C-9 canonical-name rulings, by `module:Name`.
    pub renames: BTreeMap<String, Rename>,
    /// C-9 parameter renames.
    pub param_renames: Vec<ParamRename>,
    /// C-16 constant-value rulings, by `module:Name`.
    pub constant_overrides: BTreeMap<String, ConstantOverride>,
    /// Config-key bindings, in emission order.
    pub bindings: Vec<Binding>,
    /// Adjudicated semantic tiers for config keys; absent keys are honestly
    /// unreviewed.
    pub config_statuses: BTreeMap<String, LedgerAnnotation>,
    /// Adjudicated semantic tiers, by `module:Name`.
    pub statuses: BTreeMap<String, LedgerAnnotation>,
    /// The authored §16.6 fringe and its revisit triggers.
    pub out_of_tier: BTreeMap<String, OutOfTier>,
    /// Reviewed-identity digest committed in the overlay metadata.
    pub reviewed_ledger_digest: String,
}

impl Schema {
    /// Parse and merge the two layers.
    ///
    /// # Errors
    /// [`SchemaError`] naming the file and line for a malformed row, or the
    /// dangling key for an overlay that has gone stale against the pin.
    pub fn parse(extracted: &str, overlay: &str) -> Result<Self, SchemaError> {
        let ex = Sections::parse("API_SCHEMA.tsv", extracted, EXTRACTED_SECTIONS)?;
        let ov = Sections::parse("API_OVERLAY.tsv", overlay, OVERLAY_SECTIONS)?;
        let meta = ex.meta("meta", EXTRACTED_META_KEYS)?;
        let overlay_meta = ov.meta("meta", OVERLAY_META_KEYS)?;
        if meta["schema_version"] != "1" {
            return Err(SchemaError::Document {
                file: ex.file,
                detail: format!(
                    "unsupported schema_version {:?}; expected \"1\"",
                    meta["schema_version"]
                ),
            });
        }
        // ubs:ignore -- public format version, not secret data
        if overlay_meta["overlay_version"] != "1" {
            return Err(SchemaError::Document {
                file: ov.file,
                detail: format!(
                    "unsupported overlay_version {:?}; expected \"1\"",
                    overlay_meta["overlay_version"]
                ),
            });
        }
        let reviewed_ledger_digest = &overlay_meta["ledger_reviewed_digest"];
        if reviewed_ledger_digest.len() != 64
            || !reviewed_ledger_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(SchemaError::Document {
                file: ov.file,
                detail:
                    "ledger_reviewed_digest must be an exact 64-character lowercase hex identity"
                        .to_owned(),
            });
        }
        if meta["generator"] != "scripts/gen_api_schema.py" {
            return Err(SchemaError::Document {
                file: ex.file,
                detail: format!(
                    "unexpected generator {:?}; expected \"scripts/gen_api_schema.py\"",
                    meta["generator"]
                ),
            });
        }
        let reference_commit = &meta["reference_commit"];
        if reference_commit.len() != 40
            || !reference_commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(SchemaError::Document {
                file: ex.file,
                detail: "reference_commit must be an exact 40-character lowercase hex identity"
                    .to_owned(),
            });
        }
        let wildcard_exports_text = &meta["wildcard_exports"];
        let wildcard_exports_canonical = wildcard_exports_text == "0"
            || (!wildcard_exports_text.starts_with('0')
                && wildcard_exports_text
                    .bytes()
                    .all(|byte| byte.is_ascii_digit()));
        if !wildcard_exports_canonical {
            return Err(SchemaError::Document {
                file: ex.file,
                detail: "wildcard_exports must be a canonical nonnegative integer".to_owned(),
            });
        }
        let wildcard_exports =
            wildcard_exports_text
                .parse::<usize>()
                .map_err(|_| SchemaError::Document {
                    file: ex.file,
                    detail: "wildcard_exports exceeds the supported integer range".to_owned(),
                })?;
        let mut schema = Self {
            meta,
            reviewed_ledger_digest: reviewed_ledger_digest.clone(),
            ..Self::default()
        };

        for row in ex.typed("symbols", 6)? {
            let f = &row.fields;
            schema.symbols.push(Symbol {
                module: f[0].clone(),
                name: f[1].clone(),
                kind: SymbolKind::parse(&f[2]).ok_or_else(|| SchemaError::Field {
                    file: ex.file,
                    line: row.line,
                    column: "kind",
                    found: f[2].clone(),
                })?,
                origin: f[3].clone(),
                exported: binary_bool(ex.file, row.line, "exported", &f[4])?,
                detail: opt(&f[5]).map(str::to_owned),
            });
        }

        let actual_exports = schema
            .symbols
            .iter()
            .filter(|symbol| symbol.exported)
            .map(|symbol| symbol.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        // ubs:ignore -- public API census, not secret data
        if actual_exports != wildcard_exports {
            return Err(SchemaError::Document {
                file: ex.file,
                detail: format!(
                    "wildcard_exports declares {wildcard_exports}, but [symbols] contains {actual_exports} unique exported names"
                ),
            });
        }

        for row in ex.typed("params", 6)? {
            let f = &row.fields;
            schema.params.push(Param {
                owner: f[0].clone(),
                ordinal: canonical_unsigned(ex.file, row.line, "ordinal", &f[1])?,
                name: f[2].clone(),
                kind: ParamKind::parse(&f[3]).ok_or_else(|| SchemaError::Field {
                    file: ex.file,
                    line: row.line,
                    column: "kind",
                    found: f[3].clone(),
                })?,
                annotation: opt(&f[4]).map(str::to_owned),
                default: opt(&f[5]).map(str::to_owned),
            });
        }

        let mut flag_identities = BTreeMap::new();
        for row in ex.typed("flags", 7)? {
            let f = &row.fields;
            record_identity(
                &mut flag_identities,
                ex.file,
                row.line,
                "flags",
                f[0].clone(),
            )?;
            schema.flags.push(Flag {
                options: f[0].clone(),
                dest: opt(&f[1]).map(str::to_owned),
                action: f[2].clone(),
                nargs: opt(&f[3]).map(str::to_owned),
                default: opt(&f[4]).map(str::to_owned),
                ty: opt(&f[5]).map(str::to_owned),
                help: opt(&f[6]).map(str::to_owned),
            });
        }

        let mut config_identities = BTreeMap::new();
        for row in ex.typed("config", 4)? {
            let f = &row.fields;
            record_identity(
                &mut config_identities,
                ex.file,
                row.line,
                "config",
                f[0].clone(),
            )?;
            schema.config.push(ConfigKey {
                path: f[0].clone(),
                kind: ValueKind::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                    file: ex.file,
                    line: row.line,
                    column: "kind",
                    found: f[1].clone(),
                })?,
                default: opt(&f[2]).map(str::to_owned),
                in_reference: binary_bool(ex.file, row.line, "reference", &f[3])?,
            });
        }

        let symbol_keys: std::collections::BTreeSet<String> =
            schema.symbols.iter().map(Symbol::key).collect();

        let mut canonical_identities = BTreeMap::new();
        for row in ov.typed("canonical", 4)? {
            let f = &row.fields;
            if !symbol_keys.contains(&f[0]) {
                return Err(SchemaError::DanglingOverlay {
                    section: "canonical",
                    key: f[0].clone(),
                });
            }
            record_identity(
                &mut canonical_identities,
                ov.file,
                row.line,
                "canonical",
                f[0].clone(),
            )?;
            schema.renames.insert(
                f[0].clone(),
                Rename {
                    symbol: f[0].clone(),
                    canonical: f[1].clone(),
                    ruling: f[2].clone(),
                    note: opt(&f[3]).map(str::to_owned),
                },
            );
        }

        let mut param_canonical_identities = BTreeMap::new();
        for row in ov.typed("param_canonical", 4)? {
            let f = &row.fields;
            let known = schema
                .params
                .iter()
                .any(|p| p.owner == f[0] && p.name == f[1]);
            if !known {
                return Err(SchemaError::DanglingOverlay {
                    section: "param_canonical",
                    key: format!("{}#{}", f[0], f[1]),
                });
            }
            record_identity(
                &mut param_canonical_identities,
                ov.file,
                row.line,
                "param_canonical",
                format!("{}#{}", f[0], f[1]),
            )?;
            schema.param_renames.push(ParamRename {
                owner: f[0].clone(),
                reference_name: f[1].clone(),
                canonical: f[2].clone(),
                ruling: f[3].clone(),
            });
        }

        let mut constant_identities = BTreeMap::new();
        for row in ov.typed("constants", 4)? {
            let f = &row.fields;
            let symbol = schema
                .symbols
                .iter()
                .find(|s| s.key() == f[0])
                .ok_or_else(|| SchemaError::DanglingOverlay {
                    section: "constants",
                    key: f[0].clone(),
                })?;
            if symbol.kind != SymbolKind::Constant {
                return Err(SchemaError::DanglingOverlay {
                    section: "constants",
                    key: f[0].clone(),
                });
            }
            if f[1].is_empty() {
                return Err(SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "spelling",
                    found: f[1].clone(),
                });
            }
            record_identity(
                &mut constant_identities,
                ov.file,
                row.line,
                "constants",
                f[0].clone(),
            )?;
            schema.constant_overrides.insert(
                f[0].clone(),
                ConstantOverride {
                    symbol: f[0].clone(),
                    spelling: f[1].clone(),
                    ruling: f[2].clone(),
                    note: opt(&f[3]).map(str::to_owned),
                },
            );
        }

        for row in ov.typed("optional_config", 3)? {
            let f = &row.fields;
            record_identity(
                &mut config_identities,
                ov.file,
                row.line,
                "optional_config",
                f[0].clone(),
            )?;
            schema.config.push(ConfigKey {
                path: f[0].clone(),
                kind: ValueKind::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "kind",
                    found: f[1].clone(),
                })?,
                default: None,
                in_reference: binary_bool(ov.file, row.line, "reference", &f[2])?,
            });
        }

        let config_keys: std::collections::BTreeSet<String> =
            schema.config.iter().map(|key| key.path.clone()).collect();

        let mut config_status_identities = BTreeMap::new();
        for row in ov.typed("config_status", 5)? {
            let f = &row.fields;
            if !config_keys.contains(&f[0]) {
                return Err(SchemaError::DanglingOverlay {
                    section: "config_status",
                    key: f[0].clone(),
                });
            }
            record_identity(
                &mut config_status_identities,
                ov.file,
                row.line,
                "config_status",
                f[0].clone(),
            )?;
            let status = Status::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                file: ov.file,
                line: row.line,
                column: "status",
                found: f[1].clone(),
            })?;
            schema.config_statuses.insert(
                f[0].clone(),
                LedgerAnnotation {
                    status,
                    evidence: f[2].clone(),
                    tests: f[3].clone(),
                    notes: f[4].clone(),
                },
            );
        }

        let mut out_of_tier_identities = BTreeMap::new();
        for row in ov.typed("out_of_tier", 5)? {
            let f = &row.fields;
            if !is_out_of_tier_id(&f[0]) {
                return Err(SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "out_of_tier id",
                    found: f[0].clone(),
                });
            }
            record_identity(
                &mut out_of_tier_identities,
                ov.file,
                row.line,
                "out_of_tier",
                f[0].clone(),
            )?;
            let status = Status::parse(&f[2]).ok_or_else(|| SchemaError::Field {
                file: ov.file,
                line: row.line,
                column: "status",
                found: f[2].clone(),
            })?;
            if !matches!(status, Status::Tiered | Status::Excluded) {
                return Err(SchemaError::EvidenceContract {
                    detail: format!(
                        "out-of-tier row `{}` has status `{}`; only tiered or excluded is valid",
                        f[0], f[2]
                    ),
                });
            }
            if f[1] == NONE || f[3] == NONE || f[4] == NONE {
                return Err(SchemaError::EvidenceContract {
                    detail: format!(
                        "out-of-tier row `{}` must name a surface, rationale, and revisit trigger",
                        f[0]
                    ),
                });
            }
            schema.out_of_tier.insert(
                f[0].clone(),
                OutOfTier {
                    id: f[0].clone(),
                    surface: f[1].clone(),
                    status,
                    rationale: f[3].clone(),
                    revisit_trigger: f[4].clone(),
                },
            );
        }

        let mut binding_identities = BTreeMap::new();
        for row in ov.typed("config_binding", 4)? {
            let f = &row.fields;
            record_identity(
                &mut binding_identities,
                ov.file,
                row.line,
                "config_binding",
                f[0].clone(),
            )?;
            schema.bindings.push(Binding {
                path: f[0].clone(),
                struct_name: f[1].clone(),
                field: f[2].clone(),
                accessor: f[3].clone(),
            });
        }

        let mut status_identities = BTreeMap::new();
        for row in ov.typed("status", 5)? {
            let f = &row.fields;
            if !symbol_keys.contains(&f[0]) {
                return Err(SchemaError::DanglingOverlay {
                    section: "status",
                    key: f[0].clone(),
                });
            }
            record_identity(
                &mut status_identities,
                ov.file,
                row.line,
                "status",
                f[0].clone(),
            )?;
            let status = Status::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                file: ov.file,
                line: row.line,
                column: "status",
                found: f[1].clone(),
            })?;
            schema.statuses.insert(
                f[0].clone(),
                LedgerAnnotation {
                    status,
                    evidence: f[2].clone(),
                    tests: f[3].clone(),
                    notes: f[4].clone(),
                },
            );
        }

        for row in ov.typed("flag_binding", 7)? {
            let f = &row.fields;
            schema.flag_bindings.push(FlagBinding {
                options: f[0].clone(),
                status: Status::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "status",
                    found: f[1].clone(),
                })?,
                evidence: f[2].clone(),
                command: CliCommand::parse(&f[3]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "command",
                    found: f[3].clone(),
                })?,
                binding: f[4].clone(),
                tests: f[5].clone(),
                note: f[6].clone(),
            });
        }

        for row in ov.typed("native_flags", 11)? {
            let f = &row.fields;
            schema.native_flags.push(NativeFlag {
                options: f[0].clone(),
                action: f[1].clone(),
                nargs: opt(&f[2]).map(str::to_owned),
                default: opt(&f[3]).map(str::to_owned),
                ty: opt(&f[4]).map(str::to_owned),
                command: CliCommand::parse(&f[5]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "command",
                    found: f[5].clone(),
                })?,
                binding: f[6].clone(),
                status: Status::parse(&f[7]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "status",
                    found: f[7].clone(),
                })?,
                evidence: f[8].clone(),
                tests: f[9].clone(),
                help: f[10].clone(),
            });
        }

        for row in ov.typed("subcommands", 5)? {
            let f = &row.fields;
            let command = CliCommand::parse(&f[0]).ok_or_else(|| SchemaError::Field {
                file: ov.file,
                line: row.line,
                column: "command",
                found: f[0].clone(),
            })?;
            if command == CliCommand::Global {
                return Err(SchemaError::CliContract {
                    detail: "`global` is a flag scope, not a subcommand".to_owned(),
                });
            }
            schema.subcommands.push(Subcommand {
                command,
                status: Status::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "status",
                    found: f[1].clone(),
                })?,
                evidence: f[2].clone(),
                tests: f[3].clone(),
                help: f[4].clone(),
            });
        }

        for row in ov.typed("exit_codes", 3)? {
            let f = &row.fields;
            schema.exit_codes.push(ExitCode {
                code: canonical_unsigned(ov.file, row.line, "code", &f[0])?,
                name: f[1].clone(),
                meaning: f[2].clone(),
            });
        }

        for row in ov.typed("flag_interaction", 5)? {
            let f = &row.fields;
            schema.flag_interactions.push(FlagInteraction {
                id: f[0].clone(),
                kind: FlagInteractionKind::parse(&f[1]).ok_or_else(|| SchemaError::Field {
                    file: ov.file,
                    line: row.line,
                    column: "interaction",
                    found: f[1].clone(),
                })?,
                operands: f[2].split('|').map(str::to_owned).collect(),
                exit_code: opt(&f[3]).map(str::to_owned),
                message: f[4].clone(),
            });
        }

        schema.check_cli_contract()?;
        schema.check_evidence_contract()?;
        let actual = schema.ledger_reviewed_digest();
        // ubs:ignore -- public integrity hash, not secret data
        if actual != schema.reviewed_ledger_digest {
            return Err(SchemaError::ReviewRatchet {
                expected: schema.reviewed_ledger_digest.clone(),
                actual,
            });
        }
        Ok(schema)
    }

    fn check_cli_contract(&self) -> Result<(), SchemaError> {
        let extracted: std::collections::BTreeSet<&str> = self
            .flags
            .iter()
            .map(|flag| flag.options.as_str())
            .collect();
        let mut ruled = std::collections::BTreeSet::new();
        let mut problems = Vec::new();
        for binding in &self.flag_bindings {
            if !extracted.contains(binding.options.as_str()) {
                return Err(SchemaError::DanglingOverlay {
                    section: "flag_binding",
                    key: binding.options.clone(),
                });
            }
            if !ruled.insert(binding.options.as_str()) {
                problems.push(format!("`{}` is ruled more than once", binding.options));
            }
        }
        for option in extracted.difference(&ruled) {
            problems.push(format!("extracted flag `{option}` has no ruling"));
        }
        if !problems.is_empty() {
            return Err(SchemaError::FlagCoverage {
                detail: problems.join("; "),
            });
        }

        let mut aliases = std::collections::BTreeMap::new();
        let mut bindings = std::collections::BTreeSet::new();
        for binding in &self.flag_bindings {
            if !bindings.insert(binding.binding.as_str()) {
                return Err(SchemaError::CliContract {
                    detail: format!("flag binding `{}` is duplicated", binding.binding),
                });
            }
            for alias in binding.options.split(',') {
                if let Some(previous) = aliases.insert(alias, binding.binding.as_str()) {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "option alias `{alias}` belongs to both `{previous}` and `{}`",
                            binding.binding
                        ),
                    });
                }
            }
        }
        for flag in &self.native_flags {
            if !matches!(flag.action.as_str(), "store" | "store_true") {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "native flag `{}` has unsupported action `{}`",
                        flag.options, flag.action
                    ),
                });
            }
            if let Some(ty) = flag.ty.as_deref()
                && !matches!(
                    ty,
                    "int"
                        | "usize"
                        | "u64"
                        | "u16"
                        | "ip"
                        | "output_format"
                        | "engine"
                        | "preview_codec"
                        | "path"
                        | "pack"
                )
            {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "native flag `{}` has unsupported value type `{ty}`",
                        flag.options
                    ),
                });
            }
            if !bindings.insert(flag.binding.as_str()) {
                return Err(SchemaError::CliContract {
                    detail: format!("flag binding `{}` is duplicated", flag.binding),
                });
            }
            for alias in flag.options.split(',') {
                if let Some(previous) = aliases.insert(alias, flag.binding.as_str()) {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "option alias `{alias}` belongs to both `{previous}` and `{}`",
                            flag.binding
                        ),
                    });
                }
            }
        }

        let mut commands = std::collections::BTreeSet::new();
        for subcommand in &self.subcommands {
            if !commands.insert(subcommand.command) {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "subcommand `{}` is declared twice",
                        subcommand.command.as_str()
                    ),
                });
            }
        }
        if !commands.contains(&CliCommand::Render) {
            return Err(SchemaError::CliContract {
                detail: "the default `render` subcommand is missing".to_owned(),
            });
        }

        let mut exit_names = std::collections::BTreeSet::new();
        let mut exit_values = std::collections::BTreeSet::new();
        for exit in &self.exit_codes {
            if !exit_names.insert(exit.name.as_str()) {
                return Err(SchemaError::CliContract {
                    detail: format!("exit-code name `{}` is duplicated", exit.name),
                });
            }
            if !exit_values.insert(exit.code) {
                return Err(SchemaError::CliContract {
                    detail: format!("exit-code value `{}` is duplicated", exit.code),
                });
            }
        }
        if !exit_values.contains(&0) {
            return Err(SchemaError::CliContract {
                detail: "exit code 0 must be declared".to_owned(),
            });
        }

        let mut interaction_ids = std::collections::BTreeSet::new();
        for interaction in &self.flag_interactions {
            if !interaction_ids.insert(interaction.id.as_str()) {
                return Err(SchemaError::CliContract {
                    detail: format!("interaction id `{}` is duplicated", interaction.id),
                });
            }
            if interaction.operands.is_empty() || interaction.operands.iter().any(String::is_empty)
            {
                return Err(SchemaError::CliContract {
                    detail: format!("interaction `{}` has an empty operand", interaction.id),
                });
            }
            for operand in &interaction.operands {
                let (binding, selected_values) = operand
                    .split_once('=')
                    .map_or((operand.as_str(), None), |(binding, values)| {
                        (binding, Some(values))
                    });
                if !bindings.contains(binding) {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "interaction `{}` names unknown binding `{binding}`",
                            interaction.id
                        ),
                    });
                }
                if selected_values.is_some_and(|values| {
                    values.is_empty()
                        || values.contains('=')
                        || values.split(',').any(str::is_empty)
                }) {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "interaction `{}` has an invalid value selector `{operand}`",
                            interaction.id
                        ),
                    });
                }
            }
            match interaction.kind {
                FlagInteractionKind::AtMostOne if interaction.operands.len() < 2 => {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "interaction `{}` needs at least two operands",
                            interaction.id
                        ),
                    });
                }
                FlagInteractionKind::Conflicts | FlagInteractionKind::Implies
                    if interaction.operands.len() != 2 =>
                {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "interaction `{}` needs exactly two operands",
                            interaction.id
                        ),
                    });
                }
                FlagInteractionKind::RequiresAny if interaction.operands.len() < 2 => {
                    return Err(SchemaError::CliContract {
                        detail: format!(
                            "interaction `{}` needs a trigger and requirement",
                            interaction.id
                        ),
                    });
                }
                _ => {}
            }
            if interaction.kind == FlagInteractionKind::Implies
                && interaction.operands[1].contains('=')
            {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "interaction `{}` cannot imply a value selector",
                        interaction.id
                    ),
                });
            }
            if interaction.kind == FlagInteractionKind::Exclusive
                && interaction
                    .operands
                    .iter()
                    .any(|operand| operand.contains('='))
            {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "interaction `{}` cannot use value selectors",
                        interaction.id
                    ),
                });
            }
            if let Some(exit_code) = interaction.exit_code.as_deref()
                && !exit_names.contains(exit_code)
            {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "interaction `{}` names unknown exit code `{exit_code}`",
                        interaction.id
                    ),
                });
            }
            if interaction.kind == FlagInteractionKind::Implies && interaction.exit_code.is_some() {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "non-failing implication `{}` must not carry an exit code",
                        interaction.id
                    ),
                });
            }
            if interaction.kind != FlagInteractionKind::Implies && interaction.exit_code.is_none() {
                return Err(SchemaError::CliContract {
                    detail: format!(
                        "failing interaction `{}` must name an exit code",
                        interaction.id
                    ),
                });
            }
        }
        Ok(())
    }

    /// Every config key is bound exactly once, and every binding names a key.
    ///
    /// This is what makes the generated extraction trustworthy: a key added
    /// to `default_config.yml` without a binding, or a binding left behind by
    /// a removed key, fails here rather than producing code that silently
    /// ignores a knob.
    ///
    /// # Errors
    /// [`SchemaError::ConfigCoverage`] naming every offending path.
    pub fn check_config_coverage(&self) -> Result<(), SchemaError> {
        let keys: std::collections::BTreeSet<&str> =
            self.config.iter().map(|c| c.path.as_str()).collect();
        let mut bound = std::collections::BTreeSet::new();
        let mut problems = Vec::new();

        for binding in &self.bindings {
            if !bound.insert(binding.path.as_str()) {
                problems.push(format!("`{}` is bound twice", binding.path));
            }
        }
        // A map-valued key (`colors`, `key_bindings`, `directories.subdirs`)
        // is bound as a whole, so everything beneath it is covered by it. A
        // parent that is merely structural (`camera`) needs no binding of its
        // own — its children carry the values.
        for key in self.config.iter().filter(|c| c.kind != ValueKind::Map) {
            let path = key.path.as_str();
            let covered =
                bound.contains(path) || bound.iter().any(|b| path.starts_with(&format!("{b}.")));
            if !covered {
                problems.push(format!("config key `{path}` has no binding"));
            }
        }
        for binding in &bound {
            if !keys.contains(binding) {
                problems.push(format!("binding `{binding}` names no config key"));
            }
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(SchemaError::ConfigCoverage {
                detail: problems.join("; "),
            })
        }
    }

    fn check_evidence_contract(&self) -> Result<(), SchemaError> {
        let validate = |identity: &str,
                        status: Status,
                        evidence: &str,
                        tests: &str,
                        notes: &str| {
            match status {
                Status::Improved if !is_behavior_note_id(evidence) => {
                    Err(SchemaError::EvidenceContract {
                        detail: format!(
                            "improved `{identity}` must cite a canonical BN-* identity, found `{evidence}`"
                        ),
                    })
                }
                Status::Tiered | Status::Excluded => {
                    let Some(ruling) = self.out_of_tier.get(evidence) else {
                        return Err(SchemaError::EvidenceContract {
                            detail: format!(
                                "{} `{identity}` must cite a declared OOT-* identity, found `{evidence}`",
                                status.as_str()
                            ),
                        });
                    };
                    if ruling.status != status {
                        return Err(SchemaError::EvidenceContract {
                            detail: format!(
                                "{} `{identity}` cites `{evidence}`, whose out-of-tier status is `{}`",
                                status.as_str(),
                                ruling.status.as_str()
                            ),
                        });
                    }
                    Ok(())
                }
                // ubs:ignore -- public evidence sentinel, not secret data
                Status::Unreviewed if evidence != NONE => Err(SchemaError::EvidenceContract {
                    detail: format!(
                        "unreviewed `{identity}` must carry `{NONE}` evidence, found `{evidence}`"
                    ),
                }),
                _ if evidence.is_empty() => Err(SchemaError::EvidenceContract {
                    detail: format!("`{identity}` has empty evidence"),
                }),
                _ => Ok(()),
            }?;
            // ubs:ignore -- public ledger enum, not secret data
            if status == Status::Unreviewed {
                if tests != NONE || notes != NONE {
                    return Err(SchemaError::EvidenceContract {
                        detail: format!(
                            "unreviewed `{identity}` must carry `{NONE}` tests and notes"
                        ),
                    });
                }
            } else if tests == NONE || notes == NONE {
                return Err(SchemaError::EvidenceContract {
                    detail: format!(
                        "reviewed `{identity}` must name executed tests and a semantic note"
                    ),
                });
            }
            Ok(())
        };

        for (identity, annotation) in &self.statuses {
            validate(
                identity,
                annotation.status,
                &annotation.evidence,
                &annotation.tests,
                &annotation.notes,
            )?;
        }
        for (identity, annotation) in &self.config_statuses {
            validate(
                &format!("config:{identity}"),
                annotation.status,
                &annotation.evidence,
                &annotation.tests,
                &annotation.notes,
            )?;
        }
        for binding in &self.flag_bindings {
            validate(
                &format!("cli-flag:{}", binding.options),
                binding.status,
                &binding.evidence,
                &binding.tests,
                &binding.note,
            )?;
        }
        for flag in &self.native_flags {
            validate(
                &format!("cli-flag:{}", flag.options),
                flag.status,
                &flag.evidence,
                &flag.tests,
                &flag.help,
            )?;
        }
        for command in &self.subcommands {
            validate(
                &format!("cli-command:{}", command.command.as_str()),
                command.status,
                &command.evidence,
                &command.tests,
                &command.help,
            )?;
        }
        Ok(())
    }

    fn symbol_signature_defaults(&self, symbol: &Symbol) -> String {
        let direct_owner = symbol.key();
        let constructor_owner = (symbol.kind == SymbolKind::Class) // ubs:ignore -- public schema kind, not secret data
            .then(|| format!("{direct_owner}.__init__"));
        let mut params = self
            .params
            .iter()
            .filter(|param| {
                param.owner == direct_owner
                    || constructor_owner
                        .as_deref()
                        .is_some_and(|owner| param.owner == owner)
            })
            .collect::<Vec<_>>();
        params.sort_unstable_by_key(|param| param.ordinal);
        if !params.is_empty() {
            let rendered = params
                .into_iter()
                .map(|param| {
                    let prefix = match param.kind {
                        ParamKind::VarPositional => "*",
                        ParamKind::VarKeyword => "**",
                        _ => "",
                    };
                    let mut field = format!("{prefix}{}[{}]", param.name, param.kind.as_str());
                    if let Some(annotation) = &param.annotation {
                        let _ = write!(field, ":{annotation}");
                    }
                    if let Some(default) = &param.default {
                        let _ = write!(field, "={default}");
                    }
                    field
                })
                .collect::<Vec<_>>()
                .join(", ");
            return format!("({rendered})");
        }

        symbol.detail.as_deref().map_or_else(
            || NONE.to_owned(),
            |detail| {
                let label = match symbol.kind {
                    SymbolKind::Class => "bases",
                    SymbolKind::Attribute | SymbolKind::Constant => "default",
                    _ => "detail",
                };
                format!("{label}={detail}")
            },
        )
    }

    fn flag_signature_defaults(
        action: &str,
        nargs: Option<&str>,
        default: Option<&str>,
        value_type: Option<&str>,
    ) -> String {
        let default = default.or_else(|| (action == "store_true").then_some("False"));
        format!(
            "action={action};nargs={};type={};default={}",
            nargs.unwrap_or(NONE),
            value_type.unwrap_or(NONE),
            default.unwrap_or(NONE)
        )
    }

    fn ledger_rows(&self) -> Vec<LedgerRow> {
        let mut rows = Vec::with_capacity(
            self.symbols.len()
                + self.flags.len()
                + self.native_flags.len()
                + self.subcommands.len()
                + self.config.len(),
        );

        rows.extend(self.symbols.iter().map(|symbol| {
            let annotation = self.statuses.get(&symbol.key());
            LedgerRow {
                module: symbol.module.clone(),
                symbol: symbol.name.clone(),
                canonical: self.canonical_name(symbol),
                kind: symbol.kind.as_str().to_owned(),
                exported: symbol.exported,
                signature_defaults: self.symbol_signature_defaults(symbol),
                status: annotation.map_or(Status::Unreviewed, |row| row.status),
                evidence: annotation.map_or_else(|| NONE.to_owned(), |row| row.evidence.clone()),
                tests: annotation.map_or_else(|| NONE.to_owned(), |row| row.tests.clone()),
                notes: annotation.map_or_else(|| NONE.to_owned(), |row| row.notes.clone()),
            }
        }));

        for flag in &self.flags {
            if let Some(binding) = self
                .flag_bindings
                .iter()
                .find(|binding| binding.options == flag.options)
            {
                rows.push(LedgerRow {
                    module: format!("fmn.cli.{}", binding.command.as_str()),
                    symbol: binding.options.clone(),
                    canonical: binding.binding.clone(),
                    kind: "cli_flag".to_owned(),
                    exported: false,
                    signature_defaults: Self::flag_signature_defaults(
                        &flag.action,
                        flag.nargs.as_deref(),
                        flag.default.as_deref(),
                        flag.ty.as_deref(),
                    ),
                    status: binding.status,
                    evidence: binding.evidence.clone(),
                    tests: binding.tests.clone(),
                    notes: binding.note.clone(),
                });
            }
        }
        rows.extend(self.native_flags.iter().map(|flag| LedgerRow {
            module: format!("fmn.cli.{}", flag.command.as_str()),
            symbol: flag.options.clone(),
            canonical: flag.binding.clone(),
            kind: "cli_flag".to_owned(),
            exported: false,
            signature_defaults: Self::flag_signature_defaults(
                &flag.action,
                flag.nargs.as_deref(),
                flag.default.as_deref(),
                flag.ty.as_deref(),
            ),
            status: flag.status,
            evidence: flag.evidence.clone(),
            tests: flag.tests.clone(),
            notes: flag.help.clone(),
        }));
        rows.extend(self.subcommands.iter().map(|command| LedgerRow {
            module: "fmn.cli".to_owned(),
            symbol: command.command.as_str().to_owned(),
            canonical: command.command.as_str().to_owned(),
            kind: "cli_command".to_owned(),
            exported: false,
            signature_defaults: NONE.to_owned(),
            status: command.status,
            evidence: command.evidence.clone(),
            tests: command.tests.clone(),
            notes: command.help.clone(),
        }));
        rows.extend(self.config.iter().map(|key| {
            let annotation = self.config_statuses.get(&key.path);
            LedgerRow {
                module: "fmn.config".to_owned(),
                symbol: key.path.clone(),
                canonical: key.path.clone(),
                kind: "config_key".to_owned(),
                exported: false,
                signature_defaults: format!(
                    "kind={};default={}",
                    key.kind.as_str(),
                    key.default.as_deref().unwrap_or(NONE)
                ),
                status: annotation.map_or(Status::Unreviewed, |row| row.status),
                evidence: annotation.map_or_else(|| NONE.to_owned(), |row| row.evidence.clone()),
                tests: annotation.map_or_else(|| NONE.to_owned(), |row| row.tests.clone()),
                notes: annotation.map_or_else(|| NONE.to_owned(), |row| row.notes.clone()),
            }
        }));

        rows.sort_unstable_by(|left, right| {
            (&left.module, &left.symbol, &left.kind).cmp(&(
                &right.module,
                &right.symbol,
                &right.kind,
            ))
        });
        rows
    }

    /// SHA-256 over every reviewed Ledger row and out-of-tier ruling.
    ///
    /// Committing this value in `API_OVERLAY.tsv [meta]` makes any downgrade
    /// or evidence rewrite an explicit authored policy change instead of
    /// something artifact regeneration can hide.
    #[must_use]
    pub fn ledger_reviewed_digest(&self) -> String {
        let mut material = String::new();
        for row in self.ledger_rows() {
            if let Some(line) = row.reviewed_identity_line() {
                material.push_str(&line);
            }
        }
        for ruling in self.out_of_tier.values() {
            let _ = writeln!(
                material,
                "out_of_tier\t{}\t{}\t{}\t{}\t{}",
                ruling.id,
                ruling.surface,
                ruling.status.as_str(),
                ruling.rationale,
                ruling.revisit_trigger
            );
        }
        fmn_hash::sha256(material.as_bytes()).to_hex()
    }

    /// Number of rows in the complete Parity Ledger across every schema
    /// surface.
    #[must_use]
    pub fn ledger_row_count(&self) -> usize {
        self.ledger_rows().len()
    }

    /// Number of complete-Ledger rows carrying one semantic status.
    #[must_use]
    pub fn ledger_status_count(&self, status: Status) -> usize {
        self.ledger_rows()
            .iter()
            .filter(|row| row.status == status)
            .count()
    }

    /// Symbols the wildcard surface binds — the import-surface inventory
    /// §1.6 says the Ledger has to enumerate, because `manimlib` has no
    /// `__all__` to trust.
    #[must_use]
    pub fn exported(&self) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.exported).collect()
    }

    /// Symbols of one kind.
    #[must_use]
    pub fn of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        // ubs:ignore -- public schema kind, not secret data
        self.symbols.iter().filter(|s| s.kind == kind).collect()
    }

    /// The name the Rust front door and the docs use for a symbol: its
    /// canonical name where C-9 ruled one, otherwise the Reference's.
    #[must_use]
    pub fn canonical_name(&self, symbol: &Symbol) -> String {
        self.renames.get(&symbol.key()).map_or_else(
            || symbol.name.clone(),
            |rename| match symbol.name.rsplit_once('.') {
                Some((owner, _)) => format!("{owner}.{}", rename.canonical),
                None => rename.canonical.clone(),
            },
        )
    }

    /// The adjudicated tier of a symbol, defaulting to
    /// [`Status::Unreviewed`].
    #[must_use]
    pub fn status(&self, symbol: &Symbol) -> Status {
        self.statuses
            .get(&symbol.key())
            .map_or(Status::Unreviewed, |annotation| annotation.status)
    }

    /// The Reference commit the extracted layer was taken from.
    #[must_use]
    pub fn reference_commit(&self) -> &str {
        self.meta
            .get("reference_commit")
            .map_or("unknown", String::as_str)
    }
}

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// How a generated artifact spells a comment.
#[derive(Clone, Copy)]
enum Comment {
    /// One marker per line — Rust doc comments and TSV `#` headers.
    PerLine(&'static str),
    /// One `<!--` ... `-->` block, because Markdown has no line comment and
    /// repeating the opener per line produces a broken document that renders
    /// as literal text.
    Block,
}

/// Header stamped on every generated artifact, so a reader who finds one
/// without the schema in hand still knows not to hand-edit it.
fn banner(comment: Comment, artifact: &str, commit: &str) -> String {
    let body = format!(
        "{artifact}\n\
         \n\
         GENERATED from API_SCHEMA.tsv + API_OVERLAY.tsv by\n\
         fmn_conformance::schema — regenerate, never hand-edit.\n\
         Reference pin: {commit}\n\
         \n\
         Regenerate:  UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance \\\n\
         \x20                --test api_schema\n"
    );
    match comment {
        Comment::PerLine(marker) => body
            .lines()
            .map(|line| {
                if line.is_empty() {
                    format!("{marker}\n")
                } else {
                    format!("{marker} {line}\n")
                }
            })
            .collect(),
        Comment::Block => format!("<!--\n{body}-->\n"),
    }
}

/// Generate `crates/fmn-config/src/generated.rs`: the typed extraction of the
/// config document, one `cx.<accessor>("<path>")?` per bound key.
///
/// W1CONF hand-wrote this body; generating it means the config surface has
/// exactly one definition (the schema) instead of two that can drift.
#[must_use]
pub fn generate_config_rs(schema: &Schema) -> String {
    let mut out = String::new();
    out.push_str(&banner(
        Comment::PerLine("//!"),
        "The typed config extraction (§16.2, fm-vn6).",
        schema.reference_commit(),
    ));
    out.push_str(
        "//!\n\
         //! The bound keys come from API_OVERLAY.tsv's `[config_binding]`\n\
         //! section; the accessors are `Cx`'s in `config.rs`. A key added to\n\
         //! `default_config.yml` without a binding fails the coverage check\n\
         //! before it can reach this file.\n\n",
    );
    // One `use` per line. A braced list would be repacked by rustfmt's width
    // algorithm, which the generator would then have to reimplement exactly
    // to stay byte-stable; separate statements are left alone.
    let mut structs: Vec<&str> = schema
        .bindings
        .iter()
        .map(|b| b.struct_name.as_str())
        .collect();
    structs.push("ConfigError");
    structs.push("Cx");
    structs.sort_unstable();
    structs.dedup();
    for name in &structs {
        let _ = writeln!(out, "use crate::config::{name};");
    }
    out.push_str("use crate::yaml::Value;\n\n");

    out.push_str(
        "/// Type a fully merged configuration document.\n\
         ///\n\
         /// # Errors\n\
         /// A [`ConfigError`] naming the key path and the expected-vs-found\n\
         /// shapes.\n\
         pub(crate) fn config_from_value(root: Value) -> Result<Config, ConfigError> {\n\
         \x20   let cx = Cx { root: &root };\n\
         \x20   let config = Config {\n",
    );

    let mut open: Option<&str> = None;
    for binding in &schema.bindings {
        if binding.is_top_level() {
            if open.is_some() {
                out.push_str("        },\n");
                open = None;
            }
            let _ = writeln!(
                out,
                "        {}: cx.{}(\"{}\")?,",
                binding.field, binding.accessor, binding.path
            );
            continue;
        }
        if open != Some(binding.struct_name.as_str()) {
            if open.is_some() {
                out.push_str("        },\n");
            }
            let _ = writeln!(
                out,
                "        {}: {} {{",
                binding.outer_field(),
                binding.struct_name
            );
            open = Some(binding.struct_name.as_str());
        }
        let _ = writeln!(
            out,
            "            {}: cx.{}(\"{}\")?,",
            binding.field, binding.accessor, binding.path
        );
    }
    if open.is_some() {
        out.push_str("        },\n");
    }

    out.push_str(
        "        // Placed below, once the borrows of `root` have ended.\n\
         \x20       raw: Value::Null,\n\
         \x20   };\n\
         \x20   Ok(Config {\n\
         \x20       raw: root,\n\
         \x20       ..config\n\
         \x20   })\n\
         }\n",
    );
    out
}

/// Generate the Parity Ledger's rows (§16.1): Python symbols plus the CLI and
/// config surfaces, carrying signatures/defaults and review evidence.
///
/// fm-iz4 owns the Ledger itself; this is the machine-generated substrate it
/// consumes, so the Ledger can never disagree with the schema about what the
/// surface *is*.
#[must_use]
pub fn generate_ledger_tsv(schema: &Schema) -> String {
    let mut out = banner(
        Comment::PerLine("#"),
        "The Parity Ledger (§16.1).",
        schema.reference_commit(),
    );
    out.push_str(
        "#\n\
         # module\tsymbol\tcanonical\tkind\texported\tsignature_defaults\tstatus\tevidence\ttests\tnotes\n",
    );
    for row in schema.ledger_rows() {
        out.push_str(&row.line());
        out.push('\n');
    }
    out
}

/// Generate the §16.6 out-of-tier ledger with a concrete revisit trigger for
/// every honest fringe or excluded surface.
#[must_use]
pub fn generate_out_of_tier_tsv(schema: &Schema) -> String {
    let mut out = banner(
        Comment::PerLine("#"),
        "The out-of-tier ledger (§16.6).",
        schema.reference_commit(),
    );
    out.push_str(
        "#\n\
         # id\tsurface\tstatus\trationale\trevisit_trigger\n",
    );
    for ruling in schema.out_of_tier.values() {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}",
            ruling.id,
            ruling.surface,
            ruling.status.as_str(),
            ruling.rationale,
            ruling.revisit_trigger
        );
    }
    out
}

/// Generate the honest reviewed-row coverage badge for the complete Ledger.
#[must_use]
pub fn generate_coverage_badge_svg(schema: &Schema) -> String {
    let total = schema.ledger_row_count();
    let unreviewed = schema.ledger_status_count(Status::Unreviewed);
    let reviewed = total.saturating_sub(unreviewed);
    let numerator = u128::try_from(reviewed).unwrap_or(0).saturating_mul(1_000);
    let denominator = u128::try_from(total).unwrap_or(1).max(1);
    let tenths = numerator / denominator;
    let percentage = format!("{}.{}%", tenths / 10, tenths % 10);
    let color = match tenths {
        900.. => "#4c1",
        750.. => "#97ca00",
        500.. => "#a4a61d",
        250.. => "#dfb317",
        100.. => "#fe7d37",
        _ => "#e05d44",
    };
    let value = format!("{reviewed}/{total} reviewed ({percentage})");
    format!(
        "<!-- GENERATED from the API schema; never hand-edit. Reference pin: {}; reviewed digest: {} -->\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"280\" height=\"20\" role=\"img\" aria-label=\"Parity Ledger: {value}\">\n\
         <title>Parity Ledger: {value}</title>\n\
         <linearGradient id=\"s\" x2=\"0\" y2=\"100%\"><stop offset=\"0\" stop-color=\"#fff\" stop-opacity=\".7\"/><stop offset=\".1\" stop-color=\"#aaa\" stop-opacity=\".1\"/><stop offset=\".9\" stop-opacity=\".3\"/><stop offset=\"1\" stop-opacity=\".5\"/></linearGradient>\n\
         <clipPath id=\"r\"><rect width=\"280\" height=\"20\" rx=\"3\"/></clipPath>\n\
         <g clip-path=\"url(#r)\"><rect width=\"96\" height=\"20\" fill=\"#555\"/><rect x=\"96\" width=\"184\" height=\"20\" fill=\"{color}\"/><rect width=\"280\" height=\"20\" fill=\"url(#s)\"/></g>\n\
         <g fill=\"#fff\" text-anchor=\"middle\" font-family=\"Verdana,Geneva,DejaVu Sans,sans-serif\" font-size=\"11\"><text x=\"48\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">parity ledger</text><text x=\"48\" y=\"14\">parity ledger</text><text x=\"188\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">{value}</text><text x=\"188\" y=\"14\">{value}</text></g>\n\
         </svg>\n",
        schema.reference_commit(),
        schema.ledger_reviewed_digest()
    )
}

/// Generate the CLI flag table (§13.6) — W9's normative source for the flag
/// surface it keeps "where it still means something".
#[must_use]
pub fn generate_cli_table_md(schema: &Schema) -> String {
    let mut out = banner(
        Comment::Block,
        "The CLI flag table (§13.6).",
        schema.reference_commit(),
    );
    out.push_str("\n# The `fmn` flag surface\n\n");
    let _ = writeln!(
        out,
        "The Reference's `manimlib/config.py` declares {} options. Every row has \
         exactly one authored ruling and generated-parser binding; coverage is \
         fail-closed, so the inventory cannot quietly shrink.\n",
        schema.flags.len()
    );
    out.push_str(
        "## Reference flags\n\n\
         | Options | Command | Binding | Status | Evidence | Action | Default | Semantics |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for flag in &schema.flags {
        let Some(binding) = schema
            .flag_bindings
            .iter()
            .find(|binding| binding.options == flag.options)
        else {
            continue;
        };
        let default = flag.default.as_deref().unwrap_or_else(|| {
            if flag.action == "store_true" {
                "false"
            } else {
                "—"
            }
        });
        let _ = writeln!(
            out,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} |",
            flag.options,
            binding.command.as_str(),
            binding.binding,
            binding.status.as_str(),
            binding.evidence,
            flag.action,
            default,
            binding.note
        );
    }

    out.push_str(
        "\n## Native flags\n\n\
         | Options | Command | Binding | Status | Evidence | Action | Default | Help |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for flag in &schema.native_flags {
        let _ = writeln!(
            out,
            "| `{}` | {} | `{}` | {} | {} | {} | {} | {} |",
            flag.options,
            flag.command.as_str(),
            flag.binding,
            flag.status.as_str(),
            flag.evidence,
            flag.action,
            flag.default.as_deref().unwrap_or("—"),
            flag.help
        );
    }

    out.push_str("\n## Commands\n\n| Command | Status | Evidence | Meaning |\n|---|---|---|---|\n");
    for subcommand in &schema.subcommands {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            subcommand.command.as_str(),
            subcommand.status.as_str(),
            subcommand.evidence,
            subcommand.help
        );
    }

    out.push_str("\n## Exit codes\n\n| Code | Identity | Meaning |\n|---:|---|---|\n");
    for exit in &schema.exit_codes {
        let _ = writeln!(
            out,
            "| {} | `{}` | {} |",
            exit.code, exit.name, exit.meaning
        );
    }

    out.push_str(
        "\n## Flag interactions\n\n\
         These rules are emitted into the parser artifact and executed after \
         token collection. `implies` rules are non-failing; every other rule \
         names the stable exit identity returned on violation.\n\n\
         | Rule | Kind | Bindings | Exit | Diagnostic |\n\
         |---|---|---|---|---|\n",
    );
    for interaction in &schema.flag_interactions {
        let _ = writeln!(
            out,
            "| `{}` | {} | `{}` | {} | {} |",
            interaction.id,
            interaction.kind.as_str(),
            interaction.operands.join("`, `"),
            interaction.exit_code.as_deref().unwrap_or("—"),
            interaction.message
        );
    }
    out
}

/// Generate the typed parser contract consumed by `fmn-cli`.
///
/// This artifact deliberately contains data, not parser logic. `fmn-cli`
/// executes the generated action/arity/interaction vocabulary without a
/// runtime dependency on the schema crate or the repository TSV files.
#[must_use]
pub fn generate_cli_rs(schema: &Schema) -> String {
    let mut out = banner(
        Comment::PerLine("//!"),
        "The typed `fmn` parser contract (§13.6).",
        schema.reference_commit(),
    );
    out.push_str(
        "\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum CommandScope {\n\
         \x20   Global,\n\
         \x20   Render,\n\
         \x20   Doctor,\n\
         \x20   Batch,\n\
         \x20   Studio,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum FlagAction {\n\
         \x20   SetTrue,\n\
         \x20   Store,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum FlagArity {\n\
         \x20   None,\n\
         \x20   One,\n\
         \x20   Optional,\n\
         \x20   Many,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum FlagStatus {\n\
         \x20   Same,\n\
         \x20   Improved,\n\
         \x20   Tiered,\n\
         \x20   Excluded,\n\
         \x20   Unreviewed,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum FlagSource {\n\
         \x20   Reference,\n\
         \x20   Native,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct FlagSpec {\n\
         \x20   pub options: &'static [&'static str],\n\
         \x20   pub binding: &'static str,\n\
         \x20   pub command: CommandScope,\n\
         \x20   pub action: FlagAction,\n\
         \x20   pub arity: FlagArity,\n\
         \x20   pub default: Option<&'static str>,\n\
         \x20   pub value_type: Option<&'static str>,\n\
         \x20   pub status: FlagStatus,\n\
         \x20   pub source: FlagSource,\n\
         \x20   pub help: &'static str,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct SubcommandSpec {\n\
         \x20   pub command: CommandScope,\n\
         \x20   pub status: FlagStatus,\n\
         \x20   pub help: &'static str,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct ExitCodeSpec {\n\
         \x20   pub code: u8,\n\
         \x20   pub name: &'static str,\n\
         \x20   pub meaning: &'static str,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum InteractionKind {\n\
         \x20   AtMostOne,\n\
         \x20   Conflicts,\n\
         \x20   RequiresAny,\n\
         \x20   Implies,\n\
         \x20   Exclusive,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct InteractionSpec {\n\
         \x20   pub id: &'static str,\n\
         \x20   pub kind: InteractionKind,\n\
         \x20   pub operands: &'static [&'static str],\n\
         \x20   pub exit_code: Option<&'static str>,\n\
         \x20   pub message: &'static str,\n\
         }\n\n",
    );

    out.push_str("pub const FLAG_SPECS: &[FlagSpec] = &[\n");
    for flag in &schema.flags {
        let Some(binding) = schema
            .flag_bindings
            .iter()
            .find(|binding| binding.options == flag.options)
        else {
            continue;
        };
        // A tiered or excluded flag does not do what the Reference's help
        // says, so `fmn --help` states the ruling and fmn's behavior instead.
        let help = match binding.status {
            Status::Tiered | Status::Excluded => format!(
                "Not available in fmn ({} {}): {}",
                binding.status.as_str(),
                binding.evidence,
                binding.note
            ),
            _ => flag.help.clone().unwrap_or_default(),
        };
        write_flag_spec(
            &mut out,
            &flag.options,
            &binding.binding,
            binding.command,
            &flag.action,
            flag.nargs.as_deref(),
            flag.default.as_deref(),
            flag.ty.as_deref(),
            binding.status,
            "Reference",
            &help,
        );
    }
    for flag in &schema.native_flags {
        write_flag_spec(
            &mut out,
            &flag.options,
            &flag.binding,
            flag.command,
            &flag.action,
            flag.nargs.as_deref(),
            flag.default.as_deref(),
            flag.ty.as_deref(),
            flag.status,
            "Native",
            &flag.help,
        );
    }
    out.push_str("];\n\n");

    out.push_str("pub const SUBCOMMAND_SPECS: &[SubcommandSpec] = &[\n");
    for subcommand in &schema.subcommands {
        let _ = writeln!(
            out,
            "    SubcommandSpec {{\n        command: CommandScope::{},\n        \
             status: FlagStatus::{},\n        help: {:?},\n    }},",
            command_variant(subcommand.command),
            status_variant(subcommand.status),
            subcommand.help
        );
    }
    out.push_str("];\n\n");

    out.push_str("pub const EXIT_CODE_SPECS: &[ExitCodeSpec] = &[\n");
    for exit in &schema.exit_codes {
        let _ = writeln!(
            out,
            "    ExitCodeSpec {{\n        code: {},\n        name: {:?},\n        \
             meaning: {:?},\n    }},",
            exit.code, exit.name, exit.meaning
        );
    }
    out.push_str("];\n\n");

    out.push_str("pub const INTERACTION_SPECS: &[InteractionSpec] = &[\n");
    for interaction in &schema.flag_interactions {
        let inline_operands = interaction
            .operands
            .iter()
            .map(|operand| format!("{operand:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let inline_line = format!("        operands: &[{inline_operands}],\n");
        // rustfmt's default `array_width` is 60. Emit the same shape so the
        // generated artifact remains stable after the mandatory fmt gate.
        let operands = if inline_operands.chars().count() <= 60 {
            inline_line
        } else {
            let values = interaction
                .operands
                .iter()
                .map(|operand| format!("            {operand:?},\n"))
                .collect::<String>();
            format!("        operands: &[\n{values}        ],\n")
        };
        let exit = interaction
            .exit_code
            .as_ref()
            .map_or_else(|| "None".to_owned(), |name| format!("Some({name:?})"));
        let _ = writeln!(
            out,
            "    InteractionSpec {{\n        id: {:?},\n        kind: \
             InteractionKind::{},\n{}        exit_code: {},\n        \
             message: {:?},\n    }},",
            interaction.id,
            interaction_variant(interaction.kind),
            operands,
            exit,
            interaction.message
        );
    }
    out.push_str("];\n");
    out
}

#[allow(clippy::too_many_arguments)]
fn write_flag_spec(
    out: &mut String,
    options: &str,
    binding: &str,
    command: CliCommand,
    action: &str,
    nargs: Option<&str>,
    default: Option<&str>,
    value_type: Option<&str>,
    status: Status,
    source: &str,
    help: &str,
) {
    let options = options
        .split(',')
        .map(|option| format!("{option:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let (action, arity) = if action == "store_true" {
        ("SetTrue", "None")
    } else {
        (
            "Store",
            match nargs {
                Some("'?'") | Some("?") => "Optional",
                Some("'*'") | Some("*") => "Many",
                _ => "One",
            },
        )
    };
    let default = default.map_or_else(
        || {
            if action == "SetTrue" {
                "Some(\"false\")".to_owned()
            } else {
                "None".to_owned()
            }
        },
        |value| format!("Some({value:?})"),
    );
    let value_type =
        value_type.map_or_else(|| "None".to_owned(), |value| format!("Some({value:?})"));
    let _ = writeln!(
        out,
        "    FlagSpec {{\n        options: &[{}],\n        binding: {:?},\n        \
         command: CommandScope::{},\n        action: FlagAction::{},\n        arity: \
         FlagArity::{},\n        default: {},\n        value_type: {},\n        status: \
         FlagStatus::{},\n        source: FlagSource::{},\n        help: {:?},\n    }},",
        options,
        binding,
        command_variant(command),
        action,
        arity,
        default,
        value_type,
        status_variant(status),
        source,
        help
    );
}

const fn command_variant(command: CliCommand) -> &'static str {
    match command {
        CliCommand::Global => "Global",
        CliCommand::Render => "Render",
        CliCommand::Doctor => "Doctor",
        CliCommand::Batch => "Batch",
        CliCommand::Studio => "Studio",
    }
}

const fn status_variant(status: Status) -> &'static str {
    match status {
        Status::Same => "Same",
        Status::Improved => "Improved",
        Status::Tiered => "Tiered",
        Status::Excluded => "Excluded",
        Status::Unreviewed => "Unreviewed",
    }
}

const fn interaction_variant(kind: FlagInteractionKind) -> &'static str {
    match kind {
        FlagInteractionKind::AtMostOne => "AtMostOne",
        FlagInteractionKind::Conflicts => "Conflicts",
        FlagInteractionKind::RequiresAny => "RequiresAny",
        FlagInteractionKind::Implies => "Implies",
        FlagInteractionKind::Exclusive => "Exclusive",
    }
}

/// Generate the schema's human-facing summary — the docs artifact of §16.2,
/// and the place the import-surface and C-9 numbers are published.
#[must_use]
pub fn generate_docs_md(schema: &Schema) -> String {
    let mut out = banner(
        Comment::Block,
        "The API schema summary (§16.2).",
        schema.reference_commit(),
    );
    out.push_str("\n# The one API schema\n\n");
    let _ = writeln!(
        out,
        "Generated from `API_SCHEMA.tsv` (extracted from the pinned Reference) and \
         `API_OVERLAY.tsv` (authored). Reference pin `{}`.\n",
        schema.reference_commit()
    );

    out.push_str("## Surface inventory\n\n| Kind | Total | Wildcard-exported |\n|---|---|---|\n");
    for kind in [
        SymbolKind::Class,
        SymbolKind::Method,
        SymbolKind::Property,
        SymbolKind::Attribute,
        SymbolKind::Function,
        SymbolKind::Constant,
        SymbolKind::LeakedImport,
    ] {
        let all = schema.of_kind(kind);
        let exported = all.iter().filter(|s| s.exported).count();
        let _ = writeln!(out, "| {} | {} | {} |", kind.as_str(), all.len(), exported);
    }

    let exported = schema.exported();
    let mut exported_name_counts = BTreeMap::new();
    for symbol in &exported {
        *exported_name_counts
            .entry(symbol.name.as_str())
            .or_insert(0_usize) += 1;
    }
    let duplicate_names: Vec<&str> = exported_name_counts
        .iter()
        .filter_map(|(name, count)| (*count > 1).then_some(*name))
        .collect();
    let duplicate_list = duplicate_names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(
        out,
        "\n`from manimlib import *` binds {} unique names from {} wildcard-exported \
         schema rows. The {} duplicate rows are {duplicate_list}. The Reference \
         declares no `__all__` (§1.6), so the unique-name count is the *computed* \
         wildcard closure, leaked third-party imports included — enumerating it is \
         the only way to know what the surface actually is.\n",
        exported_name_counts.len(),
        exported.len(),
        exported.len() - exported_name_counts.len()
    );

    let _ = writeln!(
        out,
        "## Parity Ledger coverage\n\nThe single Ledger contains {} rows: {} Python \
         symbols, {} Reference CLI flags, {} FrankenManim-native CLI flags, {} \
         CLI commands, and {} config keys. Its reviewed-identity ratchet is \
         `{}`.\n",
        schema.ledger_row_count(),
        schema.symbols.len(),
        schema.flags.len(),
        schema.native_flags.len(),
        schema.subcommands.len(),
        schema.config.len(),
        schema.ledger_reviewed_digest()
    );

    let _ = writeln!(
        out,
        "## Semantic tiers (§16.1)\n\n| Status | Ledger rows |\n|---|---|"
    );
    for status in [
        Status::Same,
        Status::Improved,
        Status::Tiered,
        Status::Excluded,
        Status::Unreviewed,
    ] {
        let n = schema.ledger_status_count(status);
        let _ = writeln!(out, "| {} | {} |", status.as_str(), n);
    }
    out.push_str(
        "\n`unreviewed` is the honest default for a surface nobody has \
         adjudicated yet; it is the number the Parity Ledger ratchets down. \
         Every improved row resolves to a Behavior Note; every tiered or \
         excluded row resolves to `docs/api/out_of_tier.tsv`.\n",
    );

    let _ = writeln!(
        out,
        "\n## Canonical names (Appendix C, C-9)\n\n{} symbols and {} parameters \
         carry a public-surface typo. The Rust front door and these docs use the \
         canonical name; `fmn-python` binds both, so source-unedited scenes keep \
         working.\n",
        schema.renames.len(),
        schema.param_renames.len()
    );
    out.push_str("| Reference | Canonical |\n|---|---|\n");
    for rename in schema.renames.values() {
        let _ = writeln!(out, "| `{}` | `{}` |", rename.symbol, rename.canonical);
    }

    let _ = writeln!(
        out,
        "\n## Config keys\n\n{} keys, {} of them shared with the Reference's \
         `default_config.yml`. Every one is bound to a Rust field by \
         `API_OVERLAY.tsv`, and the binding generates \
         `crates/fmn-config/src/generated.rs`.\n",
        schema.config.len(),
        schema.config.iter().filter(|c| c.in_reference).count()
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const EXTRACT: &str = "[meta]\nschema_version\t1\n\
        reference_commit\t0123456789abcdef0123456789abcdef01234567\n\
        generator\tscripts/gen_api_schema.py\n\
        wildcard_exports\t1\n\n\
        [symbols]\nm\tA\tclass\tdefined\t1\tobject\n\
        m\tA.foo_listner\tmethod\tdefined\t0\t-\n\n\
        [params]\nm:A.foo_listner\t0\tself\tpositional_or_keyword\t-\t-\n\n\
        [flags]\n-w,--write\twrite\tstore_true\t-\tFalse\t-\twrite it\n\n\
        [config]\ncamera.fps\tint\t30\t1\n";

    const OVERLAY: &str = "[meta]\noverlay_version\t1\n\
        ledger_reviewed_digest\t369fe193fc544ebe53abf947173452e5d86ae3645529f8ab9613c8f80ee6fb7a\n\n\
        [canonical]\nm:A.foo_listner\tfoo_listener\tC-9\t-\n\n\
        [param_canonical]\n\n\
        [config_binding]\ncamera.fps\tCameraConfig\tfps\tu32\n\n\
        [config_status]\n\n\
        [out_of_tier]\nOOT-FIXTURE\tfixture surface\ttiered\tbounded reason\tconcrete trigger\n\n\
        [status]\nm:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better\n\n\
        [flag_binding]\n-w,--write\tsame\t-\trender\twrite\ttests/cli.rs\tkept\n\n\
        [native_flags]\n-h,--help\tstore_true\t-\tFalse\t-\tglobal\thelp\timproved\tBN-15\ttests/cli.rs\tHelp\n\n\
        [subcommands]\nrender\tsame\t-\ttests/cli.rs\tRender scenes\n\n\
        [exit_codes]\n0\tsuccess\tCompleted\n2\tusage\tBad arguments\n\n\
        [flag_interaction]\n";

    fn schema() -> Schema {
        Schema::parse(EXTRACT, OVERLAY).expect("fixture parses")
    }

    #[test]
    fn the_two_layers_merge_into_one_effective_schema() {
        let s = schema();
        assert_eq!(s.reference_commit(), TEST_COMMIT);
        assert_eq!(s.symbols.len(), 2);
        assert_eq!(s.flags.len(), 1);
        assert_eq!(s.flag_bindings.len(), 1);
        assert_eq!(s.native_flags.len(), 1);
        assert_eq!(s.subcommands.len(), 1);
        assert_eq!(s.exit_codes.len(), 2);
        assert_eq!(s.config.len(), 1);
        assert_eq!(s.bindings.len(), 1);
    }

    #[test]
    fn a_c9_ruling_renames_only_the_member_leaf() {
        let s = schema();
        let method = s
            .symbols
            .iter()
            // ubs:ignore -- public schema kind in a unit-test fixture
            .find(|x| x.kind == SymbolKind::Method)
            .unwrap();
        assert_eq!(s.canonical_name(method), "A.foo_listener");
        let class = s
            .symbols
            .iter()
            // ubs:ignore -- public schema kind in a unit-test fixture
            .find(|x| x.kind == SymbolKind::Class)
            .unwrap();
        assert_eq!(
            s.canonical_name(class),
            "A",
            "an unruled symbol keeps its name"
        );
    }

    #[test]
    fn an_unadjudicated_symbol_reads_as_unreviewed() {
        let s = schema();
        let method = s
            .symbols
            .iter()
            // ubs:ignore -- public schema kind in a unit-test fixture
            .find(|x| x.kind == SymbolKind::Method)
            .unwrap();
        assert_eq!(s.status(method), Status::Unreviewed);
        let class = s
            .symbols
            .iter()
            // ubs:ignore -- public schema kind in a unit-test fixture
            .find(|x| x.kind == SymbolKind::Class)
            .unwrap();
        assert_eq!(s.status(class), Status::Improved);
    }

    #[test]
    fn reviewed_rulings_require_structured_evidence_tests_and_notes() {
        let bad_note = OVERLAY.replace("m:A\timproved\tBN-01", "m:A\timproved\tdocs/a.md");
        let error = Schema::parse(EXTRACT, &bad_note).unwrap_err();
        assert!(
            matches!(error, SchemaError::EvidenceContract { .. }),
            "got {error}"
        );
        assert!(error.to_string().contains("BN-*"), "got {error}");

        let no_tests = OVERLAY.replace("\ttests/a.rs\tA is deliberately better", "\t-\t-");
        let error = Schema::parse(EXTRACT, &no_tests).unwrap_err();
        assert!(
            matches!(error, SchemaError::EvidenceContract { .. }),
            "got {error}"
        );
        assert!(error.to_string().contains("executed tests"), "got {error}");

        let unknown_tier = OVERLAY.replace(
            "m:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better",
            "m:A\ttiered\tOOT-MISSING\ttests/a.rs\tA is deliberately tiered",
        );
        let error = Schema::parse(EXTRACT, &unknown_tier).unwrap_err();
        assert!(
            matches!(error, SchemaError::EvidenceContract { .. }),
            "got {error}"
        );
        assert!(error.to_string().contains("declared OOT-*"), "got {error}");

        let mismatched_tier = OVERLAY.replace(
            "m:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better",
            "m:A\texcluded\tOOT-FIXTURE\ttests/a.rs\tA is deliberately excluded",
        );
        let error = Schema::parse(EXTRACT, &mismatched_tier).unwrap_err();
        assert!(
            matches!(error, SchemaError::EvidenceContract { .. }),
            "got {error}"
        );
        assert!(
            error.to_string().contains("out-of-tier status is `tiered`"),
            "got {error}"
        );
    }

    #[test]
    fn a_reviewed_status_cannot_drop_without_amending_the_ratchet() {
        let downgraded = OVERLAY.replace(
            "m:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better",
            "m:A\tunreviewed\t-\t-\t-",
        );
        let error = Schema::parse(EXTRACT, &downgraded).unwrap_err();
        assert!(
            matches!(error, SchemaError::ReviewRatchet { .. }),
            "got {error}"
        );

        let weakened_trigger = OVERLAY.replace("concrete trigger", "vague trigger");
        let error = Schema::parse(EXTRACT, &weakened_trigger).unwrap_err();
        assert!(
            matches!(error, SchemaError::ReviewRatchet { .. }),
            "got {error}"
        );
    }

    #[test]
    fn an_overlay_naming_an_unknown_symbol_is_stale_not_ignored() {
        let stale = OVERLAY.replace("m:A.foo_listner", "m:Gone.method");
        let err = Schema::parse(EXTRACT, &stale).unwrap_err();
        assert!(
            matches!(
                err,
                SchemaError::DanglingOverlay {
                    section: "canonical",
                    ..
                }
            ),
            "got {err}"
        );
        assert!(err.to_string().contains("gen_api_schema.py"));
    }

    #[test]
    fn section_headers_are_closed_unique_and_canonical() {
        let unknown = OVERLAY.replace("[status]", "[statuz]");
        let error = Schema::parse(EXTRACT, &unknown).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("unknown section [statuz]")),
            "got {error}"
        );

        let repeated = format!("{OVERLAY}\n[status]\n");
        let error = Schema::parse(EXTRACT, &repeated).unwrap_err();
        assert!(
            matches!(
                &error,
                SchemaError::DuplicateIdentity {
                    section: "section",
                    key,
                    ..
                } if key == "status"
            ),
            "got {error}"
        );

        let indented = OVERLAY.replace("[status]", " [status]");
        let error = Schema::parse(EXTRACT, &indented).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("surrounding whitespace")),
            "got {error}"
        );

        let crlf = OVERLAY.replace('\n', "\r\n");
        let error = Schema::parse(EXTRACT, &crlf).unwrap_err();
        assert!(matches!(error, SchemaError::Document { .. }), "got {error}");

        let no_final_newline = OVERLAY.trim_end_matches('\n');
        let error = Schema::parse(EXTRACT, no_final_newline).unwrap_err();
        assert!(matches!(error, SchemaError::Document { .. }), "got {error}");

        let padded_field = OVERLAY.replace("m:A\timproved", "m:A\t improved");
        let error = Schema::parse(EXTRACT, &padded_field).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("TSV fields")),
            "got {error}"
        );

        let wide_row = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\na\tb\tc\td\te\tf\tg\th\ti\tj\tk\tl\tm\tn\to\tp\tq\n",
        );
        let error = Schema::parse(EXTRACT, &wide_row).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("16 tab-separated fields")),
            "got {error}"
        );
    }

    #[test]
    fn document_and_line_resource_limits_are_enforced_first() {
        let oversized_document = "x".repeat(MAX_SCHEMA_DOCUMENT_BYTES + 1);
        let error =
            Sections::parse("API_SCHEMA.tsv", &oversized_document, EXTRACTED_SECTIONS).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("format limit")),
            "got {error}"
        );

        let oversized_line = format!("[meta]\n{}\n", "x".repeat(MAX_SCHEMA_LINE_BYTES + 1));
        let error =
            Sections::parse("API_SCHEMA.tsv", &oversized_line, EXTRACTED_SECTIONS).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("line exceeds")),
            "got {error}"
        );
    }

    #[test]
    fn metadata_is_closed_complete_versioned_and_self_consistent() {
        let unknown = EXTRACT.replace("schema_version\t1", "schema_version\t1\nfuture_mode\tmaybe");
        let error = Schema::parse(&unknown, OVERLAY).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Syntax { detail, .. } if detail.contains("unknown [meta] key")),
            "got {error}"
        );

        let missing = EXTRACT.replace("generator\tscripts/gen_api_schema.py\n", "");
        let error = Schema::parse(&missing, OVERLAY).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("missing required key")),
            "got {error}"
        );

        let future = OVERLAY.replace("overlay_version\t1", "overlay_version\t2");
        let error = Schema::parse(EXTRACT, &future).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("unsupported overlay_version")),
            "got {error}"
        );

        let truncated_pin = EXTRACT.replace(TEST_COMMIT, "abc123");
        let error = Schema::parse(&truncated_pin, OVERLAY).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("40-character lowercase hex")),
            "got {error}"
        );

        let wrong_count = EXTRACT.replace("wildcard_exports\t1", "wildcard_exports\t2");
        let error = Schema::parse(&wrong_count, OVERLAY).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("[symbols] contains 1")),
            "got {error}"
        );

        let noncanonical_count = EXTRACT.replace("wildcard_exports\t1", "wildcard_exports\t01");
        let error = Schema::parse(&noncanonical_count, OVERLAY).unwrap_err();
        assert!(
            matches!(&error, SchemaError::Document { detail, .. } if detail.contains("canonical nonnegative integer")),
            "got {error}"
        );

        let nonbinary = EXTRACT.replace("camera.fps\tint\t30\t1", "camera.fps\tint\t30\tyes");
        let error = Schema::parse(&nonbinary, OVERLAY).unwrap_err();
        assert!(
            matches!(
                error,
                SchemaError::Field {
                    column: "reference",
                    ..
                }
            ),
            "got {error}"
        );
    }

    #[test]
    fn metadata_and_authored_rulings_cannot_overwrite_earlier_rows() {
        let duplicate_meta = EXTRACT.replace(
            &format!("reference_commit\t{TEST_COMMIT}"),
            &format!("reference_commit\t{TEST_COMMIT}\nreference_commit\tdef456"),
        );
        let duplicate_canonical = OVERLAY.replace(
            "m:A.foo_listner\tfoo_listener\tC-9\t-",
            "m:A.foo_listner\tfoo_listener\tC-9\t-\n\
             m:A.foo_listner\tother_listener\tC-9\tBN-duplicate",
        );
        let duplicate_status = OVERLAY.replace(
            "m:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better",
            "m:A\timproved\tBN-01\ttests/a.rs\tA is deliberately better\n\
             m:A\tsame\tdocs/a.md\ttests/a.rs\tduplicate",
        );
        for (file, overlay, section) in [
            (duplicate_meta.as_str(), OVERLAY, "meta"),
            (EXTRACT, duplicate_canonical.as_str(), "canonical"),
            (EXTRACT, duplicate_status.as_str(), "status"),
        ] {
            let error = Schema::parse(file, overlay).unwrap_err();
            assert!(
                matches!(
                    &error,
                    SchemaError::DuplicateIdentity {
                        section: found,
                        ..
                    } if found == &section
                ),
                "{section}: got {error}"
            );
        }
    }

    #[test]
    fn keyed_extracted_and_cross_layer_identities_are_unique() {
        let duplicate_flag = EXTRACT.replace(
            "-w,--write\twrite\tstore_true\t-\tFalse\t-\twrite it",
            "-w,--write\twrite\tstore_true\t-\tFalse\t-\twrite it\n\
             -w,--write\twrite-again\tstore_true\t-\tFalse\t-\tduplicate",
        );
        let error = Schema::parse(&duplicate_flag, OVERLAY).unwrap_err();
        assert!(
            matches!(
                &error,
                SchemaError::DuplicateIdentity {
                    section: "flags",
                    ..
                }
            ),
            "got {error}"
        );

        let duplicate_extracted_config = EXTRACT.replace(
            "camera.fps\tint\t30\t1",
            "camera.fps\tint\t30\t1\ncamera.fps\tint\t60\t1",
        );
        let error = Schema::parse(&duplicate_extracted_config, OVERLAY).unwrap_err();
        assert!(
            matches!(
                &error,
                SchemaError::DuplicateIdentity {
                    section: "config",
                    ..
                }
            ),
            "got {error}"
        );

        let duplicate_config = OVERLAY.replace(
            "[config_binding]",
            "[optional_config]\ncamera.fps\tint\t1\n\n[config_binding]",
        );
        let error = Schema::parse(EXTRACT, &duplicate_config).unwrap_err();
        assert!(
            matches!(
                &error,
                SchemaError::DuplicateIdentity {
                    section: "optional_config",
                    previous_file: "API_SCHEMA.tsv",
                    ..
                }
            ),
            "got {error}"
        );
    }

    #[test]
    fn every_extracted_flag_needs_exactly_one_authored_binding() {
        let missing = OVERLAY.replace(
            "-w,--write\tsame\t-\trender\twrite\ttests/cli.rs\tkept\n",
            "",
        );
        let err = Schema::parse(EXTRACT, &missing).unwrap_err(); // ubs:ignore — negative parser test
        assert!(matches!(err, SchemaError::FlagCoverage { .. }), "got {err}");
        assert!(err.to_string().contains("has no ruling"), "got {err}");

        let stale = OVERLAY.replace("-w,--write\tsame", "--gone\tsame");
        let err = Schema::parse(EXTRACT, &stale).unwrap_err(); // ubs:ignore — negative parser test
        assert!(
            matches!(
                err,
                SchemaError::DanglingOverlay {
                    section: "flag_binding",
                    ..
                }
            ),
            "got {err}"
        );
    }

    #[test]
    fn interactions_are_checked_against_bindings_and_exit_codes() {
        let unknown_binding = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\nbad\tconflicts\twrite|ghost\tusage\tbad\n",
        );
        let err = Schema::parse(EXTRACT, &unknown_binding).unwrap_err(); // ubs:ignore — negative parser test
        assert!(matches!(err, SchemaError::CliContract { .. }), "got {err}");
        assert!(err.to_string().contains("unknown binding"), "got {err}");

        let unknown_exit = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\nbad\texclusive\twrite\tghost\tbad\n",
        );
        let err = Schema::parse(EXTRACT, &unknown_exit).unwrap_err(); // ubs:ignore — negative parser test
        assert!(matches!(err, SchemaError::CliContract { .. }), "got {err}");
        assert!(err.to_string().contains("unknown exit code"), "got {err}");

        let allowed_modifier = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\nquery\texclusive\twrite|help\tusage\tquery only\n",
        );
        Schema::parse(EXTRACT, &allowed_modifier)
            .expect("exclusive rules may name consumed modifiers after the action");

        let selected_value = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\nselected\tconflicts\twrite=movie,frame|help\tusage\tselected values\n",
        );
        Schema::parse(EXTRACT, &selected_value)
            .expect("conflict operands may select declared binding values");

        let bad_selector = OVERLAY.replace(
            "[flag_interaction]\n",
            "[flag_interaction]\nbad\tconflicts\twrite=|help\tusage\tbad selector\n",
        );
        let err = Schema::parse(EXTRACT, &bad_selector).unwrap_err();
        assert!(matches!(err, SchemaError::CliContract { .. }), "got {err}");
        assert!(err.to_string().contains("value selector"), "got {err}");
    }

    #[test]
    fn native_flag_value_types_are_fail_closed() {
        let unknown_type = OVERLAY.replace(
            "-h,--help\tstore_true\t-\tFalse\t-\tglobal",
            "-h,--help\tstore_true\t-\tFalse\tmystery\tglobal",
        );
        let err = Schema::parse(EXTRACT, &unknown_type).unwrap_err();
        assert!(matches!(err, SchemaError::CliContract { .. }), "got {err}");
        assert!(
            err.to_string().contains("unsupported value type"),
            "got {err}"
        );
    }

    #[test]
    fn a_row_with_the_wrong_arity_names_its_line() {
        let broken = EXTRACT.replace("m\tA\tclass\tdefined\t1\tobject", "m\tA\tclass");
        let err = Schema::parse(&broken, OVERLAY).unwrap_err();
        assert!(matches!(err, SchemaError::Arity { .. }), "got {err}");
        if let SchemaError::Arity {
            line,
            expected,
            found,
            ..
        } = err
        {
            assert_eq!((expected, found), (6, 3));
            assert!(line > 0);
        }
    }

    #[test]
    fn an_unknown_enumerated_word_is_a_named_field_error() {
        let broken = EXTRACT.replace("\tclass\t", "\tklass\t");
        let err = Schema::parse(&broken, OVERLAY).unwrap_err();
        assert!(
            matches!(err, SchemaError::Field { column: "kind", .. }),
            "got {err}"
        );
    }

    #[test]
    fn a_row_before_any_section_header_is_refused() {
        let err = Schema::parse("orphan\trow\n[meta]\n", OVERLAY).unwrap_err();
        assert!(
            matches!(err, SchemaError::Sectionless { line: 1, .. }),
            "got {err}"
        );
    }

    #[test]
    fn config_coverage_catches_both_directions() {
        let s = schema();
        s.check_config_coverage().expect("the fixture is covered");

        let unbound = OVERLAY.replace("camera.fps\tCameraConfig\tfps\tu32\n", "");
        let s = Schema::parse(EXTRACT, &unbound).unwrap();
        let err = s.check_config_coverage().unwrap_err();
        assert!(err.to_string().contains("has no binding"), "got {err}");

        let extra = OVERLAY.replace(
            "[config_binding]\n",
            "[config_binding]\ncamera.ghost\tCameraConfig\tghost\tu32\n",
        );
        let s = Schema::parse(EXTRACT, &extra).unwrap();
        let err = s.check_config_coverage().unwrap_err();
        assert!(err.to_string().contains("names no config key"), "got {err}");
    }

    #[test]
    fn every_generator_stamps_the_reference_pin() {
        let s = schema();
        for artifact in [
            generate_config_rs(&s),
            generate_ledger_tsv(&s),
            generate_out_of_tier_tsv(&s),
            generate_cli_table_md(&s),
            generate_cli_rs(&s),
            generate_docs_md(&s),
        ] {
            assert!(artifact.contains(TEST_COMMIT), "artifact lost the pin");
            assert!(
                artifact.contains("never hand-edit"),
                "artifact lost its banner"
            );
        }
    }

    #[test]
    fn generation_is_a_pure_function_of_the_schema() {
        let s = schema();
        assert_eq!(generate_config_rs(&s), generate_config_rs(&schema()));
        assert_eq!(generate_ledger_tsv(&s), generate_ledger_tsv(&schema()));
        assert_eq!(
            generate_out_of_tier_tsv(&s),
            generate_out_of_tier_tsv(&schema())
        );
        assert_eq!(
            generate_coverage_badge_svg(&s),
            generate_coverage_badge_svg(&schema())
        );
        assert_eq!(generate_cli_rs(&s), generate_cli_rs(&schema()));
    }

    #[test]
    fn complete_ledger_rows_carry_signatures_tests_and_notes() {
        let s = schema();
        let ledger = generate_ledger_tsv(&s);
        let rows = ledger
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 6);
        assert!(rows.iter().all(|line| line.split('\t').count() == 10));
        assert!(ledger.contains("bases=object\timproved\tBN-01\ttests/a.rs"));
        assert!(ledger.contains("self[positional_or_keyword]"));
        assert!(ledger.contains("kind=int;default=30\tunreviewed\t-\t-\t-"));

        let badge = generate_coverage_badge_svg(&s);
        assert!(badge.contains("4/6 reviewed (66.6%)"), "{badge}");
    }
}
