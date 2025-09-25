//! A library for styling terminal text with ANSI escape sequences.
//!
//! Provides functionality to create styled text with foreground/background colors,
//! text modifiers (bold, italic, etc.), and batch styling operations.

use crate::{
    error::StylerError,
    parser::{Mk, parse_style},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Constants for ANSI escape sequences
const ESC: char = '\x1b';

/// ANSI reset sequence that clears all formatting
pub(crate) const RESET: &str = "\x1b[0m";

/// Generates a Control Sequence Introducer (CSI) with the given formats
///
/// # Arguments
/// * `formats` - The format codes to include in the CSI
pub(crate) fn csi(formats: &str) -> String {
    format!("{ESC}[{formats}m")
}

/// Wraps text with ANSI escape sequences and resets at the end
///
/// # Arguments
/// * `text` - The text to wrap
/// * `formats` - The format codes to apply
fn wrap(text: &str, formats: &str) -> String {
    if text.is_empty() || formats.is_empty() {
        return text.to_string();
    }

    format!("{}{text}{RESET}", csi(formats))
}

/// Provides `.style()` method for [`Style`] and [`CompiledStyle`]
pub trait Stylable {
    /// Apply this style to the provided text
    fn style(&self, text: impl AsRef<str>) -> String;
}

impl Stylable for Style {
    fn style(&self, text: impl AsRef<str>) -> String {
        wrap(text.as_ref(), &self.collect())
    }
}

impl Stylable for CompiledStyle {
    fn style(&self, text: impl AsRef<str>) -> String {
        wrap(text.as_ref(), &self.0)
    }
}

/// Trait for easily applying compiled styles to strings
pub trait Stylize {
    /// Apply a style to this string
    fn apply(&self, style: &impl Stylable) -> String;
}

impl Stylize for str {
    fn apply(&self, style: &impl Stylable) -> String {
        style.style(self)
    }
}

/// Color Types with their respective ANSI code offsets
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) enum ClrType {
    #[default]
    Fg = 0,
    Bg = 10,       // +10 to convert to background
    FgBright = 60, // +60 to make the color intense
    BgBright = 70, // +70 = 10 + 60 (convert to intense background)
}

impl ClrType {
    /// Get the Control Sequence Introducer code depending on the type
    ///
    /// Returns:
    /// - `38` for Foreground modification
    /// - `48` for Background modification
    fn get_csi(self) -> u8 {
        match self {
            Self::Fg | Self::FgBright => 38,
            Self::Bg | Self::BgBright => 48,
        }
    }
}

/// Colors mapped to their respective ANSI codes
#[repr(u8)]
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Color {
    Black = 30,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    #[default]
    White,

    /// Indexed ANSI color (256-color mode)
    Indexed(u8),

    /// True RGB color
    #[allow(clippy::upper_case_acronyms)]
    RGB(u8, u8, u8),
}

use Color::*;

impl Color {
    /// Check whether this is a basic ANSI color
    pub fn is_color(self) -> bool {
        matches!(
            self,
            Black | Red | Green | Yellow | Blue | Magenta | Cyan | White
        )
    }

    /// Check whether this is an RGB color
    pub fn is_rgb(self) -> bool {
        matches!(self, RGB(_, _, _))
    }

    /// Check whether this is an indexed ANSI color
    pub fn is_indexed(self) -> bool {
        matches!(self, Indexed(_))
    }

    /// Convert a char to [`Color`]
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'r' => Some(Red),
            'g' => Some(Green),
            'b' => Some(Blue),
            'c' => Some(Cyan),
            'm' => Some(Magenta),
            'y' => Some(Yellow),
            'k' => Some(Black),
            'w' => Some(White),
            _ => None,
        }
    }

    /// Get the corresponding ANSI code for basic colors
    fn to_num(self) -> u8 {
        match self {
            Black => 30,
            Red => 31,
            Green => 32,
            Yellow => 33,
            Blue => 34,
            Magenta => 35,
            Cyan => 36,
            White => 37,
            _ => 0,
        }
    }

    /// Format the color for the given color type into an ANSI escaped string
    pub(crate) fn format(self, ct: ClrType) -> String {
        match self {
            Indexed(i) => format!("{};5;{}", ct.get_csi(), i),
            RGB(r, g, b) => format!("{};2;{};{};{}", ct.get_csi(), r, g, b),
            color => format!("{}", color.to_num() + ct as u8),
        }
    }
}

/// ANSI text modifiers
///
/// This implementation only includes widely compatible codes.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Modifier {
    Reset = 0,     // -    1
    Bold,          // b    2
    Dim,           // d    4
    Italic,        // i    8
    Underline,     // u   16
    Blink,         // k   32
    Invert = 7,    // v   64
    Hide,          // h  128
    Strike,        // s  256
    DoubleUL = 21, // l  512
    Overline = 53, // o 1024
}

