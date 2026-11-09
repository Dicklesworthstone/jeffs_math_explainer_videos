//! Real Python callbacks at the native Studio serial input boundary. The worker
//! retains its Scene on the original interpreter thread. Neither the supervisor
//! nor another thread obtains its Stage, proxies or callback authority.

use super::*;
use fmn_scene::{CommandKind, EventPayload, Key, Modifiers, MouseButton};
use fmn_studio::advance::is_advance_command;
use fmn_studio::{ServiceError, SupervisorRequest, WorkerErrorCode, WorkerResponse};

fn failure(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}

fn with_capture<T>(
    scene: &Bound<'_, PyScene>,
    f: impl FnOnce(&mut Capture) -> Result<T, ServiceError>,
) -> Result<T, ServiceError> {
    let render = Arc::clone(&scene.try_borrow().map_err(failure)?.render);
    let mut slot = render.lock().map_err(failure)?;
    let Some(PortalRenderSession::Preview(capture)) = slot.as_mut() else {
        return Err(failure(
            "live Studio capture ownership was lost; reload the scene",
        ));
    };
    f(capture)
}

/// The worker freezes this callable at startup, outside authored scene fields.
/// It must run unborrowed, before callbacks and before publishing their pixels.
fn validate_inputs(py: Python<'_>, validator: Option<&Py<PyAny>>) -> PyResult<()> {
    if let Some(validator) = validator
        && !validator.call0(py)?.bind(py).is_none()
    {
        return Err(PyTypeError::new_err(
            "Studio input validator must return None",
        ));
    }
    Ok(())
}

/// Snapshot after synchronization, never while a Python callback has an engine
/// borrow. The original engine/generation must survive all camera descriptors.
fn refresh(scene: &Bound<'_, PyScene>, validator: Option<&Py<PyAny>>) -> PyResult<()> {
    validate_inputs(scene.py(), validator)?;
    let engine = Rc::clone(&scene.try_borrow()?.engine);
    portal_playback::synchronize(scene)?;
    // Match Scene.update_mobjects at the ordinary release boundary. Native
    // show() supplies the native zero-dt pass, not host-language updaters.
    // Run the Python half while unborrowed so followers, controls and camera
    // updaters reflect an input edit even when its callback did not play/wait.
    run_python_updaters(scene, 0.0)?;
    synchronize_portal_camera(scene)?;
    if !Rc::ptr_eq(&engine, &scene.try_borrow()?.engine) {
        return Err(PyRuntimeError::new_err(
            "live Studio engine changed during synchronization",
        ));
    }
    // Includes lazy imports and input edits made by zero-dt updaters and
    // camera descriptors. No frame has replaced the healthy recording yet.
    validate_inputs(scene.py(), validator)?;
    with_capture(scene, |capture| {
        capture.live_refresh = true;
        Ok(())
    })
    .map_err(native_error)?;
    let render = Arc::clone(&scene.try_borrow()?.render);
    let mut sink = PortalSceneSink {
        render,
        ..PortalSceneSink::default()
    };
    let result = engine.borrow_mut().show(&mut sink).map_err(native_error);
    with_capture(scene, |capture| {
        capture.live_refresh = false;
        Ok(())
    })
    .map_err(native_error)?;
    result
}

