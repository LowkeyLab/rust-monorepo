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
mod json_serialization_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_empty_repository_serialization() {
        let repo = TaskRepository::new();

        // Use serde_json to serialize to a string
        let serialized = serde_json::to_string(&repo.tasks).unwrap();

        // Check the serialized JSON
        assert_eq!(serialized, "{}");
    }

    #[test]
    fn test_task_serialization() {
        // Create a fixed timestamp for consistent testing
        let timestamp = chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let task = Task {
            id: 42,
            description: "Test task".to_string(),
            status: Status::Todo,
            created_at: timestamp,
            updated_at: timestamp,
        };

        let serialized = serde_json::to_string(&task).unwrap();

        // Verify the serialized JSON structure
        let expected = r#"{"id":42,"description":"Test task","status":"Todo","created_at":"2023-01-01T12:00:00Z","updated_at":"2023-01-01T12:00:00Z"}"#;
        assert_eq!(serialized, expected);

        // Test deserialization works correctly
        let deserialized: Task = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_repository_serialization_roundtrip() {
        let mut repo = TaskRepository::new();
        let timestamp = chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        // Add a few tasks
        for i in 1..=3 {
            let task = Task {
                id: i,
                description: format!("Task {}", i),
                status: Status::Todo,
                created_at: timestamp,
                updated_at: timestamp,
            };
            repo.add(task);
        }

        // Serialize to buffer
        let mut buffer = Vec::new();
        repo.save(&mut buffer);

        // Deserialize and verify
        let deserialized: HashMap<u32, Task> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(deserialized.len(), 3);

        // Verify each task was correctly serialized and deserialized
        for i in 1..=3 {
            let original = repo.tasks.get(&i).unwrap();
            let deserialized = deserialized.get(&i).unwrap();

            assert_eq!(deserialized.id, original.id);
            assert_eq!(deserialized.description, original.description);
            assert_eq!(deserialized.status, original.status);
            assert_eq!(deserialized.created_at, original.created_at);
            assert_eq!(deserialized.updated_at, original.updated_at);
        }
    }

    #[test]
    fn test_status_enum_serialization() {
        let status = Status::Todo;

        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"Todo\"");

        let deserialized: Status = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Status::Todo);
    }

    #[test]
    fn test_custom_json_format() {
        // Create a task with a fixed timestamp
        let timestamp = chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let task = Task {
            id: 1,
            description: "Sample task".to_string(),
            status: Status::Todo,
            created_at: timestamp,
            updated_at: timestamp,
        };

        // Test pretty-printed JSON
        let pretty_json = serde_json::to_string_pretty(&task).unwrap();

        // Verify the pretty-printed format has multiple lines and proper indentation
        assert!(pretty_json.contains("\n"));
        assert!(pretty_json.contains("  \""));

        // Verify we can deserialize the pretty-printed JSON
        let deserialized: Task = serde_json::from_str(&pretty_json).unwrap();
        assert_eq!(deserialized, task);
    }

    #[test]
    fn test_save_to_custom_writer() {
        let mut repo = TaskRepository::new();
        let timestamp = chrono::Utc::now();

        let task = Task {
            id: 1,
            description: "Test task".to_string(),
            status: Status::Todo,
            created_at: timestamp,
            updated_at: timestamp,
        };

        repo.add(task);

        // Use an in-memory cursor as a writer
        let mut cursor = Cursor::new(Vec::new());
        repo.save(&mut cursor);

        // Get the cursor's buffer
        let buffer = cursor.into_inner();

        // Verify we can deserialize from the cursor's buffer
        let deserialized: HashMap<u32, Task> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.contains_key(&1));
        assert_eq!(deserialized.get(&1).unwrap().description, "Test task");
    }
}
