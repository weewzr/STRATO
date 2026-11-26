//! Command line of strato-webtools; see [`strato_webtools::cli`].

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = strato_webtools::cli::main_with(&args);
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
