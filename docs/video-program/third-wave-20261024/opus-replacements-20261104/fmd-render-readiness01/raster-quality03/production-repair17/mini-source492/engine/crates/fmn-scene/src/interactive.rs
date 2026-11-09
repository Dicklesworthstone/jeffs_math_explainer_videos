//! `InteractiveScene`: deterministic selection and editing behavior (§13.2).
//!
//! This is the engine behavior, not a window implementation. Browser/TUI
//! hosts enqueue typed events on the contained [`Scene`]; the Scene dispatches
//! them at its single pre-capture boundary. Selection adornments are immutable
//! descriptions for Studio/Lumen overlays, so helper rectangles never pollute
//! the user's Stage or replay state.

mod history;

pub use history::DEFAULT_HISTORY_LIMIT;

use std::cell::RefCell;
use std::rc::Rc;

use fmn_core::color::Srgb;
use fmn_core::types::Vec3;
use fmn_mobject::{BoundingBox, Mob, Mobject, Snapshot, Stage};

use crate::events::{
    DispatchState, EventError, EventListener, EventListenerId, EventPayload, EventPropagation,
    EventTarget, EventType, InputEvent, Key, Modifiers, MouseButton,
};
use crate::runtime::Scene;

const SELECTION_BUFFER: f64 = 1.0e-2;
const CLICK_SIZE: f64 = 1.0e-2;
const NUDGE: f64 = 0.05;

/// The key-binding actions named by the pinned Reference configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveAction {
    /// Orbit a 3D camera.
    Pan3D,
    /// Pan a 2D camera.
    Pan,
    /// Reset camera state.
    Reset,
    /// Quit the interactive host.
    Quit,
    /// Sweep/click selection.
    Select,
    /// Clear selection.
    Unselect,
    /// Free-axis grab.
    Grab,
    /// X-constrained grab.
    XGrab,
    /// Y-constrained grab.
    YGrab,
    /// Z-constrained grab.
    ZGrab,
    /// Resize selection.
    Resize,
    /// Pick a color.
    Color,
    /// Show pointer/time information.
    Information,
    /// Toggle the crosshair.
    Cursor,
}

/// One pinned key name from `default_config.yml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    /// Action.
    pub action: InteractiveAction,
    /// Default key.
    pub key: Key,
}

/// The exact default key map at Reference commit
/// `6199a00d4c1b1127ebe45cb629c3f22538b10e13`.
pub const REFERENCE_KEYBOARD_MAP: &[KeyBinding] = &[
    KeyBinding {
        action: InteractiveAction::Pan3D,
        key: Key::Character('d'),
    },
    KeyBinding {
        action: InteractiveAction::Pan,
        key: Key::Character('f'),
    },
    KeyBinding {
        action: InteractiveAction::Reset,
        key: Key::Character('r'),
    },
    KeyBinding {
        action: InteractiveAction::Quit,
        key: Key::Character('q'),
    },
    KeyBinding {
        action: InteractiveAction::Select,
        key: Key::Character('s'),
    },
    KeyBinding {
        action: InteractiveAction::Unselect,
        key: Key::Character('u'),
    },
    KeyBinding {
        action: InteractiveAction::Grab,
        key: Key::Character('g'),
    },
    KeyBinding {
        action: InteractiveAction::XGrab,
        key: Key::Character('h'),
    },
    KeyBinding {
        action: InteractiveAction::YGrab,
        key: Key::Character('v'),
    },
    KeyBinding {
        action: InteractiveAction::ZGrab,
        key: Key::Character('z'),
    },
    KeyBinding {
        action: InteractiveAction::Resize,
        key: Key::Character('t'),
    },
    KeyBinding {
        action: InteractiveAction::Color,
        key: Key::Character('c'),
    },
    KeyBinding {
        action: InteractiveAction::Information,
        key: Key::Character('i'),
    },
    KeyBinding {
        action: InteractiveAction::Cursor,
        key: Key::Character('k'),
    },
];

/// A Studio overlay describing one selected mobject.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionHighlight {
    /// Tracked mobject.
    pub target: Mob,
    /// Current family bounding box.
    pub bounds: BoundingBox,
}

/// The active drag-selection rectangle, if selection is being swept.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionRectangle {
    /// Fixed first corner.
    pub start: Vec3,
    /// Current opposite corner.
    pub end: Vec3,
    /// Axis-aligned rectangle.
    pub bounds: BoundingBox,
}