/// Use the ordinary native clock and updater ordering, without manufacturing
/// authored waits or consuming play indices. No borrow spans host callbacks.
/// Intermediate steps are deliberately not published: a command either yields
/// its completed final view or freezes input with the last good view retained.
fn advance(scene: &Bound<'_, PyScene>, frames: u32, validator: Option<&Py<PyAny>>) -> PyResult<()> {
    validate_inputs(scene.py(), validator)?;
    let engine = Rc::clone(&scene.try_borrow()?.engine);
    let render = Arc::clone(&scene.try_borrow()?.render);
    let mut sink = PortalSceneSink {
        render,
        ..PortalSceneSink::default()
    };
    for index in 0..frames {
        portal_playback::synchronize(scene)?;
        if !Rc::ptr_eq(&engine, &scene.try_borrow()?.engine) {
            return Err(PyRuntimeError::new_err(
                "live Studio engine changed before clock stepping",
            ));
        }
        let frame = engine
            .borrow_mut()
            .prepare_idle_frame(&mut sink)
            .map_err(native_error)?;
        run_python_updaters(scene, frame.dt())?;
        synchronize_portal_camera(scene)?;
        if !Rc::ptr_eq(&engine, &scene.try_borrow()?.engine) {
            return Err(PyRuntimeError::new_err(
                "live Studio engine changed during clock stepping",
            ));
        }
        // Scans are command-boundary work, not another per-frame watcher.
        // Earlier steps have no publication; validate after the last Python
        // callback before allowing the native final capture to replace it.
        if index + 1 == frames {
            validate_inputs(scene.py(), validator)?;
        }
        // Complete the normal updater pass, capturing only the final step.
        // Do not call refresh/show afterwards: its zero-dt updater pass would
        // execute unconditional user callbacks twice for every live frame.
        with_capture(scene, |c| {
            c.live_refresh = index + 1 == frames;
            Ok(())
        })
        .map_err(native_error)?;
        let result = engine
            .borrow_mut()
            .complete_idle_frame(frame, &mut sink)
            .map_err(native_error);
        with_capture(scene, |c| {
            c.live_refresh = false;
            Ok(())
        })
        .map_err(native_error)?;
        result?;
    }
    Ok(())
}

struct LiveWorker {
    scene: Py<PyScene>,
    input_validator: Option<Py<PyAny>>,
    name: String,
    build: ProtocolDigest,
    state_hash: Option<ProtocolDigest>,
    failed: Option<String>,
}

impl LiveWorker {
    fn new(scene: &Bound<'_, PyScene>) -> PyResult<Self> {
        let input_validator = scene
            .getattr("__dict__")?
            .cast::<PyDict>()?
            .get_item("_fmn_studio_validate_inputs")?
            .map(Bound::unbind);
        if input_validator
            .as_ref()
            .is_some_and(|v| !v.bind(scene.py()).is_callable())
        {
            return Err(PyTypeError::new_err(
                "Studio input validator must be callable",
            ));
        }
        let empty = with_capture(scene, |capture| {
            if capture.live {
                return Err(failure("live Studio is already serving this scene"));
            }
            Ok(capture.is_empty())
        })
        .map_err(native_error)?;
        if empty {
            refresh(scene, input_validator.as_ref())?;
        }
        let (name, build) = with_capture(scene, |capture| {
            capture.recorded.enable_live_input()?;
            capture.recorded.enable_live_advance()?;
            capture.live = true;
            Ok((capture.scene.clone(), capture.recorded.build_id()))
        })
        .map_err(native_error)?;
        refresh(scene, input_validator.as_ref())?;
        let state_hash =
            with_capture(scene, |c| Ok(c.recorded.last_state_hash())).map_err(native_error)?;
        Ok(Self {
            scene: scene.clone().unbind(),
            input_validator,
            name,
            build,
            state_hash,
            failed: None,
        })
    }

