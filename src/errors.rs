#[derive(Debug)]
pub enum GetTendrilsError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
}

impl From<std::io::Error> for GetTendrilsError {
    fn from(err: std::io::Error) -> Self {
        GetTendrilsError::IoError(err)
    }
}

impl From<serde_json::Error> for GetTendrilsError {
    fn from(err: serde_json::Error) -> Self {
        GetTendrilsError::ParseError(err)
    }
}

#[derive(Debug)]
pub enum TendrilActionError {
    Duplicate,
    IoError(std::io::Error),
    /// Occurs when a tendril action does not match its
    /// mode (such as trying to pull a link tendril)
    ModeMismatch,
    ResolveTendrilError(ResolveTendrilError),
    Recursion,
    /// Occurs when a command is executed as a dry-run
    Skipped,
    TypeMismatch,
}

impl From<ResolveTendrilError> for TendrilActionError {
    fn from(err: ResolveTendrilError) -> Self {
        TendrilActionError::ResolveTendrilError(err)
    }
}

impl From<std::io::Error> for TendrilActionError {
    fn from(err: std::io::Error) -> Self {
        TendrilActionError::IoError(err)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidTendrilError {
    InvalidGroup,
    InvalidName,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveTendrilError {
    EnvVarError(std::env::VarError),
    InvalidTendril(InvalidTendrilError),
    PathParseError,
}

impl From<std::env::VarError> for ResolveTendrilError {
    fn from(err: std::env::VarError) -> Self {
        ResolveTendrilError::EnvVarError(err)
    }
}

impl From<InvalidTendrilError> for ResolveTendrilError {
    fn from(err: InvalidTendrilError) -> Self {
        ResolveTendrilError::InvalidTendril(err)
    }
}