/// Clipboard data owned by the Scene rather than an OS subprocess/global.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum InteractiveClipboard {
    /// Nothing copied.
    #[default]
    Empty,
    /// Detached family templates in this Scene's arena.
    Mobjects(Vec<Mob>),
    /// Text supplied by a host. Scribe consumers may turn it into Text/Tex.
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrabAxis {
    Free,
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
struct GrabGesture {
    axis: GrabAxis,
    mouse_to_center: Vec3,
    recorded: bool,
}

#[derive(Debug, Clone)]
struct ResizeGesture {
    pivot: Vec3,
    reference: Vec3,
    last_scale: [f64; 3],
    recorded: bool,
}

struct UndoState {
    stage: Rc<Snapshot>,
    selection: Vec<Mob>,
    clipboard: InteractiveClipboard,
}

struct InteractionState {
    selection: Vec<Mob>,
    unselectables: Vec<Mob>,
    select_top_level: bool,
    selection_start: Option<Vec3>,
    selection_current: Vec3,
    selection_sweeping: bool,
    grab: Option<GrabGesture>,
    resize: Option<ResizeGesture>,
    color_picking: bool,
    information_visible: bool,
    cursor_visible: bool,
    clipboard: InteractiveClipboard,
    history: history::EditHistory,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            selection: Vec::new(),
            unselectables: Vec::new(),
            select_top_level: true,
            selection_start: None,
            selection_current: [0.0; 3],
            selection_sweeping: false,
            grab: None,
            resize: None,
            color_picking: false,
            information_visible: false,
            cursor_visible: false,
            clipboard: InteractiveClipboard::Empty,
            history: history::EditHistory::default(),
        }
    }
}

impl InteractionState {
    fn handle(
        &mut self,
        event: &InputEvent,
        dispatch: &DispatchState,
        stage: &mut Stage,
    ) -> EventPropagation {
        self.prune(stage);
        match &event.payload {
            EventPayload::KeyPress { key, modifiers } => {
                self.key_press(*key, *modifiers, dispatch, stage);
            }
            EventPayload::KeyRelease { key, .. } => {
                self.key_release(*key, stage);
            }
            EventPayload::MouseMotion {
                point, modifiers, ..
            }
            | EventPayload::MouseDrag {
                point, modifiers, ..
            } => {
                self.pointer_motion(*point, *modifiers, dispatch, stage);
            }
            EventPayload::MousePress {
                point,
                button,
                modifiers,
            } => {
                if *button == MouseButton::Left
                    && modifiers.intersects(Modifiers::PRIMARY)
                    && let Some(mob) = self.topmost_at(stage, *point)
                {
                    self.toggle_selection(stage, &[mob]);
                }
            }
            EventPayload::MouseRelease { point, button, .. } => {
                if *button == MouseButton::Left && self.color_picking {
                    self.choose_color(stage, *point);
                    self.color_picking = false;
                }
            }
            EventPayload::MouseScroll { .. } => {}
        }
        EventPropagation::Continue
    }

