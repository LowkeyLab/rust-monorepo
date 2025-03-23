use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct Task {
    id: u32,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
enum Status {
    #[default]
    Todo,
}

fn save(task_list: HashMap<u32, Task>, writer: impl std::io::Write) {
    serde_json::to_writer(writer, &task_list).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_empty_map() {
        let task_list = HashMap::new();
        let mut buffer = Vec::new();

        save(task_list, &mut buffer);

        assert_eq!(buffer, b"{}");
    }

    #[test]
    fn test_save_single_task() {
        let mut task_list = HashMap::new();
        let now = chrono::Utc::now();

        let task = Task {
            id: 1,
            description: "Test task".to_string(),
            status: Status::Todo,
            created_at: now,
            updated_at: now,
        };

        task_list.insert(1, task);

        let mut buffer = Vec::new();
        save(task_list, &mut buffer);

        // Deserialize back to verify
        let deserialized: HashMap<u32, Task> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.contains_key(&1));
        assert_eq!(deserialized.get(&1).unwrap().description, "Test task");
    }

    #[test]
    fn test_save_multiple_tasks() {
        let mut task_list = HashMap::new();
        let now = chrono::Utc::now();

        for i in 1..=3 {
            let task = Task {
                id: i,
                description: format!("Task {}", i),
                status: Status::Todo,
                created_at: now,
                updated_at: now,
            };

            task_list.insert(i, task);
        }

        let mut buffer = Vec::new();
        save(task_list, &mut buffer);

        // Deserialize back to verify
        let deserialized: HashMap<u32, Task> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(deserialized.len(), 3);
        for i in 1..=3 {
            assert!(deserialized.contains_key(&i));
            assert_eq!(
                deserialized.get(&i).unwrap().description,
                format!("Task {}", i)
            );
        }
    }

    #[test]
    fn test_save_to_file() {
        use std::fs::{self, File};
        use std::io::Read;

        let mut task_list = HashMap::new();
        let now = chrono::Utc::now();

        let task = Task {
            id: 42,
            description: "File test task".to_string(),
            status: Status::Todo,
            created_at: now,
            updated_at: now,
        };

        task_list.insert(42, task);

        let temp_file = "test_tasks.json";
        {
            let file = File::create(temp_file).unwrap();
            save(task_list, file);
        }

        // Read file content
        let mut file_content = String::new();
        File::open(temp_file)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();

        // Deserialize and verify content
        let deserialized: HashMap<u32, Task> = serde_json::from_str(&file_content).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.contains_key(&42));
        assert_eq!(deserialized.get(&42).unwrap().description, "File test task");

        // Clean up
        fs::remove_file(temp_file).unwrap();
    }
}
