mod tagstore;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "a")]
    Add { tag: String, path: PathBuf },
    #[command(alias = "rm")]
    Remove { tag: String, path: PathBuf },
    #[command(alias = "ls")]
    List { tag: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { tag, path } => {
            println!("Would add tag '{}' to {}", tag, path.display());
        }
        Commands::Remove { tag, path } => {
            println!("Would remove tag '{}' from {}", tag, path.display());
        }
        Commands::List { tag } => {
            println!("Would list all files tagged with '{}'", tag);
        }
    }
}
