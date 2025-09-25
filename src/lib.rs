//! Sigma

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

mod parser;

pub mod error;
pub mod style;

/// Module for CLI support
#[cfg(feature = "cli")]
pub mod cli;

/// Module for Markup support
#[cfg(feature = "markup")]
pub mod markup;

/// Basic imports
pub mod prelude {
    pub use super::style::{Color, Stylable, Style};
}

/// Unit Tests
#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[cfg(test)]
    mod spec_test {
        use super::*;

        #[test]
        fn empty_initialization() {
            assert_eq!(Style::new(), Style::new_from_cli_spec("").unwrap());
        }

        #[test]
        fn raw_colors() {
            /* Foreground */

            assert_eq!(
                Style::new().fg(Color::Red),
                Style::new_from_cli_spec("f r").unwrap()
            );

            assert_eq!(
                Style::new().fg(Color::Red).fg_brighten(),
                Style::new_from_cli_spec("fb r").unwrap()
            );

            assert_eq!(
                Style::new().fg_brighten().fg(Color::Red),
                Style::new_from_cli_spec("fb r").unwrap()
            );

            /* Background */

            assert_eq!(
                Style::new().bg(Color::Red),
                Style::new_from_cli_spec("b r").unwrap()
            );

            assert_eq!(
                Style::new().bg(Color::Red).bg_brighten(),
                Style::new_from_cli_spec("bb r").unwrap()
            );

            assert_eq!(
                Style::new().bg_brighten().bg(Color::Red),
                Style::new_from_cli_spec("bb r").unwrap()
            );
        }

        #[test]
        fn indexed() {
            assert_eq!(
                Style::new().fg_index(99),
                Style::new_from_cli_spec("f 99").unwrap()
            );

            assert_eq!(
                Style::new().bg_index(99),
                Style::new_from_cli_spec("b 99").unwrap()
            );
        }

        #[test]
        fn rgb_hex() {
            /* Foreground */

            assert_eq!(
                Style::new().fg_rgb(0, 128, 255),
                Style::new_from_cli_spec("f ,128,255").unwrap()
            );

            assert_eq!(
                Style::new().fg_rgb(0, 0x80, 0xFF),
                Style::new_from_cli_spec("f #0080FF").unwrap()
            );

            assert_eq!(
                Style::new_from_cli_spec("f #ABC").unwrap(),
                Style::new_from_cli_spec("f #AABBCC").unwrap()
            );

            assert_eq!(
                Style::new().fg_rgb(0xAA, 0xBB, 0xCC),
                Style::new_from_cli_spec("f #AABBCC").unwrap()
            );

            assert_eq!(
                Style::new().fg_rgb(0xAA, 0xBB, 0xCC),
                Style::new_from_cli_spec("f #ABC").unwrap()
            );

            assert_eq!(
                Style::new().fg_rgb(0xA, 0xB, 0xC),
                Style::new_from_cli_spec("f #0A0B0C").unwrap()
            );

            /* Background */

            assert_eq!(
                Style::new().bg_rgb(0, 128, 255),
                Style::new_from_cli_spec("b ,128,255").unwrap()
            );

            assert_eq!(
                Style::new().bg_rgb(0, 0x80, 0xFF),
                Style::new_from_cli_spec("b #0080FF").unwrap()
            );

            assert_eq!(
                Style::new_from_cli_spec("b #ABC").unwrap(),
                Style::new_from_cli_spec("b #AABBCC").unwrap()
            );

            assert_eq!(
                Style::new().bg_rgb(0xAA, 0xBB, 0xCC),
                Style::new_from_cli_spec("b #AABBCC").unwrap()
            );

            assert_eq!(
                Style::new().bg_rgb(0xAA, 0xBB, 0xCC),
                Style::new_from_cli_spec("b #ABC").unwrap()
            );

            assert_eq!(
                Style::new().bg_rgb(0xA, 0xB, 0xC),
                Style::new_from_cli_spec("b #0A0B0C").unwrap()
            );
        }

        #[test]
        fn modifiers() {
            assert_eq!(
                Style::new().bold(),
                Style::new_from_cli_spec("m b").unwrap()
            );

            assert_eq!(
                Style::new()
                    .italic()
                    .double_ul()
                    .overline()
                    .bold()
                    .underline(),
                Style::new_from_cli_spec("m ilobu").unwrap()
            );
        }
    }
}
