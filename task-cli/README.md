# Task CLI

## Overview

Task CLI is a simple command-line task management application built in Rust. It
allows you to create, update, and manage your tasks directly from the terminal.

This project was done as part
of [roadmap.sh](https://roadmap.sh/projects/task-tracker)

## Features

- Add new tasks with descriptions
- Update existing task descriptions
- Change task status (Todo, In Progress, Done)
- Delete tasks
- List all tasks or filter by status
- Persistent storage using JSON

## Installation

### Requirements

- Rust 1.85.0 or newer
- Cargo package manager

### Building from source

1. Clone this repository:
    ```
    git clone <repository-url>
       cd task-cli
    ```

2. Build the application:
    ```
    cargo build --release
    ```

3. The executable will be available at `target/release/task-cli`

## Usage

Task CLI provides several commands to help you manage your tasks:

### Adding a task

```
task-cli add "Complete the project documentation"
```

### Updating a task

```
task-cli update 1 "Update the project documentation with examples"
```

### Marking a task as in progress

```
task-cli mark-in-progress 1
```

### Marking a task as done

```
task-cli mark-done 1
```

### Deleting a task

```
task-cli delete 1
```

### Listing tasks

List all tasks:

```
task-cli list
```

Filter tasks by status:

```
task-cli list todo
task-cli list in-progress
task-cli list done
```

## Data Storage

Task CLI stores your tasks in a file named `tasks.json` in the current
directory. This file is automatically created when you add your first task.

## Development

Task CLI is built with the following libraries:

- `clap` for command-line argument parsing
- `serde` and `serde_json` for JSON serialization
- `anyhow` for error handling

To contribute to this project:

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for
details.