mod api;
mod auth;
mod cli;
mod config;
mod env;
mod files;
mod models;
mod update;

/// Every failure is a sentence for the user, printed after `ormedia: `.
type Result<T> = std::result::Result<T, String>;

fn main() -> std::process::ExitCode {
    cli::main()
}
