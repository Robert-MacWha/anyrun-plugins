use anyrun_plugin::HandleResult;

use crate::{Matcher, SimpleMatch};

/// Basic handler that displays a preset  match.
pub struct DisplayHandler {
    title: String,
    icon: String,
    description: String,
}

impl DisplayHandler {
    pub fn new(title: &str, icon: &str, description: &str) -> Self {
        DisplayHandler {
            title: title.to_string(),
            icon: icon.to_string(),
            description: description.to_string(),
        }
    }
}

impl Matcher for DisplayHandler {
    fn get_matches(&self, _text: Vec<&str>) -> Vec<SimpleMatch> {
        return vec![SimpleMatch::new(&self.title, &self.icon, &self.description)];
    }

    fn handle(&self, _selection: SimpleMatch) -> HandleResult {
        HandleResult::Close
    }
}
