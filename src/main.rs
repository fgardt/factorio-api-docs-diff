use std::process::ExitCode;

#[cfg(feature = "diff")]
use fapi_diff::CLI;

#[cfg(feature = "diff")]
fn main() -> ExitCode {
    let cli = CLI.with_borrow(std::clone::Clone::clone);

    if let Err(e) = cli.stage.compare(&cli.source, &cli.target) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}

#[cfg(not(feature = "diff"))]
const fn main() -> ExitCode {
    compile_error!("This binary was not compiled with the 'diff' feature enabled");
    ExitCode::FAILURE
}