use Modifier::*;

impl Modifier {
    /// Convert a char to [`Modifier`]
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'b' => Some(Bold),
            'd' => Some(Dim),
            'i' => Some(Italic),
            'u' => Some(Underline),
            'k' => Some(Blink),
            'v' => Some(Invert),
            'h' => Some(Hide),
            's' => Some(Strike),
            'l' => Some(DoubleUL),
            'o' => Some(Overline),
            _ => None,
        }
    }
}

/// The core styling builder for creating styled text.
///
/// # Example
/// ```rust
/// use cli_styler::prelude::*;
///
/// let style_1 = Style::new()
///     .fg_rgb(0, 255, 255)
///     .bg_index(198)
///     .italic()
///     .overline()
///     .underline();
///
/// let style_2 = Style::new_from_cli_spec("fb r b #ABC m ilobu").unwrap();
///
/// assert_eq!(style_1.style("Hello"), "\u{1b}[38;2;0;255;255;48;5;198;3;53;4mHello\u{1b}[0m");
/// assert_eq!(style_2.style("Hello"), "\u{1b}[91;48;2;170;187;204;3;21;53;1;4mHello\u{1b}[0m");
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Style {
    /// Foreground color & type
    pub(crate) fg: Option<(Color, ClrType)>,
    /// Background color & type
    pub(crate) bg: Option<(Color, ClrType)>,
    /// Modifiers for the text
    pub(crate) mdfs: Vec<Modifier>,
}

impl Style {
    /// Creates a new, empty [`Style`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new [`Style`] from the provided spec
    pub fn new_from_cli_spec(spec: impl AsRef<str>) -> Result<Self, StylerError> {
        match parse_style(spec, Mk) {
            Ok(style) => Ok(style),
            Err(err) => panic!("Error encountered during cli parsing: {err}"),
        }
    }

    /// Set the foreground color (supports indexed and RGB colors)
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = match self.fg {
            Some((_, ct)) => Some((color, ct)),
            None => Some((color, ClrType::Fg)),
        };
        self
    }

    /// Set the background color (supports indexed and RGB colors)
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = match self.bg {
            Some((_, ct)) => Some((color, ct)),
            None => Some((color, ClrType::Bg)),
        };
        self
    }

    /// Set the foreground color as RGB
    pub fn fg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fg = match self.fg {
            Some((_, ct)) => Some((Color::RGB(r, g, b), ct)),
            None => Some((Color::RGB(r, g, b), ClrType::Fg)),
        };
        self
    }

    /// Set the background color as RGB
    pub fn bg_rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.bg = match self.bg {
            Some((_, ct)) => Some((Color::RGB(r, g, b), ct)),
            None => Some((Color::RGB(r, g, b), ClrType::Bg)),
        };
        self
    }

    /// Set the foreground color as an indexed color
    pub fn fg_index(mut self, i: u8) -> Self {
        self.fg = Some((Color::Indexed(i), ClrType::Fg));
        self
    }

    /// Set the background color as an indexed color
    pub fn bg_index(mut self, i: u8) -> Self {
        self.bg = Some((Color::Indexed(i), ClrType::Bg));
        self
    }

    /// Brighten the foreground color (for ANSI colors only)
    pub fn fg_brighten(mut self) -> Self {
        self.fg = match self.fg {
            Some((clr, _)) => Some((clr, ClrType::FgBright)),
            None => Some((Color::default(), ClrType::FgBright)),
        };
        self
    }

    /// Brighten the background color (for ANSI colors only)
    pub fn bg_brighten(mut self) -> Self {
        self.bg = match self.bg {
            Some((clr, _)) => Some((clr, ClrType::BgBright)),
            None => Some((Color::default(), ClrType::BgBright)),
        };
        self
    }

    /// Apply bold styling
    pub fn bold(self) -> Self {
        self.insert_modifier(Modifier::Bold)
    }

    /// Apply dim styling
    pub fn dim(self) -> Self {
        self.insert_modifier(Modifier::Dim)
    }

    /// Apply italic styling
    pub fn italic(self) -> Self {
        self.insert_modifier(Modifier::Italic)
    }

    /// Apply underline styling
    pub fn underline(self) -> Self {
        self.insert_modifier(Modifier::Underline)
    }

    /// Apply blink styling
    pub fn blink(self) -> Self {
        self.insert_modifier(Modifier::Blink)
    }

    /// Swap foreground and background colors
    pub fn invert(self) -> Self {
        self.insert_modifier(Modifier::Invert)
    }

    /// Hide text (revealed when selected)
    pub fn hide(self) -> Self {
        self.insert_modifier(Modifier::Hide)
    }

    /// Apply strikethrough styling
    pub fn strike(self) -> Self {
        self.insert_modifier(Modifier::Strike)
    }

    /// Apply double underline styling
    pub fn double_ul(self) -> Self {
        self.insert_modifier(Modifier::DoubleUL)
    }

    /// Apply overline styling
    pub fn overline(self) -> Self {
        self.insert_modifier(Modifier::Overline)
    }

    /// Compile this style into a `CompiledStyle` for efficient reuse
    pub fn compile(&self) -> CompiledStyle {
        CompiledStyle(self.collect())
    }

    /// Reset all styling
    pub fn reset(self) -> Self {
        self.insert_modifier(Modifier::Reset)
    }

    /// Checks whether a `Style` has any effect on text or not
    pub fn is_empty(&self) -> bool {
        if self.fg.is_some() {
            return false;
        }

        if self.bg.is_some() {
            return false;
        }

        if !self.mdfs.is_empty() {
            return false;
        }

        true
    }

    /// Internal helper to add a modifier
    pub(crate) fn insert_modifier(mut self, mdf: Modifier) -> Self {
        self.mdfs.push(mdf);
        self
    }

    /// Collect all modifiers into an ANSI escape sequence string
    pub(crate) fn collect(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut modifiers = Vec::new();

        if let Some((fgclr, ct)) = self.fg {
            modifiers.push(fgclr.format(ct));
        }

        if let Some((bgclr, ct)) = self.bg {
            modifiers.push(bgclr.format(ct));
        }

        modifiers.extend(self.mdfs.iter().map(|&mdf| (mdf as u8).to_string()));

        modifiers.join(";")
    }
}

