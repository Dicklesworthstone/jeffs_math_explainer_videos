//! The typed config extraction (§16.2, fm-vn6).
//!
//! GENERATED from API_SCHEMA.tsv + API_OVERLAY.tsv by
//! fmn_conformance::schema — regenerate, never hand-edit.
//! Reference pin: 6199a00d4c1b1127ebe45cb629c3f22538b10e13
//!
//! Regenerate:  UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance \
//!                  --test api_schema
//!
//! The bound keys come from API_OVERLAY.tsv's `[config_binding]`
//! section; the accessors are `Cx`'s in `config.rs`. A key added to
//! `default_config.yml` without a binding fails the coverage check
//! before it can reach this file.

use crate::config::CameraConfig;
use crate::config::Config;
use crate::config::ConfigError;
use crate::config::Cx;
use crate::config::DeterminismConfig;
use crate::config::DirectoriesConfig;
use crate::config::EmbedConfig;
use crate::config::FileWriterConfig;
use crate::config::MobjectConfig;
use crate::config::RenderConfig;
use crate::config::ResolutionOptions;
use crate::config::SceneConfig;
use crate::config::SizesConfig;
use crate::config::TexConfig;
use crate::config::TextConfig;
use crate::config::VMobjectConfig;
use crate::config::WindowConfig;
use crate::yaml::Value;

/// Type a fully merged configuration document.
///
/// # Errors
/// A [`ConfigError`] naming the key path and the expected-vs-found
/// shapes.
pub(crate) fn config_from_value(root: Value) -> Result<Config, ConfigError> {
    let cx = Cx { root: &root };
    let config = Config {
        directories: DirectoriesConfig {
            mirror_module_path: cx.bool("directories.mirror_module_path")?,
            base: cx.string("directories.base")?,
            subdirs: cx.string_map("directories.subdirs")?,
            cache: cx.string("directories.cache")?,
            removed_mirror_prefix: cx.opt_string("directories.removed_mirror_prefix")?,
        },
        window: WindowConfig {
            position_string: cx.string("window.position_string")?,
            monitor_index: cx.u32("window.monitor_index")?,
            full_screen: cx.bool("window.full_screen")?,
            position: cx.opt_tuple_i64("window.position")?,
            size: cx.opt_tuple_u32("window.size")?,
        },
        camera: CameraConfig {
            resolution: cx.tuple_u32("camera.resolution")?,
            background_color: cx.string("camera.background_color")?,
            fps: cx.u32("camera.fps")?,
            background_opacity: cx.f64("camera.background_opacity")?,
        },
        file_writer: FileWriterConfig {
            ffmpeg_bin: cx.string("file_writer.ffmpeg_bin")?,
            video_codec: cx.string("file_writer.video_codec")?,
            pixel_format: cx.string("file_writer.pixel_format")?,
            video_crf: cx.opt_u32("file_writer.video_crf")?,
            video_preset: cx.opt_string("file_writer.video_preset")?,
            video_tune: cx.opt_string("file_writer.video_tune")?,
            video_bitrate: cx.opt_u32("file_writer.video_bitrate")?,
            saturation: cx.f64("file_writer.saturation")?,
            gamma: cx.f64("file_writer.gamma")?,
        },
        scene: SceneConfig {
            show_animation_progress: cx.bool("scene.show_animation_progress")?,
            leave_progress_bars: cx.bool("scene.leave_progress_bars")?,
            preview_while_skipping: cx.bool("scene.preview_while_skipping")?,
            default_wait_time: cx.f64("scene.default_wait_time")?,
        },
        vmobject: VMobjectConfig {
            default_stroke_width: cx.f64("vmobject.default_stroke_width")?,
            default_stroke_color: cx.string("vmobject.default_stroke_color")?,
            default_fill_color: cx.string("vmobject.default_fill_color")?,
        },
        mobject: MobjectConfig {
            default_mobject_color: cx.string("mobject.default_mobject_color")?,
            default_light_color: cx.string("mobject.default_light_color")?,
        },
        tex: TexConfig {
            template: cx.string("tex.template")?,
            font_size_for_unit_height: cx.f64("tex.font_size_for_unit_height")?,
        },
        text: TextConfig {
            font: cx.string("text.font")?,
            alignment: cx.string("text.alignment")?,
            font_size_for_unit_height: cx.f64("text.font_size_for_unit_height")?,
        },
        embed: EmbedConfig {
            exception_mode: cx.string("embed.exception_mode")?,
            autoreload: cx.bool("embed.autoreload")?,
        },
        resolution_options: ResolutionOptions {
            low: cx.tuple_u32("resolution_options.low")?,
            med: cx.tuple_u32("resolution_options.med")?,
            high: cx.tuple_u32("resolution_options.high")?,
            uhd: cx.tuple_u32("resolution_options.4k")?,
        },
        sizes: SizesConfig {
            frame_height: cx.f64("sizes.frame_height")?,
            small_buff: cx.f64("sizes.small_buff")?,
            med_small_buff: cx.f64("sizes.med_small_buff")?,
            med_large_buff: cx.f64("sizes.med_large_buff")?,
            large_buff: cx.f64("sizes.large_buff")?,
            default_mobject_to_edge_buff: cx.f64("sizes.default_mobject_to_edge_buff")?,
            default_mobject_to_mobject_buff: cx.f64("sizes.default_mobject_to_mobject_buff")?,
        },
        key_bindings: cx.string_map("key_bindings")?,
        colors: cx.string_map("colors")?,
        log_level: cx.log_level("log_level")?,
        universal_import_line: cx.string("universal_import_line")?,
        ignore_manimlib_modules_on_reload: cx.bool("ignore_manimlib_modules_on_reload")?,
        determinism: DeterminismConfig {
            mode: cx.determinism_mode("determinism.mode")?,
            seed: cx.u64("determinism.seed")?,
        },
        render: RenderConfig {
            engine: cx.engine("render.engine")?,
            aa: cx.aa_policy("render.aa")?,
            threads: cx.thread_policy("render.threads")?,
        },
        // Placed below, once the borrows of `root` have ended.
        raw: Value::Null,
    };
    Ok(Config {
        raw: root,
        ..config
    })
}
