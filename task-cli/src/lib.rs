#[derive(Debug, Default, Eq, PartialEq, Clone)]
struct Task {
    id: u32,
    description: String,
    status: Status,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
enum Status {
    #[default]
    Todo,
}

trait TaskRepository {
    fn add(&mut self, task: Task);
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn find_by_id(&self, id: u32) -> Option<&Task>;
}

struct InMemoryTaskRepository {
    to_be_saved: Vec<Task>,
    tasks: Vec<Task>,
}

impl InMemoryTaskRepository {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            to_be_saved: Vec::new(),
        }
    }
}

impl TaskRepository for InMemoryTaskRepository {
    fn add(&mut self, task: Task) {
        self.to_be_saved.push(task);
    }
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tasks.append(&mut self.to_be_saved);
        Ok(())
    }

    fn find_by_id(&self, id: u32) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_task_repository() {
        let _ = InMemoryTaskRepository::new();
    }

    #[test]
    fn can_add_new_task() {
        let mut repo = InMemoryTaskRepository::new();
        let task = Task::default();

        repo.add(task.clone());
        repo.save().unwrap();

        assert_eq!(repo.tasks.len(), 1);
        assert_eq!(repo.find_by_id(task.id), Some(&task));
    }
}
