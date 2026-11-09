//! The embedded Studio UI assets (§13.5): compiled into the binary, versioned
//! with it, never served from the filesystem.
//!
//! The Studio's browser UI is exactly what this module compiles in: a script,
//! stylesheet, and index-shell template. The HTTP host ([`crate::host`]) serves
//! these bytes at exact routes and has no mechanism to read UI files at
//! runtime: there is no path from a request to a filesystem open. An asset
//! change is a source change, which means it is reviewed, versioned, and
//! content-addressed like any other code.
//!
//! # Version coupling
//!
//! [`STUDIO_UI_VERSION`] is the fmn-studio crate version, baked in at compile
//! time: the UI asset set shares the binary's crate version. The host stamps
//! every UI response with that header so a connected browser can detect a
//! stale page against a restarted Studio.

use std::collections::TryReserveError;

/// The UI asset set's version — exactly the fmn-studio crate version.
///
/// Because the assets are compiled in, this is also the version of every byte
/// the Studio's UI routes can serve. `env!("CARGO_PKG_VERSION")` is a
/// compile-time constant, so the coupling cannot drift at runtime.
pub const STUDIO_UI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The response header that stamps every served UI asset with
/// [`STUDIO_UI_VERSION`].
pub const STUDIO_UI_VERSION_HEADER: &str = "X-FMN-Studio-Version";

/// One embedded UI asset: an exact route, its content type, and its bytes.
#[derive(Clone, Copy, Debug)]
pub struct UiAsset {
    /// The exact request path this asset serves at.
    pub route: &'static str,
    /// The response `Content-Type`.
    pub content_type: &'static str,
    /// The complete response body, compiled into the binary.
    pub bytes: &'static [u8],
}

/// The Studio browser script, embedded verbatim.
const STUDIO_JS: &str = include_str!("studio.js");
const STUDIO_CSS: &str = include_str!("studio.css");

/// The static prefix of the index shell. The capability token and suffix are
/// appended by [`studio_index_html`]; the UI version meta tag is baked in at
/// compile time via [`STUDIO_UI_VERSION`].
const STUDIO_INDEX_HTML_PREFIX: &str = concat!(
    "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">",
    "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
    "<meta name=\"referrer\" content=\"no-referrer\">",
    "<meta name=\"fmn-studio-ui-version\" content=\"",
    env!("CARGO_PKG_VERSION"),
    "\">",
    "<title>FrankenManim Studio</title></head><body>",
    r#"<header><div><p class="eyebrow">FRANKENMANIM</p><h1>Studio</h1></div>
<div class="session"><p id="worker" role="status">Connecting to worker…</p>
<button id="inspect">Refresh inspector</button><button id="restart">Restart worker</button>
<button id="save-session" disabled>Save session</button><span id="session-save" role="status"></span></div></header>
<p id="error" role="alert" hidden></p>
<main><section class="stage" aria-label="Scene preview and timeline">
<div class="preview-wrap"><canvas id="preview" width="960" height="540" tabindex="0"
 aria-label="Scene preview. Use the timeline below to navigate."></canvas>
<canvas id="overlay" width="960" height="540" aria-hidden="true"></canvas></div>
<p id="display" role="status">Waiting for the first frame…</p>
<form id="timeline-form"><label for="timeline">Timeline <output id="position">—</output></label>
<input id="timeline" type="range" min="0" max="0" value="0" disabled>
<div class="transport"><button type="button" id="first" aria-label="First frame">|←</button>
<button type="button" id="previous" aria-label="Previous frame">←</button>
<button type="button" id="play">Play</button>
<button type="button" id="next" aria-label="Next frame">→</button>
<button type="button" id="last" aria-label="Last frame">→|</button>
<label for="frame">Frame</label><input id="frame" type="number" min="0" max="0" value="0" required>
<button id="seek" type="submit">Go</button></div></form>
<p id="replay">Release the timeline to commit a replay position.</p>
<div class="transport" aria-label="Live scene clock">
<button type="button" id="live-step" disabled>Step live frame</button>
<button type="button" id="live-run" disabled>Run live</button>
<span id="live-clock" role="status">Live clock unavailable on this capture.</span></div>
<fieldset id="layers"><legend>Debug overlays</legend>
<label><input type="checkbox" value="1"> Tiles</label>
<label><input type="checkbox" value="2"> Control points</label>
<label><input type="checkbox" value="4"> Bounds</label>
<label><input type="checkbox" value="8"> Winding</label>
<label><input type="checkbox" value="16"> Depth</label></fieldset>
<p id="overlay-state">Overlays off.</p>
<label><input id="input-events" type="checkbox" disabled> Send preview input to the scene</label>
<p id="input-support">Checking scene input support…</p></section>
<aside aria-label="Scene inspector"><section><h2>Family tree</h2>
<p id="tree-state"></p><div id="tree" role="tree" aria-label="Scene family"></div></section>
<section aria-label="Selected object"><h2 id="selection">Select an object</h2>
<div id="details"></div></section></aside></main>
<footer>Arrow keys navigate the family tree. Home / End select its first / last visible item.
 The timeline supports arrow keys, Home and End.</footer>"#,
    "<script src=\"/studio.js?cap="
);