    fn key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
        dispatch: &DispatchState,
        stage: &mut Stage,
    ) {
        let primary = modifiers.intersects(Modifiers::PRIMARY);
        let shift = modifiers.contains(Modifiers::SHIFT);

        if primary {
            match (key, shift) {
                (Key::Character('c'), _) => self.copy_selection(stage),
                (Key::Character('v'), _) => self.paste_selection(stage),
                (Key::Character('x'), _) => {
                    self.copy_selection(stage);
                    self.delete_selection(stage);
                }
                (Key::Character('a'), _) => {
                    self.clear_selection(stage);
                    let all = self.selection_search_set(stage);
                    self.add_to_selection(stage, &all);
                }
                (Key::Character('g'), true) => self.ungroup_selection(stage),
                (Key::Character('g'), false) => self.group_selection(stage),
                (Key::Character('t'), _) => {
                    self.select_top_level = !self.select_top_level;
                    self.refresh_selection_scope(stage);
                }
                (Key::Character('z'), true) => {
                    self.restore_redo(stage);
                }
                (Key::Character('z'), false) => {
                    self.restore_undo(stage);
                }
                _ => {}
            }
            return;
        }

        match key {
            Key::Character('s') if modifiers.is_empty() => {
                self.selection_start = Some(dispatch.mouse_point());
                self.selection_current = dispatch.mouse_point();
                self.selection_sweeping = false;
                self.cursor_visible = true;
            }
            Key::Character('u') => self.clear_selection(stage),
            Key::Character('g') if modifiers.is_empty() => {
                self.prepare_grab(stage, dispatch.mouse_point(), GrabAxis::Free);
            }
            Key::Character('h') if modifiers.is_empty() => {
                self.prepare_grab(stage, dispatch.mouse_point(), GrabAxis::X);
            }
            Key::Character('v') if modifiers.is_empty() => {
                self.prepare_grab(stage, dispatch.mouse_point(), GrabAxis::Y);
            }
            Key::Character('z') if modifiers.is_empty() => {
                self.prepare_grab(stage, dispatch.mouse_point(), GrabAxis::Z);
            }
            Key::Character('t') if modifiers.is_empty() || modifiers == Modifiers::SHIFT => {
                self.prepare_resize(stage, dispatch.mouse_point(), shift);
            }
            Key::Character('c') if modifiers.is_empty() => {
                if !self.selection.is_empty() {
                    self.color_picking = !self.color_picking;
                }
            }
            Key::Character('i') if modifiers.is_empty() => {
                self.information_visible = true;
            }
            Key::Character('k') if modifiers.is_empty() => {
                self.cursor_visible = !self.cursor_visible;
            }
            Key::ArrowLeft => self.nudge(stage, [-1.0, 0.0, 0.0], shift),
            Key::ArrowUp => self.nudge(stage, [0.0, 1.0, 0.0], shift),
            Key::ArrowRight => self.nudge(stage, [1.0, 0.0, 0.0], shift),
            Key::ArrowDown => self.nudge(stage, [0.0, -1.0, 0.0], shift),
            Key::Backspace => self.delete_selection(stage),
            _ => {}
        }
    }

    fn key_release(&mut self, key: Key, stage: &mut Stage) {
        match key {
            Key::Character('s') => self.gather_new_selection(stage),
            Key::Character('g' | 'h' | 'v' | 'z') => {
                self.grab = None;
            }
            Key::Character('t') => {
                self.resize = None;
            }
            Key::Character('i') => {
                self.information_visible = false;
            }
            _ => {}
        }
    }

    fn pointer_motion(
        &mut self,
        point: Vec3,
        modifiers: Modifiers,
        dispatch: &DispatchState,
        stage: &mut Stage,
    ) {
        self.selection_current = point;
        if let Some(grab) = self.grab {
            self.handle_grab(stage, point, grab);
        } else if self.resize.is_some() {
            self.handle_resize(stage, point, modifiers);
        } else if self.selection_start.is_some()
            && modifiers.contains(Modifiers::SHIFT)
            && dispatch.is_key_pressed(Key::Character('s'))
        {
            self.selection_sweeping = true;
            if let Some(mob) = self.topmost_at(stage, point) {
                self.add_to_selection(stage, &[mob]);
            }
        }
    }

    fn prepare_grab(&mut self, stage: &Stage, mouse: Vec3, axis: GrabAxis) {
        let Some(bounds) = selection_bounds(stage, &self.selection) else {
            return;
        };
        // Record at the first actual movement, not a cancelled key press.
        let recorded = self.grab.is_some_and(|gesture| gesture.recorded);
        self.grab = Some(GrabGesture {
            axis,
            mouse_to_center: sub(mouse, bounds.mid),
            recorded,
        });
    }

    fn handle_grab(&mut self, stage: &mut Stage, mouse: Vec3, grab: GrabGesture) {
        let Some(bounds) = selection_bounds(stage, &self.selection) else {
            return;
        };
        let desired = sub(mouse, grab.mouse_to_center);
        let mut delta = sub(desired, bounds.mid);
        match grab.axis {
            GrabAxis::Free => {}
            GrabAxis::X => {
                delta[1] = 0.0;
                delta[2] = 0.0;
            }
            GrabAxis::Y => {
                delta[0] = 0.0;
                delta[2] = 0.0;
            }
            GrabAxis::Z => {
                delta[0] = 0.0;
                delta[1] = 0.0;
            }
        }
        if delta.iter().any(|component| *component != 0.0) {
            if !grab.recorded {
                self.save_undo(stage);
                if let Some(gesture) = self.grab.as_mut() {
                    gesture.recorded = true;
                }
            }
            stage.shift_many(&self.selection, delta);
        }
    }

    fn prepare_resize(&mut self, stage: &Stage, mouse: Vec3, about_corner: bool) {
        let Some(bounds) = selection_bounds(stage, &self.selection) else {
            return;
        };
        let recorded = self.resize.as_ref().is_some_and(|gesture| gesture.recorded);
        let pivot = if about_corner {
            bounds.point([
                sign(bounds.mid[0] - mouse[0]),
                sign(bounds.mid[1] - mouse[1]),
                sign(bounds.mid[2] - mouse[2]),
            ])
        } else {
            bounds.mid
        };
        self.resize = Some(ResizeGesture {
            pivot,
            reference: sub(mouse, pivot),
            last_scale: [1.0; 3],
            recorded,
        });
    }

    fn handle_resize(&mut self, stage: &mut Stage, point: Vec3, modifiers: Modifiers) {
        let Some(mut gesture) = self.resize.clone() else {
            return;
        };
        let vector = sub(point, gesture.pivot);
        if modifiers.intersects(Modifiers::CONTROL) {
            for (axis, &component) in vector.iter().enumerate().take(2) {
                let reference = gesture.reference[axis];
                if reference.abs() <= f64::EPSILON {
                    continue;
                }
                let ratio = component / reference;
                let cumulative = if ratio.abs() < 1.0e-6 {
                    1.0e-6_f64.copysign(ratio)
                } else {
                    ratio
                };
                let incremental = cumulative / gesture.last_scale[axis];
                if incremental != 1.0 {
                    if !gesture.recorded {
                        self.save_undo(stage);
                        gesture.recorded = true;
                    }
                    stage.stretch_many_about_point(
                        &self.selection,
                        incremental,
                        axis,
                        gesture.pivot,
                    );
                    gesture.last_scale[axis] = cumulative;
                }
            }
        } else {
            let reference_norm = norm(gesture.reference);
            if reference_norm <= f64::EPSILON {
                return;
            }
            let absolute = (norm(vector) / reference_norm).max(1.0e-6);
            let incremental = absolute / gesture.last_scale[0];
            if incremental != 1.0 {
                if !gesture.recorded {
                    self.save_undo(stage);
                    gesture.recorded = true;
                }
                stage.scale_many_about_point(&self.selection, incremental, gesture.pivot);
                gesture.last_scale = [absolute; 3];
            }
        }
        self.resize = Some(gesture);
    }

    fn nudge(&mut self, stage: &mut Stage, direction: Vec3, large: bool) {
        if self.selection.is_empty() {
            return;
        }
        self.save_undo(stage);
        let amount = if large { NUDGE * 10.0 } else { NUDGE };
        let delta = scaled(direction, amount);
        stage.shift_many(&self.selection, delta);
    }

    fn gather_new_selection(&mut self, stage: &mut Stage) {
        let swept = std::mem::take(&mut self.selection_sweeping);
        let Some(start) = self.selection_start.take() else {
            return;
        };
        if swept {
            return;
        }
        let bounds = rectangle_bounds(start, self.selection_current);
        let tiny = bounds.width() + bounds.height() < CLICK_SIZE;
        let mut additions = Vec::new();
        for mob in self.selection_search_set(stage).into_iter().rev() {
            if boxes_intersect(bounds, stage.get_bounding_box(mob), SELECTION_BUFFER) {
                additions.push(mob);
                if tiny {
                    break;
                }
            }
        }
        self.toggle_selection(stage, &additions);
    }

    fn topmost_at(&self, stage: &Stage, point: Vec3) -> Option<Mob> {
        self.selection_search_set(stage)
            .into_iter()
            .rev()
            .find(|mob| point_in_box(point, stage.get_bounding_box(*mob), SELECTION_BUFFER))
    }

    fn topmost_color_source(&self, stage: &Stage, point: Vec3) -> Option<Mob> {
        let mut members = Vec::new();
        for &root in stage.roots() {
            for member in stage.family(root) {
                if self.unselectables.contains(&member)
                    || stage
                        .get(member)
                        .is_none_or(|entry| entry.buffer.is_empty())
                    || members.contains(&member)
                {
                    continue;
                }
                members.push(member);
            }
        }
        members
            .into_iter()
            .rev()
            .find(|mob| point_in_box(point, stage.get_bounding_box(*mob), SELECTION_BUFFER))
    }

    fn selection_search_set(&self, stage: &Stage) -> Vec<Mob> {
        let roots: Vec<_> = stage
            .roots()
            .iter()
            .copied()
            .filter(|mob| !self.unselectables.contains(mob))
            .collect();
        if self.select_top_level {
            return roots;
        }
        let mut members = Vec::new();
        for root in roots {
            for member in stage.family(root) {
                if self.unselectables.contains(&member)
                    || stage
                        .get(member)
                        .is_none_or(|entry| entry.buffer.is_empty())
                    || members.contains(&member)
                {
                    continue;
                }
                members.push(member);
            }
        }
        members
    }

    fn refresh_selection_scope(&mut self, stage: &mut Stage) {
        let current = self.selection.clone();
        self.clear_selection(stage);
        let next = if self.select_top_level {
            stage
                .roots()
                .iter()
                .copied()
                .filter(|root| {
                    stage
                        .family(*root)
                        .iter()
                        .any(|member| current.contains(member))
                })
                .collect()
        } else {
            let mut members = Vec::new();
            for mob in current {
                for member in stage.family(mob) {
                    if stage
                        .get(member)
                        .is_some_and(|entry| !entry.buffer.is_empty())
                        && !members.contains(&member)
                    {
                        members.push(member);
                    }
                }
            }
            members
        };
        self.add_to_selection(stage, &next);
    }

    fn add_to_selection(&mut self, stage: &mut Stage, mobs: &[Mob]) {
        for &mob in mobs {
            if !stage.contains(mob)
                || self.unselectables.contains(&mob)
                || self.selection.contains(&mob)
            {
                continue;
            }
            self.selection.push(mob);
            stage.set_animating_status(mob, true, true);
        }
    }

    fn toggle_selection(&mut self, stage: &mut Stage, mobs: &[Mob]) {
        for &mob in mobs {
            if let Some(index) = self.selection.iter().position(|selected| *selected == mob) {
                self.selection.remove(index);
                stage.set_animating_status(mob, false, true);
                for &selected in &self.selection {
                    stage.set_animating_status(selected, true, true);
                }
            } else {
                self.add_to_selection(stage, &[mob]);
            }
        }
    }

    fn clear_selection(&mut self, stage: &mut Stage) {
        for mob in self.selection.drain(..) {
            stage.set_animating_status(mob, false, true);
        }
    }

    fn delete_selection(&mut self, stage: &mut Stage) {
        if self.selection.is_empty() {
            return;
        }
        self.save_undo(stage);
        let selected = self.selection.clone();
        stage.remove_many_from_scene(&selected);
        self.clear_selection(stage);
    }

    fn copy_selection(&mut self, stage: &mut Stage) {
        let mut templates = Vec::new();
        for mob in &self.selection {
            if let Ok(copy) = stage.copy_family(*mob) {
                templates.push(copy);
            }
        }
        let replacement = if templates.is_empty() {
            InteractiveClipboard::Empty
        } else {
            InteractiveClipboard::Mobjects(templates)
        };
        self.replace_clipboard(stage, replacement);
    }

    fn paste_selection(&mut self, stage: &mut Stage) {
        let InteractiveClipboard::Mobjects(templates) = &self.clipboard else {
            return;
        };
        let templates = templates.clone();
        self.save_undo(stage);
        let mut copies = Vec::new();
        for template in templates {
            if let Ok(copy) = stage.copy_family(template)
                && stage.add_to_scene(copy).is_ok()
            {
                copies.push(copy);
            }
        }
        self.clear_selection(stage);
        self.add_to_selection(stage, &copies);
    }

    fn group_selection(&mut self, stage: &mut Stage) {
        if self.selection.is_empty() {
            return;
        }
        self.save_undo(stage);
        let children = self.selection.clone();
        let group = stage.add(Mobject::new());
        for child in &children {
            stage.remove_from_scene(*child);
            if stage.attach(group, *child).is_err() {
                return;
            }
        }
        if stage.add_to_scene(group).is_ok() {
            self.clear_selection(stage);
            self.add_to_selection(stage, &[group]);
        }
    }

    fn ungroup_selection(&mut self, stage: &mut Stage) {
        if self.selection.is_empty() {
            return;
        }
        self.save_undo(stage);
        let groups = self.selection.clone();
        let mut pieces = Vec::new();
        for group in groups {
            let children = stage
                .get(group)
                .map(|entry| entry.submobjects().to_vec())
                .unwrap_or_default();
            if children.is_empty() {
                continue;
            }
            stage.remove_from_scene(group);
            for child in children {
                stage.detach(group, child);
                if stage.add_to_scene(child).is_ok() {
                    pieces.push(child);
                }
            }
        }
        self.clear_selection(stage);
        self.add_to_selection(stage, &pieces);
    }

    fn choose_color(&mut self, stage: &mut Stage, point: Vec3) {
        let Some(source) = self.topmost_color_source(stage, point) else {
            return;
        };
        let Some(color) = read_color(stage, source) else {
            return;
        };
        self.save_undo(stage);
        write_color(stage, &self.selection, color);
    }

    fn replace_clipboard(&mut self, stage: &mut Stage, replacement: InteractiveClipboard) {
        let previous = std::mem::replace(&mut self.clipboard, replacement);
        let InteractiveClipboard::Mobjects(templates) = previous else {
            return;
        };

        let mut family = Vec::new();
        for template in templates {
            for member in stage.family(template) {
                if !family.contains(&member) {
                    family.push(member);
                }
            }
        }
        for member in family.into_iter().rev() {
            let _ = stage.delete(member);
        }
    }

    fn prune(&mut self, stage: &Stage) {
        self.selection.retain(|mob| stage.contains(*mob));
        self.unselectables.retain(|mob| stage.contains(*mob));
    }
}

