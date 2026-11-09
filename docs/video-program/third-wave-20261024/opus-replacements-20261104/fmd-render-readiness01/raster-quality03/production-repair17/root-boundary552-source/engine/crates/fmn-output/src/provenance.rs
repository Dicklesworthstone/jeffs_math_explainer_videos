//! Canonical FMNP input-closure manifests (plan §16.7).
//!
//! A manifest is a durable statement about *why* an artifact has its bytes,
//! not a receipt that a command happened to run.  The closure therefore
//! requires an explicit contribution from every C1--C10 class, orders byte
//! inputs by virtual path, and recomputes its headline digest when decoding.

use std::fmt;
use std::fmt::Write as _;

use fmn_hash::{Digest, Limits, Reader, Schema, SerialError, UnknownPolicy, Writer, sha256};

/// The durable provenance document.
pub const PROVENANCE_SCHEMA: Schema = Schema::new(*b"FMNP", 10, 1, 0);
/// Canonical aggregation of the ordered C1--C10 item list.
pub const CLOSURE_SCHEMA: Schema = Schema::new(*b"FMNP", 11, 1, 0);
/// Canonical aggregation of the platform-neutral items only.
pub const SEMANTIC_SCHEMA: Schema = Schema::new(*b"FMNP", 12, 1, 0);
/// Virtual-path prefix of closure items that describe the build platform
/// (target triple, target features). They stay in the closure digest; the
/// semantic digest leaves them out, so two certified platforms that render
/// the same input share one semantic digest (plan §16.7: equal semantic
/// digest implies equal certified bits).
pub const PLATFORM_ITEM_PREFIX: &str = "platform/";

const MAX_ITEMS: usize = 4_096;
const MAX_OUTPUTS: usize = 4_096;

/// Standard or certified determinism contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestMode {
    /// Seeded, build/platform-local determinism.
    Standard,
    /// Cross-platform certified artifact contract.
    Certified,
}

impl ManifestMode {
    /// Stable manifest spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Certified => "certified",
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Standard => 0,
            Self::Certified => 1,
        }
    }

    fn from_code(code: u8) -> Result<Self, ManifestError> {
        match code {
            0 => Ok(Self::Standard),
            1 => Ok(Self::Certified),
            _ => Err(ManifestError::Invalid("unknown determinism mode")),
        }
    }
}

