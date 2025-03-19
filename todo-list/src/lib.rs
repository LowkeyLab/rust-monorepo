mod repository;

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