/// Engine-side interactive Scene.
pub struct InteractiveScene {
    scene: Scene,
    state: Rc<RefCell<InteractionState>>,
    listener_ids: Vec<EventListenerId>,
}

impl InteractiveScene {
    /// Wrap a Scene and register its global interaction listeners.
    pub fn new(mut scene: Scene) -> Result<Self, EventError> {
        let state = Rc::new(RefCell::new(InteractionState::default()));
        let mut listener_ids = Vec::new();
        for event_type in [
            EventType::MouseMotion,
            EventType::MousePress,
            EventType::MouseRelease,
            EventType::MouseDrag,
            EventType::MouseScroll,
            EventType::KeyPress,
            EventType::KeyRelease,
        ] {
            let shared = Rc::clone(&state);
            let listener = EventListener::new(
                event_type,
                EventTarget::Global,
                move |event, _target, dispatch, stage| {
                    shared.borrow_mut().handle(event, dispatch, stage)
                },
            );
            listener_ids.push(scene.event_dispatcher_mut().add_listener(listener)?);
        }
        Ok(Self {
            scene,
            state,
            listener_ids,
        })
    }

    /// Shared Scene.
    #[must_use]
    pub const fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Mutable Scene.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Consume the wrapper. Registered callbacks remain valid because they own
    /// the shared interaction state.
    #[must_use]
    pub fn into_scene(self) -> Scene {
        self.scene
    }