/// One field in a canonical structural C-item document.
#[derive(Clone, Copy, Debug)]
pub enum StructuralField<'a> {
    /// UTF-8 identity.
    Text(&'a str),
    /// Exact bytes.
    Bytes(&'a [u8]),
    /// Unsigned integer.
    U64(u64),
    /// Boolean.
    Bool(bool),
    /// Content digest.
    Digest(Digest),
    /// An explicitly inapplicable/absent influence.
    Absent(&'a str),
}

/// One ordered contribution to C1--C10.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureItem {
    /// C-item number, inclusive 1--10.
    pub item_id: u8,
    /// Virtual path for byte inputs; `None` for structural inputs/absence.
    pub virtual_path: Option<String>,
    /// Raw-byte digest for byte inputs, canonical-document digest otherwise.
    pub digest: Digest,
    /// Stable human-readable identity.
    pub detail: String,
}

impl ClosureItem {
    /// Bind an exact byte input to the virtual path the scene addressed.
    ///
    /// # Errors
    /// [`ManifestError::Invalid`] for an invalid item id or empty path.
    pub fn byte_input(
        item_id: u8,
        virtual_path: impl Into<String>,
        bytes: &[u8],
        detail: impl Into<String>,
    ) -> Result<Self, ManifestError> {
        Self::digest_input(item_id, virtual_path, sha256(bytes), detail)
    }

    /// Bind a byte input by the SHA-256 of its bytes, for inputs hashed
    /// while streaming rather than held (a portal scene's file reads). Equal
    /// to [`Self::byte_input`] over the same bytes.
    ///
    /// # Errors
    /// [`ManifestError::Invalid`] for an invalid item id or empty path.
    pub fn digest_input(
        item_id: u8,
        virtual_path: impl Into<String>,
        digest: Digest,
        detail: impl Into<String>,
    ) -> Result<Self, ManifestError> {
        let virtual_path = virtual_path.into();
        validate_item_id(item_id)?;
        if virtual_path.is_empty() {
            return Err(ManifestError::Invalid(
                "byte input has an empty virtual path",
            ));
        }
        Ok(Self {
            item_id,
            virtual_path: Some(virtual_path),
            digest,
            detail: detail.into(),
        })
    }

    /// Hash a structural item through a schema separated by C-item number.
    ///
    /// # Errors
    /// Canonical serialization failure or an invalid item id.
    pub fn structural(
        item_id: u8,
        detail: impl Into<String>,
        fields: &[StructuralField<'_>],
    ) -> Result<Self, ManifestError> {
        validate_item_id(item_id)?;
        let mut writer = Writer::new(Schema::new(*b"FMNP", 100 + u32::from(item_id), 1, 0));
        writer.put_u8(item_id);
        writer.put_u32(wire_count(fields.len())?);
        for field in fields {
            match field {
                StructuralField::Text(value) => {
                    writer.put_u8(1).put_str(value);
                }
                StructuralField::Bytes(value) => {
                    writer.put_u8(2).put_bytes(value);
                }
                StructuralField::U64(value) => {
                    writer.put_u8(3).put_u64(*value);
                }
                StructuralField::Bool(value) => {
                    writer.put_u8(4).put_bool(*value);
                }
                StructuralField::Digest(value) => {
                    writer.put_u8(5).put_digest(value);
                }
                StructuralField::Absent(reason) => {
                    writer.put_u8(6).put_str(reason);
                }
            }
        }
        Ok(Self {
            item_id,
            virtual_path: None,
            digest: sha256(&writer.finish()?),
            detail: detail.into(),
        })
    }

    /// A structural item addressed by a virtual path, so one C-class can
    /// carry several structural contributions (e.g. C3's platform-neutral
    /// toolchain and its `platform/` target record).
    ///
    /// # Errors
    /// As [`Self::structural`], or an empty path.
    pub fn structural_at(
        item_id: u8,
        virtual_path: impl Into<String>,
        detail: impl Into<String>,
        fields: &[StructuralField<'_>],
    ) -> Result<Self, ManifestError> {
        let virtual_path = virtual_path.into();
        if virtual_path.is_empty() {
            return Err(ManifestError::Invalid(
                "structural item has an empty virtual path",
            ));
        }
        let mut item = Self::structural(item_id, detail, fields)?;
        item.virtual_path = Some(virtual_path);
        Ok(item)
    }

    /// Whether this item describes the build platform rather than the
    /// render's semantic input ([`PLATFORM_ITEM_PREFIX`]).
    #[must_use]
    pub fn is_platform_specific(&self) -> bool {
        self.virtual_path
            .as_deref()
            .is_some_and(|path| path.starts_with(PLATFORM_ITEM_PREFIX))
    }
}

/// Redundant high-value identities rendered prominently in a manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestIdentity {
    /// Exact git commit or release build identity.
    pub build_id: String,
    /// Full `SUITE.lock` digest.
    pub suite_lock_digest: Digest,
    /// Exact pinned Rust nightly.
    pub toolchain: String,
    /// Rust target triple compiled into the executable.
    pub target_triple: String,
    /// SUITE.lock target-feature set.
    pub target_features: String,
    /// Semantic renderer/execution-engine identity.
    pub engine: String,
    /// Compiled SIMD tier.
    pub simd_tier: String,
    /// C10 declared-configuration digest.
    pub declared_config_digest: Digest,
}

/// One published artifact identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestOutput {
    /// Virtual artifact path.
    pub virtual_path: String,
    /// Stable artifact kind.
    pub kind: String,
    /// Raw-file or canonical artifact-tree digest.
    pub digest: Digest,
    /// Whether this output participates in the certified bit promise.
    pub certified: bool,
}

/// What [`ProvenanceManifest::compare`] found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestComparison {
    /// Equal platform-neutral semantic digests: the same render input.
    pub semantic_equal: bool,
    /// Equal full closure digests (same platform as well).
    pub closure_equal: bool,
    /// Both manifests are certified, so their certified outputs are bound
    /// to be bit-identical when the semantic digests agree.
    pub certified_both: bool,
    /// Certified output paths whose digests differ or which only one side
    /// published.
    pub differing_outputs: Vec<String>,
}

impl ManifestComparison {
    /// The certified promise holds: same semantic input, both certified,
    /// every certified output bit-identical.
    #[must_use]
    pub fn certified_bits_agree(&self) -> bool {
        self.semantic_equal && self.certified_both && self.differing_outputs.is_empty()
    }
}

/// Complete FMNP sidecar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenanceManifest {
    /// Determinism contract.
    pub mode: ManifestMode,
    /// Headline digest over `items` only.
    pub closure_digest: Digest,
    /// Ordered complete C1--C10 closure.
    pub items: Vec<ClosureItem>,
    /// Prominent build/toolchain/execution identities.
    pub identity: ManifestIdentity,
    /// Published artifacts.
    pub outputs: Vec<ManifestOutput>,
    /// Replay journal identity when a journal is live.
    pub journal_ref: Option<Digest>,
}

