use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "a")]
    Add(Add),
    #[command(alias = "rm")]
    Remove(Remove),
    #[command(alias = "ls")]
    List(List),
    #[command(alias = "s")]
    Search(Search),
    #[command(alias = "at")]
    Autotag(Autotag),
}

#[derive(Parser)]
pub struct Add {
    pub tag: String,
    #[clap(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    #[clap(short, long)]
    pub recursive: bool,
    #[clap(long)] // FIX: Think of a good short bind that doesn't overlap help
    pub hidden: bool,
}

#[derive(Parser)]
pub struct Remove {
    pub tag: String,
    #[clap(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    #[clap(short, long)]
    pub recursive: bool,
    #[clap(long)] // FIX: Think of a good short bind that doesn't overlap help
    pub hidden: bool,
}

#[derive(Parser)]
pub struct List {
    pub tag: String,
    #[clap(long)]
    pub dirs: bool,
    #[clap(long)]
    pub files: bool,
}

#[derive(Parser)]
pub struct Search {
    #[clap(required = true, num_args = 1..)]
    pub tags: Vec<String>,
    #[clap(long)]
    pub any: bool,
    #[clap(long)]
    pub dirs: bool,
    #[clap(long)]
    pub files: bool,
    #[clap(short, long, num_args = 1..)]
    pub exclude: Vec<String>,
}

#[derive(Parser)]
pub struct Autotag {
    #[clap(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
    #[clap(short, long)]
    pub recursive: bool,
    #[clap(long)] // FIX: Think of a good short bind that doesn't overlap help
    pub hidden: bool,
}