    /// Listener tokens registered by this behavior layer.
    #[must_use]
    pub fn listener_ids(&self) -> &[EventListenerId] {
        &self.listener_ids
    }

    /// Selected handles in selection order.
    #[must_use]
    pub fn selection(&self) -> Vec<Mob> {
        self.state.borrow().selection.clone()
    }

    /// Whether selection searches top-level scene roots.
    #[must_use]
    pub fn selects_top_level(&self) -> bool {
        self.state.borrow().select_top_level
    }

    /// Current selection rectangle for a Studio overlay.
    #[must_use]
    pub fn selection_rectangle(&self) -> Option<SelectionRectangle> {
        let state = self.state.borrow();
        let start = state.selection_start?;
        Some(SelectionRectangle {
            start,
            end: state.selection_current,
            bounds: rectangle_bounds(start, state.selection_current),
        })
    }

    /// Current selection highlights for a Studio overlay.
    #[must_use]
    pub fn selection_highlights(&self) -> Vec<SelectionHighlight> {
        self.state
            .borrow()
            .selection
            .iter()
            .copied()
            .filter(|mob| self.scene.stage().contains(*mob))
            .map(|target| SelectionHighlight {
                target,
                bounds: self.scene.stage().get_bounding_box(target),
            })
            .collect()
    }

