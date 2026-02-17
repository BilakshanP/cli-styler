//! Parsing module which has all the primitive and functions used by other modules.

use crate::{
    error::ParsingError,
    style::{Color, Modifier, Style},
};

/// Defines the parsing mode for the parser.
#[derive(Clone, Copy)]
pub(crate) enum ParsingMode {
    /// Markup mode
    Markup,

    /// Command Line mode
    #[cfg(feature = "cli")]
    CommandLine,
}

/// Constant for easier access
#[allow(non_upper_case_globals)]
pub(crate) const Mk: ParsingMode = ParsingMode::Markup;

/// Constant for easier access
#[cfg(feature = "cli")]
#[allow(non_upper_case_globals)]
pub(crate) const Cli: ParsingMode = ParsingMode::CommandLine;

/// Enum defining the output Tokens of the parser
#[derive(Debug)]
#[cfg(feature = "markup")]
pub(crate) enum Token {
    /// End of a Tag '>'
    End,
    /// An empty Tag '<>' or '</>'
    Empty,

    /// Applied format/style
    Fmt(Style),
    /// Text encompassed
    Text(String),
}

/// Inner state used by the parser state machine
#[derive(Debug, Default)]
#[cfg(feature = "markup")]
pub(crate) enum State {
    /// Text encompassed
    #[default]
    Text,

    /// Encountered '<'
    Lt,
    /// Encountered '/'
    BackSlash,
    /// Encountered '</'
    MaybeClose,

    /// Text inside '<>'
    Tag(String),
}

#[cfg(feature = "markup")]
pub(crate) fn tokenize(s: impl AsRef<str>, mode: ParsingMode) -> Result<Vec<Token>, ParsingError> {
    let s = s.as_ref();

    let mut text = String::new();

    let mut state = State::Text;
    let mut tokens = Vec::new();

    let mut chars = s.chars().peekable();

    loop {
        let ch = chars.next();

        state = match state {
            State::Lt => match ch {
                None => Err(ParsingError::Eof(">".to_string()))?,
                Some('/') => State::MaybeClose,
                Some('>') => {
                    tokens.push(Token::Empty);
                    State::default()
                }
                Some(c) => State::Tag(c.to_string()),
            },

            State::Tag(mut tag_content) => match ch {
                None => Err(ParsingError::Eof(format!("Tag name: {tag_content}")))?,
                Some('>') => {
                    tokens.push(Token::Fmt(parse_style(tag_content, mode)?));
                    State::default()
                }
                Some(c) => {
                    if c == ','
                        || c == '#'
                        || c.is_ascii_digit()
                        || c.is_ascii_whitespace()
                        || c.is_ascii_alphanumeric()
                    {
                        tag_content.push(c);
                        State::Tag(tag_content)
                    } else {
                        Err(ParsingError::InvalidTagChar(c))?
                    }
                }
            },

            State::BackSlash => {
                match ch {
                    None => text.push('\\'),
                    Some('<') => text.push('<'),
                    Some(c) => {
                        text.push('\\');
                        text.push(c);
                    }
                }

                State::default()
            }

            State::MaybeClose => match ch {
                None => Err(ParsingError::Eof("</".to_string()))?,
                Some('>') => {
                    tokens.push(Token::End);
                    State::default()
                }
                Some(c) => todo!("Named closing tags are not supported: {}", c),
            },

            State::Text => match ch {
                None => {
                    if !text.is_empty() {
                        tokens.push(Token::Text(std::mem::take(&mut text)));
                    }

                    break;
                }
                Some('\\') => State::BackSlash,
                Some('<') => {
                    if !text.is_empty() {
                        tokens.push(Token::Text(std::mem::take(&mut text)));
                    }

                    State::Lt
                }
                Some(c) => {
                    text.push(c);
                    State::Text
                }
            },
        }
    }

    Ok(tokens)
}

/// Parses the style spec
pub(crate) fn parse_style(s: impl AsRef<str>, mode: ParsingMode) -> Result<Style, ParsingError> {
    let s = s.as_ref();
    let arguments = s.split_whitespace().collect::<Vec<_>>();

    let length = arguments.len();

    if length > 6 {
        Err(ParsingError::TooManyArgs(s.to_string(), length))?
    }

    if length % 2 == 1 {
        Err(ParsingError::MissingParamVal(s.to_string()))?
    }

    let mut style = Style::new();

    for arg in arguments.chunks_exact(2) {
        if let [param, val] = arg {
            style = match *param {
                "f" => style.fg(parse_color(val, mode)?),
                "b" => style.bg(parse_color(val, mode)?),
                "fb" => style.fg(parse_color(val, mode)?).fg_brighten(),
                "bb" => style.bg(parse_color(val, mode)?).bg_brighten(),
                "m" => {
                    style.mdfs.extend(parse_modfiers(val)?);
                    style
                }
                invalid => Err(ParsingError::InvalidParamName(invalid.to_string()))?,
            }
        }
    }

    Ok(style)
}

/// Parse the color spec for the style(s)
fn parse_color(s: &str, mode: ParsingMode) -> Result<Color, ParsingError> {
    let s = s.to_lowercase();

    // 1-letter aliases
    if s.len() == 1 && !s.chars().next().unwrap().is_ascii_digit() {
        return match s.as_str() {
            "r" => Ok(Color::Red),
            "g" => Ok(Color::Green),
            "b" => Ok(Color::Blue),
            "c" => Ok(Color::Cyan),
            "m" => Ok(Color::Magenta),
            "y" => Ok(Color::Yellow),
            "k" => Ok(Color::Black),
            "w" => Ok(Color::White),
            invalid_color => Err(ParsingError::InvalidClrSpec(invalid_color.to_string()))?,
        };
    }

    // Numeric input -> Indexed
    if let Ok(i) = s.parse() {
        return Ok(Color::Indexed(i));
    }

    // Hex input -> RGB
    if let Some(hex) = match mode {
        ParsingMode::Markup => s.strip_prefix('#'),

        #[cfg(feature = "cli")]
        ParsingMode::CommandLine => s.strip_suffix('#'),
    } {
        let expanded = match hex.len() {
            2 => hex.repeat(3),
            3 => hex
                .chars()
                .flat_map(|c| std::iter::repeat_n(c, 2))
                .collect(),
            6 => hex.to_string(),
            l => Err(ParsingError::InvalidHexClr(hex.to_string(), l))?,
        };

        let r = u8::from_str_radix(&expanded[0..2], 16)
            .map_err(|_| ParsingError::InvalidHexComp('r', expanded.clone()))?;
        let g = u8::from_str_radix(&expanded[2..4], 16)
            .map_err(|_| ParsingError::InvalidHexComp('g', expanded.clone()))?;
        let b = u8::from_str_radix(&expanded[4..6], 16)
            .map_err(|_| ParsingError::InvalidHexComp('b', expanded.clone()))?;

        return Ok(Color::RGB(r, g, b));
    }

    // RGB-style input: "255,,128"
    let parts = s
        .split(',')
        .map(|s| s.trim().parse::<u8>().unwrap_or(0))
        .collect::<Vec<_>>();

    if parts.len() == 3 {
        return Ok(Color::RGB(parts[0], parts[1], parts[2]));
    }

    Err(ParsingError::UnknownClrFmt(s.to_string()))
}

/// Parse the modifiers for the style(s)
fn parse_modfiers(input: &str) -> Result<Vec<Modifier>, ParsingError> {
    let mut modifers = Vec::new();

    for ch in input.chars() {
        modifers.push(Modifier::from_char(ch).ok_or(ParsingError::InvalidModifier(ch))?)
    }

    Ok(modifers)
}
