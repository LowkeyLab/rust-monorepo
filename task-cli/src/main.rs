use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;
use task_cli::TaskRepository;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, ValueEnum)]
enum StatusArg {
    Todo,
    InProgress,
    Done,
}

impl From<StatusArg> for task_cli::Status {
    fn from(status_arg: StatusArg) -> Self {
        match status_arg {
            StatusArg::Todo => task_cli::Status::Todo,
            StatusArg::InProgress => task_cli::Status::InProgress,
            StatusArg::Done => task_cli::Status::Done,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Add {
        description: String,
    },
    Update {
        id: u32,
        description: String,
    },
    #[command(name = "mark-in-progress")]
    MarkInProgress {
        id: u32,
    },
    #[command(name = "mark-done")]
    MarkDone {
        id: u32,
    },
    Delete {
        id: u32,
    },
    List {
        // Optional positional argument for status
        status: Option<StatusArg>,
    },
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
            let id = tasks.add_task(description);
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
        Commands::MarkInProgress { id } => {
            let mut file = open_file_and_truncate(path);
            tasks
                .mark_in_progress(id)
                .expect("cannot mark task as in progress");
            tasks.save_as_json(&mut file);
            println!("Task with ID {} marked as in progress", id);
        }
        Commands::MarkDone { id } => {
            let mut file = open_file_and_truncate(path);
            tasks.mark_done(id).expect("cannot mark task as done");
            tasks.save_as_json(&mut file);
            println!("Task with ID {} marked as done", id);
        }
        Commands::Delete { id } => {
            tasks.delete_task(id);
            let mut file = open_file_and_truncate(path);
            tasks.save_as_json(&mut file);
            println!("Task with ID {} deleted", id);
        }
        Commands::List { status } => {
            let filtered_status = status.map(task_cli::Status::from);

            let Some(status) = filtered_status else {
                // Show all tasks
                println!("{}", tasks);
                return Ok(());
            };

            // Filter tasks by status
            println!("Listing tasks with status: {:?}", status);
            for task in tasks.get_tasks_with_status(status) {
                println!("{}", task);
            }
        }
    };

    Ok(())
}