impl ProvenanceManifest {
    /// Validate, sort, and seal a complete input closure.
    ///
    /// # Errors
    /// Missing C1--C10 classes, duplicate item identities, invalid outputs,
    /// or canonical serialization failure.
    pub fn new(
        mode: ManifestMode,
        mut items: Vec<ClosureItem>,
        identity: ManifestIdentity,
        outputs: Vec<ManifestOutput>,
        journal_ref: Option<Digest>,
    ) -> Result<Self, ManifestError> {
        items.sort_by(|left, right| {
            (left.item_id, left.virtual_path.as_deref().unwrap_or(""))
                .cmp(&(right.item_id, right.virtual_path.as_deref().unwrap_or("")))
        });
        validate_items(&items)?;
        validate_identity(&items, &identity)?;
        validate_outputs(&outputs)?;
        if mode == ManifestMode::Certified && certified_build_refusal(&identity.build_id).is_some()
        {
            return Err(ManifestError::Invalid(
                "a certified manifest requires an identified build (ADR-0025)",
            ));
        }
        let closure_digest = closure_digest(&items)?;
        Ok(Self {
            mode,
            closure_digest,
            items,
            identity,
            outputs,
            journal_ref,
        })
    }

    /// Serialize the durable canonical FMNP/1 document.
    ///
    /// # Errors
    /// Canonical serialization failure.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ManifestError> {
        validate_items(&self.items)?;
        validate_identity(&self.items, &self.identity)?;
        validate_outputs(&self.outputs)?;
        if closure_digest(&self.items)? != self.closure_digest {
            return Err(ManifestError::DigestMismatch);
        }
        let mut writer = Writer::new(PROVENANCE_SCHEMA);
        writer.put_u8(self.mode.code());
        writer.put_digest(&self.closure_digest);
        put_identity(&mut writer, &self.identity);
        writer.put_u32(wire_count(self.items.len())?);
        for item in &self.items {
            writer.put_u8(item.item_id);
            writer.put_bool(item.virtual_path.is_some());
            if let Some(path) = &item.virtual_path {
                writer.put_str(path);
            }
            writer.put_digest(&item.digest);
            writer.put_str(&item.detail);
        }
        writer.put_u32(wire_count(self.outputs.len())?);
        for output in &self.outputs {
            writer.put_str(&output.virtual_path);
            writer.put_str(&output.kind);
            writer.put_digest(&output.digest);
            writer.put_bool(output.certified);
        }
        writer.put_bool(self.journal_ref.is_some());
        if let Some(journal_ref) = self.journal_ref {
            writer.put_digest(&journal_ref);
        }
        Ok(writer.finish()?)
    }

    /// Decode and independently verify a canonical FMNP/1 document.
    ///
    /// # Errors
    /// Framing/field errors, resource-limit violations, incomplete C1--C10,
    /// or a headline closure digest that does not match the item list.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ManifestError> {
        let mut reader = Reader::open(
            bytes,
            PROVENANCE_SCHEMA,
            Limits::DEFAULT,
            UnknownPolicy::Strict,
        )?;
        let mode = ManifestMode::from_code(reader.get_u8()?)?;
        let stated_digest = reader.get_digest()?;
        let identity = get_identity(&mut reader)?;
        let item_count = bounded_count(reader.get_u32()?, MAX_ITEMS, "closure item")?;
        let mut items = Vec::new();
        items
            .try_reserve(item_count)
            .map_err(|_| ManifestError::AllocationFailed("closure items"))?;
        for _ in 0..item_count {
            let item_id = reader.get_u8()?;
            let virtual_path = if reader.get_bool()? {
                Some(owned(reader.get_str()?, "closure virtual path")?)
            } else {
                None
            };
            items.push(ClosureItem {
                item_id,
                virtual_path,
                digest: reader.get_digest()?,
                detail: owned(reader.get_str()?, "closure detail")?,
            });
        }
        let output_count = bounded_count(reader.get_u32()?, MAX_OUTPUTS, "manifest output")?;
        let mut outputs = Vec::new();
        outputs
            .try_reserve(output_count)
            .map_err(|_| ManifestError::AllocationFailed("manifest outputs"))?;
        for _ in 0..output_count {
            outputs.push(ManifestOutput {
                virtual_path: owned(reader.get_str()?, "output virtual path")?,
                kind: owned(reader.get_str()?, "output kind")?,
                digest: reader.get_digest()?,
                certified: reader.get_bool()?,
            });
        }
        let journal_ref = if reader.get_bool()? {
            Some(reader.get_digest()?)
        } else {
            None
        };
        reader.finish()?;
        let manifest = Self::new(mode, items, identity, outputs, journal_ref)?;
        if manifest.closure_digest != stated_digest {
            return Err(ManifestError::DigestMismatch);
        }
        Ok(manifest)
    }

    /// The platform-neutral semantic closure digest: the closure without
    /// its [`PLATFORM_ITEM_PREFIX`] items. Two certified manifests with
    /// equal semantic digests rendered the same input, so their certified
    /// outputs must be bit-identical (plan §16.7).
    ///
    /// # Errors
    /// Canonical serialization failure.
    pub fn semantic_digest(&self) -> Result<Digest, ManifestError> {
        semantic_digest(&self.items)
    }

    /// Compare two manifests the way the certified-matrix verifier does:
    /// equal semantic digests, and then every certified output digest.
    ///
    /// # Errors
    /// Canonical serialization failure.
    pub fn compare(&self, other: &Self) -> Result<ManifestComparison, ManifestError> {
        let semantic_equal = self.semantic_digest()? == other.semantic_digest()?;
        let certified = |manifest: &Self| -> std::collections::BTreeMap<String, Digest> {
            manifest
                .outputs
                .iter()
                .filter(|output| output.certified)
                .map(|output| (output.virtual_path.clone(), output.digest))
                .collect()
        };
        let (left, right) = (certified(self), certified(other));
        let mut differing_outputs: Vec<String> = left
            .iter()
            .filter(|(path, digest)| right.get(*path) != Some(*digest))
            .map(|(path, _)| path.clone())
            .collect();
        differing_outputs.extend(
            right
                .keys()
                .filter(|path| !left.contains_key(*path))
                .cloned(),
        );
        differing_outputs.sort();
        Ok(ManifestComparison {
            semantic_equal,
            closure_equal: self.closure_digest == other.closure_digest,
            certified_both: self.mode == ManifestMode::Certified
                && other.mode == ManifestMode::Certified,
            differing_outputs,
        })
    }

    /// Human-readable rendering paired with the canonical binary document.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "manifest_version = \"1.0\"");
        let _ = writeln!(text, "mode = {:?}", self.mode.name());
        let _ = writeln!(text, "closure_digest = \"{}\"", self.closure_digest);
        if let Ok(semantic) = self.semantic_digest() {
            let _ = writeln!(text, "semantic_digest = \"{semantic}\"");
        }
        let _ = writeln!(text, "build_id = {:?}", self.identity.build_id);
        let _ = writeln!(
            text,
            "suite_lock_digest = \"{}\"",
            self.identity.suite_lock_digest
        );
        let _ = writeln!(text, "toolchain = {:?}", self.identity.toolchain);
        let _ = writeln!(text, "target_triple = {:?}", self.identity.target_triple);
        let _ = writeln!(
            text,
            "target_features = {:?}",
            self.identity.target_features
        );
        let _ = writeln!(text, "engine = {:?}", self.identity.engine);
        let _ = writeln!(text, "simd_tier = {:?}", self.identity.simd_tier);
        let _ = writeln!(
            text,
            "declared_config_digest = \"{}\"",
            self.identity.declared_config_digest
        );
        for item in &self.items {
            let _ = writeln!(text, "\n[[items]]");
            let _ = writeln!(text, "id = {}", item.item_id);
            if let Some(path) = &item.virtual_path {
                let _ = writeln!(text, "virtual_path = {path:?}");
            }
            let _ = writeln!(text, "digest = \"{}\"", item.digest);
            let _ = writeln!(text, "detail = {:?}", item.detail);
        }
        for output in &self.outputs {
            let _ = writeln!(text, "\n[[outputs]]");
            let _ = writeln!(text, "virtual_path = {:?}", output.virtual_path);
            let _ = writeln!(text, "kind = {:?}", output.kind);
            let _ = writeln!(text, "digest = \"{}\"", output.digest);
            let _ = writeln!(text, "certified = {}", output.certified);
        }
        if let Some(journal_ref) = self.journal_ref {
            let _ = writeln!(text, "\njournal_ref = \"{journal_ref}\"");
        }
        text
    }
}

