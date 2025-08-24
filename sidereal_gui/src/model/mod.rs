use thiserror::Error;

pub(crate) mod indi_server_handler;
pub(crate) mod planetarium_handler;

pub type SiderealResult<T> = Result<T, SiderealError>;

#[derive(Error, Debug, Clone)]

pub enum SiderealError {
    #[error("ConfigError: {0}")]
    ConfigError(String),
    #[error("ParseError: {0}")]
    ParseError(String),
    #[error("ServerError: {0}")]
    ServerError(String),
    #[error("PlanetariumError: {0}")]
    PlanetariumError(String),
    #[error("ServerConnectionError: {0}")]
    ServerConnectionError(String),
    #[error("FormatError: {0}")]
    FormatError(String),
}
