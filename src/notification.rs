use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub timeout: Option<Duration>,
}

impl Notification {
    pub fn new(message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            message: message.into(),
            level,
            created_at: Instant::now(),
            timeout: Some(Duration::from_secs(5)),
        }
    }

    pub fn persistent(message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            message: message.into(),
            level,
            created_at: Instant::now(),
            timeout: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationLevel::Info)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, NotificationLevel::Warning)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationLevel::Error)
    }

    pub fn persistent_info(message: impl Into<String>) -> Self {
        Self::persistent(message, NotificationLevel::Info)
    }

    pub fn is_expired(&self) -> bool {
        match self.timeout {
            Some(timeout) => self.created_at.elapsed() > timeout,
            None => false,
        }
    }

    pub fn remaining_time(&self) -> Duration {
        match self.timeout {
            Some(timeout) => timeout.saturating_sub(self.created_at.elapsed()),
            None => Duration::from_secs(u64::MAX),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NotificationManager {
    current: std::sync::Arc<std::sync::Mutex<Option<Notification>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            current: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn show(&mut self, notification: Notification) {
        if let Ok(mut current) = self.current.lock() {
            *current = Some(notification);
        }
    }

    pub fn show_info(&mut self, message: impl Into<String>) {
        self.show(Notification::info(message));
    }

    pub fn show_warning(&mut self, message: impl Into<String>) {
        self.show(Notification::warning(message));
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.show(Notification::error(message));
    }

    pub fn dismiss(&mut self) {
        if let Ok(mut current) = self.current.lock() {
            *current = None;
        }
    }

    pub fn get_current(&self) -> Option<Notification> {
        self.current.lock().ok().and_then(|c| c.clone())
    }

    pub fn update(&mut self) -> bool {
        if let Ok(mut current) = self.current.lock() {
            if let Some(ref notification) = *current {
                if notification.is_expired() {
                    *current = None;
                    return true;
                }
            }
        }
        false
    }

    pub fn has_notification(&self) -> bool {
        self.current
            .lock()
            .ok()
            .map(|c| c.is_some())
            .unwrap_or(false)
    }
}
