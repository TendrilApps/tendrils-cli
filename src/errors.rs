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
pub enum PushPullError {
    Duplicate,
    IoError(std::io::Error),
    ResolveTendrilError(ResolveTendrilError),
    Recursion,
    Skipped,
    TypeMismatch,
    Unsupported,
}

impl From<ResolveTendrilError> for PushPullError {
    fn from(err: ResolveTendrilError) -> Self {
        PushPullError::ResolveTendrilError(err)
    }
}

impl From<std::io::Error> for PushPullError {
    fn from(err: std::io::Error) -> Self {
        PushPullError::IoError(err)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidTendrilError {
    InvalidApp,
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
