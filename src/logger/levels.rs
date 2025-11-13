use serde::Serialize;

/// Log levels for filtering and categorizing log messages.
///
/// Levels are ordered by severity: Debug < Info < Warn < Error
#[derive(Serialize, PartialEq, PartialOrd, Default, Copy, Clone, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    /// Debug level - lowest severity (default)
    #[default]
    Debug = 0,
    /// Informational messages
    Info = 1,
    /// Warning messages
    Warn = 2,
    /// Error messages - highest severity
    Error = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
        }
    }
}
