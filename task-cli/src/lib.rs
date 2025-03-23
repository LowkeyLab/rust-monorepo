use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Clone)]
struct Task {
    id: u32,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Clone)]
enum Status {
    #[default]
    Todo,
}

struct TaskRepository {
    tasks: HashMap<u32, Task>,
}

trait SaveTask {
    fn save(&self, writer: impl std::io::Write);
}

impl TaskRepository {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    fn new_from_json(json: &str) -> Self {
        let tasks: HashMap<u32, Task> = serde_json::from_str(json).unwrap();
        Self { tasks }
    }

    fn add(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }
}

impl SaveTask for TaskRepository {
    fn save(&self, writer: impl std::io::Write) {
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

        // Create a task
        let now = chrono::Utc::now();
        let task = Task {
            id: 1,
            description: "Test task".to_string(),
            status: Status::Todo,
            created_at: now,
            updated_at: now,
        };

        // Add the task to the repository
        repo.add(task.clone());

        // Verify the task was added correctly
        assert_eq!(repo.tasks.len(), 1);
        assert!(repo.tasks.contains_key(&1));

        // Retrieve the task and verify its properties
        let retrieved_task = repo.tasks.get(&1).unwrap();
        assert_eq!(retrieved_task.id, 1);
        assert_eq!(retrieved_task.description, "Test task");
        assert_eq!(retrieved_task.status, Status::Todo);
        assert_eq!(retrieved_task.created_at, now);
        assert_eq!(retrieved_task.updated_at, now);
    }

    #[test]
    fn test_add_multiple_tasks() {
        // Create a new task repository
        let mut repo = TaskRepository::new();
        let now = chrono::Utc::now();

        // Add multiple tasks
        for i in 1..=3 {
            let task = Task {
                id: i,
                description: format!("Task {}", i),
                status: Status::Todo,
                created_at: now,
                updated_at: now,
            };

            repo.add(task);
        }

        // Verify all tasks were added
        assert_eq!(repo.tasks.len(), 3);

        // Check each task
        for i in 1..=3 {
            assert!(repo.tasks.contains_key(&i));
            let task = repo.tasks.get(&i).unwrap();
            assert_eq!(task.id, i);
            assert_eq!(task.description, format!("Task {}", i));
        }
    }

    #[test]
    fn test_add_overwrites_existing_task() {
        // Create a new task repository
        let mut repo = TaskRepository::new();
        let now = chrono::Utc::now();

        // Create and add initial task
        let initial_task = Task {
            id: 1,
            description: "Initial description".to_string(),
            status: Status::Todo,
            created_at: now,
            updated_at: now,
        };

        repo.add(initial_task);

        // Create a new task with same ID but different description
        let updated_task = Task {
            id: 1,                                          // Same ID
            description: "Updated description".to_string(), // Different description
            status: Status::Todo,
            created_at: now,
            updated_at: now,
        };

        // Add the updated task
        repo.add(updated_task);

        // Verify there's still only one task
        assert_eq!(repo.tasks.len(), 1);

        // Verify it has the updated description
        let task = repo.tasks.get(&1).unwrap();
        assert_eq!(task.description, "Updated description");
    }
}

#[cfg(test)]
mod new_from_json_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_empty_repository_from_json() {
        let json = "{}";
        let repo = TaskRepository::new_from_json(json);
        assert!(repo.tasks.is_empty());
    }

    #[test]
    fn test_single_task_from_json() {
        let now = Utc::now();
        let json = format!(
            r#"{{
            "1": {{
                "id": 1,
                "description": "Test task",
                "status": "Todo",
                "created_at": "{}",
                "updated_at": "{}"
            }}
        }}"#,
            now, now
        );

        let repo = TaskRepository::new_from_json(&json);

        assert_eq!(repo.tasks.len(), 1);
        let task = repo.tasks.get(&1).unwrap();
        assert_eq!(task.id, 1);
        assert_eq!(task.description, "Test task");
        assert_eq!(task.status, Status::Todo);
    }

    #[test]
    fn test_multiple_tasks_from_json() {
        let now = Utc::now();
        let json = format!(
            r#"{{
            "1": {{
                "id": 1,
                "description": "First task",
                "status": "Todo",
                "created_at": "{}",
                "updated_at": "{}"
            }},
            "2": {{
                "id": 2,
                "description": "Second task",
                "status": "Todo",
                "created_at": "{}",
                "updated_at": "{}"
            }}
        }}"#,
            now, now, now, now
        );

        let repo = TaskRepository::new_from_json(&json);

        assert_eq!(repo.tasks.len(), 2);
        assert!(repo.tasks.contains_key(&1));
        assert!(repo.tasks.contains_key(&2));
    }

    #[test]
    fn test_invalid_json() {
        let json = "invalid json";
        // This assumes new_from_json panics or returns an error for invalid JSON
        // May need adjusting depending on actual error handling
        let result = std::panic::catch_unwind(|| TaskRepository::new_from_json(json));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod json_serialization_tests {
    use super::*;

    #[test]
    fn test_empty_repository_serialization() {
        let repo = TaskRepository::new();

        // Serialize to string
        let serialized = serde_json::to_string(&repo.tasks).unwrap();

        // Check the serialized JSON
        assert_eq!(serialized, "{}");

        // Test roundtrip using new_from_json
        let deserialized = TaskRepository::new_from_json(&serialized);
        assert!(deserialized.tasks.is_empty());
    }

    #[test]
    fn test_task_serialization() {
        let mut repo = TaskRepository::new();
        let task = Task {
            id: 1,
            description: "Test task".to_string(),
            status: Status::Todo,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        repo.add(task.clone());

        // Serialize to string
        let serialized = serde_json::to_string(&repo.tasks).unwrap();

        // Deserialize using new_from_json
        let deserialized = TaskRepository::new_from_json(&serialized);

        // Check if the task was properly serialized and deserialized
        assert_eq!(deserialized.tasks.len(), 1);
        assert_eq!(deserialized.tasks.get(&1), Some(&task));
    }

    #[test]
    fn test_repository_serialization_roundtrip() {
        let mut repo = TaskRepository::new();
        let task1 = Task {
            id: 1,
            description: "First task".to_string(),
            status: Status::Todo,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let task2 = Task {
            id: 2,
            description: "Second task".to_string(),
            status: Status::Todo,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        repo.add(task1);
        repo.add(task2);

        // Serialize to string
        let serialized = serde_json::to_string(&repo.tasks).unwrap();

        // Deserialize using new_from_json
        let deserialized = TaskRepository::new_from_json(&serialized);

        // Check if tasks are equal
        assert_eq!(deserialized.tasks.len(), 2);
        assert_eq!(deserialized.tasks, repo.tasks);
    }

    // The other tests can remain as they are since they test specific functionality
    // not directly related to new_from_json
}
