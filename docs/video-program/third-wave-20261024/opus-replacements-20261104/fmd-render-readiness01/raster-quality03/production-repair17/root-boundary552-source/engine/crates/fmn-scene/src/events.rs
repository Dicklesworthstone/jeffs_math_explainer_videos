//! Typed input events and the deterministic dispatcher (§13.2, fm-eiw).
//!
//! The Reference surface is kept where it is semantic: seven event types,
//! insertion-ordered listeners, pointer hit testing, drag capture on press,
//! pressed-key state, and propagation stopping only on an explicit refusal.
//! The Rust surface deliberately uses the correctly spelled `listener`
//! vocabulary (Appendix C-9); typo aliases belong only in `fmn-python`.
//!
//! Dispatch is a serial-front-end operation. A callback receives the live
//! [`Stage`] only while the Scene is at its pre-capture boundary, and an
//! [`InputEvent`] carries both a stable sequence and an exact
//! [`RationalTime`] for journal replay.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};

use fmn_anim::RationalTime;
use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Stage};

/// The Reference's seven event classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Pointer movement without a pressed button.
    MouseMotion,
    /// Pointer button press.
    MousePress,
    /// Pointer button release.
    MouseRelease,
    /// Pointer movement while a button is held.
    MouseDrag,
    /// Wheel or trackpad scroll.
    MouseScroll,
    /// Keyboard key press.
    KeyPress,
    /// Keyboard key release.
    KeyRelease,
}

impl EventType {
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::MouseMotion => 0,
            Self::MousePress => 1,
            Self::MouseRelease => 2,
            Self::MouseDrag => 3,
            Self::MouseScroll => 4,
            Self::KeyPress => 5,
            Self::KeyRelease => 6,
        }
    }

    pub(crate) fn from_code(code: u8) -> Result<Self, EventError> {
        Ok(match code {
            0 => Self::MouseMotion,
            1 => Self::MousePress,
            2 => Self::MouseRelease,
            3 => Self::MouseDrag,
            4 => Self::MouseScroll,
            5 => Self::KeyPress,
            6 => Self::KeyRelease,
            _ => return Err(EventError::Malformed("event type")),
        })
    }

    const fn is_mouse(self) -> bool {
        matches!(
            self,
            Self::MouseMotion
                | Self::MousePress
                | Self::MouseRelease
                | Self::MouseDrag
                | Self::MouseScroll
        )
    }
}

/// Platform-neutral pointer buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Primary/left button.
    Left,
    /// Middle button.
    Middle,
    /// Secondary/right button.
    Right,
    /// A host-defined additional button.
    Other(u16),
}

impl MouseButton {
    pub(crate) const fn code(self) -> (u8, u16) {
        match self {
            Self::Left => (0, 0),
            Self::Middle => (1, 0),
            Self::Right => (2, 0),
            Self::Other(code) => (3, code),
        }
    }

    pub(crate) fn from_code(tag: u8, value: u16) -> Result<Self, EventError> {
        Ok(match tag {
            0 if value == 0 => Self::Left,
            1 if value == 0 => Self::Middle,
            2 if value == 0 => Self::Right,
            3 => Self::Other(value),
            _ => return Err(EventError::Malformed("mouse button")),
        })
    }
}

/// Platform-neutral keys used by Scene and Studio input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A Unicode scalar. Shortcut characters are canonicalized to lowercase.
    Character(char),
    /// Backspace/delete-backward.
    Backspace,
    /// Left arrow.
    ArrowLeft,
    /// Up arrow.
    ArrowUp,
    /// Right arrow.
    ArrowRight,
    /// Down arrow.
    ArrowDown,
    /// Escape.
    Escape,
    /// Enter/return.
    Enter,
    /// Tab.
    Tab,
    /// A host key not otherwise represented, with a stable host-defined code.
    Other(u32),
}

impl Key {
    fn canonicalized(self) -> Self {
        match self {
            Self::Character(character) => Self::Character(character.to_ascii_lowercase()),
            other => other,
        }
    }

    pub(crate) const fn code(self) -> (u8, u32) {
        match self {
            Self::Character(character) => (0, character as u32),
            Self::Backspace => (1, 0),
            Self::ArrowLeft => (2, 0),
            Self::ArrowUp => (3, 0),
            Self::ArrowRight => (4, 0),
            Self::ArrowDown => (5, 0),
            Self::Escape => (6, 0),
            Self::Enter => (7, 0),
            Self::Tab => (8, 0),
            Self::Other(code) => (9, code),
        }
    }

