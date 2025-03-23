use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Task {
    id: u32,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: 1,
            description: "Default description".to_string(),
            status: Status::Todo,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Status {
    #[default]
    Todo,
}

pub struct TaskRepository {
    tasks: HashMap<u32, Task>,
}

impl Default for TaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn new_from_json(json: &str) -> Self {
        let tasks: HashMap<u32, Task> = serde_json::from_str(json).unwrap();
        Self { tasks }
    }

    pub fn add(&mut self, description: String) -> u32 {
        let next_id: u32 = self.tasks.len() as u32 + 1;
        let task = Task::default();
        self.tasks.insert(
            next_id,
            Task {
                id: next_id,
                description,
                ..task
            },
        );
        next_id
    }

    pub fn save_as_json(&self, writer: impl std::io::Write) {
        serde_json::to_writer(writer, &self.tasks).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_task() {
        // Create a new task repository
        let mut repo = TaskRepository::new();

        // Add a task using the new method signature
        repo.add("Test task".to_string());

        // Verify the task was added correctly
        assert_eq!(repo.tasks.len(), 1);
        assert!(repo.tasks.contains_key(&1)); // Assuming IDs start at 1

        // Retrieve the task and verify its properties
        let retrieved_task = repo.tasks.get(&1).unwrap();
        assert_eq!(retrieved_task.id, 1);
        assert_eq!(retrieved_task.description, "Test task");
        assert_eq!(retrieved_task.status, Status::Todo);

        // We can't assert exact timestamps anymore since they're generated internally
        // but we can check that they exist
        assert!(retrieved_task.created_at <= chrono::Utc::now());
        assert!(retrieved_task.updated_at <= chrono::Utc::now());
    }

    #[test]
    fn test_add_multiple_tasks() {
        let mut repo = TaskRepository::new();

        // Add multiple tasks with the new method signature
        repo.add("Task 1".to_string());
        repo.add("Task 2".to_string());
        repo.add("Task 3".to_string());

        // Verify all tasks were added
        assert_eq!(repo.tasks.len(), 3);
        assert!(repo.tasks.contains_key(&1));
        assert!(repo.tasks.contains_key(&2));
        assert!(repo.tasks.contains_key(&3));

        // Check descriptions
        assert_eq!(repo.tasks.get(&1).unwrap().description, "Task 1");
        assert_eq!(repo.tasks.get(&2).unwrap().description, "Task 2");
        assert_eq!(repo.tasks.get(&3).unwrap().description, "Task 3");
    }

    #[test]
    fn test_add_overwrites_existing_task() {
        // Note: This test may need to be completely reconsidered if the new
        // implementation doesn't allow for overwriting by ID (since tasks are
        // now created internally)

        let mut repo = TaskRepository::new();

        // Add initial task
        repo.add("Initial task".to_string());
        let initial_id = 1; // Assuming first task gets ID 1

        // Verify initial task
        assert_eq!(repo.tasks.len(), 1);
        assert_eq!(
            repo.tasks.get(&initial_id).unwrap().description,
            "Initial task"
        );

        // Add another task (which should get a new ID, not overwrite)
        repo.add("Second task".to_string());

        // Verify there are now two tasks
        assert_eq!(repo.tasks.len(), 2);
        assert_eq!(
            repo.tasks.get(&initial_id).unwrap().description,
            "Initial task"
        );
        assert_eq!(repo.tasks.get(&2).unwrap().description, "Second task");
    }
}