/// A pre-compiled style for efficient repeated use.
///
/// Use this for global styles that won't change after initialization.
///
/// # Example
/// ```rust
/// use std::sync::LazyLock;
/// use cli_styler::style::{Color, CompiledStyle, Style, Stylable};
///
/// static WARNING: LazyLock<CompiledStyle> = LazyLock::new(|| {
///     Style::new()
///         .fg(Color::Red)
///         .fg_brighten()
///         .bg(Color::Yellow)
///         .bg_brighten()
///         .compile()
/// });
///
/// static ERROR: LazyLock<CompiledStyle> = LazyLock::new(|| {
///     CompiledStyle::new_from_cli_spec("f #fff b 255,,").unwrap()
/// });
///
/// assert_eq!(WARNING.style("Warning!!"), "\u{1b}[91;103mWarning!!\u{1b}[0m");
/// assert_eq!(ERROR.style("Error!!"), "\u{1b}[38;2;255;255;255;48;2;255;0;0mError!!\u{1b}[0m");
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CompiledStyle(String);

impl CompiledStyle {
    /// Constructs a new [`CompiledStyle`] from a [`Style`]
    pub fn new(style: Style) -> Self {
        style.compile()
    }

    /// Constructs a new [`CompiledStyle`] from the provided spec
    pub fn new_from_cli_spec(spec: impl AsRef<str>) -> Result<Self, StylerError> {
        match parse_style(spec, Mk) {
            Ok(style) => Ok(style.compile()),
            Err(err) => panic!("Error encountered during cli parsing: {err}"),
        }
    }
}

/// Bundles the `text` and the `spec` (style) together
#[derive(Default)]
pub struct Part {
    /// text to which the provided spec would be applied to
    pub text: String,
    /// spec which would be applied to the provided text
    pub spec: String,
}

#[cfg(feature = "cli")]
impl Part {
    /// Parses CLI spec
    fn style(&self) -> Result<String, StylerError> {
        Style::new_from_cli_spec(&self.spec).map(|st| st.style(&self.text))
    }
}

/// A builder for creating complex styled text with multiple segments
#[derive(Default)]
#[allow(clippy::missing_docs_in_private_items)]
#[cfg(feature = "cli")]
pub struct BatchStyler {
    parts: Vec<Part>,
}

#[cfg(feature = "cli")]
impl BatchStyler {
    /// Create a new BatchStyler
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert `text` and `specs` to the current instance
    pub fn push(mut self, text: impl ToString, spec: impl ToString) -> Self {
        self.parts.push(Part {
            text: text.to_string(),
            spec: spec.to_string(),
        });

        self
    }

    /// Collects all parts of the Styler and returns a string
    pub fn build(self) -> Result<String, StylerError> {
        self.collect().map(|v| v.concat())
    }

    /// Collects all parts of the Styler and returns a string with the provided separator
    pub fn build_with_separator(self, sep: impl AsRef<str>) -> Result<String, StylerError> {
        self.collect().map(|v| v.join(sep.as_ref()))
    }

    /// Collect and merge the input into the final output
    fn collect(self) -> Result<Vec<String>, StylerError> {
        self.parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| {
                part.style()
                    .map_err(|err| StylerError::BatchError(i, Box::new(err)))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