    pub(crate) fn from_code(tag: u8, value: u32) -> Result<Self, EventError> {
        Ok(match tag {
            0 => Self::Character(
                char::from_u32(value).ok_or(EventError::Malformed("key character"))?,
            ),
            1..=8 if value != 0 => return Err(EventError::Malformed("key payload")),
            1 => Self::Backspace,
            2 => Self::ArrowLeft,
            3 => Self::ArrowUp,
            4 => Self::ArrowRight,
            5 => Self::ArrowDown,
            6 => Self::Escape,
            7 => Self::Enter,
            8 => Self::Tab,
            9 => Self::Other(value),
            _ => return Err(EventError::Malformed("key tag")),
        })
    }
}

/// Keyboard modifiers as a stable bit set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// Shift.
    pub const SHIFT: Self = Self(1 << 0);
    /// Control.
    pub const CONTROL: Self = Self(1 << 1);
    /// macOS Command.
    pub const COMMAND: Self = Self(1 << 2);
    /// Alt/Option.
    pub const ALT: Self = Self(1 << 3);
    /// Either Control or Command, the cross-platform primary shortcut.
    pub const PRIMARY: Self = Self(Self::CONTROL.0 | Self::COMMAND.0);
    /// Every accepted bit.
    pub const ALL: Self = Self(Self::SHIFT.0 | Self::CONTROL.0 | Self::COMMAND.0 | Self::ALT.0);

    /// No modifiers.
    pub const NONE: Self = Self(0);

    /// Construct after rejecting unknown bits.
    pub fn from_bits(bits: u8) -> Result<Self, EventError> {
        if bits & !Self::ALL.0 != 0 {
            return Err(EventError::Malformed("modifier bits"));
        }
        Ok(Self(bits))
    }

    /// Stable serialized bit pattern.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Whether every bit in `other` is present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any bit in `other` is present.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether no modifiers are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Typed payload for one of the seven event classes.