    fn handle_attached(
        &mut self,
        py: Python<'_>,
        request: SupervisorRequest,
    ) -> Result<WorkerResponse, ServiceError> {
        let scene = self.scene.bind(py);
        let result = match request {
            SupervisorRequest::Play {
                scene: name,
                command,
            } if is_advance_command(&command) => {
                if let Some(error) = &self.failed {
                    return Err(failure(format!(
                        "live input is frozen after callback failure; reload: {error}"
                    )));
                }
                let request =
                    with_capture(scene, |c| c.recorded.prepare_live_advance(&name, &command))?;
                if let Err(error) = advance(scene, request.frames, self.input_validator.as_ref()) {
                    let message = error.to_string();
                    self.failed = Some(message.clone());
                    let _ = with_capture(scene, |c| {
                        c.recorded.disable_live_input();
                        Ok(())
                    });
                    return Err(failure(format!(
                        "live updater failed; input frozen until reload: {message}"
                    )));
                }
                let committed = with_capture(scene, |c| c.recorded.commit_live_advance(command));
                if let Err(error) = &committed {
                    self.failed = Some(error.to_string());
                    let _ = with_capture(scene, |c| {
                        c.recorded.disable_live_input();
                        Ok(())
                    });
                }
                committed
            }
            SupervisorRequest::Play {
                scene: name,
                command,
            } if command.kind == CommandKind::Input => {
                if let Some(error) = &self.failed {
                    return Err(failure(format!(
                        "live input is frozen after callback failure; reload: {error}"
                    )));
                }
                let input =
                    with_capture(scene, |c| c.recorded.prepare_live_input(&name, &command))?;
                // No Rust scene borrow or generation mutex crosses this call.
                let dispatched = validate_inputs(py, self.input_validator.as_ref())
                    .and_then(|()| dispatch(scene, input.event))
                    .and_then(|()| refresh(scene, self.input_validator.as_ref()));
                if let Err(error) = dispatched {
                    let message = error.to_string();
                    self.failed = Some(message.clone());
                    let _ = with_capture(scene, |c| {
                        c.recorded.disable_live_input();
                        Ok(())
                    });
                    return Err(failure(format!(
                        "live callback failed; input frozen until reload: {message}"
                    )));
                }
                let committed = with_capture(scene, |c| c.recorded.commit_live_input(command));
                if let Err(error) = &committed {
                    self.failed = Some(error.to_string());
                    let _ = with_capture(scene, |c| {
                        c.recorded.disable_live_input();
                        Ok(())
                    });
                }
                committed
            }
            request => with_capture(scene, |c| c.recorded.handle(request)),
        };
        self.state_hash = with_capture(scene, |c| Ok(c.recorded.last_state_hash()))?;
        result
    }
}

