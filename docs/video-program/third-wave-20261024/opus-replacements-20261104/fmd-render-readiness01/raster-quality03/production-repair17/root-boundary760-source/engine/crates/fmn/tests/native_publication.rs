//! No-clobber is enforced at publication, not merely by an early exists check.

use std::path::Path;
use std::sync::Arc;

use fmn::prelude::*;
use fmn_platform::fs::{FileSystem, VirtualFs};

fn options(format: RenderFormat) -> RenderOptions {
    let mut options = RenderOptions::new("/movie").unwrap();
    options.format = format;
    options.config.camera.resolution = (32, 24);
    options.config.camera.fps = 8;
    options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
    options
}

struct Movie {
    competing_writer: Option<Arc<VirtualFs>>,
}

impl SceneConstruct for Movie {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Circle::new())?;
        stage.wait(0.25)?;
        if let Some(fs) = &self.competing_writer {
            // The renderer has already captured frames, but its output worker
            // cannot publish until scene completion. An exists-only preflight
            // would miss this deterministic competing publication.
            fs.insert("/movie", b"foreign generation".to_vec());
        }
        Ok(())
    }
}

#[test]
fn gif_and_y4m_preserve_existing_files() {
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let fs = Arc::new(VirtualFs::new());
        fs.insert("/movie", b"foreign generation".to_vec());
        let result = render_with_fs(
            &mut Movie {
                competing_writer: None,
            },
            options(format),
            fs.clone(),
        );
        assert!(result.is_err(), "{format:?} must refuse replacement");
        assert_eq!(fs.read(Path::new("/movie")).unwrap(), b"foreign generation");
    }
}

#[test]
fn gif_and_y4m_preserve_competing_publications_after_capture() {
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let fs = Arc::new(VirtualFs::new());
        let result = render_with_fs(
            &mut Movie {
                competing_writer: Some(fs.clone()),
            },
            options(format),
            fs.clone(),
        );
        assert!(
            result.is_err(),
            "{format:?} must atomically refuse the competing writer"
        );
        assert_eq!(fs.read(Path::new("/movie")).unwrap(), b"foreign generation");
    }
}

#[test]
fn no_clobber_allows_new_complete_files() {
    for format in [RenderFormat::Gif, RenderFormat::Y4m] {
        let fs = Arc::new(VirtualFs::new());
        let result = render_with_fs(
            &mut Movie {
                competing_writer: None,
            },
            options(format),
            fs.clone(),
        )
        .unwrap();
        assert_eq!(result.artifact.frame_count, 2);
        assert_eq!(result.emission.stats.emitted, 2);
        assert!(fs.exists(Path::new("/movie")));
    }
}
