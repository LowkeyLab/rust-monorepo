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

struct TaskRepository {
    tasks: Vec<Task>,
}

impl TaskRepository {
    fn new() -> Self {
        Self { tasks: vec![] }
    }
    fn add(&mut self, task: Task) {
        self.tasks.push(task);
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
        let _ = TaskRepository::new();
    }

    #[test]
    fn can_add_new_task() {
        let mut repo = TaskRepository::new();
        let task = Task::default();

        repo.add(task.clone());

        assert_eq!(repo.tasks.len(), 1);
        assert_eq!(repo.find_by_id(task.id), Some(&task));
    }
}