#[derive(Debug, Clone, PartialEq)]
pub enum EventPayload {
    /// Pointer movement.
    MouseMotion {
        /// Current scene-space point.
        point: Vec3,
        /// Delta from the preceding point.
        delta: Vec3,
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Pointer button press.
    MousePress {
        /// Current scene-space point.
        point: Vec3,
        /// Pressed button.
        button: MouseButton,
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Pointer button release.
    MouseRelease {
        /// Current scene-space point.
        point: Vec3,
        /// Released button.
        button: MouseButton,
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Pointer drag.
    MouseDrag {
        /// Current scene-space point.
        point: Vec3,
        /// Delta from the preceding point.
        delta: Vec3,
        /// Held button.
        button: MouseButton,
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Wheel/trackpad scroll.
    MouseScroll {
        /// Current scene-space point.
        point: Vec3,
        /// Horizontal/vertical scroll offset.
        offset: [f64; 2],
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Key press.
    KeyPress {
        /// Pressed key.
        key: Key,
        /// Active modifiers.
        modifiers: Modifiers,
    },
    /// Key release.
    KeyRelease {
        /// Released key.
        key: Key,
        /// Active modifiers.
        modifiers: Modifiers,
    },
}

impl EventPayload {
    /// Payload's event class.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        match self {
            Self::MouseMotion { .. } => EventType::MouseMotion,
            Self::MousePress { .. } => EventType::MousePress,
            Self::MouseRelease { .. } => EventType::MouseRelease,
            Self::MouseDrag { .. } => EventType::MouseDrag,
            Self::MouseScroll { .. } => EventType::MouseScroll,
            Self::KeyPress { .. } => EventType::KeyPress,
            Self::KeyRelease { .. } => EventType::KeyRelease,
        }
    }

    /// Scene-space point carried by a mouse event.
    #[must_use]
    pub const fn point(&self) -> Option<Vec3> {
        match self {
            Self::MouseMotion { point, .. }
            | Self::MousePress { point, .. }
            | Self::MouseRelease { point, .. }
            | Self::MouseDrag { point, .. }
            | Self::MouseScroll { point, .. } => Some(*point),
            Self::KeyPress { .. } | Self::KeyRelease { .. } => None,
        }
    }

    /// Modifiers carried by this event.
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        match self {
            Self::MouseMotion { modifiers, .. }
            | Self::MousePress { modifiers, .. }
            | Self::MouseRelease { modifiers, .. }
            | Self::MouseDrag { modifiers, .. }
            | Self::MouseScroll { modifiers, .. }
            | Self::KeyPress { modifiers, .. }
            | Self::KeyRelease { modifiers, .. } => *modifiers,
        }
    }

    /// Validate all numeric and bit-set invariants.
    pub fn validate(&self) -> Result<(), EventError> {
        if let Some(point) = self.point() {
            validate_vec3(point)?;
        }
        match self {
            Self::MouseMotion { delta, .. } | Self::MouseDrag { delta, .. } => {
                validate_vec3(*delta)?;
            }
            Self::MouseScroll { offset, .. } => {
                if offset.iter().any(|value| !value.is_finite()) {
                    return Err(EventError::NonFiniteCoordinate);
                }
            }
            Self::MousePress { .. }
            | Self::MouseRelease { .. }
            | Self::KeyPress { .. }
            | Self::KeyRelease { .. } => {}
        }
        Modifiers::from_bits(self.modifiers().bits())?;
        Ok(())
    }

    fn canonicalize(&mut self) {
        match self {
            Self::MouseMotion { point, delta, .. } => {
                canonicalize_vec3(point);
                canonicalize_vec3(delta);
            }
            Self::MouseDrag { point, delta, .. } => {
                canonicalize_vec3(point);
                canonicalize_vec3(delta);
            }
            Self::MousePress { point, .. } | Self::MouseRelease { point, .. } => {
                canonicalize_vec3(point);
            }
            Self::MouseScroll { point, offset, .. } => {
                canonicalize_vec3(point);
                canonicalize_zero(&mut offset[0]);
                canonicalize_zero(&mut offset[1]);
            }
            Self::KeyPress { key, .. } | Self::KeyRelease { key, .. } => {
                *key = key.canonicalized();
            }
        }
    }
}

fn validate_vec3(vector: Vec3) -> Result<(), EventError> {
    if vector.iter().any(|value| !value.is_finite()) {
        Err(EventError::NonFiniteCoordinate)
    } else {
        Ok(())
    }
}

fn canonicalize_vec3(vector: &mut Vec3) {
    for value in vector {
        canonicalize_zero(value);
    }
}

fn canonicalize_zero(value: &mut f64) {
    if *value == 0.0 {
        *value = 0.0;
    }
}

/// One canonical, replayable event.
#[derive(Debug, Clone, PartialEq)]
pub struct InputEvent {
    /// Stable input order within the session.
    pub sequence: u64,
    /// Exact serial-front-end dispatch time.
    pub timestamp: RationalTime,
    /// Typed payload.
    pub payload: EventPayload,
}

impl InputEvent {
    /// Validate and canonicalize a recorded event.
    pub fn new(
        sequence: u64,
        timestamp: RationalTime,
        mut payload: EventPayload,
    ) -> Result<Self, EventError> {
        if timestamp.fps() == 0 {
            return Err(EventError::ZeroFps);
        }
        if timestamp.frames() < 0 {
            return Err(EventError::TimestampBeforeZero);
        }
        payload.validate()?;
        payload.canonicalize();
        Ok(Self {
            sequence,
            timestamp,
            payload,
        })
    }

    /// Recheck an event received from a durable or host boundary.
    pub fn validate(&self) -> Result<(), EventError> {
        if self.timestamp.fps() == 0 {
            return Err(EventError::ZeroFps);
        }
        if self.timestamp.frames() < 0 {
            return Err(EventError::TimestampBeforeZero);
        }
        self.payload.validate()
    }

    /// Event class.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.payload.event_type()
    }
}

/// Thread-safe host-to-Scene input queue.
///
/// Studio/browser/TUI adapters keep a clone and submit payloads while a Scene
/// is driving a segment. The serial front end drains arrival order at each
/// pre-capture boundary, assigns exact timestamps and sequence ids there, and
/// journals the result. No callback or Stage reference crosses this queue.
#[derive(Clone)]
pub struct EventInbox {
    queue: Arc<Mutex<VecDeque<EventPayload>>>,
    capacity: usize,
}

impl fmt::Debug for EventInbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventInbox")
            .field("pending", &self.len())
            .field("capacity", &self.capacity)
            .finish()
    }
}

