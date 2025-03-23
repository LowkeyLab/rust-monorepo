use crate::Status::{InProgress, Todo};
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
    InProgress,
}
impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Todo => write!(f, "To Do"),
            InProgress => write!(f, "In Progress"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskRepository {
    tasks: HashMap<u32, Task>,
    next_id: u32,
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

    pub fn delete_task(&mut self, id: u32) {
        self.tasks.remove(&id);
    }

    pub fn add_task(&mut self, description: String) -> u32 {
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

    pub fn mark_in_progress(&mut self, id: u32) -> Result<(), String> {
        let Some(task) = self.tasks.get_mut(&id) else {
            return Err(format!("Task with ID {} not found", id));
        };
        task.status = InProgress;
        task.updated_at = chrono::Utc::now();
        Ok(())
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
        repo.add_task("Test task".to_string());

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
        repo.add_task("Task 1".to_string());
        repo.add_task("Task 2".to_string());
        repo.add_task("Task 3".to_string());

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
        let id = repo.add_task("Test task".to_string());

        // Verify the ID matches what we'd expect
        assert_eq!(id, 1, "First task should have ID 1");

        // Verify next_id was incremented
        assert_eq!(repo.next_id, 2, "next_id should be incremented to 2");
    }

    #[test]
    fn test_next_id_increments_correctly_for_multiple_tasks() {
        let mut repo = TaskRepository::new();

        // Add several tasks
        let id1 = repo.add_task("Task 1".to_string());
        let id2 = repo.add_task("Task 2".to_string());
        let id3 = repo.add_task("Task 3".to_string());

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
        original_repo.add_task("Task 1".to_string());
        original_repo.add_task("Task 2".to_string());

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
        let id = repo.add_task("Task with custom ID".to_string());

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
        repo.add_task("Task 1".to_string());
        repo.add_task("Task 2".to_string());
        repo.add_task("Task 3".to_string());

        // Remove a task (assuming a remove method exists or simulating removal)
        repo.tasks.remove(&2);

        // next_id should still be 4
        assert_eq!(
            repo.next_id, 4,
            "next_id should not change when tasks are removed"
        );

        // Add another task and check its ID
        let id = repo.add_task("Task 4".to_string());
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
        let id = repo.add_task("Original task".to_string());

        let result = repo.update_task(id, "Updated description".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, "Updated description");
    }

    #[test]
    fn test_update_task_preserves_status() {
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Original task".to_string());

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
        let id = repo.add_task("Task for timestamp check".to_string());

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
        let id1 = repo.add_task("First task".to_string());
        let id2 = repo.add_task("Second task".to_string());

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
        let id = repo.add_task("Original description".to_string());

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
        let id = repo.add_task("Initial description".to_string());

        let result = repo.update_task(id, "".to_string());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, "");
    }

    #[test]
    fn test_update_task_with_long_description() {
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Short description".to_string());

        // Create a very long description
        let long_description = "a".repeat(1000);

        let result = repo.update_task(id, long_description.clone());
        assert!(result.is_ok());

        let updated_task = repo.get_task(id).unwrap();
        assert_eq!(updated_task.description, long_description);
    }
}
#[cfg(test)]
mod delete_task_tests {
    use super::*;

    #[test]
    fn test_delete_existing_task() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Test task".to_string());

        // Act
        repo.delete_task(id);

        // Assert
        assert!(repo.get_task(id).is_none());
    }

    #[test]
    fn test_delete_nonexistent_task_does_not_panic() {
        // Arrange
        let mut repo = TaskRepository::new();
        let nonexistent_id = 9999;

        // Act & Assert - should not panic
        repo.delete_task(nonexistent_id);
    }

    #[test]
    fn test_delete_task_maintains_other_tasks() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id1 = repo.add_task("Task 1".to_string());
        let id2 = repo.add_task("Task 2".to_string());
        let id3 = repo.add_task("Task 3".to_string());

        // Act
        repo.delete_task(id2);

        // Assert
        assert!(repo.get_task(id1).is_some());
        assert!(repo.get_task(id2).is_none());
        assert!(repo.get_task(id3).is_some());
    }

    #[test]
    fn test_delete_task_cannot_retrieve_deleted_task() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Doomed task".to_string());

        // Save the task data before deletion
        let _ = repo.get_task(id).unwrap().clone();

        // Act
        repo.delete_task(id);

        // Assert
        assert!(repo.get_task(id).is_none());

        // Add a new task and verify it gets a new ID
        let new_id = repo.add_task("New task".to_string());
        assert_ne!(id, new_id);
    }

    #[test]
    fn test_delete_and_readd_with_same_description() {
        // Arrange
        let mut repo = TaskRepository::new();
        let description = "Recurring task".to_string();
        let id1 = repo.add_task(description.clone());

        // Act
        repo.delete_task(id1);
        let id2 = repo.add_task(description);

        // Assert
        assert_ne!(id1, id2); // Should get a new ID
        assert!(repo.get_task(id1).is_none());
        assert!(repo.get_task(id2).is_some());
    }

    #[test]
    fn test_delete_all_tasks() {
        // Arrange
        let mut repo = TaskRepository::new();
        let ids = vec![
            repo.add_task("Task 1".to_string()),
            repo.add_task("Task 2".to_string()),
            repo.add_task("Task 3".to_string()),
        ];

        // Act
        for id in ids {
            repo.delete_task(id);
        }

        // Assert - repository should be empty
        // This assumes there's a way to get all tasks or check if empty
        for id in 1..10 {
            assert!(repo.get_task(id).is_none());
        }
    }

    #[test]
    fn test_delete_same_task_twice() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task to delete twice".to_string());

        // Act
        repo.delete_task(id);

        // Should not panic when deleting again
        repo.delete_task(id);

        // Assert
        assert!(repo.get_task(id).is_none());
    }
}

