use anyrun_plugin::HandleResult;
use matcher::{Matcher, SimpleMatch};

use crate::watson::{ProjectTagCombo, parse_relative_time};

/// Matcher for creating new watson projects with optional tags.
pub struct ProjectMatcher {
    known_projects: Vec<String>,
    known_tags: Vec<String>,
    combinations: Vec<ProjectTagCombo>,
}

impl ProjectMatcher {
    pub fn new(
        known_projects: Vec<String>,
        known_tags: Vec<String>,
        combinations: Vec<ProjectTagCombo>,
    ) -> Self {
        Self {
            known_projects,
            known_tags,
            combinations,
        }
    }
}

/// Matcher for adding new tags to existing watson projects.
pub struct TagMatcher {
    project: String,
    known_tags: Vec<String>,
}

pub struct TimeMatcher {
    project: String,
}

impl Matcher for ProjectMatcher {
    fn get_matches(&self, mut input: Vec<&str>) -> Vec<SimpleMatch> {
        //? If there are at least two inputs, then the first is a project
        //? name and the second is a tag. Delegate to TagMatcher.
        if input.len() >= 2 {
            let project = input.remove(0).to_string();
            let mut matches = Vec::new();
            matches.extend(
                TagMatcher {
                    project: project.clone(),
                    known_tags: self.known_tags.clone(),
                }
                .get_matches(input.clone()),
            );
            matches.extend(
                TimeMatcher {
                    project: project.clone(),
                }
                .get_matches(input),
            );

            return matches;
        }

        let text;
        if input.is_empty() {
            text = "";
        } else {
            text = input.remove(0);
        }

        let mut matches = Vec::new();

        // Add a wildcard match for new projects
        let title = format!("{} {}", text, "");
        let command = format!("exec|watson start {}", text);
        matches.push(SimpleMatch::new(&title, "", &command));

        // Add matches for known projects
        for project in &self.known_projects {
            if project.contains(text) {
                let title = format!("{} {}", project, "");
                let command = format!("exec|watson start {}", project);
                matches.push(SimpleMatch::new(&title, "", &command));
            }
        }

        // Add matches for known combinations of projects and tags
        for combo in &self.combinations {
            let project = &combo.project;
            let tags = &combo.tags;
            if project.contains(text) {
                let tags_str = tags.iter().map(|t| format!("+{} ", t)).collect::<String>();
                let title = format!("{} {}", project, tags_str);
                let command = format!("exec|watson start {} {}", project, tags_str);
                matches.push(SimpleMatch::new(&title, "", &command));
            }
        }

        return matches;
    }

    fn handle(&self, _selection: SimpleMatch) -> HandleResult {
        HandleResult::Refresh(false)
    }
}

impl TagMatcher {
    fn get_matches(&self, mut input: Vec<&str>) -> Vec<SimpleMatch> {
        let current_text = input.pop().unwrap_or("");
        let text = input.join(" ").trim().to_string();

        let mut matches = Vec::new();
        if !current_text.starts_with("+") {
            return matches;
        }

        // Add a wildcard match for new tags
        //? Split to deal with case where text is empty and we get double spaces
        let title = format!("{} {} {}", self.project, text, current_text)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let command = format!("exec|watson start {} {}", self.project, current_text);
        matches.push(SimpleMatch::new(&title, "", &command));

        // Add matches for known tags
        for tag in &self.known_tags {
            let tag = format!("+{}", tag);
            if tag.contains(current_text) {
                //? Split to deal with case where text is empty and we get double spaces
                let title = format!("{} {} {}", self.project, text, tag)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                let command = format!("exec|watson start {} {} {}", self.project, text, tag)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                matches.push(SimpleMatch::new(&title, "", &command));
            }
        }

        return matches;
    }
}

impl TimeMatcher {
    fn get_matches(&self, mut input: Vec<&str>) -> Vec<SimpleMatch> {
        let current_text = input.pop().unwrap_or("");
        let text = input.join(" ");

        let mut matches = Vec::new();
        if !current_text.starts_with("-") {
            return matches;
        }

        // Parse time from current_text
        let time_str = current_text.trim_start_matches("-").trim();
        let time = parse_relative_time(time_str);
        let Some(time) = time else {
            return matches;
        };

        let title = format!("{} {} {}", self.project, text, current_text);
        let command = format!("exec|watson start {} {} --at {}", self.project, text, time);
        matches.push(SimpleMatch::new(&title, "", &command));

        return matches;
    }
}
