#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    UnexpectedToken(String),
    InvalidSyntax(String),
    UnexpectedEndOfInput,
    FileNotFound(String),
    IoError(String),
    XmlError(String),
    EncodingError(String),
}

pub trait Parser {
    type Output;
    fn parse(&self, path: &std::path::Path) -> Result<Self::Output, ParserError>;
}