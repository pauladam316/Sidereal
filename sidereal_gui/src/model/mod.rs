use thiserror::Error;

pub(crate) mod planetarium_handler;

pub type SiderealResult<T> = Result<T, SiderealError>;

#[derive(Error, Debug, Clone)]

pub enum SiderealError {
    #[error("ServerError: {reason}")]
    ServerError { reason: String },
    #[error("ConfigError: {reason}")]
    ConfigError { reason: String },
    #[error("ParseError: {0}")]
    ParseError(String),
}
