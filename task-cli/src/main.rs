use clap::{Parser, Subcommand};
use std::fs;
use std::fs::{File, OpenOptions};
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
    Update { id: u32, description: String },
    List,
}

fn open_file_and_truncate(path: &Path) -> File {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("cannot open file");
    file
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    const TASK_FILE: &str = "tasks.json";

    let path = Path::new(TASK_FILE);

    let mut tasks = if !path.exists() {
        TaskRepository::new()
    } else {
        let contents = fs::read_to_string(path).expect("cannot read file that currently exists");
        if contents.is_empty() {
            TaskRepository::new()
        } else {
            TaskRepository::new_from_json(&contents)
        }
    };

    match args.command {
        Commands::Add { description } => {
            let mut file = open_file_and_truncate(path);
            let id = tasks.add(description);
            tasks.save_as_json(&mut file);
            println!("Task added with ID {}", id);
        }
        Commands::Update { id, description } => {
            let mut file = open_file_and_truncate(path);
            tasks
                .update_task(id, description)
                .expect("cannot update task");
            tasks.save_as_json(&mut file);
        }
        Commands::List => {
            println!("{}", tasks);
        }
    };

    Ok(())
}
