//! Frozen source identity crosses the real live input and idle-clock boundary.
use super::*;
use fmn_studio::advance::{StudioAdvance, studio_advance_command};
use fmn_studio::protocol::{StudioInput, studio_input_command};

fn frame(worker: &mut LiveWorker) -> Vec<u8> {
    let WorkerResponse::Frame(FrameStream { payload: FramePayload::Pipe { bytes, .. }, .. }) = worker
        .handle(SupervisorRequest::Scrub { scene: "Live".into(), frame: 1 }).unwrap()
    else { panic!("expected native PNG") };
    bytes
}

#[test]
fn portal_studio_live_input_identity_preserves_the_last_good_frame() {
    for mode in ["before", "callback", "updater", "advance"] {
        crate::with_python_test_module("live input identity", |py, _module, globals| {
            let source = std::ffi::CString::new(r#"
import manimlib as m
valid, mode, events = True, 'idle', 0
def validate():
    if not valid:
        raise RuntimeError('declared project inputs changed')
class Live(m.Scene):
    def on_key_press(self, symbol, modifiers):
        global valid, events
        events += 1
        self.box.shift(m.RIGHT)
        if mode == 'callback':
            # Mutating this field cannot disable the worker's frozen owner.
            self.__dict__.pop('_fmn_studio_validate_inputs', None)
            valid = False
s = Live()
s._fmn_studio_validate_inputs = validate
s._begin_studio_capture('Live', '11'*32, '22'*32, 96, 54, 8, 1, 0, 8, 1024*1024)
s.camera._core.set_pixel_shape(96, 54)
s.camera.fps = 8
s.box = m.Square(fill_opacity=1)
def update(box, dt):
    global valid
    if (mode == 'updater' and dt == 0) or (mode == 'advance' and dt > 0):
        box.shift(m.UP)
        valid = False
s.box.add_updater(update)
s.add(s.box)
s.wait(0.25)
"#).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals)).unwrap();
            let scene = globals.get_item("s").unwrap().unwrap().cast_into::<PyScene>().unwrap();
            let mut worker = LiveWorker::new(&scene).unwrap();
            let before = frame(&mut worker);
            globals.set_item("mode", mode).unwrap();
            if mode == "before" { globals.set_item("valid", false).unwrap(); }
            let command = if mode == "advance" {
                studio_advance_command("Live", StudioAdvance { frame: 1, revision: 0, frames: 2 }).unwrap()
            } else {
                studio_input_command("Live", &StudioInput { frame: 1, revision: 0, target: None,
                    event: EventPayload::KeyPress { key: Key::Character('a'), modifiers: Modifiers::NONE } }).unwrap()
            };
            let error = worker.handle(SupervisorRequest::Play { scene: "Live".into(), command }).unwrap_err();
            assert!(error.to_string().contains("declared project inputs changed"), "{mode}: {error}");
            assert!(worker.failed.is_some(), "{mode}");
            assert_eq!(before, frame(&mut worker), "{mode} replaced the healthy capture");
            if mode == "before" {
                assert_eq!(globals.get_item("events").unwrap().unwrap().extract::<u32>().unwrap(), 0);
            }
            scene.call_method0("_abort_render").unwrap();
        });
    }
}

#[test]
fn portal_studio_live_validator_refuses_noncallable_or_success_shaped_false() {
    crate::with_python_test_module("live validator contract", |py, _module, globals| {
        py.run(c"import manimlib as m; s=m.Scene(); s._fmn_studio_validate_inputs=False", Some(globals), Some(globals)).unwrap();
        let scene = globals.get_item("s").unwrap().unwrap().cast_into::<PyScene>().unwrap();
        assert!(LiveWorker::new(&scene).is_err());
        let validator = py.eval(c"lambda: False", None, None).unwrap().unbind();
        assert!(validate_inputs(py, Some(&validator)).is_err());
        let valid = py.eval(c"lambda: None", None, None).unwrap().unbind();
        validate_inputs(py, Some(&valid)).unwrap();
    });
}