#[cfg(test)]
mod mark_in_progress_tests {
    use super::*;

    #[test]
    fn test_mark_task_as_in_progress_success() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Test task".to_string());

        // Act
        let result = repo.mark_in_progress(id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(repo.get_task(id).unwrap().status, Status::InProgress);
    }

    #[test]
    fn test_mark_nonexistent_task_returns_error() {
        // Arrange
        let mut repo = TaskRepository::new();
        let nonexistent_id = 999;

        // Act
        let result = repo.mark_in_progress(nonexistent_id);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains(&nonexistent_id.to_string()));
    }

    #[test]
    fn test_mark_already_in_progress_task() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task in progress".to_string());
        repo.mark_in_progress(id).unwrap(); // First transition

        // Act
        let result = repo.mark_in_progress(id); // Second transition

        // Assert
        assert!(result.is_ok());
        assert_eq!(repo.get_task(id).unwrap().status, Status::InProgress);
    }

    #[test]
    fn test_mark_in_progress_updates_timestamp() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task to update".to_string());
        let before = repo.get_task(id).unwrap().updated_at;

        // Add a small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(5));

        // Act
        repo.mark_in_progress(id).unwrap();

        // Assert
        let after = repo.get_task(id).unwrap().updated_at;
        assert!(
            after > before,
            "The updated_at timestamp should increase after marking as in-progress"
        );
    }

    #[test]
    fn test_mark_in_progress_preserves_description() {
        // Arrange
        let mut repo = TaskRepository::new();
        let description = "Task with description".to_string();
        let id = repo.add_task(description.clone());

        // Act
        repo.mark_in_progress(id).unwrap();

        // Assert
        assert_eq!(repo.get_task(id).unwrap().description, description);
    }

    #[test]
    fn test_mark_in_progress_preserves_id() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task with ID".to_string());

        // Act
        repo.mark_in_progress(id).unwrap();

        // Assert
        assert_eq!(repo.get_task(id).unwrap().id, id);
    }

    #[test]
    fn test_mark_in_progress_preserves_created_at() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task with creation time".to_string());
        let created_at = repo.get_task(id).unwrap().created_at;

        // Act
        repo.mark_in_progress(id).unwrap();

        // Assert
        assert_eq!(repo.get_task(id).unwrap().created_at, created_at);
    }

    #[test]
    fn test_mark_multiple_tasks_in_progress() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id1 = repo.add_task("First task".to_string());
        let id2 = repo.add_task("Second task".to_string());

        // Act
        let result1 = repo.mark_in_progress(id1);
        let result2 = repo.mark_in_progress(id2);

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(repo.get_task(id1).unwrap().status, Status::InProgress);
        assert_eq!(repo.get_task(id2).unwrap().status, Status::InProgress);
    }

    #[test]
    fn test_mark_deleted_task_in_progress() {
        // Arrange
        let mut repo = TaskRepository::new();
        let id = repo.add_task("Task to delete".to_string());
        repo.delete_task(id);

        // Act
        let result = repo.mark_in_progress(id);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains(&id.to_string()));
    }
}
