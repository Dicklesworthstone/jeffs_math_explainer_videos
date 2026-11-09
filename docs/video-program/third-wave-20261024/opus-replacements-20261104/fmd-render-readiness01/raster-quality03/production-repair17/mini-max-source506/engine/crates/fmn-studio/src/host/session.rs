//! Export committed replay data, never a filesystem path or process authority.
use super::*;

impl StudioWorkerSession {
    /// Enable portable session downloads for this host-selected input context.
    /// `committed_frame` comes from a validated resume or zero for a new session.
    pub fn with_saved_session_context(
        mut self,
        context: Digest,
        committed_frame: i64,
    ) -> Result<Self, HostError> {
        if committed_frame < 0 {
            return Err(HostError::Configuration("negative restored session frame"));
        }
        self.session_context = Some(context);
        self.committed_frame
            .store(committed_frame, Ordering::Release);
        Ok(self)
    }

    /// Snapshot only acknowledged journal entries under the host operation lock.
    /// Preview playback does not implicitly become a committed edit or position.
    pub fn save_session(&self) -> Result<Vec<u8>, HostError> {
        let _operation = lock(&self.operations);
        let context = self.session_context.ok_or(HostError::BadRequest(
            "saved sessions are unavailable for this scene adapter",
        ))?;
        let supervisor = lock(&self.supervisor);
        Ok(supervisor.save_session(
            context,
            self.committed_frame.load(Ordering::Acquire),
            &*self.asset_ok,
        )?)
    }
}

impl HostHandler {
    pub(super) fn saved_session(&self, stream: &mut TcpStream) -> Result<(), HostError> {
        let bytes = self.session.save_session()?;
        write_response_with_headers(
            stream,
            200,
            "OK",
            "application/vnd.frankenmanim.studio-session",
            &bytes,
            &[("X-FMN-SHA256", &sha256(&bytes).to_hex())],
        )
    }
}
