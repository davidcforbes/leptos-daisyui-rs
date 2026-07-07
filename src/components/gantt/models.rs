use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::style::TaskType;

/// Represents a task in the Gantt chart
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GanttTask {
    /// Unique identifier for the task
    pub id: String,

    /// Display name of the task
    pub name: String,

    /// Task start date/time
    pub start: DateTime<Utc>,

    /// Task end date/time
    pub end: DateTime<Utc>,

    /// Task completion progress (0.0 to 1.0)
    pub progress: f32,

    /// Type of task (Task, Milestone, Project)
    pub task_type: TaskType,

    /// Parent task ID for hierarchical tasks
    pub parent_id: Option<String>,

    /// List of task dependencies
    pub dependencies: Vec<String>,

    /// Assigned users/resources
    pub assignees: Vec<String>,

    /// Custom color for the task bar (hex color)
    pub color: Option<String>,

    /// Per-task read-only flag (overrides global read-only mode)
    #[serde(default)]
    pub read_only: bool,

    /// Additional metadata for custom extensions
    pub metadata: HashMap<String, String>,
}

/// Represents a dependency relationship between two tasks
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskDependency {
    /// Source task ID (the task that must be completed)
    pub source_id: String,

    /// Target task ID (the task that depends on the source)
    pub target_id: String,

    /// Type of dependency relationship
    pub dependency_type: DependencyType,

    /// Number of days lag/lead time (positive for lag, negative for lead)
    pub lag_days: i32,
}

/// Type of dependency relationship between tasks
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DependencyType {
    /// Finish-to-Start: Source task must finish before target can start
    #[default]
    FS,

    /// Start-to-Start: Both tasks must start at the same time
    SS,

    /// Finish-to-Finish: Both tasks must finish at the same time
    FF,

    /// Start-to-Finish: Source task must start before target can finish
    SF,
}

impl DependencyType {
    /// Returns the string representation of the dependency type
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::FS => "FS",
            DependencyType::SS => "SS",
            DependencyType::FF => "FF",
            DependencyType::SF => "SF",
        }
    }
}

/// Content fingerprint of a task, combined with its position in the list,
/// used as the row key inside [`GanttChart`](super::component::GanttChart)'s
/// `<For>` lists (task list rows and task bars) so a row/bar re-renders
/// whenever any of its render-relevant fields change — not just when the
/// task's `id` stays the same. Without this, updating a task in place (same
/// `id`, changed progress/dates/name/type — e.g. a milestone's progress
/// crossing 1.0, which should flip its marker from outline to filled) left
/// `<For>` reusing the stale view.
///
/// `DateTime<Utc>` and `f32` don't implement `Hash`, and `HashMap` iteration
/// order isn't stable, so this hashes fields manually rather than deriving
/// `Hash` on `GanttTask` directly: dates via `timestamp_millis()`, `progress`
/// via `to_bits()`, and `metadata` via its entries sorted by key.
pub fn task_key(index: usize, task: &GanttTask) -> (usize, u64) {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    task.id.hash(&mut hasher);
    task.name.hash(&mut hasher);
    task.start.timestamp_millis().hash(&mut hasher);
    task.end.timestamp_millis().hash(&mut hasher);
    task.progress.to_bits().hash(&mut hasher);
    task.task_type.hash(&mut hasher);
    task.parent_id.hash(&mut hasher);
    task.dependencies.hash(&mut hasher);
    task.assignees.hash(&mut hasher);
    task.color.hash(&mut hasher);
    task.read_only.hash(&mut hasher);

    let mut metadata_entries: Vec<(&String, &String)> = task.metadata.iter().collect();
    metadata_entries.sort_by(|a, b| a.0.cmp(b.0));
    metadata_entries.hash(&mut hasher);

    (index, hasher.finish())
}

