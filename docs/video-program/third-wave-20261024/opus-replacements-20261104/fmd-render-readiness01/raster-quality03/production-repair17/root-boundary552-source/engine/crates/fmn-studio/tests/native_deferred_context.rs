//! Deferred constructors retain the native RNG and input registration owners.

use std::cell::Cell;
use std::rc::Rc;

use fmn_mobject::Mobject;
use fmn_scene::{
    EventListener, EventPayload, EventPropagation, EventTarget, EventType, Key, Modifiers,
    NullSceneSink, RuntimeConfig, Scene,
};
use fmn_studio::native::{NativeSceneProgram, NativeSegment};

fn scene(seed: u64) -> Scene {
    Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        seed,
    )
    .unwrap()
}

#[test]
fn deferred_random_choices_consume_the_exact_ordinary_scene_substream() {
    for seed in [0, 1, 71] {
        let mut ordinary = scene(seed);
        let value = ordinary.stage_mut().add_value_tracker(0.0);
        ordinary.add(&[value]).unwrap();
        ordinary.wait(Some(0.5), &mut NullSceneSink).unwrap();
        let first = ordinary.rng_mut().next_f64();
        let second = ordinary.rng_mut().next_u64();
        let expected_value = first + f64::from(second.to_le_bytes()[0]);
        ordinary
            .stage_mut()
            .set_tracker_value(value, expected_value)
            .unwrap();
        ordinary.wait(Some(0.5), &mut NullSceneSink).unwrap();

        let mut source = scene(seed);
        let value = source.stage_mut().add_value_tracker(0.0);
        source.add(&[value]).unwrap();
        let calls = Rc::new(Cell::new(0));
        let seen = Rc::clone(&calls);
        let mut program = NativeSceneProgram::new(
            source,
            vec![
                NativeSegment::Wait {
                    duration: Some(0.5),
                },
                NativeSegment::edit(move |context| {
                    seen.set(seen.get() + 1);
                    let first = context.rng_mut().next_f64();
                    let second = context.rng_mut().next_u64();
                    context
                        .stage_mut()
                        .set_tracker_value(value, first + f64::from(second.to_le_bytes()[0]))?;
                    Ok(())
                }),
                NativeSegment::Wait {
                    duration: Some(0.5),
                },
            ],
            9,
        )
        .unwrap();
        program.advance_to(4).unwrap();
        program.state_bytes().unwrap();
        assert_eq!(calls.get(), 0);
        program.advance_to(8).unwrap();
        assert_eq!(
            program.preview().stage().tracker_value(value),
            Some(expected_value)
        );
        program.state_bytes().unwrap();
        program.advance_to(8).unwrap();
        assert_eq!(calls.get(), 1);
        assert!(program.next_frame().unwrap().is_none());
        // SceneState includes the actual sequential RNG, not just a drawn point.
        assert_eq!(
            program.state_bytes().unwrap(),
            ordinary.state_bytes().unwrap()
        );
    }
}

fn branching(seed: u64) -> NativeSceneProgram {
    NativeSceneProgram::new(
        scene(seed),
        vec![NativeSegment::defer(|context| {
            let value = context.rng_mut().next_f64();
            let count = if value < 0.5 { 2 } else { 3 };
            for index in 0..count {
                let mob = context.stage_mut().add(Mobject::from_points(&[[
                    f64::from(index),
                    value,
                    0.0,
                ]]));
                context.stage_mut().add_to_scene(mob)?;
            }
            Ok(vec![NativeSegment::Wait {
                duration: Some(0.5),
            }])
        })],
        4,
    )
    .unwrap()
}

#[test]
fn seeded_control_flow_reconstructs_the_same_native_topology_and_rng_state() {
    let mut first = branching(71);
    let mut same = branching(71);
    let mut different = branching(72);
    first.advance_to(4).unwrap();
    same.advance_to(4).unwrap();
    different.advance_to(4).unwrap();
    assert_eq!(first.state_bytes().unwrap(), same.state_bytes().unwrap());
    assert_ne!(
        first.state_bytes().unwrap(),
        different.state_bytes().unwrap()
    );
    assert!([2, 3].contains(&first.preview().stage().roots().len()));
}

#[test]
fn late_native_listeners_are_inactive_before_construction_and_handle_the_new_object_afterward() {
    let hits = Rc::new(Cell::new(0));
    let observed = Rc::clone(&hits);
    let mut program = NativeSceneProgram::new(
        scene(71),
        vec![
            NativeSegment::Wait {
                duration: Some(0.5),
            },
            NativeSegment::defer(move |context| {
                let target = context.stage_mut().add(Mobject::from_points(&[[0.0; 3]]));
                context.stage_mut().add_to_scene(target)?;
                context
                    .event_dispatcher_mut()
                    .add_listener(EventListener::new(
                        EventType::KeyPress,
                        EventTarget::Global,
                        move |event, _, _, stage| {
                            if matches!(
                                event.payload,
                                EventPayload::KeyPress {
                                    key: Key::Other(1001),
                                    ..
                                }
                            ) {
                                observed.set(observed.get() + 1);
                                stage.shift_many(&[target], [1.0, 0.0, 0.0]);
                            }
                            EventPropagation::Continue
                        },
                    ))?;
                Ok(vec![NativeSegment::Wait {
                    duration: Some(0.5),
                }])
            }),
        ],
        8,
    )
    .unwrap();
    let key = || EventPayload::KeyPress {
        key: Key::Other(1001),
        modifiers: Modifiers::NONE,
    };
    program.dispatch(key()).unwrap();
    program.advance_to(4).unwrap();
    program.dispatch(key()).unwrap();
    assert_eq!(hits.get(), 0);
    program.advance_to(5).unwrap();
    let time = program.preview().scene().time();
    program.dispatch(key()).unwrap();
    assert_eq!(hits.get(), 1);
    let stage = program.preview().stage();
    assert_eq!(stage.get_bounding_box(stage.roots()[0]).mid[0], 1.0);
    assert_eq!(program.preview().scene().time(), time);
}
