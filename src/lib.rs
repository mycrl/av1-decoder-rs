mod nal;
mod utils;

#[derive(Debug)]
pub enum H264DecodeErrorKind {
    UnSupports,
    UnknownData,
}

impl std::fmt::Display for H264DecodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub struct H264DecodeError {
    pub kind: H264DecodeErrorKind,
    pub help: Option<&'static str>,
}

impl H264DecodeError {
    fn default_from(kind: H264DecodeErrorKind) -> Self {
        Self { kind, help: None }
    }
}

impl std::error::Error for H264DecodeError {}

impl std::fmt::Display for H264DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.help {
            Some(help) => write!(f, "{} - {}", self.kind, help),
            None => write!(f, "{}", self.kind),
        }
    }
}
