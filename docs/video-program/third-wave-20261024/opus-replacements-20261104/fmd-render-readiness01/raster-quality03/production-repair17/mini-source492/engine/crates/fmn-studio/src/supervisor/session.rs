//! Portable saved Studio sessions over the existing canonical replay journal.
//!
//! A saved session is data, not a worker launch specification: it contains no
//! executable, environment, capability token, or filesystem authority. The host
//! must independently select the scene, configuration, artifact and asset verifier.
//! Resume is strict: a changed input, opaque effect or divergent replay refuses
//! rather than silently returning a partially restored editing session.

use super::*;
use fmn_hash::serial::{Limits, Reader, Schema, UnknownPolicy, Writer};

/// The bounded, checksummed, versioned Studio session container.
pub const SAVED_SESSION_SCHEMA: Schema = Schema::new(*b"FMNS", 1, 1, 0);
/// Total byte ceiling, including the nested journal and checksums.
pub const MAX_SAVED_SESSION_BYTES: usize = 64 * 1024 * 1024;
/// A scene name is metadata, not a place to hide an unbounded document.
const MAX_SCENE_BYTES: usize = 64 * 1024;

/// A complete committed session, independent of the best-effort warm cache.
#[derive(Debug)]
pub struct SavedSession {
    scene: String,
    worker_build: Digest,
    context: Digest,
    committed_frame: i64,
    journal: Journal,
}

fn journal_limit(limits: ProtocolLimits) -> usize {
    limits.max_journal_bytes.min(limits.max_field_bytes)
}

fn container_limits(limits: ProtocolLimits) -> Limits {
    Limits {
        max_total: MAX_SAVED_SESSION_BYTES,
        max_field: limits.max_field_bytes.min(MAX_SAVED_SESSION_BYTES),
    }
}

fn validate_journal(journal: &Journal, limits: ProtocolLimits) -> Result<(), SupervisorError> {
    let checkpoint_limit = limits.max_checkpoint_bytes.min(limits.max_field_bytes);
    for entry in journal.entries() {
        if let Some(bytes) = &entry.checkpoint {
            if bytes.len() > checkpoint_limit {
                return Err(SupervisorError::InvalidSession(
                    "saved checkpoint exceeds protocol budget",
                ));
            }
            // Public content-integrity identifiers, not authentication secrets.
            if sha256(bytes) != entry.state_hash {
                return Err(SupervisorError::InvalidSession(
                    "saved checkpoint checksum mismatch",
                ));
            }
        }
    }
    Ok(())
}

fn require_replayable(
    journal: &Journal,
    asset_ok: &dyn Fn(&AssetRead) -> bool,
) -> Result<Vec<CommandRecord>, SupervisorError> {
    let commands = try_clone_command_records(journal)?;
    let plan = plan_replay(journal, &commands, asset_ok);
    if plan.reuse != commands.len() || plan.reason.is_some() {
        return Err(SupervisorError::InvalidSession(
            "saved session requires unchanged inputs and no opaque replay barriers",
        ));
    }
    Ok(commands)
}

impl SavedSession {
    /// Decode under the receiving supervisor's actual protocol budgets.
    pub fn from_bytes(bytes: &[u8], limits: ProtocolLimits) -> Result<Self, SupervisorError> {
        let mut reader = Reader::open(
            bytes,
            SAVED_SESSION_SCHEMA,
            container_limits(limits),
            UnknownPolicy::Strict,
        )?;
        let scene = reader.get_str()?;
        if scene.is_empty() || scene.len() > MAX_SCENE_BYTES.min(limits.max_field_bytes) {
            return Err(SupervisorError::InvalidSession("invalid saved scene name"));
        }
        let scene = try_clone_string(scene, "saved scene name")?;
        let worker_build = reader.get_digest()?;
        let context = reader.get_digest()?;
        let committed_frame = reader.get_i64()?;
        if committed_frame < 0 {
            return Err(SupervisorError::InvalidSession("negative saved frame"));
        }
        let journal_bytes = reader.get_bytes()?;
        if journal_bytes.len() > journal_limit(limits) {
            return Err(SupervisorError::InvalidSession(
                "saved journal exceeds protocol budget",
            ));
        }
        // Verify the entire outer envelope before allocating a decoded journal.
        reader.finish()?;
        let journal = Journal::from_bytes(journal_bytes)?;
        validate_journal(&journal, limits)?;
        if journal.entries().is_empty() && committed_frame != 0 {
            return Err(SupervisorError::InvalidSession(
                "nonzero saved frame without a committed journal",
            ));
        }
        Ok(Self {
            scene,
            worker_build,
            context,
            committed_frame,
            journal,
        })
    }

