use clap::{Parser, Subcommand};
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use task_cli::TaskRepository;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Add { description: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    const TASK_FILE: &str = "tasks.json";

    let path = Path::new(TASK_FILE);

    let mut tasks = if !path.exists() {
        TaskRepository::default()
    } else {
        let contents = fs::read_to_string(path).expect("cannot read file that currently exists");
        TaskRepository::new_from_json(&contents)
    };

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(TASK_FILE)
        .expect("cannot open file");

    match args.command {
        Commands::Add { description } => {
            let id = tasks.add(description);
            tasks.save_as_json(&mut file);
            println!("Task added with ID {}", id);
        }
    };

    Ok(())
}
