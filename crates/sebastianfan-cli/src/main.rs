mod cli;
mod commands;
mod error;
mod paths;
mod render;
mod store;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    match commands::run(&cli) {
        Ok(output) => {
            render::print_block(&output);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", error.report());
            ExitCode::FAILURE
        }
    }
}
