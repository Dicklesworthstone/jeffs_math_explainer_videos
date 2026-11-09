//! fm-aef embedded-UI acceptance (§13.5): the Studio's browser UI is compiled
//! into the binary, version-stamped with it, and served at exact routes over a
//! real loopback socket — with byte-exact content-hash checks against the
//! embedded asset table. No runtime file serving exists to test against; the
//! served bytes *are* the compiled constants.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_hash::sha256;
use fmn_platform::clock::{Clock, FakeClock};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_studio::{
    BuildError, CapabilityToken, FrameHub, LaunchError, ProtocolLimits, RebuildDriver,
    STUDIO_UI_VERSION, STUDIO_UI_VERSION_HEADER, StudioHost, StudioHostConfig, StudioWorkerSession,
    Supervisor, SupervisorConfig, WorkerArtifact, WorkerChannel, WorkerLauncher, ui_assets,
};

/// The UI routes never touch the worker; a launcher that refuses by name
/// proves the test would fail loudly rather than silently spawn something.
struct NoLauncher;

impl WorkerLauncher for NoLauncher {
    fn launch(
        &mut self,
        _artifact: &WorkerArtifact,
        _limits: ProtocolLimits,
    ) -> Result<Box<dyn WorkerChannel>, LaunchError> {
        Err(LaunchError::InvalidArtifact(
            "the UI asset test never launches a worker",
        ))
    }
}

impl RebuildDriver for NoLauncher {
    fn rebuild(&mut self) -> Result<WorkerArtifact, BuildError> {
        Err(BuildError::new("the UI asset test never rebuilds a worker"))
    }
}

struct TestHost {
    host: StudioHost,
    authority: String,
    capability_hex: String,
}

fn test_host() -> TestHost {
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
    let root = if cfg!(windows) {
        r"C:\ui-cache"
    } else {
        "/ui-cache"
    };
    let cache = Store::open(fs, Arc::clone(&clock), root, StoreConfig::default())
        .expect("store opens")
        .namespace(
            "studio-replay",
            1,
            NamespacePolicy {
                ceiling_bytes: None,
            },
        )
        .expect("namespace opens");
    let supervisor = Supervisor::new(
        Box::new(NoLauncher),
        Arc::clone(&clock),
        cache,
        SupervisorConfig::default(),
    );
    let session =
        StudioWorkerSession::new("Ui", supervisor, Box::new(NoLauncher), Arc::new(|_| true))
            .expect("session binds");
    let token = CapabilityToken::new([0x42; 32]).expect("nonzero capability");
    let capability_hex = token.try_expose_hex().expect("token hex storage");
    let frames = FrameHub::new(2, 1024 * 1024).expect("frame hub");
    let host = StudioHost::bind(
        Arc::new(session),
        frames,
        token,
        clock,
        StudioHostConfig::default(),
    )
    .expect("host binds");
    let authority = host
        .local_addr()
        .map(|addr| addr.to_string())
        .expect("bound address");
    TestHost {
        host,
        authority,
        capability_hex,
    }
}

/// Serve exactly one request on a worker thread and return the raw response.
fn exchange(test: &TestHost, request: &str) -> Vec<u8> {
    let address = test.host.local_addr().expect("bound address");
    let server = std::thread::spawn({
        let request = request.to_owned();
        move || {
            let mut stream = TcpStream::connect(address).expect("connect");
            stream.write_all(request.as_bytes()).expect("write request");
            stream.flush().expect("flush request");
            let mut response = Vec::new();
            stream.read_to_end(&mut response).expect("read response");
            response
        }
    });
    test.host.serve_once().expect("serve one connection");
    server.join().expect("client thread")
}

fn header<'a>(response: &'a [u8], name: &str) -> Option<&'a str> {
    let text = std::str::from_utf8(response).expect("headers are UTF-8");
    let head = text.split("\r\n\r\n").next().expect("header block");
    head.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

fn body(response: &[u8]) -> &[u8] {
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("header terminator");
    &response[split + 4..]
}

fn status(response: &[u8]) -> &str {
    std::str::from_utf8(response)
        .expect("status line is UTF-8")
        .split("\r\n")
        .next()
        .expect("status line")
}

#[test]
fn ui_version_is_compiled_in_and_stamps_the_asset_table() {
    assert_eq!(
        STUDIO_UI_VERSION,
        env!("CARGO_PKG_VERSION"),
        "the UI asset set is versioned with the binary by construction"
    );
    assert!(!ui_assets().is_empty(), "the Studio ships UI assets");
    for asset in ui_assets() {
        assert!(!asset.bytes.is_empty(), "{} is not empty", asset.route);
    }
}

#[test]
fn embedded_assets_serve_at_their_routes_with_the_right_content_hash() {
    let test = test_host();
    for asset in ui_assets() {
        let response = exchange(
            &test,
            &format!(
                "GET {}?cap={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                asset.route, test.capability_hex, test.authority
            ),
        );
        assert_eq!(status(&response), "HTTP/1.1 200 OK");
        assert_eq!(header(&response, "Content-Type"), Some(asset.content_type));
        assert_eq!(
            header(&response, STUDIO_UI_VERSION_HEADER),
            Some(STUDIO_UI_VERSION),
            "the served asset carries the binary's version"
        );
        assert_eq!(
            body(&response),
            asset.bytes,
            "the served bytes are the compiled-in bytes — no filesystem read"
        );
        assert_eq!(
            sha256(body(&response)),
            sha256(asset.bytes),
            "content hash of the served route matches the embedded asset"
        );
    }
}

#[test]
fn index_shell_is_per_session_and_version_stamped() {
    let test = test_host();
    let response = exchange(
        &test,
        &format!(
            "GET /?cap={} HTTP/1.1\r\nHost: {}\r\n\r\n",
            test.capability_hex, test.authority
        ),
    );
    assert_eq!(status(&response), "HTTP/1.1 200 OK");
    assert_eq!(
        header(&response, STUDIO_UI_VERSION_HEADER),
        Some(STUDIO_UI_VERSION)
    );
    let html = std::str::from_utf8(body(&response)).expect("index is UTF-8");
    assert_eq!(
        header(&response, "Content-Security-Policy"),
        Some(
            "default-src 'none'; img-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'"
        )
    );
    assert!(
        html.contains(&format!(
            "name=\"fmn-studio-ui-version\" content=\"{STUDIO_UI_VERSION}\""
        )),
        "the shell advertises the compiled-in UI version"
    );
    assert!(
        html.contains(&format!("/studio.js?cap={}", test.capability_hex)),
        "the shell wires the session capability into the script URL"
    );
}

#[test]
fn unknown_and_traversal_paths_have_no_route() {
    let test = test_host();
    // Clean but unknown paths: authenticated, then refused by the exact-route
    // table.
    for path in ["/nope", "/assets/ui.js", "/studio.js/"] {
        let response = exchange(
            &test,
            &format!(
                "GET {path}?cap={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                test.capability_hex, test.authority
            ),
        );
        assert_eq!(
            status(&response),
            "HTTP/1.1 404 Not Found",
            "{path} must not resolve to any file or asset"
        );
    }
    // Ambiguous paths are rejected at the parser, before any routing.
    for path in ["/studio.js/..", "/..%2f..%2fetc%2fpasswd"] {
        let response = exchange(
            &test,
            &format!(
                "GET {path}?cap={} HTTP/1.1\r\nHost: {}\r\n\r\n",
                test.capability_hex, test.authority
            ),
        );
        assert_eq!(
            status(&response),
            "HTTP/1.1 400 Bad Request",
            "{path} is ambiguous and must never reach a route"
        );
    }
}
