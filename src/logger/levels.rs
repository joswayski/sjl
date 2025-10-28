use serde::Serialize;

/// Log levels for filtering and categorizing log messages.
///
/// Levels are ordered by severity: Debug < Info < Warn < Error
#[derive(Serialize, PartialEq, PartialOrd, Default)]
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
