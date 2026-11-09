//! Host-entropy capability for non-semantic secrets.
//!
//! Scene randomness never comes from this module: deterministic programs use
//! `fmn-core`'s seeded RNG.  This capability exists for host-owned secrets
//! such as the Studio's loopback bearer token, which must be unpredictable
//! but must never enter a certified render's input closure.

use std::fmt;
#[cfg(unix)]
use std::io::Read as _;

/// Failure to obtain the requested host entropy.
#[derive(Debug)]
pub enum EntropyError {
    /// This safe-`std` build has no audited native entropy implementation.
    Unavailable(&'static str),
    /// The operating-system entropy source could not be read completely.
    Io(std::io::Error),
}

impl fmt::Display for EntropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => f.write_str(message),
            Self::Io(error) => write!(f, "host entropy failed: {error}"),
        }
    }
}

impl std::error::Error for EntropyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Unavailable(_) => None,
        }
    }
}

/// Explicit source of nondeterministic host bytes.
pub trait HostEntropy: Send + Sync {
    /// Fill the complete destination or return a typed capability failure.
    fn fill(&self, destination: &mut [u8]) -> Result<(), EntropyError>;
}

/// Audited safe-`std` host entropy.
///
/// Unix systems expose the kernel CSPRNG through `/dev/urandom`. Other
/// targets fail closed until they have an equally narrow safe implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdHostEntropy;

impl HostEntropy for StdHostEntropy {
    fn fill(&self, destination: &mut [u8]) -> Result<(), EntropyError> {
        #[cfg(unix)]
        {
            let mut source = std::fs::File::open("/dev/urandom").map_err(EntropyError::Io)?;
            source.read_exact(destination).map_err(EntropyError::Io)
        }
        #[cfg(not(unix))]
        {
            let _ = destination;
            Err(EntropyError::Unavailable(
                "no audited safe-std host entropy capability is registered on this target",
            ))
        }
    }
}

/// Deterministic entropy fixture for hosts and protocol tests.
#[derive(Clone, Debug)]
pub struct ScriptedEntropy {
    bytes: Vec<u8>,
}

impl ScriptedEntropy {
    /// Construct a repeating scripted byte source.
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl HostEntropy for ScriptedEntropy {
    fn fill(&self, destination: &mut [u8]) -> Result<(), EntropyError> {
        if self.bytes.is_empty() && !destination.is_empty() {
            return Err(EntropyError::Unavailable("scripted entropy is empty"));
        }
        for (byte, scripted) in destination.iter_mut().zip(self.bytes.iter().cycle()) {
            *byte = *scripted;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_entropy_fills_the_entire_request() {
        let mut bytes = [0_u8; 7];
        ScriptedEntropy::new(vec![1, 2, 3])
            .fill(&mut bytes)
            .expect("scripted entropy");
        assert_eq!(bytes, [1, 2, 3, 1, 2, 3, 1]);
    }

    #[test]
    fn empty_script_refuses_nonempty_requests() {
        let mut byte = [0_u8; 1];
        assert!(ScriptedEntropy::new(Vec::new()).fill(&mut byte).is_err());
    }
}