impl Default for GanttTask {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            start: Utc::now(),
            end: Utc::now(),
            progress: 0.0,
            task_type: TaskType::default(),
            parent_id: None,
            dependencies: Vec::new(),
            assignees: Vec::new(),
            color: None,
            read_only: false,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_gantt_task_default() {
        let task = GanttTask::default();

        assert_eq!(task.id, "");
        assert_eq!(task.name, "");
        assert_eq!(task.progress, 0.0);
        assert_eq!(task.task_type, TaskType::Task);
        assert!(task.parent_id.is_none());
        assert!(task.dependencies.is_empty());
        assert!(task.assignees.is_empty());
        assert!(task.color.is_none());
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_gantt_task_custom() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();

        let task = GanttTask {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            start,
            end,
            progress: 0.5,
            task_type: TaskType::Task,
            parent_id: Some("parent-1".to_string()),
            dependencies: vec!["dep-1".to_string()],
            assignees: vec!["user1".to_string(), "user2".to_string()],
            color: Some("#ff0000".to_string()),
            read_only: false,
            metadata: HashMap::from([("category".to_string(), "development".to_string())]),
        };

        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.start, start);
        assert_eq!(task.end, end);
        assert_eq!(task.progress, 0.5);
        assert_eq!(task.task_type, TaskType::Task);
        assert_eq!(task.parent_id, Some("parent-1".to_string()));
        assert_eq!(task.dependencies.len(), 1);
        assert_eq!(task.assignees.len(), 2);
        assert_eq!(task.color, Some("#ff0000".to_string()));
        assert_eq!(
            task.metadata.get("category"),
            Some(&"development".to_string())
        );
    }

    #[test]
    fn test_task_types() {
        assert_eq!(TaskType::Task.as_str(), "task");
        assert_eq!(TaskType::Milestone.as_str(), "milestone");
        assert_eq!(TaskType::Project.as_str(), "project");
        assert_eq!(TaskType::default(), TaskType::Task);
    }

    #[test]
    fn test_task_dependency() {
        let dep = TaskDependency {
            source_id: "task-1".to_string(),
            target_id: "task-2".to_string(),
            dependency_type: DependencyType::FS,
            lag_days: 2,
        };

        assert_eq!(dep.source_id, "task-1");
        assert_eq!(dep.target_id, "task-2");
        assert_eq!(dep.dependency_type, DependencyType::FS);
        assert_eq!(dep.lag_days, 2);
    }

    #[test]
    fn test_dependency_types() {
        assert_eq!(DependencyType::FS.as_str(), "FS");
        assert_eq!(DependencyType::SS.as_str(), "SS");
        assert_eq!(DependencyType::FF.as_str(), "FF");
        assert_eq!(DependencyType::SF.as_str(), "SF");
        assert_eq!(DependencyType::default(), DependencyType::FS);
    }

    #[test]
    fn test_gantt_task_serialization() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();

        let task = GanttTask {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            start,
            end,
            progress: 0.75,
            task_type: TaskType::Milestone,
            parent_id: None,
            dependencies: vec![],
            assignees: vec!["user1".to_string()],
            color: None,
            read_only: false,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: GanttTask = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, task.id);
        assert_eq!(deserialized.name, task.name);
        assert_eq!(deserialized.progress, task.progress);
        assert_eq!(deserialized.task_type, task.task_type);
    }

    #[test]
    fn test_task_dependency_serialization() {
        let dep = TaskDependency {
            source_id: "task-1".to_string(),
            target_id: "task-2".to_string(),
            dependency_type: DependencyType::SS,
            lag_days: 0,
        };

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: TaskDependency = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.source_id, dep.source_id);
        assert_eq!(deserialized.target_id, dep.target_id);
        assert_eq!(deserialized.dependency_type, dep.dependency_type);
        assert_eq!(deserialized.lag_days, dep.lag_days);
    }

    fn sample_task() -> GanttTask {
        GanttTask {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            start: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap(),
            progress: 0.5,
            task_type: TaskType::Task,
            parent_id: None,
            dependencies: vec![],
            assignees: vec![],
            color: None,
            read_only: false,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_task_key_same_content_same_key() {
        let a = sample_task();
        let b = sample_task();
        assert_eq!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_progress_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.progress = 0.75;
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_name_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.name = "Renamed Task".to_string();
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_start_date_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.start = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_end_date_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.end = Utc.with_ymd_and_hms(2024, 1, 20, 0, 0, 0).unwrap();
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_task_type_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.task_type = TaskType::Milestone;
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_read_only_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.read_only = true;
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_changed_color_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.color = Some("#ff0000".to_string());
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_different_id_differs() {
        let a = sample_task();
        let mut b = sample_task();
        b.id = "task-2".to_string();
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }

    #[test]
    fn test_task_key_different_index_differs() {
        let a = sample_task();
        assert_ne!(task_key(0, &a), task_key(1, &a));
    }

    #[test]
    fn test_task_key_milestone_progress_crossing_done_differs() {
        // Milestone crossing progress >= 1.0 (outline -> filled) must change
        // the key even though id/name/dates/task_type are unchanged.
        let mut a = sample_task();
        a.task_type = TaskType::Milestone;
        a.progress = 0.99;
        let mut b = a.clone();
        b.progress = 1.0;
        assert_ne!(task_key(0, &a), task_key(0, &b));
    }
}
