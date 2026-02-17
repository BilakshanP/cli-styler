//! This module contains all error types used throught the crate

use thiserror::Error;

/// Error types for the crate
#[derive(Debug, Error)]
pub enum StylerError {
    /// Symbolises missing --text field
    #[error("Missing required --text argument")]
    MissingText,

    /// Invalid color specifier
    #[error("Invalid color specification: {0}")]
    InvalidColor(String),

    /// Invalid text modfier
    #[error("Invalid modifier: {0}")]
    InvalidModifier(char),

    /// Invalid spec argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Missing value for a parameter
    #[error("Expected value after {0}")]
    MissingValue(String),

    /// Invalid RGB format
    #[error("Invalid RGB format: {0}")]
    InvalidRgbFormat(String),

    /// Invalid HEX format
    #[error("Invalid hex color: {0}")]
    InvalidHexColor(String),

    /// Signifies errors ecountered by the [`crate::parser`] module
    #[error("Encountered an error while parsing: {0}")]
    ParsingError(ParsingError),

    /// Signifies errors encountered by the [`crate::style::BatchStyler`] type
    #[error("Encountered an error during batchoperation at index ({0}): {1}")]
    BatchError(usize, Box<StylerError>),
}

/// Error type used in [`crate::parser`]
#[derive(Debug, Error)]
pub enum ParsingError {
    /// Signifies an invalid EOF during tag parsing
    #[error("End of File after: {0}")]
    Eof(String),

    /// Signifies an invalid charcater inside the tag/style
    #[error("Invalid character in tag name: {0}")]
    InvalidTagChar(char),

    /// Occurs when too many arguments have been passed
    #[error("Too many arguments (<=6): {0}:{1}")]
    TooManyArgs(String, usize),

    /// Occurs when a paramater is missing its value
    #[error("Missing parameter value: {0}")]
    MissingParamVal(String),

    /// Invalid parameter name
    #[error("Invalid paramater name: {0}")]
    InvalidParamName(String),

    /// Invalid color spec
    #[error("Invalid color alias: {0}")]
    InvalidClrSpec(String),

    /// Invalid hex color spec
    #[error("Invalid hex color length: {0}:{1}")]
    InvalidHexClr(String, usize),

    /// Invalid compoenent in hex
    #[error("Invalid Hex ({0}) component: {1}")]
    InvalidHexComp(char, String),

    /// Invalid color component
    #[error("Unknown color format: {0}")]
    UnknownClrFmt(String),

    /// Extra/unnecessary closing tag "</>"
    #[error("Unexpected closing tag")]
    UnexpectedClosingTag,

    /// Missing closing tag "</>"
    #[error("Unclosed Tags")]
    UnclosedTags,

    /// Invalid text modfier
    #[error("Invalid modifier: {0}")]
    InvalidModifier(char),
}
