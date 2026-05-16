#[derive(Debug)]
pub enum LoggerError {
    ContextSerializable(String, serde_json::Error),
    ContextMustBeObject,
}
