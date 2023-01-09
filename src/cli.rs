use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    Query(Query)
}

#[derive(Args, Debug)]
pub struct Query {
    pub query: Option<String>,
    #[arg(short='s',long="server")]
    pub server: Option<String>,
    #[arg(short='e',long="eval")]
    pub eval: Vec<String>

}