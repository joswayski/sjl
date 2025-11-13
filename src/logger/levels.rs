use crate::colors::ColorSettings;
use owo_colors::OwoColorize;
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
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Error => "ERROR",
            Self::Info => "INFO",
            Self::Warn => "WARN",
        }
    }

    #[must_use]
    pub fn get_colored_string(&self, color_settings: &ColorSettings) -> String {
        let level_str = self.as_str();

        let level_text = match self {
            Self::Debug => level_str.truecolor(
                color_settings.debug.red,
                color_settings.debug.green,
                color_settings.debug.blue,
            ),
            Self::Info => level_str.truecolor(
                color_settings.info.red,
                color_settings.info.green,
                color_settings.info.blue,
            ),
            Self::Warn => level_str.truecolor(
                color_settings.warn.red,
                color_settings.warn.green,
                color_settings.warn.blue,
            ),
            Self::Error => level_str.truecolor(
                color_settings.error.red,
                color_settings.error.green,
                color_settings.error.blue,
            ),
        };

        level_text.to_string()
    }
}