    /// Scene-owned clipboard.
    #[must_use]
    pub fn clipboard(&self) -> InteractiveClipboard {
        self.state.borrow().clipboard.clone()
    }

    /// Replace text clipboard data supplied by a host.
    pub fn set_clipboard_text(&mut self, text: impl Into<String>) {
        let Self { scene, state, .. } = self;
        state
            .borrow_mut()
            .replace_clipboard(scene.stage_mut(), InteractiveClipboard::Text(text.into()));
    }

    /// Exclude whole families from selection/hit search.
    pub fn disable_interaction(&mut self, mobs: &[Mob]) {
        let mut state = self.state.borrow_mut();
        for &mob in mobs {
            for member in self.scene.stage().family(mob) {
                if !state.unselectables.contains(&member) {
                    state.unselectables.push(member);
                }
            }
        }
    }

    /// Re-enable whole families for selection/hit search.
    pub fn enable_interaction(&mut self, mobs: &[Mob]) {
        let mut state = self.state.borrow_mut();
        for &mob in mobs {
            let family = self.scene.stage().family(mob);
            state
                .unselectables
                .retain(|member| !family.contains(member));
        }
    }

    /// Whether the information overlay should be shown.
    #[must_use]
    pub fn information_visible(&self) -> bool {
        self.state.borrow().information_visible
    }

