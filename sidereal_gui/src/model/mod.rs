use thiserror::Error;

pub(crate) mod tracking_manager;

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
    #[error("gRPC Error: {0}")]
    GrpcError(String),
}
