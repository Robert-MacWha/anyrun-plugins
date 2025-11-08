use ::serde_json::from_str;
use chrono::{Duration, Local};
use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Frame {
    project: String,
    tags: Vec<String>,
}

/// Parse relative time (e.g., "1h30m") and convert to absolute time (HH:MM)
pub fn parse_relative_time(relative: &str) -> Option<String> {
    let relative = relative.to_lowercase();
    let mut total_minutes = 0i64;

    let mut current_num = String::new();
    for ch in relative.chars() {
        if ch.is_ascii_digit() {
            current_num.push(ch);
        } else if ch == 'h' {
            if let Ok(hours) = current_num.parse::<i64>() {
                total_minutes += hours * 60;
            }
            current_num.clear();
        } else if ch == 'm' {
            if let Ok(minutes) = current_num.parse::<i64>() {
                total_minutes += minutes;
            }
            current_num.clear();
        }
    }

    if total_minutes == 0 {
        return None;
    }

    // Get current local time and subtract the offset
    let now = Local::now();
    let target_time = now - Duration::minutes(total_minutes);

    // Format as HH:MM for watson
    Some(target_time.format("%H:%M").to_string())
}

/// Get all watson tags
pub fn get_tags() -> Vec<String> {
    Command::new("watson")
        .arg("tags")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| {
            s.lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Get current active task information
/// Returns (project, tags, elapsed_description) if active, None otherwise
pub fn get_current_status() -> Option<(String, Vec<String>, String)> {
    let output = Command::new("watson")
        .arg("status")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())?;

    let output = output.trim();

    if output.contains("No project started") || output.is_empty() {
        return None;
    }

    // Parse status output
    // Format: "Project <project> [tag1, tag2] started <time ago> (<timestamp>)"
    // Example: "Project anyrun-watson [coding] started 3 hours ago (2025.11.06 17:00:00-0500)"

    let project: String;
    let mut tags = Vec::new();
    let elapsed: String;

    // Extract project name (between "Project " and either " [" or " started")
    let proj_start = output.find("Project ")?;
    let after_project = &output[proj_start + 8..];

    if let Some(bracket_pos) = after_project.find(" [") {
        project = after_project[..bracket_pos].to_string();

        // Extract tags (between "[" and "]")
        if let Some(tag_start) = after_project.find('[') {
            if let Some(tag_end) = after_project.find(']') {
                let tag_str = &after_project[tag_start + 1..tag_end];
                tags = tag_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    } else if let Some(started_pos) = after_project.find(" started") {
        project = after_project[..started_pos].to_string();
    } else {
        // Fallback: couldn't parse, return None
        return None;
    }

    // Extract elapsed description (between "started " and " (")
    let started_pos = output.find("started ")?;
    let after_started = &output[started_pos + 8..];
    if let Some(paren_pos) = after_started.find(" (") {
        elapsed = after_started[..paren_pos].to_string();
    } else {
        // No elapsed time found, use empty string
        elapsed = String::new();
    }

    Some((project, tags, elapsed))
}

/// Represents a project+tags combination from history
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectTagCombo {
    pub project: String,
    pub tags: Vec<String>,
}

impl ProjectTagCombo {
    pub fn new(project: String, tags: Vec<String>) -> Self {
        Self { project, tags }
    }
}

/// Get unique project+tag combinations from watson history
/// Uses recent frames to build a list of frequently used combinations
pub fn get_project_tag_combinations() -> Vec<ProjectTagCombo> {
    let output = Command::new("watson")
        .arg("log")
        .arg("--json")
        .arg("-a") // Get all frames
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    if let Some(json_str) = output {
        if let Ok(frames) = from_str::<Vec<Frame>>(&json_str) {
            let mut seen = HashSet::new();
            let mut combinations = Vec::new();

            // Iterate in reverse to get most recent first
            for frame in frames.into_iter().rev() {
                let combo = ProjectTagCombo::new(frame.project, frame.tags);

                // Only add unique combinations
                if seen.insert(combo.clone()) {
                    combinations.push(combo);
                }

                // Limit to reasonable number to avoid performance issues
                if combinations.len() >= 50 {
                    break;
                }
            }

            return combinations;
        }
    }

    Vec::new()
}
