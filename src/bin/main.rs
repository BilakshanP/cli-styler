#[cfg(feature = "cli")]
fn main() {
    cli_styler::cli::wrapped_run();
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!(
        "Error: 'cli' feature not enabled, pass the \"--features=cli\" flag during compilation."
    );
}