impl WorkerService for LiveWorker {
    fn build_id(&self) -> ProtocolDigest {
        self.build
    }
    fn active_scene(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn last_state_hash(&self) -> Option<ProtocolDigest> {
        self.state_hash
    }
    fn begin_session(
        &mut self,
        supervisor: ProtocolDigest,
        max_bytes: usize,
    ) -> Result<(), ServiceError> {
        Python::attach(|py| {
            with_capture(self.scene.bind(py), |c| {
                c.recorded.begin_session(supervisor, max_bytes)
            })
        })
    }
    fn handle(&mut self, request: SupervisorRequest) -> Result<WorkerResponse, ServiceError> {
        Python::attach(|py| self.handle_attached(py, request))
    }
}

pub(crate) fn serve(scene: &Bound<'_, PyScene>) -> PyResult<()> {
    let mut worker = LiveWorker::new(scene)?;
    let result = scene
        .py()
        .detach(move || {
            fmn_studio::serve_worker(
                &mut worker,
                &mut std::io::stdin().lock(),
                &mut std::io::stdout().lock(),
                ProtocolLimits::default(),
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
        })
        .map_err(PyRuntimeError::new_err);
    // Release the retained capture and live proxies on the original worker
    // thread, also on malformed protocol / peer disconnect.
    let aborted = scene.call_method0("_abort_render");
    result?;
    aborted.map(|_| ())
}

// The portal already speaks these pinned pyglet-compatible integer tokens;
// this adapter imports no window toolkit and introduces no event implementation.
fn modifiers(value: Modifiers) -> u32 {
    u32::from(value.contains(Modifiers::SHIFT))
        | (u32::from(value.contains(Modifiers::CONTROL)) << 1)
        | (u32::from(value.contains(Modifiers::ALT)) << 2)
        | (u32::from(value.contains(Modifiers::COMMAND)) << 6)
}
fn button(value: MouseButton) -> u32 {
    match value {
        MouseButton::Left => 1,
        MouseButton::Middle => 2,
        MouseButton::Right => 4,
        MouseButton::Other(v) => u32::from(v),
    }
}
fn key(value: Key) -> u32 {
    match value {
        // Browser text keys include Shift/Caps Lock casing. Reference ASCII
        // key symbols are unshifted; modifiers travel separately. Normalize
        // only this portal, not the native Studio wire or Unicode characters.
        Key::Character(c) => c.to_ascii_lowercase() as u32,
        Key::Backspace => 0xff08,
        Key::Tab => 0xff09,
        Key::Enter => 0xff0d,
        Key::Escape => 0xff1b,
        Key::ArrowLeft => 0xff51,
        Key::ArrowUp => 0xff52,
        Key::ArrowRight => 0xff53,
        Key::ArrowDown => 0xff54,
        Key::Other(v) => v,
    }
}

fn dispatch(scene: &Bound<'_, PyScene>, event: EventPayload) -> PyResult<()> {
    event.validate().map_err(native_error)?;
    // Browser coordinates are down-positive in the advertised fixed plane.
    // The native camera performs the inverse rotation, translation and zoom;
    // the existing dispatcher maps fixed-frame controls back exactly once.
    let frame = scene.getattr("frame")?;
    let core: Bound<'_, PyCameraFrameCore> = frame.getattr("_core")?.cast_into()?;
    let camera = core.borrow().frame.clone();
    let np = scene.py().import("numpy")?;
    let deliver = scene
        .py()
        .import("fmn_python.interaction")?
        .getattr("dispatch_live_input")?;
    let point = |p: [f64; 3], relative| {
        let p = camera.from_fixed_frame_point([p[0], -p[1], p[2]], relative);
        np.call_method1("array", (p,))
    };
    match event {
        EventPayload::MouseMotion {
            point: p,
            delta,
            modifiers: m,
        } => {
            deliver.call1((
                scene,
                "on_mouse_motion",
                (point(p, false)?, point(delta, true)?),
                modifiers(m),
            ))?;
        }
        EventPayload::MousePress {
            point: p,
            button: b,
            modifiers: m,
        } => {
            deliver.call1((
                scene,
                "on_mouse_press",
                (point(p, false)?, button(b), modifiers(m)),
                modifiers(m),
            ))?;
        }
        EventPayload::MouseRelease {
            point: p,
            button: b,
            modifiers: m,
        } => {
            deliver.call1((
                scene,
                "on_mouse_release",
                (point(p, false)?, button(b), modifiers(m)),
                modifiers(m),
            ))?;
        }
        EventPayload::MouseDrag {
            point: p,
            delta,
            button: b,
            modifiers: m,
        } => {
            deliver.call1((
                scene,
                "on_mouse_drag",
                (
                    point(p, false)?,
                    point(delta, true)?,
                    button(b),
                    modifiers(m),
                ),
                modifiers(m),
            ))?;
        }
        EventPayload::MouseScroll {
            point: p,
            offset,
            modifiers: m,
        } => {
            // Wheel units are pixels on the browser wire. Existing Scene zoom
            // expects an up-positive y pixel offset and a scene-space vector.
            let height = scene
                .getattr("camera")?
                .call_method0("get_pixel_height")?
                .extract::<u32>()?;
            if height == 0 {
                return Err(PyValueError::new_err("live camera has zero pixel height"));
            }
            let pixel_size = camera.height() / f64::from(height);
            let vector = np.call_method1(
                "array",
                ([offset[0] * pixel_size, -offset[1] * pixel_size, 0.0],),
            )?;
            deliver.call1((
                scene,
                "on_mouse_scroll",
                (point(p, false)?, vector, offset[0], -offset[1]),
                modifiers(m),
            ))?;
        }
        EventPayload::KeyPress {
            key: k,
            modifiers: m,
        } => {
            deliver.call1((scene, "on_key_press", (key(k), modifiers(m)), modifiers(m)))?;
        }
        EventPayload::KeyRelease {
            key: k,
            modifiers: m,
        } => {
            deliver.call1((
                scene,
                "on_key_release",
                (key(k), modifiers(m)),
                modifiers(m),
            ))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "live_input_identity_tests.rs"]
mod input_identity_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_studio::protocol::{StudioInput, studio_input_command};

    #[test]
    fn portal_studio_ascii_key_symbols_keep_modifiers_separate() {
        for (input, expected) in [('A', 'a'), ('T', 't'), ('Z', 'z'), ('g', 'g')] {
            assert_eq!(key(Key::Character(input)), expected as u32);
        }
        assert_eq!(key(Key::Character('Σ')), 0x03a3);
        assert_eq!(key(Key::Character('!')), u32::from(b'!'));
        assert_eq!(key(Key::ArrowRight), 0xff53);
        assert_eq!(modifiers(Modifiers::SHIFT), 1);
        assert_eq!(modifiers(Modifiers::CONTROL), 2);
        assert_eq!(modifiers(Modifiers::ALT), 4);
        assert_eq!(modifiers(Modifiers::COMMAND), 64);
    }

    #[test]
    fn portal_studio_live_executes_real_callbacks_and_freezes_failed_input() {
        crate::with_python_test_module("live Studio input", |py, _module, globals| {
            let source = std::ffi::CString::new(
                r#"
import manimlib as m
class Live(m.Scene):
    def on_key_press(self, symbol, modifiers):
        if symbol == ord('e'):
            self.box.shift(m.RIGHT)
            raise ValueError('callback exploded')
        self.box.shift(m.RIGHT)
        if symbol != ord('i'):
            self.wait(0.125)
s = Live()
s._begin_studio_capture('Live', '11'*32, '22'*32, 96, 54, 8, 1, 0, 8, 1024*1024)
s.camera._core.set_pixel_shape(96, 54)
s.camera.fps = 8
s.box = m.Square(fill_opacity=1)
s.follower = m.Square(side_length=0.25, fill_opacity=1)
s.follower.add_updater(lambda mob: mob.move_to(s.box.get_center() + m.UP))
s.add(s.box, s.follower)
s.wait(0.25)
"#,
            )
            .unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .unwrap();
            let scene = globals
                .get_item("s")
                .unwrap()
                .unwrap()
                .cast_into::<PyScene>()
                .unwrap();
            let mut worker = LiveWorker::new(&scene).unwrap();
            worker
                .handle(SupervisorRequest::Scrub {
                    scene: "Live".into(),
                    frame: 1,
                })
                .unwrap();
            let command = |revision, k| {
                studio_input_command(
                    "Live",
                    &StudioInput {
                        frame: 1,
                        revision,
                        target: None,
                        event: EventPayload::KeyPress {
                            key: Key::Character(k),
                            modifiers: Modifiers::NONE,
                        },
                    },
                )
                .unwrap()
            };
            worker
                .handle(SupervisorRequest::Play {
                    scene: "Live".into(),
                    command: command(0, 'i'),
                })
                .unwrap();
            py.run(c"import numpy as np; assert np.allclose(s.follower.get_center(), s.box.get_center() + m.UP)", Some(globals), Some(globals)).unwrap();
            let state = worker.last_state_hash();
            assert!(
                worker
                    .handle(SupervisorRequest::Play {
                        scene: "Live".into(),
                        command: command(0, 'a')
                    })
                    .is_err()
            );
            assert!(
                worker
                    .handle(SupervisorRequest::Play {
                        scene: "Live".into(),
                        command: command(1, 'e')
                    })
                    .is_err()
            );
            assert_eq!(worker.last_state_hash(), state);
            assert!(
                worker
                    .handle(SupervisorRequest::Play {
                        scene: "Live".into(),
                        command: command(1, 'a')
                    })
                    .is_err()
            );
            let WorkerResponse::StudioData { bytes, .. } = worker
                .handle(SupervisorRequest::Inspect {
                    scene: "Live".into(),
                })
                .unwrap()
            else {
                panic!()
            };
            assert!(
                String::from_utf8(bytes)
                    .unwrap()
                    .contains("\"input_events\":false")
            );
            scene.call_method0("_abort_render").unwrap();
        });
    }
}
