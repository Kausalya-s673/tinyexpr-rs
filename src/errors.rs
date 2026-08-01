use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token")]
    UnexpectedToken,

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("unknown identifier: {0}")]
    UnknownIdentifier(String),

    #[error("Unexpected character '{0}'")]
    UnexpectedCharacter(char),

    #[error("invalid number")]
    InvalidNumber,
}
