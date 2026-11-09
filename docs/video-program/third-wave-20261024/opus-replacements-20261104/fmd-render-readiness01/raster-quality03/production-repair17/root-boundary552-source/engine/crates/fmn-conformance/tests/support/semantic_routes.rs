//! The native golden-producing routes, each drawing the built-in semantic
//! witness (fm-5wq.46). Shared by `tests/semantic_oracles.rs` and the e2e
//! scenario `render_matrix.semantic_oracles.native.v1`, so both judge the
//! same frames. The oracles live in `fmn_conformance::semantic`.

use std::path::Path;
use std::sync::Arc;

use fmn::builtins::{SEMANTIC_WITNESS_SCENE_NAME, SemanticWitnessScene};
use fmn::prelude::{
    BundleExportOptions, RenderFormat, RenderOptions, Srgb, export_bundle_bytes, render_with_fs,
};
use fmn::rendering::render_bundle_with_fs;
use fmn_codec::{PngLimits, decode_png};
use fmn_conformance::semantic::{
    Coverage, Frame, FrameShape, Reading, failures, flipped_rows, read_witness,
};
use fmn_platform::fs::{FileSystem, VirtualFs};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 180;
const FPS: u32 = 8;
/// `fmn-wasm`'s player paints its own fixed black background.
const PLAYER_BACKGROUND: [u8; 3] = [0, 0, 0];

/// The oracles a planted vertical mirror must fail on every route.
pub const MUST_REJECT_FLIP: [&str; 5] = [
    "orientation.triangle",
    "orientation.f_shape",
    "placement.up_dot",
    "colour.fill",
    "reading_order.tex",
];

/// One route's first frame of the witness.
pub struct RouteFrame {
    pub route: &'static str,
    pub rgba: Vec<u8>,
    pub background: [u8; 3],
}

impl RouteFrame {
    fn frame(&self) -> Frame<'_> {
        Frame {
            width: WIDTH as usize,
            height: HEIGHT as usize,
            rgba: &self.rgba,
        }
    }

    /// The oracles on the frame as rendered, then on its planted vertical
    /// mirror.
    #[must_use]
    pub fn read(&self) -> (Vec<Reading>, Vec<Reading>) {
        let shape = FrameShape::default_for(WIDTH as usize, HEIGHT as usize);
        let rendered = read_witness(
            self.frame(),
            shape,
            self.background,
            Coverage::WithReadingOrder,
        );
        let mirrored = flipped_rows(self.frame());
        let flipped = read_witness(
            Frame {
                rgba: &mirrored,
                ..self.frame()
            },
            shape,
            self.background,
            Coverage::WithReadingOrder,
        );
        (rendered, flipped)
    }

    /// `Ok` when the frame passes every oracle and its mirror fails each of
    /// [`MUST_REJECT_FLIP`].
    ///
    /// # Errors
    /// Names the failed oracles, or the ones the mirror passed.
    pub fn verdict(&self) -> Result<(), String> {
        let (rendered, flipped) = self.read();
        let failed: Vec<_> = failures(&rendered).iter().map(|r| r.oracle).collect();
        if !failed.is_empty() {
            return Err(format!("{}: oracles failed: {failed:?}", self.route));
        }
        let rejected: Vec<_> = failures(&flipped).iter().map(|r| r.oracle).collect();
        let passed: Vec<_> = MUST_REJECT_FLIP
            .iter()
            .filter(|oracle| !rejected.contains(oracle))
            .collect();
        if !passed.is_empty() {
            return Err(format!(
                "{}: a vertically mirrored frame passed {passed:?}",
                self.route
            ));
        }
        Ok(())
    }
}

/// The resolved `camera.background_color` every native route renders over.
fn configured_background() -> Result<[u8; 3], String> {
    let options = RenderOptions::new("/background").map_err(|e| e.to_string())?;
    let color =
        Srgb::from_hex(&options.config.camera.background_color).map_err(|e| e.to_string())?;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |channel: f64| (channel * 255.0).round() as u8;
    Ok([byte(color.r), byte(color.g), byte(color.b)])
}

fn options(path: &str) -> Result<RenderOptions, String> {
    let mut options = RenderOptions::new(path).map_err(|e| e.to_string())?;
    options.format = RenderFormat::PngSequence;
    options.config.camera.resolution = (WIDTH, HEIGHT);
    options.config.camera.fps = FPS;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
    Ok(options)
}

