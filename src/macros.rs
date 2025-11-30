#[macro_export]
macro_rules! debug {
    ($msg:expr, $data:expr) => {
        $crate::get_global_logger().__log_with_message(
            Some(std::borrow::Cow::from($msg)),
            &$data,
            $crate::LogLevel::Debug,
        )
    };
    ($data:expr) => {
        $crate::get_global_logger().__log_with_message(None, &$data, $crate::LogLevel::Debug)
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr, $data:expr) => {
        $crate::get_global_logger().__log_with_message(
            Some(std::borrow::Cow::from($msg)),
            &$data,
            $crate::LogLevel::Info,
        )
    };
    ($data:expr) => {
        $crate::get_global_logger().__log_with_message(None, &$data, $crate::LogLevel::Info)
    };
}

#[macro_export]
macro_rules! warn {
    ($msg:expr, $data:expr) => {
        $crate::get_global_logger().__log_with_message(
            Some(std::borrow::Cow::from($msg)),
            &$data,
            $crate::LogLevel::Warn,
        )
    };
    ($data:expr) => {
        $crate::get_global_logger().__log_with_message(None, &$data, $crate::LogLevel::Warn)
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr, $data:expr) => {
        $crate::get_global_logger().__log_with_message(
            Some(std::borrow::Cow::from($msg)),
            &$data,
            $crate::LogLevel::Error,
        )
    };
    ($data:expr) => {
        $crate::get_global_logger().__log_with_message(None, &$data, $crate::LogLevel::Error)
    };
}