    /// Whether the cursor/crosshair overlay should be shown.
    #[must_use]
    pub fn cursor_visible(&self) -> bool {
        self.state.borrow().cursor_visible
    }
}

impl Default for InteractiveScene {
    fn default() -> Self {
        Self::new(Scene::default()).expect("fresh dispatcher has listener capacity")
    }
}

impl std::ops::Deref for InteractiveScene {
    type Target = Scene;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

impl std::ops::DerefMut for InteractiveScene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}

fn selection_bounds(stage: &Stage, selection: &[Mob]) -> Option<BoundingBox> {
    let mut any = false;
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for &mob in selection {
        if !stage.contains(mob) {
            continue;
        }
        let bounds = stage.get_bounding_box(mob);
        any = true;
        for axis in 0..3 {
            min[axis] = min[axis].min(bounds.min[axis]);
            max[axis] = max[axis].max(bounds.max[axis]);
        }
    }
    any.then(|| bounds_from_min_max(min, max))
}

fn rectangle_bounds(start: Vec3, end: Vec3) -> BoundingBox {
    let mut min = [0.0; 3];
    let mut max = [0.0; 3];
    for axis in 0..3 {
        min[axis] = start[axis].min(end[axis]);
        max[axis] = start[axis].max(end[axis]);
    }
    bounds_from_min_max(min, max)
}

fn bounds_from_min_max(min: Vec3, max: Vec3) -> BoundingBox {
    BoundingBox {
        min,
        mid: [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ],
        max,
    }
}

fn point_in_box(point: Vec3, bounds: BoundingBox, buffer: f64) -> bool {
    (0..3).all(|axis| {
        point[axis] >= bounds.min[axis] - buffer && point[axis] <= bounds.max[axis] + buffer
    })
}

fn boxes_intersect(left: BoundingBox, right: BoundingBox, buffer: f64) -> bool {
    (0..3).all(|axis| {
        left.max[axis] >= right.min[axis] - buffer && left.min[axis] <= right.max[axis] + buffer
    })
}

fn read_color(stage: &Stage, mob: Mob) -> Option<Srgb> {
    for member in stage.family(mob) {
        let entry = stage.get(member)?;
        for field in ["fill_rgba", "stroke_rgba", "rgba"] {
            let Some(value) = entry.buffer.read(0, field) else {
                continue;
            };
            if field == "fill_rgba" && value[3] == 0.0 {
                continue;
            }
            return Some(Srgb {
                r: f64::from(value[0]),
                g: f64::from(value[1]),
                b: f64::from(value[2]),
            });
        }
    }
    None
}

#[allow(clippy::cast_possible_truncation)]
fn write_color(stage: &mut Stage, mobs: &[Mob], color: Srgb) {
    let mut members = Vec::new();
    for &mob in mobs {
        for member in stage.family(mob) {
            if !members.contains(&member) {
                members.push(member);
            }
        }
    }
    for member in members {
        let Some(entry) = stage.get_mut(member) else {
            continue;
        };
        for field in ["fill_rgba", "stroke_rgba", "rgba"] {
            for index in 0..entry.buffer.len() {
                let Some(mut value) = entry.buffer.read(index, field) else {
                    break;
                };
                value[0] = color.r as f32;
                value[1] = color.g as f32;
                value[2] = color.b as f32;
                entry.buffer.write(index, field, &value);
            }
        }
    }
}

fn sub(left: Vec3, right: Vec3) -> Vec3 {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scaled(vector: Vec3, scalar: f64) -> Vec3 {
    [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar]
}

fn norm(vector: Vec3) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

fn sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}