    /// Encode the nested journal without depending on cache contents or paths.
    pub fn to_bytes(&self, limits: ProtocolLimits) -> Result<Vec<u8>, SupervisorError> {
        if self.scene.is_empty() || self.scene.len() > MAX_SCENE_BYTES.min(limits.max_field_bytes) {
            return Err(SupervisorError::InvalidSession("invalid saved scene name"));
        }
        if self.committed_frame < 0
            || (self.journal.entries().is_empty() && self.committed_frame != 0)
        {
            return Err(SupervisorError::InvalidSession(
                "invalid saved committed frame",
            ));
        }
        validate_journal(&self.journal, limits)?;
        let bytes = self.journal.to_bytes()?;
        if bytes.len() > journal_limit(limits) {
            return Err(SupervisorError::InvalidSession(
                "saved journal exceeds protocol budget",
            ));
        }
        let mut writer = Writer::with_limits(SAVED_SESSION_SCHEMA, container_limits(limits));
        writer
            .put_str(&self.scene)
            .put_digest(&self.worker_build)
            .put_digest(&self.context)
            .put_i64(self.committed_frame)
            .put_bytes(&bytes);
        Ok(writer.finish()?)
    }

    /// The last committed position, not a transient scrub/playback preview.
    #[must_use]
    pub const fn committed_frame(&self) -> i64 {
        self.committed_frame
    }

    /// Validate against host-selected scene and resolved configuration identity.
    /// No configuration, paths, argv or code are accepted from the saved file.
    pub fn validate_context(&self, scene: &str, context: Digest) -> Result<(), SupervisorError> {
        if self.scene != scene || self.context != context {
            return Err(SupervisorError::InvalidSession(
                "saved scene or render configuration does not match",
            ));
        }
        Ok(())
    }
}

impl Supervisor {
    /// Export only acknowledged commands and their verified checkpoints.
    /// The caller serializes this with commit/restart operations and supplies
    /// the last committed frame, never an in-flight or preview position.
    pub fn save_session(
        &self,
        context: Digest,
        committed_frame: i64,
        asset_ok: &dyn Fn(&AssetRead) -> bool,
    ) -> Result<Vec<u8>, SupervisorError> {
        let scene = self.scene.as_deref().ok_or(SupervisorError::NoSession)?;
        let worker_build = self
            .artifact
            .as_ref()
            .ok_or(SupervisorError::NoWorker)?
            .build_id;
        require_replayable(&self.journal, asset_ok)?;
        SavedSession {
            scene: try_clone_string(scene, "exported session scene")?,
            worker_build,
            context,
            committed_frame,
            journal: self.journal.try_clone()?,
        }
        .to_bytes(self.config.protocol_limits)
    }

