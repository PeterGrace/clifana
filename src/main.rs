mod cli;
mod cfg_file;
mod query;
mod prometheus;

use std::cmp::Ordering;
use clap::Parser;
use cli::Cli;
use cfg_file::ConfigFile;
use query::execute_query;

#[macro_use]
extern crate log;


fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = ConfigFile::new(cli.config)?;

    let log_level_int = match cli.debug.cmp(&config.log_level) {
        Ordering::Greater => cli.debug,
        _ => config.log_level
    };
    let log_level: String = match log_level_int {
        0 => "WARN".to_string(),
        1 => "INFO".to_string(),
        _ => "DEBUG".to_string()
    };
    std::env::set_var("RUST_LOG", log_level);
    pretty_env_logger::init();

    match &cli.command {
      crate::cli::Commands::Query(args) => {
        execute_query(&config, args)?;
      }
    };
    Ok(())
}
