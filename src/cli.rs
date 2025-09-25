use std::env;

use crate::{
    error::StylerError,
    parser::{Cli, parse_style},
    style::Stylable,
};

#[cfg(feature = "markup")]
use crate::markup::Markup;

/// CLI Handler
pub fn run() -> Result<(), StylerError> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        println!("{}", include_str!("spec/concise.txt"));
        return Ok(());
    }

    if args.len() == 1 && args[0] == "--help" {
        println!("{}", include_str!("spec/verbose.txt"));
        return Ok(());
    }

    if args.len() == 2 && args[0] == "--markup" {
        #[cfg(feature = "markup")]
        {
            let mk = Markup::new_cli(&args[1]).map_err(StylerError::ParsingError)?;
            println!("{}", mk.render());
        }

        #[cfg(not(feature = "markup"))]
        eprintln!(
            "Error: 'markup' feature not enabled, pass the \"--all-features\" flag during compilation."
        );

        return Ok(());
    }

    let text = &args[0];
    let style = parse_style(args[1..].join(" "), Cli).map_err(StylerError::ParsingError)?;

    println!("{}", style.style(text));

    Ok(())
}

/// Silenced CLI Handler
pub fn wrapped_run() {
    match run() {
        Ok(()) => {}
        Err(err) => eprintln!("{err}"),
    }
}
