use std::fmt::{Display, Formatter};

use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub title: String,
    pub created_at: NaiveDate,
    pub completed_at: Option<NaiveDate>,
}

impl Todo {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();
        if !s.starts_with("- [") {
            return None;
        }

        // find title start and end
        let after_bracket = s.find("] ")? + 2;
        let paren_idx = s.find(" (created at: ")?;
        let title = s[after_bracket..paren_idx].trim().to_string();

        // extract timestamps section
        let times = &s[paren_idx + " (created at: ".len()..s.len() - 1];
        let mut parts = times.split(", completed at: ");
        let created_str = parts.next()?.trim();
        let completed_str = parts.next()?.trim();

        let created_at = NaiveDate::parse_from_str(created_str, "%Y-%m-%d").ok()?;
        let completed_at = if completed_str == "None" {
            None
        } else {
            Some(NaiveDate::parse_from_str(completed_str, "%Y-%m-%d").ok()?)
        };

        Some(Todo {
            title,
            created_at,
            completed_at,
        })
    }
}

impl Display for Todo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let completed = self.completed_at.is_some();
        let status = if completed { "x" } else { " " };
        let completed_at = self
            .completed_at
            .map_or("None".to_string(), |dt| dt.to_string());

        write!(
            f,
            "- [{}] {} (created at: {}, completed at: {})",
            status, self.title, self.created_at, completed_at
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_todo() {
        let todo = Todo {
            title: "Buy milk".into(),
            created_at: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            completed_at: None,
        };

        let todo_str = todo.to_string();
        assert_eq!(
            todo_str,
            "- [ ] Buy milk (created at: 2024-06-01, completed at: None)"
        );

        let parsed_todo = Todo::from_str(&todo_str).unwrap();
        assert_eq!(todo, parsed_todo);
    }

    #[test]
    fn test_todo_completed() {
        let todo = Todo {
            title: "Buy milk".into(),
            created_at: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            completed_at: Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
        };

        let todo_str = todo.to_string();
        assert_eq!(
            todo_str,
            "- [x] Buy milk (created at: 2024-06-01, completed at: 2025-06-01)"
        );

        let parsed_todo = Todo::from_str(&todo_str).unwrap();
        assert_eq!(todo, parsed_todo);
    }
}
