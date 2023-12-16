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
pub enum ResolvePathError {
    EnvVarError(std::env::VarError),
    PathParseError,
}

impl From<std::env::VarError> for ResolvePathError {
    fn from(err: std::env::VarError) -> Self {
        ResolvePathError::EnvVarError(err)
    }
}