    /// Start a new host session from saved data through the ordinary worker
    /// handshake/checkpoint/replay machinery. This is deliberately not an
    /// in-place replacement of a running session; failed imports cannot erase
    /// existing editing work. Caller-selected artifact identity must match.
    pub fn resume_saved_session(
        &mut self,
        builder: &mut dyn RebuildDriver,
        saved: SavedSession,
        scene: &str,
        context: Digest,
        asset_ok: &dyn Fn(&AssetRead) -> bool,
    ) -> Result<RecoveryReport, SupervisorError> {
        if self.generation != 0 || self.scene.is_some() || self.worker.is_some() {
            return Err(SupervisorError::InvalidSession(
                "resume requires a fresh supervisor",
            ));
        }
        saved.validate_context(scene, context)?;
        // SavedSession may have been decoded by an embedder with wider limits.
        // Recheck the complete receiving envelope before rebuilding any worker.
        saved.to_bytes(self.config.protocol_limits)?;
        validate_journal(&saved.journal, self.config.protocol_limits)?;
        let commands = require_replayable(&saved.journal, asset_ok)?;
        let started = self.clock.monotonic();
        let artifact = builder.rebuild()?;
        if artifact.build_id != saved.worker_build {
            return Err(SupervisorError::InvalidSession(
                "saved worker build does not match the selected executable",
            ));
        }
        self.install_session(saved.scene, saved.journal)?;
        self.launch_and_handshake(artifact)?;
        match self.recover(&commands, asset_ok, started) {
            Ok(report)
                if report.plan.reuse == commands.len()
                    && report.diverged_at.is_none()
                    && !report.cold_fallback =>
            {
                Ok(report)
            }
            Ok(_) => {
                self.shutdown_worker();
                Err(SupervisorError::InvalidSession(
                    "saved session replay diverged; refusing partial recovery",
                ))
            }
            Err(error) => {
                self.shutdown_worker();
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_scene::{CommandKind, EffectClass, Entry};

    fn saved() -> SavedSession {
        let mut journal = Journal::new();
        journal
            .record(Entry {
                command: CommandRecord {
                    kind: CommandKind::Play,
                    identity: sha256(b"command"),
                    label: "a committed command".into(),
                },
                effect: EffectClass::Pure,
                reads: vec![AssetRead {
                    path: "native/source".into(),
                    digest: sha256(b"source"),
                }],
                subprocesses: vec![],
                checkpoint: Some(b"checkpoint".to_vec()),
                state_hash: sha256(b"checkpoint"),
            })
            .unwrap();
        SavedSession {
            scene: "Scene Δ".into(),
            worker_build: sha256(b"build"),
            context: sha256(b"config"),
            committed_frame: 12,
            journal,
        }
    }

    #[test]
    fn saved_session_roundtrip_is_canonical_and_self_contained() {
        let limits = ProtocolLimits::default();
        let bytes = saved().to_bytes(limits).unwrap();
        assert_eq!(&bytes[..4], b"FMNS");
        let decoded = SavedSession::from_bytes(&bytes, limits).unwrap();
        assert_eq!(decoded.committed_frame(), 12);
        assert_eq!(decoded.to_bytes(limits).unwrap(), bytes);
        assert_eq!(
            decoded.journal.entries()[0].checkpoint.as_deref(),
            Some(b"checkpoint".as_slice())
        );
        decoded
            .validate_context("Scene Δ", sha256(b"config"))
            .unwrap();
    }

    #[test]
    fn saved_session_corruption_truncation_and_unknown_fields_refuse() {
        let limits = ProtocolLimits::default();
        let bytes = saved().to_bytes(limits).unwrap();
        for end in [0, 4, bytes.len() - 1] {
            assert!(SavedSession::from_bytes(&bytes[..end], limits).is_err());
        }
        let mut damaged = bytes.clone();
        damaged[40] ^= 1;
        assert!(SavedSession::from_bytes(&damaged, limits).is_err());
        let mut writer = Writer::new(SAVED_SESSION_SCHEMA);
        let saved = saved();
        writer
            .put_str(&saved.scene)
            .put_digest(&saved.worker_build)
            .put_digest(&saved.context)
            .put_i64(12)
            .put_bytes(&saved.journal.to_bytes().unwrap())
            .put_u8(99);
        assert!(SavedSession::from_bytes(&writer.finish().unwrap(), limits).is_err());
    }

    #[test]
    fn saved_session_uses_receivers_journal_and_checkpoint_budgets() {
        let bytes = saved().to_bytes(ProtocolLimits::default()).unwrap();
        for limits in [
            ProtocolLimits {
                max_journal_bytes: 1,
                ..ProtocolLimits::default()
            },
            ProtocolLimits {
                max_checkpoint_bytes: 1,
                ..ProtocolLimits::default()
            },
            ProtocolLimits {
                max_field_bytes: 1,
                ..ProtocolLimits::default()
            },
        ] {
            assert!(SavedSession::from_bytes(&bytes, limits).is_err());
            assert!(saved().to_bytes(limits).is_err());
        }
    }

    #[test]
    fn saved_session_cannot_change_scene_config_or_ignore_changed_assets() {
        let saved = saved();
        assert!(saved.validate_context("Other", saved.context).is_err());
        assert!(
            saved
                .validate_context(&saved.scene, sha256(b"changed seed"))
                .is_err()
        );
        assert!(require_replayable(&saved.journal, &|_| false).is_err());
        assert_eq!(
            require_replayable(&saved.journal, &|read| read.digest == sha256(b"source"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn saved_session_rejects_bad_checkpoint_even_with_valid_outer_checksum() {
        let limits = ProtocolLimits::default();
        let saved = saved();
        let mut journal = Journal::new();
        let mut entry = saved.journal.entries()[0].clone();
        entry.state_hash = sha256(b"forged");
        journal.record(entry).unwrap();
        let mut writer = Writer::new(SAVED_SESSION_SCHEMA);
        writer
            .put_str(&saved.scene)
            .put_digest(&saved.worker_build)
            .put_digest(&saved.context)
            .put_i64(12)
            .put_bytes(&journal.to_bytes().unwrap());
        assert!(SavedSession::from_bytes(&writer.finish().unwrap(), limits).is_err());
    }

    #[test]
    fn saved_session_rejects_opaque_effects_instead_of_claiming_recovery() {
        let mut journal = Journal::new();
        let mut entry = saved().journal.entries()[0].clone();
        entry.effect = EffectClass::Opaque;
        journal.record(entry).unwrap();
        assert!(require_replayable(&journal, &|_| true).is_err());
    }

    #[test]
    fn saved_session_empty_zero_is_valid_but_negative_or_uncommitted_frames_refuse() {
        let limits = ProtocolLimits::default();
        let mut saved = saved();
        saved.committed_frame = -1;
        assert!(saved.to_bytes(limits).is_err());
        saved.committed_frame = 12;
        saved.journal = Journal::new();
        assert!(saved.to_bytes(limits).is_err());
        saved.committed_frame = 0;
        let bytes = saved.to_bytes(limits).unwrap();
        assert_eq!(
            SavedSession::from_bytes(&bytes, limits)
                .unwrap()
                .committed_frame(),
            0
        );
    }
}
