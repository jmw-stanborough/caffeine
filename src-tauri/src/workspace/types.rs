use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub completed_at: Option<i64>,
    pub due_at: Option<i64>,
    pub reminder_at: Option<i64>,
    pub reminder_sent: bool,
    pub reminder_acknowledged: bool,
    pub recurrence: Option<String>,
    pub created_at: i64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateTodoInput {
    pub title: String,
    pub due_at: Option<i64>,
    pub reminder_at: Option<i64>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateTodoInput {
    pub id: String,
    pub title: String,
    pub due_at: Option<i64>,
    pub reminder_at: Option<i64>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteSummary {
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteDocument {
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub content: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectedScriptLanguage {
    pub id: String,
    pub label: String,
    pub extension: String,
    pub executable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptSummary {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub extension: String,
    pub language_id: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptDocument {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub extension: String,
    pub language_id: String,
    pub content: String,
    pub path: String,
    pub updated_at_ms: i64,
}