impl EventInbox {
    /// Empty inbox with an exact pending-payload ceiling.
    ///
    /// Zero capacity explicitly disables host ingress. Storage for every
    /// admissible payload is reserved before the handle is published, so a
    /// successful [`Self::submit`] never grows the queue.
    pub fn new(capacity: usize) -> Result<Self, EventError> {
        let mut queue = VecDeque::new();
        queue
            .try_reserve_exact(capacity)
            .map_err(|_| EventError::InboxStorageUnavailable { capacity })?;
        Ok(Self {
            queue: Arc::new(Mutex::new(queue)),
            capacity,
        })
    }

    /// Validate, canonicalize, and append one host payload.
    pub fn submit(&self, payload: EventPayload) -> Result<(), EventError> {
        let canonical = InputEvent::new(0, RationalTime::zero(1), payload)?.payload;
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if queue.len() >= self.capacity {
            return Err(EventError::InboxFull {
                capacity: self.capacity,
            });
        }
        queue.push_back(canonical);
        Ok(())
    }

    /// Maximum pending payloads admitted across every clone.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of payloads waiting for a serial boundary.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Whether no payloads are waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn drain_if_at_most(
        &self,
        capacity: u128,
    ) -> Result<Option<Vec<EventPayload>>, EventError> {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending =
            u128::try_from(queue.len()).map_err(|_| EventError::InboxStorageUnavailable {
                capacity: queue.len(),
            })?;
        if pending > capacity {
            return Ok(None);
        }
        let mut payloads = Vec::new();
        payloads.try_reserve_exact(queue.len()).map_err(|_| {
            EventError::InboxStorageUnavailable {
                capacity: queue.len(),
            }
        })?;
        payloads.extend(queue.drain(..));
        Ok(Some(payloads))
    }

    pub(crate) fn clear(&self) {
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

/// Input/dispatcher refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    /// Pointer coordinates or deltas must be finite.
    NonFiniteCoordinate,
    /// Event timestamps cannot precede scene time zero.
    TimestampBeforeZero,
    /// A timestamp grid must have nonzero fps.
    ZeroFps,
    /// A replay event belongs to a different rational frame grid.
    TimestampGridMismatch {
        /// Scene fps.
        expected: u32,
        /// Event fps.
        found: u32,
    },
    /// Replay timestamps moved backward or a sequence id was reused.
    ReplayOutOfOrder,
    /// The session event sequence cannot be incremented without reuse.
    SequenceExhausted,
    /// The configured host-input queue has no free slot.
    InboxFull {
        /// Complete queue capacity shared by every handle clone.
        capacity: usize,
    },
    /// Storage for the configured inbox or one atomic drain was unavailable.
    InboxStorageUnavailable {
        /// Number of payload slots requested.
        capacity: usize,
    },
    /// Listener tokens cannot be incremented without reuse.
    ListenerIdExhausted,
    /// A durable enum, scalar, or bit set was invalid.
    Malformed(&'static str),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteCoordinate => f.write_str("event coordinates must be finite"),
            Self::TimestampBeforeZero => f.write_str("event timestamp cannot precede scene zero"),
            Self::ZeroFps => f.write_str("event timestamp fps must be nonzero"),
            Self::TimestampGridMismatch { expected, found } => {
                write!(
                    f,
                    "event timestamp uses {found} fps; scene uses {expected} fps"
                )
            }
            Self::ReplayOutOfOrder => f.write_str(
                "replay timestamps must not move backward and sequence ids must increase",
            ),
            Self::SequenceExhausted => f.write_str("event sequence space is exhausted"),
            Self::InboxFull { capacity } => {
                write!(f, "event inbox is full at its {capacity}-payload capacity")
            }
            Self::InboxStorageUnavailable { capacity } => write!(
                f,
                "event inbox could not reserve storage for {capacity} payloads"
            ),
            Self::ListenerIdExhausted => f.write_str("event listener id space is exhausted"),
            Self::Malformed(what) => write!(f, "malformed event {what}"),
        }
    }
}

impl std::error::Error for EventError {}

/// A callback's propagation decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPropagation {
    /// Continue with the next eligible listener.
    Continue,
    /// Stop this event after the current listener.
    Stop,
}

/// Stable listener removal token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventListenerId(u64);