const STUDIO_INDEX_HTML_SUFFIX: &str = "\"></script></body></html>";
const STUDIO_INDEX_HTML_STATIC_BYTES: usize =
    STUDIO_INDEX_HTML_PREFIX.len() + STUDIO_INDEX_HTML_SUFFIX.len();

/// The static embedded UI asset set, in route order.
///
/// The index shell is not in this table: it is per-session (it carries the
/// capability token) and is produced by [`studio_index_html`].
static UI_ASSETS: &[UiAsset] = &[
    UiAsset {
        route: "/studio.css",
        content_type: "text/css; charset=utf-8",
        bytes: STUDIO_CSS.as_bytes(),
    },
    UiAsset {
        route: "/studio.js",
        content_type: "text/javascript; charset=utf-8",
        bytes: STUDIO_JS.as_bytes(),
    },
];

/// The complete embedded UI asset set.
pub fn ui_assets() -> &'static [UiAsset] {
    UI_ASSETS
}

/// Look up the embedded asset for an exact request path.
#[must_use]
pub fn ui_asset(route: &str) -> Option<&'static UiAsset> {
    UI_ASSETS.iter().find(|asset| asset.route == route)
}

/// Render the index shell for one session's capability token (hex form).
///
/// The token appears only inside the script URL's `cap` query, exactly as
/// [`CapabilityToken::try_expose_hex`](crate::CapabilityToken::try_expose_hex)
/// formats it; the template performs no other substitution.
///
/// # Errors
///
/// Returns the allocator's refusal when the complete rendered shell cannot be
/// reserved.
pub fn studio_index_html(capability_hex: &str) -> Result<String, TryReserveError> {
    studio_index_html_with_capacity(
        capability_hex,
        studio_index_html_capacity(capability_hex.len()),
    )
}

pub(crate) const fn studio_index_html_capacity(capability_bytes: usize) -> usize {
    STUDIO_INDEX_HTML_STATIC_BYTES.saturating_add(capability_bytes)
}

pub(crate) fn studio_index_html_with_capacity(
    capability_hex: &str,
    capacity: usize,
) -> Result<String, TryReserveError> {
    let mut rendered = String::new();
    rendered.try_reserve_exact(capacity)?;
    let required = studio_index_html_capacity(capability_hex.len());
    rendered.try_reserve_exact(required)?;
    rendered.push_str(STUDIO_INDEX_HTML_PREFIX);
    rendered.push_str(capability_hex);
    rendered.push_str(STUDIO_INDEX_HTML_SUFFIX);
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_the_crate_version_and_the_template_is_baked() {
        assert_eq!(STUDIO_UI_VERSION, env!("CARGO_PKG_VERSION"));
        assert!(STUDIO_INDEX_HTML_PREFIX.contains(&format!(
            "name=\"fmn-studio-ui-version\" content=\"{STUDIO_UI_VERSION}\""
        )));
        let rendered = studio_index_html("abc123").unwrap();
        assert!(rendered.contains("/studio.js?cap=abc123"));
        assert!(rendered.contains("id=\"restart\""));
        assert!(STUDIO_JS.contains("api(\"/api/restart\""));
    }

    #[test]
    fn index_html_refuses_capacity_overflow() {
        assert!(studio_index_html_with_capacity("abc123", usize::MAX).is_err());
    }

    #[test]
    fn asset_routes_are_exact_and_unique() {
        let assets = ui_assets();
        assert!(!assets.is_empty());
        for asset in assets {
            assert!(asset.route.starts_with('/'));
            assert!(!asset.bytes.is_empty());
            assert_eq!(
                assets
                    .iter()
                    .filter(|other| other.route == asset.route)
                    .count(),
                1,
                "duplicate route {}",
                asset.route
            );
            assert_eq!(ui_asset(asset.route).map(|a| a.bytes), Some(asset.bytes));
        }
        assert!(ui_asset("/").is_none(), "the index shell is per-session");
        assert!(ui_asset("/studio.js/").is_none(), "no prefix matching");
        assert!(ui_asset("/etc/passwd").is_none());
    }
}
