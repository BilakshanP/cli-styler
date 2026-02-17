use crate::{
    error::ParsingError,
    parser::{Mk, ParsingMode, Token, tokenize},
    style::{CompiledStyle, Stylable, Style},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Markup AST
#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::missing_docs_in_private_items)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
enum AstTk {
    Text(String),
    Tree(Markup),
}

/// Markup Tree parent struct
#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::missing_docs_in_private_items)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Markup {
    /// Style portion
    st: CompiledStyle,
    children: Vec<AstTk>,
}

impl Markup {
    /// Parse markup text and return a new [`Markup`] struct.
    pub fn new(s: impl AsRef<str>) -> Result<Self, ParsingError> {
        Self::markup_parser(s, Mk)
    }

    /// Parse markup text with CLI mode and return a new [`Markup`] struct.
    #[cfg(feature = "cli")]
    pub(crate) fn new_cli(s: impl AsRef<str>) -> Result<Self, ParsingError> {
        use crate::parser::Cli;

        Self::markup_parser(s, Cli)
    }

    /// Collect and merge the input into the final output
    pub fn render(self) -> String {
        let mut output = String::new();

        for tk in self.children {
            let fragment = match tk {
                AstTk::Text(text) => self.st.style(text),
                AstTk::Tree(ast) => ast.render(),
            };

            output.push_str(&fragment);
        }

        output
    }

    /// Parses markup spec
    fn markup_parser(s: impl AsRef<str>, mode: ParsingMode) -> Result<Self, ParsingError> {
        let tokens = tokenize(s, mode)?;

        let mut stack = Vec::new();
        let mut current_nodes = Vec::new();

        for token in tokens {
            match token {
                Token::Text(text) => current_nodes.push(AstTk::Text(text)),

                Token::Fmt(style) => {
                    stack.push((style, current_nodes));
                    current_nodes = Vec::new();
                }

                Token::Empty => {
                    stack.push((Style::new(), current_nodes));
                    current_nodes = Vec::new();
                }

                Token::End => {
                    let (style, mut parent_nodes) =
                        stack.pop().ok_or(ParsingError::UnexpectedClosingTag)?;
                    let ast = Markup {
                        st: style.compile(),
                        children: current_nodes,
                    };

                    parent_nodes.push(AstTk::Tree(ast));
                    current_nodes = parent_nodes;
                }
            }
        }

        if !stack.is_empty() {
            Err(ParsingError::UnclosedTags)?
        }

        Ok(Self {
            st: Style::new().compile(),
            children: current_nodes,
        })
    }
}