impl EventListenerId {
    /// Stable integer value for protocol/inspector surfaces.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// What a listener is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTarget {
    /// Scene-wide listener; always eligible.
    Global,
    /// A listener hit-tested against this mobject for pointer events.
    Mobject(Mob),
}

/// State after the dispatcher has incorporated the current event.
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchState {
    mouse_point: Vec3,
    mouse_drag_point: Vec3,
    pressed_keys: Vec<Key>,
}

impl DispatchState {
    /// Latest pointer position.
    #[must_use]
    pub const fn mouse_point(&self) -> Vec3 {
        self.mouse_point
    }

    /// Latest drag position.
    #[must_use]
    pub const fn mouse_drag_point(&self) -> Vec3 {
        self.mouse_drag_point
    }

    /// Pressed keys, in first-press order.
    #[must_use]
    pub fn pressed_keys(&self) -> &[Key] {
        &self.pressed_keys
    }

    /// Whether a key is pressed.
    #[must_use]
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key.canonicalized())
    }
}

type EventCallback =
    dyn FnMut(&InputEvent, EventTarget, &DispatchState, &mut Stage) -> EventPropagation;

/// One typed listener.
pub struct EventListener {
    event_type: EventType,
    target: EventTarget,
    callback: Box<EventCallback>,
}

impl EventListener {
    /// Construct a listener. Registration assigns its stable id.
    #[must_use]
    pub fn new(
        event_type: EventType,
        target: EventTarget,
        callback: impl FnMut(&InputEvent, EventTarget, &DispatchState, &mut Stage) -> EventPropagation
        + 'static,
    ) -> Self {
        Self {
            event_type,
            target,
            callback: Box::new(callback),
        }
    }

    /// Event class.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Target.
    #[must_use]
    pub const fn target(&self) -> EventTarget {
        self.target
    }
}

impl fmt::Debug for EventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventListener")
            .field("event_type", &self.event_type)
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

struct RegisteredListener {
    id: EventListenerId,
    listener: EventListener,
}

/// Deterministic, insertion-ordered event dispatcher.
pub struct EventDispatcher {
    listeners: Vec<RegisteredListener>,
    next_listener_id: Option<u64>,
    mouse_point: Vec3,
    mouse_drag_point: Vec3,
    pressed_keys: Vec<Key>,
    drag_capture: Vec<EventListenerId>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self {
            listeners: Vec::new(),
            next_listener_id: Some(0),
            mouse_point: [0.0; 3],
            mouse_drag_point: [0.0; 3],
            pressed_keys: Vec::new(),
            drag_capture: Vec::new(),
        }
    }
}

impl fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("listeners", &self.listeners.len())
            .field("mouse_point", &self.mouse_point)
            .field("mouse_drag_point", &self.mouse_drag_point)
            .field("pressed_keys", &self.pressed_keys)
            .field("drag_capture", &self.drag_capture)
            .finish()
    }
}

impl EventDispatcher {
    /// Empty dispatcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a listener and return its stable removal token.
    pub fn add_listener(&mut self, listener: EventListener) -> Result<EventListenerId, EventError> {
        let raw = self
            .next_listener_id
            .ok_or(EventError::ListenerIdExhausted)?;
        let id = EventListenerId(raw);
        self.next_listener_id = raw.checked_add(1);
        self.listeners.push(RegisteredListener { id, listener });
        Ok(id)
    }

    /// Remove one registered listener. Unknown ids are an idempotent no-op.
    pub fn remove_listener(&mut self, id: EventListenerId) -> bool {
        let before = self.listeners.len();
        self.listeners.retain(|slot| slot.id != id);
        self.drag_capture.retain(|captured| *captured != id);
        self.listeners.len() != before
    }

    /// Remove every listener matching a target and event class.
    pub fn remove_listeners(&mut self, target: EventTarget, event_type: EventType) -> usize {
        let removed: Vec<_> = self
            .listeners
            .iter()
            .filter(|slot| slot.listener.target == target && slot.listener.event_type == event_type)
            .map(|slot| slot.id)
            .collect();
        self.listeners.retain(|slot| !removed.contains(&slot.id));
        self.drag_capture
            .retain(|captured| !removed.contains(captured));
        removed.len()
    }

    /// Total registered listeners.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Listener ids for one class, in registration order.
    #[must_use]
    pub fn listener_ids(&self, event_type: EventType) -> Vec<EventListenerId> {
        self.listeners
            .iter()
            .filter(|slot| slot.listener.event_type == event_type)
            .map(|slot| slot.id)
            .collect()
    }