/// Manifest construction/verification failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    /// Canonical container failure.
    Serial(SerialError),
    /// Structurally invalid manifest.
    Invalid(&'static str),
    /// A wire count exceeded a deliberate decoder budget.
    CountLimit {
        /// Field name.
        field: &'static str,
        /// Observed count.
        count: usize,
        /// Maximum admitted count.
        max: usize,
    },
    /// Fallible ownership allocation failed.
    AllocationFailed(&'static str),
    /// The stated headline digest did not match the canonical item list.
    DigestMismatch,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial(error) => write!(f, "FMNP container: {error}"),
            Self::Invalid(detail) => write!(f, "invalid FMNP manifest: {detail}"),
            Self::CountLimit { field, count, max } => {
                write!(f, "FMNP {field} count {count} exceeds limit {max}")
            }
            Self::AllocationFailed(field) => {
                write!(f, "FMNP could not reserve storage for {field}")
            }
            Self::DigestMismatch => f.write_str("FMNP closure digest does not match its item list"),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serial(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SerialError> for ManifestError {
    fn from(error: SerialError) -> Self {
        Self::Serial(error)
    }
}

fn validate_item_id(item_id: u8) -> Result<(), ManifestError> {
    if (1..=10).contains(&item_id) {
        Ok(())
    } else {
        Err(ManifestError::Invalid("closure item id is outside C1--C10"))
    }
}

fn validate_items(items: &[ClosureItem]) -> Result<(), ManifestError> {
    if items.len() > MAX_ITEMS {
        return Err(ManifestError::CountLimit {
            field: "closure item",
            count: items.len(),
            max: MAX_ITEMS,
        });
    }
    let mut present = [false; 10];
    let mut previous: Option<(u8, Option<&str>)> = None;
    for item in items {
        validate_item_id(item.item_id)?;
        if item.detail.is_empty() {
            return Err(ManifestError::Invalid("closure item detail is empty"));
        }
        let identity = (item.item_id, item.virtual_path.as_deref());
        if let Some(previous) = previous {
            if previous == identity {
                return Err(ManifestError::Invalid("duplicate closure item identity"));
            }
            if previous > identity {
                return Err(ManifestError::Invalid(
                    "closure items are not in canonical order",
                ));
            }
        }
        previous = Some(identity);
        present[usize::from(item.item_id - 1)] = true;
    }
    if present.iter().all(|value| *value) {
        Ok(())
    } else {
        Err(ManifestError::Invalid(
            "closure does not contain every C1--C10 class",
        ))
    }
}

fn validate_outputs(outputs: &[ManifestOutput]) -> Result<(), ManifestError> {
    if outputs.is_empty() {
        return Err(ManifestError::Invalid("manifest has no output artifact"));
    }
    if outputs.len() > MAX_OUTPUTS {
        return Err(ManifestError::CountLimit {
            field: "output",
            count: outputs.len(),
            max: MAX_OUTPUTS,
        });
    }
    if outputs
        .iter()
        .any(|output| output.virtual_path.is_empty() || output.kind.is_empty())
    {
        return Err(ManifestError::Invalid("manifest output identity is empty"));
    }
    Ok(())
}

/// Why this build cannot sign certified output, if it cannot (ADR-0025).
///
/// A certified closure must name the sources it was built from. A clean
/// commit (`git:<commit>`), a commit plus the digest of its uncommitted
/// compiled inputs (`git:<commit>+dirty:<sha256>`), and an explicit
/// `FMN_BUILD_ID` all do. A build without git metadata or `FMN_BUILD_ID`
/// (`unidentified:`), or one whose tree state git could not report
/// (`+unverified`), does not.
#[must_use]
pub fn certified_build_refusal(build_id: &str) -> Option<&'static str> {
    if build_id.starts_with("unidentified:") {
        Some(
            "certified output requires an identified build: this binary was built without git \
             metadata or FMN_BUILD_ID, so its closure cannot name its sources",
        )
    } else if build_id.ends_with("+unverified") {
        Some(
            "certified output requires an identified build: git could not report the tree state \
             this binary was built from",
        )
    } else {
        None
    }
}

fn validate_identity(
    items: &[ClosureItem],
    identity: &ManifestIdentity,
) -> Result<(), ManifestError> {
    let build = items.iter().find(|item| {
        item.item_id == 2 && item.virtual_path.as_deref() == Some("franken_manim.build")
    });
    if build.is_none_or(|item| item.digest != sha256(identity.build_id.as_bytes())) {
        return Err(ManifestError::Invalid(
            "prominent build identity disagrees with C2",
        ));
    }
    let suite = items
        .iter()
        .find(|item| item.item_id == 2 && item.virtual_path.as_deref() == Some("SUITE.lock"));
    if suite.is_none_or(|item| item.digest != identity.suite_lock_digest) {
        return Err(ManifestError::Invalid(
            "prominent SUITE.lock identity disagrees with C2",
        ));
    }
    let declared = items
        .iter()
        .find(|item| item.item_id == 10 && item.virtual_path.is_none());
    if declared.is_none_or(|item| item.digest != identity.declared_config_digest) {
        return Err(ManifestError::Invalid(
            "prominent declared configuration disagrees with C10",
        ));
    }
    Ok(())
}

fn closure_digest(items: &[ClosureItem]) -> Result<Digest, ManifestError> {
    let mut writer = Writer::new(CLOSURE_SCHEMA);
    writer.put_u32(wire_count(items.len())?);
    for item in items {
        writer.put_u8(item.item_id);
        writer.put_bool(item.virtual_path.is_some());
        if let Some(path) = &item.virtual_path {
            writer.put_str(path);
        }
        writer.put_digest(&item.digest);
    }
    Ok(sha256(&writer.finish()?))
}

fn semantic_digest(items: &[ClosureItem]) -> Result<Digest, ManifestError> {
    let neutral: Vec<&ClosureItem> = items
        .iter()
        .filter(|item| !item.is_platform_specific())
        .collect();
    let mut writer = Writer::new(SEMANTIC_SCHEMA);
    writer.put_u32(wire_count(neutral.len())?);
    for item in neutral {
        writer.put_u8(item.item_id);
        writer.put_bool(item.virtual_path.is_some());
        if let Some(path) = &item.virtual_path {
            writer.put_str(path);
        }
        writer.put_digest(&item.digest);
    }
    Ok(sha256(&writer.finish()?))
}

fn put_identity(writer: &mut Writer, identity: &ManifestIdentity) {
    writer.put_str(&identity.build_id);
    writer.put_digest(&identity.suite_lock_digest);
    writer.put_str(&identity.toolchain);
    writer.put_str(&identity.target_triple);
    writer.put_str(&identity.target_features);
    writer.put_str(&identity.engine);
    writer.put_str(&identity.simd_tier);
    writer.put_digest(&identity.declared_config_digest);
}

fn get_identity(reader: &mut Reader<'_>) -> Result<ManifestIdentity, ManifestError> {
    Ok(ManifestIdentity {
        build_id: owned(reader.get_str()?, "build identity")?,
        suite_lock_digest: reader.get_digest()?,
        toolchain: owned(reader.get_str()?, "toolchain identity")?,
        target_triple: owned(reader.get_str()?, "target triple")?,
        target_features: owned(reader.get_str()?, "target features")?,
        engine: owned(reader.get_str()?, "engine identity")?,
        simd_tier: owned(reader.get_str()?, "SIMD tier")?,
        declared_config_digest: reader.get_digest()?,
    })
}

fn wire_count(count: usize) -> Result<u32, ManifestError> {
    u32::try_from(count).map_err(|_| ManifestError::Invalid("manifest count exceeds u32"))
}

fn bounded_count(count: u32, max: usize, field: &'static str) -> Result<usize, ManifestError> {
    let count = usize::try_from(count).map_err(|_| ManifestError::CountLimit {
        field,
        count: usize::MAX,
        max,
    })?;
    if count > max {
        Err(ManifestError::CountLimit { field, count, max })
    } else {
        Ok(count)
    }
}

fn owned(value: &str, field: &'static str) -> Result<String, ManifestError> {
    let mut result = String::new();
    result
        .try_reserve(value.len())
        .map_err(|_| ManifestError::AllocationFailed(field))?;
    result.push_str(value);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_manifest() -> ProvenanceManifest {
        manifest_with(ManifestMode::Certified, "git:0123456789abcdef").expect("complete manifest")
    }

    fn manifest_with(
        mode: ManifestMode,
        build_id: &str,
    ) -> Result<ProvenanceManifest, ManifestError> {
        let mut items = vec![
            ClosureItem::structural(1, "C1", &[StructuralField::U64(1)]).expect("valid C1"),
            ClosureItem::byte_input(2, "franken_manim.build", build_id.as_bytes(), "build")
                .expect("valid build item"),
            ClosureItem::byte_input(2, "SUITE.lock", b"suite", "suite lock")
                .expect("valid suite item"),
        ];
        for id in 3..=10 {
            items.push(
                ClosureItem::structural(
                    id,
                    format!("C{id}"),
                    &[StructuralField::U64(u64::from(id))],
                )
                .expect("valid structural item"),
            );
        }
        let declared_config_digest = items
            .iter()
            .find(|item| item.item_id == 10)
            .expect("C10 item")
            .digest;
        ProvenanceManifest::new(
            mode,
            items,
            ManifestIdentity {
                build_id: build_id.to_owned(),
                suite_lock_digest: sha256(b"suite"),
                toolchain: "nightly-test".to_owned(),
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                target_features: "baseline".to_owned(),
                engine: "certified-cpu:scalar:1".to_owned(),
                simd_tier: "portable".to_owned(),
                declared_config_digest,
            },
            vec![ManifestOutput {
                virtual_path: "scene/frame_000000.png".to_owned(),
                kind: "canonical_png".to_owned(),
                digest: sha256(b"png"),
                certified: true,
            }],
            None,
        )
    }

    #[test]
    fn certified_output_requires_a_build_that_names_its_sources() {
        let commit = "git:0123456789abcdef0123456789abcdef01234567";
        let dirty = format!("{commit}+dirty:{}", "a".repeat(64));
        for identified in [commit, dirty.as_str(), "release:fmn-cli:0.4.0"] {
            assert_eq!(certified_build_refusal(identified), None, "{identified}");
            assert!(
                manifest_with(ManifestMode::Certified, identified).is_ok(),
                "{identified}"
            );
        }
        for unnamed in [
            "unidentified:fmn-cli:0.4.0",
            "git:0123456789abcdef+unverified",
        ] {
            assert!(certified_build_refusal(unnamed).is_some(), "{unnamed}");
            assert_eq!(
                manifest_with(ManifestMode::Certified, unnamed),
                Err(ManifestError::Invalid(
                    "a certified manifest requires an identified build (ADR-0025)"
                )),
                "{unnamed}"
            );
            // Standard output claims no cross-platform identity; it records
            // whatever identity the build has.
            assert!(
                manifest_with(ManifestMode::Standard, unnamed).is_ok(),
                "{unnamed}"
            );
        }
    }

    #[test]
    fn a_streamed_digest_binds_the_same_input_as_its_bytes() {
        let bytes = b"x,y\n1,2\n";
        let held = ClosureItem::byte_input(6, "read/a/data.csv", bytes, "scene read")
            .expect("valid byte input");
        let streamed = ClosureItem::digest_input(6, "read/a/data.csv", sha256(bytes), "scene read")
            .expect("valid digest input");
        assert_eq!(held, streamed);
        let other = ClosureItem::digest_input(6, "read/a/data.csv", sha256(b"x,y\n1,3\n"), "r")
            .expect("valid digest input");
        assert_ne!(held.digest, other.digest);
        assert!(ClosureItem::digest_input(6, "", sha256(bytes), "scene read").is_err());
        assert!(ClosureItem::digest_input(11, "read/a", sha256(bytes), "scene read").is_err());
    }

    #[test]
    fn fmnp_round_trip_recomputes_the_closure() {
        let manifest = complete_manifest();
        let bytes = manifest.to_bytes().expect("encode manifest");
        assert_eq!(ProvenanceManifest::from_bytes(&bytes), Ok(manifest));
    }

    #[test]
    fn incomplete_closure_is_refused() {
        let mut manifest = complete_manifest();
        manifest.items.pop();
        assert!(matches!(
            manifest.to_bytes(),
            Err(ManifestError::Invalid(
                "closure does not contain every C1--C10 class"
            ))
        ));
    }

    #[test]
    fn mutated_noncanonical_order_is_refused() {
        let mut manifest = complete_manifest();
        manifest.items.swap(0, 1);
        assert!(matches!(
            manifest.to_bytes(),
            Err(ManifestError::Invalid(
                "closure items are not in canonical order"
            ))
        ));
    }

    /// A certified manifest as one platform writes it: C3 split into its
    /// platform-neutral toolchain item and a `platform/target` item.
    fn manifest_on(triple: &str, config: u64, png: &[u8]) -> ProvenanceManifest {
        let build_id = "git:0123456789abcdef";
        let mut items = vec![
            ClosureItem::structural(1, "C1", &[StructuralField::U64(1)]).expect("valid C1"),
            ClosureItem::byte_input(2, "franken_manim.build", build_id.as_bytes(), "build")
                .expect("valid build item"),
            ClosureItem::byte_input(2, "SUITE.lock", b"suite", "suite lock")
                .expect("valid suite item"),
            ClosureItem::structural(3, "toolchain", &[StructuralField::Text("nightly-test")])
                .expect("valid C3 toolchain"),
            ClosureItem::structural_at(
                3,
                "platform/target",
                "target",
                &[StructuralField::Text(triple)],
            )
            .expect("valid C3 platform"),
            ClosureItem::structural(4, "C4", &[StructuralField::U64(config)]).expect("valid C4"),
        ];
        for id in 5..=10 {
            items.push(
                ClosureItem::structural(
                    id,
                    format!("C{id}"),
                    &[StructuralField::U64(u64::from(id))],
                )
                .expect("valid structural item"),
            );
        }
        let declared_config_digest = items
            .iter()
            .find(|item| item.item_id == 10)
            .expect("C10 item")
            .digest;
        ProvenanceManifest::new(
            ManifestMode::Certified,
            items,
            ManifestIdentity {
                build_id: build_id.to_owned(),
                suite_lock_digest: sha256(b"suite"),
                toolchain: "nightly-test".to_owned(),
                target_triple: triple.to_owned(),
                target_features: "baseline".to_owned(),
                engine: "certified-cpu:scalar:1".to_owned(),
                simd_tier: "portable".to_owned(),
                declared_config_digest,
            },
            vec![ManifestOutput {
                virtual_path: "scene/frame_000000.png".to_owned(),
                kind: "canonical_png".to_owned(),
                digest: sha256(png),
                certified: true,
            }],
            None,
        )
        .expect("platform manifest")
    }

    #[test]
    fn the_semantic_digest_ignores_only_the_platform_record() {
        let x86 = manifest_on("x86_64-unknown-linux-gnu", 7, b"png");
        let arm = manifest_on("aarch64-apple-darwin", 7, b"png");
        assert_ne!(x86.closure_digest, arm.closure_digest);
        assert_eq!(x86.semantic_digest(), arm.semantic_digest());
        let across = x86.compare(&arm).expect("compare");
        assert!(across.semantic_equal && !across.closure_equal);
        assert!(across.certified_bits_agree(), "{across:?}");
        assert!(x86.to_text().contains(&format!(
            "semantic_digest = \"{}\"",
            x86.semantic_digest().expect("digest")
        )));
        // Round trip keeps the platform item and so the semantic digest.
        let decoded =
            ProvenanceManifest::from_bytes(&x86.to_bytes().expect("encode")).expect("decode");
        assert_eq!(decoded.semantic_digest(), x86.semantic_digest());

        // A different render input is a different semantic closure.
        let other_config = manifest_on("aarch64-apple-darwin", 8, b"png");
        let changed = x86.compare(&other_config).expect("compare");
        assert!(!changed.semantic_equal && !changed.certified_bits_agree());

        // Same input, different certified bits: the promise is broken and
        // the verifier names the output.
        let drifted = manifest_on("aarch64-apple-darwin", 7, b"other png");
        let broken = x86.compare(&drifted).expect("compare");
        assert!(broken.semantic_equal && !broken.certified_bits_agree());
        assert_eq!(broken.differing_outputs, ["scene/frame_000000.png"]);

        assert!(matches!(
            ClosureItem::structural_at(3, "", "empty", &[]),
            Err(ManifestError::Invalid(
                "structural item has an empty virtual path"
            ))
        ));
    }

    /// `item_id` names the C1--C10 class, so one class can hold several
    /// items (C2 carries both the build and SUITE.lock). What must be
    /// unique is an item's identity, (class, virtual path), and a repeated
    /// identity is refused rather than silently double-counted.
    #[test]
    fn a_class_holds_many_items_but_an_identity_appears_once() {
        let manifest = complete_manifest();
        assert_eq!(
            manifest
                .items
                .iter()
                .filter(|item| item.item_id == 2)
                .count(),
            2
        );
        assert!(manifest.to_bytes().is_ok());
        let mut duplicated = complete_manifest();
        let at = duplicated
            .items
            .iter()
            .position(|item| item.virtual_path.as_deref() == Some("SUITE.lock"))
            .expect("the SUITE.lock item");
        let suite = duplicated.items[at].clone();
        duplicated.items.insert(at + 1, suite);
        assert!(matches!(
            duplicated.to_bytes(),
            Err(ManifestError::Invalid("duplicate closure item identity"))
        ));
    }

    #[test]
    fn prominent_identity_cannot_disagree_with_the_closure() {
        let mut manifest = complete_manifest();
        manifest.identity.build_id = "git:ffffffffffffffff".to_owned();
        assert!(matches!(
            manifest.to_bytes(),
            Err(ManifestError::Invalid(
                "prominent build identity disagrees with C2"
            ))
        ));
    }

    #[test]
    fn text_names_every_closure_class_and_output_digest() {
        let manifest = complete_manifest();
        let text = manifest.to_text();
        for id in 1..=10 {
            assert!(text.contains(&format!("id = {id}")));
        }
        assert!(text.contains(&manifest.closure_digest.to_hex()));
        assert!(text.contains(&manifest.outputs[0].digest.to_hex()));
    }
}
