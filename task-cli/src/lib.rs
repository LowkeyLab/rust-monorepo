use crate::Status::Todo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Task {
    id: u32,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} [Status: {}]",
            self.id, self.description, self.status
        )
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Status {
    #[default]
    Todo,
}
impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Todo => write!(f, "To Do"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskRepository {
    tasks: HashMap<u32, Task>,
    next_id: u32,
}

impl TaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn new_from_json(json: &str) -> Self {
        serde_json::from_str(json).expect("cannot deserialize repository")
    }

    pub fn get_task(&self, id: u32) -> Option<&Task> {
        self.tasks.get(&id)
    }

    pub fn update_task(&mut self, id: u32, description: String) -> Result<(), String> {
        match self.tasks.get_mut(&id) {
            Some(task) => {
                task.description = description;
                task.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => Err(format!("Task with ID {} not found", id)),
        }
    }

    pub fn add(&mut self, description: String) -> u32 {
        let curr_id = self.next_id;
        self.tasks.insert(
            curr_id,
            Task {
                id: curr_id,
                description,
                status: Todo,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        );
        self.next_id += 1;
        curr_id
    }

    pub fn save_as_json(&self, writer: impl std::io::Write) {
        serde_json::to_writer(writer, &self).expect("cannot serialize repository");
    }
}

impl Display for TaskRepository {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Task Repository ({} tasks):", self.tasks.len())?;

        if self.tasks.is_empty() {
            writeln!(f, "  No tasks found.")?;
        } else {
            // Get a sorted list of task IDs for consistent output
            let mut task_ids: Vec<&u32> = self.tasks.keys().collect();
            task_ids.sort();

            // Format each task
            for id in task_ids {
                let task = &self.tasks[id];
                writeln!(f, "  {}", task)?;
            }
        }

        Ok(())
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
}

#[cfg(test)]
mod next_id_tests {
    use super::*;

    #[test]
    fn test_new_repository_starts_with_id_one() {
        let repo = TaskRepository::new();
        assert_eq!(
            repo.next_id, 1,
            "New repository should start with next_id = 1"
        );
    }

    #[test]
    fn test_next_id_increments_after_adding_task() {
        let mut repo = TaskRepository::new();

        // Add a task and capture the returned ID
        let id = repo.add("Test task".to_string());

        // Verify the ID matches what we'd expect
        assert_eq!(id, 1, "First task should have ID 1");

        // Verify next_id was incremented
        assert_eq!(repo.next_id, 2, "next_id should be incremented to 2");
    }

    #[test]
    fn test_next_id_increments_correctly_for_multiple_tasks() {
        let mut repo = TaskRepository::new();

        // Add several tasks
        let id1 = repo.add("Task 1".to_string());
        let id2 = repo.add("Task 2".to_string());
        let id3 = repo.add("Task 3".to_string());

        // Verify IDs and next_id value
        assert_eq!(id1, 1, "First task should have ID 1");
        assert_eq!(id2, 2, "Second task should have ID 2");
        assert_eq!(id3, 3, "Third task should have ID 3");
        assert_eq!(repo.next_id, 4, "next_id should be incremented to 4");
    }

    #[test]
    fn test_next_id_preserved_when_loading_from_json() {
        // Create a repository with some tasks and a specific next_id
        let mut original_repo = TaskRepository::new();
        original_repo.add("Task 1".to_string());
        original_repo.add("Task 2".to_string());

        // At this point, next_id should be 3
        assert_eq!(original_repo.next_id, 3);

        // Serialize the repository
        let mut buffer = Vec::new();
        original_repo.save_as_json(&mut buffer);
        let json = String::from_utf8(buffer).unwrap();

        // Create a new repository from this JSON
        let loaded_repo = TaskRepository::new_from_json(&json);

        // Verify the next_id was preserved
        assert_eq!(
            loaded_repo.next_id, 3,
            "next_id should be preserved when loading from JSON"
        );
    }

    #[test]
    fn test_add_task_uses_next_id() {
        let mut repo = TaskRepository::new();

        // Manually set next_id to a custom value
        repo.next_id = 42;

        // Add a task
        let id = repo.add("Task with custom ID".to_string());

        // Verify the task got the expected ID and next_id was incremented
        assert_eq!(
            id, 42,
            "Task should have been assigned the current next_id value"
        );
        assert_eq!(repo.next_id, 43, "next_id should have been incremented");
        assert!(
            repo.tasks.contains_key(&42),
            "Task should be stored with ID 42"
        );
    }

    #[test]
    fn test_json_with_next_id_respects_provided_value() {
        // Create JSON with explicit next_id
        let json = r#"
        {
            "tasks": {
                "1": {
                    "id": 1,
                    "description": "Task 1",
                    "status": "Todo",
                    "created_at": "2023-01-01T00:00:00Z",
                    "updated_at": "2023-01-01T00:00:00Z"
                }
            },
            "next_id": 100
        }
        "#;

        // Load the repository
        let repo = TaskRepository::new_from_json(json);

        // Verify the specified next_id is used
        assert_eq!(
            repo.next_id, 100,
            "Explicit next_id in JSON should be respected"
        );
    }

    #[test]
    fn test_next_id_maintained_after_removing_tasks() {
        let mut repo = TaskRepository::new();

        // Add tasks
        repo.add("Task 1".to_string());
        repo.add("Task 2".to_string());
        repo.add("Task 3".to_string());

        // Remove a task (assuming a remove method exists or simulating removal)
        repo.tasks.remove(&2);

        // next_id should still be 4
        assert_eq!(
            repo.next_id, 4,
            "next_id should not change when tasks are removed"
        );

        // Add another task and check its ID
        let id = repo.add("Task 4".to_string());
        assert_eq!(
            id, 4,
            "New task should get ID 4, not reuse the removed ID 2"
        );
    }
}
#[cfg(test)]
mod update_task_tests {
    use super::*;

    #[test]
    fn test_update_nonexistent_task_returns_error() {
        let mut repo = TaskRepository::new();
        let result = repo.update_task(1, "Updated task".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Task with ID 1 not found");
    }

    #[test]
    fn test_update_task_description_success() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Original task".to_string());

        let result = repo.update_task(id, "Updated description".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, "Updated description");
    }

    #[test]
    fn test_update_task_preserves_status() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Original task".to_string());

        // Assuming tasks start with Status::Todo
        // First update the status to something else if your code supports it
        // For example: repo.set_status(id, Status::InProgress);

        // Now update the description
        let result = repo.update_task(id, "Updated description".to_string());
        assert!(result.is_ok());

        // Check that status is preserved
        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.status, Status::Todo); // Or whatever the status was before
    }

    #[test]
    fn test_update_task_updates_timestamp() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Task for timestamp check".to_string());

        let original_task = repo.get_task(id).unwrap().clone();

        // Wait a small amount of time to ensure timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(5));

        let result = repo.update_task(id, "Updated for timestamp".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert!(updated_task.updated_at > original_task.updated_at);
    }

    #[test]
    fn test_update_multiple_tasks() {
        let mut repo = TaskRepository::new();
        let id1 = repo.add("First task".to_string());
        let id2 = repo.add("Second task".to_string());

        // Update first task
        let result1 = repo.update_task(id1, "Updated first".to_string());
        assert!(result1.is_ok());

        // Update second task
        let result2 = repo.update_task(id2, "Updated second".to_string());
        assert!(result2.is_ok());

        // Check both updates worked correctly
        let task1 = repo.get_task(id1).unwrap();
        assert_eq!(task1.description, "Updated first");

        let task2 = repo.get_task(id2).unwrap();
        assert_eq!(task2.description, "Updated second");
    }

    #[test]
    fn test_update_task_with_same_description() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Original description".to_string());

        let original_task = repo.get_task(id).unwrap().clone();

        // Wait a small amount of time to ensure timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(5));

        // Update with the same description
        let result = repo.update_task(id, "Original description".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, original_task.description);
        // Even with the same content, updated_at should be refreshed
        assert!(updated_task.updated_at > original_task.updated_at);
    }

    #[test]
    fn test_update_task_empty_description() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Initial description".to_string());

        let result = repo.update_task(id, "".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, "");
    }

    #[test]
    fn test_update_task_with_long_description() {
        let mut repo = TaskRepository::new();
        let id = repo.add("Short description".to_string());

        // Create a very long description
        let long_description = "a".repeat(1000);

        let result = repo.update_task(id, long_description.clone());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, long_description);
    }
}