    /// Current dispatcher state.
    #[must_use]
    pub fn state(&self) -> DispatchState {
        DispatchState {
            mouse_point: self.mouse_point,
            mouse_drag_point: self.mouse_drag_point,
            pressed_keys: self.pressed_keys.clone(),
        }
    }

    /// Clear transient pointer/key/drag state while retaining registrations.
    pub fn reset_state(&mut self) {
        self.mouse_point = [0.0; 3];
        self.mouse_drag_point = [0.0; 3];
        self.pressed_keys.clear();
        self.drag_capture.clear();
    }

    /// Incorporate and dispatch one event.
    ///
    /// Pointer listeners are hit-tested. Drag listeners are captured on press
    /// and remain the only eligible drag listeners until release. Key listeners
    /// are not hit-tested. In every case, registration order is preserved and
    /// only [`EventPropagation::Stop`] terminates propagation.
    pub fn dispatch(&mut self, event: &InputEvent, stage: &mut Stage) -> EventPropagation {
        self.update_state(event, stage);
        let state = self.state();
        let event_type = event.event_type();
        let eligible: Vec<_> = if event_type == EventType::MouseDrag {
            self.drag_capture.clone()
        } else {
            self.listeners
                .iter()
                .filter(|slot| slot.listener.event_type == event_type)
                .filter(|slot| {
                    !event_type.is_mouse()
                        || target_matches(slot.listener.target, event.payload.point(), stage)
                })
                .map(|slot| slot.id)
                .collect()
        };

        for id in eligible {
            let Some(slot) = self.listeners.iter_mut().find(|slot| slot.id == id) else {
                continue;
            };
            let result = (slot.listener.callback)(event, slot.listener.target, &state, stage);
            if result == EventPropagation::Stop {
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }

    fn update_state(&mut self, event: &InputEvent, stage: &Stage) {
        if let Some(point) = event.payload.point() {
            self.mouse_point = point;
        }
        match &event.payload {
            EventPayload::MouseDrag { point, .. } => {
                self.mouse_drag_point = *point;
            }
            EventPayload::KeyPress { key, .. } => {
                let key = key.canonicalized();
                if !self.pressed_keys.contains(&key) {
                    self.pressed_keys.push(key);
                }
            }
            EventPayload::KeyRelease { key, .. } => {
                let key = key.canonicalized();
                self.pressed_keys.retain(|pressed| *pressed != key);
            }
            EventPayload::MousePress { point, .. } => {
                self.mouse_drag_point = *point;
                self.drag_capture = self
                    .listeners
                    .iter()
                    .filter(|slot| slot.listener.event_type == EventType::MouseDrag)
                    .filter(|slot| target_matches(slot.listener.target, Some(*point), stage))
                    .map(|slot| slot.id)
                    .collect();
            }
            EventPayload::MouseRelease { .. } => {
                self.drag_capture.clear();
            }
            EventPayload::MouseMotion { .. } | EventPayload::MouseScroll { .. } => {}
        }
    }
}

fn target_matches(target: EventTarget, point: Option<Vec3>, stage: &Stage) -> bool {
    match target {
        EventTarget::Global => true,
        EventTarget::Mobject(mob) => {
            let Some(point) = point else {
                return true;
            };
            if !stage.contains(mob)
                || !stage.family(mob).iter().any(|member| {
                    stage
                        .get(*member)
                        .is_some_and(|entry| !entry.buffer.is_empty())
                })
            {
                return false;
            }
            let bounds = stage.get_bounding_box(mob);
            const BUFFER: f64 = 1.0e-2;
            (0..3).all(|axis| {
                point[axis] >= bounds.min[axis] - BUFFER && point[axis] <= bounds.max[axis] + BUFFER
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_key_tags_require_the_canonical_zero_payload() {
        let fixed = [
            (1, Key::Backspace),
            (2, Key::ArrowLeft),
            (3, Key::ArrowUp),
            (4, Key::ArrowRight),
            (5, Key::ArrowDown),
            (6, Key::Escape),
            (7, Key::Enter),
            (8, Key::Tab),
        ];
        for (tag, key) in fixed {
            assert_eq!(Key::from_code(tag, 0), Ok(key));
            assert_eq!(
                Key::from_code(tag, 1),
                Err(EventError::Malformed("key payload"))
            );
        }
    }
}
