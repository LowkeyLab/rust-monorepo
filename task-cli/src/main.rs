use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Add,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Add => println!("Add command"),
    }
}