fn decode(route: &'static str, bytes: &[u8]) -> Result<RouteFrame, String> {
    let image = decode_png(bytes, &PngLimits::default()).map_err(|e| format!("{route}: {e}"))?;
    if (image.width, image.height) != (WIDTH, HEIGHT) {
        return Err(format!(
            "{route}: frame is {}x{}",
            image.width, image.height
        ));
    }
    Ok(RouteFrame {
        route,
        rgba: image.rgba,
        background: configured_background()?,
    })
}

fn first_png(route: &'static str, fs: &dyn FileSystem, dir: &str) -> Result<RouteFrame, String> {
    let bytes = fs
        .read(&Path::new(dir).join("frame_000000.png"))
        .map_err(|e| format!("{route}: no first frame: {e}"))?;
    decode(route, &bytes)
}

/// `fmn::render` straight from the library.
pub fn library_2d() -> Result<RouteFrame, String> {
    let fs = Arc::new(VirtualFs::new());
    render_with_fs(
        &mut SemanticWitnessScene::new(),
        options("/witness")?,
        fs.clone(),
    )
    .map_err(|e| format!("library_2d: {e}"))?;
    first_png("library_2d", fs.as_ref(), "/witness")
}

fn cli(route: &'static str, scene: &str) -> Result<RouteFrame, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    // A fresh directory: the CLI publishes without clobbering.
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "semantic-oracles-{route}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{route}: {e}"))?;
    let dir_text = dir
        .to_str()
        .ok_or_else(|| format!("{route}: output path is not UTF-8"))?;
    let resolution = format!("{WIDTH}x{HEIGHT}");
    let fps = FPS.to_string();
    let output = fmn_cli::run([
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        resolution.as_str(),
        "--fps",
        fps.as_str(),
        "--threads",
        "1",
        "--video_dir",
        dir_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        scene,
    ]);
    let png = dir.join(scene).join("frame_000000.png");
    let bytes = std::fs::read(&png).map_err(|e| {
        format!(
            "{route}: no first frame at {} ({e}); stdout {:?}",
            png.display(),
            output.stdout
        )
    })?;
    decode(route, &bytes)
}

/// The shipped CLI's affine route, in process.
pub fn cli_2d() -> Result<RouteFrame, String> {
    cli("cli_2d", SEMANTIC_WITNESS_SCENE_NAME)
}

/// The shipped CLI's fixed-camera (perspective kernel) route, in process.
pub fn cli_camera() -> Result<RouteFrame, String> {
    cli("cli_camera", "semantic_witness_camera.v1")
}

/// The witness as an FMTL/1 bundle, the input both players share.
fn witness_bundle() -> Result<Vec<u8>, String> {
    let mut export = BundleExportOptions::new().map_err(|e| e.to_string())?;
    export.config.camera.fps = FPS;
    Ok(
        export_bundle_bytes(&mut SemanticWitnessScene::new(), export)
            .map_err(|e| format!("export: {e}"))?
            .bundle
            .bytes,
    )
}

/// The native FMTL/1 player.
pub fn fmtl_native() -> Result<RouteFrame, String> {
    let fs = Arc::new(VirtualFs::new());
    render_bundle_with_fs(&witness_bundle()?, options("/fmtl")?, fs.clone())
        .map_err(|e| format!("fmtl_native: {e}"))?;
    first_png("fmtl_native", fs.as_ref(), "/fmtl")
}

/// The browser player's own frame path (`fmn-wasm`'s `FmnPlayer`, which
/// owns its Y mapping and background), run natively: its success paths need
/// no JS host. wasm-smoke covers execution inside a wasm VM.
pub fn fmtl_wasm_player() -> Result<RouteFrame, String> {
    let mut player = fmn_wasm::player::FmnPlayer::from_bundle(&witness_bundle()?)
        .map_err(|_| "fmtl_wasm_player: the bundle did not load".to_owned())?;
    player
        .set_viewport(WIDTH, HEIGHT)
        .map_err(|_| "fmtl_wasm_player: viewport refused".to_owned())?;
    let rgba = player
        .render_frame(0)
        .map_err(|_| "fmtl_wasm_player: frame 0 did not render".to_owned())?;
    Ok(RouteFrame {
        route: "fmtl_wasm_player",
        rgba,
        background: PLAYER_BACKGROUND,
    })
}

/// Every native route, in a fixed order.
pub const ROUTES: [fn() -> Result<RouteFrame, String>; 5] = [
    library_2d,
    cli_2d,
    cli_camera,
    fmtl_native,
    fmtl_wasm_player,
];
