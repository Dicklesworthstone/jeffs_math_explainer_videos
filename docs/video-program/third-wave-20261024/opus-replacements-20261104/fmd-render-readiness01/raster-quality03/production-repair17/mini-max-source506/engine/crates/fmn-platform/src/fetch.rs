//! The asset-fetcher capability: **the only place network exists** (D2).
//!
//! Core never implements networking and carries no TLS. A host that wants
//! remote assets provides its own [`AssetFetcher`]; the engine's default is
//! [`NoNetwork`], whose error names the alternative — exactly the D2 rule
//! that a missing capability is a *named* error, never a silent fallback.
//! Fetched bytes enter the input closure by content hash (C6 in
//! docs/INPUT_CLOSURE.md); recording that is the caller's job, which is why
//! the trait deals in whole byte payloads, not streams.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

/// One fetch request. `max_bytes` is a hard resource cap the implementation
/// must enforce (a fetched asset is untrusted input, §16.5).
#[derive(Clone, Debug)]
pub struct FetchRequest {
    /// The asset URL (scheme interpretation is the host's business).
    pub url: String,
    /// Hard cap on the payload size; exceeding it is [`FetchError::TooLarge`].
    pub max_bytes: u64,
}

/// A fetch failure. Every variant names the URL it failed for.
#[derive(Debug)]
pub enum FetchError {
    /// No fetcher capability is present ([`NoNetwork`]): the named
    /// capability error, with the remedy in the message.
    CapabilityAbsent {
        /// The URL that was requested.
        url: String,
    },
    /// The asset does not exist.
    NotFound {
        /// The URL that was requested.
        url: String,
    },
    /// The payload exceeded [`FetchRequest::max_bytes`].
    TooLarge {
        /// The URL that was requested.
        url: String,
        /// The cap that was exceeded.
        limit: u64,
    },
    /// Any other host-side failure.
    Failed {
        /// The URL that was requested.
        url: String,
        /// Host-provided detail.
        detail: String,
    },
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapabilityAbsent { url } => write!(
                f,
                "no AssetFetcher capability: cannot fetch {url:?}; provide the asset \
                 locally or hand the engine a host AssetFetcher implementation"
            ),
            Self::NotFound { url } => write!(f, "asset not found: {url:?}"),
            Self::TooLarge { url, limit } => {
                write!(f, "asset {url:?} exceeds the {limit}-byte fetch cap")
            }
            Self::Failed { url, detail } => write!(f, "fetch of {url:?} failed: {detail}"),
        }
    }
}

impl std::error::Error for FetchError {}

/// The asset-fetcher capability, provided by the host.
pub trait AssetFetcher: Send + Sync {
    /// Fetch the asset, subject to the request's byte cap.
    ///
    /// # Errors
    /// A [`FetchError`] naming the URL.
    fn fetch(&self, request: &FetchRequest) -> Result<Vec<u8>, FetchError>;
}

/// The default capability: no network, ever. Every fetch is a named error.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoNetwork;

impl AssetFetcher for NoNetwork {
    fn fetch(&self, request: &FetchRequest) -> Result<Vec<u8>, FetchError> {
        Err(FetchError::CapabilityAbsent {
            url: request.url.clone(),
        })
    }
}

/// The test double: canned `url → bytes` responses, with a request log so
/// tests can assert exactly what the engine asked for.
#[derive(Debug, Default)]
pub struct ScriptedFetcher {
    responses: BTreeMap<String, Vec<u8>>,
    requests: Mutex<Vec<String>>,
}

impl ScriptedFetcher {
    /// An empty scripted fetcher (every fetch is [`FetchError::NotFound`]).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Script a response.
    pub fn script(&mut self, url: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.responses.insert(url.into(), bytes.into());
    }

    /// The URLs fetched so far, in request order.
    #[must_use]
    pub fn requests(&self) -> Vec<String> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl AssetFetcher for ScriptedFetcher {
    fn fetch(&self, request: &FetchRequest) -> Result<Vec<u8>, FetchError> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(request.url.clone());
        match self.responses.get(&request.url) {
            None => Err(FetchError::NotFound {
                url: request.url.clone(),
            }),
            Some(bytes) if bytes.len() as u64 > request.max_bytes => Err(FetchError::TooLarge {
                url: request.url.clone(),
                limit: request.max_bytes,
            }),
            Some(bytes) => Ok(bytes.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_network_names_the_capability_and_remedy() {
        let err = NoNetwork
            .fetch(&FetchRequest {
                url: "https://example.com/a.svg".into(),
                max_bytes: 1024,
            })
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("AssetFetcher"), "{msg}");
        assert!(msg.contains("a.svg"), "{msg}");
    }

    #[test]
    fn scripted_fetcher_enforces_caps_and_logs() {
        let mut f = ScriptedFetcher::new();
        f.script("u1", b"payload".to_vec());
        let ok = f.fetch(&FetchRequest {
            url: "u1".into(),
            max_bytes: 100,
        });
        assert_eq!(ok.unwrap(), b"payload");
        assert!(matches!(
            f.fetch(&FetchRequest {
                url: "u1".into(),
                max_bytes: 3
            }),
            Err(FetchError::TooLarge { limit: 3, .. })
        ));
        assert!(matches!(
            f.fetch(&FetchRequest {
                url: "u2".into(),
                max_bytes: 3
            }),
            Err(FetchError::NotFound { .. })
        ));
        assert_eq!(f.requests(), vec!["u1", "u1", "u2"]);
    }
}
