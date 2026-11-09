//! Immutable executable generations owned by the development build host.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::fail;
use crate::{BuildError, ProtocolDigest, protocol_digest};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(super) struct ArtifactStore {
    parent: PathBuf,
    directory: Option<PathBuf>,
    images: Vec<(ProtocolDigest, PathBuf)>,
}

impl ArtifactStore {
    pub fn new(parent: PathBuf) -> Self {
        Self {
            parent,
            directory: None,
            images: Vec::new(),
        }
    }

    pub fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    pub fn publish(
        &mut self,
        source: &Path,
        max_bytes: u64,
        max_images: usize,
    ) -> Result<(PathBuf, ProtocolDigest), BuildError> {
        let (bytes, permissions) = read_image(source, max_bytes)?;
        let digest = protocol_digest(&bytes);
        if let Some((_, path)) = self.images.iter().find(|(key, _)| *key == digest) {
            let (stored, _) = read_image(path, max_bytes)?;
            if stored != bytes {
                return Err(fail(
                    "a published worker image changed; refusing to reuse it",
                ));
            }
            return Ok((path.clone(), digest));
        }
        if self.images.len() >= max_images {
            return Err(fail(
                "retained worker-image limit reached; start a new build session",
            ));
        }
        self.images.try_reserve(1).map_err(fail)?;
        if self.directory.is_none() {
            fs::create_dir_all(&self.parent).map_err(fail)?;
            // create_dir is exclusive. Never reuse an existing directory or
            // follow an attacker-created candidate symlink. This is not a
            // sandbox against a hostile owner of the supplied parent directory.
            for _ in 0..128 {
                let id = NEXT_DIRECTORY
                    .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                    .map_err(|_| fail("worker-image namespace exhausted"))?;
                let directory = self
                    .parent
                    .join(format!("fmn-worker-{}-{id}", std::process::id()));
                let mut builder = fs::DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                match builder.create(&directory) {
                    Ok(()) => {
                        self.directory = Some(directory);
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(fail(error)),
                }
            }
        }
        let directory = self
            .directory
            .as_ref()
            .ok_or_else(|| fail("cannot reserve a private image directory"))?;
        let path = directory.join(format!(
            "worker-{}{}",
            self.images.len(),
            std::env::consts::EXE_SUFFIX
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(fail)?;
        let result = (|| {
            file.write_all(&bytes)?;
            file.set_permissions(permissions)?;
            file.sync_all()
        })();
        drop(file);
        if let Err(error) = result {
            let _ = fs::remove_file(&path); // Only the file we just created.
            return Err(fail(error));
        }
        self.images.push((digest, path.clone()));
        Ok((path, digest))
    }

    pub fn cleanup(self) -> Result<(), BuildError> {
        if let Some(directory) = self.directory {
            fs::remove_dir_all(directory).map_err(fail)?;
        }
        Ok(())
    }
}

fn read_image(path: &Path, max_bytes: u64) -> Result<(Vec<u8>, fs::Permissions), BuildError> {
    // Reject directories, pipes and symlinks before reading. Recheck through
    // the opened file as well. Safe std pathname access is not an atomic
    // same-user anti-retargeting sandbox; the host owns its build directories.
    if !fs::symlink_metadata(path)
        .map_err(fail)?
        .file_type()
        .is_file()
    {
        return Err(fail(
            "Cargo output must be a regular, non-symlink executable file",
        ));
    }
    let file = File::open(path).map_err(fail)?;
    let metadata = file.metadata().map_err(fail)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(fail(
            "Cargo executable is empty, non-regular or exceeds the image budget",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(usize::try_from(metadata.len()).map_err(fail)?)
        .map_err(fail)?;
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(fail)?;
    if bytes.is_empty() || bytes.len() as u64 > max_bytes {
        return Err(fail(
            "Cargo executable changed size beyond its image budget",
        ));
    }
    Ok((bytes, metadata.permissions()))
}
