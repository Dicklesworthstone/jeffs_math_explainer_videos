//! Saved sessions bind data to host-selected configuration, never choose code.
use super::*;
use fmn_studio::supervisor::session::{MAX_SAVED_SESSION_BYTES, SavedSession};

type Prepared = (Option<fmn_studio::ProtocolDigest>, Option<SavedSession>);

pub(super) fn prepare(
    fs: &dyn FileSystem,
    command: &StudioCommand,
    scene: &str,
    config: &fmn_config::Config,
    limits: fmn_studio::ProtocolLimits,
) -> Result<Prepared, CliError> {
    if !studio_live::selected(&command.render) {
        if command.restore_session.is_some() {
            return Err(CliError::new(
                "capability",
                "--restore-session requires a registered live native Studio scene",
            ));
        }
        return Ok((None, None));
    }
    studio_live::validate_selection(&command.render)?;
    // The canonical config includes the fully resolved input settings, with
    // scheduler width normalized by the engine's existing identity contract.
    // Scene identity is separately checked by SavedSession::validate_context.
    let bytes = config
        .canonical_bytes()
        .map_err(|error| internal(error.to_string()))?;
    let context = fmn_studio::protocol_digest(&bytes);
    let Some(path) = &command.restore_session else {
        return Ok((Some(context), None));
    };
    if fs
        .node_kind_no_follow(path)
        .map_err(|error| CliError::new("scene", error.to_string()))?
        != Some(FsNodeKind::RegularFile)
    {
        return Err(CliError::new(
            "scene",
            "saved session must be an existing regular file, not a directory, link or device",
        ));
    }
    let bytes = fs
        .read_bounded(path, MAX_SAVED_SESSION_BYTES)
        .map_err(|error| CliError::new("scene", format!("read saved session: {error}")))?;
    let saved = SavedSession::from_bytes(&bytes, limits)
        .map_err(|error| CliError::new("scene", format!("decode saved session: {error}")))?;
    saved
        .validate_context(scene, context)
        .map_err(|error| CliError::new("scene", format!("saved session context: {error}")))?;
    Ok((Some(context), Some(saved)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_session_flag_is_a_studio_only_typed_path() {
        let Invocation::Studio(command) = parse_args([
            "studio",
            "--restore-session",
            "saved scene.fmns",
            "@builtin",
            "interactive.v1",
        ])
        .unwrap() else {
            panic!("Studio invocation")
        };
        assert_eq!(
            command.restore_session.as_deref(),
            Some(Path::new("saved scene.fmns"))
        );
        assert!(
            parse_args([
                "render",
                "--restore-session",
                "saved.fmns",
                "@builtin",
                "interactive.v1"
            ])
            .is_err()
        );
        assert!(parse_args(["studio", "--restore-session"]).is_err());
    }

    #[test]
    fn restore_session_refuses_unregistered_adapters_before_reading() {
        let fs = fmn_platform::fs::VirtualFs::new();
        let Invocation::Studio(command) = parse_args([
            "studio",
            "--restore-session",
            "missing.fmns",
            "recorded.fmtl",
            "RecordedScene",
        ])
        .unwrap() else {
            panic!("Studio invocation")
        };
        let config = resolve_render_config(&fs, &command.render).unwrap();
        let error = prepare(
            &fs,
            &command,
            "RecordedScene",
            &config,
            fmn_studio::ProtocolLimits::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("registered live"));
    }

    #[test]
    fn ordinary_native_builtins_receive_saved_session_contexts() {
        let fs = fmn_platform::fs::VirtualFs::new();
        for name in [
            "circle_shift.v1",
            "tex_span.v1",
            "formula_sheet.v1",
            "semantic_witness.v1",
            "sound_cue.v1",
            "semantic_witness_camera.v1",
        ] {
            let Invocation::Studio(mut command) = parse_args(["studio", "@builtin", name]).unwrap()
            else {
                panic!("Studio invocation")
            };
            let config = resolve_render_config(&fs, &command.render).unwrap();
            let (context, saved) = prepare(
                &fs,
                &command,
                name,
                &config,
                fmn_studio::ProtocolLimits::default(),
            )
            .unwrap();
            assert!(context.is_some(), "{name}");
            assert!(saved.is_none());
            command.restore_session = Some(PathBuf::from("missing.fmns"));
            let error = prepare(
                &fs,
                &command,
                name,
                &config,
                fmn_studio::ProtocolLimits::default(),
            )
            .unwrap_err();
            assert!(
                error.to_string().contains("existing regular file"),
                "{name}: {error}"
            );
        }
    }
}
