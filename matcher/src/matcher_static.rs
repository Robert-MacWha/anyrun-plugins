use std::process::Command;

use anyrun_plugin::HandleResult;

use crate::{Matcher, SimpleMatch};

pub struct StaticMatcher {
    children: Vec<(SimpleMatch, Box<dyn Matcher>)>,
    max_results: usize,
}

impl StaticMatcher {
    pub fn new() -> Self {
        StaticMatcher {
            children: Vec::new(),
            max_results: 1000,
        }
    }

    pub fn with_child(mut self, key: SimpleMatch, matcher: Box<dyn Matcher>) -> Self {
        self.children.push((key, matcher));
        self
    }

    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
}

impl Matcher for StaticMatcher {
    fn get_matches(&self, mut input: Vec<&str>) -> Vec<SimpleMatch> {
        let text;
        if input.is_empty() {
            text = "";
        } else {
            text = input.remove(0);
        }

        // If any child matches perfectly, return its matches
        for (key, matcher) in &self.children {
            if key.title.to_lowercase() == text.to_lowercase() {
                let child_matches = matcher.get_matches(input);
                if !child_matches.is_empty() {
                    return child_matches;
                }
                return vec![key.clone()];
            }
        }

        // Otherwise, match the child keys that contain the text
        let mut matches = Vec::new();
        for (key, _matcher) in &self.children {
            if key.title.to_lowercase().contains(&text.to_lowercase()) {
                matches.push(key.clone());
            }
        }
        matches.truncate(self.max_results);
        matches
    }

    fn handle(&self, selection: SimpleMatch) -> HandleResult {
        let description = selection.description.clone();
        if description.starts_with("exec|") {
            let command = description.trim_start_matches("exec|");
            let output = Command::new("sh").arg("-c").arg(command).output();

            if let Err(e) = output {
                // TODO: Find a good way to report errors, perhaps bubbling up to the UI.
                eprintln!("Error executing command '{}': {}", command, e);
                return HandleResult::Refresh(false);
            }

            return HandleResult::Close;
        }

        if description.starts_with("copy|") {
            let to_copy = description.trim_start_matches("copy|");
            return HandleResult::Copy(to_copy.as_bytes().into());
        }

        HandleResult::Refresh(false)
    }
}
