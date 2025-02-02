use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use merge2::Merge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotificationAction {
    #[default]
    OpenNotification,
    Close,
}

#[derive(Debug, Hash, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum InputAction {
    #[default]
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
    TouchSwipeUp,
    TouchSwipeDown,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum InputModifier {
    Shift,
    #[default]
    Ctrl,
    Alt,
    Super,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    pub modifier: Option<InputModifier>,
    pub action: NotificationAction,
}

impl InputEvent {
    pub fn new(action: NotificationAction) -> Self {
        Self {
            action,
            modifier: None,
        }
    }

    pub fn with_modifiers(mut self, modifier: InputModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Merge)]
#[merge(strategy = merge2::hashmap::replace)]
#[serde(transparent)]
pub struct Action(HashMap<InputAction, InputEvent>);

impl Action {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, action: InputAction, event: InputEvent) {
        self.0.entry(action).or_insert(event);
    }
}

impl Default for Action {
    fn default() -> Self {
        let mut action = Self::new();
        action.add(
            InputAction::LeftClick,
            InputEvent::new(NotificationAction::OpenNotification),
        );
        action.add(
            InputAction::RightClick,
            InputEvent::new(NotificationAction::Close),
        );
        action.add(
            InputAction::ScrollUp,
            InputEvent::new(NotificationAction::Close),
        );
        action
    }
}

impl From<HashMap<InputAction, InputEvent>> for Action {
    fn from(map: HashMap<InputAction, InputEvent>) -> Self {
        Self(map)
    }
}

impl Deref for Action {
    type Target = HashMap<InputAction, InputEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Action {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
