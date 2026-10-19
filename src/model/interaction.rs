#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SettingsFieldAvailability {
    #[default]
    Available,
    Unavailable {
        reason: String,
    },
}

impl SettingsFieldAvailability {
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::Unavailable { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsFieldValidity {
    #[default]
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SettingsInteractionGate {
    #[default]
    Open,
    Gated {
        reason: String,
    },
}

impl SettingsInteractionGate {
    pub fn gated(reason: impl Into<String>) -> Self {
        Self::Gated {
            reason: reason.into(),
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Open => None,
            Self::Gated { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFooterCommand {
    Accept,
    Apply,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFieldInputRefusal {
    ValueByteLimitExceeded {
        limit: usize,
        attempted_bytes: usize,
    },
}
use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone)]
pub struct SettingsInteractionAdmission {
    current: Weak<Cell<bool>>,
}

impl SettingsInteractionAdmission {
    pub(crate) fn new(current: &Rc<Cell<bool>>) -> Self {
        Self {
            current: Rc::downgrade(current),
        }
    }

    pub fn is_open(&self) -> bool {
        self.current.upgrade().is_some_and(|current| current.get())
    }
}
