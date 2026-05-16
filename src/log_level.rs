pub(crate) enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

impl LogLevel {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}
